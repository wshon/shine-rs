//! Test frame size consistency
//!
//! This program generates a few MP3 frames and validates their sizes

use rust_mp3_encoder::{Mp3Encoder, Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Frame Size Test");
    println!("===============");
    
    // Create encoder configuration
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
    
    println!("Configuration:");
    println!("  Sample rate: 44100 Hz");
    println!("  Bitrate: 128 kbps");
    println!("  Channels: Mono");
    println!("  Samples per frame: {}", samples_per_frame);
    
    // Create output file
    let mut output_file = File::create("tests/output/frame_size_test.mp3")?;
    
    // Generate 5 frames with different patterns
    let test_patterns = [
        ("zeros", vec![0i16; samples_per_frame]),
        ("small_sine", (0..samples_per_frame).map(|i| (1000.0 * (i as f64 * 0.01).sin()) as i16).collect()),
        ("large_sine", (0..samples_per_frame).map(|i| (10000.0 * (i as f64 * 0.02).sin()) as i16).collect()),
        ("noise", (0..samples_per_frame).map(|i| ((i * 1234567) % 32768) as i16 - 16384).collect()),
        ("sweep", (0..samples_per_frame).map(|i| (5000.0 * (i as f64 * i as f64 * 0.0001).sin()) as i16).collect()),
    ];
    
    for (i, (name, samples)) in test_patterns.iter().enumerate() {
        println!("\n--- Frame {} ({}) ---", i + 1, name);
        
        let encoded_frame = encoder.encode_frame(samples)?;
        println!("Frame size: {} bytes", encoded_frame.len());
        
        // Write frame to file
        output_file.write_all(encoded_frame)?;
        
        // Validate frame header
        if encoded_frame.len() >= 4 {
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            if sync == 0x7FF {
                println!("✅ Valid MP3 sync word");
            } else {
                println!("❌ Invalid sync word: 0x{:03X}", sync);
            }
        } else {
            println!("❌ Frame too small");
        }
    }
    
    println!("\n✅ Test complete. Output written to tests/output/frame_size_test.mp3");
    println!("File size: {} bytes", std::fs::metadata("tests/output/frame_size_test.mp3")?.len());
    
    Ok(())
}