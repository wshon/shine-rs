//! Quantization and rate control for MP3 encoding
//!
//! This module implements the quantization loop that controls the
//! trade-off between audio quality and bitrate by adjusting quantization
//! step sizes and managing the bit reservoir.
//!
//! The implementation strictly follows the shine reference implementation
//! in ref/shine/src/lib/l3loop.c

use crate::types::{ShineGlobalConfig, GRANULE_SIZE, GrInfo, ShinePsyXmin};
use crate::tables::{SHINE_SCALE_FACT_BAND_INDEX, SHINE_SLEN1_TAB, SHINE_SLEN2_TAB};
use crate::huffman::SHINE_HUFFMAN_TABLE;
use std::f64::consts::LN_2;

/// Constants from shine (matches l3loop.c exactly)
#[allow(dead_code)] // May be used in future implementations
const CBLIMIT: usize = 21;
const SFB_LMAX: usize = 22;
const EN_TOT_KRIT: i32 = 10;
const EN_DIF_KRIT: i32 = 100;
const EN_SCFSI_BAND_KRIT: i32 = 10;
const XM_SCFSI_BAND_KRIT: i32 = 10;
/// Multiplication macros matching shine's mult_noarch_gcc.h
/// These implement fixed-point arithmetic operations

/// Multiply with rounding and 31-bit right shift (matches shine mulsr)
#[inline]
pub fn mulsr(a: i32, b: i32) -> i32 {
    (((a as i64 * b as i64) + 0x40000000i64) >> 31) as i32
}

/// Multiply with rounding and 32-bit right shift (matches shine mulr)
#[inline]
pub fn mulr(a: i32, b: i32) -> i32 {
    (((a as i64 * b as i64) + 0x80000000i64) >> 32) as i32
}

/// Absolute value function (matches shine labs)
#[inline]
pub fn labs(x: i32) -> i32 {
    x.abs()
}

/// Inner loop: find optimal quantization step size for given scalefactors
/// Corresponds to shine_inner_loop() in l3loop.c
///
/// The code selects the best quantizerStepSize for a particular set
/// of scalefacs.
pub fn shine_inner_loop(
    ix: &mut [i32],
    max_bits: i32,
    gr: i32,
    ch: i32,
    config: &mut ShineGlobalConfig,
) -> i32 {
    let mut bits: i32;
    let mut _c1bits: i32;
    let mut bvbits: i32;

    // Following shine's logic exactly:
    // if (max_bits < 0) cod_info->quantizerStepSize--;
    if max_bits < 0 {
        let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
        cod_info.quantizer_step_size -= 1;
    }

    // Main quantization loop - following shine's do-while structure exactly
    loop {
        // while (quantize(ix, ++cod_info->quantizerStepSize, config) > 8192)
        //   ; /* within table range? */
        let mut quantizer_step_size = {
            let cod_info = &config.side_info.gr[gr as usize].ch[ch as usize].tt;
            cod_info.quantizer_step_size
        };

        loop {
            quantizer_step_size += 1;
            if quantize(ix, quantizer_step_size, config) <= 8192 {
                break;
            }
        }

        // Update quantizer step size
        {
            let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
            cod_info.quantizer_step_size = quantizer_step_size;
        }

        // Process with current step size
        {
            let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
            calc_runlen(ix, cod_info); // rzero,count1,big_values
            bits = count1_bitcount(ix, cod_info); // count1_table selection
            _c1bits = bits;
        }

        // Subdivide and select tables - avoid borrowing conflicts by separating operations
        {
            let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
            calc_runlen(ix, cod_info); // rzero,count1,big_values
            bits = count1_bitcount(ix, cod_info); // count1_table selection
            _c1bits = bits;
        }

        // Create a temporary copy for subdivide to avoid borrowing conflicts
        {
            let mut cod_info_copy = config.side_info.gr[gr as usize].ch[ch as usize].tt.clone();
            subdivide(&mut cod_info_copy, config); // bigvalues sfb division
            config.side_info.gr[gr as usize].ch[ch as usize].tt = cod_info_copy;
        }

        {
            let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
            bigv_tab_select(ix, cod_info); // codebook selection
            bvbits = bigv_bitcount(ix, cod_info); // bit count
        }

        bits += bvbits;

        if bits <= max_bits {
            break;
        }
    }

    bits
}
/// Outer loop: controls masking conditions and computes best scalefac and global gain
/// Corresponds to shine_outer_loop() in l3loop.c
///
/// The outer iteration loop controls the masking conditions
/// of all scalefactorbands. It computes the best scalefac and
/// global gain. This module calls the inner iteration loop.
pub fn shine_outer_loop(
    max_bits: i32,
    _l3_xmin: &mut ShinePsyXmin, // the allowed distortion of the scalefactor
    ix: &mut [i32], // vector of quantized values ix(0..575)
    gr: i32,
    ch: i32,
    config: &mut ShineGlobalConfig,
) -> i32 {
    let bits: i32;
    let huff_bits: i32;

    // Extract quantizer step size to avoid borrowing conflicts
    let quantizer_step_size = {
        let mut cod_info = config.side_info.gr[gr as usize].ch[ch as usize].tt.clone();
        let result = bin_search_step_size(max_bits, ix, &mut cod_info, config);
        config.side_info.gr[gr as usize].ch[ch as usize].tt = cod_info;
        result
    };

    let part2_length = part2_length(gr, ch, config) as u32;
    huff_bits = max_bits - part2_length as i32;

    // Update cod_info with extracted values
    {
        let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
        cod_info.quantizer_step_size = quantizer_step_size;
        cod_info.part2_length = part2_length;
    }

    bits = shine_inner_loop(ix, huff_bits, gr, ch, config);

    // Update final values
    let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
    cod_info.part2_3_length = cod_info.part2_length + bits as u32;

    cod_info.part2_3_length as i32
}

