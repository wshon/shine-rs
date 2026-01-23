//! Data structure consistency test
//!
//! This test validates that our Rust data structures maintain
//! functional consistency with shine's C structures.

use rust_mp3_encoder::quantization::GranuleInfo;
use rust_mp3_encoder::shine_config::{ShineSideInfo, L3Loop, ShineGlobalConfig, MAX_CHANNELS, MAX_GRANULES, GRANULE_SIZE};
use rust_mp3_encoder::config::Config;
use std::mem;

fn main() {
    println!("=== Data Structure Consistency Test ===\n");
    
    // Test 1: GranuleInfo default values
    test_granule_info_defaults();
    
    // Test 2: ShineSideInfo structure
    test_shine_side_info();
    
    // Test 3: L3Loop table initialization
    test_l3loop_initialization();
    
    // Test 4: ShineGlobalConfig creation and initialization
    test_shine_global_config();
    
    // Test 5: Memory layout validation
    test_memory_layout();
    
    println!("\n=== All Tests Passed ===");
}

fn test_granule_info_defaults() {
    println!("Test 1: GranuleInfo default values");
    
    let gi = GranuleInfo::default();
    
    // Verify default values match shine's expectations
    assert_eq!(gi.part2_3_length, 0);
    assert_eq!(gi.big_values, 0);
    assert_eq!(gi.count1, 0);
    assert_eq!(gi.global_gain, 210); // Default global gain in shine
    assert_eq!(gi.scalefac_compress, 0);
    assert_eq!(gi.table_select, [1, 1, 1]); // Default to table 1 (table 0 doesn't exist)
    assert_eq!(gi.region0_count, 0);
    assert_eq!(gi.region1_count, 0);
    assert_eq!(gi.preflag, 0);
    assert_eq!(gi.scalefac_scale, 0);
    assert_eq!(gi.count1table_select, 0);
    assert_eq!(gi.part2_length, 0);
    assert_eq!(gi.sfb_lmax, 20); // SFB_LMAX - 1 = 21 - 1 = 20
    assert_eq!(gi.address1, 0);
    assert_eq!(gi.address2, 0);
    assert_eq!(gi.address3, 0);
    assert_eq!(gi.quantizer_step_size, 0);
    assert_eq!(gi.slen, [0, 0, 0, 0]);
    
    println!("  ✓ GranuleInfo defaults are correct");
}

fn test_shine_side_info() {
    println!("Test 2: ShineSideInfo structure");
    
    let side_info = ShineSideInfo::default();
    
    // Verify structure layout
    assert_eq!(side_info.private_bits, 0);
    assert_eq!(side_info.resv_drain, 0);
    assert_eq!(side_info.scfsi.len(), MAX_CHANNELS);
    assert_eq!(side_info.scfsi[0].len(), 4);
    assert_eq!(side_info.gr.len(), MAX_GRANULES);
    assert_eq!(side_info.gr[0].len(), MAX_CHANNELS);
    
    println!("  ✓ ShineSideInfo structure is correct");
}

fn test_l3loop_initialization() {
    println!("Test 3: L3Loop table initialization");
    
    let config = Config::default();
    let mut shine_config = ShineGlobalConfig::new(config).expect("Failed to create ShineGlobalConfig");
    shine_config.initialize().expect("Failed to initialize ShineGlobalConfig");
    
    // Verify L3Loop tables are initialized correctly
    let l3loop = &shine_config.l3loop;
    
    // Check steptab table (quantization step sizes)
    // steptab[i] = 2^((127-i)/4) for i = 0..127
    assert!(l3loop.steptab[0] > 0.0, "steptab[0] should be positive");
    assert!(l3loop.steptab[127] > 0.0, "steptab[127] should be positive");
    assert!(l3loop.steptab[0] > l3loop.steptab[127], "steptab should be decreasing");
    
    // Check steptabi table (integer version)
    assert!(l3loop.steptabi[0] > 0, "steptabi[0] should be positive");
    assert!(l3loop.steptabi[127] > 0, "steptabi[127] should be positive");
    assert!(l3loop.steptabi[0] > l3loop.steptabi[127], "steptabi should be decreasing");
    
    // Check int2idx table (x^(3/4) lookup)
    assert_eq!(l3loop.int2idx[0], 0, "int2idx[0] should be 0");
    assert!(l3loop.int2idx[9999] > 0, "int2idx[9999] should be positive");
    
    // Verify the relationship: int2idx should be increasing
    for i in 1..100 {
        assert!(l3loop.int2idx[i] >= l3loop.int2idx[i-1], 
                "int2idx should be non-decreasing at index {}", i);
    }
    
    println!("  ✓ L3Loop tables initialized correctly");
}

