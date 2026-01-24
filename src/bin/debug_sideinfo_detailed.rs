//! Detailed side information debugging tool
//! 
//! This tool creates a minimal test case to debug side information encoding
//! by comparing our implementation with shine's expected behavior.

use rust_mp3_encoder::types::{ShineGlobalConfig, GrInfo};
use rust_mp3_encoder::bitstream::BitstreamWriter;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Detailed Side Information Debug ===");
    
    // Create a minimal config that matches shine's test case
    let mut config = ShineGlobalConfig::new();
    
    // Set up basic parameters to match shine
    config.wave.channels = 2;
    config.wave.samplerate = 44100;
    config.mpeg.version = 3; // MPEG_I = 3 (not 1!)
    config.mpeg.layer = 1;   // LAYER_III = 1
    config.mpeg.granules_per_frame = 2;
    config.mpeg.mode = 1;    // Joint stereo
    config.mpeg.bitr = 128;
    config.mpeg.bitrate_index = 9;
    config.mpeg.samplerate_index = 0; // 44100 Hz
    config.mpeg.padding = 0;
    config.mpeg.crc = 0;
    config.mpeg.ext = 0;
    config.mpeg.mode_ext = 0;
    config.mpeg.copyright = 0;
    config.mpeg.original = 1;
    config.mpeg.emph = 0;
    
    // Initialize side info with test values
    config.side_info.private_bits = 0;
    config.side_info.resv_drain = 0;
    
    // Initialize SCFSI (all zeros for first frame)
    for ch in 0..2 {
        for band in 0..4 {
            config.side_info.scfsi[ch][band] = 0;
        }
    }
    
    // Set up granule information with test values
    for gr in 0..2 {
        for ch in 0..2 {
            let gi = &mut config.side_info.gr[gr].ch[ch].tt;
            
            // Set test values that should produce known bit patterns
            if gr == 0 && ch == 0 {
                // First granule, first channel - set some realistic values
                gi.part2_3_length = 3056; // This matches what we saw in debug output
                gi.big_values = 144;       // Half of 288 (max big_values)
                gi.global_gain = 210;      // Default value
                gi.scalefac_compress = 0;
                gi.table_select = [1, 2, 3]; // Some test table selections
                gi.region0_count = 7;      // Max 4 bits (0-15)
                gi.region1_count = 7;      // Max 3 bits (0-7) - FIXED: was 13
                gi.preflag = 0;
                gi.scalefac_scale = 0;
                gi.count1table_select = 0;
                gi.count1 = 10;
                gi.part2_length = 56; // Some reasonable part2 length
            } else {
                // Other granules/channels - set to minimal values
                gi.part2_3_length = 0;
                gi.big_values = 0;
                gi.global_gain = 210;
                gi.scalefac_compress = 0;
                gi.table_select = [0, 0, 0];
                gi.region0_count = 0;
                gi.region1_count = 0;
                gi.preflag = 0;
                gi.scalefac_scale = 0;
                gi.count1table_select = 0;
                gi.count1 = 0;
                gi.part2_length = 0;
            }
        }
    }
    
    println!("Configuration setup complete:");
    println!("  Channels: {}", config.wave.channels);
    println!("  Sample rate: {}", config.wave.samplerate);
    println!("  Bitrate: {}", config.mpeg.bitr);
    println!("  Version: {} (MPEG-{})", config.mpeg.version, if config.mpeg.version == 1 { "I" } else { "II" });
    
    // Print detailed side info before encoding
    println!("\n=== Side Information Before Encoding ===");
    for gr in 0..2 {
        for ch in 0..2 {
            let gi = &config.side_info.gr[gr].ch[ch].tt;
            println!("Granule {} Channel {}:", gr, ch);
            println!("  part2_3_length: {}", gi.part2_3_length);
            println!("  big_values: {}", gi.big_values);
            println!("  global_gain: {}", gi.global_gain);
            println!("  scalefac_compress: {}", gi.scalefac_compress);
            println!("  table_select: {:?}", gi.table_select);
            println!("  region0_count: {}", gi.region0_count);
            println!("  region1_count: {}", gi.region1_count);
            println!("  preflag: {}", gi.preflag);
            println!("  scalefac_scale: {}", gi.scalefac_scale);
            println!("  count1table_select: {}", gi.count1table_select);
            println!("  count1: {}", gi.count1);
            println!("  part2_length: {}", gi.part2_length);
        }
    }
    
    // Create a new bitstream for encoding
    config.bs = BitstreamWriter::new(8192);
    
    // Encode the side information
    println!("\n=== Encoding Side Information ===");
    match rust_mp3_encoder::bitstream::format_bitstream(&mut config) {
        Ok(_) => {
            println!("Side information encoded successfully");
            
            // Get the encoded data
            let data = config.bs.get_data();
            println!("Encoded {} bytes", data.len());
            
            // Print first 32 bytes in hex
            println!("\nFirst 32 bytes of encoded data:");
            for (i, &byte) in data.iter().take(32).enumerate() {
                if i % 8 == 0 {
                    print!("\n{:04X}: ", i);
                }
                print!("{:02X} ", byte);
            }
            println!();
            
            // Print bit-by-bit analysis of first 8 bytes
            println!("\nBit-by-bit analysis of first 8 bytes:");
            for (i, &byte) in data.iter().take(8).enumerate() {
                println!("Byte {}: 0x{:02X} = {:08b}", i, byte, byte);
            }
            
            // Analyze frame header (first 4 bytes)
            if data.len() >= 4 {
                let header = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                println!("\nFrame header analysis:");
                println!("  Raw header: 0x{:08X}", header);
                println!("  Sync: 0x{:03X} (should be 0x7FF)", (header >> 21) & 0x7FF);
                println!("  Version: {} (should be {})", (header >> 19) & 0x3, config.mpeg.version);
                println!("  Layer: {} (should be {})", (header >> 17) & 0x3, config.mpeg.layer);
                println!("  CRC: {} (should be {})", (header >> 16) & 0x1, if config.mpeg.crc == 0 { 1 } else { 0 });
                println!("  Bitrate: {} (should be {})", (header >> 12) & 0xF, config.mpeg.bitrate_index);
                println!("  Sample rate: {} (should be {})", (header >> 10) & 0x3, config.mpeg.samplerate_index % 3);
                println!("  Padding: {} (should be {})", (header >> 9) & 0x1, config.mpeg.padding);
                println!("  Private: {} (should be {})", (header >> 8) & 0x1, config.mpeg.ext);
                println!("  Mode: {} (should be {})", (header >> 6) & 0x3, config.mpeg.mode);
                println!("  Mode ext: {} (should be {})", (header >> 4) & 0x3, config.mpeg.mode_ext);
                println!("  Copyright: {} (should be {})", (header >> 3) & 0x1, config.mpeg.copyright);
                println!("  Original: {} (should be {})", (header >> 2) & 0x1, config.mpeg.original);
                println!("  Emphasis: {} (should be {})", header & 0x3, config.mpeg.emph);
            }
            
            // Save to file for comparison
            let mut file = File::create("debug_sideinfo_output.mp3")?;
            file.write_all(data)?;
            println!("\nSaved encoded data to debug_sideinfo_output.mp3");
            
            // Compare with expected shine output
            println!("\n=== Comparison with Expected Output ===");
            println!("Our output (first 8 bytes): {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", 
                     data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]);
            println!("Shine output (expected):     FF FB 92 04 00 00 03 93");
            
            // Analyze the differences
            let expected = [0xFF, 0xFB, 0x92, 0x04, 0x00, 0x00, 0x03, 0x93];
            for i in 0..8.min(data.len()) {
                if data[i] != expected[i] {
                    println!("Difference at byte {}: got 0x{:02X}, expected 0x{:02X}", i, data[i], expected[i]);
                }
            }
            
        }
        Err(e) => {
            println!("Error encoding side information: {}", e);
        }
    }
    
    Ok(())
}