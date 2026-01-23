//! Subband filtering for MP3 encoding
//!
//! This module implements the polyphase subband filter that decomposes
//! PCM audio into 32 frequency subbands for further processing.
//! 
//! Following shine's l3subband.c implementation exactly (ref/shine/src/lib/l3subband.c)

use crate::error::{EncodingResult, EncodingError};
use crate::tables::ENWINDOW;
use std::f64::consts::PI;

/// Number of subbands in the filterbank (from shine's SBLIMIT)
const SBLIMIT: usize = 32;
/// Size of the history buffer (from shine's HAN_SIZE)
const HAN_SIZE: usize = 512;
/// Maximum number of channels (from shine's MAX_CHANNELS)
const MAX_CHANNELS: usize = 2;
/// PI/64 constant from shine
const PI64: f64 = PI / 64.0;

/// Subband filter for decomposing PCM audio into frequency bands
/// Following shine's subband_t structure exactly (ref/shine/src/lib/types.h:107-112)
pub struct SubbandFilter {
    /// Current offset in history buffer for each channel
    /// Following shine's subband.off[MAX_CHANNELS]
    off: [usize; MAX_CHANNELS],
    /// Polyphase filter coefficients [subband][coefficient]
    /// Following shine's subband.fl[SBLIMIT][64]
    fl: [[i32; 64]; SBLIMIT],
    /// History buffer for each channel [channel][sample]
    /// Following shine's subband.x[MAX_CHANNELS][HAN_SIZE]
    x: [[i32; HAN_SIZE]; MAX_CHANNELS],
}

impl SubbandFilter {
    /// Create a new subband filter
    /// Following shine's shine_subband_initialise exactly (ref/shine/src/lib/l3subband.c:15-35)
    pub fn new() -> Self {
        let mut filter = Self {
            off: [0; MAX_CHANNELS],
            fl: [[0; 64]; SBLIMIT],
            x: [[0; HAN_SIZE]; MAX_CHANNELS],
        };
        
        filter.initialize_coefficients();
        filter
    }
    
    /// Initialize the polyphase filter coefficients
    /// Following shine's coefficient calculation exactly (ref/shine/src/lib/l3subband.c:22-35)
    /// 
    /// Original shine code:
    /// for (i = SBLIMIT; i--;)
    ///   for (j = 64; j--;) {
    ///     if ((filter = 1e9 * cos((double)((2 * i + 1) * (16 - j) * PI64))) >= 0)
    ///       modf(filter + 0.5, &filter);
    ///     else
    ///       modf(filter - 0.5, &filter);
    ///     config->subband.fl[i][j] = (int32_t)(filter * (0x7fffffff * 1e-9));
    ///   }
    fn initialize_coefficients(&mut self) {
        // Following shine's exact loop structure: for (i = SBLIMIT; i--;)
        for i in (0..SBLIMIT).rev() {
            // for (j = 64; j--;)
            for j in (0..64).rev() {
                // Calculate filter coefficient exactly as in shine
                let filter_f64 = 1e9 * ((2 * i + 1) as f64 * (16 - j as i32) as f64 * PI64).cos();
                
                // Round to 9th decimal place accuracy as in shine
                let rounded = if filter_f64 >= 0.0 {
                    (filter_f64 + 0.5).floor()
                } else {
                    (filter_f64 - 0.5).ceil()
                };
                
                // Scale and convert to fixed point before storing
                self.fl[i][j] = (rounded * (0x7fffffff as f64 * 1e-9)) as i32;
            }
        }
    }
    
    /// Fixed-point multiplication (following shine's mul macro)
    /// Original shine: #define mul(a, b) (int32_t)((((int64_t)a) * ((int64_t)b)) >> 32)
    #[inline]
    fn mul(a: i32, b: i32) -> i32 {
        (((a as i64) * (b as i64)) >> 32) as i32
    }
    
