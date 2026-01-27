//! Simple MP3 encoding example
//!
//! This example demonstrates the most basic usage of the MP3 encoder
//! using the convenience function encode_pcm_to_mp3().
//!
//! Usage: cargo run --example simple_encoding <input.wav> <output.mp3>

use shine_rs::mp3_encoder::{encode_pcm_to_mp3, Mp3EncoderConfig, StereoMode};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use hound;

/// Simple WAV file reader using hound library
fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32, u16), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    
    // Validate format requirements
    if spec.sample_format != hound::SampleFormat::Int {
        return Err("Only integer PCM format is supported".into());
    }
    
    if spec.bits_per_sample != 16 {
        return Err("Only 16-bit samples are supported".into());
    }

    let sample_rate = spec.sample_rate;
    let channels = spec.channels;

    // Read all samples
    let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    let samples = samples?;

    if samples.is_empty() {
        return Err("No audio data found in WAV file".into());
    }

    Ok((samples, sample_rate, channels))
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