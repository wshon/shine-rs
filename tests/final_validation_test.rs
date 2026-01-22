//! Final validation test for task 11.7.4
//!
//! This test validates that our MP3 encoder generates standard-compliant
//! MP3 files that can be decoded by FFmpeg and other standard players.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::process::Command;

/// Comprehensive validation test
#[test]
fn test_comprehensive_mp3_validation() {
    // Ensure output directory exists
    create_dir_all("tests/output").expect("Failed to create output directory");
    
    println!("=== Comprehensive MP3 Validation Test ===");
    
    // Test different configurations
    let test_configs = vec![
        ("mono_44100_128", Config {
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
        }),
        ("stereo_44100_128", Config {
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
        }),
        ("joint_stereo_48000_192", Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 48000,
            },
            mpeg: MpegConfig {
                mode: StereoMode::JointStereo,
                bitrate: 192,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        }),
    ];
    
    for (name, config) in test_configs {
        println!("\n--- Testing configuration: {} ---", name);
        
        // Generate test audio
        let sample_rate = config.wave.sample_rate;
        let channels = config.wave.channels as usize;
        let duration = 3.0; // 3 seconds
        let samples_count = (sample_rate as f32 * duration) as usize;
        
        let mut pcm_data = Vec::with_capacity(samples_count * channels);
        
        // Generate a complex test signal with multiple frequencies
        for i in 0..samples_count {
            let t = i as f32 / sample_rate as f32;
            
            // Mix of different frequencies to test encoding quality
            let freq1 = 440.0; // A4
            let freq2 = 880.0; // A5
            let freq3 = 220.0; // A3
            
            let sample1 = (t * freq1 * 2.0 * std::f32::consts::PI).sin() * 5000.0;
            let sample2 = (t * freq2 * 2.0 * std::f32::consts::PI).sin() * 3000.0;
            let sample3 = (t * freq3 * 2.0 * std::f32::consts::PI).sin() * 2000.0;
            
            let mixed_sample = (sample1 + sample2 + sample3) as i16;
            
            if channels == 1 {
                pcm_data.push(mixed_sample);
            } else {
                pcm_data.push(mixed_sample); // Left channel
                pcm_data.push((mixed_sample as f32 * 0.8) as i16); // Right channel (slightly different)
            }
        }
        
        // Create encoder and encode
        let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
        let samples_per_frame = encoder.samples_per_frame();
        let frame_size = samples_per_frame * channels;
        let mut mp3_data = Vec::new();
        
        println!("Encoding {} samples in frames of {} samples each", 
                 pcm_data.len(), frame_size);
        
        let mut frame_count = 0;
        for chunk in pcm_data.chunks(frame_size) {
            if chunk.len() == frame_size {
                let frame_data = if channels == 1 {
                    encoder.encode_frame(chunk).expect("Failed to encode frame")
                } else {
                    encoder.encode_frame_interleaved(chunk).expect("Failed to encode frame")
                };
                mp3_data.extend_from_slice(frame_data);
                frame_count += 1;
            }
        }
        
        // Flush remaining data
        let final_data = encoder.flush().expect("Failed to flush");
        mp3_data.extend_from_slice(final_data);
        
        println!("Encoded {} frames, total MP3 size: {} bytes", frame_count, mp3_data.len());
        
        // Write MP3 file
        let filename = format!("tests/output/final_validation_{}.mp3", name);
        let mut file = File::create(&filename).expect("Failed to create MP3 file");
        file.write_all(&mp3_data).expect("Failed to write MP3 data");
        
        // Validate with FFmpeg
        validate_with_ffmpeg(&filename, &config);
        
        // Basic format validation
        validate_mp3_format(&mp3_data);
        
        println!("✓ Configuration {} passed all validations", name);
    }
    
    println!("\n=== All validations passed! ===");
}

/// Validate MP3 file with FFmpeg
fn validate_with_ffmpeg(filepath: &str, config: &Config) {
    println!("Validating {} with FFmpeg...", filepath);
    
    // Try to decode with FFmpeg
    let output_wav = format!("{}.validation.wav", filepath);
    let result = Command::new("ffmpeg")
        .args(&[
            "-y", // Overwrite output
            "-v", "error", // Only show errors
            "-i", filepath,
            "-f", "wav",
            &output_wav
        ])
        .output();
    
    match result {
        Ok(output) => {
            if output.status.success() {
                println!("✓ FFmpeg successfully decoded {}", filepath);
                
                // Clean up
                let _ = std::fs::remove_file(&output_wav);
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                panic!("FFmpeg failed to decode {}: {}", filepath, error);
            }
        },
        Err(e) => {
            println!("⚠ FFmpeg not available ({}), skipping decode validation", e);
        }
    }
    
    // Validate with FFprobe
    let result = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration,bit_rate,format_name",
            "-show_entries", "stream=codec_name,sample_rate,channels,bit_rate",
            "-of", "csv=p=0",
            filepath
        ])
        .output();
    
    match result {
        Ok(output) => {
            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout);
                println!("FFprobe info: {}", info.trim());
                
                // Validate format information
                assert!(info.contains("mp3"), "Should be identified as MP3 format");
                assert!(info.contains(&config.wave.sample_rate.to_string()), 
                        "Sample rate should match");
                assert!(info.contains(&(config.wave.channels as u32).to_string()), 
                        "Channel count should match");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("FFprobe error: {}", error);
            }
        },
        Err(e) => {
            println!("⚠ FFprobe not available ({}), skipping metadata validation", e);
        }
    }
}

/// Basic MP3 format validation
fn validate_mp3_format(mp3_data: &[u8]) {
    println!("Validating MP3 format structure...");
    
    // Check minimum file size
    assert!(mp3_data.len() > 100, "MP3 file should be substantial");
    
    // Count sync words
    let mut sync_count = 0;
    let mut pos = 0;
    
    while pos < mp3_data.len().saturating_sub(4) {
        // Look for MP3 sync word (11 bits of 1s)
        let sync = ((mp3_data[pos] as u16) << 3) | ((mp3_data[pos + 1] as u16) >> 5);
        
        if sync == 0x7FF {
            sync_count += 1;
            
            // Basic header validation
            let header = ((mp3_data[pos] as u32) << 24) |
                        ((mp3_data[pos + 1] as u32) << 16) |
                        ((mp3_data[pos + 2] as u32) << 8) |
                        (mp3_data[pos + 3] as u32);
            
            let version = (header >> 19) & 0x3;
            let layer = (header >> 17) & 0x3;
            let bitrate_index = (header >> 12) & 0xF;
            let sample_rate_index = (header >> 10) & 0x3;
            
            // Validate header fields
            assert!(version == 3 || version == 2 || version == 0, "Valid MPEG version");
            assert!(layer == 1, "Should be Layer III");
            assert!(bitrate_index != 0 && bitrate_index != 15, "Valid bitrate index");
            assert!(sample_rate_index != 3, "Valid sample rate index");
            
            // Skip to next potential frame (rough estimate)
            pos += 400; // Approximate frame size
        } else {
            pos += 1;
        }
    }
    
    println!("Found {} valid MP3 sync words", sync_count);
    assert!(sync_count > 0, "Should find at least one valid MP3 frame");
    
    println!("✓ MP3 format validation passed");
}