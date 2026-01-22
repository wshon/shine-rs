//! Debug tool to inspect side_info during encoding

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

fn main() {
    println!("Side Info Debug Tool");
    println!("===================");
    
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
    
    let mut encoder = Mp3Encoder::new(config).unwrap();
    let samples_per_frame = encoder.samples_per_frame();
    
    println!("Samples per frame: {}", samples_per_frame);
    
    // Test with small constant values that typically cause big_values issues
    println!("\n--- Testing with constant value 1 ---");
    let test_data = vec![1i16; samples_per_frame];
    
    match encoder.encode_frame(&test_data) {
        Ok(frame) => {
            println!("SUCCESS: Frame encoded, size: {} bytes", frame.len());
            
            // The issue is that we can't directly access the side_info from the encoder
            // But we can see if the warning message appears in bitstream.rs
            println!("Check console output above for any big_values clamping warnings");
        },
        Err(e) => {
            println!("ERROR: Encoding failed: {:?}", e);
        }
    }
    
    println!("\n--- Testing with constant value 1000 ---");
    let test_data = vec![1000i16; samples_per_frame];
    
    match encoder.encode_frame(&test_data) {
        Ok(frame) => {
            println!("SUCCESS: Frame encoded, size: {} bytes", frame.len());
            println!("Check console output above for any big_values clamping warnings");
        },
        Err(e) => {
            println!("ERROR: Encoding failed: {:?}", e);
        }
    }
    
    println!("\n--- Testing with sine wave ---");
    let sine_data: Vec<i16> = (0..samples_per_frame)
        .map(|i| {
            let t = i as f64 / 44100.0;
            (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16
        })
        .collect();
    
    match encoder.encode_frame(&sine_data) {
        Ok(frame) => {
            println!("SUCCESS: Frame encoded, size: {} bytes", frame.len());
            println!("Check console output above for any big_values clamping warnings");
        },
        Err(e) => {
            println!("ERROR: Encoding failed: {:?}", e);
        }
    }
}