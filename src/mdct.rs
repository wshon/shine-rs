//! MDCT (Modified Discrete Cosine Transform) implementation
//!
//! This module implements the MDCT analysis for MP3 encoding, including
//! the calculation of MDCT coefficients and aliasing reduction butterfly.
//! The implementation strictly follows the shine reference implementation
//! in ref/shine/src/lib/l3mdct.c

use crate::types::{ShineGlobalConfig, GRANULE_SIZE, SBLIMIT};
use std::f64::consts::PI;
use lazy_static::lazy_static;

/// PI/36 constant for MDCT calculations (matches shine PI36)
const PI36: f64 = PI / 36.0;

/// PI/72 constant for MDCT calculations (matches shine PI72)
const PI72: f64 = PI / 72.0;

/// Aliasing reduction coefficients (matches shine's MDCT_CA and MDCT_CS macros)
/// These are table B.9 coefficients for aliasing reduction from the ISO standard

/// MDCT_CA macro: coef / sqrt(1.0 + (coef * coef)) * 0x7fffffff
fn mdct_ca(coef: f64) -> i32 {
    (coef / (1.0 + coef * coef).sqrt() * 0x7fffffff as f64) as i32
}

/// MDCT_CS macro: 1.0 / sqrt(1.0 + (coef * coef)) * 0x7fffffff  
fn mdct_cs(coef: f64) -> i32 {
    (1.0 / (1.0 + coef * coef).sqrt() * 0x7fffffff as f64) as i32
}

lazy_static! {
    /// Aliasing reduction CA coefficients (matches shine MDCT_CA0-7)
    static ref MDCT_CA0: i32 = mdct_ca(-0.6);
    static ref MDCT_CA1: i32 = mdct_ca(-0.535);
    static ref MDCT_CA2: i32 = mdct_ca(-0.33);
    static ref MDCT_CA3: i32 = mdct_ca(-0.185);
    static ref MDCT_CA4: i32 = mdct_ca(-0.095);
    static ref MDCT_CA5: i32 = mdct_ca(-0.041);
    static ref MDCT_CA6: i32 = mdct_ca(-0.0142);
    static ref MDCT_CA7: i32 = mdct_ca(-0.0037);
    
    /// Aliasing reduction CS coefficients (matches shine MDCT_CS0-7)
    static ref MDCT_CS0: i32 = mdct_cs(-0.6);
    static ref MDCT_CS1: i32 = mdct_cs(-0.535);
    static ref MDCT_CS2: i32 = mdct_cs(-0.33);
    static ref MDCT_CS3: i32 = mdct_cs(-0.185);
    static ref MDCT_CS4: i32 = mdct_cs(-0.095);
    static ref MDCT_CS5: i32 = mdct_cs(-0.041);
    static ref MDCT_CS6: i32 = mdct_cs(-0.0142);
    static ref MDCT_CS7: i32 = mdct_cs(-0.0037);
}
/// Multiplication macros matching shine's mult_noarch_gcc.h
/// These implement fixed-point arithmetic operations

/// Basic multiplication with 32-bit right shift (matches shine mul)
#[inline]
fn mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 32) as i32
}

/// Initialize multiplication operation (matches shine mul0 macro)
#[inline]
fn mul0(a: i32, b: i32) -> i32 {
    mul(a, b)
}

/// Multiply and add operation (matches shine muladd macro)
#[inline]
fn muladd(acc: i32, a: i32, b: i32) -> i32 {
    acc + mul(a, b)
}

/// Finalize multiplication (matches shine mulz macro - no-op)
#[inline]
fn mulz(value: i32) -> i32 {
    value
}

/// Complex multiplication (matches shine cmuls macro exactly)
/// Performs complex multiplication with aliasing reduction coefficients
#[inline]
fn cmuls(are: i32, aim: i32, bre: i32, bim: i32) -> (i32, i32) {
    let tre = ((are as i64 * bre as i64 - aim as i64 * bim as i64) >> 31) as i32;
    let dim = ((are as i64 * bim as i64 + aim as i64 * bre as i64) >> 31) as i32;
    (tre, dim)
}

