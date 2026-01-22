//! Frame header debugging test
//!
//! This test analyzes the MP3 frame headers to ensure they are correctly formatted.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::Write;

/// Analyze MP3 frame headers
fn analyze_frame_headers(mp3_data: &[u8]) {
    println!("Analyzing MP3 frame headers...");
    println!("Total file size: {} bytes", mp3_data.len());
    
    let mut pos = 0;
    let mut frame_count = 0;
    
    while pos < mp3_data.len().saturating_sub(4) {
        // Look for sync word (11 bits of 1s)
        let sync = ((mp3_data[pos] as u16) << 3) | ((mp3_data[pos + 1] as u16) >> 5);
        
        if sync == 0x7FF {
            frame_count += 1;
            println!("\n--- Frame {} at position {} ---", frame_count, pos);
            
            // Parse frame header
            let header = ((mp3_data[pos] as u32) << 24) |
                        ((mp3_data[pos + 1] as u32) << 16) |
                        ((mp3_data[pos + 2] as u32) << 8) |
                        (mp3_data[pos + 3] as u32);
            
            println!("Header bytes: {:02X} {:02X} {:02X} {:02X}", 
                     mp3_data[pos], mp3_data[pos + 1], mp3_data[pos + 2], mp3_data[pos + 3]);
            
            // Decode header fields
            let sync_word = (header >> 21) & 0x7FF;
            let version = (header >> 19) & 0x3;
            let layer = (header >> 17) & 0x3;
            let protection = (header >> 16) & 0x1;
            let bitrate_index = (header >> 12) & 0xF;
            let sample_rate_index = (header >> 10) & 0x3;
            let padding = (header >> 9) & 0x1;
            let private_bit = (header >> 8) & 0x1;
            let channel_mode = (header >> 6) & 0x3;
            let mode_extension = (header >> 4) & 0x3;
            let copyright = (header >> 3) & 0x1;
            let original = (header >> 2) & 0x1;
            let emphasis = header & 0x3;
            
            println!("Sync word: 0x{:03X} (should be 0x7FF)", sync_word);
            println!("Version: {} (0=MPEG-2.5, 1=reserved, 2=MPEG-2, 3=MPEG-1)", version);
            println!("Layer: {} (1=Layer III, 2=Layer II, 3=Layer I)", layer);
            println!("Protection: {} (0=CRC, 1=no CRC)", protection);
            println!("Bitrate index: {} ", bitrate_index);
            println!("Sample rate index: {}", sample_rate_index);
            println!("Padding: {}", padding);
            println!("Private bit: {}", private_bit);
            println!("Channel mode: {} (0=stereo, 1=joint stereo, 2=dual channel, 3=mono)", channel_mode);
            println!("Mode extension: {}", mode_extension);
            println!("Copyright: {}", copyright);
            println!("Original: {}", original);
            println!("Emphasis: {}", emphasis);
            
            // Calculate frame size based on header
            let frame_size = calculate_frame_size(bitrate_index, sample_rate_index, padding, version, layer);
            println!("Calculated frame size: {} bytes", frame_size);
            
            // Move to next potential frame
            if frame_size > 0 {
                pos += frame_size;
            } else {
                pos += 1; // If we can't calculate frame size, move by 1 byte
            }
            
            // Only analyze first few frames to avoid spam
            if frame_count >= 5 {
                break;
            }
        } else {
            pos += 1;
        }
    }
    
    println!("\nTotal frames found: {}", frame_count);
}

/// Calculate MP3 frame size
fn calculate_frame_size(bitrate_index: u32, sample_rate_index: u32, padding: u32, version: u32, layer: u32) -> usize {
    // Bitrate table for MPEG-1 Layer III
    let bitrates = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
    
    // Sample rate table
    let sample_rates = match version {
        3 => [44100, 48000, 32000, 0], // MPEG-1
        2 => [22050, 24000, 16000, 0], // MPEG-2
        0 => [11025, 12000, 8000, 0],  // MPEG-2.5
        _ => return 0,
    };
    
    if bitrate_index == 0 || bitrate_index == 15 || sample_rate_index == 3 {
        return 0; // Invalid indices
    }
    
    let bitrate = bitrates[bitrate_index as usize] * 1000; // Convert to bps
    let sample_rate = sample_rates[sample_rate_index as usize];
    
    if bitrate == 0 || sample_rate == 0 {
        return 0;
    }
    
    // Frame size calculation for Layer III
    let samples_per_frame = if version == 3 { 1152 } else { 576 }; // MPEG-1 vs MPEG-2/2.5
    let frame_size = (samples_per_frame * bitrate / sample_rate / 8) + padding as usize;
    
    frame_size
}

#[test]
fn test_frame_header_analysis() {
    // Ensure output directory exists
    create_dir_all("tests/output").expect("Failed to create output directory");
    
    // Create a simple mono test
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
    
    // Generate short test audio
    let sample_rate = 44100;
    let duration = 0.5; // 0.5 seconds
    let samples_count = (sample_rate as f32 * duration) as usize;
    
    let mut pcm_data = Vec::with_capacity(samples_count);
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 16000.0;
        pcm_data.push(sample as i16);
    }
    
    // Encode the audio
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    let samples_per_frame = encoder.samples_per_frame();
    let mut mp3_data = Vec::new();
    
    println!("Encoding {} samples in frames of {} samples each", 
             pcm_data.len(), samples_per_frame);
    
    for chunk in pcm_data.chunks(samples_per_frame) {
        if chunk.len() == samples_per_frame {
            let frame_data = encoder.encode_frame(chunk)
                .expect("Failed to encode frame");
            mp3_data.extend_from_slice(frame_data);
            // println!("Encoded frame: {} bytes", frame_data.len());
        }
    }
    
    // Flush remaining data
    let final_data = encoder.flush().expect("Failed to flush");
    mp3_data.extend_from_slice(final_data);
    
    println!("Total MP3 data: {} bytes", mp3_data.len());
    
    // Write the file for analysis
    let mut file = File::create("tests/output/header_debug.mp3")
        .expect("Failed to create debug file");
    file.write_all(&mp3_data)
        .expect("Failed to write debug file");
    
    // Analyze the frame headers
    analyze_frame_headers(&mp3_data);
}