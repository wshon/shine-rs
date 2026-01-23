//! Encoder Output Analysis Tests
//!
//! Integration tests for analyzing MP3 encoder output distribution and data flow.
//! These tests help diagnose encoding pipeline issues and verify correct data processing.

use rust_mp3_encoder::{Config, Mp3Encoder};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

/// Standard test configuration for analysis
fn create_test_config() -> Config {
    Config {
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
    }
}

/// Test encoder output data distribution
/// 
/// This test analyzes the byte distribution in encoded MP3 frames to detect
/// issues like all-zero main data or incorrect frame structure.
#[test]
fn test_encoder_output_distribution() {
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
    
    // Create strong test signal to ensure non-zero output
    let pcm_data: Vec<i16> = (0..1152*2)
        .map(|i| {
            if i % 2 == 0 {
                10000  // Left channel: strong signal
            } else {
                -10000 // Right channel: strong signal
            }
        })
        .collect();
    
    let result = encoder.encode_frame_interleaved(&pcm_data);
    assert!(result.is_ok(), "Encoding should succeed");
    
    let encoded_frame = result.unwrap();
    assert!(!encoded_frame.is_empty(), "Encoded frame should not be empty");
    
    // Analyze byte distribution
    let mut byte_counts = [0usize; 256];
    for &byte in encoded_frame.iter() {
        byte_counts[byte as usize] += 1;
    }
    
    // Verify frame has proper structure
    assert!(encoded_frame.len() >= 4, "Frame should have at least 4 bytes for header");
    
    // Check sync word (should be 0xFFE or 0xFFF)
    let sync_word = ((encoded_frame[0] as u16) << 4) | ((encoded_frame[1] as u16) >> 4);
    assert!(sync_word == 0xFFE || sync_word == 0xFFF, "Invalid sync word: 0x{:03X}", sync_word);
    
    // Calculate main data region
    let sideinfo_len = if config.mpeg_version() == rust_mp3_encoder::config::MpegVersion::Mpeg1 {
        if config.wave.channels == Channels::Stereo { 32 } else { 17 }
    } else {
        if config.wave.channels == Channels::Stereo { 17 } else { 9 }
    };
    
    let main_data_start = 4 + sideinfo_len;
    
    if encoded_frame.len() > main_data_start {
        let main_data = &encoded_frame[main_data_start..];
        let non_zero_main_data = main_data.iter().filter(|&&b| b != 0).count();
        
        println!("Main data analysis:");
        println!("  Total main data bytes: {}", main_data.len());
        println!("  Non-zero main data bytes: {}", non_zero_main_data);
        println!("  Non-zero percentage: {:.1}%", 
                non_zero_main_data as f64 / main_data.len() as f64 * 100.0);
        
        // Main data should contain some non-zero bytes for strong input signal
        assert!(non_zero_main_data > 0, 
               "Main data should contain non-zero bytes for strong input signal. Found {} non-zero bytes out of {} - this indicates a critical encoding pipeline failure", 
               non_zero_main_data, main_data.len());
        
        // At least some percentage of main data should be non-zero for strong signal
        let non_zero_percentage = non_zero_main_data as f64 / main_data.len() as f64 * 100.0;
        assert!(non_zero_percentage > 1.0, 
               "Expected >1% non-zero main data for strong signal, got {:.1}% - encoding pipeline is not processing audio data correctly", 
               non_zero_percentage);
    }
}

/// Test encoding pipeline with different signal amplitudes
/// 
/// Verifies that the encoder produces appropriate output for signals
/// of varying amplitudes.
#[test]
fn test_encoding_pipeline_amplitudes() {
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    let amplitudes = [100i16, 1000i16, 10000i16, i16::MAX/2];
    
    for &amp in amplitudes.iter() {
        let pcm_data: Vec<i16> = vec![amp; 1152*2];
        let result = encoder.encode_frame_interleaved(&pcm_data);
        assert!(result.is_ok(), "Encoding failed for amplitude {}", amp);
        
        let encoded = result.unwrap();
        assert!(!encoded.is_empty(), "Encoded frame should not be empty for amplitude {}", amp);
        
        // For non-zero input, we should get meaningful non-zero output
        if amp > 0 {
            let non_zero_bytes = encoded.iter().filter(|&&b| b != 0 && b != 0xFF).count();
            assert!(non_zero_bytes > 10, // More than just header + side info
                   "Expected substantial non-zero output for amplitude {}, got {} non-zero bytes - encoding pipeline failure", 
                   amp, non_zero_bytes);
            
            // Check main data specifically for larger amplitudes
            if amp >= 1000 {
                let sideinfo_len = 32; // Stereo MPEG-1
                let main_data_start = 4 + sideinfo_len;
                if encoded.len() > main_data_start {
                    let main_data = &encoded[main_data_start..];
                    let non_zero_main = main_data.iter().filter(|&&b| b != 0).count();
                    assert!(non_zero_main > 0,
                           "Main data should be non-zero for amplitude {} - critical encoding failure", amp);
                }
            }
        }
    }
}