/// Main iteration loop for encoding
/// Corresponds to shine_iteration_loop() in l3loop.c
pub fn shine_iteration_loop(config: &mut ShineGlobalConfig) {
    #[cfg(any(debug_assertions, feature = "diagnostics"))]
    let frame_num = crate::get_current_frame_number();

    let mut l3_xmin = ShinePsyXmin::default();
    let mut ix: *mut i32;
    
    // Store xrmax for the first channel and granule for test data collection
    #[cfg(feature = "diagnostics")]
    let mut saved_xrmax = 0i32;

    // Process each channel and granule
    for ch in (0..config.wave.channels).rev() {
        for gr in 0..config.mpeg.granules_per_frame {
            // setup pointers
            ix = config.l3_enc[ch as usize][gr as usize].as_mut_ptr();
            config.l3loop.xr = config.mdct_freq[ch as usize][gr as usize].as_ptr() as *mut i32;

            // Precalculate the square, abs, and maximum, for use later on.
            config.l3loop.xrmax = 0;
            for i in (0..GRANULE_SIZE).rev() {
                let xr_val = unsafe { *config.l3loop.xr.add(i) };
                config.l3loop.xrsq[i] = mulsr(xr_val, xr_val);
                config.l3loop.xrabs[i] = labs(xr_val);
                if config.l3loop.xrabs[i] > config.l3loop.xrmax {
                    config.l3loop.xrmax = config.l3loop.xrabs[i];
                }
            }

            // Set sfb_lmax and calculate xmin
            {
                let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
                cod_info.sfb_lmax = (SFB_LMAX - 1) as u32; // gr_deco
                calc_xmin(&config.ratio, cod_info, &mut l3_xmin, gr, ch);
            }

            if config.mpeg.version == 3 { // MPEG_I = 3
                // MPEG_I - handle borrowing carefully by cloning l3_xmin temporarily
                calc_scfsi(&mut l3_xmin, ch, gr, config);
            }

            // calculation of number of available bit( per granule )
            let pe_value = config.pe[ch as usize][gr as usize].clone();
            let max_bits = crate::reservoir::shine_max_reservoir_bits(&pe_value, &config);

            // Debug logging for algorithm verification
            #[cfg(any(debug_assertions, feature = "diagnostics"))]
            {
                use log::debug;
                let debug_frames = std::env::var("RUST_MP3_DEBUG_FRAMES")
                    .unwrap_or_else(|_| "6".to_string())
                    .parse::<i32>()
                    .unwrap_or(6);
                if frame_num <= debug_frames && ch == 0 && gr == 0 {
                    debug!("[Frame {}] xrmax={}, max_bits={}", frame_num, config.l3loop.xrmax, max_bits);
                    // Save xrmax for the first channel and granule
                    #[cfg(feature = "diagnostics")]
                    {
                        saved_xrmax = config.l3loop.xrmax;
                    }
                }
            }

            // reset of iteration variables
            for i in 0..config.scalefactor.l[gr as usize][ch as usize].len() {
                config.scalefactor.l[gr as usize][ch as usize][i] = 0;
            }
            for i in 0..config.scalefactor.s[gr as usize][ch as usize].len() {
                for j in 0..config.scalefactor.s[gr as usize][ch as usize][i].len() {
                    config.scalefactor.s[gr as usize][ch as usize][i][j] = 0;
                }
            }

            // Reset cod_info values
            {
                let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
                for i in 0..4 {
                    cod_info.slen[i] = 0;
                }

                cod_info.part2_3_length = 0;
                cod_info.big_values = 0;
                cod_info.count1 = 0;
                cod_info.scalefac_compress = 0;
                cod_info.table_select[0] = 0;
                cod_info.table_select[1] = 0;
                cod_info.table_select[2] = 0;
                cod_info.region0_count = 0;
                cod_info.region1_count = 0;
                cod_info.part2_length = 0;
                cod_info.preflag = 0;
                cod_info.scalefac_scale = 0;
                cod_info.count1table_select = 0;
            }

            // all spectral values zero ?
            let _part2_3_length = if config.l3loop.xrmax != 0 {
                let ix_slice = unsafe { std::slice::from_raw_parts_mut(ix, GRANULE_SIZE) };
                let length = shine_outer_loop(
                    max_bits,
                    &mut l3_xmin,
                    ix_slice.try_into().unwrap(),
                    gr,
                    ch,
                    config,
                ) as u32;

                // Update part2_3_length after outer loop
                let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
                cod_info.part2_3_length = length;
                length
            } else {
                0u32
            };

            // Adjust reservoir and set global gain
            {
                let quantizer_step_size = {
                    let cod_info = &config.side_info.gr[gr as usize].ch[ch as usize].tt;
                    cod_info.quantizer_step_size
                };
                
                // Call reservoir adjust first (matches Shine order)
                let cod_info_copy = config.side_info.gr[gr as usize].ch[ch as usize].tt.clone();
                crate::reservoir::shine_resv_adjust(&cod_info_copy, config);
                
                // Set global gain AFTER reservoir adjustment (matches Shine)
                let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
                cod_info.global_gain = (quantizer_step_size + 210) as u32;

                // Debug output for verification (but don't record data here)
                #[cfg(feature = "diagnostics")]
                {
                    use log::debug;
                    let debug_frames = std::env::var("RUST_MP3_DEBUG_FRAMES")
                        .unwrap_or_else(|_| "6".to_string())
                        .parse::<i32>()
                        .unwrap_or(6);
                    if frame_num <= debug_frames && ch == 0 && gr == 0 {
                        debug!("[Frame {}] part2_3_length={}, quantizer_step_size={}, global_gain={}",
                                 frame_num, cod_info.part2_3_length, quantizer_step_size, cod_info.global_gain);
                        println!("[RUST DEBUG] Intermediate: xrmax={}, max_bits={}, part2_3_length={}, quantizer_step_size={}, global_gain={}",
                                 config.l3loop.xrmax, max_bits, cod_info.part2_3_length, quantizer_step_size, cod_info.global_gain);
                    }
                }

                // Suppress unused variable warning in release mode
                #[cfg(not(any(debug_assertions, feature = "diagnostics")))]
                let _ = part2_3_length;
            }
        } // for gr
    } // for ch

    crate::reservoir::shine_resv_frame_end(config);
    
    // Record quantization data AFTER shine_resv_frame_end (matches Shine final output)
    #[cfg(feature = "diagnostics")]
    {
        let debug_frames = std::env::var("RUST_MP3_DEBUG_FRAMES")
            .unwrap_or_else(|_| "6".to_string())
            .parse::<i32>()
            .unwrap_or(6);
        
        if frame_num <= debug_frames {
            // Record data for the first channel and granule (ch=0, gr=0) after all adjustments
            let cod_info = &config.side_info.gr[0].ch[0].tt;
            let max_bits = crate::reservoir::shine_max_reservoir_bits(&config.pe[0][0], &config);
            
            crate::diagnostics_data::record_quant_data(
                saved_xrmax,  // Use the saved xrmax from ch=0, gr=0
                max_bits,
                cod_info.part2_3_length,  // Final value after all reservoir adjustments
                cod_info.quantizer_step_size,
                cod_info.global_gain
            );
            
            println!("[RUST DEBUG] Final Recording: xrmax={}, max_bits={}, part2_3_length={}, quantizer_step_size={}, global_gain={}",
                     saved_xrmax, max_bits, cod_info.part2_3_length, cod_info.quantizer_step_size, cod_info.global_gain);
        }
    }
}
/// Calculate scale factor selection information (scfsi)
/// Corresponds to calc_scfsi() in l3loop.c
fn calc_scfsi(
    l3_xmin: &mut ShinePsyXmin,
    ch: i32,
    gr: i32,
    config: &mut ShineGlobalConfig,
) {
    let l3_side = &mut config.side_info;
    // This is the scfsi_band table from 2.4.2.7 of the IS
    const SCFSI_BAND_LONG: [i32; 5] = [0, 6, 11, 16, 21];

    let mut condition = 0;
    let mut temp: i32;

    let samplerate_index = match config.wave.samplerate {
        44100 => 0, 48000 => 1, 32000 => 2, 22050 => 3, 24000 => 4,
        16000 => 5, 11025 => 6, 12000 => 7, 8000 => 8, _ => 0,
    };

    let scalefac_band_long = &SHINE_SCALE_FACT_BAND_INDEX[samplerate_index];

    config.l3loop.xrmaxl[gr as usize] = config.l3loop.xrmax;

    // the total energy of the granule
    temp = 0;
    for i in (0..GRANULE_SIZE).rev() {
        temp += config.l3loop.xrsq[i] >> 10; // a bit of scaling to avoid overflow
    }
    if temp != 0 {
        config.l3loop.en_tot[gr as usize] =
            ((temp as f64 * 4.768371584e-7).ln() / LN_2) as i32; // 1024 / 0x7fffffff
    } else {
        config.l3loop.en_tot[gr as usize] = 0;
    }

    // the energy of each scalefactor band, en
    // the allowed distortion of each scalefactor band, xm
    for sfb in (0..21).rev() {
        let start = scalefac_band_long[sfb] as usize;
        let end = scalefac_band_long[sfb + 1] as usize;

        temp = 0;
        for i in start..end {
            if i < GRANULE_SIZE {
                temp += config.l3loop.xrsq[i] >> 10;
            }
        }
        if temp != 0 {
            config.l3loop.en[gr as usize][sfb] =
                ((temp as f64 * 4.768371584e-7).ln() / LN_2) as i32;
        } else {
            config.l3loop.en[gr as usize][sfb] = 0;
        }

        if l3_xmin.l[gr as usize][ch as usize][sfb] != 0.0 {
            config.l3loop.xm[gr as usize][sfb] =
                (l3_xmin.l[gr as usize][ch as usize][sfb].ln() / LN_2) as i32;
        } else {
            config.l3loop.xm[gr as usize][sfb] = 0;
        }
    }

    if gr == 1 {
        for gr2 in (0..2).rev() {
            // The spectral values are not all zero
            if config.l3loop.xrmaxl[gr2] != 0 {
                condition += 1;
            }
            condition += 1;
        }
        if (config.l3loop.en_tot[0] - config.l3loop.en_tot[1]).abs() < EN_TOT_KRIT {
            condition += 1;
        }
        let mut tp = 0;
        for sfb in (0..21).rev() {
            tp += (config.l3loop.en[0][sfb] - config.l3loop.en[1][sfb]).abs();
        }
        if tp < EN_DIF_KRIT {
            condition += 1;
        }

        if condition == 6 {
            for scfsi_band in 0..4 {
                let mut sum0 = 0;
                let mut sum1 = 0;
                l3_side.scfsi[ch as usize][scfsi_band] = 0;
                let start = SCFSI_BAND_LONG[scfsi_band] as usize;
                let end = SCFSI_BAND_LONG[scfsi_band + 1] as usize;
                for sfb in start..end {
                    sum0 += (config.l3loop.en[0][sfb] - config.l3loop.en[1][sfb]).abs();
                    sum1 += (config.l3loop.xm[0][sfb] - config.l3loop.xm[1][sfb]).abs();
                }

                if sum0 < EN_SCFSI_BAND_KRIT && sum1 < XM_SCFSI_BAND_KRIT {
                    l3_side.scfsi[ch as usize][scfsi_band] = 1;
                } else {
                    l3_side.scfsi[ch as usize][scfsi_band] = 0;
                }
            }
        } else {
            for scfsi_band in 0..4 {
                l3_side.scfsi[ch as usize][scfsi_band] = 0;
            }
        }
    }
}

