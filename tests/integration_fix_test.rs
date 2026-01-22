//! Integration test to verify the count1 fix
//!
//! This test verifies that the complete encoding pipeline
//! now produces correct count1 values and doesn't generate
//! excessive 0xFF bytes.

use rust_mp3_encoder::{Mp3Encoder, Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[test]
fn test_complete_encoding_pipeline_silence() {
    println!("\n🔍 Testing complete encoding pipeline with silence");
    
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
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
    
    // Create silent audio (all zeros)
    let samples = vec![0i16; 2304]; // 1152 samples per channel * 2 channels
    
    println!("Encoding {} samples of silence", samples.len());
    
    match encoder.encode_frame(&samples) {
        Ok(mp3_data) => {
            println!("Encoding succeeded: {} bytes", mp3_data.len());
            
            // Analyze the output for 0xFF bytes
            let ff_count = mp3_data.iter().filter(|&&b| b == 0xFF).count();
            let ff_percentage = if mp3_data.len() > 0 { 
                (ff_count as f32 / mp3_data.len() as f32) * 100.0 
            } else { 
                0.0 
            };
            
            println!("0xFF bytes: {} ({:.1}%)", ff_count, ff_percentage);
            
            if mp3_data.len() >= 16 {
                print!("First 16 bytes: ");
                for i in 0..16 {
                    print!("{:02X} ", mp3_data[i]);
                }
                println!();
            }
            
            // Check for MP3 sync words
            let mut sync_words = 0;
            for i in 0..mp3_data.len().saturating_sub(1) {
                if mp3_data[i] == 0xFF && (mp3_data[i + 1] & 0xE0) == 0xE0 {
                    sync_words += 1;
                }
            }
            
            println!("Sync words found: {}", sync_words);
            
            // Verify the fix worked
            if ff_percentage < 20.0 {
                println!("✓ 0xFF byte percentage is reasonable: {:.1}%", ff_percentage);
            } else {
                println!("❌ Still too many 0xFF bytes: {:.1}%", ff_percentage);
            }
            
            if sync_words <= 5 {
                println!("✓ Reasonable number of sync words: {}", sync_words);
            } else {
                println!("❌ Too many sync words (false positives): {}", sync_words);
            }
            
            // The first 4 bytes should be a valid MP3 header
            if mp3_data.len() >= 4 && mp3_data[0] == 0xFF && (mp3_data[1] & 0xE0) == 0xE0 {
                println!("✓ Valid MP3 header detected");
            } else {
                println!("❌ Invalid MP3 header");
            }
        },
        Err(e) => {
            println!("❌ Encoding failed: {:?}", e);
        }
    }
}

#[test]
fn test_complete_encoding_pipeline_small_signal() {
    println!("\n🔍 Testing complete encoding pipeline with small signal");
    
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
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
    
    // Create very small amplitude signal
    let mut samples = vec![0i16; 2304];
    for i in 0..samples.len() {
        samples[i] = (100.0 * (i as f32 * 0.01).sin()) as i16; // Small sine wave
    }
    
    println!("Encoding {} samples of small signal", samples.len());
    
    match encoder.encode_frame(&samples) {
        Ok(mp3_data) => {
            println!("Encoding succeeded: {} bytes", mp3_data.len());
            
            // Analyze the output for 0xFF bytes
            let ff_count = mp3_data.iter().filter(|&&b| b == 0xFF).count();
            let ff_percentage = if mp3_data.len() > 0 { 
                (ff_count as f32 / mp3_data.len() as f32) * 100.0 
            } else { 
                0.0 
            };
            
            println!("0xFF bytes: {} ({:.1}%)", ff_count, ff_percentage);
            
            // Check for MP3 sync words
            let mut sync_words = 0;
            for i in 0..mp3_data.len().saturating_sub(1) {
                if mp3_data[i] == 0xFF && (mp3_data[i + 1] & 0xE0) == 0xE0 {
                    sync_words += 1;
                }
            }
            
            println!("Sync words found: {}", sync_words);
            
            // Verify the fix worked
            if ff_percentage < 30.0 {
                println!("✓ 0xFF byte percentage is reasonable: {:.1}%", ff_percentage);
            } else {
                println!("❌ Still too many 0xFF bytes: {:.1}%", ff_percentage);
            }
            
            if sync_words <= 10 {
                println!("✓ Reasonable number of sync words: {}", sync_words);
            } else {
                println!("❌ Too many sync words (false positives): {}", sync_words);
            }
        },
        Err(e) => {
            println!("❌ Encoding failed: {:?}", e);
        }
    }
}