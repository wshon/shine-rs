//! Bit reservoir implementation for MP3 encoding
//!
//! This module implements the bit reservoir mechanism described in C.1.5.4.2.2 of the IS.
//! The bit reservoir allows frames to borrow bits from future frames for better quality
//! distribution, following shine's reservoir.c implementation exactly.

use crate::bitstream::SideInfo;

/// Bit reservoir for managing bit allocation across frames
/// Following shine's reservoir implementation
#[derive(Debug)]
pub struct BitReservoir {
    /// Current reservoir size in bits
    pub reservoir_size: i32,
    /// Maximum reservoir size in bits
    pub reservoir_max: i32,
    /// Mean bits per frame
    pub mean_bits: i32,
}

impl BitReservoir {
    /// Create a new bit reservoir
    /// Following shine's initialization logic
    pub fn new(bitrate: u32, sample_rate: u32, _channels: u8) -> Self {
        // Calculate mean bits per frame following shine's logic
        let samples_per_frame = if sample_rate >= 32000 { 1152 } else { 576 };
        let mean_bits = (bitrate * 1000 * samples_per_frame) / sample_rate;
        
        // Calculate maximum reservoir size (following shine's logic)
        // For MPEG-1: max 511 bytes = 4088 bits
        // For MPEG-2/2.5: max 255 bytes = 2040 bits  
        let reservoir_max = if sample_rate >= 32000 { 4088 } else { 2040 };
        
        Self {
            reservoir_size: 0,
            reservoir_max,
            mean_bits: mean_bits as i32,
        }
    }
    
    /// Get maximum reservoir bits for current granule
    /// Following shine's shine_max_reservoir_bits function exactly
    pub fn max_reservoir_bits(&self, perceptual_entropy: f64, channels: u8) -> i32 {
        let mean_bits = self.mean_bits / channels as i32;
        let mut max_bits = mean_bits;
        
        if max_bits > 4095 {
            max_bits = 4095;
        }
        
        if self.reservoir_max == 0 {
            return max_bits;
        }
        
        let more_bits = (perceptual_entropy * 3.1) as i32 - mean_bits;
        let mut add_bits = 0;
        
        if more_bits > 100 {
            let frac = (self.reservoir_size * 6) / 10;
            
            if frac < more_bits {
                add_bits = frac;
            } else {
                add_bits = more_bits;
            }
        }
        
        let over_bits = self.reservoir_size - ((self.reservoir_max << 3) / 10) - add_bits;
        if over_bits > 0 {
            add_bits += over_bits;
        }
        
        max_bits += add_bits;
        if max_bits > 4095 {
            max_bits = 4095;
        }
        
        max_bits
    }
    
    /// Adjust reservoir after granule encoding
    /// Following shine's shine_ResvAdjust function exactly
    pub fn adjust_reservoir(&mut self, bits_used: i32, channels: u8) {
        self.reservoir_size += (self.mean_bits / channels as i32) - bits_used;
    }
    
    /// Finalize reservoir at frame end
    /// Following shine's shine_ResvFrameEnd function exactly
    pub fn frame_end(&mut self, side_info: &mut SideInfo, channels: u8) -> i32 {
        let ancillary_pad = 0;
        
        // Handle odd mean_bits for stereo
        if channels == 2 && (self.mean_bits & 1) != 0 {
            self.reservoir_size += 1;
        }
        
        let mut over_bits = self.reservoir_size - self.reservoir_max;
        if over_bits < 0 {
            over_bits = 0;
        }
        
        self.reservoir_size -= over_bits;
        let mut stuffing_bits = over_bits + ancillary_pad;
        
        // Ensure byte alignment
        let alignment_bits = self.reservoir_size % 8;
        if alignment_bits != 0 {
            stuffing_bits += alignment_bits;
            self.reservoir_size -= alignment_bits;
        }
        
        if stuffing_bits > 0 && !side_info.granules.is_empty() {
            // Plan A: put all into the first granule
            if side_info.granules[0].part2_3_length + (stuffing_bits as u32) < 4095 {
                side_info.granules[0].part2_3_length += stuffing_bits as u32;
                stuffing_bits = 0;
            } else {
                // Plan B: distribute throughout granules
                for granule in &mut side_info.granules {
                    if stuffing_bits == 0 {
                        break;
                    }
                    let extra_bits = 4095 - granule.part2_3_length;
                    let bits_this_gr = if extra_bits < (stuffing_bits as u32) { 
                        extra_bits 
                    } else { 
                        stuffing_bits as u32
                    };
                    granule.part2_3_length += bits_this_gr;
                    stuffing_bits -= bits_this_gr as i32;
                }
            }
        }
        
        stuffing_bits // Return remaining stuffing bits for ancillary data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_reservoir_initialization(
            bitrate in 32u32..=320,
            sample_rate in prop::sample::select(&[44100u32, 48000, 32000, 22050, 24000, 16000]),
            channels in 1u8..=2,
        ) {
            let reservoir = BitReservoir::new(bitrate, sample_rate, channels);
            
            prop_assert!(reservoir.mean_bits > 0, "Mean bits should be positive");
            prop_assert!(reservoir.reservoir_max > 0, "Max reservoir should be positive");
            prop_assert_eq!(reservoir.reservoir_size, 0, "Initial reservoir size should be zero");
            
            // Verify reservoir max follows MPEG standards
            if sample_rate >= 32000 {
                prop_assert_eq!(reservoir.reservoir_max, 4088, "MPEG-1 max reservoir should be 4088 bits");
            } else {
                prop_assert_eq!(reservoir.reservoir_max, 2040, "MPEG-2/2.5 max reservoir should be 2040 bits");
            }
        }

        #[test]
        fn test_max_reservoir_bits_bounds(
            bitrate in 32u32..=320,
            sample_rate in prop::sample::select(&[44100u32, 48000, 32000, 22050, 24000, 16000]),
            channels in 1u8..=2,
            pe in 0.0f64..=1000.0,
        ) {
            let reservoir = BitReservoir::new(bitrate, sample_rate, channels);
            let max_bits = reservoir.max_reservoir_bits(pe, channels);
            
            prop_assert!(max_bits > 0, "Max bits should be positive");
            prop_assert!(max_bits <= 4095, "Max bits should not exceed 4095");
        }

        #[test]
        fn test_reservoir_adjustment(
            bitrate in 32u32..=320,
            sample_rate in prop::sample::select(&[44100u32, 48000, 32000, 22050, 24000, 16000]),
            channels in 1u8..=2,
            bits_used in 0i32..=4095,
        ) {
            let mut reservoir = BitReservoir::new(bitrate, sample_rate, channels);
            let initial_size = reservoir.reservoir_size;
            
            reservoir.adjust_reservoir(bits_used, channels);
            
            let expected_change = (reservoir.mean_bits / channels as i32) - bits_used;
            prop_assert_eq!(
                reservoir.reservoir_size, 
                initial_size + expected_change,
                "Reservoir size should change by expected amount"
            );
        }
    }
}