/// Calculate part2 length (scalefactors)
/// Corresponds to part2_length() in l3loop.c
pub fn part2_length(gr: i32, ch: i32, config: &mut ShineGlobalConfig) -> i32 {
    let mut bits = 0;
    let gi = &config.side_info.gr[gr as usize].ch[ch as usize].tt;

    let slen1 = SHINE_SLEN1_TAB[gi.scalefac_compress as usize % SHINE_SLEN1_TAB.len()];
    let slen2 = SHINE_SLEN2_TAB[gi.scalefac_compress as usize % SHINE_SLEN2_TAB.len()];

    if gr == 0 || config.side_info.scfsi[ch as usize][0] == 0 {
        bits += 6 * slen1;
    }

    if gr == 0 || config.side_info.scfsi[ch as usize][1] == 0 {
        bits += 5 * slen1;
    }

    if gr == 0 || config.side_info.scfsi[ch as usize][2] == 0 {
        bits += 5 * slen2;
    }

    if gr == 0 || config.side_info.scfsi[ch as usize][3] == 0 {
        bits += 5 * slen2;
    }

    bits
}

/// Calculate allowed distortion for each scalefactor band
/// Corresponds to calc_xmin() in l3loop.c
fn calc_xmin(
    _ratio: &crate::types::ShinePsyRatio,
    cod_info: &mut GrInfo,
    l3_xmin: &mut ShinePsyXmin,
    gr: i32,
    ch: i32,
) {
    for sfb in (0..cod_info.sfb_lmax as usize).rev() {
        // note. xmin will always be zero with no psychoacoustic model
        l3_xmin.l[gr as usize][ch as usize][sfb] = 0.0;
    }
}

