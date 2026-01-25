//! # Rust MP3 Encoder
//!
//! A pure Rust implementation of an MP3 encoder based on the shine library.
//! This library provides a complete MP3 Layer III encoding solution with
//! support for various sample rates, bitrates, and channel configurations.
//!

use std::sync::atomic::{AtomicI32, Ordering};

/// Global frame counter for debugging consistency across modules
pub static GLOBAL_FRAME_COUNT: AtomicI32 = AtomicI32::new(0);

/// Get the current frame number and increment the global counter
pub fn get_next_frame_number() -> i32 {
    GLOBAL_FRAME_COUNT.fetch_add(1, Ordering::SeqCst) + 1
}

/// Get the current frame number without incrementing
pub fn get_current_frame_number() -> i32 {
    GLOBAL_FRAME_COUNT.load(Ordering::SeqCst)
}

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

#[cfg(test)]
pub mod tests;

// Re-export high-level interface (recommended for most users)
pub use mp3_encoder::{
    Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3,
    SUPPORTED_SAMPLE_RATES, SUPPORTED_BITRATES
};

// Re-export low-level interface (for advanced users)
pub use encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_flush, shine_close, shine_set_config_mpeg_defaults};
pub use error::{EncoderError, ConfigError, InputDataError, EncodingError, EncodingResult};
pub use types::ShineGlobalConfig;
