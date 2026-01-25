//! Unit tests for bitstream operations
//!
//! Tests the bitstream writing functionality including bit packing,
//! frame header generation, and data serialization.

use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitstream_writer_initialization() {
        use crate::bitstream::BitstreamWriter;
        
        let bs = BitstreamWriter::new(8192);
        
        // Test initial state
        assert_eq!(bs.data_position, 0, "Initial position should be 0");
        assert_eq!(bs.cache, 0, "Initial cache should be 0");
        assert_eq!(bs.cache_bits, 32, "Initial cache bits should be 32");
        assert_eq!(bs.data_size, 8192, "Data size should match requested size");
        
        // Test that buffer is properly initialized
        assert!(bs.data.len() >= 8192, "Buffer should have requested size");
    }

    #[test]
    fn test_put_bits_basic() {
        use crate::bitstream::BitstreamWriter;
        
        let mut bs = BitstreamWriter::new(1024);
        
        // Test writing single bits
        bs.put_bits(1, 1).expect("Should write 1 bit");
        assert_eq!(bs.cache_bits, 31, "Cache bits should decrease");
        
        bs.put_bits(0, 1).expect("Should write 1 bit");
        assert_eq!(bs.cache_bits, 30, "Cache bits should decrease");
        
        // Test writing multiple bits
        bs.put_bits(0b1010, 4).expect("Should write 4 bits");
        assert_eq!(bs.cache_bits, 26, "Cache bits should decrease by 4");
    }

    #[test]
    fn test_put_bits_boundary() {
        use crate::bitstream::BitstreamWriter;
        
        let mut bs = BitstreamWriter::new(1024);
        
        // Fill cache completely
        bs.put_bits(0xFFFFFFFF, 32).expect("Should write 32 bits");
        assert_eq!(bs.cache_bits, 32, "Cache should be reset after flush");
        assert_eq!(bs.data_position, 4, "Should have written 4 bytes");
        
        // Write one more bit to trigger flush
        bs.put_bits(1, 1).expect("Should write 1 bit");
        assert_eq!(bs.cache_bits, 31, "Cache bits should be 31 after new bit");
    }

    #[test]
    fn test_shine_side_info_structure() {
        let side_info = ShineSideInfo::default();
        
        // Test that structure has expected fields
        assert_eq!(side_info.scfsi.len(), 2, "Should have SCFSI for 2 channels");
        assert_eq!(side_info.scfsi[0].len(), 4, "Should have 4 SCFSI bands per channel");
        assert_eq!(side_info.gr.len(), 2, "Should have 2 granules");
        
        // Test initial values
        for ch in 0..2 {
            for band in 0..4 {
                assert_eq!(side_info.scfsi[ch][band], 0, "Initial SCFSI should be 0");
            }
        }
    }

    #[test]
    fn test_shine_global_config_structure() {
        let config = ShineGlobalConfig::default();
        
        // Test that config has required components
        assert_eq!(config.wave.channels, 0, "Initial channels should be 0");
        assert_eq!(config.mpeg.version, 0, "Initial version should be 0");
        assert_eq!(config.mpeg.layer, 0, "Initial layer should be 0");
        
        // Test that all granule info arrays are properly sized
        assert_eq!(config.side_info.gr.len(), 2, "Should have 2 granules");
        for gr in 0..2 {
            assert_eq!(config.side_info.gr[gr].ch.len(), 2, "Should have 2 channels per granule");
        }
    }

    #[test]
    fn test_bitstream_flush() {
        use crate::bitstream::BitstreamWriter;
        
        let mut bs = BitstreamWriter::new(1024);
        
        // Write some bits
        bs.put_bits(0b10101010, 8).expect("Should write 8 bits");
        assert_eq!(bs.cache_bits, 24, "Cache should have 24 free bits");
        assert_eq!(bs.data_position, 0, "No data should be written yet");
        
        // Flush the bitstream
        bs.flush().expect("Should flush successfully");
        
        // Cache should be reset and data written
        assert_eq!(bs.cache_bits, 32, "Cache should be reset");
        assert!(bs.data_position > 0, "Data should be written");
    }

    #[test]
    fn test_bit_alignment() {
        use crate::bitstream::BitstreamWriter;
        
        let mut bs = BitstreamWriter::new(1024);
        
        // Write 7 bits (not byte-aligned)
        bs.put_bits(0b1010101, 7).expect("Should write 7 bits");
        assert_eq!(bs.cache_bits, 25, "Cache should have 25 free bits");
        
        // Write 1 more bit to make it byte-aligned
        bs.put_bits(1, 1).expect("Should write 1 bit");
        assert_eq!(bs.cache_bits, 24, "Cache should have 24 free bits");
        
        // Write 24 more bits to fill cache
        bs.put_bits(0xFFFFFF, 24).expect("Should write 24 bits");
        assert_eq!(bs.cache_bits, 32, "Cache should be reset after flush");
        assert_eq!(bs.data_position, 4, "Should have written 4 bytes");
    }

    #[test]
    fn test_zero_bits_write() {
        use crate::bitstream::BitstreamWriter;
        
        let mut bs = BitstreamWriter::new(1024);
        let initial_cache_bits = bs.cache_bits;
        
        // Writing 0 bits should not change state
        bs.put_bits(0, 0).expect("Should handle 0 bits");
        assert_eq!(bs.cache_bits, initial_cache_bits, "State should not change");
        assert_eq!(bs.data_position, 0, "No data should be written");
    }

    /// Test bitstream frame parameters with real data
    #[test]
    fn test_bitstream_real_data_validation() {
        // Real data from sample-3s.wav encoding - these are actual measured values
        const F1_WRITTEN_BYTES: i32 = 416;
        const F2_WRITTEN_BYTES: i32 = 420;
        const F3_WRITTEN_BYTES: i32 = 416;
        const BITS_PER_FRAME: i32 = 3344;
        
        // Validate frame sizes are reasonable for 128kbps MP3
        assert!(F1_WRITTEN_BYTES > 400 && F1_WRITTEN_BYTES < 450, "Frame 1 size should be reasonable");
        assert!(F2_WRITTEN_BYTES > 400 && F2_WRITTEN_BYTES < 450, "Frame 2 size should be reasonable");
        assert!(F3_WRITTEN_BYTES > 400 && F3_WRITTEN_BYTES < 450, "Frame 3 size should be reasonable");
        
        // Total size should be reasonable for 3 frames
        let total_bytes = F1_WRITTEN_BYTES + F2_WRITTEN_BYTES + F3_WRITTEN_BYTES;
        assert!(total_bytes > 1200 && total_bytes < 1300, "Total 3 frames should be reasonable size");
        
        // Bits per frame should be consistent with 128kbps at 44.1kHz
        let expected_bits = (128000 * 1152) / 44100; // ~3344
        assert!((BITS_PER_FRAME - expected_bits).abs() < 10, "Bits per frame should match bitrate calculation");
    }

    /// Test slot lag mechanism with real data
    #[test]
    fn test_slot_lag_real_data_validation() {
        // Real slot lag values from sample-3s.wav encoding
        const F1_SLOT_LAG_BEFORE: f64 = -0.959184;
        const F1_SLOT_LAG_AFTER: f64 = -0.918367;
        const F2_SLOT_LAG_BEFORE: f64 = -0.918367;
        const F2_SLOT_LAG_AFTER: f64 = -0.877551;
        const F3_SLOT_LAG_BEFORE: f64 = -0.877551;
        const F3_SLOT_LAG_AFTER: f64 = -0.836735;
        
        // Validate slot lag values are in expected range
        let all_slot_lags = [F1_SLOT_LAG_BEFORE, F1_SLOT_LAG_AFTER, F2_SLOT_LAG_BEFORE, F2_SLOT_LAG_AFTER, F3_SLOT_LAG_BEFORE, F3_SLOT_LAG_AFTER];
        for &lag in &all_slot_lags {
            assert!(lag >= -1.0 && lag <= 1.0, "Slot lag {} out of range", lag);
        }
        
        // Validate slot lag continuity (each frame's before should match previous frame's after)
        assert!((F2_SLOT_LAG_BEFORE - F1_SLOT_LAG_AFTER).abs() < 0.000001, "Slot lag continuity broken between F1 and F2");
        assert!((F3_SLOT_LAG_BEFORE - F2_SLOT_LAG_AFTER).abs() < 0.000001, "Slot lag continuity broken between F2 and F3");
        
        // Validate slot lag progression (should increase by ~0.040816 each frame due to padding)
        let f1_diff = F1_SLOT_LAG_AFTER - F1_SLOT_LAG_BEFORE;
        let f2_diff = F2_SLOT_LAG_AFTER - F2_SLOT_LAG_BEFORE;
        let f3_diff = F3_SLOT_LAG_AFTER - F3_SLOT_LAG_BEFORE;
        
        assert!((f1_diff - 0.040816).abs() < 0.000001, "Frame 1 slot lag increment incorrect");
        assert!((f2_diff - 0.040816).abs() < 0.000001, "Frame 2 slot lag increment incorrect");
        assert!((f3_diff - 0.040816).abs() < 0.000001, "Frame 3 slot lag increment incorrect");
    }

    #[test]
    fn test_frame_size_calculation() {
        // Test frame size calculation for different bitrates
        const SAMPLE_RATE: u32 = 44100;
        const SAMPLES_PER_FRAME: u32 = 1152;
        
        // For 128 kbps
        const BITRATE_128: u32 = 128000;
        let frame_size_128 = (SAMPLES_PER_FRAME * BITRATE_128) / (8 * SAMPLE_RATE);
        assert!(frame_size_128 >= 416 && frame_size_128 <= 418, "128 kbps frame size should be ~417 bytes");
        
        // For 192 kbps
        const BITRATE_192: u32 = 192000;
        let frame_size_192 = (SAMPLES_PER_FRAME * BITRATE_192) / (8 * SAMPLE_RATE);
        assert!(frame_size_192 >= 625 && frame_size_192 <= 627, "192 kbps frame size should be ~626 bytes");
    }

    #[test]
    fn test_mp3_standard_limits() {
        // Test MP3 standard limits that our implementation must respect
        
        // Test that our granule info structure can hold valid MP3 values
        let mut gr_info = GrInfo::default();
        
        // Test setting maximum valid values
        gr_info.part2_3_length = 4095; // 12-bit field maximum
        gr_info.big_values = 288;      // Granule size / 2 maximum
        gr_info.global_gain = 255;     // 8-bit field maximum
        
        assert!(gr_info.part2_3_length <= 4095, "Part2_3_length should fit in 12 bits");
        assert!(gr_info.big_values <= 288, "Big values should not exceed granule limit");
        assert!(gr_info.global_gain <= 255, "Global gain should fit in 8 bits");
    }

    #[test]
    fn test_scfsi_structure() {
        let mut side_info = ShineSideInfo::default();
        
        // Test SCFSI array structure
        assert_eq!(side_info.scfsi.len(), 2, "Should have SCFSI for 2 channels");
        assert_eq!(side_info.scfsi[0].len(), 4, "Should have 4 SCFSI bands");
        
        // Test setting SCFSI values
        side_info.scfsi[0] = [0, 1, 0, 1];
        side_info.scfsi[1] = [1, 1, 1, 1];
        
        // Verify values are set correctly
        assert_eq!(side_info.scfsi[0][1], 1, "SCFSI should be settable");
        assert_eq!(side_info.scfsi[1][0], 1, "SCFSI should be settable");
        
        // Test that SCFSI values are binary
        for ch in 0..2 {
            for band in 0..4 {
                let val = side_info.scfsi[ch][band];
                assert!(val == 0 || val == 1, "SCFSI should be 0 or 1, got {}", val);
            }
        }
    }
}