/// Initialize quantization loop tables
/// Corresponds to shine_loop_initialise() in l3loop.c
pub fn shine_loop_initialise(config: &mut ShineGlobalConfig) {
    // quantize: stepsize conversion, fourth root of 2 table.
    // The table is inverted (negative power) from the equation given
    // in the spec because it is quicker to do x*y than x/y.
    // The 0.5 is for rounding.
    for i in (0..128).rev() {
        config.l3loop.steptab[i] = (2.0_f64).powf((127 - i as i32) as f64 / 4.0);
        if (config.l3loop.steptab[i] * 2.0) > 0x7fffffff as f64 {
            config.l3loop.steptabi[i] = 0x7fffffff;
        } else {
            // The table is multiplied by 2 to give an extra bit of accuracy.
            // In quantize, the long multiply does not shift its result left one
            // bit to compensate.
            config.l3loop.steptabi[i] = (config.l3loop.steptab[i] * 2.0 + 0.5) as i32;
        }
    }

    // quantize: vector conversion, three quarter power table.
    // The 0.5 is for rounding, the .0946 comes from the spec.
    for i in (0..10000).rev() {
        config.l3loop.int2idx[i] = ((i as f64).sqrt().sqrt() * (i as f64).sqrt() - 0.0946 + 0.5) as i32;
    }
}
/// Quantize MDCT coefficients
/// Corresponds to quantize() in l3loop.c
pub fn quantize(ix: &mut [i32], stepsize: i32, config: &mut ShineGlobalConfig) -> i32 {
    let mut max = 0;
    let scalei: i32;
    let mut scale: f64;
    let mut dbl: f64;

    scalei = config.l3loop.steptabi[(stepsize + 127).clamp(0, 127) as usize]; // 2**(-stepsize/4)

    // a quick check to see if ixmax will be less than 8192
    // this speeds up the early calls to bin_search_StepSize
    if mulr(config.l3loop.xrmax, scalei) > 165140 {
        // 8192**(4/3)
        max = 16384; // no point in continuing, stepsize not big enough
    } else {
        for i in 0..GRANULE_SIZE {
            // This calculation is very sensitive. The multiply must round its
            // result or bad things happen to the quality.
            let ln = mulr(labs(unsafe { *config.l3loop.xr.add(i) }), scalei);

            if ln < 10000 {
                // ln < 10000 catches most values
                ix[i] = config.l3loop.int2idx[ln as usize]; // quick look up method
            } else {
                // outside table range so have to do it using floats
                scale = config.l3loop.steptab[(stepsize + 127).clamp(0, 127) as usize]; // 2**(-stepsize/4)
                dbl = (config.l3loop.xrabs[i] as f64) * scale * 4.656612875e-10; // 0x7fffffff
                ix[i] = (dbl.sqrt().sqrt() * dbl.sqrt()) as i32; // dbl**(3/4)
            }

            // calculate ixmax while we're here
            // note. ix cannot be negative
            if max < ix[i] {
                max = ix[i];
            }
        }
    }

    max
}

