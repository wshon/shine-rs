//! Big values debugging tool
//!
//! This tool adds detailed logging to track big_values calculation
//! and identify exactly where the value becomes too large.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::env;

/// Test with minimal input and detailed big_values logging
fn test_with_big_values_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== BIG_VALUES DEBUGGING ===");
    
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
    
    let mut encoder = Mp3Encoder::new(config)?;
    let samples_per_frame = encoder.samples_per_frame();
    
    println!("DEBUG: Samples per frame: {}", samples_per_frame);
    
    // Test 1: All zeros (should have big_values = 0)
    println!("\n--- Test 1: All zeros ---");
    let zero_data = vec![0i16; samples_per_frame];
    match encoder.encode_frame(&zero_data) {
        Ok(frame) => {
            println!("SUCCESS: Zero frame encoded, size: {} bytes", frame.len());
            analyze_mp3_frame(frame, "zero_frame");
        },
        Err(e) => {
            println!("ERROR: Zero frame failed: {:?}", e);
        }
    }
    
    // Test 2: Very small constant (should have small big_values)
    println!("\n--- Test 2: Small constant (1) ---");
    let small_data = vec![1i16; samples_per_frame];
    match encoder.encode_frame(&small_data) {
        Ok(frame) => {
            println!("SUCCESS: Small constant frame encoded, size: {} bytes", frame.len());
            analyze_mp3_frame(frame, "small_constant");
        },
        Err(e) => {
            println!("ERROR: Small constant frame failed: {:?}", e);
        }
    }
    
    // Test 3: Larger constant (might cause big_values issues)
    println!("\n--- Test 3: Larger constant (1000) ---");
    let large_data = vec![1000i16; samples_per_frame];
    match encoder.encode_frame(&large_data) {
        Ok(frame) => {
            println!("SUCCESS: Large constant frame encoded, size: {} bytes", frame.len());
            analyze_mp3_frame(frame, "large_constant");
        },
        Err(e) => {
            println!("ERROR: Large constant frame failed: {:?}", e);
        }
    }
    
    // Test 4: Maximum amplitude
    println!("\n--- Test 4: Maximum amplitude (32767) ---");
    let max_data = vec![32767i16; samples_per_frame];
    match encoder.encode_frame(&max_data) {
        Ok(frame) => {
            println!("SUCCESS: Max amplitude frame encoded, size: {} bytes", frame.len());
            analyze_mp3_frame(frame, "max_amplitude");
        },
        Err(e) => {
            println!("ERROR: Max amplitude frame failed: {:?}", e);
        }
    }
    
    // Test 5: Simple sine wave
    println!("\n--- Test 5: Simple sine wave ---");
    let sine_data: Vec<i16> = (0..samples_per_frame)
        .map(|i| {
            let t = i as f64 / 44100.0;
            (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16
        })
        .collect();
    
    match encoder.encode_frame(&sine_data) {
        Ok(frame) => {
            println!("SUCCESS: Sine wave frame encoded, size: {} bytes", frame.len());
            analyze_mp3_frame(frame, "sine_wave");
        },
        Err(e) => {
            println!("ERROR: Sine wave frame failed: {:?}", e);
        }
    }
    
    Ok(())
}

/// Analyze MP3 frame to extract big_values from side information
fn analyze_mp3_frame(frame: &[u8], name: &str) {
    if frame.len() < 32 {
        println!("  Frame too small for analysis");
        return;
    }
    
    // Parse MP3 frame header
    let header = ((frame[0] as u32) << 24) | 
                ((frame[1] as u32) << 16) | 
                ((frame[2] as u32) << 8) | 
                (frame[3] as u32);
    
    // Check sync word
    if (header >> 21) != 0x7FF {
        println!("  Invalid sync word: {:X}", header >> 21);
        return;
    }
    
    // Extract frame info
    let mpeg_version = (header >> 19) & 0x3;
    let layer = (header >> 17) & 0x3;
    let bitrate_index = (header >> 12) & 0xF;
    let sample_rate_index = (header >> 10) & 0x3;
    let padding = (header >> 9) & 0x1;
    let mode = (header >> 6) & 0x3;
    
    println!("  Frame header analysis:");
    println!("    MPEG version: {}, Layer: {}, Bitrate index: {}", mpeg_version, layer, bitrate_index);
    println!("    Sample rate index: {}, Padding: {}, Mode: {}", sample_rate_index, padding, mode);
    
    // Calculate side info offset (after 4-byte header)
    let side_info_start = 4;
    
    // For MPEG-1 Layer III mono: side info is 17 bytes
    // For MPEG-1 Layer III stereo: side info is 32 bytes
    let side_info_len = if mode == 3 { 17 } else { 32 }; // Mode 3 = mono
    
    if frame.len() < side_info_start + side_info_len {
        println!("  Frame too small for side info analysis");
        return;
    }
    
    println!("  Side info analysis:");
    
    // Extract side info bytes
    let side_info = &frame[side_info_start..side_info_start + side_info_len];
    
    // Parse side info for mono MPEG-1
    if mode == 3 { // Mono
        // Skip main_data_begin (9 bits) + private_bits (5 bits) + scfsi (4 bits) = 18 bits total
        let mut bit_offset = 18;
        
        // For each granule (2 granules in MPEG-1)
        for gr in 0..2 {
            println!("    Granule {}:", gr);
            
            // Extract part2_3_length (12 bits)
            let part2_3_length = extract_bits(side_info, bit_offset, 12);
            bit_offset += 12;
            
            // Extract big_values (9 bits) - THIS IS THE KEY FIELD
            let big_values = extract_bits(side_info, bit_offset, 9);
            bit_offset += 9;
            
            // Extract global_gain (8 bits)
            let global_gain = extract_bits(side_info, bit_offset, 8);
            bit_offset += 8;
            
            println!("      part2_3_length: {}", part2_3_length);
            println!("      big_values: {} (MAX ALLOWED: 288)", big_values);
            println!("      global_gain: {}", global_gain);
            
            if big_values > 288 {
                println!("      ❌ BIG_VALUES TOO BIG: {} > 288", big_values);
            } else {
                println!("      ✅ big_values within limits");
            }
            
            // Skip remaining fields for this analysis
            // scalefac_compress(4) + window_switching_flag(1) + table_select(15) + 
            // region0_count(4) + region1_count(3) + preflag(1) + scalefac_scale(1) + count1table_select(1) = 30 bits
            bit_offset += 30;
        }
    }
    
    println!("  Analysis complete for {}", name);
}

/// Extract bits from byte array at given bit offset
fn extract_bits(data: &[u8], bit_offset: usize, num_bits: usize) -> u16 {
    let mut result = 0u16;
    
    for i in 0..num_bits {
        let byte_index = (bit_offset + i) / 8;
        let bit_index = 7 - ((bit_offset + i) % 8);
        
        if byte_index < data.len() {
            let bit = (data[byte_index] >> bit_index) & 1;
            result = (result << 1) | (bit as u16);
        }
    }
    
    result
}

fn main() {
    println!("Big Values Debug Tool");
    println!("====================");
    
    if let Err(e) = test_with_big_values_logging() {
        eprintln!("Debug test failed: {}", e);
    }
    
    println!("\n=== BIG_VALUES DEBUGGING COMPLETE ===");
}