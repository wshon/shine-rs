//! Deep debugging tool to examine encoding pipeline step by step
//!
//! This tool adds logging at each stage of the encoding process to identify
//! exactly where the "big_values too big" problem originates.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::env;
use std::fs::File;
use std::io::{Read, Write};

/// Create a multi-frame MP3 file for proper validation
fn create_multi_frame_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== CREATING MULTI-FRAME TEST ===");
    
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
    let mut mp3_data = Vec::new();
    
    println!("DEBUG: Creating 10 frames for validation");
    
    // Create 10 frames with different patterns
    for frame_num in 0..10 {
        println!("DEBUG: Encoding frame {}", frame_num + 1);
        
        let frame_data: Vec<i16> = (0..samples_per_frame)
            .flat_map(|i| {
                let t = (i + frame_num * samples_per_frame) as f64 / 44100.0;
                let freq = 440.0 + (frame_num as f64 * 110.0); // Different frequency per frame
                let amplitude = 1000.0 * (1.0 - frame_num as f64 * 0.1); // Decreasing amplitude
                let sample = (amplitude * (2.0 * std::f64::consts::PI * freq * t).sin()) as i16;
                vec![sample, sample] // Stereo
            })
            .collect();
        
        match encoder.encode_frame_interleaved(&frame_data) {
            Ok(encoded_frame) => {
                println!("  SUCCESS: Frame {} encoded, size: {} bytes", frame_num + 1, encoded_frame.len());
                mp3_data.extend_from_slice(encoded_frame);
            },
            Err(e) => {
                println!("  ERROR: Frame {} failed: {:?}", frame_num + 1, e);
                return Err(format!("Frame {} encoding failed: {:?}", frame_num + 1, e).into());
            }
        }
    }
    
    // Save multi-frame file
    let mut file = File::create("debug_multi_frame.mp3")?;
    file.write_all(&mp3_data)?;
    
    println!("DEBUG: Created debug_multi_frame.mp3 with {} bytes", mp3_data.len());
    
    Ok(())
}