/// Calculate maximum value in range
pub fn ix_max(ix: &[i32], begin: u32, end: u32) -> i32 {
    let mut max = 0;
    let start = begin as usize;
    let end = (end as usize).min(GRANULE_SIZE);

    for i in start..end {
        if max < ix[i] {
            max = ix[i];
        }
    }
    max
}

/// Calculate run length encoding information
/// Corresponds to calc_runlen() in l3loop.c
pub fn calc_runlen(ix: &mut [i32], cod_info: &mut GrInfo) {
    let mut i = GRANULE_SIZE;
    let mut _rzero = 0;

    // Count trailing zero pairs
    while i > 1 {
        i -= 2;
        if ix[i] == 0 && ix[i + 1] == 0 {
            _rzero += 1;
        } else {
            i += 2;
            break;
        }
    }

    cod_info.count1 = 0;
    while i > 3 {
        i -= 4;
        if ix[i] <= 1 && ix[i + 1] <= 1 && ix[i + 2] <= 1 && ix[i + 3] <= 1 {
            cod_info.count1 += 1;
        } else {
            i += 4;
            break;
        }
    }

    cod_info.big_values = (i >> 1) as u32;
}

/// Count bits for count1 region
/// Corresponds to count1_bitcount() in l3loop.c
pub fn count1_bitcount(ix: &[i32], cod_info: &mut GrInfo) -> i32 {
    let mut sum0 = 0;
    let mut sum1 = 0;

    let mut i = (cod_info.big_values << 1) as usize;
    for _k in 0..cod_info.count1 {
        if i + 3 >= GRANULE_SIZE {
            break;
        }

        let v = ix[i];
        let w = ix[i + 1];
        let x = ix[i + 2];
        let y = ix[i + 3];

        let p = (v + (w << 1) + (x << 2) + (y << 3)) as usize;

        let mut signbits = 0;
        if v != 0 { signbits += 1; }
        if w != 0 { signbits += 1; }
        if x != 0 { signbits += 1; }
        if y != 0 { signbits += 1; }

        sum0 += signbits;
        sum1 += signbits;

        // Use huffman tables 32 and 33 for count1 (matches shine exactly)
        if let Some(hlen) = SHINE_HUFFMAN_TABLE[32].hlen {
            if p < hlen.len() {
                sum0 += hlen[p] as i32;
            }
        } else {
            // WARNING: This branch doesn't exist in shine - added for safety
            log::warn!("Missing hlen table for Huffman table 32");
        }

        if let Some(hlen) = SHINE_HUFFMAN_TABLE[33].hlen {
            if p < hlen.len() {
                sum1 += hlen[p] as i32;
            }
        } else {
            // WARNING: This branch doesn't exist in shine - added for safety
            log::warn!("Missing hlen table for Huffman table 33");
        }

        i += 4;
    }

    if sum0 < sum1 {
        cod_info.count1table_select = 0;
        sum0
    } else {
        cod_info.count1table_select = 1;
        sum1
    }
}

