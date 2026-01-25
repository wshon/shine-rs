//! Unit tests for PCM utilities
//!
//! Tests the PCM data processing functions including deinterleaving
//! and format conversion utilities.

use crate::pcm_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deinterleave_non_interleaved_stereo() {
        let pcm_data = vec![1, 2, 3, 4, 5, 6]; // L: [1,2,3], R: [4,5,6]
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_non_interleaved(&pcm_data, 2, 3, &mut buffers);
        
        assert_eq!(buffers[0], vec![1, 2, 3], "Left channel should be extracted correctly");
        assert_eq!(buffers[1], vec![4, 5, 6], "Right channel should be extracted correctly");
    }

    #[test]
    fn test_deinterleave_interleaved_stereo() {
        let pcm_data = vec![1, 4, 2, 5, 3, 6]; // Interleaved: L1,R1,L2,R2,L3,R3
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 3, &mut buffers);
        
        assert_eq!(buffers[0], vec![1, 2, 3], "Left channel should be deinterleaved correctly");
        assert_eq!(buffers[1], vec![4, 5, 6], "Right channel should be deinterleaved correctly");
    }

    #[test]
    fn test_deinterleave_mono() {
        let pcm_data = vec![1, 2, 3, 4];
        let mut buffers = vec![Vec::new()];
        
        deinterleave_pcm_non_interleaved(&pcm_data, 1, 4, &mut buffers);
        
        assert_eq!(buffers[0], vec![1, 2, 3, 4], "Mono channel should be copied correctly");
    }

    #[test]
    fn test_deinterleave_partial_data() {
        let pcm_data = vec![1, 2]; // Less data than expected
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 3, &mut buffers);
        
        assert_eq!(buffers[0], vec![1], "Should handle partial left channel");
        assert_eq!(buffers[1], vec![2], "Should handle partial right channel");
    }

    #[test]
    fn test_deinterleave_empty_data() {
        let pcm_data = vec![];
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 0, &mut buffers);
        
        assert_eq!(buffers[0], Vec::<i16>::new(), "Left channel should be empty");
        assert_eq!(buffers[1], Vec::<i16>::new(), "Right channel should be empty");
    }

    #[test]
    fn test_deinterleave_large_data() {
        // Test with larger data set to ensure performance
        let mut pcm_data = Vec::new();
        for i in 0..2048 {
            pcm_data.push(i as i16);           // Left channel: 0, 2, 4, ...
            pcm_data.push((i + 10000) as i16); // Right channel: 10000, 10002, 10004, ...
        }
        
        let mut buffers = vec![Vec::new(), Vec::new()];
        deinterleave_pcm_interleaved(&pcm_data, 2, 2048, &mut buffers);
        
        assert_eq!(buffers[0].len(), 2048, "Left channel should have correct length");
        assert_eq!(buffers[1].len(), 2048, "Right channel should have correct length");
        
        // Verify first few values
        assert_eq!(buffers[0][0], 0, "First left sample should be correct");
        assert_eq!(buffers[1][0], 10000, "First right sample should be correct");
        assert_eq!(buffers[0][1], 1, "Second left sample should be correct");
        assert_eq!(buffers[1][1], 10001, "Second right sample should be correct");
    }

    #[test]
    fn test_deinterleave_boundary_values() {
        // Test with boundary values for i16
        let pcm_data = vec![i16::MIN, i16::MAX, 0, -1];
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 2, &mut buffers);
        
        assert_eq!(buffers[0], vec![i16::MIN, 0], "Should handle MIN and zero correctly");
        assert_eq!(buffers[1], vec![i16::MAX, -1], "Should handle MAX and -1 correctly");
    }

    #[test]
    fn test_deinterleave_single_sample() {
        // Test with single sample per channel
        let pcm_data = vec![100, 200];
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        deinterleave_pcm_interleaved(&pcm_data, 2, 1, &mut buffers);
        
        assert_eq!(buffers[0], vec![100], "Single left sample should be correct");
        assert_eq!(buffers[1], vec![200], "Single right sample should be correct");
    }

    #[test]
    fn test_deinterleave_buffer_reuse() {
        // Test that buffers are properly cleared and reused
        let pcm_data1 = vec![1, 2, 3, 4];
        let pcm_data2 = vec![10, 20, 30, 40];
        let mut buffers = vec![Vec::new(), Vec::new()];
        
        // First deinterleave
        deinterleave_pcm_interleaved(&pcm_data1, 2, 2, &mut buffers);
        assert_eq!(buffers[0], vec![1, 3], "First deinterleave left channel");
        assert_eq!(buffers[1], vec![2, 4], "First deinterleave right channel");
        
        // Clear and reuse buffers
        buffers[0].clear();
        buffers[1].clear();
        
        // Second deinterleave
        deinterleave_pcm_interleaved(&pcm_data2, 2, 2, &mut buffers);
        assert_eq!(buffers[0], vec![10, 30], "Second deinterleave left channel");
        assert_eq!(buffers[1], vec![20, 40], "Second deinterleave right channel");
    }
}