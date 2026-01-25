//! Unit tests for types and constants
//!
//! Tests the type definitions, constants, and utility functions
//! to ensure they match the shine reference implementation.

use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_constants_match_shine() {
        // Verify that constants match shine's values exactly
        assert_eq!(GRANULE_SIZE, 576, "Granule size should match shine");
        assert_eq!(MAX_CHANNELS, 2, "Max channels should match shine");
        assert_eq!(MAX_GRANULES, 2, "Max granules should match shine");
        assert_eq!(SBLIMIT, 32, "Subband limit should match shine");
        assert_eq!(HAN_SIZE, 512, "HAN size should match shine");
        assert_eq!(BLKSIZE, 1024, "Block size should match shine");
        assert_eq!(SCALE, 32768, "Scale factor should match shine");
        
        // Verify mathematical constants
        assert!((PI - 3.14159265358979).abs() < 1e-15, "PI should be accurate");
        assert!((SQRT2 - 1.41421356237).abs() < 1e-10, "SQRT2 should be accurate");
        assert!((LN2 - 0.69314718).abs() < 1e-8, "LN2 should be accurate");
    }

    #[test]
    fn test_swab32_function() {
        // Test byte swapping function (matches shine's SWAB32)
        assert_eq!(swab32(0x12345678), 0x78563412, "Should swap bytes correctly");
        assert_eq!(swab32(0x00000000), 0x00000000, "Zero should remain zero");
        assert_eq!(swab32(0xFFFFFFFF), 0xFFFFFFFF, "All ones should remain all ones");
        assert_eq!(swab32(0x12000000), 0x00000012, "Should handle partial bytes");
    }

    #[test]
    fn test_structure_sizes() {
        // Verify that structures have reasonable sizes
        // These tests ensure we don't accidentally create oversized structures
        
        println!("GrInfo size: {}", mem::size_of::<GrInfo>());
        println!("ShineSideInfo size: {}", mem::size_of::<ShineSideInfo>());
        println!("L3Loop size: {}", mem::size_of::<L3Loop>());
        println!("ShineGlobalConfig size: {}", mem::size_of::<ShineGlobalConfig>());
        
        // Basic sanity checks - structures shouldn't be empty
        assert!(mem::size_of::<GrInfo>() > 0, "GrInfo should have non-zero size");
        assert!(mem::size_of::<ShineSideInfo>() > 0, "ShineSideInfo should have non-zero size");
        assert!(mem::size_of::<L3Loop>() > 0, "L3Loop should have non-zero size");
        assert!(mem::size_of::<ShineGlobalConfig>() > 0, "ShineGlobalConfig should have non-zero size");
        
        // Verify structures aren't unreasonably large
        assert!(mem::size_of::<ShineGlobalConfig>() < 1024 * 1024, "Config should not be too large");
    }

    #[test]
    fn test_default_values() {
        let config = Box::new(ShineGlobalConfig::default());
        
        // Verify default values match shine's expectations
        assert_eq!(config.wave.channels, 2, "Default channels should be stereo");
        assert_eq!(config.wave.samplerate, 44100, "Default sample rate should be 44.1kHz");
        assert_eq!(config.mpeg.version, 1, "Default MPEG version");
        assert_eq!(config.mpeg.layer, 1, "Default layer should be III");
        assert_eq!(config.mpeg.granules_per_frame, 2, "Default granules per frame");
        assert_eq!(config.mpeg.bitr, 128, "Default bitrate should be 128 kbps");
        assert_eq!(config.mpeg.bits_per_slot, 8, "Default bits per slot");
        assert_eq!(config.mpeg.bitrate_index, 9, "Default bitrate index for 128 kbps");
        assert_eq!(config.mpeg.samplerate_index, 0, "Default sample rate index for 44.1kHz");
        assert_eq!(config.mpeg.original, 1, "Default original flag");
        
        let gr_info = GrInfo::default();
        assert_eq!(gr_info.global_gain, 210, "Default global gain should match shine");
        assert_eq!(gr_info.sfb_lmax, 21, "Default sfb_lmax should match shine");
    }

    #[test]
    fn test_array_bounds() {
        // Test that array indices are within expected bounds
        let config = Box::new(ShineGlobalConfig::default());
        
        // Verify array dimensions match shine's expectations
        assert_eq!(config.l3_enc.len(), MAX_CHANNELS, "L3 enc should have MAX_CHANNELS");
        assert_eq!(config.l3_enc[0].len(), MAX_GRANULES, "L3 enc should have MAX_GRANULES");
        assert_eq!(config.l3_enc[0][0].len(), GRANULE_SIZE, "L3 enc should have GRANULE_SIZE");
        
        assert_eq!(config.mdct_freq.len(), MAX_CHANNELS, "MDCT freq should have MAX_CHANNELS");
        assert_eq!(config.mdct_freq[0].len(), MAX_GRANULES, "MDCT freq should have MAX_GRANULES");
        assert_eq!(config.mdct_freq[0][0].len(), GRANULE_SIZE, "MDCT freq should have GRANULE_SIZE");
        
        assert_eq!(config.scalefactor.l.len(), MAX_GRANULES, "Scalefactor should have MAX_GRANULES");
        assert_eq!(config.scalefactor.l[0].len(), MAX_CHANNELS, "Scalefactor should have MAX_CHANNELS");
        assert_eq!(config.scalefactor.l[0][0].len(), 22, "Scalefactor should have 22 bands");
    }

    #[test]
    fn test_granule_info_structure() {
        let gr_info = GrInfo::default();
        
        // Test that GrInfo has expected fields and ranges
        assert!(gr_info.global_gain <= 255, "Global gain should fit in 8 bits");
        assert!(gr_info.sfb_lmax <= 22, "SFB lmax should be within valid range");
        assert_eq!(gr_info.table_select.len(), 3, "Should have 3 table select values");
        assert_eq!(gr_info.slen.len(), 4, "Should have 4 slen values");
        
        // Test that arrays are properly sized
        for &table_sel in &gr_info.table_select {
            assert!(table_sel <= 31, "Table select should be 5-bit value");
        }
    }

    #[test]
    fn test_side_info_structure() {
        let side_info = ShineSideInfo::default();
        
        // Test SCFSI structure
        assert_eq!(side_info.scfsi.len(), MAX_CHANNELS, "SCFSI should have MAX_CHANNELS");
        assert_eq!(side_info.scfsi[0].len(), 4, "SCFSI should have 4 bands");
        
        // Test granule structure
        assert_eq!(side_info.gr.len(), MAX_GRANULES, "Should have MAX_GRANULES");
        for gr in 0..MAX_GRANULES {
            assert_eq!(side_info.gr[gr].ch.len(), MAX_CHANNELS, "Each granule should have MAX_CHANNELS");
        }
        
        // Test initial SCFSI values
        for ch in 0..MAX_CHANNELS {
            for band in 0..4 {
                let scfsi_val = side_info.scfsi[ch][band];
                assert!(scfsi_val == 0 || scfsi_val == 1, "SCFSI should be binary");
            }
        }
    }

    #[test]
    fn test_subband_structure() {
        let subband = Subband::default();
        
        // Test offset array
        assert_eq!(subband.off.len(), MAX_CHANNELS, "Offset should have MAX_CHANNELS");
        for &offset in &subband.off {
            assert_eq!(offset, 0, "Initial offset should be zero");
        }
        
        // Test filter coefficient array
        assert_eq!(subband.fl.len(), SBLIMIT, "FL should have SBLIMIT entries");
        assert_eq!(subband.fl[0].len(), 64, "Each FL entry should have 64 coefficients");
        
        // Test filter buffer
        assert_eq!(subband.x.len(), MAX_CHANNELS, "X buffer should have MAX_CHANNELS");
        assert_eq!(subband.x[0].len(), HAN_SIZE, "Each X buffer should have HAN_SIZE");
    }

    #[test]
    fn test_mp3_standard_compliance() {
        // Test that our constants comply with MP3 standard limits
        
        // Granule size must be 576 for Layer III
        assert_eq!(GRANULE_SIZE, 576, "MP3 Layer III requires 576 samples per granule");
        
        // Maximum 2 channels for stereo
        assert_eq!(MAX_CHANNELS, 2, "MP3 supports maximum 2 channels");
        
        // 32 subbands for Layer III
        assert_eq!(SBLIMIT, 32, "MP3 Layer III uses 32 subbands");
        
        // Test that big_values limit is enforced
        let gr_info = GrInfo::default();
        // big_values should not exceed GRANULE_SIZE / 2
        assert!(gr_info.big_values <= (GRANULE_SIZE / 2) as u32, "big_values should not exceed granule limit");
    }
}