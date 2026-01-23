//! Quantization and rate control for MP3 encoding
//!
//! This module implements the quantization loop that controls the
//! trade-off between audio quality and bitrate by adjusting quantization
//! step sizes and managing the bit reservoir.
//! 
//! The implementation strictly follows the shine reference implementation
//! in ref/shine/src/lib/l3loop.c

use crate::shine_config::{ShineGlobalConfig, MAX_CHANNELS, MAX_GRANULES, GRANULE_SIZE, SBLIMIT};
use crate::tables::{SCALE_FACT_BAND_INDEX, SLEN1_TAB, SLEN2_TAB, HUFFMAN_TABLES, COUNT1_TABLES};
use crate::reservoir::shine_max_reservoir_bits;
use std::f64::consts::LN_2;

/// Constants from shine (matches l3loop.c exactly)
const E: f64 = 2.71828182845;
const CBLIMIT: usize = 21;
const SFB_LMAX: usize = 22;
const EN_TOT_KRIT: i32 = 10;
const EN_DIF_KRIT: i32 = 100;
const EN_SCFSI_BAND_KRIT: i32 = 10;
const XM_SCFSI_BAND_KRIT: i32 = 10;

/// Granule information structure following shine's gr_info exactly
/// (ref/shine/src/lib/types.h:114-133)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GranuleInfo {
    /// Length of part2_3 data in bits
    pub part2_3_length: u32,
    /// Number of big values (must be <= 288 per MP3 standard)
    pub big_values: u32,
    /// Number of count1 quadruples
    pub count1: u32,
    /// Global gain value
    pub global_gain: u32,
    /// Scale factor compression
    pub scalefac_compress: u32,
    /// Huffman table selection [region]
    pub table_select: [u32; 3],
    /// Region 0 count
    pub region0_count: u32,
    /// Region 1 count
    pub region1_count: u32,
    /// Pre-emphasis flag
    pub preflag: u32,
    /// Scale factor scale
    pub scalefac_scale: u32,
    /// Count1 table selection
    pub count1table_select: u32,
    /// Part2 length in bits
    pub part2_length: u32,
    /// Scale factor band limit for long blocks
    pub sfb_lmax: u32,
    /// Region addresses for Huffman coding
    pub address1: u32,
    pub address2: u32,
    pub address3: u32,
    /// Quantizer step size (signed to match shine's int)
    pub quantizer_step_size: i32,
    /// Scale factor lengths [slen_type]
    pub slen: [u32; 4],
}

impl Default for GranuleInfo {
    fn default() -> Self {
        Self {
            part2_3_length: 0,
            big_values: 0,
            count1: 0,
            global_gain: 210,
            scalefac_compress: 0,
            table_select: [0, 0, 0],
            region0_count: 0,
            region1_count: 0,
            preflag: 0,
            scalefac_scale: 0,
            count1table_select: 0,
            part2_length: 0,
            sfb_lmax: SFB_LMAX as u32 - 1,
            address1: 0,
            address2: 0,
            address3: 0,
            quantizer_step_size: 0,
            slen: [0, 0, 0, 0],
        }
    }
}

/// Psychoacoustic minimum structure
#[derive(Debug)]
pub struct ShinePsyXmin {
    pub l: [[[f64; 21]; MAX_CHANNELS]; MAX_GRANULES],
}

impl Default for ShinePsyXmin {
    fn default() -> Self {
        Self {
            l: [[[0.0; 21]; MAX_CHANNELS]; MAX_GRANULES],
        }
    }
}
/// Multiplication macros matching shine's mult_noarch_gcc.h
/// These implement fixed-point arithmetic operations

/// Multiply with rounding and 31-bit right shift (matches shine mulsr)
#[inline]
fn mulsr(a: i32, b: i32) -> i32 {
    (((a as i64 * b as i64) + 0x40000000i64) >> 31) as i32
}

/// Multiply with rounding and 32-bit right shift (matches shine mulr)
#[inline]
fn mulr(a: i32, b: i32) -> i32 {
    (((a as i64 * b as i64) + 0x80000000i64) >> 32) as i32
}

/// Absolute value function (matches shine labs)
#[inline]
fn labs(x: i32) -> i32 {
    x.abs()
}

