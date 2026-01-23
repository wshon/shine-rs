//! # Rust MP3 Encoder
//!
//! A pure Rust implementation of an MP3 encoder based on the shine library.
//! This library provides a complete MP3 Layer III encoding solution with
//! support for various sample rates, bitrates, and channel configurations.
//!
//! ## Example
//!
//! ```rust,no_run
//! use rust_mp3_encoder::{Mp3Encoder, Config};
//!
//! let config = Config::new();
//! let mut encoder = Mp3Encoder::new(config)?;
//! 
//! // Encode PCM data
//! let pcm_data = vec![0i16; 1152]; // One frame of stereo PCM data
//! let mp3_frame = encoder.encode_frame(&pcm_data)?;
//! 
//! // Flush remaining data
//! let final_frame = encoder.flush()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod config;
pub mod tables;
pub mod bitstream;
pub mod subband;
pub mod mdct;
pub mod quantization;
pub mod reservoir;
pub mod huffman;
pub mod encoder;
pub mod error;
pub mod shine_config;
pub mod pcm_utils;

// Re-export main types for convenience
pub use config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
pub use encoder::Mp3Encoder;
pub use error::{EncoderError, ConfigError, InputDataError, EncodingError};

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, EncoderError>;