/// Test with the original WAV file using the same approach as wav2mp3
fn test_full_wav_encoding(wav_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TESTING FULL WAV ENCODING ===");
    
    // Read WAV file (simplified version)
    let mut file = File::open(wav_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Parse WAV (basic parsing)
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;
    let mut pcm_data = Vec::new();
    
    let mut pos = 12;
    while pos < buffer.len() - 8 {
        if pos + 8 > buffer.len() { break; }
        
        let chunk_id = &buffer[pos..pos+4];
        let chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
        let chunk_data_start = pos + 8;
        let chunk_data_end = chunk_data_start + chunk_size as usize;
        
        if chunk_data_end > buffer.len() { break; }
        
        match chunk_id {
            b"fmt " => {
                if chunk_size >= 16 {
                    channels = u16::from_le_bytes([buffer[chunk_data_start+2], buffer[chunk_data_start+3]]);
                    sample_rate = u32::from_le_bytes([
                        buffer[chunk_data_start+4], buffer[chunk_data_start+5], 
                        buffer[chunk_data_start+6], buffer[chunk_data_start+7]
                    ]);
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
        if chunk_size % 2 == 1 { pos += 1; }
    }
    
    println!("DEBUG: WAV loaded - {} Hz, {} channels, {} samples", 
             sample_rate, channels, pcm_data.len());
    
    // Create encoder configuration
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
    let mut mp3_data = Vec::new();
    
    println!("DEBUG: Frame size: {} samples, encoding {} frames", 
             frame_size, pcm_data.len() / frame_size);
    
    // Encode frames one by one with detailed logging
    let mut frame_count = 0;
    for (chunk_idx, chunk) in pcm_data.chunks(frame_size).enumerate() {
        if chunk.len() == frame_size {
            println!("DEBUG: Encoding frame {} of {}", chunk_idx + 1, pcm_data.len() / frame_size);
            
            // Analyze this chunk
            let mut min_val = chunk[0];
            let mut max_val = chunk[0];
            for &sample in chunk {
                min_val = min_val.min(sample);
                max_val = max_val.max(sample);
            }
            
            println!("  Frame {} analysis: min={}, max={}, peak={}%", 
                     chunk_idx + 1, min_val, max_val, 
                     (max_val.abs().max(min_val.abs()) as f64 / 32768.0 * 100.0));
            
            let result = if channels == 1 {
                encoder.encode_frame(chunk)
            } else {
                encoder.encode_frame_interleaved(chunk)
            };
            
            match result {
                Ok(encoded_frame) => {
                    println!("  SUCCESS: Frame {} encoded, size: {} bytes", chunk_idx + 1, encoded_frame.len());
                    mp3_data.extend_from_slice(encoded_frame);
                    frame_count += 1;
                    
                    // Stop after first few frames to avoid too much output
                    if frame_count >= 5 {
                        println!("DEBUG: Stopping after 5 frames for analysis");
                        break;
                    }
                },
                Err(e) => {
                    println!("  ERROR: Frame {} failed: {:?}", chunk_idx + 1, e);
                    
                    // Try with reduced amplitude
                    println!("  RETRY: Trying with 50% amplitude");
                    let reduced_chunk: Vec<i16> = chunk.iter().map(|&s| s / 2).collect();
                    
                    let retry_result = if channels == 1 {
                        encoder.encode_frame(&reduced_chunk)
                    } else {
                        encoder.encode_frame_interleaved(&reduced_chunk)
                    };
                    
                    match retry_result {
                        Ok(encoded_frame) => {
                            println!("  SUCCESS: Frame {} encoded with reduced amplitude, size: {} bytes", 
                                     chunk_idx + 1, encoded_frame.len());
                            mp3_data.extend_from_slice(encoded_frame);
                        },
                        Err(e2) => {
                            println!("  ERROR: Frame {} still failed with reduced amplitude: {:?}", chunk_idx + 1, e2);
                            return Err(format!("Frame {} encoding failed even with reduced amplitude", chunk_idx + 1).into());
                        }
                    }
                }
            }
        }
    }
    
    // Save the result
    if !mp3_data.is_empty() {
        let mut file = File::create("debug_wav_encoded.mp3")?;
        file.write_all(&mp3_data)?;
        println!("DEBUG: Created debug_wav_encoded.mp3 with {} bytes", mp3_data.len());
    }
    
    Ok(())
}

/// Test different bitrates to see if the problem is bitrate-specific
fn test_different_bitrates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TESTING DIFFERENT BITRATES ===");
    
    let bitrates = [32, 64, 96, 128, 160, 192, 256, 320];
    
    for &bitrate in &bitrates {
        println!("\n--- Testing bitrate: {} kbps ---", bitrate);
        
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Stereo,
                bitrate,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        match Mp3Encoder::new(config) {
            Ok(mut encoder) => {
                let samples_per_frame = encoder.samples_per_frame();
                
                // Create test data
                let test_data: Vec<i16> = (0..samples_per_frame * 2)
                    .map(|i| {
                        let t = i as f64 / 44100.0;
                        (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16
                    })
                    .collect();
                
                match encoder.encode_frame_interleaved(&test_data) {
                    Ok(frame) => {
                        println!("  SUCCESS: {} kbps encoded, size: {} bytes", bitrate, frame.len());
                    },
                    Err(e) => {
                        println!("  ERROR: {} kbps failed: {:?}", bitrate, e);
                    }
                }
            },
            Err(e) => {
                println!("  ERROR: {} kbps encoder creation failed: {:?}", bitrate, e);
            }
        }
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    println!("Deep MP3 Encoder Debug Tool");
    println!("===========================");
    
    // Test 1: Multi-frame encoding
    if let Err(e) = create_multi_frame_test() {
        eprintln!("Multi-frame test failed: {}", e);
    }
    
    // Test 2: Different bitrates
    if let Err(e) = test_different_bitrates() {
        eprintln!("Bitrate test failed: {}", e);
    }
    
    // Test 3: Full WAV encoding if provided
    if args.len() >= 2 {
        let wav_path = &args[1];
        if std::path::Path::new(wav_path).exists() {
            if let Err(e) = test_full_wav_encoding(wav_path) {
                eprintln!("WAV encoding test failed: {}", e);
            }
        }
    }
    
    println!("\n=== DEEP DIAGNOSIS COMPLETE ===");
    println!("Generated files for analysis:");
    println!("- debug_multi_frame.mp3");
    println!("- debug_wav_encoded.mp3 (if WAV provided)");
    println!("\nTest these with:");
    println!("ffmpeg -v error -i debug_multi_frame.mp3 -f null -");
    println!("mpck debug_multi_frame.mp3 -v");
}