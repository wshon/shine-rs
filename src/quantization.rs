//! Quantization and rate control for MP3 encoding
//!
//! This module implements the quantization loop that controls the
//! trade-off between audio quality and bitrate by adjusting quantization
//! step sizes and managing the bit reservoir.

use crate::error::{EncodingError, EncodingResult};
use crate::reservoir::BitReservoir;

/// Number of MDCT coefficients per granule
pub const GRANULE_SIZE: usize = 576;

/// Quantization loop for rate control and quality management
pub struct QuantizationLoop {
    /// Quantization step table (floating point)
    step_table: [f32; 256],
    /// Integer version of step table for fixed-point arithmetic
    step_table_i32: [i32; 256],
    /// Integer to index lookup table for quantization
    int2idx: [u32; 10000],
    /// Bit reservoir for rate control
    #[allow(dead_code)]
    reservoir: BitReservoir,
}

/// Granule information structure
#[derive(Debug, Clone)]
pub struct GranuleInfo {
    /// Length of part2_3 data in bits
    pub part2_3_length: u32,
    /// Number of big values
    pub big_values: u32,
    /// Global gain value
    pub global_gain: u32,
    /// Scale factor compression
    pub scalefac_compress: u32,
    /// Huffman table selection
    pub table_select: [u32; 3],
    /// Region 0 count
    pub region0_count: u32,
    /// Region 1 count
    pub region1_count: u32,
    /// Pre-emphasis flag
    pub preflag: bool,
    /// Scale factor scale
    pub scalefac_scale: bool,
    /// Count1 table selection
    pub count1table_select: bool,
    /// Quantizer step size
    pub quantizer_step_size: i32,
    /// Number of count1 quadruples
    pub count1: u32,
    /// Part2 length in bits
    pub part2_length: u32,
    /// Region addresses for Huffman coding
    pub address1: u32,
    pub address2: u32,
    pub address3: u32,
}

impl Default for GranuleInfo {
    fn default() -> Self {
        Self {
            part2_3_length: 0,
            big_values: 0,
            global_gain: 0,
            scalefac_compress: 0,
            table_select: [0; 3],
            region0_count: 0,
            region1_count: 0,
            preflag: false,
            scalefac_scale: false,
            count1table_select: false,
            quantizer_step_size: 0,
            count1: 0,
            part2_length: 0,
            address1: 0,
            address2: 0,
            address3: 0,
        }
    }
}

impl QuantizationLoop {
    /// Create a new quantization loop
    pub fn new() -> Self {
        let mut quantizer = Self {
            step_table: [0.0; 256],
            step_table_i32: [0; 256],
            int2idx: [0; 10000],
            reservoir: BitReservoir::new(7680), // Maximum reservoir size for Layer III
        };
        
        quantizer.initialize_tables();
        quantizer
    }
    
    /// Initialize quantization lookup tables
    fn initialize_tables(&mut self) {
        // Initialize step table: 2^(-stepsize/4)
        // The table is inverted (negative power) from the equation given
        // in the spec because it is quicker to do x*y than x/y.
        for i in 0..256 {
            self.step_table[i] = (2.0_f32).powf((127 - i as i32) as f32 / 4.0);
            
            // Convert to fixed point with extra bit of accuracy
            // The table is multiplied by 2 to give an extra bit of accuracy.
            if (self.step_table[i] * 2.0) > 0x7fffffff as f32 {
                self.step_table_i32[i] = 0x7fffffff;
            } else {
                self.step_table_i32[i] = ((self.step_table[i] * 2.0) + 0.5) as i32;
            }
        }
        
        // Initialize int2idx table: quantization index lookup
        // The 0.5 is for rounding, the 0.0946 comes from the spec.
        for i in 0..10000 {
            let val = (i as f64).sqrt().sqrt() * (i as f64).sqrt() - 0.0946 + 0.5;
            self.int2idx[i] = val.max(0.0) as u32;
        }
    }
    
    /// Quantize MDCT coefficients using non-linear quantization
    /// Returns the maximum quantized value
    pub fn quantize(&self, mdct_coeffs: &[i32; GRANULE_SIZE], stepsize: i32, output: &mut [i32; GRANULE_SIZE]) -> i32 {
        let mut max_value = 0;
        
        // Get the step size from the table
        let step_index = (stepsize + 127).clamp(0, 255) as usize;
        let scalei = self.step_table_i32[step_index];
        
        // Find maximum absolute value for quick check
        let xrmax = mdct_coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0);
        
