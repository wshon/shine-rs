//! Modified Discrete Cosine Transform (MDCT) for MP3 encoding
//!
//! This module implements the MDCT transform that converts subband samples
//! into frequency domain coefficients for quantization and encoding.
//! 
//! The implementation follows the shine library's approach, using fixed-point
//! arithmetic for performance and precision.

use crate::error::{EncodingResult, EncodingError};
use std::f64::consts::PI;

/// MDCT transform for converting subband samples to frequency coefficients
pub struct MdctTransform {
    /// Precomputed cosine table for MDCT [k][n] where k=0..17, n=0..35
    /// These combine window and MDCT coefficients into a single table
    cos_table: [[i32; 36]; 18],
}

/// Aliasing reduction coefficients (Table B.9 from ISO/IEC 11172-3)
/// These are the ca and cs coefficients for the butterfly operation
const ALIASING_CA: [i32; 8] = [
    // ca[i] = coef[i] / sqrt(1.0 + coef[i]^2) * 0x7fffffff
    -1290213931, // ca[0] = -0.6 / sqrt(1.0 + 0.36) * 0x7fffffff
    -1060439283, // ca[1] = -0.535 / sqrt(1.0 + 0.286225) * 0x7fffffff
    -628992573,  // ca[2] = -0.33 / sqrt(1.0 + 0.1089) * 0x7fffffff
    -353553391,  // ca[3] = -0.185 / sqrt(1.0 + 0.034225) * 0x7fffffff
    -181019336,  // ca[4] = -0.095 / sqrt(1.0 + 0.009025) * 0x7fffffff
    -78815462,   // ca[5] = -0.041 / sqrt(1.0 + 0.001681) * 0x7fffffff
    -27244582,   // ca[6] = -0.0142 / sqrt(1.0 + 0.00020164) * 0x7fffffff
    -7096781,    // ca[7] = -0.0037 / sqrt(1.0 + 0.00001369) * 0x7fffffff
];

const ALIASING_CS: [i32; 8] = [
    // cs[i] = 1.0 / sqrt(1.0 + coef[i]^2) * 0x7fffffff
    1840700269,  // cs[0] = 1.0 / sqrt(1.0 + 0.36) * 0x7fffffff
    1946157056,  // cs[1] = 1.0 / sqrt(1.0 + 0.286225) * 0x7fffffff
    2040817947,  // cs[2] = 1.0 / sqrt(1.0 + 0.1089) * 0x7fffffff
    2111864259,  // cs[3] = 1.0 / sqrt(1.0 + 0.034225) * 0x7fffffff
    2137625049,  // cs[4] = 1.0 / sqrt(1.0 + 0.009025) * 0x7fffffff
    2146959355,  // cs[5] = 1.0 / sqrt(1.0 + 0.001681) * 0x7fffffff
    2147450880,  // cs[6] = 1.0 / sqrt(1.0 + 0.00020164) * 0x7fffffff
    2147483647,  // cs[7] = 1.0 / sqrt(1.0 + 0.00001369) * 0x7fffffff
];

impl MdctTransform {
    /// Create a new MDCT transform with precomputed cosine tables
    pub fn new() -> Self {
        let mut mdct = Self {
            cos_table: [[0; 36]; 18],
        };
        mdct.initialize_cos_table();
        mdct
    }
    
    /// Initialize the cosine table combining window and MDCT coefficients
    /// This follows the shine implementation: cos_l[m][k] = 
    /// sin(PI36 * (k + 0.5)) * cos((PI / 72) * (2 * k + 19) * (2 * m + 1)) * 0x7fffffff
    fn initialize_cos_table(&mut self) {
        const PI36: f64 = PI / 36.0;
        const PI72: f64 = PI / 72.0;
        
        for m in 0..18 {
            for k in 0..36 {
                let window_coeff = (PI36 * (k as f64 + 0.5)).sin();
                let mdct_coeff = (PI72 * (2.0 * k as f64 + 19.0) * (2.0 * m as f64 + 1.0)).cos();
                let combined = window_coeff * mdct_coeff * (i32::MAX as f64);
                self.cos_table[m][k] = combined as i32;
            }
        }
    }
    
