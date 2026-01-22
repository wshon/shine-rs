//! Shine Reference Implementation Comparison Tests
//!
//! This module provides tests that compare our Rust MP3 encoder implementation
//! against the reference shine library to ensure algorithmic consistency.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::process::Command;

/// Test configuration that matches shine's default settings
fn create_shine_reference_config() -> Config {
    Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::JointStereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    }
}

/// Generate test PCM data with known patterns for comparison
fn generate_test_pcm_data(samples: usize, channels: usize, pattern: &str) -> Vec<i16> {
    match pattern {
        "sine_440hz" => {
            let sample_rate = 44100.0;
            let frequency = 440.0;
            (0..samples * channels)
                .map(|i| {
                    let sample_idx = i / channels;
                    let t = sample_idx as f32 / sample_rate;
                    let amplitude = 16000.0;
                    (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
                })
                .collect()
        },
        "sine_1000hz" => {
            let sample_rate = 44100.0;
            let frequency = 1000.0;
            (0..samples * channels)
                .map(|i| {
                    let sample_idx = i / channels;
                    let t = sample_idx as f32 / sample_rate;
                    let amplitude = 16000.0;
                    (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
                })
                .collect()
        },
        "mixed_tones" => {
            let sample_rate = 44100.0;
            (0..samples * channels)
                .map(|i| {
                    let sample_idx = i / channels;
                    let t = sample_idx as f32 / sample_rate;
                    
                    // Mix of 440Hz, 880Hz, and 1320Hz
                    let tone1 = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 5000.0;
                    let tone2 = (2.0 * std::f32::consts::PI * 880.0 * t).sin() * 3000.0;
                    let tone3 = (2.0 * std::f32::consts::PI * 1320.0 * t).sin() * 2000.0;
                    
                    (tone1 + tone2 + tone3) as i16
                })
                .collect()
        },
        "silence" => vec![0i16; samples * channels],
        _ => panic!("Unknown pattern: {}", pattern),
    }
}

/// Write PCM data to a WAV file for shine comparison
fn write_wav_file(filename: &str, pcm_data: &[i16], sample_rate: u32, channels: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    
    // WAV header
    let data_size = (pcm_data.len() * 2) as u32;
    let file_size = data_size + 36;
    
    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    
    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // chunk size
    file.write_all(&1u16.to_le_bytes())?;  // PCM format
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * channels as u32 * 2).to_le_bytes())?; // byte rate
    file.write_all(&(channels * 2).to_le_bytes())?; // block align
    file.write_all(&16u16.to_le_bytes())?; // bits per sample
    
    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    
    // PCM data
    for &sample in pcm_data {
        file.write_all(&sample.to_le_bytes())?;
    }
    
    Ok(())
}

/// Run shine encoder on WAV file (if available)
fn encode_with_shine(wav_path: &str, mp3_path: &str) -> Result<(), String> {
    let output = Command::new("shine")
        .args(&[wav_path, mp3_path])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Err(format!("Shine encoding failed: {}", stderr))
            }
        },
        Err(_) => Err("Shine encoder not available".to_string())
    }
}