/// Initialize MDCT coefficients
/// Corresponds to shine_mdct_initialise() in l3mdct.c
///
/// Prepares the MDCT coefficients by combining window and MDCT coefficients
/// into a single table, scaled and converted to fixed point.
pub fn shine_mdct_initialise(config: &mut ShineGlobalConfig) {
    // Prepare the MDCT coefficients (matches shine implementation exactly)
    for m in (0..18).rev() {  // m from 17 down to 0 (matches shine: for (m = 18; m--;))
        for k in (0..36).rev() {  // k from 35 down to 0 (matches shine: for (k = 36; k--;))
            // Combine window and MDCT coefficients into a single table
            // Scale and convert to fixed point before storing
            // (matches shine formula exactly)
            config.mdct.cos_l[m][k] = (
                (PI36 * (k as f64 + 0.5)).sin() *
                ((PI / 72.0) * (2 * k + 19) as f64 * (2 * m + 1) as f64).cos() *
                0x7fffffff as f64
            ) as i32;
        }
    }
}
/// MDCT subband analysis
/// Corresponds to shine_mdct_sub() in l3mdct.c
///
/// Performs the complete MDCT analysis including:
/// 1. Polyphase filtering to generate subband samples
/// 2. MDCT transformation of subband samples to frequency domain
/// 3. Aliasing reduction butterfly operations
pub fn shine_mdct_sub(config: &mut ShineGlobalConfig, stride: i32) {
    #[cfg(any(debug_assertions, feature = "diagnostics"))]
    let frame_num = crate::get_current_frame_number();
    
    let mut mdct_in = [0i32; 36];
    
    // Process each channel (matches shine: for (ch = config->wave.channels; ch--;))
    for ch in (0..config.wave.channels).rev() {
        let ch_idx = ch as usize;
        
        // Process each granule (matches shine: for (gr = 0; gr < config->mpeg.granules_per_frame; gr++))
        for gr in 0..config.mpeg.granules_per_frame {
            let gr_idx = gr as usize;
            
            // Polyphase filtering (matches shine implementation exactly)
            // for (k = 0; k < 18; k += 2)
            for k in (0..18).step_by(2) {
                // Create a fresh buffer reference for each k iteration
                // This is critical - we need to track the buffer pointer correctly
                let buffer_slice = unsafe { 
                    std::slice::from_raw_parts(config.buffer[ch_idx], GRANULE_SIZE)
                };
                let mut buffer_ref = buffer_slice;
                
                // First subband filtering call - directly write to l3_sb_sample
                // shine_window_filter_subband(&config->buffer[ch], &config->l3_sb_sample[ch][gr + 1][k][0], ch, config, stride);
                crate::subband::shine_window_filter_subband(
                    &mut buffer_ref,
                    &mut config.l3_sb_sample[ch_idx][gr_idx + 1][k],
                    ch_idx,
                    &mut config.subband,
                    stride as usize
                );
                
                // Second subband filtering call - directly write to l3_sb_sample
                // CRITICAL: Use the updated buffer_ref from the first call
                // shine_window_filter_subband(&config->buffer[ch], &config->l3_sb_sample[ch][gr + 1][k + 1][0], ch, config, stride);
                crate::subband::shine_window_filter_subband(
                    &mut buffer_ref,
                    &mut config.l3_sb_sample[ch_idx][gr_idx + 1][k + 1],
                    ch_idx,
                    &mut config.subband,
                    stride as usize
                );
                
                // Update the main buffer pointer to reflect the consumed samples
                // This is critical - we need to advance the buffer pointer for the next k iteration
                // In shine, the buffer pointer is automatically advanced by the subband filter calls
                config.buffer[ch_idx] = buffer_ref.as_ptr() as *mut i16;
                
                // Compensate for inversion in the analysis filter
                // (every odd index of band AND k) - matches shine exactly
                for band in (1..32).step_by(2) {  // band = 1, 3, 5, ..., 31
                    config.l3_sb_sample[ch_idx][gr_idx + 1][k + 1][band] *= -1;
                }
            }
            
            // Perform IMDCT of 18 previous + 18 current subband samples
            // (matches shine: for (band = 0; band < 32; band++))
            for band in 0..32 {
                // Prepare input for MDCT (matches shine exactly)
                for k in (0..18).rev() {  // k from 17 down to 0 (matches shine: for (k = 18; k--;))
                    mdct_in[k] = config.l3_sb_sample[ch_idx][gr_idx][k][band];
                    mdct_in[k + 18] = config.l3_sb_sample[ch_idx][gr_idx + 1][k][band];
                }
                

                
                // Calculation of the MDCT
                // In the case of long blocks (block_type 0,1,3) there are
                // 36 coefficients in the time domain and 18 in the frequency domain
                for k in (0..18).rev() {  // k from 17 down to 0 (matches shine: for (k = 18; k--;))
                    let mut vm: i32;
                    
                    // Start with the last coefficient (matches shine exactly)
                    vm = mul0(mdct_in[35], config.mdct.cos_l[k][35]);
                    
                    // Process remaining coefficients in groups of 7 (matches shine's unrolled loop exactly)
                    let mut j = 35;
                    while j > 0 {
                        if j >= 7 {
                            vm = muladd(vm, mdct_in[j - 1], config.mdct.cos_l[k][j - 1]);
                            vm = muladd(vm, mdct_in[j - 2], config.mdct.cos_l[k][j - 2]);
                            vm = muladd(vm, mdct_in[j - 3], config.mdct.cos_l[k][j - 3]);
                            vm = muladd(vm, mdct_in[j - 4], config.mdct.cos_l[k][j - 4]);
                            vm = muladd(vm, mdct_in[j - 5], config.mdct.cos_l[k][j - 5]);
                            vm = muladd(vm, mdct_in[j - 6], config.mdct.cos_l[k][j - 6]);
                            vm = muladd(vm, mdct_in[j - 7], config.mdct.cos_l[k][j - 7]);
                            j -= 7;
                        } else {
                            break;
                        }
                    }
                    
                    vm = mulz(vm);
                    
                    // Store result in mdct_freq array
                    // Note: shine accesses mdct_freq as mdct_enc[band][k] where mdct_enc = (int32_t(*)[18])config->mdct_freq[ch][gr]
                    // This means mdct_freq[ch][gr][band*18 + k]
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + k] = vm;
                    
                    // Print key MDCT coefficients for verification (debug mode only)
                    #[cfg(any(debug_assertions, feature = "diagnostics"))]
                    {
                        use log::debug;
                        let debug_frames = std::env::var("RUST_MP3_DEBUG_FRAMES")
                            .unwrap_or_else(|_| "6".to_string())
                            .parse::<i32>()
                            .unwrap_or(6);
                        if frame_num <= debug_frames && ch == 0 && gr == 0 && band == 0 && k >= 15 {
                            debug!("[Frame {}] MDCT[{}][{}][{}][{}] = {}", 
                                     frame_num, ch, gr, band, k, vm);
                        }
                        // Record MDCT coefficient for test collection
                        crate::test_data::record_mdct_coeff(k, vm);
                    }
                }
                
                // Perform aliasing reduction butterfly (matches shine exactly)
                if band != 0 {
                    // Apply aliasing reduction for each of the 8 coefficients
                    // (matches shine's cmuls calls exactly)
                    
                    // Get current values
                    let curr_0 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 0];
                    let prev_17 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 17];
                    let (new_curr_0, new_prev_17) = cmuls(curr_0, prev_17, *MDCT_CS0, *MDCT_CA0);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 0] = new_curr_0;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 17] = new_prev_17;
                    
                    let curr_1 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 1];
                    let prev_16 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 16];
                    let (new_curr_1, new_prev_16) = cmuls(curr_1, prev_16, *MDCT_CS1, *MDCT_CA1);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 1] = new_curr_1;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 16] = new_prev_16;
                    
                    let curr_2 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 2];
                    let prev_15 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 15];
                    let (new_curr_2, new_prev_15) = cmuls(curr_2, prev_15, *MDCT_CS2, *MDCT_CA2);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 2] = new_curr_2;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 15] = new_prev_15;
                    
                    let curr_3 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 3];
                    let prev_14 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 14];
                    let (new_curr_3, new_prev_14) = cmuls(curr_3, prev_14, *MDCT_CS3, *MDCT_CA3);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 3] = new_curr_3;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 14] = new_prev_14;
                    
                    let curr_4 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 4];
                    let prev_13 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 13];
                    let (new_curr_4, new_prev_13) = cmuls(curr_4, prev_13, *MDCT_CS4, *MDCT_CA4);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 4] = new_curr_4;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 13] = new_prev_13;
                    
                    let curr_5 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 5];
                    let prev_12 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 12];
                    let (new_curr_5, new_prev_12) = cmuls(curr_5, prev_12, *MDCT_CS5, *MDCT_CA5);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 5] = new_curr_5;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 12] = new_prev_12;
                    
                    let curr_6 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 6];
                    let prev_11 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 11];
                    let (new_curr_6, new_prev_11) = cmuls(curr_6, prev_11, *MDCT_CS6, *MDCT_CA6);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 6] = new_curr_6;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 11] = new_prev_11;
                    
                    let curr_7 = config.mdct_freq[ch_idx][gr_idx][band * 18 + 7];
                    let prev_10 = config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 10];
                    let (new_curr_7, new_prev_10) = cmuls(curr_7, prev_10, *MDCT_CS7, *MDCT_CA7);
                    config.mdct_freq[ch_idx][gr_idx][band * 18 + 7] = new_curr_7;
                    config.mdct_freq[ch_idx][gr_idx][(band - 1) * 18 + 10] = new_prev_10;
                }
            }
        }
        
        // Save latest granule's subband samples to be used in the next mdct call
        // (matches shine: memcpy(config->l3_sb_sample[ch][0], config->l3_sb_sample[ch][config->mpeg.granules_per_frame], sizeof(config->l3_sb_sample[0][0])))
        for k in 0..18 {
            for band in 0..SBLIMIT {
                config.l3_sb_sample[ch_idx][0][k][band] = 
                    config.l3_sb_sample[ch_idx][config.mpeg.granules_per_frame as usize][k][band];
            }
        }
        
        // Debug: Print saved data for verification (debug mode only)
        #[cfg(any(debug_assertions, feature = "diagnostics"))]
        {
            use log::debug;
            let debug_frames = std::env::var("RUST_MP3_DEBUG_FRAMES")
                .unwrap_or_else(|_| "6".to_string())
                .parse::<i32>()
                .unwrap_or(6);
            if frame_num <= debug_frames && ch == 0 {
                debug!("[Frame {}] Saved l3_sb_sample[{}][0][0][0] = {}", 
                         frame_num, ch, config.l3_sb_sample[ch_idx][0][0][0]);
            }
            // Record l3_sb_sample for test collection
            crate::test_data::record_sb_sample(ch as usize, config.l3_sb_sample[ch_idx][0][0][0]);
        }
    }
}