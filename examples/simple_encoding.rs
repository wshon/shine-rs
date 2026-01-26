//! Simple MP3 encoding example
//!
//! This example demonstrates the most basic usage of the MP3 encoder
//! using the convenience function encode_pcm_to_mp3().
//!
//! Usage: cargo run --example simple_encoding <input.wav> <output.mp3>

use shine_rs::mp3_encoder::{encode_pcm_to_mp3, Mp3EncoderConfig, StereoMode};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Simple WAV file reader
fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32, u16), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    if buffer.len() < 44 {
        return Err("WAV file too small".into());
    }
    
    // Validate RIFF header
    if &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
        return Err("Not a valid WAV file".into());
    }
    
    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut pcm_data = Vec::new();
    let mut fmt_found = false;
    let mut data_found = false;
    
    // Parse chunks
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
                    if audio_format != 1 {
                        return Err("Only PCM format supported".into());
                    }
                    
                    channels = u16::from_le_bytes([buffer[chunk_data_start+2], buffer[chunk_data_start+3]]);
                    sample_rate = u32::from_le_bytes([
                        buffer[chunk_data_start+4], buffer[chunk_data_start+5], 
                        buffer[chunk_data_start+6], buffer[chunk_data_start+7]
                    ]);
                    
                    let bits_per_sample = u16::from_le_bytes([buffer[chunk_data_start+14], buffer[chunk_data_start+15]]);
                    if bits_per_sample != 16 {
                        return Err("Only 16-bit samples supported".into());
                    }
                    
                    fmt_found = true;
                }
            },
            b"data" => {
                if !fmt_found {
                    return Err("Data chunk found before fmt chunk".into());
                }
                
                // Convert bytes to i16 samples
                for i in (chunk_data_start..chunk_data_end).step_by(2) {
                    if i + 1 < buffer.len() {
                        let sample = i16::from_le_bytes([buffer[i], buffer[i+1]]);
                        pcm_data.push(sample);
                    }
                }
                data_found = true;
            },
            _ => {} // Skip unknown chunks
        }
        
        pos = chunk_data_end;
        if chunk_size % 2 == 1 {
            pos += 1; // WAV chunks are word-aligned
        }
    }
    
    if !fmt_found {
        return Err("No fmt chunk found".into());
    }
    
    if !data_found {
        return Err("No data chunk found".into());
    }
    
    if sample_rate == 0 || channels == 0 || pcm_data.is_empty() {
        return Err("Invalid WAV file data".into());
    }
    
    Ok((pcm_data, sample_rate, channels))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check command line arguments
    if args.len() != 3 {
        eprintln!("Simple MP3 Encoding Example");
        eprintln!("===========================");
        eprintln!();
        eprintln!("Usage: {} <input.wav> <output.mp3>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} input.wav output.mp3", args[0]);
        eprintln!("  {} testing/fixtures/audio/sample-3s.wav my_output.mp3", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    println!("Simple MP3 Encoding Example");
    println!("===========================");
    println!("Input:  {}", input_path);
    println!("Output: {}", output_path);
    println!();
    
    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    // Read WAV file
    println!("Reading WAV file...");
    let (pcm_data, sample_rate, channels) = read_wav_file(input_path)?;
    
    println!("WAV info: {} Hz, {} channels, {} samples", 
             sample_rate, channels, pcm_data.len());
    
    // Determine appropriate stereo mode
    let stereo_mode = if channels == 1 {
        StereoMode::Mono
    } else {
        StereoMode::JointStereo
    };
    
    // Create encoder configuration based on WAV properties
    let config = Mp3EncoderConfig::new()
        .sample_rate(sample_rate)
        .bitrate(128)  // Use standard 128 kbps
        .channels(channels as u8)
        .stereo_mode(stereo_mode);
    
    println!("Encoding to MP3 ({}Hz, 128kbps, {:?})...", 
             sample_rate, stereo_mode);
    
    // Encode PCM data to MP3 using the convenience function
    let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;
    
    // Write to file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&mp3_data)?;
    
    // Calculate statistics
    let input_size = pcm_data.len() * 2; // 16-bit samples = 2 bytes each
    let compression_ratio = input_size as f64 / mp3_data.len() as f64;
    let duration = pcm_data.len() as f64 / (sample_rate as f64 * channels as f64);
    let actual_bitrate = (mp3_data.len() as f64 * 8.0) / (duration * 1000.0);
    
    println!();
    println!("✅ Encoding completed successfully!");
    println!("   Input size:  {} bytes ({} samples)", input_size, pcm_data.len());
    println!("   Output size: {} bytes", mp3_data.len());
    println!("   Compression: {:.1}:1", compression_ratio);
    println!("   Duration:    {:.2} seconds", duration);
    println!("   Bitrate:     {:.1} kbps", actual_bitrate);
    println!("   Saved to:    {}", output_path);
    
    Ok(())
}