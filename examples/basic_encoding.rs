//! Basic MP3 encoding example
//!
//! This example demonstrates how to use the MP3 encoder to encode
//! PCM audio data into MP3 format.

use rust_mp3_encoder::{Mp3Encoder, Config, WaveConfig, MpegConfig, Channels, StereoMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create encoder configuration
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::JointStereo,
            bitrate: 128,
            ..Default::default()
        },
    };
    
    // Create encoder
    let mut encoder = Mp3Encoder::new(config)?;
    
    println!("Created MP3 encoder:");
    println!("  Sample rate: {} Hz", encoder.config().wave.sample_rate);
    println!("  Channels: {:?}", encoder.config().wave.channels);
    println!("  Bitrate: {} kbps", encoder.config().mpeg.bitrate);
    println!("  Samples per frame: {}", encoder.samples_per_frame());
    
    // Generate some test audio (sine wave)
    let samples_per_frame = encoder.samples_per_frame();
    let channels = encoder.config().wave.channels as usize;
    let mut pcm_data = vec![0i16; samples_per_frame * channels];
    
    // Generate a 440 Hz sine wave
    let sample_rate = encoder.config().wave.sample_rate as f32;
    let frequency = 440.0; // A4 note
    
    for i in 0..samples_per_frame {
        let t = i as f32 / sample_rate;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * 32767.0) as i16;
        
        // Fill both channels with the same data
        for ch in 0..channels {
            pcm_data[i * channels + ch] = sample_i16;
        }
    }
    
    println!("Generated {} samples of test audio", pcm_data.len());
    
    // Note: Actual encoding will be implemented in later tasks
    // For now, we just demonstrate the setup
    println!("Encoder setup complete. Encoding implementation will be added in later tasks.");
    
    Ok(())
}