/// Inner loop: find optimal quantization step size for given scalefactors
/// Corresponds to shine_inner_loop() in l3loop.c
///
/// The code selects the best quantizerStepSize for a particular set
/// of scalefacs.
pub fn shine_inner_loop(
    ix: &mut [i32; GRANULE_SIZE],
    max_bits: i32,
    cod_info: &mut GranuleInfo,
    gr: i32,
    ch: i32,
    config: &mut ShineGlobalConfig,
) -> i32 {
    let mut bits: i32;
    let mut c1bits: i32;
    let mut bvbits: i32;

    // Following shine's logic exactly:
    // if (max_bits < 0) cod_info->quantizerStepSize--;
    if max_bits < 0 {
        cod_info.quantizer_step_size -= 1;
    }

    // Main quantization loop - following shine's do-while structure exactly
    loop {
        // while (quantize(ix, ++cod_info->quantizerStepSize, config) > 8192)
        //   ; /* within table range? */
        loop {
            cod_info.quantizer_step_size += 1;
            if quantize(ix, cod_info.quantizer_step_size, config) <= 8192 {
                break;
            }
        }

        calc_runlen(ix, cod_info); // rzero,count1,big_values
        bits = count1_bitcount(ix, cod_info); // count1_table selection
        c1bits = bits;
        subdivide(cod_info, config); // bigvalues sfb division
        bigv_tab_select(ix, cod_info); // codebook selection
        bvbits = bigv_bitcount(ix, cod_info); // bit count
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
    l3_xmin: &mut ShinePsyXmin, // the allowed distortion of the scalefactor
    ix: &mut [i32; GRANULE_SIZE], // vector of quantized values ix(0..575)
    gr: i32,
    ch: i32,
    config: &mut ShineGlobalConfig,
) -> i32 {
    let mut bits: i32;
    let huff_bits: i32;
    let side_info = &mut config.side_info;
    let cod_info = &mut side_info.gr[gr as usize].ch[ch as usize].tt;

    cod_info.quantizer_step_size = bin_search_step_size(max_bits, ix, cod_info, config);

    cod_info.part2_length = part2_length(gr, ch, config) as u32;
    huff_bits = max_bits - cod_info.part2_length as i32;

    bits = shine_inner_loop(ix, huff_bits, cod_info, gr, ch, config);
    cod_info.part2_3_length = cod_info.part2_length + bits as u32;

    cod_info.part2_3_length as i32
}

/// Main iteration loop for encoding
/// Corresponds to shine_iteration_loop() in l3loop.c
pub fn shine_iteration_loop(config: &mut ShineGlobalConfig) {
    let mut l3_xmin = ShinePsyXmin::default();
    let mut max_bits: i32;
    let mut ix: *mut i32;

    // Following shine's exact loop structure:
    // for (ch = config->wave.channels; ch--;)
    for ch in (0..config.wave.channels).rev() {
        // for (gr = 0; gr < config->mpeg.granules_per_frame; gr++)
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

            let cod_info = &mut config.side_info.gr[gr as usize].ch[ch as usize].tt;
            cod_info.sfb_lmax = (SFB_LMAX - 1) as u32; // gr_deco

            calc_xmin(&config.ratio, cod_info, &mut l3_xmin, gr, ch);

            if config.mpeg.version == 1 {
                // MPEG_I
                calc_scfsi(&mut l3_xmin, ch, gr, config);
            }

            // calculation of number of available bit( per granule )
            max_bits = shine_max_reservoir_bits(&config.pe[ch as usize][gr as usize], config);

            // reset of iteration variables
            for i in 0..config.scalefactor.l[gr as usize][ch as usize].len() {
                config.scalefactor.l[gr as usize][ch as usize][i] = 0;
            }
            for i in 0..config.scalefactor.s[gr as usize][ch as usize].len() {
                for j in 0..config.scalefactor.s[gr as usize][ch as usize][i].len() {
                    config.scalefactor.s[gr as usize][ch as usize][i][j] = 0;
                }
            }

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

            // all spectral values zero ?
            if config.l3loop.xrmax != 0 {
                let ix_slice = unsafe { std::slice::from_raw_parts_mut(ix, GRANULE_SIZE) };
                cod_info.part2_3_length = shine_outer_loop(
                    max_bits,
                    &mut l3_xmin,
                    ix_slice.try_into().unwrap(),
                    gr,
                    ch,
                    config,
                ) as u32;
            }

            crate::reservoir::shine_resv_adjust(cod_info, config);
            cod_info.global_gain = (cod_info.quantizer_step_size + 210) as u32;
        } // for gr
    } // for ch

    crate::reservoir::shine_resv_frame_end(config);
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

    let samplerate_index = match config.wave.sample_rate {
        44100 => 0, 48000 => 1, 32000 => 2, 22050 => 3, 24000 => 4,
        16000 => 5, 11025 => 6, 12000 => 7, 8000 => 8, _ => 0,
    };

    let scalefac_band_long = &SCALE_FACT_BAND_INDEX[samplerate_index];

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
fn part2_length(gr: i32, ch: i32, config: &mut ShineGlobalConfig) -> i32 {
    let mut bits = 0;
    let gi = &config.side_info.gr[gr as usize].ch[ch as usize].tt;

    let slen1 = SLEN1_TAB[gi.scalefac_compress as usize % SLEN1_TAB.len()];
    let slen2 = SLEN2_TAB[gi.scalefac_compress as usize % SLEN2_TAB.len()];

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
    ratio: &crate::shine_config::ShinePsyRatio,
    cod_info: &mut GranuleInfo,
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
fn quantize(ix: &mut [i32; GRANULE_SIZE], stepsize: i32, config: &mut ShineGlobalConfig) -> i32 {
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
fn ix_max(ix: &[i32; GRANULE_SIZE], begin: u32, end: u32) -> i32 {
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
fn calc_runlen(ix: &mut [i32; GRANULE_SIZE], cod_info: &mut GranuleInfo) {
    let mut i = GRANULE_SIZE;
    let mut rzero = 0;

    // Count trailing zero pairs
    while i > 1 {
        i -= 2;
        if ix[i] == 0 && ix[i + 1] == 0 {
            rzero += 1;
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
fn count1_bitcount(ix: &[i32; GRANULE_SIZE], cod_info: &mut GranuleInfo) -> i32 {
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

        if p < COUNT1_TABLES[0].lengths.len() {
            sum0 += COUNT1_TABLES[0].lengths[p] as i32;
        }
        if p < COUNT1_TABLES[1].lengths.len() {
            sum1 += COUNT1_TABLES[1].lengths[p] as i32;
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