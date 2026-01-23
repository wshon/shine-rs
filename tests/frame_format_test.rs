//! MP3 frame format and size validation tests
//!
//! This module tests MP3 frame structure, size calculation, and format compliance.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
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

    /// Calculate expected frame size based on MP3 parameters
    fn calculate_expected_frame_size(bitrate: u32, sample_rate: u32, padding: bool) -> usize {
        let samples_per_frame = 1152; // MPEG-1 Layer III
        let bits_per_slot = 8;
        
        let avg_slots_per_frame = (samples_per_frame as f64 / sample_rate as f64) * 
                                 (1000.0 * bitrate as f64 / bits_per_slot as f64);
        
        let whole_slots = avg_slots_per_frame as usize;
        whole_slots + if padding { 1 } else { 0 }
    }

    #[test]
    fn test_frame_size_consistency_mono() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };

        let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
        let samples_per_frame = encoder.samples_per_frame();
        
        // Test multiple different patterns
        let test_patterns = [
            ("zeros", vec![0i16; samples_per_frame]),
            ("sine_440", (0..samples_per_frame).map(|i| {
                let t = i as f64 / 44100.0;
                (8000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16
            }).collect()),
            ("white_noise", (0..samples_per_frame).map(|i| {
                ((i * 1234567) % 65536) as i16 - 32767
            }).collect()),
        ];

        let expected_size = calculate_expected_frame_size(128, 44100, true);
        
        for (name, samples) in &test_patterns {
            let encoded_frame = encoder.encode_frame(samples)
                .expect(&format!("Failed to encode {}", name));
            
            assert_eq!(
                encoded_frame.len(), expected_size,
                "Frame size mismatch for pattern '{}': got {}, expected {}",
                name, encoded_frame.len(), expected_size
            );
            
            // Verify MP3 sync word
            assert!(encoded_frame.len() >= 4, "Frame too small for pattern '{}'", name);
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            assert_eq!(sync, 0x7FF, "Invalid sync word for pattern '{}'", name);
        }
    }

    #[test]
    fn test_frame_size_consistency_stereo() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
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

        let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
        let samples_per_frame = encoder.samples_per_frame();
        
        // Generate stereo test data (interleaved)
        let mut stereo_samples = Vec::with_capacity(samples_per_frame * 2);
        for i in 0..samples_per_frame {
            let t = i as f64 / 44100.0;
            let left = (5000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16;
            let right = (5000.0 * (2.0 * std::f64::consts::PI * 880.0 * t).sin()) as i16;
            stereo_samples.push(left);
            stereo_samples.push(right);
        }
        
        let encoded_frame = encoder.encode_frame_interleaved(&stereo_samples)
            .expect("Failed to encode stereo frame");
        
        let expected_size = calculate_expected_frame_size(128, 44100, true);
        assert_eq!(
            encoded_frame.len(), expected_size,
            "Stereo frame size mismatch: got {}, expected {}",
            encoded_frame.len(), expected_size
        );
    }

    #[test]
    fn test_frame_header_structure() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };

        let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
        let samples = vec![1000i16; encoder.samples_per_frame()];
        
        let encoded_frame = encoder.encode_frame(&samples)
            .expect("Failed to encode frame");
        
        assert!(encoded_frame.len() >= 4, "Frame must have at least 4-byte header");
        
        // Parse frame header
        let header = ((encoded_frame[0] as u32) << 24) |
                    ((encoded_frame[1] as u32) << 16) |
                    ((encoded_frame[2] as u32) << 8) |
                    (encoded_frame[3] as u32);
        
        // Verify sync word (bits 31-21)
        let sync = (header >> 21) & 0x7FF;
        assert_eq!(sync, 0x7FF, "Invalid sync word: 0x{:03X}", sync);
        
        // Verify MPEG version (bits 20-19) - should be 11 for MPEG-1
        let version = (header >> 19) & 0x3;
        assert_eq!(version, 0x3, "Invalid MPEG version: {}", version);
        
        // Verify layer (bits 18-17) - should be 01 for Layer III
        let layer = (header >> 17) & 0x3;
        assert_eq!(layer, 0x1, "Invalid layer: {}", layer);
        
        // Verify bitrate index (bits 15-12) - should be 9 for 128kbps MPEG-1
        let bitrate_index = (header >> 12) & 0xF;
        assert_eq!(bitrate_index, 9, "Invalid bitrate index: {}", bitrate_index);
        
        // Verify sample rate index (bits 11-10) - should be 0 for 44100Hz
        let samplerate_index = (header >> 10) & 0x3;
        assert_eq!(samplerate_index, 0, "Invalid sample rate index: {}", samplerate_index);
        
        // Verify channel mode (bits 7-6) - should be 11 for mono
        let channel_mode = (header >> 6) & 0x3;
        assert_eq!(channel_mode, 3, "Invalid channel mode: {}", channel_mode);
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
        fn test_frame_size_property(
            bitrate in prop::sample::select(&[32u32, 64, 96, 128, 160, 192, 256, 320]),
            sample_rate in prop::sample::select(&[44100u32, 48000, 32000]),
            channels in prop::sample::select(&[Channels::Mono, Channels::Stereo])
        ) {
            setup_clean_errors();
            
            let mode = match channels {
                Channels::Mono => StereoMode::Mono,
                Channels::Stereo => StereoMode::Stereo,
            };
            
            let config = Config {
                wave: WaveConfig { channels, sample_rate },
                mpeg: MpegConfig {
                    mode, bitrate,
                    emphasis: Emphasis::None,
                    copyright: false,
                    original: true,
                },
            };
            
            // Skip invalid combinations
            if config.validate().is_err() {
                return Ok(());
            }
            
            let mut encoder = Mp3Encoder::new(config).unwrap();
            let samples_per_frame = encoder.samples_per_frame();
            
            let channel_count = match channels {
                Channels::Mono => 1,
                Channels::Stereo => 2,
            };
            
            let pcm_data: Vec<i16> = (0..samples_per_frame * channel_count)
                .map(|i| ((i * 12345) % 20000) as i16 - 10000)
                .collect();
            
            let encoded_frame = if channels == Channels::Stereo {
                encoder.encode_frame_interleaved(&pcm_data)
            } else {
                encoder.encode_frame(&pcm_data)
            };
            
            prop_assert!(encoded_frame.is_ok(), "Encoding should succeed");
            
            let frame = encoded_frame.unwrap().to_vec(); // Copy the frame data
            
            // Frame should have reasonable size
            prop_assert!(frame.len() >= 100, "Frame should be at least 100 bytes");
            prop_assert!(frame.len() <= 2000, "Frame should not exceed 2000 bytes");
            
            // Frame should start with valid sync word
            prop_assert!(frame.len() >= 4, "Frame should have header");
            let sync = ((frame[0] as u16) << 3) | ((frame[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Frame should have valid sync word");
            
            // Frame size should be consistent for same parameters
            let frame2_result = if channels == Channels::Stereo {
                encoder.encode_frame_interleaved(&pcm_data)
            } else {
                encoder.encode_frame(&pcm_data)
            };
            
            prop_assert!(frame2_result.is_ok(), "Second encoding should succeed");
            let frame2 = frame2_result.unwrap();
            
            prop_assert_eq!(frame.len(), frame2.len(), "Frame size should be consistent");
        }
    }
}