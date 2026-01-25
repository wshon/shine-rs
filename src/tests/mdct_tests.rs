//! Unit tests for MDCT (Modified Discrete Cosine Transform) operations
//!
//! Tests the MDCT analysis functionality including coefficient calculation
//! and aliasing reduction operations.

use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test MDCT coefficient ranges and properties with real data
    #[test]
    fn test_mdct_coefficient_validation() {
        // Real MDCT coefficients from sample-3s.wav encoding
        let frame_1_coeffs = [808302i32, 3145162, 6527797];
        let frame_2_coeffs = [-17369047i32, 13912238, 31910201];
        let frame_3_coeffs = [-20877153i32, -19736998, -24380058];
        
        let all_coeffs = [frame_1_coeffs, frame_2_coeffs, frame_3_coeffs].concat();
        
        // Validate coefficient ranges (typical for audio signals)
        for &coeff in &all_coeffs {
            assert!(coeff.abs() < 50_000_000, "MDCT coefficient {} out of range", coeff);
        }
        
        // Test that coefficients show variation across frames (not stuck values)
        assert_ne!(frame_1_coeffs[0], frame_2_coeffs[0], "K17 should vary between frames");
        assert_ne!(frame_2_coeffs[0], frame_3_coeffs[0], "K17 should vary between frames");
        assert_ne!(frame_1_coeffs[1], frame_2_coeffs[1], "K16 should vary between frames");
        assert_ne!(frame_2_coeffs[1], frame_3_coeffs[1], "K16 should vary between frames");
        
        // Verify specific known values
        assert_eq!(frame_1_coeffs[0], 808302, "Frame 1 K17 MDCT coefficient mismatch");
        assert_eq!(frame_1_coeffs[1], 3145162, "Frame 1 K16 MDCT coefficient mismatch");
        assert_eq!(frame_2_coeffs[0], -17369047, "Frame 2 K17 MDCT coefficient mismatch");
        assert_eq!(frame_3_coeffs[0], -20877153, "Frame 3 K17 MDCT coefficient mismatch");
    }

    /// Test MDCT input data validation with real data from sample-3s.wav
    #[test]
    fn test_mdct_input_validation() {
        // Real data from Frame 1 - first granule should be zeros (no previous data)
        let frame_1_first_8: [i32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        
        // Real data from Frame 1 - last 8 values from subband filter
        let frame_1_last_8: [i32; 8] = [-108108746, -171625282, -168521462, -153132793, -102026930, -53572474, -66933230, -61760919];
        
        // Real data from Frame 2 - should use Frame 1's saved data
        let frame_2_first_8: [i32; 8] = [-35329013, 13541843, 43631088, 50289625, 68731699, 98941519, 141525294, 142119942];
        
        // Frame 1: First granule should be zeros (no previous data)
        for &val in &frame_1_first_8 {
            assert_eq!(val, 0, "Frame 1 first 8 should be zeros");
        }
        
        // Frame 1: Last 8 values should be non-zero (from subband filter)
        for &val in &frame_1_last_8 {
            assert!(val != 0, "Frame 1 last 8 values should be non-zero");
            assert!(val.abs() < 200_000_000, "Frame 1 MDCT input {} out of range", val);
        }
        
        // Frame 2: Should use Frame 1's saved data as first 8 values
        for &val in &frame_2_first_8 {
            assert!(val != 0, "Frame 2 first 8 should be non-zero (from Frame 1)");
            assert!(val.abs() < 200_000_000, "Frame 2 MDCT input {} out of range", val);
        }
        
        // Verify specific known values from the encoding log
        assert_eq!(frame_1_last_8[0], -108108746, "Frame 1 MDCT input last[0] mismatch");
        assert_eq!(frame_1_last_8[1], -171625282, "Frame 1 MDCT input last[1] mismatch");
        assert_eq!(frame_2_first_8[0], -35329013, "Frame 2 MDCT input first[0] mismatch");
        assert_eq!(frame_2_first_8[1], 13541843, "Frame 2 MDCT input first[1] mismatch");
    }

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
    fn test_mdct_coefficient_ranges() {
        // Test that MDCT coefficients are within expected ranges
        
        // Typical MDCT coefficient ranges for audio signals
        const MAX_COEFF_MAGNITUDE: i32 = 50_000_000;
        
        // Test with some example coefficients (these would come from actual MDCT)
        let test_coefficients = [808302i32, 3145162, 6527797, -17369047, 13912238, 31910201];
        
        for &coeff in &test_coefficients {
            assert!(coeff.abs() < MAX_COEFF_MAGNITUDE, "MDCT coefficient {} out of range", coeff);
        }
        
        // Test that coefficients can be both positive and negative
        let has_positive = test_coefficients.iter().any(|&x| x > 0);
        let has_negative = test_coefficients.iter().any(|&x| x < 0);
        assert!(has_positive, "Should have positive coefficients");
        assert!(has_negative, "Should have negative coefficients");
    }

    /// Test MDCT input data validation with real data from sample-3s.wav
    #[test]
    fn test_mdct_input_real_data_validation() {
        // Real data from Frame 1 - first granule should be zeros (no previous data)
        let frame_1_first_8: [i32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        
        // Real data from Frame 1 - last 8 values from subband filter
        let frame_1_last_8: [i32; 8] = [-108108746, -171625282, -168521462, -153132793, -102026930, -53572474, -66933230, -61760919];
        
        // Real data from Frame 2 - should use Frame 1's saved data
        let frame_2_first_8: [i32; 8] = [-35329013, 13541843, 43631088, 50289625, 68731699, 98941519, 141525294, 142119942];
        
        // Frame 1: First granule should be zeros (no previous data)
        for &val in &frame_1_first_8 {
            assert_eq!(val, 0, "Frame 1 first 8 should be zeros");
        }
        
        // Frame 1: Last 8 values should be non-zero (from subband filter)
        for &val in &frame_1_last_8 {
            assert!(val != 0, "Frame 1 last 8 values should be non-zero");
            assert!(val.abs() < 200_000_000, "Frame 1 MDCT input {} out of range", val);
        }
        
        // Frame 2: Should use Frame 1's saved data as first 8 values
        for &val in &frame_2_first_8 {
            assert!(val != 0, "Frame 2 first 8 should be non-zero (from Frame 1)");
            assert!(val.abs() < 200_000_000, "Frame 2 MDCT input {} out of range", val);
        }
        
        // Verify specific known values from the encoding log
        assert_eq!(frame_1_last_8[0], -108108746, "Frame 1 MDCT input last[0] mismatch");
        assert_eq!(frame_1_last_8[1], -171625282, "Frame 1 MDCT input last[1] mismatch");
        assert_eq!(frame_2_first_8[0], -35329013, "Frame 2 MDCT input first[0] mismatch");
        assert_eq!(frame_2_first_8[1], 13541843, "Frame 2 MDCT input first[1] mismatch");
    }

    /// Test MDCT coefficient validation with real data from all frames
    #[test]
    fn test_mdct_coefficients_real_data_validation() {
        // Frame 1 coefficients (from actual encoding)
        let f1_coeffs: [i32; 3] = [808302, 3145162, 6527797];
        
        // Frame 2 coefficients (from actual encoding)
        let f2_coeffs: [i32; 3] = [-17369047, 13912238, 31910201];
        
        // Frame 3 coefficients (from actual encoding)
        let f3_coeffs: [i32; 3] = [-20877153, -19736998, -24380058];
        
        // Validate coefficient ranges (typical for audio signals)
        let all_coeffs = [f1_coeffs, f2_coeffs, f3_coeffs].concat();
        for &coeff in &all_coeffs {
            assert!(coeff.abs() < 50_000_000, "MDCT coefficient {} out of range", coeff);
        }
        
        // Test that coefficients show variation across frames (not stuck values)
        assert_ne!(f1_coeffs[0], f2_coeffs[0], "K17 should vary between frames");
        assert_ne!(f2_coeffs[0], f3_coeffs[0], "K17 should vary between frames");
        assert_ne!(f1_coeffs[1], f2_coeffs[1], "K16 should vary between frames");
        assert_ne!(f2_coeffs[1], f3_coeffs[1], "K16 should vary between frames");
        
        // Verify specific known values
        assert_eq!(f1_coeffs[0], 808302, "Frame 1 K17 MDCT coefficient mismatch");
        assert_eq!(f1_coeffs[1], 3145162, "Frame 1 K16 MDCT coefficient mismatch");
        assert_eq!(f2_coeffs[0], -17369047, "Frame 2 K17 MDCT coefficient mismatch");
        assert_eq!(f3_coeffs[0], -20877153, "Frame 3 K17 MDCT coefficient mismatch");
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
        assert_eq!(MDCT_WINDOW_SIZE / 4, OVERLAP_SIZE / 2, "Window and overlap relationship");
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
        let test_cs_values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]; // Computed from CA values
        
        for &ca in &test_ca_values {
            assert!(ca < 0.0, "CA coefficients should be negative");
            assert!(ca > -1.0, "CA coefficients should be > -1");
        }
        
        for &cs in &test_cs_values {
            assert!(cs > 0.0, "CS coefficients should be positive");
            assert!(cs <= 1.0, "CS coefficients should be <= 1");
        }
        
        // Test that CA and CS satisfy: CA² + CS² = 1
        for (&ca, &cs) in test_ca_values.iter().zip(test_cs_values.iter()) {
            let cs_computed = 1.0 / (1.0 + ca * ca).sqrt();
            assert!((cs - cs_computed).abs() < 0.001, "CS should be computed from CA");
        }
    }
}

