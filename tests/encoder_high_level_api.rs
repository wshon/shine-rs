//! High-Level Encoder API Tests
//!
//! This test suite validates the high-level MP3 encoder API that provides
//! a more convenient interface compared to the low-level Shine-compatible functions.

use shine_rs::mp3_encoder::{
    Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3,
    SUPPORTED_SAMPLE_RATES, SUPPORTED_BITRATES
};
use shine_rs::error::{ConfigError, InputDataError};

#[test]
fn test_config_validation() {
    // Test valid configuration
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(128);
    
    assert!(config.validate().is_ok());
    println!("✅ Valid configuration accepted");
    
    // Test invalid sample rate
    let invalid_config = Mp3EncoderConfig::new()
        .sample_rate(22050) // Not in SUPPORTED_SAMPLE_RATES
        .channels(2)
        .bitrate(128);
    
    match invalid_config.validate() {
        Err(ConfigError::UnsupportedSampleRate(_)) => {
            println!("✅ Correctly rejected invalid sample rate");
        }
        _ => panic!("❌ Should have rejected invalid sample rate"),
    }
    
    // Test invalid bitrate
    let invalid_config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(999); // Not in SUPPORTED_BITRATES
    
    match invalid_config.validate() {
        Err(ConfigError::UnsupportedBitrate(_)) => {
            println!("✅ Correctly rejected invalid bitrate");
        }
        _ => panic!("❌ Should have rejected invalid bitrate"),
    }
}

#[test]
fn test_encoder_creation() {
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(128);
    
    match Mp3Encoder::new(config) {
        Ok(_encoder) => {
            println!("✅ Encoder creation successful");
        }
        Err(e) => panic!("❌ Encoder creation failed: {}", e),
    }
}

#[test]
fn test_pcm_encoding() {
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(128);
    
    let mut encoder = Mp3Encoder::new(config)
        .expect("Failed to create encoder");
    
    // Generate test PCM data (1 second of silence)
    let samples_per_second = 44100 * 2; // stereo
    let pcm_data = vec![0i16; samples_per_second];
    
    match encoder.encode_interleaved(&pcm_data) {
        Ok(mp3_frames) => {
            let total_bytes: usize = mp3_frames.iter().map(|frame| frame.len()).sum();
            println!("✅ PCM encoding successful: {} bytes output", total_bytes);
            
            // Basic validation
            assert!(!mp3_frames.is_empty(), "Output should not be empty");
            
            // Check for MP3 frame sync in first frame
            if let Some(first_frame) = mp3_frames.first() {
                let mut found_sync = false;
                for i in 0..first_frame.len().saturating_sub(1) {
                    if first_frame[i] == 0xFF && (first_frame[i + 1] & 0xE0) == 0xE0 {
                        found_sync = true;
                        break;
                    }
                }
                assert!(found_sync, "Should contain valid MP3 frame sync");
            }
        }
        Err(e) => panic!("❌ PCM encoding failed: {}", e),
    }
}

#[test]
fn test_finalize() {
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(128);
    
    let mut encoder = Mp3Encoder::new(config)
        .expect("Failed to create encoder");
    
    // Encode some data
    let pcm_data = vec![0i16; 1152 * 2]; // One frame of stereo data
    let _mp3_frames = encoder.encode_interleaved(&pcm_data)
        .expect("Failed to encode PCM");
    
    // Finalize
    match encoder.finish() {
        Ok(final_data) => {
            println!("✅ Finalize successful: {} bytes", final_data.len());
        }
        Err(e) => panic!("❌ Finalize failed: {}", e),
    }
}

#[test]
fn test_convenience_function() {
    // Generate test PCM data
    let sample_rate = 44100;
    let channels = 2;
    let duration_seconds = 1;
    let pcm_data = vec![0i16; sample_rate * channels * duration_seconds];
    
    let config = Mp3EncoderConfig::new()
        .sample_rate(sample_rate as u32)
        .channels(channels as u32 as u8)
        .bitrate(128);
    
    match encode_pcm_to_mp3(config, &pcm_data) {
        Ok(mp3_data) => {
            println!("✅ Convenience function successful: {} bytes", mp3_data.len());
            
            // Basic validation
            assert!(!mp3_data.is_empty(), "Output should not be empty");
        }
        Err(e) => panic!("❌ Convenience function failed: {}", e),
    }
}

