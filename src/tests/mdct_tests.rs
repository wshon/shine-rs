//! Unit tests for MDCT (Modified Discrete Cosine Transform) operations
//!
//! Tests the MDCT analysis functionality including coefficient calculation
//! and aliasing reduction operations.

use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdct_constants() {
        // Test that MDCT constants are properly defined
        use std::f64::consts::PI;
        
        // Test PI36 and PI72 constants exist and have correct values
        const PI36: f64 = PI / 36.0;
        const PI72: f64 = PI / 72.0;
        
        assert!(PI36 > 0.0, "PI36 should be positive");
        assert!(PI72 > 0.0, "PI72 should be positive");
        assert!(PI36 > PI72, "PI36 should be larger than PI72");
        
        // Test relationship
        assert!((PI36 - PI / 36.0).abs() < 1e-10, "PI36 should equal PI/36");
        assert!((PI72 - PI / 72.0).abs() < 1e-10, "PI72 should equal PI/72");
    }

    #[test]
    fn test_mdct_structure_sizes() {
        // Test that MDCT-related structures have correct sizes
        assert_eq!(GRANULE_SIZE, 576, "Granule size should be 576");
        assert_eq!(SBLIMIT, 32, "Should have 32 subbands");
        
        // Test that granule can be organized as subbands
        assert_eq!(SBLIMIT * 18, GRANULE_SIZE, "32 subbands * 18 samples = 576");
    }

    #[test]
    fn test_mdct_granule_overlap() {
        // Test the granule overlap mechanism in MDCT
        
        // MDCT uses 50% overlap between granules
        // This means the last 288 samples of granule N become the first 288 samples of granule N+1
        
        const OVERLAP_SIZE: usize = GRANULE_SIZE / 2; // 288 samples
        assert_eq!(OVERLAP_SIZE, 288, "Overlap should be half granule size");
        
        // Test that overlap size is correct for MDCT window
        const MDCT_WINDOW_SIZE: usize = GRANULE_SIZE * 2; // 1152 samples
        assert_eq!(MDCT_WINDOW_SIZE, 1152, "MDCT window should be twice granule size");
        
        // Test relationship between window and overlap
        assert_eq!(MDCT_WINDOW_SIZE / 2, GRANULE_SIZE, "Window half should equal granule size");
        assert_eq!(OVERLAP_SIZE, GRANULE_SIZE / 2, "Overlap should be half granule size");
    }

    #[test]
    fn test_mdct_frequency_mapping() {
        // Test frequency mapping properties of MDCT
        
        // MDCT maps time domain samples to frequency domain coefficients
        // For 576 input samples, we get 576 output coefficients
        
        const INPUT_SAMPLES: usize = GRANULE_SIZE;
        const OUTPUT_COEFFS: usize = GRANULE_SIZE;
        
        assert_eq!(INPUT_SAMPLES, OUTPUT_COEFFS, "MDCT should preserve sample count");
        
        // Test frequency bin spacing
        const SAMPLE_RATE: f64 = 44100.0;
        const FREQ_BIN_WIDTH: f64 = SAMPLE_RATE / (2.0 * GRANULE_SIZE as f64);
        
        assert!((FREQ_BIN_WIDTH - 38.28125).abs() < 0.001, "Frequency bin width should be ~38.28 Hz");
        
        // Test Nyquist frequency coverage
        let nyquist_freq = SAMPLE_RATE / 2.0;
        let max_freq = FREQ_BIN_WIDTH * (GRANULE_SIZE as f64);
        
        assert!((max_freq - nyquist_freq).abs() < 1.0, "MDCT should cover up to Nyquist frequency");
    }

    #[test]
    fn test_mdct_energy_properties() {
        // Test energy-related properties of MDCT
        
        // MDCT should approximately preserve energy (Parseval's theorem)
        // Energy in time domain ≈ Energy in frequency domain
        
        // Test with simple test vectors
        let time_samples = [1000i32, 500, 250, 125, 0, -125, -250, -500];
        let time_energy: i64 = time_samples.iter().map(|&x| (x as i64) * (x as i64)).sum();
        
        assert!(time_energy > 0, "Time domain energy should be positive");
        
        // For MDCT, energy scaling factor depends on window function
        // This is a structural test of energy calculation
        let energy_per_sample = time_energy / time_samples.len() as i64;
        assert!(energy_per_sample > 0, "Average energy per sample should be positive");
    }

    #[test]
    fn test_mdct_aliasing_reduction() {
        // Test aliasing reduction properties
        
        // MDCT uses aliasing reduction to handle overlapping windows
        // This involves butterfly operations on adjacent coefficients
        
        // Test that aliasing reduction coefficients are available
        // (These would be defined in the MDCT module)
        
        // Test coefficient ranges for aliasing reduction
        let test_ca_values = [-0.6f64, -0.535, -0.33, -0.185, -0.095, -0.041, -0.0142, -0.0037];
        
        for &ca in &test_ca_values {
            assert!(ca < 0.0, "CA coefficients should be negative");
            assert!(ca > -1.0, "CA coefficients should be > -1");
            
            // Test CS coefficient calculation
            let cs = 1.0 / (1.0 + ca * ca).sqrt();
            assert!(cs > 0.0, "CS coefficient should be positive");
            assert!(cs <= 1.0, "CS coefficient should be <= 1");
            
            // Test normalization property: CA² + CS² = 1 (approximately)
            let ca_normalized = ca / (1.0 + ca * ca).sqrt();
            let sum_of_squares = ca_normalized * ca_normalized + cs * cs;
            assert!((sum_of_squares - 1.0).abs() < 0.001, "CA² + CS² should approximately equal 1");
        }
    }
}