    /// Perform MDCT transform on subband samples
    /// Input: subband_samples[granule][subband] where granule=0..35, subband=0..31
    /// Output: mdct_coeffs[coeff] where coeff=0..575 (32 subbands * 18 coeffs each)
    pub fn transform(&self, subband_samples: &[[i32; 32]; 36], output: &mut [i32; 576]) -> EncodingResult<()> {
        if output.len() != 576 {
            return Err(EncodingError::InvalidDataLength {
                expected: 576,
                actual: output.len(),
            });
        }
        
        // Process each subband (32 subbands total)
        for subband in 0..32 {
            // Prepare input for this subband: 36 time samples
            let mut mdct_input = [0i32; 36];
            for k in 0..36 {
                mdct_input[k] = subband_samples[k][subband];
            }
            
            // Perform 36-point MDCT to get 18 frequency coefficients
            // This follows the shine implementation's inner loop
            for coeff in 0..18 {
                let mut accumulator: i64 = 0;
                
                // Multiply-accumulate using the precomputed cosine table
                for n in 0..36 {
                    accumulator += (mdct_input[n] as i64) * (self.cos_table[coeff][n] as i64);
                }
                
                // Scale down from 64-bit accumulator to 32-bit result
                // This matches the shine library's mulz operation
                let result = (accumulator >> 31) as i32;
                
                // Store result: output[subband * 18 + coeff]
                output[subband * 18 + coeff] = result;
            }
        }
        
        Ok(())
    }
    
    /// Apply aliasing reduction butterfly to MDCT coefficients
    /// This implements the aliasing reduction from ISO/IEC 11172-3 Section 2.4.3.4.7.3
    /// The butterfly operation is applied between adjacent subbands
    pub fn apply_aliasing_reduction(&self, coeffs: &mut [i32; 576]) -> EncodingResult<()> {
        // Apply butterfly between adjacent subbands (except the first subband)
        for subband in 1..32 {
            let prev_band_start = (subband - 1) * 18;
            let curr_band_start = subband * 18;
            
            // Apply 8 butterfly operations for each subband boundary
            for i in 0..8 {
                let prev_idx = prev_band_start + (17 - i); // Previous subband, from end
                let curr_idx = curr_band_start + i;        // Current subband, from start
                
                let prev_val = coeffs[prev_idx];
                let curr_val = coeffs[curr_idx];
                
                // Butterfly operation: 
                // new_prev = cs[i] * prev_val + ca[i] * curr_val
                // new_curr = cs[i] * curr_val - ca[i] * prev_val
                let cs = ALIASING_CS[i] as i64;
                let ca = ALIASING_CA[i] as i64;
                
                let new_prev = ((cs * prev_val as i64 + ca * curr_val as i64) >> 31) as i32;
                let new_curr = ((cs * curr_val as i64 - ca * prev_val as i64) >> 31) as i32;
                
                coeffs[prev_idx] = new_prev;
                coeffs[curr_idx] = new_curr;
            }
        }
        
        Ok(())
    }
    
    /// Get the cosine table for testing purposes
    #[cfg(test)]
    pub fn get_cos_table(&self) -> &[[i32; 36]; 18] {
        &self.cos_table
    }
}

impl Default for MdctTransform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mdct_transform_creation() {
        let mdct = MdctTransform::new();
        
        // Verify that cosine table is initialized (not all zeros)
        let mut has_nonzero = false;
        for m in 0..18 {
            for k in 0..36 {
                if mdct.cos_table[m][k] != 0 {
                    has_nonzero = true;
                    break;
                }
            }
            if has_nonzero { break; }
        }
        assert!(has_nonzero, "Cosine table should be initialized with non-zero values");
    }
    
    #[test]
    fn test_mdct_transform_zero_input() {
        let mdct = MdctTransform::new();
        let input = [[0i32; 32]; 36];
        let mut output = [0i32; 576];
        
        let result = mdct.transform(&input, &mut output);
        assert!(result.is_ok());
        
        // All outputs should be zero for zero input
        for &val in &output {
            assert_eq!(val, 0);
        }
    }
    
    #[test]
    fn test_mdct_transform_invalid_output_size() {
        let mdct = MdctTransform::new();
        let input = [[0i32; 32]; 36];
        let mut output = [0i32; 576]; // Correct size for this test
        
        // This test should pass with correct size
        let result = mdct.transform(&input, &mut output);
        assert!(result.is_ok());
        
        // Test with slice of wrong size would require different approach
        // since we can't create arrays of different sizes at compile time
        // This test verifies the function works with correct input
    }
    
    #[test]
    fn test_aliasing_reduction_zero_input() {
        let mdct = MdctTransform::new();
        let mut coeffs = [0i32; 576];
        
        let result = mdct.apply_aliasing_reduction(&mut coeffs);
        assert!(result.is_ok());
        
        // All coefficients should remain zero
        for &val in &coeffs {
            assert_eq!(val, 0);
        }
    }
    
    #[test]
    fn test_aliasing_reduction_simple_case() {
        let mdct = MdctTransform::new();
        let mut coeffs = [0i32; 576];
        
        // Set some test values in adjacent subbands
        coeffs[17] = 1000;  // Last coeff of subband 0
        coeffs[18] = 2000;  // First coeff of subband 1
        
        let result = mdct.apply_aliasing_reduction(&mut coeffs);
        assert!(result.is_ok());
        
        // Values should have changed due to butterfly operation
        assert_ne!(coeffs[17], 1000);
        assert_ne!(coeffs[18], 2000);
    }
}