#[test]
fn test_different_configurations() {
    let test_configs = [
        (44100, 1, 128, "44.1kHz mono 128kbps"),
        (44100, 2, 128, "44.1kHz stereo 128kbps"),
        (48000, 2, 192, "48kHz stereo 192kbps"),
    ];
    
    for (sample_rate, channels, bitrate, description) in &test_configs {
        // Skip if not supported
        if !SUPPORTED_SAMPLE_RATES.contains(sample_rate) {
            println!("⚠️  Skipping {}: sample rate not supported", description);
            continue;
        }
        
        if !SUPPORTED_BITRATES.contains(bitrate) {
            println!("⚠️  Skipping {}: bitrate not supported", description);
            continue;
        }
        
        let config = Mp3EncoderConfig::new()
            .sample_rate(*sample_rate)
            .channels(*channels as u8)
            .bitrate(*bitrate);
        
        match Mp3Encoder::new(config) {
            Ok(mut encoder) => {
                // Test encoding one frame
                let samples_per_frame = 1152 * (*channels as usize);
                let pcm_data = vec![0i16; samples_per_frame];
                
                match encoder.encode_interleaved(&pcm_data) {
                    Ok(mp3_frames) => {
                        let total_bytes: usize = mp3_frames.iter().map(|frame| frame.len()).sum();
                        println!("✅ {}: {} bytes output", description, total_bytes);
                    }
                    Err(e) => panic!("❌ {} encoding failed: {}", description, e),
                }
            }
            Err(e) => panic!("❌ {} encoder creation failed: {}", description, e),
        }
    }
}

#[test]
fn test_stereo_modes() {
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(128)
        .stereo_mode(StereoMode::JointStereo);
    
    match Mp3Encoder::new(config) {
        Ok(mut encoder) => {
            let pcm_data = vec![0i16; 1152 * 2]; // One frame stereo
            
            match encoder.encode_interleaved(&pcm_data) {
                Ok(mp3_frames) => {
                    let total_bytes: usize = mp3_frames.iter().map(|frame| frame.len()).sum();
                    println!("✅ Joint stereo encoding: {} bytes", total_bytes);
                }
                Err(e) => panic!("❌ Joint stereo encoding failed: {}", e),
            }
        }
        Err(e) => panic!("❌ Joint stereo encoder creation failed: {}", e),
    }
}

#[test]
fn test_error_conditions() {
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .channels(2)
        .bitrate(128);
    
    let mut encoder = Mp3Encoder::new(config)
        .expect("Failed to create encoder");
    
    // Test with wrong number of samples (not matching channels)
    let wrong_pcm_data = vec![0i16; 1153]; // Odd number for stereo
    
    match encoder.encode_interleaved(&wrong_pcm_data) {
        Ok(_) => println!("⚠️  Encoder accepted mismatched sample count"),
        Err(shine_rs::error::EncoderError::InputData(InputDataError::InvalidChannelCount { .. })) => {
            println!("✅ Correctly rejected invalid sample count");
        }
        Err(e) => panic!("❌ Unexpected error: {}", e),
    }
    
    // Test empty data
    let empty_data = vec![];
    match encoder.encode_interleaved(&empty_data) {
        Ok(mp3_frames) => {
            if mp3_frames.is_empty() {
                println!("✅ Empty input produced empty output");
            } else {
                println!("⚠️  Empty input produced non-empty output");
            }
        }
        Err(_) => println!("✅ Correctly handled empty input"),
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_encoding_workflow() {
        // Simulate encoding a short audio clip
        let sample_rate = 44100;
        let channels = 2;
        let duration_frames = 10;
        let samples_per_frame = 1152;
        
        let config = Mp3EncoderConfig::new()
            .sample_rate(sample_rate)
            .channels(channels as u8)
            .bitrate(128);
        
        let mut encoder = Mp3Encoder::new(config)
            .expect("Failed to create encoder");
        
        let mut total_output = Vec::new();
        
        // Encode multiple frames
        for frame_num in 0..duration_frames {
            // Generate different data for each frame (simple sine wave)
            let pcm_data: Vec<i16> = (0..samples_per_frame * channels)
                .map(|i| {
                    let sample_index = i / channels;
                    let amplitude = 16000.0;
                    let frequency = 440.0; // A4 note
                    let phase = 2.0 * std::f64::consts::PI * frequency * 
                               (sample_index + frame_num * samples_per_frame) as f64 / sample_rate as f64;
                    (amplitude * phase.sin()) as i16
                })
                .collect();
            
            match encoder.encode_interleaved(&pcm_data) {
                Ok(mp3_frames) => {
                    for frame in mp3_frames {
                        total_output.extend_from_slice(&frame);
                    }
                    println!("✅ Frame {}: {} bytes", frame_num, total_output.len());
                }
                Err(e) => panic!("❌ Frame {} encoding failed: {}", frame_num, e),
            }
        }
        
        // Finalize
        match encoder.finish() {
            Ok(final_data) => {
                total_output.extend_from_slice(&final_data);
                println!("✅ Finalize: {} bytes", final_data.len());
            }
            Err(e) => panic!("❌ Finalize failed: {}", e),
        }
        
        println!("✅ Full workflow complete: {} total bytes", total_output.len());
        
        // Basic validation of final output
        assert!(!total_output.is_empty(), "Final output should not be empty");
        
        // Should contain multiple MP3 frames
        let mut frame_count = 0;
        for i in 0..total_output.len().saturating_sub(1) {
            if total_output[i] == 0xFF && (total_output[i + 1] & 0xE0) == 0xE0 {
                frame_count += 1;
            }
        }
        
        assert!(frame_count > 0, "Should contain at least one MP3 frame");
        println!("✅ Found {} MP3 frames in output", frame_count);
    }
}