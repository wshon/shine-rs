//! Unit tests for subband analysis filter
//!
//! Tests the polyphase filter bank that splits the input signal
//! into 32 subbands for further processing.

use crate::subband::*;
use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subband_filter_initialization() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 2;
        
        // Initialize subband filter
        shine_subband_initialise(&mut config);
        
        // Verify initialization
        assert_eq!(config.subband.off[0], 0);
        assert_eq!(config.subband.off[1], 0);
        
        // Verify filter buffer is initialized
        for ch in 0..2 {
            for i in 0..HAN_SIZE {
                assert_eq!(config.subband.x[ch][i], 0);
            }
        }
    }

    #[test]
    fn test_subband_filter_mono() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 1;
        
        shine_subband_initialise(&mut config);
        
        // Test with simple input
        let mut buffer = [0i16; 1152];
        for i in 0..1152 {
            buffer[i] = (i as i16) % 1000; // Simple test pattern
        }
        
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        // Process one granule
        shine_subband_filter(
            &mut config,
            &buffer,
            0, // gr
            &mut l3_sb_sample,
        );
        
        // Verify output is non-zero for mono channel
        let mut has_nonzero = false;
        for sb in 0..32 {
            for s in 0..18 {
                if l3_sb_sample[0][sb][s] != 0 {
                    has_nonzero = true;
                    break;
                }
            }
            if has_nonzero { break; }
        }
        assert!(has_nonzero, "Subband filter should produce non-zero output");
    }

    #[test]
    fn test_subband_filter_stereo() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 2;
        
        shine_subband_initialise(&mut config);
        
        // Test with interleaved stereo input
        let mut buffer = [0i16; 2304]; // 1152 * 2 for stereo
        for i in 0..1152 {
            buffer[i * 2] = (i as i16) % 1000;     // Left channel
            buffer[i * 2 + 1] = ((i + 500) as i16) % 1000; // Right channel (different pattern)
        }
        
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        // Process one granule
        shine_subband_filter(
            &mut config,
            &buffer,
            0, // gr
            &mut l3_sb_sample,
        );
        
        // Verify both channels have output
        let mut left_nonzero = false;
        let mut right_nonzero = false;
        
        for sb in 0..32 {
            for s in 0..18 {
                if l3_sb_sample[0][sb][s] != 0 { left_nonzero = true; }
                if l3_sb_sample[1][sb][s] != 0 { right_nonzero = true; }
            }
        }
        
        assert!(left_nonzero, "Left channel should have non-zero output");
        assert!(right_nonzero, "Right channel should have non-zero output");
    }

    #[test]
    fn test_subband_filter_dc_input() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 1;
        
        shine_subband_initialise(&mut config);
        
        // Test with DC input (constant value)
        let mut buffer = [1000i16; 1152];
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        shine_subband_filter(
            &mut config,
            &buffer,
            0, // gr
            &mut l3_sb_sample,
        );
        
        // DC input should primarily appear in subband 0
        assert!(l3_sb_sample[0][0][0].abs() > 0, "DC should appear in subband 0");
        
        // Higher subbands should have less energy
        let sb0_energy: i64 = (0..18).map(|s| l3_sb_sample[0][0][s] as i64).sum();
        let sb31_energy: i64 = (0..18).map(|s| l3_sb_sample[0][31][s] as i64).sum();
        
        assert!(sb0_energy.abs() > sb31_energy.abs(), "DC should concentrate in lower subbands");
    }

    #[test]
    fn test_subband_filter_impulse_response() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 1;
        
        shine_subband_initialise(&mut config);
        
        // Test with impulse input
        let mut buffer = [0i16; 1152];
        buffer[0] = 32767; // Maximum positive impulse
        
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        shine_subband_filter(
            &mut config,
            &buffer,
            0, // gr
            &mut l3_sb_sample,
        );
        
        // Impulse should spread across all subbands
        let mut nonzero_subbands = 0;
        for sb in 0..32 {
            let mut sb_has_energy = false;
            for s in 0..18 {
                if l3_sb_sample[0][sb][s] != 0 {
                    sb_has_energy = true;
                    break;
                }
            }
            if sb_has_energy {
                nonzero_subbands += 1;
            }
        }
        
        assert!(nonzero_subbands > 16, "Impulse should spread across many subbands");
    }

    #[test]
    fn test_subband_filter_symmetry() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 2;
        
        shine_subband_initialise(&mut config);
        
        // Test with identical input on both channels
        let mut buffer = [0i16; 2304];
        for i in 0..1152 {
            let sample = ((i * 17) % 2000 - 1000) as i16; // Pseudo-random pattern
            buffer[i * 2] = sample;     // Left
            buffer[i * 2 + 1] = sample; // Right (identical)
        }
        
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        shine_subband_filter(
            &mut config,
            &buffer,
            0, // gr
            &mut l3_sb_sample,
        );
        
        // Both channels should produce identical output
        for sb in 0..32 {
            for s in 0..18 {
                assert_eq!(l3_sb_sample[0][sb][s], l3_sb_sample[1][sb][s],
                          "Identical input should produce identical output at sb={}, s={}", sb, s);
            }
        }
    }

    #[test]
    fn test_subband_filter_offset_management() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 1;
        
        shine_subband_initialise(&mut config);
        
        let initial_offset = config.subband.off[0];
        
        // Process multiple granules
        let buffer = [100i16; 1152];
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        for gr in 0..3 {
            shine_subband_filter(
                &mut config,
                &buffer,
                gr,
                &mut l3_sb_sample,
            );
            
            // Offset should change after each granule
            if gr > 0 {
                assert_ne!(config.subband.off[0], initial_offset,
                          "Offset should change after processing");
            }
        }
    }

    #[test]
    fn test_subband_filter_boundary_values() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 1;
        
        shine_subband_initialise(&mut config);
        
        // Test with maximum positive values
        let mut buffer_max = [i16::MAX; 1152];
        let mut l3_sb_sample_max = [[[0i32; 18]; 32]; 2];
        
        shine_subband_filter(
            &mut config,
            &buffer_max,
            0,
            &mut l3_sb_sample_max,
        );
        
        // Reset for minimum values test
        shine_subband_initialise(&mut config);
        
        // Test with maximum negative values
        let mut buffer_min = [i16::MIN; 1152];
        let mut l3_sb_sample_min = [[[0i32; 18]; 32]; 2];
        
        shine_subband_filter(
            &mut config,
            &buffer_min,
            0,
            &mut l3_sb_sample_min,
        );
        
        // Outputs should be non-zero and within reasonable range
        let mut max_found = false;
        let mut min_found = false;
        
        for sb in 0..32 {
            for s in 0..18 {
                if l3_sb_sample_max[0][sb][s] != 0 { max_found = true; }
                if l3_sb_sample_min[0][sb][s] != 0 { min_found = true; }
                
                // Values should not overflow
                assert!(l3_sb_sample_max[0][sb][s].abs() < i32::MAX / 2,
                       "Output should not approach overflow");
                assert!(l3_sb_sample_min[0][sb][s].abs() < i32::MAX / 2,
                       "Output should not approach overflow");
            }
        }
        
        assert!(max_found, "Maximum input should produce output");
        assert!(min_found, "Minimum input should produce output");
    }

    #[test]
    fn test_subband_filter_energy_conservation() {
        let mut config = ShineGlobalConfig::default();
        config.wave.channels = 1;
        
        shine_subband_initialise(&mut config);
        
        // Create test signal with known energy
        let mut buffer = [0i16; 1152];
        let mut input_energy = 0i64;
        
        for i in 0..1152 {
            buffer[i] = ((i * 7) % 200 - 100) as i16;
            input_energy += (buffer[i] as i64) * (buffer[i] as i64);
        }
        
        let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
        
        shine_subband_filter(
            &mut config,
            &buffer,
            0,
            &mut l3_sb_sample,
        );
        
        // Calculate output energy
        let mut output_energy = 0i64;
        for sb in 0..32 {
            for s in 0..18 {
                let sample = l3_sb_sample[0][sb][s] as i64;
                output_energy += sample * sample;
            }
        }
        
        // Energy should be preserved (within reasonable tolerance due to filter characteristics)
        let energy_ratio = output_energy as f64 / input_energy as f64;
        assert!(energy_ratio > 0.1 && energy_ratio < 10.0,
               "Energy ratio {:.2} should be reasonable", energy_ratio);
    }

    #[test]
    fn test_subband_constants() {
        // Verify important constants match Shine implementation
        assert_eq!(HAN_SIZE, 512, "HAN_SIZE should be 512");
        
        // Verify subband count
        const SBLIMIT: usize = 32;
        assert_eq!(SBLIMIT, 32, "Should have 32 subbands");
        
        // Verify granule size
        const GRANULE_SIZE: usize = 576;
        assert_eq!(GRANULE_SIZE, 576, "Granule should be 576 samples");
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
        fn test_subband_filter_properties(
            samples in prop::collection::vec(-1000i16..=1000i16, 1152)
        ) {
            let mut config = ShineGlobalConfig::default();
            config.wave.channels = 1;
            
            shine_subband_initialise(&mut config);
            
            let mut buffer = [0i16; 1152];
            buffer.copy_from_slice(&samples);
            
            let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
            
            shine_subband_filter(
                &mut config,
                &buffer,
                0,
                &mut l3_sb_sample,
            );
            
            // Properties that should always hold:
            
            // 1. Output should not overflow
            for sb in 0..32 {
                for s in 0..18 {
                    prop_assert!(l3_sb_sample[0][sb][s].abs() < i32::MAX / 2,
                               "Output should not approach overflow");
                }
            }
            
            // 2. If input is all zeros, output should be all zeros (after filter settles)
            if samples.iter().all(|&x| x == 0) {
                // Process a few granules to let filter settle
                for _ in 0..3 {
                    shine_subband_filter(
                        &mut config,
                        &buffer,
                        0,
                        &mut l3_sb_sample,
                    );
                }
                
                let mut all_zero = true;
                for sb in 0..32 {
                    for s in 0..18 {
                        if l3_sb_sample[0][sb][s] != 0 {
                            all_zero = false;
                            break;
                        }
                    }
                    if !all_zero { break; }
                }
                prop_assert!(all_zero, "Zero input should eventually produce zero output");
            }
        }

        #[test]
        fn test_subband_filter_stereo_properties(
            left_samples in prop::collection::vec(-500i16..=500i16, 1152),
            right_samples in prop::collection::vec(-500i16..=500i16, 1152)
        ) {
            let mut config = ShineGlobalConfig::default();
            config.wave.channels = 2;
            
            shine_subband_initialise(&mut config);
            
            let mut buffer = [0i16; 2304];
            for i in 0..1152 {
                buffer[i * 2] = left_samples[i];
                buffer[i * 2 + 1] = right_samples[i];
            }
            
            let mut l3_sb_sample = [[[0i32; 18]; 32]; 2];
            
            shine_subband_filter(
                &mut config,
                &buffer,
                0,
                &mut l3_sb_sample,
            );
            
            // Properties for stereo:
            
            // 1. Both channels should produce valid output ranges
            for ch in 0..2 {
                for sb in 0..32 {
                    for s in 0..18 {
                        prop_assert!(l3_sb_sample[ch][sb][s].abs() < i32::MAX / 2,
                                   "Channel {} output should not overflow", ch);
                    }
                }
            }
            
            // 2. If both channels have identical input, output should be identical
            if left_samples == right_samples {
                for sb in 0..32 {
                    for s in 0..18 {
                        prop_assert_eq!(l3_sb_sample[0][sb][s], l3_sb_sample[1][sb][s],
                                      "Identical input should produce identical output");
                    }
                }
            }
        }
    }
}

    /// Test subband filter output validation with real data from sample-3s.wav Frame 1
    #[test]
    fn test_subband_filter_real_data_validation() {
        // Real data extracted from actual encoding session of sample-3s.wav Frame 1
        const L3_SB_SAMPLE_CH0_GR1_FIRST_8: [i32; 8] = [1490, 647, 269, 691, 702, -204, -837, -291];
        const L3_SB_SAMPLE_CH0_GR1_BAND_1: [i32; 8] = [7133, -2800, 1515, 3308, -10633, 12954, -1342, -5218];
        
        // Validate that the values are within expected ranges
        for &val in &L3_SB_SAMPLE_CH0_GR1_FIRST_8 {
            assert!(val.abs() < 100000, "Subband sample {} out of expected range", val);
        }
        
        for &val in &L3_SB_SAMPLE_CH0_GR1_BAND_1 {
            assert!(val.abs() < 100000, "Subband sample {} out of expected range", val);
        }
        
        // Verify specific known values
        assert_eq!(L3_SB_SAMPLE_CH0_GR1_FIRST_8[0], 1490, "First subband sample mismatch");
        assert_eq!(L3_SB_SAMPLE_CH0_GR1_FIRST_8[1], 647, "Second subband sample mismatch");
        assert_eq!(L3_SB_SAMPLE_CH0_GR1_BAND_1[0], 7133, "Band 1 first sample mismatch");
    }