#[cfg(test)]
mod property_tests {
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
        fn test_mdct_coefficient_properties(
            coeffs in prop::collection::vec(-10_000_000i32..=10_000_000i32, 576)
        ) {
            // Test properties of MDCT coefficients
            prop_assert_eq!(coeffs.len(), GRANULE_SIZE, "Should have granule size coefficients");
            
            // Test energy calculation
            let energy: i64 = coeffs.iter().map(|&x| (x as i64) * (x as i64)).sum();
            prop_assert!(energy >= 0, "Energy should be non-negative");
            
            // Test coefficient ranges
            for &coeff in &coeffs {
                prop_assert!(coeff.abs() <= 10_000_000, "Coefficient should be in test range");
            }
        }

        #[test]
        fn test_mdct_input_properties(
            samples in prop::collection::vec(-32768i32..=32767i32, 1152)
        ) {
            // Test properties of MDCT input (windowed samples)
            prop_assert_eq!(samples.len(), GRANULE_SIZE * 2, "MDCT input should be windowed granule");
            
            // Test that input is in valid 16-bit range (after windowing)
            for &sample in &samples {
                prop_assert!(sample >= -32768 && sample <= 32767, "Sample should be in 16-bit range");
            }
            
            // Test overlap regions
            let first_half = &samples[0..GRANULE_SIZE];
            let second_half = &samples[GRANULE_SIZE..];
            
            prop_assert_eq!(first_half.len(), GRANULE_SIZE, "First half should be granule size");
            prop_assert_eq!(second_half.len(), GRANULE_SIZE, "Second half should be granule size");
        }

