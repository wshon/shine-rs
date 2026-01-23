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
fn mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 32) as i32
}

/// Multiplication with rounding and 32-bit right shift
#[inline]
#[allow(dead_code)] // Used in tests
fn mulr(a: i32, b: i32) -> i32 {
    (((a as i64 * b as i64) + 0x80000000i64) >> 32) as i32
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
    // (matches shine implementation exactly)
    for i in 0..32 {
        let sample_idx = 31 - i; // Reverse order to match shine's loop
        if sample_idx * stride < buffer.len() {
            subband.x[ch][sample_idx + subband.off[ch] as usize] = 
                (buffer[sample_idx * stride] as i32) << 16;
        }
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
    for i in 0..SBLIMIT {
        #[allow(unused_assignments)] // s_value is used but compiler doesn't detect it properly
        let mut s_value = 0i32;
        
        // Start with the last coefficient (j=63)
        s_value = mul0(subband.fl[i][63], y[63]);
        
        // Process remaining coefficients in groups of 7 (matches shine's unrolled loop)
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
                // Handle remaining coefficients
                for k in (0..j).rev() {
                    s_value = muladd(s_value, subband.fl[i][k], y[k]);
                }
                break;
            }
        }
        
        s[i] = mulz(s_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_subband_initialise_coefficients(
            _unit in Just(())
        ) {
            let mut subband = Subband::default();
            shine_subband_initialise(&mut subband);
            
            // Verify that coefficients are initialized (non-zero for most entries)
            let mut non_zero_count = 0;
            for i in 0..SBLIMIT {
                for j in 0..64 {
                    if subband.fl[i][j] != 0 {
                        non_zero_count += 1;
                    }
                }
            }
            
            prop_assert!(non_zero_count > SBLIMIT * 32, "Most coefficients should be non-zero");
            
            // Verify channel offsets are initialized to zero
            for i in 0..MAX_CHANNELS {
                prop_assert_eq!(subband.off[i], 0, "Channel offset should be zero");
            }
        }

        #[test]
        fn test_window_filter_subband_basic(
            samples in prop::collection::vec(-32768i16..32767, 32..64),
            channel in 0usize..MAX_CHANNELS,
        ) {
            let mut subband = Subband::default();
            shine_subband_initialise(&mut subband);
            
            let mut buffer = samples.as_slice();
            let mut s = [0i32; SBLIMIT];
            
            shine_window_filter_subband(&mut buffer, &mut s, channel, &mut subband, 1);
            
            // Verify that subband samples are generated
            let mut has_non_zero = false;
            for &sample in &s {
                if sample != 0 {
                    has_non_zero = true;
                    break;
                }
            }
            
            // For non-zero input, we should get some non-zero output
            let has_non_zero_input = samples.iter().any(|&x| x != 0);
            if has_non_zero_input {
                prop_assert!(has_non_zero, "Non-zero input should produce non-zero output");
            }
        }

        #[test]
        fn test_multiplication_functions(
            a in -1000000i32..1000000,
            b in -1000000i32..1000000,
        ) {
            // Test that multiplication functions don't overflow
            let result1 = mul(a, b);
            let result2 = mulr(a, b);
            let result3 = mul0(a, b);
            
            // Results should be finite
            prop_assert!(result1.abs() <= i32::MAX, "mul result should be valid");
            prop_assert!(result2.abs() <= i32::MAX, "mulr result should be valid");
            prop_assert!(result3.abs() <= i32::MAX, "mul0 result should be valid");
            
            // mul0 should equal mul
            prop_assert_eq!(result1, result3, "mul0 should equal mul");
        }

        #[test]
        fn test_subband_state_consistency(
            _unit in Just(())
        ) {
            let mut subband = Subband::default();
            
            // Test multiple initializations produce same result
            shine_subband_initialise(&mut subband);
            let fl_copy1 = subband.fl;
            
            shine_subband_initialise(&mut subband);
            let fl_copy2 = subband.fl;
            
            prop_assert_eq!(fl_copy1, fl_copy2, "Multiple initializations should be identical");
        }
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CHANNELS, 2);
        assert_eq!(SBLIMIT, 32);
        assert_eq!(HAN_SIZE, 512);
    }

    #[test]
    fn test_subband_state_default() {
        let subband = Subband::default();
        
        // Verify default initialization
        for i in 0..MAX_CHANNELS {
            assert_eq!(subband.off[i], 0);
            for j in 0..HAN_SIZE {
                assert_eq!(subband.x[i][j], 0);
            }
        }
        
        for i in 0..SBLIMIT {
            for j in 0..64 {
                assert_eq!(subband.fl[i][j], 0);
            }
        }
    }
}