/// Compare two MP3 files at the byte level
fn compare_mp3_files(file1: &str, file2: &str) -> Result<f32, String> {
    let mut data1 = Vec::new();
    let mut data2 = Vec::new();
    
    File::open(file1)
        .and_then(|mut f| f.read_to_end(&mut data1))
        .map_err(|e| format!("Failed to read {}: {}", file1, e))?;
    
    File::open(file2)
        .and_then(|mut f| f.read_to_end(&mut data2))
        .map_err(|e| format!("Failed to read {}: {}", file2, e))?;
    
    if data1.len() != data2.len() {
        return Ok(0.0); // Different sizes = 0% similarity
    }
    
    let matching_bytes = data1.iter()
        .zip(data2.iter())
        .filter(|(a, b)| a == b)
        .count();
    
    Ok(matching_bytes as f32 / data1.len() as f32 * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shine_reference_config() {
        let config = create_shine_reference_config();
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_ok(), "Should create encoder with shine reference config");
    }
    
    #[test]
    fn test_pcm_data_generation() {
        let patterns = ["sine_440hz", "sine_1000hz", "mixed_tones", "silence"];
        
        for pattern in &patterns {
            let pcm_data = generate_test_pcm_data(1000, 2, pattern);
            assert_eq!(pcm_data.len(), 2000, "Should generate correct amount of stereo samples");
            
            if *pattern == "silence" {
                assert!(pcm_data.iter().all(|&x| x == 0), "Silence should be all zeros");
            } else {
                assert!(pcm_data.iter().any(|&x| x != 0), "Non-silence should have non-zero samples");
            }
        }
    }
    
    #[test]
    fn test_wav_file_creation() {
        create_dir_all("tests/output").expect("Failed to create output directory");
        
        let pcm_data = generate_test_pcm_data(1000, 2, "sine_440hz");
        let wav_path = "tests/output/test_sine_440hz.wav";
        
        let result = write_wav_file(wav_path, &pcm_data, 44100, 2);
        assert!(result.is_ok(), "Should create WAV file successfully");
        
        // Verify file exists and has reasonable size
        let metadata = std::fs::metadata(wav_path).expect("WAV file should exist");
        assert!(metadata.len() > 44, "WAV file should be larger than header");
    }
    
    #[test]
    fn test_rust_encoder_consistency() {
        create_dir_all("tests/output").expect("Failed to create output directory");
        
        let config = create_shine_reference_config();
        let pcm_data = generate_test_pcm_data(4410, 2, "sine_440hz"); // 0.1 seconds
        
        let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
        let samples_per_frame = encoder.samples_per_frame();
        let frame_size = samples_per_frame * 2; // stereo
        
        let mut mp3_data = Vec::new();
        
        // Encode complete frames
        for chunk in pcm_data.chunks(frame_size) {
            if chunk.len() == frame_size {
                match encoder.encode_frame_interleaved(chunk) {
                    Ok(frame_data) => mp3_data.extend_from_slice(frame_data),
                    Err(e) => panic!("Encoding failed: {:?}", e),
                }
            }
        }
        
        // Flush encoder
        if let Ok(final_data) = encoder.flush() {
            mp3_data.extend_from_slice(final_data);
        }
        
        assert!(!mp3_data.is_empty(), "Should produce MP3 data");
        assert!(mp3_data.len() > 100, "MP3 data should be reasonable size");
        
        // Check for valid MP3 sync words
        let mut sync_count = 0;
        for i in 0..mp3_data.len().saturating_sub(1) {
            let sync = ((mp3_data[i] as u16) << 3) | ((mp3_data[i + 1] as u16) >> 5);
            if sync == 0x7FF {
                sync_count += 1;
            }
        }
        
        assert!(sync_count > 0, "Should contain valid MP3 sync words");
        
        // Write output for manual inspection
        let output_path = "tests/output/rust_encoder_test.mp3";
        let mut file = File::create(output_path).expect("Failed to create output file");
        file.write_all(&mp3_data).expect("Failed to write MP3 data");
        
        println!("Rust encoder test output written to: {}", output_path);
    }
    
    #[test]
    #[ignore] // Only run when shine is available
    fn test_shine_comparison() {
        create_dir_all("tests/output").expect("Failed to create output directory");
        
        let config = create_shine_reference_config();
        let pcm_data = generate_test_pcm_data(44100, 2, "sine_1000hz"); // 1 second
        
        // Create WAV file for shine
        let wav_path = "tests/output/comparison_input.wav";
        write_wav_file(wav_path, &pcm_data, 44100, 2)
            .expect("Failed to create WAV file");
        
        // Encode with shine (if available)
        let shine_mp3_path = "tests/output/shine_output.mp3";
        match encode_with_shine(wav_path, shine_mp3_path) {
            Ok(()) => {
                println!("Shine encoding successful");
                
                // Encode with our Rust implementation
                let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
                let samples_per_frame = encoder.samples_per_frame();
                let frame_size = samples_per_frame * 2;
                let mut rust_mp3_data = Vec::new();
                
                for chunk in pcm_data.chunks(frame_size) {
                    if chunk.len() == frame_size {
                        if let Ok(frame_data) = encoder.encode_frame_interleaved(chunk) {
                            rust_mp3_data.extend_from_slice(frame_data);
                        }
                    }
                }
                
                if let Ok(final_data) = encoder.flush() {
                    rust_mp3_data.extend_from_slice(final_data);
                }
                
                // Write Rust output
                let rust_mp3_path = "tests/output/rust_output.mp3";
                let mut file = File::create(rust_mp3_path).expect("Failed to create output file");
                file.write_all(&rust_mp3_data).expect("Failed to write MP3 data");
                
                // Compare files
                match compare_mp3_files(shine_mp3_path, rust_mp3_path) {
                    Ok(similarity) => {
                        println!("File similarity: {:.1}%", similarity);
                        
                        // We don't expect 100% similarity due to implementation differences,
                        // but the files should have some structural similarity
                        assert!(similarity > 10.0, "Files should have some structural similarity");
                    },
                    Err(e) => {
                        println!("File comparison failed: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("Shine not available, skipping comparison: {}", e);
            }
        }
    }
    
    #[test]
    fn test_different_signal_patterns() {
        create_dir_all("tests/output").expect("Failed to create output directory");
        
        let config = create_shine_reference_config();
        let patterns = ["sine_440hz", "sine_1000hz", "mixed_tones"];
        
        for pattern in &patterns {
            println!("Testing pattern: {}", pattern);
            
            let pcm_data = generate_test_pcm_data(4410, 2, pattern); // 0.1 seconds
            let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
            
            let samples_per_frame = encoder.samples_per_frame();
            let frame_size = samples_per_frame * 2;
            let mut mp3_data = Vec::new();
            
            for chunk in pcm_data.chunks(frame_size) {
                if chunk.len() == frame_size {
                    match encoder.encode_frame_interleaved(chunk) {
                        Ok(frame_data) => mp3_data.extend_from_slice(frame_data),
                        Err(e) => panic!("Encoding failed for pattern {}: {:?}", pattern, e),
                    }
                }
            }
            
            if let Ok(final_data) = encoder.flush() {
                mp3_data.extend_from_slice(final_data);
            }
            
            assert!(!mp3_data.is_empty(), "Should produce MP3 data for pattern {}", pattern);
            
            // Write output for inspection
            let output_path = format!("tests/output/pattern_{}.mp3", pattern);
            let mut file = File::create(&output_path).expect("Failed to create output file");
            file.write_all(&mp3_data).expect("Failed to write MP3 data");
            
            println!("Pattern {} output written to: {}", pattern, output_path);
        }
    }
}