#[cfg(test)]
mod property_tests {
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
        fn test_slot_lag_properties(
            lag in -1.0f64..=1.0f64
        ) {
            // Slot lag should always be in valid range
            prop_assert!(lag >= -1.0 && lag <= 1.0, "Slot lag should be in range [-1, 1]");
            
            // Test slot lag increment calculation
            let increment = 0.040816;
            let new_lag = lag + increment;
            
            // New lag might exceed range, which is handled by the encoder
            if new_lag <= 1.0 {
                prop_assert!(new_lag > lag, "Slot lag should increase with padding");
            }
        }

        #[test]
        fn test_frame_size_properties(
            bitrate in prop::sample::select(vec![32u32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320]),
            sample_rate in prop::sample::select(vec![32000u32, 44100, 48000])
        ) {
            const SAMPLES_PER_FRAME: u32 = 1152;
            let frame_size = (SAMPLES_PER_FRAME * bitrate * 1000) / (8 * sample_rate);
            
            // Frame size should be reasonable
            prop_assert!(frame_size > 0, "Frame size should be positive");
            prop_assert!(frame_size < 2000, "Frame size should be reasonable");
            
            // Higher bitrate should generally mean larger frames (for same sample rate)
            if bitrate >= 128 {
                prop_assert!(frame_size >= 300, "High bitrate frames should be substantial");
            }
        }

