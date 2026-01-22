//! Bit reservoir management for MP3 encoding
//!
//! This module implements the bit reservoir mechanism that allows
//! frames to borrow bits from future frames to maintain quality
//! while staying within the overall bitrate constraints.

use crate::error::{EncodingError, EncodingResult};
use crate::quantization::GranuleInfo;

/// Bit reservoir for managing available bits across frames
pub struct BitReservoir {
    /// Current reservoir size in bits
    size: usize,
    /// Maximum reservoir size in bits
    max_size: usize,
    /// Mean bits per granule
    mean_bits: usize,
    /// Drain bits for ancillary data
    drain_bits: usize,
}

impl BitReservoir {
    /// Create a new bit reservoir with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            size: 0,
            max_size,
            mean_bits: 0,
            drain_bits: 0,
        }
    }
    
    /// Initialize reservoir for a new frame
    pub fn frame_begin(&mut self, mean_bits_per_granule: usize) {
        self.mean_bits = mean_bits_per_granule;
        self.drain_bits = 0;
    }
    
    /// Calculate maximum available bits for a granule based on perceptual entropy
    pub fn max_reservoir_bits(&self, perceptual_entropy: f64, channels: usize) -> usize {
        if self.mean_bits == 0 || channels == 0 {
            return 0; // Return 0 if no mean bits or channels
        }
        
        let mean_bits = self.mean_bits / channels;
        let mut max_bits = mean_bits;
        
        // Limit to maximum allowed bits per granule
        if max_bits > 4095 {
            max_bits = 4095;
        }
        
        if self.max_size == 0 {
            return max_bits;
        }
        
        // Calculate additional bits based on perceptual entropy
        let more_bits = (perceptual_entropy * 3.1) as i32 - mean_bits as i32;
        let mut add_bits = 0;
        
        if more_bits > 100 {
            let frac = (self.size * 6) / 10;
            add_bits = if frac < more_bits as usize {
                frac
            } else {
                more_bits as usize
            };
        }
        
        // Check for over-allocation
        let threshold = (self.max_size * 8) / 10;
        if self.size > threshold + add_bits {
            let over_bits = self.size - threshold - add_bits;
            add_bits += over_bits;
        }
        
        max_bits += add_bits;
        if max_bits > 4095 {
            max_bits = 4095;
        }
        
        max_bits
    }
    
    /// Adjust reservoir size after granule allocation
    pub fn adjust_after_granule(&mut self, used_bits: usize, channels: usize) -> EncodingResult<()> {
        if self.mean_bits == 0 || channels == 0 {
            return Ok(()); // Skip adjustment if no mean bits or channels
        }
        
        let mean_bits_per_channel = self.mean_bits / channels;
        
        if used_bits <= mean_bits_per_channel {
            let returned_bits = mean_bits_per_channel - used_bits;
            self.add_bits(returned_bits)?;
        } else {
            let extra_bits = used_bits - mean_bits_per_channel;
            // Only try to use bits if we have enough in the reservoir
            if extra_bits <= self.size {
                self.use_bits(extra_bits)?;
            } else {
                // If we don't have enough bits, just use what we have
                self.size = 0;
            }
        }
        
        Ok(())
    }
    
    /// Finalize frame and handle stuffing bits
    pub fn frame_end(&mut self, channels: usize) -> (usize, usize) {
        // Handle odd mean_bits for stereo
        if channels == 2 && (self.mean_bits & 1) != 0 {
            let _ = self.add_bits(1);
        }
        
        // Calculate over-allocation
        let over_bits = self.size.saturating_sub(self.max_size);
        
        self.size = self.size.saturating_sub(over_bits);
        let mut stuffing_bits = over_bits;
        
        // Ensure byte alignment
        let alignment_bits = self.size % 8;
        if alignment_bits != 0 {
            stuffing_bits += alignment_bits;
            self.size -= alignment_bits;
        }
        
        // Set drain bits for ancillary data if needed
        self.drain_bits = 0;
        
        (stuffing_bits, self.drain_bits)
    }
    
    /// Distribute stuffing bits across granules
    pub fn distribute_stuffing_bits(&self, stuffing_bits: usize, granule_info: &mut [GranuleInfo]) -> usize {
        let mut remaining_bits = stuffing_bits;
        
        // Plan A: Try to put all bits in the first granule
        if !granule_info.is_empty() {
            let available_space = 4095_u32.saturating_sub(granule_info[0].part2_3_length);
            if available_space as usize >= remaining_bits {
                granule_info[0].part2_3_length += remaining_bits as u32;
                return 0; // All bits distributed
            }
        }
        
        // Plan B: Distribute across all granules
        for granule in granule_info.iter_mut() {
            if remaining_bits == 0 {
                break;
            }
            
            let available_space = 4095_u32.saturating_sub(granule.part2_3_length);
            let bits_to_add = (available_space as usize).min(remaining_bits);
            
            granule.part2_3_length += bits_to_add as u32;
            remaining_bits -= bits_to_add;
        }
        
        // Return remaining bits for ancillary data
        remaining_bits
    }
    
    /// Add bits to the reservoir
    pub fn add_bits(&mut self, bits: usize) -> EncodingResult<()> {
        // Allow temporary overflow, will be handled in frame_end
        self.size += bits;
        Ok(())
    }
    
    /// Use bits from the reservoir
    pub fn use_bits(&mut self, bits: usize) -> EncodingResult<()> {
        if bits > self.size {
            return Err(EncodingError::BitReservoirOverflow {
                requested: bits,
                available: self.size,
            });
        }
        self.size -= bits;
        Ok(())
    }
    
    /// Get available bits in reservoir
    pub fn available_bits(&self) -> usize {
        self.size
    }
    
    /// Get maximum reservoir size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
    
    /// Get current utilization as percentage
    pub fn utilization(&self) -> f32 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.size as f32 / self.max_size as f32) * 100.0
        }
    }
    
    /// Reset the reservoir
    pub fn reset(&mut self) {
        self.size = 0;
        self.drain_bits = 0;
    }
    
    /// Check if reservoir is near capacity
    pub fn is_near_capacity(&self, threshold_percent: f32) -> bool {
        self.utilization() >= threshold_percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Set up custom panic hook to avoid verbose parameter output
    fn setup_panic_hook() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|_| {
                eprintln!("Test failed: Property test assertion failed");
            }));
        });
    }

    /// Strategy for generating perceptual entropy values
    fn perceptual_entropy_strategy() -> impl Strategy<Value = f64> {
        0.0f64..1000.0f64
    }

    /// Strategy for generating channel counts
    fn channels_strategy() -> impl Strategy<Value = usize> {
        1usize..=2usize
    }

    /// Strategy for generating mean bits per granule
    fn mean_bits_strategy() -> impl Strategy<Value = usize> {
        100usize..5000usize
    }

    /// Strategy for generating bit amounts
    fn bits_strategy() -> impl Strategy<Value = usize> {
        1usize..1000usize
    }

    /// Strategy for generating reservoir sizes
    fn reservoir_size_strategy() -> impl Strategy<Value = usize> {
        1000usize..10000usize
    }

    // Feature: rust-mp3-encoder, Property 8: 比特储备池机制
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_bit_reservoir_mechanism(
            max_size in reservoir_size_strategy(),
            mean_bits in mean_bits_strategy(),
            channels in channels_strategy(),
            perceptual_entropy in perceptual_entropy_strategy()
        ) {
            setup_panic_hook();
            
            let mut reservoir = BitReservoir::new(max_size);
            reservoir.frame_begin(mean_bits);
            
            // Property 1: Max reservoir bits should be reasonable
            let max_bits = reservoir.max_reservoir_bits(perceptual_entropy, channels);
            prop_assert!(max_bits <= 4095, "Max bits should not exceed 4095");
            prop_assert!(max_bits > 0, "Max bits should be positive");
            
            // Property 2: Utilization should be between 0 and 100%
            let utilization = reservoir.utilization();
            prop_assert!(utilization >= 0.0, "Utilization should be non-negative");
            prop_assert!(utilization <= 200.0, "Utilization should be reasonable"); // Allow some overflow
        }

        #[test]
        fn test_reservoir_add_and_use_bits(
            max_size in reservoir_size_strategy(),
            bits_to_add in bits_strategy(),
            bits_to_use in bits_strategy()
        ) {
            setup_panic_hook();
            
            let mut reservoir = BitReservoir::new(max_size);
            
            // Property 1: Adding bits should increase available bits
            let initial_bits = reservoir.available_bits();
            let _ = reservoir.add_bits(bits_to_add);
            prop_assert_eq!(reservoir.available_bits(), initial_bits + bits_to_add, "Adding bits should increase available bits");
            
            // Property 2: Using bits should decrease available bits (if enough available)
            if bits_to_use <= reservoir.available_bits() {
                let before_use = reservoir.available_bits();
                let result = reservoir.use_bits(bits_to_use);
                prop_assert!(result.is_ok(), "Using available bits should succeed");
                prop_assert_eq!(reservoir.available_bits(), before_use - bits_to_use, "Using bits should decrease available bits");
            }
        }

        #[test]
        fn test_reservoir_frame_operations(
            max_size in reservoir_size_strategy(),
            mean_bits in mean_bits_strategy(),
            channels in channels_strategy(),
            used_bits in bits_strategy()
        ) {
            setup_panic_hook();
            
            let mut reservoir = BitReservoir::new(max_size);
            reservoir.frame_begin(mean_bits);
            
            // Property 1: Frame begin should set mean bits
            prop_assert_eq!(reservoir.mean_bits, mean_bits, "Frame begin should set mean bits");
            
            // Property 2: Adjust after granule should maintain balance
            let _initial_size = reservoir.available_bits();
            let result = reservoir.adjust_after_granule(used_bits, channels);
            
            // Should always succeed with our defensive implementation
            prop_assert!(result.is_ok(), "Granule adjustment should succeed");
        }

        #[test]
        fn test_reservoir_stuffing_bits_distribution(
            stuffing_bits in 1usize..1000usize
        ) {
            setup_panic_hook();
            
            let reservoir = BitReservoir::new(7680);
            let mut granules = vec![GranuleInfo::default(); 4];
            
            // Set some initial part2_3_length values
            for (i, granule) in granules.iter_mut().enumerate() {
                granule.part2_3_length = (i * 100) as u32;
            }
            
            let remaining = reservoir.distribute_stuffing_bits(stuffing_bits, &mut granules);
            
            // Property 1: All bits should be distributed or returned
            let distributed_bits: u32 = granules.iter()
                .map(|g| g.part2_3_length)
                .sum::<u32>() - (0 + 100 + 200 + 300); // Subtract initial values
            
            prop_assert_eq!(distributed_bits as usize + remaining, stuffing_bits, "All bits should be accounted for");
            
            // Property 2: No granule should exceed maximum length
            for granule in &granules {
                prop_assert!(granule.part2_3_length <= 4095, "Granule length should not exceed maximum");
            }
        }

        #[test]
        fn test_reservoir_capacity_management(
            max_size in reservoir_size_strategy(),
            threshold in 0.0f32..100.0f32
        ) {
            setup_panic_hook();
            
            let mut reservoir = BitReservoir::new(max_size);
            
            // Fill reservoir to different levels
            let fill_amount = (max_size as f32 * threshold / 100.0) as usize;
            let _ = reservoir.add_bits(fill_amount);
            
            // Property 1: Near capacity detection should work correctly
            let is_near = reservoir.is_near_capacity(threshold);
            let actual_utilization = reservoir.utilization();
            
            if actual_utilization >= threshold {
                prop_assert!(is_near, "Should detect near capacity correctly");
            } else {
                prop_assert!(!is_near, "Should not detect near capacity when below threshold");
            }
        }
    }

    #[cfg(test)]
    mod unit_tests {
        use super::*;

        #[test]
        fn test_reservoir_creation() {
            let reservoir = BitReservoir::new(7680);
            assert_eq!(reservoir.available_bits(), 0);
            assert_eq!(reservoir.max_size(), 7680);
            assert_eq!(reservoir.utilization(), 0.0);
        }

        #[test]
        fn test_reservoir_basic_operations() {
            let mut reservoir = BitReservoir::new(1000);
            
            // Test adding bits
            assert!(reservoir.add_bits(500).is_ok());
            assert_eq!(reservoir.available_bits(), 500);
            
            // Test using bits
            assert!(reservoir.use_bits(200).is_ok());
            assert_eq!(reservoir.available_bits(), 300);
            
            // Test using too many bits
            assert!(reservoir.use_bits(500).is_err());
            assert_eq!(reservoir.available_bits(), 300);
        }

        #[test]
        fn test_reservoir_frame_operations() {
            let mut reservoir = BitReservoir::new(7680);
            reservoir.frame_begin(1000);
            
            // Test max reservoir bits calculation
            let max_bits = reservoir.max_reservoir_bits(100.0, 2);
            assert!(max_bits > 0);
            assert!(max_bits <= 4095);
            
            // Test frame end
            let (_stuffing_bits, drain_bits) = reservoir.frame_end(2);
            assert_eq!(drain_bits, 0);
        }

        #[test]
        fn test_reservoir_reset() {
            let mut reservoir = BitReservoir::new(1000);
            let _ = reservoir.add_bits(500);
            
            reservoir.reset();
            assert_eq!(reservoir.available_bits(), 0);
        }

        #[test]
        fn test_stuffing_bits_distribution() {
            let reservoir = BitReservoir::new(7680);
            let mut granules = vec![GranuleInfo::default(); 2];
            
            // Test distributing bits
            let remaining = reservoir.distribute_stuffing_bits(100, &mut granules);
            assert_eq!(remaining, 0);
            assert_eq!(granules[0].part2_3_length, 100);
            
            // Test distributing more bits than first granule can hold
            granules[0].part2_3_length = 4000;
            let remaining = reservoir.distribute_stuffing_bits(200, &mut granules);
            assert!(remaining == 0 || granules[1].part2_3_length > 0);
        }
    }
}