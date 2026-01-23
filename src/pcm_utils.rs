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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deinterleave_non_interleaved_stereo() {
        let pcm_data = vec![1, 2, 3, 4, 5, 6]; // L: [1,2,3], R: [4,5,6]
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_non_interleaved(&pcm_data, 2, 3, &mut buffers);
        
        assert_eq!(buffers[0], vec![1, 2, 3]);
        assert_eq!(buffers[1], vec![4, 5, 6]);
    }

    #[test]
    fn test_deinterleave_interleaved_stereo() {
        let pcm_data = vec![1, 4, 2, 5, 3, 6]; // Interleaved: L1,R1,L2,R2,L3,R3
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 3, &mut buffers);
        
        assert_eq!(buffers[0], vec![1, 2, 3]);
        assert_eq!(buffers[1], vec![4, 5, 6]);
    }

    #[test]
    fn test_deinterleave_mono() {
        let pcm_data = vec![1, 2, 3, 4];
        let mut buffers = vec![Vec::new()];
        
        deinterleave_pcm_non_interleaved(&pcm_data, 1, 4, &mut buffers);
        
        assert_eq!(buffers[0], vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_deinterleave_partial_data() {
        let pcm_data = vec![1, 2]; // Less data than expected
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 3, &mut buffers);
        
        assert_eq!(buffers[0], vec![1]);
        assert_eq!(buffers[1], vec![2]);
    }
}