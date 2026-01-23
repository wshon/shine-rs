//! MDCT algorithm validation tests
//!
//! This module tests the MDCT implementation for correctness, numerical precision,
//! and compliance with the shine reference implementation.

use rust_mp3_encoder::mdct::shine_mdct_sub;
use rust_mp3_encoder::shine_config::ShineGlobalConfig;
use rust_mp3_encoder::config::{Config, WaveConfig, MpegConfig, StereoMode, Emphasis};
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_clean_errors() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                if let Some(s) = info.payload().downcast_ref::<String>() {
                    let msg = if s.len() > 200 { &s[..197] } else { s };
                    eprintln!("Test failed: {}", msg.trim());
                }
            }));
        });
    }

    fn create_test_config() -> ShineGlobalConfig {
        let config = Config {
            wave: WaveConfig {
                channels: rust_mp3_encoder::config::Channels::Stereo,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Stereo,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut shine_config = ShineGlobalConfig::new(config).unwrap();
        shine_config.initialize().unwrap();
        shine_config
    }

    #[test]
    fn test_mdct_zero_input() {
        let mut config = create_test_config();
        
        // Fill subband samples with zeros
        for ch in 0..2 {
            for gr in 0..2 {
                for t in 0..18 {
                    for sb in 0..32 {
                        config.l3_sb_sample[ch][gr][t][sb] = 0;
                        config.l3_sb_sample[ch][gr + 1][t][sb] = 0;
                    }
                }
            }
        }
        
        shine_mdct_sub(&mut config, 1);
        
        // Zero input should produce zero output (or very small values due to rounding)
        for ch in 0..2 {
            for gr in 0..2 {
                for coeff in 0..576 {
                    let val = config.mdct_freq[ch][gr][coeff];
                    assert!(val.abs() <= 1, "Zero input should produce near-zero output, got: {}", val);
                }
            }
        }
    }

    #[test]
    fn test_mdct_impulse_response() {
        let mut config = create_test_config();
        
        // Fill subband samples with zeros first
        for ch in 0..2 {
            for gr in 0..2 {
                for t in 0..18 {
                    for sb in 0..32 {
                        config.l3_sb_sample[ch][gr][t][sb] = 0;
                        config.l3_sb_sample[ch][gr + 1][t][sb] = 0;
                    }
                }
            }
        }
        
        // Create impulse at center
        config.l3_sb_sample[0][1][9][0] = 1000;
        
        shine_mdct_sub(&mut config, 1);
        
        // Impulse should spread energy across frequency bins
        let mut total_energy = 0i64;
        for coeff in 0..576 {
            let val = config.mdct_freq[0][0][coeff] as i64;
            total_energy += val * val;
        }
        assert!(total_energy > 0, "Impulse should produce non-zero energy");
    }

    #[test]
    fn test_mdct_sine_wave_input() {
        let mut config = create_test_config();
        
        // Generate sine wave in time domain
        for ch in 0..2 {
            for gr in 0..2 {
                for t in 0..18 {
                    for sb in 0..32 {
                        let phase = (t * sb) as f64 * 0.1;
                        config.l3_sb_sample[ch][gr][t][sb] = (1000.0 * phase.sin()) as i32;
                        config.l3_sb_sample[ch][gr + 1][t][sb] = (1000.0 * phase.sin()) as i32;
                    }
                }
            }
        }
        
        shine_mdct_sub(&mut config, 1);
        
        // Sine wave should produce concentrated energy in frequency domain
        let mut total_energy = 0i64;
        for coeff in 0..576 {
            let val = config.mdct_freq[0][0][coeff] as i64;
            total_energy += val * val;
        }
        assert!(total_energy > 100000, "Sine wave should produce significant energy");
    }

    #[test]
    fn test_mdct_aliasing_reduction() {
        let mut config = create_test_config();
        
        // Fill subband samples with zeros first
        for ch in 0..2 {
            for gr in 0..2 {
                for t in 0..18 {
                    for sb in 0..32 {
                        config.l3_sb_sample[ch][gr][t][sb] = 0;
                        config.l3_sb_sample[ch][gr + 1][t][sb] = 0;
                    }
                }
            }
        }
        
        // Create pattern that would cause aliasing without proper reduction
        for t in 0..18 {
            config.l3_sb_sample[0][1][t][15] = if t % 2 == 0 { 1000 } else { -1000 };
            config.l3_sb_sample[0][1][t][16] = if t % 2 == 0 { -1000 } else { 1000 };
        }
        
        shine_mdct_sub(&mut config, 1);
        
        // Verify aliasing reduction is working (coefficients should be reasonable)
        let mut max_coeff = 0i32;
        for coeff in 0..576 {
            max_coeff = max_coeff.max(config.mdct_freq[0][0][coeff].abs());
        }
        assert!(max_coeff < 100000, "Aliasing reduction should limit coefficient magnitude");
    }

    #[test]
    fn test_mdct_overflow_protection() {
        let mut config = create_test_config();
        
        // Fill with maximum values to test overflow protection
        for ch in 0..2 {
            for gr in 0..2 {
                for t in 0..18 {
                    for sb in 0..32 {
                        config.l3_sb_sample[ch][gr][t][sb] = if (t + sb) % 2 == 0 { 32767 } else { -32768 };
                        config.l3_sb_sample[ch][gr + 1][t][sb] = if (t + sb) % 2 == 0 { 32767 } else { -32768 };
                    }
                }
            }
        }
        
        shine_mdct_sub(&mut config, 1);
        
        // Verify no coefficient exceeds reasonable bounds
        for ch in 0..2 {
            for gr in 0..2 {
                for coeff in 0..576 {
                    let val = config.mdct_freq[ch][gr][coeff];
                    assert!(
                        val.abs() <= i32::MAX / 2,
                        "MDCT coefficient {} should not cause overflow",
                        val
                    );
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 50,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_mdct_energy_conservation(
            samples in prop::collection::vec(
                prop::collection::vec(
                    prop::collection::vec(
                        prop::collection::vec(-1000i32..1000, 32),
                        18
                    ),
                    3
                ),
                2
            )
        ) {
            setup_clean_errors();
            
            let mut config = create_test_config();
            
            // Copy generated samples
            for (ch, ch_samples) in samples.iter().enumerate() {
                for (gr, gr_samples) in ch_samples.iter().enumerate() {
                    for (t, t_samples) in gr_samples.iter().enumerate() {
                        for (sb, &sample) in t_samples.iter().enumerate() {
                            config.l3_sb_sample[ch][gr][t][sb] = sample;
                        }
                    }
                }
            }
            
            // Calculate input energy
            let mut input_energy = 0i64;
            for ch in 0..2 {
                for gr in 0..2 {
                    for t in 0..18 {
                        for sb in 0..32 {
                            let val = config.l3_sb_sample[ch][gr][t][sb] as i64;
                            input_energy += val * val;
                        }
                    }
                }
            }
            
            shine_mdct_sub(&mut config, 1);
            
            // Calculate output energy
            let mut output_energy = 0i64;
            for ch in 0..2 {
                for gr in 0..2 {
                    for coeff in 0..576 {
                        let val = config.mdct_freq[ch][gr][coeff] as i64;
                        output_energy += val * val;
                    }
                }
            }
            
            // Energy should be conserved (within reasonable bounds due to quantization)
            if input_energy > 0 {
                let energy_ratio = output_energy as f64 / input_energy as f64;
                prop_assert!(
                    energy_ratio > 0.1 && energy_ratio < 10.0,
                    "Energy should be approximately conserved, ratio: {}",
                    energy_ratio
                );
            }
        }

        #[test]
        fn test_mdct_linearity(
            samples1 in prop::collection::vec(
                prop::collection::vec(
                    prop::collection::vec(
                        prop::collection::vec(-500i32..500, 32),
                        18
                    ),
                    3
                ),
                2
            ),
            samples2 in prop::collection::vec(
                prop::collection::vec(
                    prop::collection::vec(
                        prop::collection::vec(-500i32..500, 32),
                        18
                    ),
                    3
                ),
                2
            )
        ) {
            setup_clean_errors();
            
            // Test MDCT linearity: MDCT(a + b) ≈ MDCT(a) + MDCT(b)
            let mut config1 = create_test_config();
            let mut config2 = create_test_config();
            let mut config_sum = create_test_config();
            
            // Set up test data
            for (ch, ch_samples) in samples1.iter().enumerate() {
                for (gr, gr_samples) in ch_samples.iter().enumerate() {
                    for (t, t_samples) in gr_samples.iter().enumerate() {
                        for (sb, &sample) in t_samples.iter().enumerate() {
                            config1.l3_sb_sample[ch][gr][t][sb] = sample;
                        }
                    }
                }
            }
            
            for (ch, ch_samples) in samples2.iter().enumerate() {
                for (gr, gr_samples) in ch_samples.iter().enumerate() {
                    for (t, t_samples) in gr_samples.iter().enumerate() {
                        for (sb, &sample) in t_samples.iter().enumerate() {
                            config2.l3_sb_sample[ch][gr][t][sb] = sample;
                        }
                    }
                }
            }
            
            // Create sum samples with overflow protection
            for ch in 0..2 {
                for gr in 0..3 {
                    for t in 0..18 {
                        for sb in 0..32 {
                            config_sum.l3_sb_sample[ch][gr][t][sb] = 
                                config1.l3_sb_sample[ch][gr][t][sb].saturating_add(
                                    config2.l3_sb_sample[ch][gr][t][sb]
                                );
                        }
                    }
                }
            }
            
            shine_mdct_sub(&mut config1, 1);
            shine_mdct_sub(&mut config2, 1);
            shine_mdct_sub(&mut config_sum, 1);
            
            // Check linearity (allowing for some numerical error)
            let mut max_error = 0i32;
            for ch in 0..2 {
                for gr in 0..2 {
                    for coeff in 0..576 {
                        let expected = config1.mdct_freq[ch][gr][coeff].saturating_add(
                            config2.mdct_freq[ch][gr][coeff]
                        );
                        let actual = config_sum.mdct_freq[ch][gr][coeff];
                        let error = (expected - actual).abs();
                        max_error = max_error.max(error);
                    }
                }
            }
            
            // Allow some error due to saturation and numerical precision
            prop_assert!(
                max_error < 1000,
                "MDCT should be approximately linear, max error: {}",
                max_error
            );
        }
    }
}