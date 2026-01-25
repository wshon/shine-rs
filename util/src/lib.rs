//! Utility functions for MP3 encoder
//!
//! This crate provides common utility functions used by the MP3 encoder,
//! including PCM audio data processing utilities.

pub mod error;
pub mod pcm_utils;

// Re-export commonly used functions and types
pub use error::*;
pub use pcm_utils::*;