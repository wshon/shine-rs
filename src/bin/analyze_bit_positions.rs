fn main() {
    println!("=== MP3 Side Information Bit Layout Analysis ===");
    
    let mut bit_pos = 0;
    
    // Frame header
    println!("Frame header:");
    println!("  Sync (11 bits): bit {}-{}", bit_pos, bit_pos + 10);
    bit_pos += 11;
    println!("  Version (2 bits): bit {}-{}", bit_pos, bit_pos + 1);
    bit_pos += 2;
    println!("  Layer (2 bits): bit {}-{}", bit_pos, bit_pos + 1);
    bit_pos += 2;
    println!("  CRC (1 bit): bit {}", bit_pos);
    bit_pos += 1;
    println!("  Bitrate (4 bits): bit {}-{}", bit_pos, bit_pos + 3);
    bit_pos += 4;
    println!("  Sample rate (2 bits): bit {}-{}", bit_pos, bit_pos + 1);
    bit_pos += 2;
    println!("  Padding (1 bit): bit {}", bit_pos);
    bit_pos += 1;
    println!("  Private (1 bit): bit {}", bit_pos);
    bit_pos += 1;
    println!("  Mode (2 bits): bit {}-{}", bit_pos, bit_pos + 1);
    bit_pos += 2;
    println!("  Mode ext (2 bits): bit {}-{}", bit_pos, bit_pos + 1);
    bit_pos += 2;
    println!("  Copyright (1 bit): bit {}", bit_pos);
    bit_pos += 1;
    println!("  Original (1 bit): bit {}", bit_pos);
    bit_pos += 1;
    println!("  Emphasis (2 bits): bit {}-{}", bit_pos, bit_pos + 1);
    bit_pos += 2;
    
    println!("\nFrame header ends at bit {} (byte {})", bit_pos, bit_pos / 8);
    
    // Side information for MPEG-I stereo
    println!("\nSide information:");
    println!("  Main data begin (9 bits): bit {}-{}", bit_pos, bit_pos + 8);
    bit_pos += 9;
    println!("  Private bits (3 bits): bit {}-{}", bit_pos, bit_pos + 2);
    bit_pos += 3;
    
    // SCFSI for 2 channels
    for ch in 0..2 {
        println!("  SCFSI ch{} (4 bits): bit {}-{}", ch, bit_pos, bit_pos + 3);
        bit_pos += 4;
    }
    
    println!("\nGranule information starts at bit {} (byte {})", bit_pos, bit_pos / 8);
    
    // Granule information for 2 granules, 2 channels
    for gr in 0..2 {
        for ch in 0..2 {
            println!("\nGranule {} Channel {}:", gr, ch);
            println!("  part2_3_length (12 bits): bit {}-{}", bit_pos, bit_pos + 11);
            bit_pos += 12;
            println!("  big_values (9 bits): bit {}-{}", bit_pos, bit_pos + 8);
            bit_pos += 9;
            println!("  global_gain (8 bits): bit {}-{}", bit_pos, bit_pos + 7);
            bit_pos += 8;
            println!("  scalefac_compress (4 bits): bit {}-{}", bit_pos, bit_pos + 3);
            bit_pos += 4;
            println!("  window_switching_flag (1 bit): bit {}", bit_pos);
            bit_pos += 1;
            
            // Table select (3 * 5 bits)
            for region in 0..3 {
                println!("  table_select[{}] (5 bits): bit {}-{}", region, bit_pos, bit_pos + 4);
                bit_pos += 5;
            }
            
            println!("  region0_count (4 bits): bit {}-{}", bit_pos, bit_pos + 3);
            bit_pos += 4;
            println!("  region1_count (3 bits): bit {}-{}", bit_pos, bit_pos + 2);
            bit_pos += 3;
            println!("  preflag (1 bit): bit {}", bit_pos);
            bit_pos += 1;
            println!("  scalefac_scale (1 bit): bit {}", bit_pos);
            bit_pos += 1;
            println!("  count1table_select (1 bit): bit {}", bit_pos);
            bit_pos += 1;
        }
    }
    
    println!("\nSide information ends at bit {} (byte {})", bit_pos, bit_pos / 8);
    
    // Analyze the specific bytes we're interested in
    println!("\n=== Analysis of bytes 6-7 (0x0BF0 vs 0x0000) ===");
    let byte6_start = 6 * 8; // bit 48
    let byte7_start = 7 * 8; // bit 56
    
    println!("Byte 6 (bits 48-55):");
    println!("Byte 7 (bits 56-63):");
    
    // The difference 0x0BF0 in binary
    let diff_value = 0x0BF0u16;
    println!("\n0x0BF0 = {:016b}", diff_value);
    println!("This represents {} in decimal", diff_value);
}