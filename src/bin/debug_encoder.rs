//! Debug encoder for detailed problem diagnosis
//!
//! This tool adds extensive logging to track values through the encoding pipeline
//! to identify where the "big_values too big" error originates.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Simple WAV reader for debugging
fn read_wav_debug(path: &str) -> Result<(Vec<i16>, u32, u16), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    if buffer.len() < 44 {
        return Err("WAV file too small".into());
    }
    
    // Basic WAV parsing (simplified for debugging)
    if &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
        return Err("Not a valid WAV file".into());
    }
    
    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut pcm_data = Vec::new();
    
    let mut pos = 12;
    while pos < buffer.len() - 8 {
        let chunk_id = &buffer[pos..pos+4];
        let chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
        let chunk_data_start = pos + 8;
        let chunk_data_end = chunk_data_start + chunk_size as usize;
        
        if chunk_data_end > buffer.len() {
            break;
        }
        
        match chunk_id {
            b"fmt " => {
                if chunk_size >= 16 {
                    let audio_format = u16::from_le_bytes([buffer[chunk_data_start], buffer[chunk_data_start+1]]);
                    if audio_format == 1 { // PCM
                        channels = u16::from_le_bytes([buffer[chunk_data_start+2], buffer[chunk_data_start+3]]);
                        sample_rate = u32::from_le_bytes([
                            buffer[chunk_data_start+4], buffer[chunk_data_start+5], 
                            buffer[chunk_data_start+6], buffer[chunk_data_start+7]
                        ]);
                        let bits_per_sample = u16::from_le_bytes([buffer[chunk_data_start+14], buffer[chunk_data_start+15]]);
                        
                        println!("DEBUG: WAV format - channels: {}, sample_rate: {}, bits: {}", 
                                channels, sample_rate, bits_per_sample);
                        
                        if bits_per_sample != 16 {
                            return Err("Only 16-bit samples supported".into());
                        }
                    }
                }
            },
            b"data" => {
                for i in (chunk_data_start..chunk_data_end).step_by(2) {
                    if i + 1 < buffer.len() {
                        let sample = i16::from_le_bytes([buffer[i], buffer[i+1]]);
                        pcm_data.push(sample);
                    }
                }
            },
            _ => {}
        }
        
        pos = chunk_data_end;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    println!("DEBUG: Loaded {} PCM samples", pcm_data.len());
    
    // Analyze input signal characteristics
    if !pcm_data.is_empty() {
        let mut min_val = pcm_data[0];
        let mut max_val = pcm_data[0];
        let mut sum = 0i64;
        
        for &sample in &pcm_data {
            min_val = min_val.min(sample);
            max_val = max_val.max(sample);
            sum += sample as i64;
        }
        
        let avg = sum / pcm_data.len() as i64;
        let dynamic_range = max_val as i32 - min_val as i32;
        
        println!("DEBUG: Input signal analysis:");
        println!("  Min: {}, Max: {}, Avg: {}", min_val, max_val, avg);
        println!("  Dynamic range: {}", dynamic_range);
        println!("  Peak amplitude: {}%", (max_val.abs().max(min_val.abs()) as f64 / 32768.0 * 100.0));
    }
    
    Ok((pcm_data, sample_rate, channels))
}