/// Subdivide big values region into regions for different Huffman tables
/// Corresponds to subdivide() in l3loop.c
pub fn subdivide(cod_info: &mut GrInfo, config: &mut ShineGlobalConfig) {
    // Subdivision table from shine (matches exactly)
    const SUBDV_TABLE: [(u32, u32); 23] = [
        (0, 0), // 0 bands
        (0, 0), // 1 bands
        (0, 0), // 2 bands
        (0, 0), // 3 bands
        (0, 0), // 4 bands
        (0, 1), // 5 bands
        (1, 1), // 6 bands
        (1, 1), // 7 bands
        (1, 2), // 8 bands
        (2, 2), // 9 bands
        (2, 3), // 10 bands
        (2, 3), // 11 bands
        (3, 4), // 12 bands
        (3, 4), // 13 bands
        (3, 4), // 14 bands
        (4, 5), // 15 bands
        (4, 5), // 16 bands
        (4, 6), // 17 bands
        (5, 6), // 18 bands
        (5, 6), // 19 bands
        (5, 7), // 20 bands
        (6, 7), // 21 bands
        (6, 7), // 22 bands
    ];

    if cod_info.big_values == 0 {
        // no big_values region
        cod_info.region0_count = 0;
        cod_info.region1_count = 0;
    } else {
        let samplerate_index = match config.wave.samplerate {
            44100 => 0, 48000 => 1, 32000 => 2, 22050 => 3, 24000 => 4,
            16000 => 5, 11025 => 6, 12000 => 7, 8000 => 8, _ => 0,
        };
        let scalefac_band_long = &SHINE_SCALE_FACT_BAND_INDEX[samplerate_index];

        let bigvalues_region = 2 * cod_info.big_values;

        // Calculate scfb_anz
        let mut scfb_anz = 0;
        while (scfb_anz < 22) && (scalefac_band_long[scfb_anz] < bigvalues_region as i32) {
            scfb_anz += 1;
        }

        let mut thiscount = SUBDV_TABLE[scfb_anz].0;
        while thiscount > 0 {
            if scalefac_band_long[thiscount as usize + 1] <= bigvalues_region as i32 {
                break;
            }
            thiscount -= 1;
        }
        cod_info.region0_count = thiscount;
        cod_info.address1 = scalefac_band_long[thiscount as usize + 1] as u32;

        let mut thiscount = SUBDV_TABLE[scfb_anz].1;
        while thiscount > 0 {
            let idx = (cod_info.region0_count + 1 + thiscount) as usize;
            if idx < 22 && scalefac_band_long[idx + 1] <= bigvalues_region as i32 {
                break;
            }
            thiscount -= 1;
        }
        cod_info.region1_count = thiscount;
        let idx = (cod_info.region0_count + 1 + thiscount) as usize;
        if idx + 1 < 22 {
            cod_info.address2 = scalefac_band_long[idx + 1] as u32;
        } else {
            cod_info.address2 = bigvalues_region;
        }

        cod_info.address3 = bigvalues_region;
    }
}

