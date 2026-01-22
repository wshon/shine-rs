//! Pipeline debugging tests
//!
//! This module tests the complete encoding pipeline step by step
//! to identify where the 0xFF byte issue originates.

use rust_mp3_encoder::{Mp3Encoder, Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[test]
fn test_pipeline_debug_silence() {
    println!("\n🔍 Debugging complete pipeline with silence");
    
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
    
    // Create silent audio (all zeros)
    let samples = vec![0i16; 2304]; // 1152 samples per channel * 2 channels
    
    println!("Input: {} samples of silence", samples.len());
    
    // Check if all samples are really zero
    let non_zero_count = samples.iter().filter(|&&x| x != 0).count();
    println!("Non-zero input samples: {}", non_zero_count);
    
    match encoder.encode_frame(&samples) {
        Ok(mp3_data) => {
            println!("Encoding succeeded: {} bytes", mp3_data.len());
            
            // Detailed analysis of the output
            let ff_count = mp3_data.iter().filter(|&&b| b == 0xFF).count();
            let ff_percentage = if mp3_data.len() > 0 { 
                (ff_count as f32 / mp3_data.len() as f32) * 100.0 
            } else { 
                0.0 
            };
            
            println!("0xFF bytes: {} ({:.1}%)", ff_count, ff_percentage);
            
            // Show the complete output in hex
            println!("Complete output ({} bytes):", mp3_data.len());
            for (i, &byte) in mp3_data.iter().enumerate() {
                if i % 16 == 0 {
                    print!("{:04X}: ", i);
                }
                print!("{:02X} ", byte);
                if i % 16 == 15 {
                    println!();
                }
            }
            if mp3_data.len() % 16 != 0 {
                println!();
            }
            
            // Analyze frame structure
            if mp3_data.len() >= 4 {
                let header = u32::from_be_bytes([mp3_data[0], mp3_data[1], mp3_data[2], mp3_data[3]]);
                println!("MP3 header: 0x{:08X}", header);
                
                // Check if it's a valid MP3 header
                if mp3_data[0] == 0xFF && (mp3_data[1] & 0xE0) == 0xE0 {
                    println!("✓ Valid MP3 sync word");
                    
                    // Decode some header fields
                    let version = (mp3_data[1] >> 3) & 0x03;
                    let layer = (mp3_data[1] >> 1) & 0x03;
                    let bitrate_index = (mp3_data[2] >> 4) & 0x0F;
                    let sample_rate_index = (mp3_data[2] >> 2) & 0x03;
                    
                    println!("  Version: {}, Layer: {}, Bitrate index: {}, Sample rate index: {}", 
                             version, layer, bitrate_index, sample_rate_index);
                } else {
                    println!("❌ Invalid MP3 sync word");
                }
            }
            
            // Look for patterns in the data
            let mut consecutive_ff = 0;
            let mut max_consecutive_ff = 0;
            
            for &byte in mp3_data.iter() {
                if byte == 0xFF {
                    consecutive_ff += 1;
                    max_consecutive_ff = max_consecutive_ff.max(consecutive_ff);
                } else {
                    consecutive_ff = 0;
                }
            }
            
            println!("Maximum consecutive 0xFF bytes: {}", max_consecutive_ff);
            
            if max_consecutive_ff > 10 {
                println!("❌ Too many consecutive 0xFF bytes - indicates encoding problem");
            } else {
                println!("✓ Consecutive 0xFF bytes within reasonable range");
            }
        },
        Err(e) => {
            println!("❌ Encoding failed: {:?}", e);
        }
    }
}

#[test]
fn test_pipeline_debug_small_signal() {
    println!("\n🔍 Debugging complete pipeline with small signal");
    
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
    
    // Create very small amplitude signal
    let mut samples = vec![0i16; 2304];
    for i in 0..samples.len() {
        samples[i] = (100.0 * (i as f32 * 0.01).sin()) as i16; // Small sine wave
    }
    
    println!("Input: {} samples of small signal", samples.len());
    
    // Check signal characteristics
    let max_amplitude = samples.iter().map(|&x| x.abs()).max().unwrap_or(0);
    let non_zero_count = samples.iter().filter(|&&x| x != 0).count();
    println!("Max amplitude: {}, Non-zero samples: {}", max_amplitude, non_zero_count);
    
    match encoder.encode_frame(&samples) {
        Ok(mp3_data) => {
            println!("Encoding succeeded: {} bytes", mp3_data.len());
            
            let ff_count = mp3_data.iter().filter(|&&b| b == 0xFF).count();
            let ff_percentage = if mp3_data.len() > 0 { 
                (ff_count as f32 / mp3_data.len() as f32) * 100.0 
            } else { 
                0.0 
            };
            
            println!("0xFF bytes: {} ({:.1}%)", ff_count, ff_percentage);
            
            // Compare with silence case
            if ff_percentage < 10.0 {
                println!("✓ Small signal produces much fewer 0xFF bytes than silence");
                println!("  This confirms the issue is specific to all-zero/near-zero inputs");
            } else {
                println!("❌ Small signal still produces too many 0xFF bytes");
            }
        },
        Err(e) => {
            println!("❌ Encoding failed: {:?}", e);
        }
    }
}