fn test_shine_global_config() {
    println!("Test 4: ShineGlobalConfig creation and initialization");
    
    let config = Config::default();
    let mut shine_config = ShineGlobalConfig::new(config).expect("Failed to create ShineGlobalConfig");
    
    // Verify initial state
    assert_eq!(shine_config.wave.channels, 2); // Default stereo
    assert_eq!(shine_config.wave.sample_rate, 44100); // Default sample rate
    assert_eq!(shine_config.mpeg.bitrate, 128); // Default bitrate
    
    // Initialize and verify
    shine_config.initialize().expect("Failed to initialize ShineGlobalConfig");
    
    // Verify MDCT tables are initialized
    assert_ne!(shine_config.mdct.cos_l[0][0], 0, "MDCT cos_l table should be initialized");
    assert_ne!(shine_config.mdct.cos_l[17][35], 0, "MDCT cos_l table should be initialized");
    
    // Verify subband filter is initialized
    assert_ne!(shine_config.subband.fl[0][0], 0, "Subband filter should be initialized");
    assert_ne!(shine_config.subband.fl[31][63], 0, "Subband filter should be initialized");
    
    println!("  ✓ ShineGlobalConfig creation and initialization successful");
}

fn test_memory_layout() {
    println!("Test 5: Memory layout validation");
    
    // Verify structure sizes are reasonable
    let gi_size = mem::size_of::<GranuleInfo>();
    let side_info_size = mem::size_of::<ShineSideInfo>();
    let l3loop_size = mem::size_of::<L3Loop>();
    
    println!("  Structure sizes:");
    println!("    GranuleInfo: {} bytes", gi_size);
    println!("    ShineSideInfo: {} bytes", side_info_size);
    println!("    L3Loop: {} bytes", l3loop_size);
    
    // Verify sizes are reasonable (not too small or too large)
    assert!(gi_size > 50 && gi_size < 200, "GranuleInfo size should be reasonable");
    assert!(side_info_size > 100 && side_info_size < 1000, "ShineSideInfo size should be reasonable");
    assert!(l3loop_size > 10000 && l3loop_size < 100000, "L3Loop size should be reasonable");
    
    // Verify alignment
    assert_eq!(mem::align_of::<GranuleInfo>(), 4, "GranuleInfo should be 4-byte aligned");
    assert_eq!(mem::align_of::<ShineSideInfo>(), 4, "ShineSideInfo should be 4-byte aligned");
    
    println!("  ✓ Memory layout validation passed");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_granule_info_field_access() {
        let mut gi = GranuleInfo::default();
        
        // Test field access and modification
        gi.big_values = 100;
        gi.global_gain = 200;
        gi.quantizer_step_size = -10;
        
        assert_eq!(gi.big_values, 100);
        assert_eq!(gi.global_gain, 200);
        assert_eq!(gi.quantizer_step_size, -10);
    }
    
    #[test]
    fn test_shine_side_info_field_access() {
        let mut side_info = ShineSideInfo::default();
        
        // Test field access and modification
        side_info.private_bits = 0x123;
        side_info.resv_drain = -50;
        side_info.scfsi[0][0] = 1;
        
        assert_eq!(side_info.private_bits, 0x123);
        assert_eq!(side_info.resv_drain, -50);
        assert_eq!(side_info.scfsi[0][0], 1);
    }
    
    #[test]
    fn test_constants_consistency() {
        // Verify constants match shine's definitions
        assert_eq!(GRANULE_SIZE, 576);
        assert_eq!(MAX_CHANNELS, 2);
        assert_eq!(MAX_GRANULES, 2);
    }
}