/// Test with minimal input to isolate the problem
fn test_minimal_input() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TESTING MINIMAL INPUT ===");
    
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
    
    let mut encoder = Mp3Encoder::new(config)?;
    let samples_per_frame = encoder.samples_per_frame();
    
    println!("DEBUG: Samples per frame: {}", samples_per_frame);
    
    // Test 1: All zeros
    println!("\n--- Test 1: All zeros ---");
    let zero_data = vec![0i16; samples_per_frame * 2];
    match encoder.encode_frame_interleaved(&zero_data) {
        Ok(frame) => {
            println!("SUCCESS: Zero frame encoded, size: {} bytes", frame.len());
            
            // Save for analysis
            let mut file = File::create("debug_zero_frame.mp3")?;
            file.write_all(frame)?;
        },
        Err(e) => {
            println!("ERROR: Zero frame failed: {:?}", e);
        }
    }
    
    // Test 2: Very small values
    println!("\n--- Test 2: Small constant values ---");
    let small_data = vec![100i16; samples_per_frame * 2];
    match encoder.encode_frame_interleaved(&small_data) {
        Ok(frame) => {
            println!("SUCCESS: Small constant frame encoded, size: {} bytes", frame.len());
        },
        Err(e) => {
            println!("ERROR: Small constant frame failed: {:?}", e);
        }
    }
    
    // Test 3: Simple sine wave (low frequency, low amplitude)
    println!("\n--- Test 3: Simple sine wave ---");
    let mut sine_data = Vec::with_capacity(samples_per_frame * 2);
    for i in 0..samples_per_frame {
        let t = i as f64 / 44100.0;
        let sample = (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16;
        sine_data.push(sample); // Left
        sine_data.push(sample); // Right
    }
    
    match encoder.encode_frame_interleaved(&sine_data) {
        Ok(frame) => {
            println!("SUCCESS: Sine wave frame encoded, size: {} bytes", frame.len());
        },
        Err(e) => {
            println!("ERROR: Sine wave frame failed: {:?}", e);
        }
    }
    
    // Test 4: Mono configuration
    println!("\n--- Test 4: Mono configuration ---");
    let mono_config = Config {
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
    
    let mut mono_encoder = Mp3Encoder::new(mono_config)?;
    let mono_data = vec![500i16; samples_per_frame];
    
    match mono_encoder.encode_frame(&mono_data) {
        Ok(frame) => {
            println!("SUCCESS: Mono frame encoded, size: {} bytes", frame.len());
        },
        Err(e) => {
            println!("ERROR: Mono frame failed: {:?}", e);
        }
    }
    
    Ok(())
}

/// Test with the actual WAV file but with reduced complexity
fn test_actual_wav_simplified(wav_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TESTING ACTUAL WAV (SIMPLIFIED) ===");
    
    let (pcm_data, sample_rate, channels) = read_wav_debug(wav_path)?;
    
    let config = Config {
        wave: WaveConfig {
            channels: if channels == 1 { Channels::Mono } else { Channels::Stereo },
            sample_rate,
        },
        mpeg: MpegConfig {
            mode: if channels == 1 { StereoMode::Mono } else { StereoMode::Stereo },
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut encoder = Mp3Encoder::new(config)?;
    let samples_per_frame = encoder.samples_per_frame();
    let frame_size = samples_per_frame * channels as usize;
    
    println!("DEBUG: Frame size: {} samples", frame_size);
    
    // Test with just the first frame
    if pcm_data.len() >= frame_size {
        println!("\n--- Testing first frame only ---");
        let first_frame = &pcm_data[0..frame_size];
        
        // Analyze this specific frame
        let mut min_val = first_frame[0];
        let mut max_val = first_frame[0];
        for &sample in first_frame {
            min_val = min_val.min(sample);
            max_val = max_val.max(sample);
        }
        
        println!("DEBUG: First frame analysis:");
        println!("  Min: {}, Max: {}", min_val, max_val);
        println!("  Peak: {}%", (max_val.abs().max(min_val.abs()) as f64 / 32768.0 * 100.0));
        
        let result = if channels == 1 {
            encoder.encode_frame(first_frame)
        } else {
            encoder.encode_frame_interleaved(first_frame)
        };
        
        match result {
            Ok(frame) => {
                println!("SUCCESS: First frame encoded, size: {} bytes", frame.len());
                
                // Save for analysis
                let mut file = File::create("debug_first_frame.mp3")?;
                file.write_all(frame)?;
                
                // Try to decode with ffmpeg to see the exact error
                println!("DEBUG: Saved debug_first_frame.mp3 for analysis");
            },
            Err(e) => {
                println!("ERROR: First frame failed: {:?}", e);
            }
        }
        
        // Test with attenuated version (reduce amplitude)
        println!("\n--- Testing attenuated first frame ---");
        let attenuated_frame: Vec<i16> = first_frame.iter()
            .map(|&sample| sample / 4) // Reduce amplitude by 75%
            .collect();
        
        let result = if channels == 1 {
            encoder.encode_frame(&attenuated_frame)
        } else {
            encoder.encode_frame_interleaved(&attenuated_frame)
        };
        
        match result {
            Ok(frame) => {
                println!("SUCCESS: Attenuated frame encoded, size: {} bytes", frame.len());
            },
            Err(e) => {
                println!("ERROR: Attenuated frame failed: {:?}", e);
            }
        }
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.wav>", args[0]);
        eprintln!("This tool performs detailed diagnosis of MP3 encoding problems");
        std::process::exit(1);
    }
    
    let wav_path = &args[1];
    
    if !Path::new(wav_path).exists() {
        eprintln!("Error: WAV file '{}' does not exist", wav_path);
        std::process::exit(1);
    }
    
    println!("MP3 Encoder Debug Tool");
    println!("=====================");
    
    // Test 1: Minimal synthetic inputs
    if let Err(e) = test_minimal_input() {
        eprintln!("Minimal input test failed: {}", e);
    }
    
    // Test 2: Actual WAV file (simplified)
    if let Err(e) = test_actual_wav_simplified(wav_path) {
        eprintln!("WAV file test failed: {}", e);
    }
    
    println!("\n=== DIAGNOSIS COMPLETE ===");
    println!("Check the generated debug files:");
    println!("- debug_zero_frame.mp3 (if generated)");
    println!("- debug_first_frame.mp3 (if generated)");
    println!("\nUse ffmpeg to analyze these files:");
    println!("ffmpeg -v error -i debug_zero_frame.mp3 -f null -");
    println!("ffmpeg -v error -i debug_first_frame.mp3 -f null -");
}