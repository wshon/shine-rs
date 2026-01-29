//! Simple MP3 encoding example
//!
//! This example demonstrates how to use the high-level Mp3Encoder API
//! to convert PCM audio data to MP3 format.

use shine_rs::mp3_encoder::{Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3};
use std::fs::File;
use std::io::Write;
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
    println!("Simple MP3 Encoding Example");
    println!("===========================");
    
    // Example 1: Test with real WAV file
    println!("\n1. Encoding real WAV file (sample-3s.wav)...");
    
    let wav_path = "testing/fixtures/audio/sample-3s.wav";
    
    // Check if WAV file exists
    if std::path::Path::new(wav_path).exists() {
        match read_wav_file(wav_path) {
            Ok((pcm_data, sample_rate, channels)) => {
                println!("   WAV info: {} Hz, {} channels, {} samples", 
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
                    .bitrate(128)
                    .channels(channels as u8)
                    .stereo_mode(stereo_mode);
                
                // Encode using convenience function
                let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;
                
                // Write to file
                let mut output_file = File::create("output_sample.mp3")?;
                output_file.write_all(&mp3_data)?;
                
                println!("   Generated {} bytes of MP3 data", mp3_data.len());
                println!("   Saved to: output_sample.mp3");
                
                // Calculate compression ratio and duration
                let input_size = pcm_data.len() * 2; // 16-bit samples
                let compression_ratio = input_size as f64 / mp3_data.len() as f64;
                let duration = pcm_data.len() as f64 / (sample_rate as f64 * channels as f64);
                
                println!("   Input size:  {} bytes", input_size);
                println!("   Output size: {} bytes", mp3_data.len());
                println!("   Compression: {:.1}:1", compression_ratio);
                println!("   Duration:    {:.2} seconds", duration);
                println!("   Actual bitrate: {:.1} kbps", 
                         (mp3_data.len() as f64 * 8.0) / (duration * 1000.0));
            },
            Err(e) => {
                println!("   Error reading WAV file: {}", e);
                println!("   Continuing with synthetic audio examples...");
            }
        }
    } else {
        println!("   WAV file not found: {}", wav_path);
        println!("   Continuing with synthetic audio examples...");
    }
    
    // Example 2: Generate synthetic audio data
    println!("\n2. Encoding synthetic audio data...");
    
    // Generate a simple sine wave (440 Hz for 1 second at 44.1 kHz, stereo)
    let sample_rate = 44100;
    let duration = 1.0; // seconds
    let frequency = 440.0; // Hz (A4 note)
    let samples_per_channel = (sample_rate as f64 * duration) as usize;
    
    let mut pcm_data = Vec::with_capacity(samples_per_channel * 2); // stereo
    
    for i in 0..samples_per_channel {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * 16383.0) as i16; // Scale to 16-bit range
        
        // Add stereo samples (left and right channels)
        pcm_data.push(sample_i16); // Left channel
        pcm_data.push(sample_i16); // Right channel
    }
    
    // Create encoder configuration
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .bitrate(128)
        .channels(2)
        .stereo_mode(StereoMode::Stereo);
    
    // Method 1: Using the convenience function
    println!("   Using convenience function...");
    let mp3_data = encode_pcm_to_mp3(config.clone(), &pcm_data)?;
    
    // Write to file
    let mut output_file = File::create("output_simple.mp3")?;
    output_file.write_all(&mp3_data)?;
    
    println!("   Generated {} bytes of MP3 data", mp3_data.len());
    println!("   Saved to: output_simple.mp3");
    
    // Method 2: Using the encoder directly (streaming approach)
    println!("\n3. Using streaming encoder...");
    
    let mut encoder = Mp3Encoder::new(config)?;
    let mut streaming_mp3_data = Vec::new();
    
    // Process data in chunks (simulating streaming)
    let chunk_size = encoder.samples_per_frame() * 4; // Process 4 frames at a time
    
    for chunk in pcm_data.chunks(chunk_size) {
        let frames = encoder.encode_interleaved(chunk)?;
        for frame in frames {
            streaming_mp3_data.extend(frame);
        }
    }
    
    // Finish encoding
    let final_data = encoder.finish()?;
    streaming_mp3_data.extend(final_data);
    
    // Write streaming result
    let mut streaming_file = File::create("output_streaming.mp3")?;
    streaming_file.write_all(&streaming_mp3_data)?;
    
    println!("   Generated {} bytes of MP3 data (streaming)", streaming_mp3_data.len());
    println!("   Saved to: output_streaming.mp3");
    
    // Example 4: Different configurations
    println!("\n4. Testing different configurations...");
    
    // Mono, lower bitrate
    let mono_config = Mp3EncoderConfig::new()
        .sample_rate(22050)
        .bitrate(64)
        .channels(1)
        .stereo_mode(StereoMode::Mono);
    
    // Convert stereo to mono by taking only left channel
    let mono_pcm: Vec<i16> = pcm_data.iter().step_by(2).cloned().collect();
    
    let mono_mp3 = encode_pcm_to_mp3(mono_config, &mono_pcm)?;
    
    let mut mono_file = File::create("output_mono.mp3")?;
    mono_file.write_all(&mono_mp3)?;
    
    println!("   Mono (22kHz, 64kbps): {} bytes -> output_mono.mp3", mono_mp3.len());
    
    // High quality stereo
    let hq_config = Mp3EncoderConfig::new()
        .sample_rate(48000)
        .bitrate(320)
        .channels(2)
        .stereo_mode(StereoMode::JointStereo);
    
    // Resample to 48kHz (simple upsampling for demo)
    let mut hq_pcm = Vec::new();
    let ratio = 48000.0 / 44100.0;
    for i in 0..(pcm_data.len() as f64 * ratio) as usize {
        let src_idx = (i as f64 / ratio) as usize;
        if src_idx < pcm_data.len() {
            hq_pcm.push(pcm_data[src_idx]);
        }
    }
    
    let hq_mp3 = encode_pcm_to_mp3(hq_config, &hq_pcm)?;
    
    let mut hq_file = File::create("output_hq.mp3")?;
    hq_file.write_all(&hq_mp3)?;
    
    println!("   HQ (48kHz, 320kbps): {} bytes -> output_hq.mp3", hq_mp3.len());
    
    // Example 5: Error handling demonstration
    println!("\n5. Error handling examples...");
    
    // Invalid configuration
    let invalid_config = Mp3EncoderConfig::new()
        .sample_rate(96000) // Unsupported sample rate
        .bitrate(128);
    
    match Mp3Encoder::new(invalid_config) {
        Ok(_) => println!("   Unexpected: invalid config was accepted"),
        Err(e) => println!("   Expected error: {}", e),
    }
    
    // Invalid bitrate for sample rate
    let invalid_combo = Mp3EncoderConfig::new()
        .sample_rate(8000)  // MPEG-2.5
        .bitrate(320);      // Too high for MPEG-2.5
    
    match Mp3Encoder::new(invalid_combo) {
        Ok(_) => println!("   Unexpected: invalid combo was accepted"),
        Err(e) => println!("   Expected error: {}", e),
    }
    
    println!("\n✅ All examples completed successfully!");
    println!("\nGenerated files:");
    if std::path::Path::new("output_sample.mp3").exists() {
        println!("  - output_sample.mp3 (from sample-3s.wav)");
    }
    println!("  - output_simple.mp3 (convenience function)");
    println!("  - output_streaming.mp3 (streaming encoder)");
    println!("  - output_mono.mp3 (mono, 22kHz, 64kbps)");
    println!("  - output_hq.mp3 (stereo, 48kHz, 320kbps)");
    
    Ok(())
}