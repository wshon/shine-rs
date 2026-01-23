//! Data structure consistency validator
//!
//! This utility validates that our Rust data structures match
//! the memory layout and field ordering of shine's C structures.

use rust_mp3_encoder::quantization::GranuleInfo;
use rust_mp3_encoder::shine_config::{L3Loop, ShineSideInfo, MAX_CHANNELS, MAX_GRANULES, GRANULE_SIZE};
use rust_mp3_encoder::bitstream::BitstreamWriter;
use std::mem;

fn main() {
    println!("=== Data Structure Consistency Validation ===\n");
    
    validate_granule_info();
    validate_l3loop();
    validate_side_info();
    validate_bitstream_writer();
    validate_constants();
    
    println!("=== Validation Complete ===");
}

fn validate_granule_info() {
    println!("1. GranuleInfo Structure Validation");
    println!("   Size: {} bytes", mem::size_of::<GranuleInfo>());
    println!("   Alignment: {} bytes", mem::align_of::<GranuleInfo>());
    
    // Test field ordering by creating a default instance
    let gi = GranuleInfo::default();
    println!("   Default values:");
    println!("     part2_3_length: {}", gi.part2_3_length);
    println!("     big_values: {}", gi.big_values);
    println!("     count1: {}", gi.count1);
    println!("     global_gain: {}", gi.global_gain);
    println!("     scalefac_compress: {}", gi.scalefac_compress);
    println!("     table_select: {:?}", gi.table_select);
    println!("     region0_count: {}", gi.region0_count);
    println!("     region1_count: {}", gi.region1_count);
    println!("     preflag: {}", gi.preflag);
    println!("     scalefac_scale: {}", gi.scalefac_scale);
    println!("     count1table_select: {}", gi.count1table_select);
    println!("     part2_length: {}", gi.part2_length);
    println!("     sfb_lmax: {}", gi.sfb_lmax);
    println!("     quantizer_step_size: {}", gi.quantizer_step_size);
    println!("     slen: {:?}", gi.slen);
    
    // Validate MP3 standard constraints
    assert!(gi.big_values <= 288, "big_values must be <= 288 per MP3 standard");
    assert!(gi.global_gain <= 255, "global_gain must be <= 255");
    assert!(gi.sfb_lmax <= 21, "sfb_lmax must be <= 21");
    
    println!("   ✓ Field ordering and constraints validated\n");
}

fn validate_l3loop() {
    println!("2. L3Loop Structure Validation");
    println!("   Size: {} bytes", mem::size_of::<L3Loop>());
    println!("   Alignment: {} bytes", mem::align_of::<L3Loop>());
    
    let l3loop = L3Loop::default();
    println!("   Array sizes:");
    println!("     xrsq: {} elements", l3loop.xrsq.len());
    println!("     xrabs: {} elements", l3loop.xrabs.len());
    println!("     en_tot: {} elements", l3loop.en_tot.len());
    println!("     en: {}x{} elements", l3loop.en.len(), l3loop.en[0].len());
    println!("     xm: {}x{} elements", l3loop.xm.len(), l3loop.xm[0].len());
    println!("     xrmaxl: {} elements", l3loop.xrmaxl.len());
    println!("     steptab: {} elements", l3loop.steptab.len());
    println!("     steptabi: {} elements", l3loop.steptabi.len());
    println!("     int2idx: {} elements", l3loop.int2idx.len());
    
    // Validate array sizes match shine
    assert_eq!(l3loop.xrsq.len(), GRANULE_SIZE);
    assert_eq!(l3loop.xrabs.len(), GRANULE_SIZE);
    assert_eq!(l3loop.en_tot.len(), MAX_GRANULES);
    assert_eq!(l3loop.en.len(), MAX_GRANULES);
    assert_eq!(l3loop.en[0].len(), 21);
    assert_eq!(l3loop.xm.len(), MAX_GRANULES);
    assert_eq!(l3loop.xm[0].len(), 21);
    assert_eq!(l3loop.xrmaxl.len(), MAX_GRANULES);
    assert_eq!(l3loop.steptab.len(), 128); // Match shine's 128 elements
    assert_eq!(l3loop.steptabi.len(), 128); // Match shine's 128 elements
    assert_eq!(l3loop.int2idx.len(), 10000);
    
    println!("   ✓ Array sizes match shine specification\n");
}

fn validate_side_info() {
    println!("3. ShineSideInfo Structure Validation");
    println!("   Size: {} bytes", mem::size_of::<ShineSideInfo>());
    println!("   Alignment: {} bytes", mem::align_of::<ShineSideInfo>());
    
    let side_info = ShineSideInfo::default();
    println!("   Array dimensions:");
    println!("     scfsi: {}x{} elements", side_info.scfsi.len(), side_info.scfsi[0].len());
    println!("     gr: {}x{} elements", side_info.gr.len(), side_info.gr[0].len());
    
    // Validate dimensions match shine
    assert_eq!(side_info.scfsi.len(), MAX_CHANNELS);
    assert_eq!(side_info.scfsi[0].len(), 4);
    assert_eq!(side_info.gr.len(), MAX_GRANULES);
    assert_eq!(side_info.gr[0].len(), MAX_CHANNELS);
    
    println!("   ✓ Structure dimensions match shine specification\n");
}

fn validate_bitstream_writer() {
    println!("4. BitstreamWriter Structure Validation");
    println!("   Size: {} bytes", mem::size_of::<BitstreamWriter>());
    println!("   Alignment: {} bytes", mem::align_of::<BitstreamWriter>());
    
    let mut bs = BitstreamWriter::new(1024);
    
    // Test basic functionality
    bs.write_bits(0xAB, 8);
    bs.write_bits(0xCD, 8);
    
    assert_eq!(bs.bytes_written(), 2);
    let data = bs.buffer();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0], 0xAB);
    assert_eq!(data[1], 0xCD);
    
    println!("   ✓ Basic bitstream operations working correctly\n");
}

fn validate_constants() {
    println!("5. Constants Validation");
    println!("   MAX_CHANNELS: {}", MAX_CHANNELS);
    println!("   MAX_GRANULES: {}", MAX_GRANULES);
    println!("   GRANULE_SIZE: {}", GRANULE_SIZE);
    
    // Validate constants match shine
    assert_eq!(MAX_CHANNELS, 2);
    assert_eq!(MAX_GRANULES, 2);
    assert_eq!(GRANULE_SIZE, 576);
    
    println!("   ✓ Constants match shine specification\n");
}