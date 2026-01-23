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
pub mod pcm_utils;
pub mod quantization;
pub mod reservoir;
pub mod subband;
pub mod tables;
pub mod types;

// Re-export commonly used types and functions for easier access
pub use encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_flush, shine_close, shine_set_config_mpeg_defaults};
pub use error::{EncodingError, EncodingResult};
pub use types::ShineGlobalConfig;