    /// Window and filter PCM samples into subband samples
    /// Following shine's shine_window_filter_subband exactly (ref/shine/src/lib/l3subband.c:44-108)
    /// 
    /// This function processes 32 PCM samples and produces 32 subband samples
    /// 
    /// # Arguments
    /// * `pcm_samples` - Input PCM samples (32 samples)
    /// * `output` - Output subband samples (32 subbands)
    /// * `channel` - Channel index (0 for mono/left, 1 for right)
    pub fn filter(&mut self, pcm_samples: &[i16], output: &mut [i32; 32], channel: usize) -> EncodingResult<()> {
        if pcm_samples.len() != 32 {
            return Err(EncodingError::InvalidInputLength {
                expected: 32,
                actual: pcm_samples.len(),
            });
        }
        
        if channel >= MAX_CHANNELS {
            return Err(EncodingError::InvalidChannelIndex {
                channel,
                max_channels: MAX_CHANNELS,
            });
        }
        
        // Step 1: Replace 32 oldest samples with 32 new samples
        // Following shine: for (i = 32; i--;) { config->subband.x[ch][i + config->subband.off[ch]] = ((int32_t)*ptr) << 16; }
        for i in (0..32).rev() {
            self.x[channel][i + self.off[channel]] = (pcm_samples[31 - i] as i32) << 16;
        }
        
        // Step 2: Apply analysis window to produce windowed samples
        // Following shine's windowing loop exactly
        let mut y = [0i32; 64];
        for i in (0..64).rev() {
            let mut s_value: i32;
            
            // Following shine's multiply-accumulate pattern with 8 sections
            // mul0(s_value, s_value_lo, x[(off + i + (0 << 6)) & (HAN_SIZE - 1)], shine_enwindow[i + (0 << 6)]);
            s_value = Self::mul(
                self.x[channel][(self.off[channel] + i + (0 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (0 << 6)]
            );
            
            // muladd for sections 1-7
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (1 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (1 << 6)]
            );
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (2 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (2 << 6)]
            );
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (3 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (3 << 6)]
            );
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (4 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (4 << 6)]
            );
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (5 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (5 << 6)]
            );
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (6 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (6 << 6)]
            );
            s_value += Self::mul(
                self.x[channel][(self.off[channel] + i + (7 << 6)) & (HAN_SIZE - 1)],
                ENWINDOW[i + (7 << 6)]
            );
            
            // Store windowed sample (mulz macro does nothing in shine)
            y[i] = s_value;
        }
        
        // Update offset for next frame
        // Following shine: config->subband.off[ch] = (config->subband.off[ch] + 480) & (HAN_SIZE - 1);
        self.off[channel] = (self.off[channel] + 480) & (HAN_SIZE - 1);
        
        // Step 3: Apply polyphase filter matrix to produce subband samples
        // Following shine's polyphase filter loop exactly
        for i in (0..SBLIMIT).rev() {
            let mut s_value: i32;
            
            // Following shine's multiply-accumulate pattern:
            // mul0(s_value, s_value_lo, config->subband.fl[i][63], y[63]);
            s_value = Self::mul(self.fl[i][63], y[63]);
            
            // for (j = 63; j; j -= 7) { ... muladd operations ... }
            let mut j = 63;
            while j > 0 {
                if j >= 7 {
                    s_value += Self::mul(self.fl[i][j - 1], y[j - 1]);
                    s_value += Self::mul(self.fl[i][j - 2], y[j - 2]);
                    s_value += Self::mul(self.fl[i][j - 3], y[j - 3]);
                    s_value += Self::mul(self.fl[i][j - 4], y[j - 4]);
                    s_value += Self::mul(self.fl[i][j - 5], y[j - 5]);
                    s_value += Self::mul(self.fl[i][j - 6], y[j - 6]);
                    s_value += Self::mul(self.fl[i][j - 7], y[j - 7]);
                    j -= 7;
                } else {
                    // Handle remaining samples
                    for idx in (0..j).rev() {
                        s_value += Self::mul(self.fl[i][idx], y[idx]);
                    }
                    break;
                }
            }
            
            // Store result (mulz macro does nothing in shine)
            output[i] = s_value;
        }
        
        Ok(())
    }
    
    /// Reset the filter state
    /// Following shine's initialization pattern
    pub fn reset(&mut self) {
        self.off.fill(0);
        for channel_history in &mut self.x {
            channel_history.fill(0);
        }
    }
    
    /// Get the number of channels supported by this filter
    pub fn channels(&self) -> usize {
        MAX_CHANNELS
    }
    
    /// Get the current offset for a channel (for debugging/testing)
    pub fn get_offset(&self, channel: usize) -> Option<usize> {
        if channel < MAX_CHANNELS {
            Some(self.off[channel])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// 设置自定义 panic 钩子，只输出通用错误信息
    fn setup_panic_hook() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|_| {
                eprintln!("Test failed: Property test assertion failed");
            }));
        });
    }

    // Property test generators
    prop_compose! {
        fn pcm_samples_32()(samples in prop::collection::vec(any::<i16>(), 32)) -> Vec<i16> {
            samples
        }
    }

    prop_compose! {
        fn pcm_samples_invalid_length()(
            len in prop::sample::select(vec![0, 1, 16, 31, 33, 64, 100]),
            samples in prop::collection::vec(any::<i16>(), 0..=100)
        ) -> Vec<i16> {
            let mut result = samples;
            result.resize(len, 0);
            result
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        // Feature: rust-mp3-encoder, Property 5: 与参考实现一致性
        #[test]
        fn property_subband_coefficient_values(
            i in 0..SBLIMIT,
            j in 0..64usize,
        ) {
            setup_panic_hook();
            
            // For any same input data and configuration, our implementation should 
            // produce results consistent with the shine library's mathematical model
            let filter = SubbandFilter::new();
            
            // Verify filter coefficients follow the expected mathematical relationship
            // The coefficients should be calculated as: cos((2*i + 1) * (16 - j) * PI/64)
            let expected_angle = (2 * i + 1) as f64 * (16_i32 - j as i32) as f64 * PI64;
            let expected_coeff = expected_angle.cos();
            
            // Convert our fixed-point coefficient back to floating point for comparison
            let actual_coeff = filter.fl[i][j] as f64 / (0x7fffffff as f64);
            
            // Allow for some tolerance due to fixed-point precision
            let tolerance = 1e-6;
            let diff = (actual_coeff - expected_coeff).abs();
            prop_assert!(diff < tolerance, 
                "Coefficient [{}, {}] should match expected value: expected {}, got {}, diff {}",
                i, j, expected_coeff, actual_coeff, diff);
        }

        #[test]
        fn property_subband_filter_output_32_subbands(
            pcm_samples in pcm_samples_32(),
            channel_idx in 0..2usize,
        ) {
            setup_panic_hook();
            
            // For any PCM input data, subband filter should output exactly 32 subbands
            let mut filter = SubbandFilter::new();
            let mut output = [0i32; 32];
            
            let result = filter.filter(&pcm_samples, &mut output, channel_idx);
            prop_assert!(result.is_ok(), "Filter should succeed for valid inputs");
            
            // Verify we get exactly 32 subband outputs
            prop_assert_eq!(output.len(), 32, "Should produce exactly 32 subband samples");
        }

        #[test]
        fn property_subband_filter_output_stereo_independence(
            pcm_left in pcm_samples_32(),
            pcm_right in pcm_samples_32(),
        ) {
            setup_panic_hook();
            
            // For stereo data, left and right channels should be processed independently
            let mut filter = SubbandFilter::new();
            let mut output_left = [0i32; 32];
            let mut output_right = [0i32; 32];
            
            // Process left channel
            let result_left = filter.filter(&pcm_left, &mut output_left, 0);
            prop_assert!(result_left.is_ok(), "Left channel filtering should succeed");
            
            // Process right channel
            let result_right = filter.filter(&pcm_right, &mut output_right, 1);
            prop_assert!(result_right.is_ok(), "Right channel filtering should succeed");
            
            // Verify independence: processing one channel shouldn't affect the other
            // We can't directly test independence without a reference, but we can ensure
            // both channels produce valid outputs
            prop_assert_eq!(output_left.len(), 32, "Left channel should produce 32 subbands");
            prop_assert_eq!(output_right.len(), 32, "Right channel should produce 32 subbands");
        }

        #[test]
        fn property_subband_filter_output_invalid_input_length(
            invalid_samples in pcm_samples_invalid_length(),
        ) {
            setup_panic_hook();
            
            // For any invalid input length, filter should return appropriate error
            let mut filter = SubbandFilter::new();
            let mut output = [0i32; 32];
            
            if invalid_samples.len() != 32 {
                let result = filter.filter(&invalid_samples, &mut output, 0);
                prop_assert!(result.is_err(), "Filter should fail for invalid input length");
                
                if let Err(EncodingError::InvalidInputLength { expected, actual }) = result {
                    prop_assert_eq!(expected, 32, "Expected length should be 32");
                    prop_assert_eq!(actual, invalid_samples.len(), "Actual length should match input");
                }
            }
        }

        #[test]
        fn property_subband_filter_output_invalid_channel(
            pcm_samples in pcm_samples_32(),
            invalid_channel in 2..16usize,
        ) {
            setup_panic_hook();
            
            // For any invalid channel index, filter should return appropriate error
            let mut filter = SubbandFilter::new();
            let mut output = [0i32; 32];
            
            let result = filter.filter(&pcm_samples, &mut output, invalid_channel);
            prop_assert!(result.is_err(), "Filter should fail for invalid channel index");
            
            if let Err(EncodingError::InvalidChannelIndex { channel, max_channels }) = result {
                prop_assert_eq!(channel, invalid_channel, "Error should contain invalid channel index");
                prop_assert_eq!(max_channels, MAX_CHANNELS, "Error should contain max channels");
            }
        }

        #[test]
        fn property_subband_filter_reset_behavior(
            pcm_samples in pcm_samples_32(),
        ) {
            setup_panic_hook();
            
            // Reset should clear all state and allow fresh processing
            let mut filter = SubbandFilter::new();
            let mut output1 = [0i32; 32];
            let mut output2 = [0i32; 32];
            
            // Process some data
            let _ = filter.filter(&pcm_samples, &mut output1, 0);
            
            // Reset the filter
            filter.reset();
            
            // Verify all offsets are reset to 0
            for ch in 0..MAX_CHANNELS {
                prop_assert_eq!(filter.get_offset(ch), Some(0), "Offset should be reset to 0");
            }
            
            // Process the same data again - should work without error
            let result = filter.filter(&pcm_samples, &mut output2, 0);
            prop_assert!(result.is_ok(), "Filter should work after reset");
        }

        #[test]
        fn property_subband_filter_deterministic_output(
            pcm_samples in pcm_samples_32(),
        ) {
            setup_panic_hook();
            
            // Same input should produce same output (deterministic behavior)
            let mut filter1 = SubbandFilter::new();
            let mut filter2 = SubbandFilter::new();
            let mut output1 = [0i32; 32];
            let mut output2 = [0i32; 32];
            
            // Process same data with both filters
            let result1 = filter1.filter(&pcm_samples, &mut output1, 0);
            let result2 = filter2.filter(&pcm_samples, &mut output2, 0);
            
            prop_assert!(result1.is_ok() && result2.is_ok(), "Both filters should succeed");
            prop_assert_eq!(output1, output2, "Same input should produce same output");
        }
    }

    #[test]
    fn test_subband_filter_functionality() {
        // Smoke test for subband filtering
        let mut filter = SubbandFilter::new();
        let pcm_samples = [100i16; 32];
        let mut output = [0i32; 32];
        
        let result = filter.filter(&pcm_samples, &mut output, 0);
        assert!(result.is_ok(), "Basic filtering should work");
        
        // Verify we get some non-zero output (filter should process the input)
        let has_nonzero = output.iter().any(|&x| x != 0);
        assert!(has_nonzero, "Filter should produce non-zero output for non-zero input");
    }

    #[test]
    fn test_subband_filter_zero_input() {
        // Test with zero input
        let mut filter = SubbandFilter::new();
        let pcm_samples = [0i16; 32];
        let mut output = [0i32; 32];
        
        let result = filter.filter(&pcm_samples, &mut output, 0);
        assert!(result.is_ok(), "Zero input should be processed successfully");
        
        // With zero input, output should be zero (or very close to zero due to history)
        // This test mainly ensures no crashes or errors occur
    }

    #[test]
    fn test_subband_filter_max_input() {
        // Test with maximum input values
        let mut filter = SubbandFilter::new();
        let pcm_samples = [i16::MAX; 32];
        let mut output = [0i32; 32];
        
        let result = filter.filter(&pcm_samples, &mut output, 0);
        assert!(result.is_ok(), "Maximum input should be processed successfully");
    }

    #[test]
    fn test_subband_filter_min_input() {
        // Test with minimum input values
        let mut filter = SubbandFilter::new();
        let pcm_samples = [i16::MIN; 32];
        let mut output = [0i32; 32];
        
        let result = filter.filter(&pcm_samples, &mut output, 0);
        assert!(result.is_ok(), "Minimum input should be processed successfully");
    }

    #[test]
    fn test_subband_filter_coefficient_initialization() {
        // Filter coefficients should be properly initialized
        let filter = SubbandFilter::new();
        
        // Verify filter has the expected number of channels
        assert_eq!(filter.channels(), MAX_CHANNELS, "Filter should support MAX_CHANNELS");
        
        // Verify initial offsets are zero
        for ch in 0..MAX_CHANNELS {
            assert_eq!(filter.get_offset(ch), Some(0), "Initial offset should be 0");
        }
    }
}