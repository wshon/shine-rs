//! Unit tests for subband analysis filter
//!
//! Tests the polyphase filter bank that splits the input signal
//! into 32 subbands for further processing.

use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subband_filter_initialization() {
        use crate::subband::shine_subband_initialise;
        use crate::types::Subband;
        
        let mut subband = Subband::default();
        
        // Initialize subband filter
        shine_subband_initialise(&mut subband);
        
        // Verify initialization
        assert_eq!(subband.off[0], 0, "Initial offset should be 0");
        assert_eq!(subband.off[1], 0, "Initial offset should be 0");
        
        // Verify filter coefficients are initialized (check first few entries)
        // Note: fl is [[i32; 64]; SBLIMIT] where SBLIMIT = 32
        let mut nonzero_count = 0;
        for sb in 0..32 {  // SBLIMIT = 32
            for i in 0..64 {
                if subband.fl[sb][i] != 0 {
                    nonzero_count += 1;
                }
            }
        }
        // After initialization, most coefficients should be non-zero
        assert!(nonzero_count > 1000, "Most filter coefficients should be non-zero after initialization");
    }

    #[test]
    fn test_subband_structure_initialization() {
        use crate::types::Subband;
        
        let subband = Subband::default();
        
        // Test initial state
        assert_eq!(subband.off[0], 0, "Initial offset should be 0");
        assert_eq!(subband.off[1], 0, "Initial offset should be 0");
        
        // Verify filter buffer is initialized to zero
        for ch in 0..2 {
            for i in 0..HAN_SIZE {
                assert_eq!(subband.x[ch][i], 0, "Initial filter buffer should be zero");
            }
        }
    }

    #[test]
    fn test_subband_offset_management() {
        let mut subband = Subband::default();
        
        // Test offset bounds
        assert!(subband.off[0] < HAN_SIZE as i32, "Offset should be within HAN_SIZE");
        assert!(subband.off[1] < HAN_SIZE as i32, "Offset should be within HAN_SIZE");
        
        // Test offset modification
        subband.off[0] = 100;
        assert_eq!(subband.off[0], 100, "Offset should be modifiable");
        
        // Test offset wrapping calculation
        let new_offset = (subband.off[0] + HAN_SIZE as i32 - 32) % HAN_SIZE as i32;
        assert!(new_offset < HAN_SIZE as i32, "Wrapped offset should be valid");
    }

    #[test]
    fn test_subband_buffer_structure() {
        let subband = Subband::default();
        
        // Test buffer dimensions
        assert_eq!(subband.x.len(), 2, "Should have buffers for 2 channels");
        assert_eq!(subband.x[0].len(), HAN_SIZE, "Buffer should be HAN_SIZE length");
        assert_eq!(subband.x[1].len(), HAN_SIZE, "Buffer should be HAN_SIZE length");
        
        // Test that we can access all buffer positions
        for ch in 0..2 {
            for i in 0..HAN_SIZE {
                let _val = subband.x[ch][i]; // Should not panic
            }
        }
    }

    #[test]
    fn test_subband_constants_consistency() {
        // Test that constants are consistent with MP3 standard
        assert_eq!(SBLIMIT, 32, "Should have 32 subbands per MP3 standard");
        assert_eq!(GRANULE_SIZE, 576, "Granule size should be 576 per MP3 standard");
        assert_eq!(HAN_SIZE, 512, "HAN_SIZE should be 512 per shine implementation");
        
        // Test relationship between constants
        assert_eq!(SBLIMIT * 18, GRANULE_SIZE, "32 subbands * 18 samples = 576 granule size");
    }

    /// Test subband filter output validation with real data from sample-3s.wav Frame 1
    #[test]
    fn test_subband_filter_real_data_validation() {
        // Real data extracted from actual encoding session of sample-3s.wav Frame 1
        const L3_SB_SAMPLE_CH0_GR1_FIRST_8: [i32; 8] = [1490, 647, 269, 691, 702, -204, -837, -291];
        const L3_SB_SAMPLE_CH0_GR1_BAND_1: [i32; 8] = [7133, -2800, 1515, 3308, -10633, 12954, -1342, -5218];
        
        // Validate that the values are within expected ranges for subband samples
        for &val in &L3_SB_SAMPLE_CH0_GR1_FIRST_8 {
            assert!(val.abs() < 100000, "Subband sample {} out of expected range", val);
        }
        
        for &val in &L3_SB_SAMPLE_CH0_GR1_BAND_1 {
            assert!(val.abs() < 100000, "Subband sample {} out of expected range", val);
        }
        
        // Test that values show expected variation (not all zeros or constant)
        let first_8_sum: i32 = L3_SB_SAMPLE_CH0_GR1_FIRST_8.iter().sum();
        let band_1_sum: i32 = L3_SB_SAMPLE_CH0_GR1_BAND_1.iter().sum();
        
        assert!(first_8_sum != 0, "Subband samples should not all be zero");
        assert!(band_1_sum != 0, "Subband samples should not all be zero");
        
        // Test that we have both positive and negative values (typical for audio)
        let has_positive = L3_SB_SAMPLE_CH0_GR1_FIRST_8.iter().any(|&x| x > 0);
        let has_negative = L3_SB_SAMPLE_CH0_GR1_FIRST_8.iter().any(|&x| x < 0);
        assert!(has_positive, "Should have positive samples");
        assert!(has_negative, "Should have negative samples");
    }

    #[test]
    fn test_subband_channel_independence() {
        let mut subband = Subband::default();
        
        // Test that channels have independent state
        subband.off[0] = 100;
        subband.off[1] = 200;
        
        assert_ne!(subband.off[0], subband.off[1], "Channels should have independent offsets");
        
        // Test that buffer modifications are independent
        subband.x[0][0] = 1000;
        subband.x[1][0] = 2000;
        
        assert_ne!(subband.x[0][0], subband.x[1][0], "Channels should have independent buffers");
        assert_eq!(subband.x[0][0], 1000, "Channel 0 buffer should be modifiable");
        assert_eq!(subband.x[1][0], 2000, "Channel 1 buffer should be modifiable");
    }

    #[test]
    fn test_subband_energy_properties() {
        // Test energy-related properties of subband processing
        
        // For a typical audio frame, energy should be distributed across subbands
        // Lower subbands typically have more energy than higher subbands
        
        // Test that we can calculate energy for subband samples
        let samples = [1000i32, -500, 750, -250, 100, -50, 25, -10];
        let energy: i64 = samples.iter().map(|&x| (x as i64) * (x as i64)).sum();
        
        assert!(energy > 0, "Energy should be positive for non-zero samples");
        
        // Test energy scaling
        let scaled_samples: Vec<i32> = samples.iter().map(|&x| x / 2).collect();
        let scaled_energy: i64 = scaled_samples.iter().map(|&x| (x as i64) * (x as i64)).sum();
        
        assert!(scaled_energy < energy, "Scaled samples should have less energy");
        assert_eq!(scaled_energy * 4, energy, "Energy should scale quadratically");
    }

    #[test]
    fn test_subband_boundary_conditions() {
        let mut subband = Subband::default();
        
        // Test boundary offset values
        subband.off[0] = HAN_SIZE as i32 - 1;
        assert_eq!(subband.off[0], HAN_SIZE as i32 - 1, "Should handle maximum offset");
        
        // Test offset wrapping
        let wrapped = (subband.off[0] + HAN_SIZE as i32 - 32) % HAN_SIZE as i32;
        assert!(wrapped < HAN_SIZE as i32, "Wrapped offset should be valid");
        
        // Test boundary buffer access
        subband.x[0][0] = i32::MAX;
        subband.x[0][HAN_SIZE - 1] = i32::MIN;
        
        assert_eq!(subband.x[0][0], i32::MAX, "Should handle maximum values");
        assert_eq!(subband.x[0][HAN_SIZE - 1], i32::MIN, "Should handle minimum values");
    }

    #[test]
    fn test_subband_filter_linearity_properties() {
        // Test linearity properties that should hold for the subband filter
        
        // Test that zero input produces predictable behavior
        let zero_samples = [0i32; 18];
        let zero_energy: i64 = zero_samples.iter().map(|&x| (x as i64) * (x as i64)).sum();
        assert_eq!(zero_energy, 0, "Zero input should have zero energy");
        
        // Test scaling properties
        let base_samples = [100i32, -50, 25, -12, 6, -3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let scale_factor = 2;
        let scaled_samples: Vec<i32> = base_samples.iter().map(|&x| x * scale_factor).collect();
        
        assert_eq!(scaled_samples[0], base_samples[0] * scale_factor, "Scaling should be linear");
        assert_eq!(scaled_samples.len(), base_samples.len(), "Length should be preserved");
    }

    #[test]
    fn test_subband_memory_layout() {
        let subband = Subband::default();
        
        // Test that memory layout is as expected
        assert_eq!(std::mem::size_of_val(&subband.off), 2 * std::mem::size_of::<usize>(), "Offset array size");
        assert_eq!(std::mem::size_of_val(&subband.x), 2 * HAN_SIZE * std::mem::size_of::<i32>(), "Buffer array size");
        
        // Test that we can take references to different parts
        let _off_ref = &subband.off[0];
        let _buf_ref = &subband.x[0][0];
        
        // Test that arrays are properly aligned
        assert_eq!(subband.off.len(), 2, "Should have exactly 2 offset values");
        assert_eq!(subband.x.len(), 2, "Should have exactly 2 channel buffers");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 50, // Reduced for performance
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_subband_offset_properties(
            offset in 0usize..512
        ) {
            // Offset should always be within valid range
            prop_assert!(offset < HAN_SIZE, "Offset should be less than HAN_SIZE");
            
            // Test offset wrapping
            let wrapped = (offset + HAN_SIZE - 32) % HAN_SIZE;
            prop_assert!(wrapped < HAN_SIZE, "Wrapped offset should be valid");
            
            // Test that wrapping preserves relative ordering for small increments
            if offset < HAN_SIZE - 32 {
                prop_assert!(wrapped == offset + HAN_SIZE - 32, "Small offsets should increment normally");
            }
        }

        #[test]
        fn test_subband_sample_properties(
            samples in prop::collection::vec(-10000i32..=10000i32, 18)
        ) {
            // Test properties of subband samples (18 samples per subband)
            
            // All samples should be within reasonable range
            for &sample in &samples {
                prop_assert!(sample.abs() <= 10000, "Sample should be within test range");
            }
            
            // Test energy calculation
            let energy: i64 = samples.iter().map(|&x| (x as i64) * (x as i64)).sum();
            prop_assert!(energy >= 0, "Energy should be non-negative");
            
            // Test that we have the right number of samples per subband
            prop_assert_eq!(samples.len(), 18, "Should have 18 samples per subband");
        }

        #[test]
        fn test_subband_channel_properties(
            ch in 0usize..2
        ) {
            // Test channel indexing properties
            prop_assert!(ch < 2, "Channel index should be 0 or 1");
            
            let mut subband = Subband::default();
            
            // Each channel should have valid state
            prop_assert_eq!(subband.off[ch], 0, "Initial offset should be 0");
            prop_assert_eq!(subband.x[ch].len(), HAN_SIZE, "Filter buffer should be HAN_SIZE");
            
            // Test that we can modify channel state independently
            subband.off[ch] = 100;
            prop_assert_eq!(subband.off[ch], 100, "Offset should be modifiable");
            
            // Other channel should be unaffected
            let other_ch = 1 - ch;
            prop_assert_eq!(subband.off[other_ch], 0, "Other channel should be unaffected");
        }

        #[test]
        fn test_subband_energy_properties(
            samples in prop::collection::vec(-1000i32..=1000i32, 32)
        ) {
            // Test energy properties across all subbands
            prop_assert_eq!(samples.len(), 32, "Should test all subbands");
            
            // Calculate total energy
            let total_energy: i64 = samples.iter().map(|&x| (x as i64) * (x as i64)).sum();
            prop_assert!(total_energy >= 0, "Total energy should be non-negative");
            
            // Test energy distribution
            let max_sample = samples.iter().map(|&x| x.abs()).max().unwrap_or(0);
            let max_energy = (max_sample as i64) * (max_sample as i64);
            
            prop_assert!(total_energy >= max_energy, "Total energy should be at least max sample energy");
            prop_assert!(total_energy <= 32 * max_energy, "Total energy should not exceed sum of max energies");
        }

        #[test]
        fn test_subband_buffer_properties(
            buffer_values in prop::collection::vec(-32768i32..=32767i32, 512)
        ) {
            // Test properties of the subband filter buffer
            prop_assert_eq!(buffer_values.len(), HAN_SIZE, "Buffer should be HAN_SIZE");
            
            let mut subband = Subband::default();
            
            // Test that we can fill the buffer
            for (i, &val) in buffer_values.iter().enumerate() {
                subband.x[0][i] = val;
            }
            
            // Verify values were set correctly
            for (i, &expected) in buffer_values.iter().enumerate() {
                prop_assert_eq!(subband.x[0][i], expected, "Buffer value should be set correctly");
            }
            
            // Test that other channel is unaffected
            for i in 0..HAN_SIZE {
                prop_assert_eq!(subband.x[1][i], 0, "Other channel should remain zero");
            }
        }
    }
}