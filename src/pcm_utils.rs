//! PCM audio data processing utilities
//!
//! This module provides utility functions for processing PCM audio data,
//! including interleaving/deinterleaving operations and WAV file reading.

use crate::error::{EncodingError, EncodingResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Read WAV file and return PCM samples, sample rate, and channel count
pub fn read_wav_file(file_path: &str) -> EncodingResult<(Vec<i16>, i32, i32)> {
    let mut file = File::open(file_path)
        .map_err(|e| EncodingError::ValidationError(format!("Failed to open WAV file: {}", e)))?;

    // Read WAV header
    let mut header = [0u8; 44];
    file.read_exact(&mut header)
        .map_err(|e| EncodingError::ValidationError(format!("Failed to read WAV header: {}", e)))?;

    // Validate WAV format
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err(EncodingError::ValidationError("Invalid WAV file format".to_string()));
    }

    // Extract format information
    let channels = u16::from_le_bytes([header[22], header[23]]) as i32;
    let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]) as i32;
    let bits_per_sample = u16::from_le_bytes([header[34], header[35]]);

    if bits_per_sample != 16 {
        return Err(EncodingError::ValidationError("Only 16-bit WAV files are supported".to_string()));
    }

    // Find data chunk
    file.seek(SeekFrom::Start(36))
        .map_err(|e| EncodingError::ValidationError(format!("Failed to seek in WAV file: {}", e)))?;

    let mut chunk_header = [0u8; 8];
    file.read_exact(&mut chunk_header)
        .map_err(|e| EncodingError::ValidationError(format!("Failed to read chunk header: {}", e)))?;

    if &chunk_header[0..4] != b"data" {
        return Err(EncodingError::ValidationError("Data chunk not found".to_string()));
    }

    let data_size = u32::from_le_bytes([chunk_header[4], chunk_header[5], chunk_header[6], chunk_header[7]]) as usize;
    let sample_count = data_size / 2; // 16-bit samples

    // Read PCM data
    let mut pcm_data = vec![0u8; data_size];
    file.read_exact(&mut pcm_data)
        .map_err(|e| EncodingError::ValidationError(format!("Failed to read PCM data: {}", e)))?;

    // Convert to i16 samples
    let mut samples = Vec::with_capacity(sample_count);
    for chunk in pcm_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample);
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
    channel_buffers: &mut [Vec<i16>]
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
    channel_buffers: &mut [Vec<i16>]
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

