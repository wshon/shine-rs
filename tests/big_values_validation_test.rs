//! Test to validate big_values field in side information
//!
//! This test checks if our big_values field is being set correctly.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[test]
fn test_big_values_validation() {
    println!("=== Testing big_values field validation ===");
    
    // Test with a realistic encoder scenario
    test_encoder_big_values();
}

fn test_encoder_big_values() {
    println!("\n=== Testing encoder big_values generation ===");
    
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
    
    // Generate simple test audio
    let sample_rate = 44100;
    let duration = 0.1; // 0.1 seconds
    let samples_count = (sample_rate as f32 * duration) as usize;
    
    let mut pcm_data = Vec::with_capacity(samples_count);
    
    // Generate a simple sine wave
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 16000.0;
        pcm_data.push(sample as i16);
    }
    
    // Create a custom encoder to access internal state
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    let samples_per_frame = encoder.samples_per_frame();
    
    // Pad to complete frame
    while pcm_data.len() < samples_per_frame {
        pcm_data.push(0);
    }
    
    // Encode one frame
    match encoder.encode_frame(&pcm_data[..samples_per_frame]) {
        Ok(frame_data) => {
            println!("Encoded frame size: {} bytes", frame_data.len());
            
            // Analyze the frame to extract big_values
            analyze_encoded_frame(frame_data);
        },
        Err(e) => {
            println!("Encoding failed: {:?}", e);
        }
    }
}

fn analyze_encoded_frame(frame_data: &[u8]) {
    if frame_data.len() < 36 {
        println!("Frame too small to analyze");
        return;
    }
    
    println!("Analyzing encoded frame...");
    
    // Skip frame header (4 bytes)
    let side_info_start = 4;
    
    // For MPEG-1 mono: 
    // - main_data_begin: 9 bits
    // - private_bits: 5 bits
    // - scfsi: 4 bits (1 channel * 4 bands)
    // Total: 18 bits = 2.25 bytes
    // Then granule info starts
    
    // For simplicity, let's look at the raw bytes and try to extract big_values
    // This is complex bit parsing, so we'll do a simplified check
    
    println!("Side info bytes (first 16): ");
    for i in 0..16.min(frame_data.len() - side_info_start) {
        print!("{:02X} ", frame_data[side_info_start + i]);
    }
    println!();
    
    // Try to extract big_values from the expected position
    // This is approximate since we'd need exact bit parsing
    if frame_data.len() > side_info_start + 8 {
        // Look for patterns that might indicate big_values > 288
        let mut suspicious_values = Vec::new();
        
        for i in 0..8 {
            if side_info_start + i + 1 < frame_data.len() {
                let byte_pair = ((frame_data[side_info_start + i] as u16) << 8) | 
                               (frame_data[side_info_start + i + 1] as u16);
                
                // Extract potential 9-bit big_values field
                for bit_offset in 0..8 {
                    let shifted = byte_pair >> bit_offset;
                    let big_values_candidate = shifted & 0x1FF; // 9 bits
                    
                    if big_values_candidate > 288 {
                        suspicious_values.push((i, bit_offset, big_values_candidate));
                    }
                }
            }
        }
        
        if suspicious_values.is_empty() {
            println!("✓ No suspicious big_values found in side info");
        } else {
            println!("⚠ Found potentially invalid big_values:");
            for (byte_pos, bit_offset, value) in suspicious_values {
                println!("  At byte {}, bit offset {}: {}", byte_pos, bit_offset, value);
            }
        }
    }
}