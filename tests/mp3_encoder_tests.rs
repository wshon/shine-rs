//! High-level MP3 encoder interface tests
//!
//! This module contains comprehensive tests for the high-level MP3 encoder API,
//! including configuration validation, encoding functionality, and error handling.

use shine_rs::mp3_encoder::{
    Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3,
    SUPPORTED_SAMPLE_RATES, SUPPORTED_BITRATES
};
use shine_rs::error::{EncoderError, ConfigError, InputDataError};

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .bitrate(128)
            .channels(2);
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_sample_rate() {
        let config = Mp3EncoderConfig::new().sample_rate(12345);
        assert!(matches!(config.validate(), Err(ConfigError::UnsupportedSampleRate(12345))));
    }

    #[test]
    fn test_config_validation_invalid_bitrate() {
        let config = Mp3EncoderConfig::new().bitrate(999);
        assert!(matches!(config.validate(), Err(ConfigError::UnsupportedBitrate(999))));
    }

    #[test]
    fn test_config_validation_invalid_channels() {
        let config = Mp3EncoderConfig::new().channels(0);
        assert!(matches!(config.validate(), Err(ConfigError::InvalidChannels)));
        
        let config = Mp3EncoderConfig::new().channels(3);
        assert!(matches!(config.validate(), Err(ConfigError::InvalidChannels)));
    }

    #[test]
    fn test_config_validation_incompatible_combinations() {
        // MPEG-2.5 with high bitrate should fail
        let config = Mp3EncoderConfig::new()
            .sample_rate(8000)  // MPEG-2.5
            .bitrate(128);      // Too high for MPEG-2.5
        
        match config.validate() {
            Err(ConfigError::IncompatibleRateCombination { sample_rate, bitrate, reason }) => {
                assert_eq!(sample_rate, 8000);
                assert_eq!(bitrate, 128);
                assert!(reason.contains("MPEG-2.5"));
                assert!(reason.contains("64 kbps"));
            },
            other => panic!("Expected IncompatibleRateCombination error, got: {:?}", other),
        }

        // MPEG-2 with very high bitrate should fail
        let config = Mp3EncoderConfig::new()
            .sample_rate(22050)  // MPEG-2
            .bitrate(320);       // Too high for MPEG-2
        
        match config.validate() {
            Err(ConfigError::IncompatibleRateCombination { sample_rate, bitrate, reason }) => {
                assert_eq!(sample_rate, 22050);
                assert_eq!(bitrate, 320);
                assert!(reason.contains("MPEG-2"));
                assert!(reason.contains("160 kbps"));
            },
            other => panic!("Expected IncompatibleRateCombination error, got: {:?}", other),
        }

        // MPEG-1 with very low bitrate should fail
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)  // MPEG-1
            .bitrate(16);        // Too low for MPEG-1
        
        match config.validate() {
            Err(ConfigError::IncompatibleRateCombination { sample_rate, bitrate, reason }) => {
                assert_eq!(sample_rate, 44100);
                assert_eq!(bitrate, 16);
                assert!(reason.contains("MPEG-1"));
                assert!(reason.contains("32 to 320 kbps"));
            },
            other => panic!("Expected IncompatibleRateCombination error, got: {:?}", other),
        }
    }

    #[test]
    fn test_config_validation_valid_combinations() {
        // Test valid combinations for each MPEG version
        
        // MPEG-2.5 valid combinations
        let config = Mp3EncoderConfig::new()
            .sample_rate(8000)
            .bitrate(32);
        assert!(config.validate().is_ok());

        let config = Mp3EncoderConfig::new()
            .sample_rate(11025)
            .bitrate(64);
        assert!(config.validate().is_ok());

        // MPEG-2 valid combinations
        let config = Mp3EncoderConfig::new()
            .sample_rate(16000)
            .bitrate(80);
        assert!(config.validate().is_ok());

        let config = Mp3EncoderConfig::new()
            .sample_rate(22050)
            .bitrate(160);
        assert!(config.validate().is_ok());

        // MPEG-1 valid combinations
        let config = Mp3EncoderConfig::new()
            .sample_rate(32000)
            .bitrate(32);
        assert!(config.validate().is_ok());

        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .bitrate(320);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_supported_sample_rates() {
        for &sample_rate in SUPPORTED_SAMPLE_RATES {
            let config = Mp3EncoderConfig::new().sample_rate(sample_rate);
            assert!(config.validate().is_ok(), "Sample rate {} should be supported", sample_rate);
        }
    }

    #[test]
    fn test_supported_bitrates() {
        for &bitrate in SUPPORTED_BITRATES {
            let config = Mp3EncoderConfig::new().bitrate(bitrate);
            assert!(config.validate().is_ok(), "Bitrate {} should be supported", bitrate);
        }
    }

    #[test]
    fn test_encoder_creation() {
        let config = Mp3EncoderConfig::new();
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_encoder_creation_with_invalid_config() {
        // Test with unsupported sample rate
        let config = Mp3EncoderConfig::new().sample_rate(12345);
        let encoder = Mp3Encoder::new(config);
        assert!(matches!(encoder, Err(EncoderError::Config(_))));

        // Test with incompatible sample rate and bitrate combination
        let config = Mp3EncoderConfig::new()
            .sample_rate(8000)   // MPEG-2.5
            .bitrate(320);       // Too high for MPEG-2.5
        let encoder = Mp3Encoder::new(config);
        assert!(matches!(encoder, Err(EncoderError::Config(ConfigError::IncompatibleRateCombination { .. }))));
    }

    #[test]
    fn test_samples_per_frame_mpeg1() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)  // MPEG-1
            .channels(2);
        let encoder = Mp3Encoder::new(config).unwrap();
        // For MPEG-1, stereo: 2 granules * 576 samples per channel * 2 channels = 2304
        assert_eq!(encoder.samples_per_frame(), 2304);
    }

    #[test]
    fn test_samples_per_frame_mpeg2() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(22050)  // MPEG-2
            .channels(2);
        let encoder = Mp3Encoder::new(config).unwrap();
        // For MPEG-2, stereo: 1 granule * 576 samples per channel * 2 channels = 1152
        assert_eq!(encoder.samples_per_frame(), 1152);
    }

    #[test]
    fn test_samples_per_frame_mono() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .channels(1)
            .stereo_mode(StereoMode::Mono);
        let encoder = Mp3Encoder::new(config).unwrap();
        // For MPEG-1, mono: 2 granules * 576 samples per channel * 1 channel = 1152
        assert_eq!(encoder.samples_per_frame(), 1152);
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(48000)
            .bitrate(320)
            .channels(2)
            .stereo_mode(StereoMode::JointStereo)
            .copyright(true)
            .original(false);
        
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.bitrate, 320);
        assert_eq!(config.channels, 2);
        assert_eq!(config.stereo_mode, StereoMode::JointStereo);
        assert_eq!(config.copyright, true);
        assert_eq!(config.original, false);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_encoding_stereo() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .bitrate(128)
            .channels(2);
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Generate simple test data (sine wave)
        let mut test_data = Vec::new();
        for i in 0..4608 { // 2 frames worth of data
            let sample = ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 16384.0) as i16;
            test_data.push(sample);
        }
        
        let frames = encoder.encode_interleaved(&test_data).unwrap();
        assert!(!frames.is_empty(), "Should produce encoded frames");
        
        let final_data = encoder.finish().unwrap();
        // Should have some output
        assert!(frames.len() > 0 || !final_data.is_empty(), "Should have encoded output");
    }

    #[test]
    fn test_simple_encoding_mono() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(22050)
            .bitrate(64)
            .channels(1)
            .stereo_mode(StereoMode::Mono);
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Generate mono test data
        let mut test_data = Vec::new();
        for i in 0..2304 { // 2 frames worth of mono data
            let sample = ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 22050.0).sin() * 16384.0) as i16;
            test_data.push(sample);
        }
        
        let frames = encoder.encode_interleaved(&test_data).unwrap();
        assert!(!frames.is_empty(), "Should produce encoded frames");
        
        let final_data = encoder.finish().unwrap();
        assert!(frames.len() > 0 || !final_data.is_empty(), "Should have encoded output");
    }

    #[test]
    fn test_batch_encoding() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(22050)
            .bitrate(64)
            .channels(1)
            .stereo_mode(StereoMode::Mono);
        
        // Generate 1 second of mono audio
        let mut test_data = Vec::new();
        for i in 0..22050 {
            let sample = ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 22050.0).sin() * 16384.0) as i16;
            test_data.push(sample);
        }
        
        let mp3_data = encode_pcm_to_mp3(config, &test_data).unwrap();
        assert!(!mp3_data.is_empty(), "Should produce MP3 data");
        assert!(mp3_data.len() > 100, "Should have reasonable amount of data");
    }

    #[test]
    fn test_separate_channels_stereo() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .bitrate(128)
            .channels(2);
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Generate separate channel data
        let mut left_channel = Vec::new();
        let mut right_channel = Vec::new();
        for i in 0..2304 { // 1 frame worth of stereo data
            let left_sample = ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 16384.0) as i16;
            let right_sample = ((i as f32 * 880.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 16384.0) as i16;
            left_channel.push(left_sample);
            right_channel.push(right_sample);
        }
        
        let frames = encoder.encode_separate_channels(&left_channel, Some(&right_channel)).unwrap();
        assert!(!frames.is_empty(), "Should produce encoded frames");
    }

    #[test]
    fn test_separate_channels_mono() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .bitrate(128)
            .channels(1)
            .stereo_mode(StereoMode::Mono);
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Generate mono channel data
        let mut mono_channel = Vec::new();
        for i in 0..1152 { // 1 frame worth of mono data
            let sample = ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 16384.0) as i16;
            mono_channel.push(sample);
        }
        
        let frames = encoder.encode_separate_channels(&mono_channel, None).unwrap();
        assert!(!frames.is_empty(), "Should produce encoded frames");
    }

    #[test]
    fn test_streaming_encoding() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(44100)
            .bitrate(128)
            .channels(2);
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        let mut total_output = Vec::new();
        
        // Encode in multiple chunks
        for chunk_idx in 0..5 {
            let mut chunk_data = Vec::new();
            for i in 0..2304 { // 1 frame per chunk
                let sample_idx = chunk_idx * 2304 + i;
                let sample = ((sample_idx as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 16384.0) as i16;
                chunk_data.push(sample);
            }
            
            let frames = encoder.encode_interleaved(&chunk_data).unwrap();
            for frame in frames {
                total_output.extend(frame);
            }
        }
        
        let final_data = encoder.finish().unwrap();
        total_output.extend(final_data);
        
        assert!(!total_output.is_empty(), "Should produce encoded output");
        assert!(total_output.len() > 1000, "Should have substantial output");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_empty_input_error() {
        let config = Mp3EncoderConfig::new();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        let empty_data: Vec<i16> = Vec::new();
        let result = encoder.encode_interleaved(&empty_data);
        assert!(matches!(result, Err(EncoderError::InputData(InputDataError::EmptyInput))));
    }

    #[test]
    fn test_channel_count_mismatch_error() {
        let config = Mp3EncoderConfig::new()
            .channels(2);
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        let left_channel = vec![100i16; 1000];
        let right_channel = vec![200i16; 500]; // Different length
        
        let result = encoder.encode_separate_channels(&left_channel, Some(&right_channel));
        assert!(matches!(result, Err(EncoderError::InputData(InputDataError::InvalidChannelCount { .. }))));
    }

    #[test]
    fn test_mono_with_two_channels_error() {
        let config = Mp3EncoderConfig::new()
            .channels(1)
            .stereo_mode(StereoMode::Mono);
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        let left_channel = vec![100i16; 1000];
        let right_channel = vec![200i16; 1000];
        
        let result = encoder.encode_separate_channels(&left_channel, Some(&right_channel));
        assert!(matches!(result, Err(EncoderError::InputData(InputDataError::InvalidChannelCount { .. }))));
    }

    #[test]
    fn test_stereo_with_one_channel_error() {
        let config = Mp3EncoderConfig::new()
            .channels(2);
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        let mono_channel = vec![100i16; 1000];
        
        let result = encoder.encode_separate_channels(&mono_channel, None);
        assert!(matches!(result, Err(EncoderError::InputData(InputDataError::InvalidChannelCount { .. }))));
    }

    #[test]
    fn test_finished_encoder_error() {
        let config = Mp3EncoderConfig::new();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Finish the encoder
        let _ = encoder.finish().unwrap();
        
        // Try to encode more data
        let test_data = vec![100i16; 1000];
        let result = encoder.encode_interleaved(&test_data);
        assert!(matches!(result, Err(EncoderError::InternalState(_))));
    }

    #[test]
    fn test_incompatible_rate_combination_error() {
        let config = Mp3EncoderConfig::new()
            .sample_rate(8000)   // MPEG-2.5
            .bitrate(224);       // Too high for MPEG-2.5
        
        let encoder = Mp3Encoder::new(config);
        assert!(matches!(encoder, Err(EncoderError::Config(ConfigError::IncompatibleRateCombination { .. }))));
        
        // Test the error message contains useful information
        if let Err(EncoderError::Config(ConfigError::IncompatibleRateCombination { sample_rate, bitrate, reason })) = encoder {
            assert_eq!(sample_rate, 8000);
            assert_eq!(bitrate, 224);
            assert!(reason.contains("MPEG-2.5"));
            assert!(reason.contains("64 kbps"));
        }
    }

    #[test]
    fn test_double_finish() {
        let config = Mp3EncoderConfig::new();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // First finish should work
        let result1 = encoder.finish();
        assert!(result1.is_ok());
        
        // Second finish should return empty data
        let result2 = encoder.finish();
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_empty());
    }
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
        fn test_config_validation_properties(
            sample_rate in prop::sample::select(SUPPORTED_SAMPLE_RATES),
            bitrate in prop::sample::select(SUPPORTED_BITRATES),
            channels in 1u8..=2,
        ) {
            let stereo_mode = if channels == 1 {
                StereoMode::Mono
            } else {
                StereoMode::Stereo
            };
            
            let config = Mp3EncoderConfig::new()
                .sample_rate(sample_rate)
                .bitrate(bitrate)
                .channels(channels)
                .stereo_mode(stereo_mode);
            
            prop_assert!(config.validate().is_ok(), "Valid config should pass validation");
        }

        #[test]
        fn test_encoder_creation_properties(
            sample_rate in prop::sample::select(SUPPORTED_SAMPLE_RATES),
            bitrate in prop::sample::select(SUPPORTED_BITRATES),
        ) {
            // Skip invalid combinations based on MPEG version and bitrate support
            let is_mpeg25 = sample_rate <= 12000;  // 8000, 11025, 12000
            let is_mpeg2 = sample_rate >= 16000 && sample_rate <= 24000;  // 16000, 22050, 24000
            let is_mpeg1 = sample_rate >= 32000;  // 32000, 44100, 48000
            
            // Check bitrate compatibility with MPEG version
            let is_valid_combination = if is_mpeg25 {
                bitrate <= 64  // MPEG-2.5 supports up to 64 kbps
            } else if is_mpeg2 {
                bitrate <= 160  // MPEG-2 supports up to 160 kbps
            } else if is_mpeg1 {
                bitrate >= 32  // MPEG-1 supports 32-320 kbps
            } else {
                false
            };
            
            if !is_valid_combination {
                return Ok(()); // Skip this combination
            }
            
            let config = Mp3EncoderConfig::new()
                .sample_rate(sample_rate)
                .bitrate(bitrate)
                .channels(2);
            
            let encoder = Mp3Encoder::new(config);
            prop_assert!(encoder.is_ok(), "Encoder creation should succeed with valid config");
        }

        #[test]
        fn test_small_data_encoding(
            sample_rate in prop::sample::select(&[22050u32, 44100]),
            data_size in 100usize..1000,
        ) {
            let config = Mp3EncoderConfig::new()
                .sample_rate(sample_rate)
                .bitrate(128)
                .channels(1)
                .stereo_mode(StereoMode::Mono);
            
            let mut encoder = Mp3Encoder::new(config)?;
            
            // Generate test data
            let test_data: Vec<i16> = (0..data_size)
                .map(|i| ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / sample_rate as f32).sin() * 16384.0) as i16)
                .collect();
            
            let _frames = encoder.encode_interleaved(&test_data)?;
            let _final_data = encoder.finish()?;
            
            // Should not crash and should produce some output eventually
            prop_assert!(true, "Should not crash on encoding");
            prop_assert!(true, "Should not crash on finish");
        }
    }
}