        // Quick check to see if max quantized value will be less than 8192
        // This speeds up the early calls to binary search
        if Self::multiply_and_round(xrmax, scalei) > 165140 { // 8192^(4/3)
            return 16384; // No point in continuing, stepsize not big enough
        }
        
        for i in 0..GRANULE_SIZE {
            let abs_coeff = mdct_coeffs[i].abs();
            
            if abs_coeff == 0 {
                output[i] = 0;
                continue;
            }
            
            // Multiply coefficient by step size
            let ln = Self::multiply_and_round(abs_coeff, scalei);
            
            let quantized = if ln < 10000 {
                // Use lookup table for fast quantization
                self.int2idx[ln as usize] as i32
            } else {
                // Outside table range, use floating point calculation
                let scale = self.step_table[step_index];
                let dbl = (abs_coeff as f64) * (scale as f64) * 4.656612875e-10; // 1.0 / 0x7fffffff
                (dbl.sqrt().sqrt() * dbl.sqrt()) as i32 // dbl^(3/4)
            };
            
            // Apply sign
            output[i] = if mdct_coeffs[i] < 0 { -quantized } else { quantized };
            
            // Track maximum value
            if quantized > max_value {
                max_value = quantized;
            }
        }
        
        max_value
    }
    
    /// Multiply two integers with rounding (fixed-point arithmetic)
    fn multiply_and_round(a: i32, b: i32) -> i32 {
        let result = (a as i64) * (b as i64);
        ((result + (1 << 30)) >> 31) as i32 // Round and shift
    }
    
    /// Calculate quantization step size for given coefficients
    pub fn calculate_step_size(&self, mdct_coeffs: &[i32; GRANULE_SIZE], target_bits: usize) -> i32 {
        // Binary search for optimal step size
        let mut low = -120;
        let mut high = 120;
        let mut best_step = 0;
        
        while low <= high {
            let mid = (low + high) / 2;
            let mut temp_output = [0i32; GRANULE_SIZE];
            let max_quantized = self.quantize(mdct_coeffs, mid, &mut temp_output);
            
            if max_quantized > 8192 {
                // Step size too small, increase it
                low = mid + 1;
            } else {
                // Calculate approximate bit count (simplified)
                let estimated_bits = self.estimate_bits(&temp_output);
                
                if estimated_bits <= target_bits {
                    best_step = mid;
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }
        
        best_step
    }
    
    /// Estimate the number of bits needed for quantized coefficients
    /// This is a simplified estimation for the binary search
    fn estimate_bits(&self, quantized: &[i32; GRANULE_SIZE]) -> usize {
        let mut bits = 0;
        
        // Count non-zero coefficients and estimate bits
        for &coeff in quantized.iter() {
            if coeff != 0 {
                let abs_val = coeff.abs();
                if abs_val == 1 {
                    bits += 2; // Rough estimate for small values
                } else if abs_val <= 15 {
                    bits += 4; // Rough estimate for medium values
                } else {
                    bits += 8; // Rough estimate for large values
                }
            }
        }
        
        bits
    }
    
    /// Quantize MDCT coefficients and encode them
    pub fn quantize_and_encode(
        &mut self,
        mdct_coeffs: &[i32; GRANULE_SIZE],
        max_bits: usize,
        side_info: &mut GranuleInfo,
        output: &mut [i32; GRANULE_SIZE]
    ) -> EncodingResult<usize> {
        // Calculate initial quantization step size
        let initial_step = self.calculate_step_size(mdct_coeffs, max_bits);
        side_info.quantizer_step_size = initial_step;
        
        // Quantize the coefficients
        let max_quantized = self.quantize(mdct_coeffs, initial_step, output);
        
        if max_quantized > 8192 {
            return Err(EncodingError::QuantizationFailed);
        }
        
        // Calculate run length encoding info
        self.calculate_run_length(output, side_info);
        
        // Set global gain (quantizer step size + 210 as per MP3 spec)
        side_info.global_gain = (initial_step + 210) as u32;
        
        // Return estimated bit count
        Ok(self.estimate_bits(output))
    }
    
    /// Calculate run length encoding information
    fn calculate_run_length(&self, quantized: &[i32; GRANULE_SIZE], side_info: &mut GranuleInfo) {
        // Count trailing zeros
        let mut _rzero = 0;
        let mut i = GRANULE_SIZE;
        
        while i > 1 {
            if quantized[i - 1] == 0 && quantized[i - 2] == 0 {
                _rzero += 1;
                i -= 2;
            } else {
                break;
            }
        }
        
        // Count quadruples (count1 region)
        side_info.count1 = 0;
        while i > 3 {
            if quantized[i - 1].abs() <= 1 && quantized[i - 2].abs() <= 1 &&
               quantized[i - 3].abs() <= 1 && quantized[i - 4].abs() <= 1 {
                side_info.count1 += 1;
                i -= 4;
            } else {
                break;
            }
        }
        
        // Set big values count
        side_info.big_values = (i / 2) as u32;
    }
    
    /// Inner loop: find optimal Huffman table selection
    #[allow(dead_code)]
    fn inner_loop(&self, _coeffs: &mut [i32; GRANULE_SIZE], _max_bits: usize, _info: &mut GranuleInfo) -> usize {
        // Implementation will be added in task 9.5
        todo!("Inner loop implementation")
    }
    
    /// Outer loop: adjust quantization step size
    #[allow(dead_code)]
    fn outer_loop(&self, _coeffs: &mut [i32; GRANULE_SIZE], _max_bits: usize, _info: &mut GranuleInfo) -> usize {
        // Implementation will be added in task 9.5
        todo!("Outer loop implementation")
    }
}

impl Default for QuantizationLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Set up custom panic hook to avoid verbose parameter output
    fn setup_panic_hook() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|_| {
                eprintln!("Test failed: Property test assertion failed");
            }));
        });
    }

    /// Strategy for generating valid MDCT coefficients
    fn mdct_coeffs_strategy() -> impl Strategy<Value = [i32; GRANULE_SIZE]> {
        prop::collection::vec(-32768i32..32768i32, GRANULE_SIZE)
            .prop_map(|v| {
                let mut arr = [0i32; GRANULE_SIZE];
                arr.copy_from_slice(&v);
                arr
            })
    }

    /// Strategy for generating valid step sizes
    fn step_size_strategy() -> impl Strategy<Value = i32> {
        -120i32..120i32
    }

    /// Strategy for generating target bit counts
    fn target_bits_strategy() -> impl Strategy<Value = usize> {
        100usize..10000usize
    }

    /// Strategy for generating perceptual entropy values
    fn perceptual_entropy_strategy() -> impl Strategy<Value = f64> {
        0.0f64..1000.0f64
    }

    /// Strategy for generating channel counts
    fn channels_strategy() -> impl Strategy<Value = usize> {
        1usize..=2usize
    }

    /// Strategy for generating mean bits per granule
    fn mean_bits_strategy() -> impl Strategy<Value = usize> {
        100usize..5000usize
    }

    // Feature: rust-mp3-encoder, Property 7: 量化和比特率控制
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_quantization_and_bitrate_control(
            mdct_coeffs in mdct_coeffs_strategy(),
            target_bits in target_bits_strategy()
        ) {
            setup_panic_hook();
            
            let mut quantizer = QuantizationLoop::new();
            let mut output = [0i32; GRANULE_SIZE];
            let mut side_info = GranuleInfo::default();
            
            // Test quantization process
            let result = quantizer.quantize_and_encode(&mdct_coeffs, target_bits, &mut side_info, &mut output);
            
            // Property 1: Quantization should succeed for valid inputs
            prop_assert!(result.is_ok(), "Quantization should succeed");
            
            // Property 2: Quantized values should be within valid range
            for &val in output.iter() {
                prop_assert!(val.abs() <= 8192, "Quantized values should be within range");
            }
            
            // Property 3: Global gain should be reasonable
            prop_assert!(side_info.global_gain >= 90, "Global gain too low");
            prop_assert!(side_info.global_gain <= 330, "Global gain too high");
            
            // Property 4: Big values count should be reasonable
            prop_assert!(side_info.big_values <= 288, "Big values count too high");
        }

        #[test]
        fn test_quantization_step_size_adjustment(
            mdct_coeffs in mdct_coeffs_strategy(),
            step_size in step_size_strategy()
        ) {
            setup_panic_hook();
            
            let quantizer = QuantizationLoop::new();
            let mut output1 = [0i32; GRANULE_SIZE];
            let mut output2 = [0i32; GRANULE_SIZE];
            
            // Test with two different step sizes
            let max1 = quantizer.quantize(&mdct_coeffs, step_size, &mut output1);
            let max2 = quantizer.quantize(&mdct_coeffs, step_size + 4, &mut output2);
            
            // Property: Larger step size should generally produce smaller quantized values
            if max1 > 0 && max2 > 0 {
                prop_assert!(max2 <= max1, "Larger step size should produce smaller values");
            }
            
            // Property: All quantized values should be within valid range
            prop_assert!(max1 <= 16384, "Max quantized value should be within range");
            prop_assert!(max2 <= 16384, "Max quantized value should be within range");
        }

        #[test]
        fn test_quantization_preserves_zero_coefficients(
            mdct_coeffs in mdct_coeffs_strategy(),
            step_size in step_size_strategy()
        ) {
            setup_panic_hook();
            
            let quantizer = QuantizationLoop::new();
            let mut output = [0i32; GRANULE_SIZE];
            
            quantizer.quantize(&mdct_coeffs, step_size, &mut output);
            
            // Property: Zero input coefficients should produce zero output
            for i in 0..GRANULE_SIZE {
                if mdct_coeffs[i] == 0 {
                    prop_assert_eq!(output[i], 0, "Zero coefficients should remain zero");
                }
            }
        }

        #[test]
        fn test_quantization_sign_preservation(
            mdct_coeffs in mdct_coeffs_strategy(),
            step_size in step_size_strategy()
        ) {
            setup_panic_hook();
            
            let quantizer = QuantizationLoop::new();
            let mut output = [0i32; GRANULE_SIZE];
            
            quantizer.quantize(&mdct_coeffs, step_size, &mut output);
            
            // Property: Sign of coefficients should be preserved
            for i in 0..GRANULE_SIZE {
                if mdct_coeffs[i] != 0 && output[i] != 0 {
                    prop_assert_eq!(
                        mdct_coeffs[i].signum(), 
                        output[i].signum(), 
                        "Sign should be preserved"
                    );
                }
            }
        }

        // Feature: rust-mp3-encoder, Property 8: 比特储备池机制
        #[test]
        fn test_bit_reservoir_integration(
            mean_bits in mean_bits_strategy(),
            channels in channels_strategy(),
            perceptual_entropy in perceptual_entropy_strategy()
        ) {
            setup_panic_hook();
            
            use crate::reservoir::BitReservoir;
            
            let mut reservoir = BitReservoir::new(7680);
            reservoir.frame_begin(mean_bits);
            
            // Property 1: Max reservoir bits should be reasonable
            let max_bits = reservoir.max_reservoir_bits(perceptual_entropy, channels);
            prop_assert!(max_bits <= 4095, "Max bits should not exceed 4095");
            prop_assert!(max_bits > 0, "Max bits should be positive");
            
            // Property 2: Frame operations should maintain consistency
            let initial_bits = reservoir.available_bits();
            let (stuffing_bits, _drain_bits) = reservoir.frame_end(channels);
            
            // Stuffing bits should be reasonable
            prop_assert!(stuffing_bits <= initial_bits + mean_bits, "Stuffing bits should be reasonable");
        }
    }

    #[cfg(test)]
    mod unit_tests {
        use super::*;

        #[test]
        fn test_quantization_loop_creation() {
            let quantizer = QuantizationLoop::new();
            
            // Test that tables are initialized
            assert_ne!(quantizer.step_table[0], 0.0);
            assert_ne!(quantizer.step_table_i32[0], 0);
            assert_ne!(quantizer.int2idx[100], 0);
        }

        #[test]
        fn test_quantization_with_zero_input() {
            let quantizer = QuantizationLoop::new();
            let zero_coeffs = [0i32; GRANULE_SIZE];
            let mut output = [0i32; GRANULE_SIZE];
            
            let max_val = quantizer.quantize(&zero_coeffs, 0, &mut output);
            
            assert_eq!(max_val, 0);
            assert!(output.iter().all(|&x| x == 0));
        }

        #[test]
        fn test_run_length_calculation() {
            let quantizer = QuantizationLoop::new();
            let mut side_info = GranuleInfo::default();
            
            // Test with some specific patterns
            let mut test_coeffs = [0i32; GRANULE_SIZE];
            test_coeffs[0] = 100;
            test_coeffs[1] = 50;
            test_coeffs[2] = 1;
            test_coeffs[3] = 1;
            
            quantizer.calculate_run_length(&test_coeffs, &mut side_info);
            
            assert!(side_info.big_values > 0);
        }

        #[test]
        fn test_step_size_calculation() {
            let quantizer = QuantizationLoop::new();
            let test_coeffs = [100i32; GRANULE_SIZE];
            
            let step_size = quantizer.calculate_step_size(&test_coeffs, 1000);
            
            // Should return a reasonable step size
            assert!(step_size >= -120);
            assert!(step_size <= 120);
        }
    }
}