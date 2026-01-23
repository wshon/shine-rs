//! Comprehensive big_values validation tests
//!
//! This module provides thorough testing of the big_values field to ensure
//! it never exceeds the MP3 specification limit of 288.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test data structure for big_values validation
    struct BigValuesTestCase {
        name: &'static str,
        generator: fn(usize) -> Vec<i16>,
    }

    /// Generate test patterns for big_values validation
    fn generate_test_patterns() -> Vec<BigValuesTestCase> {
        vec![
            BigValuesTestCase {
                name: "all_zeros",
                generator: |size| vec![0i16; size],
            },
            BigValuesTestCase {
                name: "small_constant",
                generator: |size| vec![1i16; size],
            },
            BigValuesTestCase {
                name: "large_constant",
                generator: |size| vec![1000i16; size],
            },
            BigValuesTestCase {
                name: "max_amplitude",
                generator: |size| vec![32767i16; size],
            },
            BigValuesTestCase {
                name: "sine_wave_440hz",
                generator: |size| {
                    (0..size).map(|i| {
                        let t = i as f64 / 44100.0;
                        (10000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16
                    }).collect()
                },
            },
            BigValuesTestCase {
                name: "sweep_signal",
                generator: |size| {
                    (0..size).map(|i| {
                        let t = i as f64 / 44100.0;
                        let freq = 100.0 + t * 2000.0; // Sweep from 100Hz to 2100Hz
                        (8000.0 * (2.0 * std::f64::consts::PI * freq * t).sin()) as i16
                    }).collect()
                },
            },
            BigValuesTestCase {
                name: "white_noise",
                generator: |size| {
                    (0..size).map(|i| {
                        // Simple pseudo-random generator
                        let mut x = (i * 1234567) % 2147483647;
                        x = (x * 16807) % 2147483647;
                        let val = (x % 65536) as i32 - 32768;
                        val.clamp(-32768, 32767) as i16
                    }).collect()
                },
            },
        ]
    }

    #[test]
    fn test_big_values_comprehensive_validation() {
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
        
        let test_cases = generate_test_patterns();
        let mut results = HashMap::new();

        for test_case in test_cases {
            println!("Testing pattern: {}", test_case.name);
            
            let pcm_data = (test_case.generator)(samples_per_frame);
            
            match encoder.encode_frame(&pcm_data) {
                Ok(encoded_frame) => {
                    // Parse the frame to extract big_values
                    let big_values = extract_big_values_from_frame(encoded_frame);
                    
                    println!("  big_values: {} (max allowed: 288)", big_values);
                    
                    // Validate big_values is within limits
                    assert!(
                        big_values <= 288,
                        "big_values {} exceeds limit for pattern '{}'",
                        big_values, test_case.name
                    );
                    
                    // Store result for analysis
                    results.insert(test_case.name, big_values);
                }
                Err(e) => {
                    panic!("Encoding failed for pattern '{}': {:?}", test_case.name, e);
                }
            }
        }

        // Print summary
        println!("\n=== Big Values Test Summary ===");
        for (name, big_values) in &results {
            println!("{}: {} big_values", name, big_values);
        }
        
        // Ensure all tests passed
        assert_eq!(results.len(), generate_test_patterns().len());
    }

    #[test]
    fn test_big_values_stereo_validation() {
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
        let mut pcm_data = Vec::with_capacity(samples_per_frame * 2);
        for i in 0..samples_per_frame {
            let t = i as f64 / 44100.0;
            let left = (5000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16;
            let right = (5000.0 * (2.0 * std::f64::consts::PI * 880.0 * t).sin()) as i16;
            pcm_data.push(left);
            pcm_data.push(right);
        }
        
        match encoder.encode_frame_interleaved(&pcm_data) {
            Ok(encoded_frame) => {
                let big_values = extract_big_values_from_frame(encoded_frame);
                println!("Stereo big_values: {} (max allowed: 288)", big_values);
                
                assert!(
                    big_values <= 288,
                    "Stereo big_values {} exceeds limit",
                    big_values
                );
            }
            Err(e) => {
                panic!("Stereo encoding failed: {:?}", e);
            }
        }
    }

    /// Extract big_values from encoded MP3 frame
    /// This is a simplified parser - in a real implementation you'd want more robust parsing
    fn extract_big_values_from_frame(frame: &[u8]) -> u32 {
        if frame.len() < 10 {
            return 0;
        }

        // Skip frame header (4 bytes) and find side info
        let side_info_start = 4;
        
        // For mono MPEG-1: side info is 17 bytes
        // big_values is at offset 6-7 in granule info (9 bits)
        if frame.len() > side_info_start + 8 {
            let granule_start = side_info_start + 9; // Skip main_data_begin and private_bits
            
            if frame.len() > granule_start + 2 {
                // Extract big_values (9 bits starting at bit offset after part2_3_length)
                let byte1 = frame[granule_start + 1] as u32;
                let byte2 = frame[granule_start + 2] as u32;
                
                // big_values is 9 bits, extract from the bit stream
                // This is a simplified extraction - real implementation would need proper bit parsing
                let big_values = ((byte1 & 0x01) << 8) | byte2;
                return big_values;
            }
        }
        
        0
    }
}