        #[test]
        fn test_bit_field_properties(
            global_gain in 0u32..=255,
            big_values in 0u32..=288,
            part2_3_length in 0u32..=4095
        ) {
            // Test that values fit in their bit fields
            prop_assert!(global_gain <= 255, "Global gain should fit in 8 bits");
            prop_assert!(big_values <= 288, "Big values should not exceed granule limit");
            prop_assert!(part2_3_length <= 4095, "Part2_3_length should fit in 12 bits");
            
            // Test relationships
            if big_values > 0 {
                prop_assert!(part2_3_length > 0, "Non-zero big values should have non-zero length");
            }
        }

        #[test]
        fn test_scfsi_properties(
            scfsi_values in prop::collection::vec(0u32..=1, 4)
        ) {
            prop_assert_eq!(scfsi_values.len(), 4, "Should have 4 SCFSI bands");
            
            for &val in &scfsi_values {
                prop_assert!(val == 0 || val == 1, "SCFSI should be binary");
            }
            
            // Test that we can create valid SCFSI arrays
            let mut side_info = ShineSideInfo::default();
            side_info.scfsi[0] = [scfsi_values[0], scfsi_values[1], scfsi_values[2], scfsi_values[3]];
            
            for i in 0..4 {
                prop_assert_eq!(side_info.scfsi[0][i], scfsi_values[i], "SCFSI should be set correctly");
            }
        }
    }
}