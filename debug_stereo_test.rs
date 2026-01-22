//! Debug test for stereo encoding issues
//!
//! This test helps identify why stereo encoding fails with certain signals

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== Debugging Stereo Encoding Issues ===");
    
    // Test 1: Simple stereo signal (like the failing test)
    test_simple_stereo_signal();
    
    // Test 2: Lower amplitude stereo signal
    test_low_amplitude_stereo();
    
    // Test 3: Quiet stereo signal
    test_quiet_stereo();
}

fn test_simple_stereo_signal() {
    println!("\n--- Test 1: Simple Stereo Signal (1000Hz) ---");
    
    let sample_rate = 44100u32;
    let duration = 0.1; // Short duration for debugging
    let samples_count = (sample_rate as f32 * duration) as usize;
    
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
    
    let mut frame_count = 0;
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            match encoder.encode_frame_interleaved(chunk) {
                Ok(frame_data) => {
                    mp3_data.extend_from_slice(frame_data);
                    frame_count += 1;
                    println!("Encoded frame {}: {} bytes", frame_count, frame_data.len());
                },
                Err(e) => {
                    println!("❌ Failed to encode frame {}: {:?}", frame_count + 1, e);
                    return;
                }
            }
        }
    }
    
    match encoder.flush() {
        Ok(final_data) => {
            mp3_data.extend_from_slice(final_data);
            if !final_data.is_empty() {
                println!("Flushed final data: {} bytes", final_data.len());
            }
        },
        Err(e) => {
            println!("❌ Failed to flush: {:?}", e);
            return;
        }
    }
    
    println!("Total MP3 data: {} bytes", mp3_data.len());
    
    // Write output for analysis
    let output_path = "debug_stereo_simple.mp3";
    if let Ok(mut file) = File::create(output_path) {
        let _ = file.write_all(&mp3_data);
        println!("✅ Written to {}", output_path);
    }
}

fn test_low_amplitude_stereo() {
    println!("\n--- Test 2: Low Amplitude Stereo Signal ---");
    
    let sample_rate = 44100u32;
    let duration = 0.1;
    let samples_count = (sample_rate as f32 * duration) as usize;
    
    let mut pcm_data = Vec::with_capacity(samples_count * 2);
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 1000.0; // Much lower amplitude
        let sample_i16 = sample as i16;
        pcm_data.push(sample_i16); // Left channel
        pcm_data.push(sample_i16); // Right channel
    }
    
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
    let frame_size = samples_per_frame * 2;
    let mut mp3_data = Vec::new();
    
    let mut frame_count = 0;
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            match encoder.encode_frame_interleaved(chunk) {
                Ok(frame_data) => {
                    mp3_data.extend_from_slice(frame_data);
                    frame_count += 1;
                    println!("Encoded frame {}: {} bytes", frame_count, frame_data.len());
                },
                Err(e) => {
                    println!("❌ Failed to encode frame {}: {:?}", frame_count + 1, e);
                    return;
                }
            }
        }
    }
    
    let _ = encoder.flush();
    
    let output_path = "debug_stereo_low_amp.mp3";
    if let Ok(mut file) = File::create(output_path) {
        let _ = file.write_all(&mp3_data);
        println!("✅ Written to {}", output_path);
    }
}

fn test_quiet_stereo() {
    println!("\n--- Test 3: Quiet Stereo Signal ---");
    
    let sample_rate = 44100u32;
    let duration = 0.1;
    let samples_count = (sample_rate as f32 * duration) as usize;
    
    let mut pcm_data = Vec::with_capacity(samples_count * 2);
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 100.0; // Very quiet
        let sample_i16 = sample as i16;
        pcm_data.push(sample_i16); // Left channel
        pcm_data.push(sample_i16); // Right channel
    }
    
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
    let frame_size = samples_per_frame * 2;
    let mut mp3_data = Vec::new();
    
    let mut frame_count = 0;
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            match encoder.encode_frame_interleaved(chunk) {
                Ok(frame_data) => {
                    mp3_data.extend_from_slice(frame_data);
                    frame_count += 1;
                    println!("Encoded frame {}: {} bytes", frame_count, frame_data.len());
                },
                Err(e) => {
                    println!("❌ Failed to encode frame {}: {:?}", frame_count + 1, e);
                    return;
                }
            }
        }
    }
    
    let _ = encoder.flush();
    
    let output_path = "debug_stereo_quiet.mp3";
    if let Ok(mut file) = File::create(output_path) {
        let _ = file.write_all(&mp3_data);
        println!("✅ Written to {}", output_path);
    }
}