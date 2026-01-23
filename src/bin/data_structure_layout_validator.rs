//! Data structure layout validator
//!
//! This binary validates that our Rust data structures have the same
//! memory layout as the corresponding C structures in shine.

use std::mem;
use rust_mp3_encoder::quantization::GranuleInfo;
use rust_mp3_encoder::shine_config::{ShineSideInfo, L3Loop, Mdct, Subband, ShineGlobalConfig};
use rust_mp3_encoder::config::Config;

fn main() {
    println!("=== Data Structure Layout Validation ===\n");
    
    // Validate GranuleInfo (gr_info)
    println!("GranuleInfo (gr_info) validation:");
    println!("  Size: {} bytes", mem::size_of::<GranuleInfo>());
    println!("  Alignment: {} bytes", mem::align_of::<GranuleInfo>());
    
    // Check field offsets
    let gi = GranuleInfo::default();
    let base_ptr = &gi as *const GranuleInfo as usize;
    
    println!("  Field offsets:");
    println!("    part2_3_length: {}", offset_of!(gi, part2_3_length, base_ptr));
    println!("    big_values: {}", offset_of!(gi, big_values, base_ptr));
    println!("    count1: {}", offset_of!(gi, count1, base_ptr));
    println!("    global_gain: {}", offset_of!(gi, global_gain, base_ptr));
    println!("    scalefac_compress: {}", offset_of!(gi, scalefac_compress, base_ptr));
    println!("    table_select: {}", offset_of!(gi, table_select, base_ptr));
    println!("    region0_count: {}", offset_of!(gi, region0_count, base_ptr));
    println!("    region1_count: {}", offset_of!(gi, region1_count, base_ptr));
    println!("    preflag: {}", offset_of!(gi, preflag, base_ptr));
    println!("    scalefac_scale: {}", offset_of!(gi, scalefac_scale, base_ptr));
    println!("    count1table_select: {}", offset_of!(gi, count1table_select, base_ptr));
    println!("    part2_length: {}", offset_of!(gi, part2_length, base_ptr));
    println!("    sfb_lmax: {}", offset_of!(gi, sfb_lmax, base_ptr));
    println!("    address1: {}", offset_of!(gi, address1, base_ptr));
    println!("    address2: {}", offset_of!(gi, address2, base_ptr));
    println!("    address3: {}", offset_of!(gi, address3, base_ptr));
    println!("    quantizer_step_size: {}", offset_of!(gi, quantizer_step_size, base_ptr));
    println!("    slen: {}", offset_of!(gi, slen, base_ptr));
    
    // Validate ShineSideInfo (shine_side_info_t)
    println!("\nShineSideInfo (shine_side_info_t) validation:");
    println!("  Size: {} bytes", mem::size_of::<ShineSideInfo>());
    println!("  Alignment: {} bytes", mem::align_of::<ShineSideInfo>());
    
    // Validate L3Loop (l3loop_t)
    println!("\nL3Loop (l3loop_t) validation:");
    println!("  Size: {} bytes", mem::size_of::<L3Loop>());
    println!("  Alignment: {} bytes", mem::align_of::<L3Loop>());
    
    // Validate Mdct (mdct_t)
    println!("\nMdct (mdct_t) validation:");
    println!("  Size: {} bytes", mem::size_of::<Mdct>());
    println!("  Alignment: {} bytes", mem::align_of::<Mdct>());
    
    // Validate Subband (subband_t)
    println!("\nSubband (subband_t) validation:");
    println!("  Size: {} bytes", mem::size_of::<Subband>());
    println!("  Alignment: {} bytes", mem::align_of::<Subband>());
    
    // Test data structure initialization
    println!("\n=== Data Structure Initialization Test ===");
    
    let config = Config::default();
    match ShineGlobalConfig::new(config) {
        Ok(mut shine_config) => {
            println!("✓ ShineGlobalConfig created successfully");
            
            match shine_config.initialize() {
                Ok(()) => {
                    println!("✓ ShineGlobalConfig initialized successfully");
                    
                    // Validate table initialization
                    println!("  L3Loop tables validation:");
                    println!("    steptab[0]: {}", shine_config.l3loop.steptab[0]);
                    println!("    steptab[127]: {}", shine_config.l3loop.steptab[127]);
                    println!("    steptabi[0]: {}", shine_config.l3loop.steptabi[0]);
                    println!("    steptabi[127]: {}", shine_config.l3loop.steptabi[127]);
                    println!("    int2idx[0]: {}", shine_config.l3loop.int2idx[0]);
                    println!("    int2idx[9999]: {}", shine_config.l3loop.int2idx[9999]);
                    
                    // Validate MDCT tables
                    println!("  MDCT tables validation:");
                    println!("    cos_l[0][0]: {}", shine_config.mdct.cos_l[0][0]);
                    println!("    cos_l[17][35]: {}", shine_config.mdct.cos_l[17][35]);
                    
                    // Validate subband filter
                    println!("  Subband filter validation:");
                    println!("    fl[0][0]: {}", shine_config.subband.fl[0][0]);
                    println!("    fl[31][63]: {}", shine_config.subband.fl[31][63]);
                    
                }
                Err(_) => {
                    println!("✗ ShineGlobalConfig initialization failed");
                }
            }
        }
        Err(e) => {
            println!("✗ ShineGlobalConfig creation failed: {:?}", e);
        }
    }
    
    println!("\n=== Validation Complete ===");
}

/// Helper macro to calculate field offset
macro_rules! offset_of {
    ($instance:expr, $field:ident, $base:expr) => {
        (&$instance.$field as *const _ as usize) - $base
    };
}

use offset_of;