/// Test encoding pipeline with different frequency content
/// 
/// Verifies that the encoder handles different frequency components correctly.
#[test]
fn test_encoding_pipeline_frequencies() {
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    let frequencies = [440.0, 880.0, 1760.0]; // A4, A5, A6
    
    for &freq in frequencies.iter() {
        // Generate sine wave
        let pcm_data: Vec<i16> = (0..1152*2)
            .map(|i| {
                let sample_rate = 44100.0;
                let amplitude = 5000.0;
                let phase = 2.0 * std::f64::consts::PI * freq * (i / 2) as f64 / sample_rate;
                (amplitude * phase.sin()) as i16
            })
            .collect();
        
        let result = encoder.encode_frame_interleaved(&pcm_data);
        assert!(result.is_ok(), "Encoding failed for frequency {}Hz", freq);
        
        let encoded = result.unwrap();
        assert!(!encoded.is_empty(), "Encoded frame should not be empty for frequency {}Hz", freq);
        
        // Sine wave should produce non-trivial encoding with actual audio data
        let non_zero_bytes = encoded.iter().filter(|&&b| b != 0 && b != 0xFF).count();
        assert!(non_zero_bytes > 20, // Reasonable threshold for meaningful encoding
               "Expected substantial output for {}Hz sine wave, got {} non-zero bytes - encoding pipeline failure", 
               freq, non_zero_bytes);
        
        // Check main data region specifically
        let sideinfo_len = 32; // Stereo MPEG-1
        let main_data_start = 4 + sideinfo_len;
        if encoded.len() > main_data_start {
            let main_data = &encoded[main_data_start..];
            let non_zero_main = main_data.iter().filter(|&&b| b != 0).count();
            assert!(non_zero_main > 0,
                   "Main data should contain encoded audio for {}Hz sine wave - critical encoding failure", freq);
        }
    }
}

/// Test module data flow integration
/// 
/// Verifies that data flows correctly between encoding modules.
#[test]
fn test_module_data_flow() {
    // Test subband filter
    let mut filter = rust_mp3_encoder::subband::SubbandFilter::new();
    let pcm_samples = [5000i16; 32];
    let mut subband_output = [0i32; 32];
    
    let result = filter.filter(&pcm_samples, &mut subband_output, 0);
    assert!(result.is_ok(), "Subband filtering should succeed");
    
    let non_zero_subbands = subband_output.iter().filter(|&&x| x != 0).count();
    assert!(non_zero_subbands > 0, "Subband filter should produce non-zero output for non-zero input");
    
    let max_subband = subband_output.iter().map(|&x| x.abs()).max().unwrap_or(0);
    assert!(max_subband > 0, "Subband filter should produce non-zero maximum value");
    
    // Test bit reservoir
    let mut reservoir = rust_mp3_encoder::reservoir::BitReservoir::new(128, 44100, 2);
    let max_bits = reservoir.max_reservoir_bits(100.0, 2);
    assert!(max_bits > 0, "Reservoir should allow some bits");
    
    reservoir.adjust_reservoir(1000, 2);
    let frame_end_result = reservoir.frame_end(2);
    assert!(frame_end_result.is_ok(), "Reservoir frame end should succeed");
}

/// Test shine configuration correctness
/// 
/// Verifies that the shine configuration is properly initialized.
#[test]
fn test_shine_config_correctness() {
    let config = create_test_config();
    let encoder = Mp3Encoder::new(config).unwrap();
    let shine_config = encoder.config();
    
    // Verify basic configuration
    assert_eq!(shine_config.wave.sample_rate, 44100);
    assert_eq!(shine_config.wave.channels, 2); // Stereo = 2 channels
    assert_eq!(shine_config.mpeg.bitrate, 128);
    
    // Verify derived values
    assert!(shine_config.sideinfo_len > 0, "Sideinfo length should be positive");
    assert!(shine_config.mean_bits > 0, "Mean bits should be positive");
    
    // Verify MDCT coefficients are initialized
    let mut mdct_non_zero = 0;
    for m in 0..18 {
        for k in 0..36 {
            if shine_config.mdct.cos_l[m][k] != 0 {
                mdct_non_zero += 1;
            }
        }
    }
    assert!(mdct_non_zero > 0, "MDCT coefficients should be initialized");
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 50,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        
        #[test]
        fn test_encoder_handles_random_input(
            samples in prop::collection::vec(-32768i16..32767i16, 1152*2)
        ) {
            let config = create_test_config();
            let mut encoder = Mp3Encoder::new(config).unwrap();
            
            let result = encoder.encode_frame_interleaved(&samples);
            prop_assert!(result.is_ok(), "Encoding should succeed for any valid input");
            
            let encoded = result.unwrap();
            prop_assert!(!encoded.is_empty(), "Encoded output should not be empty");
            prop_assert!(encoded.len() >= 4, "Encoded frame should have at least header");
        }
        
        #[test]
        fn test_encoder_output_consistency(
            amplitude in 1i16..10000i16
        ) {
            let config = create_test_config();
            
            let pcm_data: Vec<i16> = vec![amplitude; 1152*2];
            
            // Create separate encoder instances for each encoding
            let mut encoder1 = Mp3Encoder::new(config.clone()).unwrap();
            let mut encoder2 = Mp3Encoder::new(config).unwrap();
            
            let result1 = encoder1.encode_frame_interleaved(&pcm_data);
            let result2 = encoder2.encode_frame_interleaved(&pcm_data);
            
            prop_assert!(result1.is_ok() && result2.is_ok(), "Both encodings should succeed");
            
            let encoded1 = result1.unwrap();
            let encoded2 = result2.unwrap();
            
            // Results should be consistent for fresh encoder instances
            prop_assert!(!encoded1.is_empty() && !encoded2.is_empty(), "Both outputs should be non-empty");
            prop_assert_eq!(encoded1, encoded2, "Fresh encoder instances should produce identical output");
        }
    }
}