/// Select Huffman code tables for bigvalues regions
/// Corresponds to bigv_tab_select() in l3loop.c
pub fn bigv_tab_select(ix: &[i32], cod_info: &mut GrInfo) {
    cod_info.table_select[0] = 0;
    cod_info.table_select[1] = 0;
    cod_info.table_select[2] = 0;

    if cod_info.address1 > 0 {
        cod_info.table_select[0] = new_choose_table(ix, 0, cod_info.address1);
    }

    if cod_info.address2 > cod_info.address1 {
        cod_info.table_select[1] = new_choose_table(ix, cod_info.address1, cod_info.address2);
    }

    if (cod_info.big_values << 1) > cod_info.address2 {
        cod_info.table_select[2] = new_choose_table(ix, cod_info.address2, cod_info.big_values << 1);
    }
}

/// Choose the Huffman table that will encode ix[begin..end] with the fewest bits
/// Corresponds to new_choose_table() in l3loop.c
fn new_choose_table(ix: &[i32], begin: u32, end: u32) -> u32 {
    let max = ix_max(ix, begin, end);
    if max == 0 {
        return 0;
    }

    let mut choice = [0u32; 2];
    let mut sum = [0i32; 2];

    if max < 15 {
        // try tables with no linbits
        for i in (0..14).rev() {
            if let Some(table) = SHINE_HUFFMAN_TABLE.get(i) {
                if table.xlen > max as u32 {
                    choice[0] = i as u32;
                    break;
                }
            }
        }

        sum[0] = count_bit(ix, begin, end, choice[0]);

        match choice[0] {
            2 => {
                sum[1] = count_bit(ix, begin, end, 3);
                if sum[1] <= sum[0] {
                    choice[0] = 3;
                }
            }
            5 => {
                sum[1] = count_bit(ix, begin, end, 6);
                if sum[1] <= sum[0] {
                    choice[0] = 6;
                }
            }
            7 => {
                sum[1] = count_bit(ix, begin, end, 8);
                if sum[1] <= sum[0] {
                    choice[0] = 8;
                    sum[0] = sum[1];
                }
                sum[1] = count_bit(ix, begin, end, 9);
                if sum[1] <= sum[0] {
                    choice[0] = 9;
                }
            }
            10 => {
                sum[1] = count_bit(ix, begin, end, 11);
                if sum[1] <= sum[0] {
                    choice[0] = 11;
                    sum[0] = sum[1];
                }
                sum[1] = count_bit(ix, begin, end, 12);
                if sum[1] <= sum[0] {
                    choice[0] = 12;
                }
            }
            13 => {
                sum[1] = count_bit(ix, begin, end, 15);
                if sum[1] <= sum[0] {
                    choice[0] = 15;
                }
            }
            _ => {}
        }
    } else {
        // try tables with linbits
        let max_linbits = max - 15;

        for i in 15..24 {
            if let Some(table) = SHINE_HUFFMAN_TABLE.get(i) {
                if table.linmax >= max_linbits as u32 {
                    choice[0] = i as u32;
                    break;
                }
            }
        }

        for i in 24..32 {
            if let Some(table) = SHINE_HUFFMAN_TABLE.get(i) {
                if table.linmax >= max_linbits as u32 {
                    choice[1] = i as u32;
                    break;
                }
            }
        }

        sum[0] = count_bit(ix, begin, end, choice[0]);
        sum[1] = count_bit(ix, begin, end, choice[1]);
        if sum[1] < sum[0] {
            choice[0] = choice[1];
        }
    }

    choice[0]
}

