//! Debug test for 48kHz big_values issue

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[test]
fn debug_48khz_big_values() {
    // Test both 44100 Hz (working) and 48000 Hz (failing)
    let test_rates = vec![44100, 48000];
    
    for &sample_rate in &test_rates {
        println!("\n=== Testing {} Hz ===", sample_rate);
        
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
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Generate correct frame size for the encoder
        let samples_per_frame = encoder.samples_per_frame();
        let channels = 2;
        let total_samples = samples_per_frame * channels;
        
        let mut pcm_data = Vec::with_capacity(total_samples);
        for i in 0..samples_per_frame {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 1000.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
            let sample_i16 = sample as i16;
            pcm_data.push(sample_i16); // Left channel
            pcm_data.push(sample_i16); // Right channel
        }
        
        let result = encoder.encode_frame_interleaved(&pcm_data);
        match result {
            Ok(encoded_frame) => {
                println!("✅ Encoding succeeded, frame size: {} bytes", encoded_frame.len());
                
                // Write test output for inspection
                let filename = format!("tests/output/debug_{}hz.mp3", sample_rate);
                std::fs::write(&filename, encoded_frame).unwrap();
                println!("Written to: {}", filename);
                
                // Try to validate with FFmpeg
                let output = std::process::Command::new("ffmpeg")
                    .args(&[
                        "-v", "error",
                        "-i", &filename,
                        "-f", "null",
                        "-y",
                        if cfg!(windows) { "NUL" } else { "/dev/null" }
                    ])
                    .output();
                
                match output {
                    Ok(result) => {
                        if result.status.success() {
                            println!("✅ FFmpeg validation passed");
                        } else {
                            let stderr = String::from_utf8_lossy(&result.stderr);
                            println!("❌ FFmpeg validation failed: {}", stderr);
                        }
                    },
                    Err(e) => {
                        println!("⚠️ Could not run FFmpeg: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("❌ Encoding failed: {:?}", e);
            }
        }
    }
}