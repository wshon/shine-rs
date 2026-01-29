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
pub use diagnostics::{get_current_frame_number, get_next_frame_number, reset_frame_counter};

// Stub functions when diagnostics feature is not enabled
#[cfg(not(feature = "diagnostics"))]
pub fn reset_frame_counter() {}

#[cfg(not(feature = "diagnostics"))]
pub fn get_next_frame_number() -> i32 {
    1
}

#[cfg(not(feature = "diagnostics"))]
pub fn get_current_frame_number() -> i32 {
    1
}

// Re-export high-level interface (recommended for most users)
pub use mp3_encoder::{
    encode_pcm_to_mp3, Mp3Encoder, Mp3EncoderConfig, StereoMode, SUPPORTED_BITRATES,
    SUPPORTED_SAMPLE_RATES,
};

// Re-export low-level interface (for advanced users)
pub use encoder::{
    shine_close, shine_encode_buffer_interleaved, shine_flush, shine_initialise,
    shine_set_config_mpeg_defaults, ShineConfig, ShineMpeg, ShineWave,
};
pub use error::{ConfigError, EncoderError, EncodingError, EncodingResult, InputDataError};
pub use types::ShineGlobalConfig;
