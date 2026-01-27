//! # Rust MP3 Encoder
//!
//! A pure Rust implementation of an MP3 encoder based on the shine library.
//! This library provides a complete MP3 Layer III encoding solution with
//! support for various sample rates, bitrates, and channel configurations.
//!

use std::sync::Mutex;
use std::collections::HashMap;
use std::thread;
use lazy_static::lazy_static;

lazy_static! {
    /// Thread-local frame counters for debugging consistency across modules
    static ref THREAD_FRAME_COUNTERS: Mutex<HashMap<std::thread::ThreadId, i32>> = Mutex::new(HashMap::new());
}

/// Reset the frame counter for current thread (for testing)
pub fn reset_frame_counter() {
    let thread_id = thread::current().id();
    let mut counters = THREAD_FRAME_COUNTERS.lock().unwrap();
    counters.insert(thread_id, 0);
    
    // Also reset TestDataCollector if diagnostics feature is enabled
    #[cfg(feature = "diagnostics")]
    crate::diagnostics_data::TestDataCollector::reset();
}

/// Get the current frame number and increment the counter for current thread
pub fn get_next_frame_number() -> i32 {
    let thread_id = thread::current().id();
    let mut counters = THREAD_FRAME_COUNTERS.lock().unwrap();
    let counter = counters.entry(thread_id).or_insert(0);
    *counter += 1;
    *counter
}

/// Get the current frame number without incrementing for current thread
pub fn get_current_frame_number() -> i32 {
    let thread_id = thread::current().id();
    let counters = THREAD_FRAME_COUNTERS.lock().unwrap();
    *counters.get(&thread_id).unwrap_or(&0)
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

#[cfg(feature = "diagnostics")]
pub mod diagnostics_data;



// Re-export high-level interface (recommended for most users)
pub use mp3_encoder::{
    Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3,
    SUPPORTED_SAMPLE_RATES, SUPPORTED_BITRATES
};

// Re-export low-level interface (for advanced users)
pub use encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_flush, shine_close, shine_set_config_mpeg_defaults};
pub use error::{EncoderError, ConfigError, InputDataError, EncodingError, EncodingResult};
pub use types::ShineGlobalConfig;
