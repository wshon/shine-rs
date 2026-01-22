//! Integration tests for the MP3 encoder
//!
//! These tests verify the overall functionality of the encoder
//! with various configurations and input data.

use rust_mp3_encoder::{Mp3Encoder, Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[test]
fn test_encoder_creation() {
    let config = Config::new();
    let encoder = Mp3Encoder::new(config);
    assert!(encoder.is_ok(), "Should be able to create encoder with default config");
}

#[test]
fn test_config_validation() {
    let mut config = Config::new();
    
    // Test valid configuration
    assert!(config.validate().is_ok(), "Default config should be valid");
    
    // Test invalid sample rate
    config.wave.sample_rate = 12345;
    assert!(config.validate().is_err(), "Invalid sample rate should fail validation");
    
    // Test invalid bitrate
    config.wave.sample_rate = 44100;
    config.mpeg.bitrate = 999;
    assert!(config.validate().is_err(), "Invalid bitrate should fail validation");
}

#[test]
fn test_samples_per_frame() {
    let mut config = Config::new();
    
    // MPEG-1 should have 1152 samples per frame
    config.wave.sample_rate = 44100;
    let encoder = Mp3Encoder::new(config.clone()).unwrap();
    assert_eq!(encoder.samples_per_frame(), 1152);
    
    // MPEG-2 should have 576 samples per frame
    config.wave.sample_rate = 22050;
    let encoder = Mp3Encoder::new(config).unwrap();
    assert_eq!(encoder.samples_per_frame(), 576);
}