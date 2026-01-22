//! Debug test for big_values issue
//!
//! This test investigates the "big_values too big" error reported by FFmpeg.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::Write;

/// Debug the big_values issue
#[test]
fn test_debug_big_values_issue() {
    create_dir_all("tests/output").expect("Failed to create output directory");
    
    println!("=== Debugging big_values issue ===");
    
    // Create a simple test case that might trigger the issue
    let config = Config {
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
    };
    
    // Generate a simple test signal
    let sample_rate = 44100;
    let duration = 1.0; // 1 second
    let samples_count = (sample_rate as f32 * duration) as usize;
    let channels = 2;
    
    let mut pcm_data = Vec::with_capacity(samples_count * channels);
    
    // Generate a signal that might cause large coefficients
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        
        // Mix of high and low frequencies that might cause large MDCT coefficients
        let low_freq = (t * 100.0 * 2.0 * std::f32::consts::PI).sin() * 20000.0;
        let high_freq = (t * 8000.0 * 2.0 * std::f32::consts::PI).sin() * 15000.0;
        let mixed = (low_freq + high_freq) as i16;
        
        pcm_data.push(mixed); // Left
        pcm_data.push(mixed); // Right
    }
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    let samples_per_frame = encoder.samples_per_frame();
    let frame_size = samples_per_frame * channels;
    let mut mp3_data = Vec::new();
    
    println!("Encoding {} samples in frames of {} samples each", 
             pcm_data.len(), frame_size);
    
    let mut frame_count = 0;
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            match encoder.encode_frame_interleaved(chunk) {
                Ok(frame_data) => {
                    mp3_data.extend_from_slice(frame_data);
                    frame_count += 1;
                    println!("Frame {}: {} bytes", frame_count, frame_data.len());
                },
                Err(e) => {
                    println!("Error encoding frame {}: {:?}", frame_count, e);
                    break;
                }
            }
        }
    }
    
    // Flush
    match encoder.flush() {
        Ok(final_data) => {
            mp3_data.extend_from_slice(final_data);
            if !final_data.is_empty() {
                println!("Flushed: {} bytes", final_data.len());
            }
        },
        Err(e) => {
            println!("Error flushing: {:?}", e);
        }
    }
    
    println!("Total MP3 data: {} bytes", mp3_data.len());
    
    // Write the file
    let filepath = "tests/output/big_values_debug.mp3";
    let mut file = File::create(filepath).expect("Failed to create file");
    file.write_all(&mp3_data).expect("Failed to write MP3 data");
    
    // Analyze the MP3 structure
    analyze_mp3_structure(&mp3_data);
    
    println!("Debug MP3 written to: {}", filepath);
}

/// Analyze MP3 structure to find big_values issues
fn analyze_mp3_structure(mp3_data: &[u8]) {
    println!("\n=== MP3 Structure Analysis ===");
    
    let mut pos = 0;
    let mut frame_count = 0;
    
    while pos < mp3_data.len().saturating_sub(4) {
        // Look for sync word
        let sync = ((mp3_data[pos] as u16) << 3) | ((mp3_data[pos + 1] as u16) >> 5);
        
        if sync == 0x7FF {
            frame_count += 1;
            println!("\n--- Frame {} at position {} ---", frame_count, pos);
            
            if pos + 32 < mp3_data.len() {
                // Parse frame header
                let header = ((mp3_data[pos] as u32) << 24) |
                            ((mp3_data[pos + 1] as u32) << 16) |
                            ((mp3_data[pos + 2] as u32) << 8) |
                            (mp3_data[pos + 3] as u32);
                
                let version = (header >> 19) & 0x3;
                let bitrate_index = (header >> 12) & 0xF;
                let sample_rate_index = (header >> 10) & 0x3;
                let padding = (header >> 9) & 0x1;
                let channel_mode = (header >> 6) & 0x3;
                
                println!("Header: {:08X}", header);
                println!("Version: {}, Bitrate index: {}, Sample rate index: {}", 
                         version, bitrate_index, sample_rate_index);
                println!("Padding: {}, Channel mode: {}", padding, channel_mode);
                
                // Calculate expected frame size
                let frame_size = calculate_frame_size(bitrate_index, sample_rate_index, padding, version);
                println!("Expected frame size: {} bytes", frame_size);
                
                // Look at side information (after 4-byte header)
                if pos + 36 < mp3_data.len() {
                    println!("Side info bytes: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                             mp3_data[pos + 4], mp3_data[pos + 5], mp3_data[pos + 6], mp3_data[pos + 7],
                             mp3_data[pos + 8], mp3_data[pos + 9], mp3_data[pos + 10], mp3_data[pos + 11]);
                    
                    // Parse side info for stereo MPEG-1
                    if version == 3 && channel_mode != 3 { // MPEG-1 stereo
                        let side_info_start = pos + 4;
                        
                        // Skip main_data_begin (9 bits) and private_bits (3 bits) = 12 bits = 1.5 bytes
                        // Skip scfsi (4 bits per channel) = 8 bits = 1 byte
                        // Total: 2.5 bytes, so granule info starts at byte 7 (pos + 4 + 3)
                        
                        if side_info_start + 32 < mp3_data.len() {
                            // Parse granule 0, channel 0 big_values (12 bits starting at specific position)
                            // This is complex bit parsing, let's just check for obviously wrong values
                            
                            // Look for patterns that might indicate big_values issues
                            for i in 0..16 {
                                if side_info_start + i + 1 < mp3_data.len() {
                                    let byte_pair = ((mp3_data[side_info_start + i] as u16) << 8) | 
                                                   (mp3_data[side_info_start + i + 1] as u16);
                                    
                                    // Check for suspiciously large values that might be big_values
                                    if byte_pair > 0x8000 {
                                        println!("Suspicious large value at offset {}: 0x{:04X}", i, byte_pair);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Move to next frame
                if frame_size > 0 && frame_size < 2000 { // Sanity check
                    pos += frame_size;
                } else {
                    pos += 400; // Default skip
                }
            } else {
                pos += 1;
            }
            
            // Only analyze first few frames
            if frame_count >= 3 {
                break;
            }
        } else {
            pos += 1;
        }
    }
    
    println!("\nAnalyzed {} frames", frame_count);
}

/// Calculate MP3 frame size
fn calculate_frame_size(bitrate_index: u32, sample_rate_index: u32, padding: u32, version: u32) -> usize {
    let bitrates = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
    let sample_rates = match version {
        3 => [44100, 48000, 32000, 0], // MPEG-1
        2 => [22050, 24000, 16000, 0], // MPEG-2
        0 => [11025, 12000, 8000, 0],  // MPEG-2.5
        _ => return 0,
    };
    
    if bitrate_index == 0 || bitrate_index == 15 || sample_rate_index == 3 {
        return 0;
    }
    
    let bitrate = bitrates[bitrate_index as usize] * 1000;
    let sample_rate = sample_rates[sample_rate_index as usize];
    
    if bitrate == 0 || sample_rate == 0 {
        return 0;
    }
    
    let samples_per_frame = if version == 3 { 1152 } else { 576 };
    let frame_size = (samples_per_frame * bitrate / sample_rate / 8) + padding as usize;
    
    frame_size
}