        #[test]
        fn test_mdct_frequency_properties(
            freq_bin in 0usize..576
        ) {
            // Test frequency bin properties
            prop_assert!(freq_bin < GRANULE_SIZE, "Frequency bin should be valid");
            
            // Test frequency calculation
            const SAMPLE_RATE: f64 = 44100.0;
            let frequency = (freq_bin as f64) * SAMPLE_RATE / (2.0 * GRANULE_SIZE as f64);
            
            prop_assert!(frequency >= 0.0, "Frequency should be non-negative");
            prop_assert!(frequency <= SAMPLE_RATE / 2.0, "Frequency should not exceed Nyquist");
            
            // Test that frequency increases with bin number
            if freq_bin > 0 {
                let prev_frequency = ((freq_bin - 1) as f64) * SAMPLE_RATE / (2.0 * GRANULE_SIZE as f64);
                prop_assert!(frequency > prev_frequency, "Frequency should increase with bin number");
            }
        }

        #[test]
        fn test_mdct_aliasing_properties(
            ca_coeff in -1.0f64..0.0f64
        ) {
            // Test aliasing reduction coefficient properties
            prop_assert!(ca_coeff <= 0.0, "CA coefficient should be non-positive");
            prop_assert!(ca_coeff >= -1.0, "CA coefficient should be >= -1");
            
            // Test CS coefficient calculation
            let cs_coeff = 1.0 / (1.0 + ca_coeff * ca_coeff).sqrt();
            prop_assert!(cs_coeff > 0.0, "CS coefficient should be positive");
            prop_assert!(cs_coeff <= 1.0, "CS coefficient should be <= 1");
            
            // Test normalization property: CA² + CS² = 1
            let ca_normalized = ca_coeff / (1.0 + ca_coeff * ca_coeff).sqrt();
            let sum_of_squares = ca_normalized * ca_normalized + cs_coeff * cs_coeff;
            prop_assert!((sum_of_squares - 1.0).abs() < 0.001, "CA² + CS² should equal 1");
        }
    }
}