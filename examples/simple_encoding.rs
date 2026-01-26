//! Simple MP3 encoding example
//!
//! This example demonstrates the most basic usage of the MP3 encoder
//! using the convenience function encode_pcm_to_mp3().

use shine_rs::mp3_encoder::{encode_pcm_to_mp3, Mp3EncoderConfig, StereoMode};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple MP3 Encoding Example");
    println!("===========================");
    
    // Generate a simple sine wave (440 Hz for 2 seconds at 44.1 kHz, stereo)
    let sample_rate = 44100;
    let duration = 2.0; // seconds
    let frequency = 440.0; // Hz (A4 note)
    let samples_per_channel = (sample_rate as f64 * duration) as usize;
    
    println!("Generating {} seconds of {}Hz sine wave...", duration, frequency);
    
    let mut pcm_data = Vec::with_capacity(samples_per_channel * 2); // stereo
    
    for i in 0..samples_per_channel {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * 16383.0) as i16; // Scale to 16-bit range
        
        // Add stereo samples (left and right channels)
        pcm_data.push(sample_i16); // Left channel
        pcm_data.push(sample_i16); // Right channel
    }
    
    println!("Generated {} PCM samples", pcm_data.len());
    
    // Create encoder configuration
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .bitrate(128)
        .channels(2)
        .stereo_mode(StereoMode::Stereo);
    
    println!("Encoding to MP3 (44.1kHz, 128kbps, stereo)...");
    
    // Encode PCM data to MP3 using the convenience function
    let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;
    
    // Write to file
    let output_filename = "simple_output.mp3";
    let mut output_file = File::create(output_filename)?;
    output_file.write_all(&mp3_data)?;
    
    // Calculate statistics
    let input_size = pcm_data.len() * 2; // 16-bit samples = 2 bytes each
    let compression_ratio = input_size as f64 / mp3_data.len() as f64;
    let actual_bitrate = (mp3_data.len() as f64 * 8.0) / (duration * 1000.0);
    
    println!("✅ Encoding completed successfully!");
    println!("   Input size:  {} bytes ({} samples)", input_size, pcm_data.len());
    println!("   Output size: {} bytes", mp3_data.len());
    println!("   Compression: {:.1}:1", compression_ratio);
    println!("   Duration:    {:.1} seconds", duration);
    println!("   Bitrate:     {:.1} kbps", actual_bitrate);
    println!("   Saved to:    {}", output_filename);
    
    Ok(())
}