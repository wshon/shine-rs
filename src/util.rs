//! Utility functions for MP3 encoder
//!
//! This module provides common utility functions used by the MP3 encoder,
//! including PCM audio data processing utilities and error handling.

use std::fmt;

/// Error type for utility operations
#[derive(Debug)]
pub enum UtilError {
    /// I/O operation failed
    IoError(std::io::Error),
    /// Validation error
    ValidationError(String),
}

impl fmt::Display for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UtilError::IoError(err) => write!(f, "I/O error: {}", err),
            UtilError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for UtilError {}

impl From<std::io::Error> for UtilError {
    fn from(err: std::io::Error) -> Self {
        UtilError::IoError(err)
    }
}

/// Result type for utility operations
pub type UtilResult<T> = std::result::Result<T, UtilError>;

/// Read WAV file and return PCM samples, sample rate, and channel count
/// Uses hound library for WAV parsing
pub fn read_wav_file(file_path: &str) -> UtilResult<(Vec<i16>, i32, i32)> {
    let mut reader = hound::WavReader::open(file_path)
        .map_err(|e| UtilError::ValidationError(format!("Failed to open WAV file: {}", e)))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate as i32;
    let channels = spec.channels as i32;

    // Read all samples
    let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    let samples = samples
        .map_err(|e| UtilError::ValidationError(format!("Failed to read WAV samples: {}", e)))?;

    if samples.is_empty() {
        return Err(UtilError::ValidationError(
            "No audio data found in WAV file".to_string(),
        ));
    }

    Ok((samples, sample_rate, channels))
}

/// De-interleave non-interleaved PCM data into separate channel buffers
///
/// Takes PCM data in format [L0, L1, ..., LN, R0, R1, ..., RN] and
/// separates it into individual channel buffers.
pub fn deinterleave_pcm_non_interleaved(
    pcm_data: &[i16],
    channels: usize,
    samples_per_frame: usize,
    channel_buffers: &mut [Vec<i16>],
) {
    for ch in 0..channels {
        if ch < channel_buffers.len() {
            channel_buffers[ch].clear();
            channel_buffers[ch].reserve(samples_per_frame);

            let channel_start = ch * samples_per_frame;
            let channel_end = channel_start + samples_per_frame;

            for sample_idx in channel_start..channel_end {
                if sample_idx < pcm_data.len() {
                    channel_buffers[ch].push(pcm_data[sample_idx]);
                }
            }
        }
    }
}

/// De-interleave interleaved PCM data into separate channel buffers
///
/// Takes PCM data in format [L0, R0, L1, R1, ..., LN, RN] and
/// separates it into individual channel buffers.
pub fn deinterleave_pcm_interleaved(
    pcm_data: &[i16],
    channels: usize,
    samples_per_frame: usize,
    channel_buffers: &mut [Vec<i16>],
) {
    for ch in 0..channels {
        if ch < channel_buffers.len() {
            channel_buffers[ch].clear();
            channel_buffers[ch].reserve(samples_per_frame);
        }
    }

    for sample_idx in 0..samples_per_frame {
        for ch in 0..channels {
            if ch < channel_buffers.len() {
                let interleaved_idx = sample_idx * channels + ch;
                if interleaved_idx < pcm_data.len() {
                    channel_buffers[ch].push(pcm_data[interleaved_idx]);
                }
            }
        }
    }
}
