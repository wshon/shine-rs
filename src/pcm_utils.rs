//! PCM audio data processing utilities
//!
//! This module provides utility functions for processing PCM audio data,
//! including interleaving/deinterleaving operations.

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

