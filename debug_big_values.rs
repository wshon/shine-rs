//! Debug tool for analyzing big_values issues
//!
//! This tool helps identify why big_values is too big in stereo encoding

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== Debugging big_values Issues ===");
    
    // Test the exact same configuration as the failing test
    debug_stereo_44100hz();
}

fn debug_stereo_44100hz() {
    println!("\n--- Debugging 44100Hz Stereo Signal ---");
    
    let sample_rate = 44100u32;
    let duration = 0.1; // Very short for debugging
    let samples_count = (sample_rate as f32 * duration) as usize;
    
    // Generate the same signal as the failing test
    let mut pcm_data = Vec::with_capacity(samples_count * 2);
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * 1000.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
        let sample_i16 = sample as i16;
        pcm_data.push(sample_i16); // Left channel
        pcm_data.push(sample_i16); // Right channel
    }
    
    println!("Generated {} stereo samples", samples_count);
    println!("Sample range: {} to {}", 
             pcm_data.iter().min().unwrap(), 
             pcm_data.iter().max().unwrap());
    
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate,
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
    let frame_size = samples_per_frame * 2; // Stereo
    let mut mp3_data = Vec::new();
    
    println!("Samples per frame: {}, Frame size: {}", samples_per_frame, frame_size);
    
    // Process only the first frame for detailed analysis
    if pcm_data.len() >= frame_size {
        let chunk = &pcm_data[0..frame_size];
        
        println!("\nProcessing first frame...");
        println!("Frame data range: {} to {}", 
                 chunk.iter().min().unwrap(), 
                 chunk.iter().max().unwrap());
        
        match encoder.encode_frame_interleaved(chunk) {
            Ok(frame_data) => {
                mp3_data.extend_from_slice(frame_data);
                println!("✅ Successfully encoded frame: {} bytes", frame_data.len());
                
                // Analyze the MP3 frame structure
                analyze_mp3_frame(frame_data);
                
                // Write output for FFmpeg analysis
                let output_path = "debug_stereo_frame.mp3";
                if let Ok(mut file) = File::create(output_path) {
                    let _ = file.write_all(&mp3_data);
                    println!("Written debug frame to {}", output_path);
                    
                    // Try to validate with FFmpeg
                    validate_with_ffmpeg(output_path);
                }
            },
            Err(e) => {
                println!("❌ Failed to encode frame: {:?}", e);
            }
        }
    } else {
        println!("❌ Not enough samples for a complete frame");
    }
}

fn analyze_mp3_frame(frame_data: &[u8]) {
    println!("\n--- MP3 Frame Analysis ---");
    println!("Frame size: {} bytes", frame_data.len());
    
    if frame_data.len() < 4 {
        println!("❌ Frame too small");
        return;
    }
    
    // Check sync word
    let sync = ((frame_data[0] as u16) << 3) | ((frame_data[1] as u16) >> 5);
    println!("Sync word: 0x{:03X} (should be 0x7FF)", sync);
    
    if sync != 0x7FF {
        println!("❌ Invalid sync word!");
        return;
    }
    
    // Parse frame header
    let version = (frame_data[1] >> 3) & 0x03;
    let layer = (frame_data[1] >> 1) & 0x03;
    let protection = frame_data[1] & 0x01;
    let bitrate_index = (frame_data[2] >> 4) & 0x0F;
    let samplerate_index = (frame_data[2] >> 2) & 0x03;
    let padding = (frame_data[2] >> 1) & 0x01;
    let mode = (frame_data[3] >> 6) & 0x03;
    
    println!("MPEG version: {} (3=MPEG-1, 2=MPEG-2, 0=MPEG-2.5)", version);
    println!("Layer: {} (1=Layer III)", layer);
    println!("Protection: {} (1=no CRC)", protection);
    println!("Bitrate index: {}", bitrate_index);
    println!("Sample rate index: {}", samplerate_index);
    println!("Padding: {}", padding);
    println!("Channel mode: {} (0=stereo, 1=joint stereo, 2=dual, 3=mono)", mode);
    
    // Try to parse side info (this is complex, just show first few bytes)
    if frame_data.len() > 4 {
        println!("\nSide info (first 16 bytes):");
        for i in 4..std::cmp::min(20, frame_data.len()) {
            print!("{:02X} ", frame_data[i]);
        }
        println!();
        
        // For MPEG-1 stereo, side info is 32 bytes starting at byte 4
        if frame_data.len() >= 36 && version == 3 && mode != 3 {
            analyze_side_info(&frame_data[4..36]);
        }
    }
}

fn analyze_side_info(side_info: &[u8]) {
    println!("\n--- Side Info Analysis ---");
    
    // Parse main data begin (9 bits)
    let main_data_begin = ((side_info[0] as u16) << 1) | ((side_info[1] as u16) >> 7);
    println!("Main data begin: {}", main_data_begin);
    
    // Parse private bits (3 bits for stereo)
    let private_bits = (side_info[1] >> 4) & 0x07;
    println!("Private bits: {}", private_bits);
    
    // Parse SCFSI (4 bits per channel, 2 channels = 8 bits)
    let scfsi_ch0 = side_info[1] & 0x0F;
    let scfsi_ch1 = (side_info[2] >> 4) & 0x0F;
    println!("SCFSI ch0: 0x{:X}, ch1: 0x{:X}", scfsi_ch0, scfsi_ch1);
    
    // Parse granule info (this is very complex, just show key fields)
    // For MPEG-1 stereo: 2 granules × 2 channels = 4 granule infos
    let mut offset = 2; // After private bits and SCFSI
    
    for gr in 0..2 {
        for ch in 0..2 {
            println!("\nGranule {} Channel {}:", gr, ch);
            
            if offset + 8 < side_info.len() {
                // Parse part2_3_length (12 bits)
                let part2_3_length = ((side_info[offset] as u16) << 4) | ((side_info[offset + 1] as u16) >> 4);
                println!("  part2_3_length: {}", part2_3_length);
                
                // Parse big_values (9 bits)
                let big_values = (((side_info[offset + 1] & 0x0F) as u16) << 5) | ((side_info[offset + 2] as u16) >> 3);
                println!("  big_values: {} (max allowed: 288)", big_values);
                
                if big_values > 288 {
                    println!("  ❌ big_values {} exceeds maximum 288!", big_values);
                }
                
                // Parse global_gain (8 bits)
                let global_gain = ((side_info[offset + 2] & 0x07) << 5) | (side_info[offset + 3] >> 3);
                println!("  global_gain: {}", global_gain);
                
                offset += 8; // Approximate offset increment (actual parsing is more complex)
            }
        }
    }
}

fn validate_with_ffmpeg(mp3_path: &str) {
    println!("\n--- FFmpeg Validation ---");
    
    use std::process::Command;
    
    let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };
    
    let output = Command::new("ffmpeg")
        .args(&[
            "-v", "error",
            "-i", mp3_path,
            "-f", "null",
            "-y",
            null_device
        ])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ FFmpeg validation passed");
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("❌ FFmpeg validation failed:");
                println!("{}", stderr);
            }
        },
        Err(e) => {
            println!("❌ Failed to run FFmpeg: {}", e);
        }
    }
}