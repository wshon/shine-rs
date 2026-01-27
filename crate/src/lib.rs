//! # Rust MP3 Encoder
//!
//! A pure Rust implementation of an MP3 encoder based on the shine library.
//! This library provides a complete MP3 Layer III encoding solution with
//! support for various sample rates, bitrates, and channel configurations.
//!

pub mod bitstream;
pub mod encoder;
pub mod error;
pub mod huffman;
pub mod mdct;
pub mod mp3_encoder;
pub mod quantization;
pub mod reservoir;
pub mod subband;
pub mod tables;
pub mod types;

#[cfg(feature = "diagnostics")]
pub mod diagnostics;

// Re-export diagnostics functions for backward compatibility
#[cfg(feature = "diagnostics")]
pub use diagnostics::{reset_frame_counter, get_next_frame_number, get_current_frame_number};

// Stub functions when diagnostics feature is not enabled
#[cfg(not(feature = "diagnostics"))]
pub fn reset_frame_counter() {}

#[cfg(not(feature = "diagnostics"))]
pub fn get_next_frame_number() -> i32 { 1 }

#[cfg(not(feature = "diagnostics"))]
pub fn get_current_frame_number() -> i32 { 1 }



// Re-export high-level interface (recommended for most users)
pub use mp3_encoder::{
    Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3,
    SUPPORTED_SAMPLE_RATES, SUPPORTED_BITRATES
};

// Re-export low-level interface (for advanced users)
pub use encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_flush, shine_close, shine_set_config_mpeg_defaults};
pub use error::{EncoderError, ConfigError, InputDataError, EncodingError, EncodingResult};
pub use types::ShineGlobalConfig;
