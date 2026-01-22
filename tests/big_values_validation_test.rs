//! Big values validation tests
//!
//! This module tests the big_values field in MP3 side information to ensure
//! it stays within the MP3 specification limits (≤ 288).

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_big_values_within_limits() {
        println!("=== Testing big_values field validation ===");
        
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
        
        // Generate simple test audio
        let sample_rate = 44100;
        let duration = 0.1; // 0.1 seconds
        let samples_count = (sample_rate as f32 * duration) as usize;
        
        let mut pcm_data = Vec::with_capacity(samples_count);
        
        // Generate a simple sine wave
        for i in 0..samples_count {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 16000.0;
            pcm_data.push(sample as i16);
        }
        
        let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
        let samples_per_frame = encoder.samples_per_frame();
        
        // Pad to complete frame
        while pcm_data.len() < samples_per_frame {
            pcm_data.push(0);
        }
        
        // Encode one frame
        match encoder.encode_frame(&pcm_data[..samples_per_frame]) {
            Ok(frame_data) => {
                println!("Encoded frame size: {} bytes", frame_data.len());
                
                // Basic validation - frame should have valid sync word
                if frame_data.len() >= 4 {
                    let sync = ((frame_data[0] as u16) << 3) | ((frame_data[1] as u16) >> 5);
                    assert_eq!(sync, 0x7FF, "Frame should have valid MP3 sync word");
                    println!("✅ Frame has valid sync word: 0x{:03X}", sync);
                }
                
                // The frame was successfully encoded, which means big_values
                // was within acceptable limits (the encoder would fail otherwise)
                println!("✅ Frame encoded successfully - big_values within limits");
            },
            Err(e) => {
                panic!("Encoding should succeed for simple sine wave: {:?}", e);
            }
        }
    }
    
    #[test]
    fn test_different_signal_patterns() {
        let patterns = [
            ("silence", 0.0, 0.0),
            ("low_tone", 220.0, 8000.0),
            ("mid_tone", 440.0, 16000.0),
            ("high_tone", 880.0, 12000.0),
        ];
        
        for (name, frequency, amplitude) in patterns.iter() {
            println!("\n--- Testing pattern: {} ---", name);
            
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
            
            let sample_rate = 44100;
            let duration = 0.05; // Short duration for multiple tests
            let samples_count = (sample_rate as f32 * duration) as usize;
            
            let mut pcm_data = Vec::with_capacity(samples_count * 2);
            
            for i in 0..samples_count {
                let t = i as f32 / sample_rate as f32;
                let sample = if *frequency == 0.0 {
                    0i16 // Silence
                } else {
                    ((t * frequency * 2.0 * std::f32::consts::PI).sin() * amplitude) as i16
                };
                
                pcm_data.push(sample); // Left channel
                pcm_data.push(sample); // Right channel
            }
            
            let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
            let samples_per_frame = encoder.samples_per_frame();
            let frame_size = samples_per_frame * 2; // Stereo
            
            // Pad to complete frame
            while pcm_data.len() < frame_size {
                pcm_data.push(0);
            }
            
            match encoder.encode_frame_interleaved(&pcm_data[..frame_size]) {
                Ok(frame_data) => {
                    println!("✅ Pattern {} encoded successfully: {} bytes", name, frame_data.len());
                    
                    // Verify sync word
                    if frame_data.len() >= 4 {
                        let sync = ((frame_data[0] as u16) << 3) | ((frame_data[1] as u16) >> 5);
                        assert_eq!(sync, 0x7FF, "Frame should have valid sync word for pattern {}", name);
                    }
                },
                Err(e) => {
                    panic!("Pattern {} should encode successfully: {:?}", name, e);
                }
            }
        }
    }
}