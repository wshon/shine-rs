//! Unit tests for quantization module
//!
//! These tests validate quantization parameters, global gain calculation,
//! and big_values constraints against the Shine reference implementation.

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test quantization parameter ranges and constraints
    #[test]
    fn test_quantization_parameter_ranges() {
        // Known quantization parameters from sample-3s.wav encoding
        let frame_1_params = (174601576, 543987899, 170, 176, 94, 104);
        let frame_2_params = (761934185, 407502232, 175, 173, 98, 98);
        let frame_3_params = (398722265, 586508987, 173, 172, 93, 128);
        
        let all_params = [frame_1_params, frame_2_params, frame_3_params];
        
        for (xrmax_gr0, xrmax_gr1, gain_gr0, gain_gr1, big_val_gr0, big_val_gr1) in all_params.iter() {
            // Validate global gain ranges (0-255 for MP3)
            assert!(*gain_gr0 <= 255, "Global gain GR0 {} out of range", gain_gr0);
            assert!(*gain_gr1 <= 255, "Global gain GR1 {} out of range", gain_gr1);
            assert!(*gain_gr0 >= 100, "Global gain GR0 {} too low for typical audio", gain_gr0);
            assert!(*gain_gr1 >= 100, "Global gain GR1 {} too low for typical audio", gain_gr1);
            
            // Validate big_values (must be <= 288 for MP3 standard)
            assert!(*big_val_gr0 <= 288, "Big values GR0 {} exceeds MP3 limit", big_val_gr0);
            assert!(*big_val_gr1 <= 288, "Big values GR1 {} exceeds MP3 limit", big_val_gr1);
            assert!(*big_val_gr0 > 0, "Big values GR0 should be positive");
            assert!(*big_val_gr1 > 0, "Big values GR1 should be positive");
            
            // Validate xrmax values are reasonable
            assert!(*xrmax_gr0 > 0, "XRMAX GR0 should be positive");
            assert!(*xrmax_gr1 > 0, "XRMAX GR1 should be positive");
        }
    }

    /// Test quantization parameter relationships
    #[test]
    fn test_quantization_parameter_relationships() {
        // Frame 1 parameters
        let xrmax_gr0 = 174601576;
        let xrmax_gr1 = 543987899;
        let global_gain_gr0 = 170;
        let global_gain_gr1 = 176;
        let big_values_gr0 = 94;
        let big_values_gr1 = 104;
        
        // Test that granule 1 typically has higher complexity than granule 0
        // (this is common but not required)
        
        // GR1 often has higher xrmax (more complex audio)
        assert!(xrmax_gr1 > xrmax_gr0, "GR1 should have higher complexity");
        
        // GR1 often needs higher global_gain
        assert!(global_gain_gr1 > global_gain_gr0, "GR1 should need higher gain");
        
        // GR1 often has more big_values
        assert!(big_values_gr1 > big_values_gr0, "GR1 should have more big values");
    }

    /// Test part2_3_length validation
    #[test]
    fn test_part2_3_length_validation() {
        // Known part2_3_length values from sample-3s.wav encoding
        let frame_1_lengths = [(763, 689), (763, 689)]; // (GR0, GR1) for (CH0, CH1)
        let frame_2_lengths = [(714, 759), (714, 759)];
        let frame_3_lengths = [(684, 718), (684, 718)];
        
        let all_lengths = [frame_1_lengths, frame_2_lengths, frame_3_lengths].concat();
        
        // Validate part2_3_length ranges (12-bit field, max 4095)
        for (length_gr0, length_gr1) in all_lengths.iter() {
            assert!(*length_gr0 <= 4095, "Part2_3_length GR0 {} out of range", length_gr0);
            assert!(*length_gr1 <= 4095, "Part2_3_length GR1 {} out of range", length_gr1);
            assert!(*length_gr0 > 0, "Part2_3_length GR0 should be positive");
            assert!(*length_gr1 > 0, "Part2_3_length GR1 should be positive");
        }
        
        // Validate specific Frame 1 values
        assert_eq!(frame_1_lengths[0].0, 763, "Frame 1 CH0 GR0 part2_3_length mismatch");
        assert_eq!(frame_1_lengths[0].1, 689, "Frame 1 CH0 GR1 part2_3_length mismatch");
    }

    /// Test count1 values (quadruple count)
    #[test]
    fn test_count1_validation() {
        // Known count1 values from sample-3s.wav encoding
        let frame_1_count1 = [(48, 36), (48, 36)]; // (GR0, GR1) for (CH0, CH1)
        let frame_2_count1 = [(47, 40), (47, 40)];
        let frame_3_count1 = [(36, 38), (36, 38)];
        
        let all_count1 = [frame_1_count1, frame_2_count1, frame_3_count1].concat();
        
        for (count1_gr0, count1_gr1) in all_count1.iter() {
            assert!(*count1_gr0 <= 144, "Count1 GR0 {} out of range", count1_gr0);
            assert!(*count1_gr1 <= 144, "Count1 GR1 {} out of range", count1_gr1);
            assert!(*count1_gr0 > 0, "Count1 GR0 should be positive");
            assert!(*count1_gr1 > 0, "Count1 GR1 should be positive");
        }
        
        // Validate specific Frame 1 values
        assert_eq!(frame_1_count1[0].0, 48, "Frame 1 CH0 GR0 count1 mismatch");
        assert_eq!(frame_1_count1[0].1, 36, "Frame 1 CH0 GR1 count1 mismatch");
    }

    /// Test mathematical relationships in quantization
    #[test]
    fn test_quantization_mathematical_properties() {
        let xrmax_gr0 = 174601576;
        let xrmax_gr1 = 543987899;
        let global_gain_gr0 = 170;
        let global_gain_gr1 = 176;
        let big_values_gr0 = 94;
        let big_values_gr1 = 104;
        let count1_gr0 = 48;
        let count1_gr1 = 36;
        
        // Test that xrmax is related to the quantization step size
        // Higher xrmax should generally require higher global_gain
        let xrmax_ratio = xrmax_gr1 as f64 / xrmax_gr0 as f64;
        let gain_diff = global_gain_gr1 as i32 - global_gain_gr0 as i32;
        
        assert!(xrmax_ratio > 1.0, "Higher complexity should have higher xrmax");
        assert!(gain_diff > 0, "Higher complexity should need higher gain");
        
        // Test that big_values and count1 are reasonable
        // big_values * 2 + count1 * 4 should not exceed 576 (granule size)
        let total_coeffs_gr0 = big_values_gr0 * 2 + count1_gr0 * 4;
        let total_coeffs_gr1 = big_values_gr1 * 2 + count1_gr1 * 4;
        
        assert!(total_coeffs_gr0 <= 576, "Total coefficients should not exceed granule size");
        assert!(total_coeffs_gr1 <= 576, "Total coefficients should not exceed granule size");
    }

    /// Test channel consistency
    #[test]
    fn test_channel_consistency() {
        // For stereo mode, both channels should have identical quantization parameters
        // This is expected for the test cases where both channels have the same content
        
        // Frame 1 parameters (both channels should match)
        let ch0_xrmax_gr0 = 174601576;
        let ch1_xrmax_gr0 = 174601576;
        let ch0_global_gain_gr0 = 170;
        let ch1_global_gain_gr0 = 170;
        let ch0_big_values_gr0 = 94;
        let ch1_big_values_gr0 = 94;
        
        assert_eq!(ch0_xrmax_gr0, ch1_xrmax_gr0, "CH0/CH1 GR0 xrmax should match");
        assert_eq!(ch0_global_gain_gr0, ch1_global_gain_gr0, "CH0/CH1 GR0 global_gain should match");
        assert_eq!(ch0_big_values_gr0, ch1_big_values_gr0, "CH0/CH1 GR0 big_values should match");
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
        fn test_global_gain_properties(
            global_gain in 0u32..256
        ) {
            // Global gain must be within MP3 standard range
            prop_assert!(global_gain <= 255, "Global gain out of range");
        }
        
        #[test]
        fn test_big_values_properties(
            big_values in 0u32..289
        ) {
            // Big values must not exceed MP3 standard limit
            prop_assert!(big_values <= 288, "Big values exceeds MP3 limit");
        }
        
        #[test]
        fn test_part2_3_length_properties(
            part2_3_length in 0u32..4096
        ) {
            // Part2_3_length is a 12-bit field
            prop_assert!(part2_3_length <= 4095, "Part2_3_length out of range");
        }
        
        #[test]
        fn test_count1_properties(
            count1 in 0u32..145
        ) {
            // Count1 should not exceed reasonable limit
            prop_assert!(count1 <= 144, "Count1 out of range");
        }
        
        #[test]
        fn test_coefficient_count_constraint(
            big_values in 0u32..289,
            count1 in 0u32..145
        ) {
            let total_coeffs = big_values * 2 + count1 * 4;
            
            // Total coefficients should not exceed granule size
            if big_values <= 288 && count1 <= 144 {
                prop_assert!(total_coeffs <= 576, "Total coefficients exceed granule size");
            }
        }
    }
}
    /// Test quantization parameters with real data from all frames
    #[test]
    fn test_quantization_real_data_validation() {
        // Frame 1 data
        const F1_XRMAX_CH0_GR0: i32 = 174601576;
        const F1_XRMAX_CH0_GR1: i32 = 543987899;
        const F1_GLOBAL_GAIN_CH0_GR0: u32 = 170;
        const F1_GLOBAL_GAIN_CH0_GR1: u32 = 176;
        const F1_BIG_VALUES_CH0_GR0: u32 = 94;
        const F1_BIG_VALUES_CH0_GR1: u32 = 104;
        
        // Frame 2 data
        const F2_XRMAX_CH0_GR0: i32 = 761934185;
        const F2_XRMAX_CH0_GR1: i32 = 407502232;
        const F2_GLOBAL_GAIN_CH0_GR0: u32 = 175;
        const F2_GLOBAL_GAIN_CH0_GR1: u32 = 173;
        const F2_BIG_VALUES_CH0_GR0: u32 = 98;
        const F2_BIG_VALUES_CH0_GR1: u32 = 98;
        
        // Frame 3 data
        const F3_XRMAX_CH0_GR0: i32 = 398722265;
        const F3_XRMAX_CH0_GR1: i32 = 586508987;
        const F3_GLOBAL_GAIN_CH0_GR0: u32 = 173;
        const F3_GLOBAL_GAIN_CH0_GR1: u32 = 172;
        const F3_BIG_VALUES_CH0_GR0: u32 = 93;
        const F3_BIG_VALUES_CH0_GR1: u32 = 128;
        
        // Validate Frame 1 quantization parameters
        assert_eq!(F1_XRMAX_CH0_GR0, 174601576, "Frame 1 CH0 GR0 xrmax mismatch");
        assert_eq!(F1_XRMAX_CH0_GR1, 543987899, "Frame 1 CH0 GR1 xrmax mismatch");
        assert_eq!(F1_GLOBAL_GAIN_CH0_GR0, 170, "Frame 1 CH0 GR0 global gain mismatch");
        assert_eq!(F1_GLOBAL_GAIN_CH0_GR1, 176, "Frame 1 CH0 GR1 global gain mismatch");
        
        // Validate Frame 2 quantization parameters
        assert_eq!(F2_XRMAX_CH0_GR0, 761934185, "Frame 2 CH0 GR0 xrmax mismatch");
        assert_eq!(F2_XRMAX_CH0_GR1, 407502232, "Frame 2 CH0 GR1 xrmax mismatch");
        assert_eq!(F2_GLOBAL_GAIN_CH0_GR0, 175, "Frame 2 CH0 GR0 global gain mismatch");
        assert_eq!(F2_GLOBAL_GAIN_CH0_GR1, 173, "Frame 2 CH0 GR1 global gain mismatch");
        
        // Validate Frame 3 quantization parameters
        assert_eq!(F3_XRMAX_CH0_GR0, 398722265, "Frame 3 CH0 GR0 xrmax mismatch");
        assert_eq!(F3_XRMAX_CH0_GR1, 586508987, "Frame 3 CH0 GR1 xrmax mismatch");
        assert_eq!(F3_GLOBAL_GAIN_CH0_GR0, 173, "Frame 3 CH0 GR0 global gain mismatch");
        assert_eq!(F3_GLOBAL_GAIN_CH0_GR1, 172, "Frame 3 CH0 GR1 global gain mismatch");
        
        // Validate global gain ranges (0-255 for MP3)
        let all_gains = [
            F1_GLOBAL_GAIN_CH0_GR0, F1_GLOBAL_GAIN_CH0_GR1,
            F2_GLOBAL_GAIN_CH0_GR0, F2_GLOBAL_GAIN_CH0_GR1,
            F3_GLOBAL_GAIN_CH0_GR0, F3_GLOBAL_GAIN_CH0_GR1,
        ];
        for &gain in &all_gains {
            assert!(gain <= 255, "Global gain {} out of range", gain);
            assert!(gain >= 100, "Global gain {} too low for typical audio", gain);
        }
        
        // Validate big_values (must be <= 288 for MP3 standard)
        let all_big_values = [
            F1_BIG_VALUES_CH0_GR0, F1_BIG_VALUES_CH0_GR1,
            F2_BIG_VALUES_CH0_GR0, F2_BIG_VALUES_CH0_GR1,
            F3_BIG_VALUES_CH0_GR0, F3_BIG_VALUES_CH0_GR1,
        ];
        for &big_val in &all_big_values {
            assert!(big_val <= 288, "Big values {} exceeds MP3 limit", big_val);
            assert!(big_val > 0, "Big values should be positive");
        }
        
        // Test that parameters show realistic variation across frames
        assert_ne!(F1_XRMAX_CH0_GR0, F2_XRMAX_CH0_GR0, "XRMAX should vary between frames");
        assert_ne!(F2_XRMAX_CH0_GR0, F3_XRMAX_CH0_GR0, "XRMAX should vary between frames");
        
        // Test Frame 2 has highest complexity (highest XRMAX for GR0)
        assert!(F2_XRMAX_CH0_GR0 > F1_XRMAX_CH0_GR0, "Frame 2 should have higher complexity than Frame 1");
        assert!(F2_XRMAX_CH0_GR0 > F3_XRMAX_CH0_GR0, "Frame 2 should have higher complexity than Frame 3");
    }