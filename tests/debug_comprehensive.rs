//! Comprehensive debug tests for MP3 encoder
//!
//! This module contains all debug tests consolidated from various debug files.
//! Tests are organized by functionality and follow the testing guidelines.

mod debug_tools;

use debug_tools::*;
use rust_mp3_encoder::Config;
use rust_mp3_encoder::config::{Channels, StereoMode};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_basic_encoding() {
        // Test basic encoding functionality like debug_test.rs
        let config = DebugConfig {
            channels: Channels::Stereo,
            mode: StereoMode::Stereo,
            duration: 0.02, // Very short
            ..Default::default()
        };
        
        // Test with all zeros (should be easy to encode)
        let pcm_data = SignalGenerator::silence(&config);
        let result = DebugRunner::run_debug_test("basic_zeros", config.clone(), pcm_data);
        assert!(result.is_ok(), "All-zero encoding should succeed: {:?}", result);
        
        // Test with small constant values
        let pcm_data = SignalGenerator::constant_value(&config, 100);
        let result = DebugRunner::run_debug_test("basic_constant", config, pcm_data);
        assert!(result.is_ok(), "Constant value encoding should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_stereo_big_values() {
        let config = DebugConfig {
            channels: Channels::Stereo,
            mode: StereoMode::Stereo,
            ..Default::default()
        };
        
        let pcm_data = SignalGenerator::sine_wave(&config, 1000.0, 8000.0);
        
        let result = DebugRunner::run_debug_test("stereo_big_values", config, pcm_data);
        assert!(result.is_ok(), "Stereo encoding should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_mono_encoding() {
        let config = DebugConfig {
            channels: Channels::Mono,
            mode: StereoMode::Mono,
            ..Default::default()
        };
        
        let pcm_data = SignalGenerator::sine_wave(&config, 440.0, 16000.0);
        
        let result = DebugRunner::run_debug_test("mono_encoding", config, pcm_data);
        assert!(result.is_ok(), "Mono encoding should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_mixed_frequencies() {
        let config = DebugConfig::default();
        let pcm_data = SignalGenerator::mixed_frequencies(&config);
        
        let result = DebugRunner::run_debug_test("mixed_frequencies", config, pcm_data);
        assert!(result.is_ok(), "Mixed frequency encoding should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_quiet_signal() {
        let config = DebugConfig::default();
        let pcm_data = SignalGenerator::quiet_signal(&config);
        
        let result = DebugRunner::run_debug_test("quiet_signal", config, pcm_data);
        assert!(result.is_ok(), "Quiet signal encoding should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_silence() {
        let config = DebugConfig::default();
        let pcm_data = SignalGenerator::silence(&config);
        
        let result = DebugRunner::run_debug_test("silence", config, pcm_data);
        assert!(result.is_ok(), "Silence encoding should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_different_bitrates() {
        let bitrates = [64, 96, 128, 160, 192, 256, 320];
        
        for &bitrate in &bitrates {
            let config = DebugConfig {
                bitrate,
                duration: 0.05, // Shorter for multiple tests
                ..Default::default()
            };
            
            let pcm_data = SignalGenerator::sine_wave(&config, 1000.0, 8000.0);
            let test_name = format!("bitrate_{}", bitrate);
            
            let result = DebugRunner::run_debug_test(&test_name, config, pcm_data);
            assert!(result.is_ok(), "Bitrate {} should encode successfully: {:?}", bitrate, result);
        }
    }
    
    #[test]
    fn test_debug_different_sample_rates() {
        let sample_rates = [32000, 44100, 48000];
        
        for &sample_rate in &sample_rates {
            let config = DebugConfig {
                sample_rate,
                duration: 0.05,
                ..Default::default()
            };
            
            let pcm_data = SignalGenerator::sine_wave(&config, 1000.0, 8000.0);
            let test_name = format!("samplerate_{}", sample_rate);
            
            let result = DebugRunner::run_debug_test(&test_name, config, pcm_data);
            assert!(result.is_ok(), "Sample rate {} should encode successfully: {:?}", sample_rate, result);
        }
    }
    
    #[test]
    fn test_debug_frame_size_calculation() {
        let configs = [
            Config::default(),
            Config {
                mpeg: rust_mp3_encoder::config::MpegConfig {
                    bitrate: 64,
                    ..Default::default()
                },
                ..Default::default()
            },
            Config {
                wave: rust_mp3_encoder::config::WaveConfig {
                    sample_rate: 48000,
                    ..Default::default()
                },
                ..Default::default()
            },
        ];
        
        for (i, config) in configs.iter().enumerate() {
            println!("\n--- Frame Size Test {} ---", i + 1);
            DebugRunner::debug_frame_size(config);
        }
    }
    
    #[test]
    fn test_debug_amplitude_levels() {
        let amplitudes = [100.0, 1000.0, 8000.0, 16000.0, 32000.0];
        
        for &amplitude in &amplitudes {
            let config = DebugConfig {
                duration: 0.05,
                ..Default::default()
            };
            
            let pcm_data = SignalGenerator::sine_wave(&config, 1000.0, amplitude);
            let test_name = format!("amplitude_{}", amplitude as i32);
            
            let result = DebugRunner::run_debug_test(&test_name, config, pcm_data);
            assert!(result.is_ok(), "Amplitude {} should encode successfully: {:?}", amplitude, result);
        }
    }
    
    #[test]
    fn test_debug_pipeline_isolation() {
        // Test pipeline isolation to identify where issues occur
        let result = DebugRunner::run_pipeline_isolation_test("pipeline_isolation");
        assert!(result.is_ok(), "Pipeline isolation test should succeed: {:?}", result);
    }
    
    #[test]
    fn test_debug_frame_analysis() {
        // Test frame analysis with a simple case
        let config = DebugConfig {
            duration: 0.05,
            ..Default::default()
        };
        
        let pcm_data = SignalGenerator::sine_wave(&config, 440.0, 8000.0);
        
        // This test focuses on the analysis functionality
        let result = DebugRunner::run_debug_test("frame_analysis", config, pcm_data);
        assert!(result.is_ok(), "Frame analysis test should succeed: {:?}", result);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
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
    
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 20,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        
        #[test]
        fn prop_test_random_amplitudes(
            amplitude in 100.0f32..30000.0f32,
            frequency in 100.0f32..8000.0f32
        ) {
            setup_clean_errors();
            
            let config = DebugConfig {
                duration: 0.02, // Very short for property tests
                ..Default::default()
            };
            
            let pcm_data = SignalGenerator::sine_wave(&config, frequency, amplitude);
            let test_name = format!("prop_amp_{:.0}_freq_{:.0}", amplitude, frequency);
            
            let result = DebugRunner::run_debug_test(&test_name, config, pcm_data);
            prop_assert!(result.is_ok(), "Random signal encoding failed");
        }
        
        #[test]
        fn prop_test_random_bitrates(
            bitrate in prop::sample::select(vec![64u32, 96, 128, 160, 192, 256, 320])
        ) {
            setup_clean_errors();
            
            let config = DebugConfig {
                bitrate,
                duration: 0.02,
                ..Default::default()
            };
            
            let pcm_data = SignalGenerator::sine_wave(&config, 1000.0, 8000.0);
            let test_name = format!("prop_bitrate_{}", bitrate);
            
            let result = DebugRunner::run_debug_test(&test_name, config, pcm_data);
            prop_assert!(result.is_ok(), "Random bitrate encoding failed");
        }
    }
}