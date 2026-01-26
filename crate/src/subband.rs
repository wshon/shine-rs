//! Subband analysis filterbank implementation
//!
//! This module implements the polyphase filterbank for subband analysis,
//! which is the first step in MP3 encoding. It converts PCM samples into
//! 32 subband samples using a windowed analysis filterbank.
//!
//! The implementation strictly follows the shine reference implementation
//! in ref/shine/src/lib/l3subband.c

use crate::tables::SHINE_ENWINDOW;
use crate::types::{Subband, MAX_CHANNELS, SBLIMIT, HAN_SIZE};
use std::f64::consts::PI;

/// Multiplication macros matching shine's mult_noarch_gcc.h
/// These implement fixed-point arithmetic operations

/// Basic multiplication with 32-bit right shift
#[inline]
pub fn mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 32) as i32
}

/// Multiplication with rounding and 32-bit right shift
#[inline]
#[allow(dead_code)] // Used in tests
pub fn mulr(a: i32, b: i32) -> i32 {
    (((a as i64 * b as i64) + 0x80000000i64) >> 32) as i32
}

/// Initialize multiplication operation (matches shine mul0 macro)
#[inline]
pub fn mul0(a: i32, b: i32) -> i32 {
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

/// Initialize the subband analysis filterbank
/// Corresponds to shine_subband_initialise() in l3subband.c
///
/// Calculates the analysis filterbank coefficients and rounds to the
/// 9th decimal place accuracy of the filterbank tables in the ISO
/// document. The coefficients are stored in the fl array.
pub fn shine_subband_initialise(subband: &mut Subband) {
    // Initialize channel offsets and sample buffers (matches shine implementation)
    for i in 0..MAX_CHANNELS {
        subband.off[i] = 0;
        for j in 0..HAN_SIZE {
            subband.x[i][j] = 0;
        }
    }

    // Calculate filterbank coefficients (matches shine implementation exactly)
    for i in (0..SBLIMIT).rev() {  // matches shine: for (i = SBLIMIT; i--;)
        for j in (0..64).rev() {   // matches shine: for (j = 64; j--;)
            // Calculate filter coefficient using the same formula as shine
            // filter = 1e9 * cos((double)((2 * i + 1) * (16 - j) * PI64))
            let angle = (2 * i + 1) as f64 * (16 - j as i32) as f64 * (PI / 64.0);
            let mut filter = 1e9 * angle.cos();
            
            // Apply rounding (matches shine's modf logic)
            if filter >= 0.0 {
                filter = (filter + 0.5).floor();
            } else {
                filter = (filter - 0.5).ceil();
            }
            
            // Scale and convert to fixed point before storing
            // (matches shine: filter * (0x7fffffff * 1e-9))
            subband.fl[i][j] = (filter * (0x7fffffff as f64 * 1e-9)) as i32;
        }
    }
}

/// Windowed subband analysis filterbank
/// Corresponds to shine_window_filter_subband() in l3subband.c
///
/// Overlapping window on PCM samples:
/// 1. 32 16-bit PCM samples are scaled to fractional 2's complement and
///    concatenated to the end of the window buffer x
/// 2. The updated window buffer x is windowed by the analysis window to
///    produce the windowed sample z
/// 3. The windowed samples z are filtered by the digital filter matrix
///    to produce the subband samples s
pub fn shine_window_filter_subband(
    buffer: &mut &[i16],
    s: &mut [i32; SBLIMIT],
    ch: usize,
    subband: &mut Subband,
    stride: usize,
) {
    let mut y = [0i32; 64];
    
    // Replace 32 oldest samples with 32 new samples
    // (matches shine implementation exactly: for (i = 32; i--;))
    let mut ptr_offset = 0;
    for i in (0..32).rev() {  // i from 31 down to 0 (matches shine: for (i = 32; i--;))
        if ptr_offset < buffer.len() {
            subband.x[ch][i + subband.off[ch] as usize] = 
                (buffer[ptr_offset] as i32) << 16;
        }
        ptr_offset += stride;
    }
    
    // Advance buffer pointer (matches shine's pointer arithmetic)
    if buffer.len() >= 32 * stride {
        *buffer = &buffer[32 * stride..];
    }

    // Apply analysis window (matches shine implementation exactly)
    for i in 0..64 {
        #[allow(unused_assignments)] // s_value is used but compiler doesn't detect it properly
        let mut s_value = 0i32;
        
        // Windowing operation using shine's exact loop structure
        s_value = mul0(
            subband.x[ch][(subband.off[ch] as usize + i + (0 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (0 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (1 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (1 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (2 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (2 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (3 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (3 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (4 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (4 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (5 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (5 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (6 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (6 << 6)]
        );
        s_value = muladd(
            s_value,
            subband.x[ch][(subband.off[ch] as usize + i + (7 << 6)) & (HAN_SIZE - 1)],
            SHINE_ENWINDOW[i + (7 << 6)]
        );
        
        y[i] = mulz(s_value);
    }

    // Update circular buffer offset (matches shine modulo operation)
    subband.off[ch] = (subband.off[ch] + 480) & (HAN_SIZE as i32 - 1);

    // Apply synthesis filterbank (matches shine implementation exactly)
    for i in (0..SBLIMIT).rev() {  // i from SBLIMIT-1 down to 0 (matches shine: for (i = SBLIMIT; i--;))
        let mut s_value: i32;
        
        // Start with the last coefficient (j=63) (matches shine exactly)
        s_value = mul0(subband.fl[i][63], y[63]);
        
        // Process remaining coefficients in groups of 7 (matches shine's unrolled loop exactly)
        let mut j = 63;
        while j > 0 {
            if j >= 7 {
                s_value = muladd(s_value, subband.fl[i][j - 1], y[j - 1]);
                s_value = muladd(s_value, subband.fl[i][j - 2], y[j - 2]);
                s_value = muladd(s_value, subband.fl[i][j - 3], y[j - 3]);
                s_value = muladd(s_value, subband.fl[i][j - 4], y[j - 4]);
                s_value = muladd(s_value, subband.fl[i][j - 5], y[j - 5]);
                s_value = muladd(s_value, subband.fl[i][j - 6], y[j - 6]);
                s_value = muladd(s_value, subband.fl[i][j - 7], y[j - 7]);
                j -= 7;
            } else {
                break;  // Shine doesn't handle remaining coefficients in this loop
            }
        }
        
        s[i] = mulz(s_value);
    }
}

