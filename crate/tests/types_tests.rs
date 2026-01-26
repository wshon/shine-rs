//! Unit tests for types and constants
//!
//! Tests the type definitions, constants, and utility functions
//! to ensure they match the shine reference implementation.

use shine_rs::types::*;

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
        
        // Verify mathematical constants (now using std constants)
        assert!((PI - std::f64::consts::PI).abs() < 1e-15, "PI should match std::f64::consts::PI");
        assert!((SQRT2 - std::f64::consts::SQRT_2).abs() < 1e-15, "SQRT2 should match std::f64::consts::SQRT_2");
        assert!((LN2 - std::f64::consts::LN_2).abs() < 1e-15, "LN2 should match std::f64::consts::LN_2");
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
        
        log::debug!("GrInfo size: {}", mem::size_of::<GrInfo>());
        log::debug!("ShineSideInfo size: {}", mem::size_of::<ShineSideInfo>());
        log::debug!("L3Loop size: {}", mem::size_of::<L3Loop>());
        log::debug!("ShineGlobalConfig size: {}", mem::size_of::<ShineGlobalConfig>());
        
        // Basic sanity checks - structures shouldn't be empty
        assert!(mem::size_of::<GrInfo>() > 0, "GrInfo should have non-zero size");
        assert!(mem::size_of::<ShineSideInfo>() > 0, "ShineSideInfo should have non-zero size");
        assert!(mem::size_of::<L3Loop>() > 0, "L3Loop should have non-zero size");
        assert!(mem::size_of::<ShineGlobalConfig>() > 0, "ShineGlobalConfig should have non-zero size");
        
        // Verify structures aren't unreasonably large
        assert!(mem::size_of::<ShineGlobalConfig>() < 1024 * 1024, "Config should not be too large");
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