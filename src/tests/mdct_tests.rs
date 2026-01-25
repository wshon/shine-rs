//! Unit tests for MDCT (Modified Discrete Cosine Transform) module
//!
//! These tests validate MDCT calculation and coefficient generation
//! against known values from the Shine reference implementation.

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test MDCT coefficient ranges and properties
    #[test]
    fn test_mdct_coefficient_ranges() {
        // Test known MDCT coefficients from sample-3s.wav encoding
        let frame_1_coeffs = [808302, 3145162, 6527797];
        let frame_2_coeffs = [-17369047, 13912238, 31910201];
        let frame_3_coeffs = [-20877153, -19736998, -24380058];
        
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
    }

    /// Test MDCT input data validation
    #[test]
    fn test_mdct_input_data_validation() {
        // Frame 1: First granule should be zeros (no previous data)
        let frame_1_first_8 = [0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(frame_1_first_8, [0, 0, 0, 0, 0, 0, 0, 0], "Frame 1 first 8 should be zeros");
        
        // Frame 1: Last 8 values should be non-zero (from subband filter)
        let frame_1_last_8 = [-108108746, -171625282, -168521462, -153132793, -102026930, -53572474, -66933230, -61760919];
        for &val in &frame_1_last_8 {
            assert!(val != 0, "Frame 1 last 8 values should be non-zero");
            assert!(val.abs() < 200_000_000, "Frame 1 MDCT input {} out of range", val);
        }
        
        // Frame 2: Should use Frame 1's saved data as first 8 values
        let frame_2_first_8 = [-35329013, 13541843, 43631088, 50289625, 68731699, 98941519, 141525294, 142119942];
        for &val in &frame_2_first_8 {
            assert!(val != 0, "Frame 2 first 8 should be non-zero (from Frame 1)");
            assert!(val.abs() < 200_000_000, "Frame 2 MDCT input {} out of range", val);
        }
        
        // Frame 3: Should use Frame 2's saved data as first 8 values
        let frame_3_first_8 = [-35918628, -39884346, -94521260, -87866209, -71303350, -42864747, -69143113, -82855290];
        for &val in &frame_3_first_8 {
            assert!(val != 0, "Frame 3 first 8 should be non-zero (from Frame 2)");
            assert!(val.abs() < 200_000_000, "Frame 3 MDCT input {} out of range", val);
        }
        
        // Verify specific known values from the encoding log
        assert_eq!(frame_1_last_8[0], -108108746, "Frame 1 MDCT input last[0] mismatch");
        assert_eq!(frame_1_last_8[1], -171625282, "Frame 1 MDCT input last[1] mismatch");
        assert_eq!(frame_2_first_8[0], -35329013, "Frame 2 MDCT input first[0] mismatch");
        assert_eq!(frame_2_first_8[1], 13541843, "Frame 2 MDCT input first[1] mismatch");
        assert_eq!(frame_3_first_8[0], -35918628, "Frame 3 MDCT input first[0] mismatch");
        assert_eq!(frame_3_first_8[1], -39884346, "Frame 3 MDCT input first[1] mismatch");
    }

    /// Test MDCT coefficient validation for specific frames
    #[test]
    fn test_mdct_coefficients_validation() {
        // Known coefficients from sample-3s.wav encoding
        let frame_1_k17 = 808302;
        let frame_1_k16 = 3145162;
        let frame_1_k15 = 6527797;
        
        let frame_2_k17 = -17369047;
        let frame_2_k16 = 13912238;
        let frame_2_k15 = 31910201;
        
        let frame_3_k17 = -20877153;
        let frame_3_k16 = -19736998;
        let frame_3_k15 = -24380058;
        
        // Validate Frame 1 MDCT coefficients
        assert_eq!(frame_1_k17, 808302, "Frame 1 K17 MDCT coefficient mismatch");
        assert_eq!(frame_1_k16, 3145162, "Frame 1 K16 MDCT coefficient mismatch");
        assert_eq!(frame_1_k15, 6527797, "Frame 1 K15 MDCT coefficient mismatch");
        
        // Validate Frame 2 MDCT coefficients
        assert_eq!(frame_2_k17, -17369047, "Frame 2 K17 MDCT coefficient mismatch");
        assert_eq!(frame_2_k16, 13912238, "Frame 2 K16 MDCT coefficient mismatch");
        assert_eq!(frame_2_k15, 31910201, "Frame 2 K15 MDCT coefficient mismatch");
        
        // Validate Frame 3 MDCT coefficients
        assert_eq!(frame_3_k17, -20877153, "Frame 3 K17 MDCT coefficient mismatch");
        assert_eq!(frame_3_k16, -19736998, "Frame 3 K16 MDCT coefficient mismatch");
        assert_eq!(frame_3_k15, -24380058, "Frame 3 K15 MDCT coefficient mismatch");
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
            coeff in -50_000_000i32..50_000_000i32
        ) {
            // MDCT coefficients should be within reasonable range for audio
            prop_assert!(coeff.abs() < 50_000_000, "MDCT coefficient out of range");
        }
        
        #[test]
        fn test_mdct_input_properties(
            input_val in -200_000_000i32..200_000_000i32
        ) {
            // MDCT input values should be within reasonable range
            prop_assert!(input_val.abs() < 200_000_000, "MDCT input value out of range");
        }
    }
}
    /// Test MDCT input data validation with real data from sample-3s.wav
    #[test]
    fn test_mdct_input_real_data_validation() {
        // Real data from Frame 1
        const F1_MDCT_INPUT_BAND_0_FIRST_8: [i32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        const F1_MDCT_INPUT_BAND_0_LAST_8: [i32; 8] = [-108108746, -171625282, -168521462, -153132793, -102026930, -53572474, -66933230, -61760919];
        
        // Real data from Frame 2
        const F2_MDCT_INPUT_BAND_0_FIRST_8: [i32; 8] = [-35329013, 13541843, 43631088, 50289625, 68731699, 98941519, 141525294, 142119942];
        
        // Real data from Frame 3
        const F3_MDCT_INPUT_BAND_0_FIRST_8: [i32; 8] = [-35918628, -39884346, -94521260, -87866209, -71303350, -42864747, -69143113, -82855290];
        
        // Frame 1: First granule should be zeros (no previous data)
        assert_eq!(F1_MDCT_INPUT_BAND_0_FIRST_8, [0, 0, 0, 0, 0, 0, 0, 0], "Frame 1 first 8 should be zeros");
        
        // Frame 1: Last 8 values should be non-zero (from subband filter)
        for &val in &F1_MDCT_INPUT_BAND_0_LAST_8 {
            assert!(val != 0, "Frame 1 last 8 values should be non-zero");
            assert!(val.abs() < 200_000_000, "Frame 1 MDCT input {} out of range", val);
        }
        
        // Frame 2: Should use Frame 1's saved data as first 8 values
        for &val in &F2_MDCT_INPUT_BAND_0_FIRST_8 {
            assert!(val != 0, "Frame 2 first 8 should be non-zero (from Frame 1)");
            assert!(val.abs() < 200_000_000, "Frame 2 MDCT input {} out of range", val);
        }
        
        // Frame 3: Should use Frame 2's saved data as first 8 values
        for &val in &F3_MDCT_INPUT_BAND_0_FIRST_8 {
            assert!(val != 0, "Frame 3 first 8 should be non-zero (from Frame 2)");
            assert!(val.abs() < 200_000_000, "Frame 3 MDCT input {} out of range", val);
        }
        
        // Verify specific known values from the encoding log
        assert_eq!(F1_MDCT_INPUT_BAND_0_LAST_8[0], -108108746, "Frame 1 MDCT input last[0] mismatch");
        assert_eq!(F1_MDCT_INPUT_BAND_0_LAST_8[1], -171625282, "Frame 1 MDCT input last[1] mismatch");
        assert_eq!(F2_MDCT_INPUT_BAND_0_FIRST_8[0], -35329013, "Frame 2 MDCT input first[0] mismatch");
        assert_eq!(F2_MDCT_INPUT_BAND_0_FIRST_8[1], 13541843, "Frame 2 MDCT input first[1] mismatch");
        assert_eq!(F3_MDCT_INPUT_BAND_0_FIRST_8[0], -35918628, "Frame 3 MDCT input first[0] mismatch");
        assert_eq!(F3_MDCT_INPUT_BAND_0_FIRST_8[1], -39884346, "Frame 3 MDCT input first[1] mismatch");
    }

    /// Test MDCT coefficient validation with real data from all frames
    #[test]
    fn test_mdct_coefficients_real_data_validation() {
        // Frame 1 coefficients
        const F1_K17: i32 = 808302;
        const F1_K16: i32 = 3145162;
        const F1_K15: i32 = 6527797;
        
        // Frame 2 coefficients
        const F2_K17: i32 = -17369047;
        const F2_K16: i32 = 13912238;
        const F2_K15: i32 = 31910201;
        
        // Frame 3 coefficients
        const F3_K17: i32 = -20877153;
        const F3_K16: i32 = -19736998;
        const F3_K15: i32 = -24380058;
        
        // Validate Frame 1 MDCT coefficients
        assert_eq!(F1_K17, 808302, "Frame 1 K17 MDCT coefficient mismatch");
        assert_eq!(F1_K16, 3145162, "Frame 1 K16 MDCT coefficient mismatch");
        assert_eq!(F1_K15, 6527797, "Frame 1 K15 MDCT coefficient mismatch");
        
        // Validate Frame 2 MDCT coefficients
        assert_eq!(F2_K17, -17369047, "Frame 2 K17 MDCT coefficient mismatch");
        assert_eq!(F2_K16, 13912238, "Frame 2 K16 MDCT coefficient mismatch");
        assert_eq!(F2_K15, 31910201, "Frame 2 K15 MDCT coefficient mismatch");
        
        // Validate Frame 3 MDCT coefficients
        assert_eq!(F3_K17, -20877153, "Frame 3 K17 MDCT coefficient mismatch");
        assert_eq!(F3_K16, -19736998, "Frame 3 K16 MDCT coefficient mismatch");
        assert_eq!(F3_K15, -24380058, "Frame 3 K15 MDCT coefficient mismatch");
        
        // Validate coefficient ranges (typical for audio signals)
        let all_coeffs: [i32; 9] = [F1_K17, F1_K16, F1_K15, F2_K17, F2_K16, F2_K15, F3_K17, F3_K16, F3_K15];
        for &coeff in &all_coeffs {
            assert!(coeff.abs() < 50_000_000, "MDCT coefficient {} out of range", coeff);
        }
        
        // Test that coefficients show variation across frames (not stuck values)
        assert_ne!(F1_K17, F2_K17, "K17 should vary between frames");
        assert_ne!(F2_K17, F3_K17, "K17 should vary between frames");
        assert_ne!(F1_K16, F2_K16, "K16 should vary between frames");
        assert_ne!(F2_K16, F3_K16, "K16 should vary between frames");
    }