/// Count the number of bits necessary to code the bigvalues region
/// Corresponds to bigv_bitcount() in l3loop.c
fn bigv_bitcount(ix: &[i32], gi: &GrInfo) -> i32 {
    let mut bits = 0;

    if gi.table_select[0] != 0 {
        bits += count_bit(ix, 0, gi.address1, gi.table_select[0]);
    }
    if gi.table_select[1] != 0 {
        bits += count_bit(ix, gi.address1, gi.address2, gi.table_select[1]);
    }
    if gi.table_select[2] != 0 {
        bits += count_bit(ix, gi.address2, gi.address3, gi.table_select[2]);
    }

    bits
}

/// Count the number of bits necessary to code the subregion
/// Corresponds to count_bit() in l3loop.c
pub fn count_bit(ix: &[i32], start: u32, end: u32, table: u32) -> i32 {
    if table == 0 {
        return 0;
    }

    let table_idx = table as usize;
    if table_idx >= SHINE_HUFFMAN_TABLE.len() {
        return 0;
    }

    let h = match SHINE_HUFFMAN_TABLE.get(table_idx) {
        Some(table) => table,
        None => return 0,
    };

    let mut sum = 0;
    let ylen = h.ylen;
    let linbits = h.linbits;

    if table > 15 {
        // ESC-table is used
        let mut i = start as usize;
        while i < end as usize && i + 1 < GRANULE_SIZE {
            let mut x = ix[i];
            let mut y = ix[i + 1];

            if x > 14 {
                x = 15;
                sum += linbits as i32;
            }
            if y > 14 {
                y = 15;
                sum += linbits as i32;
            }

            let idx = (x as u32 * ylen + y as u32) as usize;
            // WARNING: Added safety check - shine assumes hlen is always valid
            if let Some(hlen) = h.hlen {
                if idx < hlen.len() {
                    sum += hlen[idx] as i32;
                }
            } else {
                // WARNING: This branch doesn't exist in shine - added for safety
                log::warn!("Missing hlen table for Huffman table {}", table_idx);
            }

            if x != 0 {
                sum += 1;
            }
            if y != 0 {
                sum += 1;
            }

            i += 2;
        }
    } else {
        // No ESC-words
        let mut i = start as usize;
        while i < end as usize && i + 1 < GRANULE_SIZE {
            let x = ix[i];
            let y = ix[i + 1];

            let idx = (x as u32 * ylen + y as u32) as usize;
            // WARNING: Added safety check - shine assumes hlen is always valid
            if let Some(hlen) = h.hlen {
                if idx < hlen.len() {
                    sum += hlen[idx] as i32;
                }
            } else {
                // WARNING: This branch doesn't exist in shine - added for safety
                log::warn!("Missing hlen table for Huffman table {}", table_idx);
            }

            if x != 0 {
                sum += 1;
            }
            if y != 0 {
                sum += 1;
            }

            i += 2;
        }
    }

    sum
}

/// Binary search for optimal quantizer step size
/// Corresponds to bin_search_StepSize() in l3loop.c
fn bin_search_step_size(
    desired_rate: i32,
    ix: &mut [i32],
    cod_info: &mut GrInfo,
    config: &mut ShineGlobalConfig,
) -> i32 {
    let mut next = -120;
    let mut count = 120;

    loop {
        let half = count / 2;

        let bit = if quantize(ix, next + half, config) > 8192 {
            100000 // fail
        } else {
            calc_runlen(ix, cod_info); // rzero,count1,big_values
            let mut bit = count1_bitcount(ix, cod_info); // count1_table selection
            subdivide(cod_info, config); // bigvalues sfb division
            bigv_tab_select(ix, cod_info); // codebook selection
            bit += bigv_bitcount(ix, cod_info); // bit count
            bit
        };

        if bit < desired_rate {
            count = half;
        } else {
            next += half;
            count -= half;
        }

        if count <= 1 {
            break;
        }
    }

    next
}
