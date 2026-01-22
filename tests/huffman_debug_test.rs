//! Huffman encoding debugging tests
//!
//! This module tests the Huffman encoder specifically to debug
//! the 0xFF byte generation issue.

use rust_mp3_encoder::huffman::HuffmanEncoder;
use rust_mp3_encoder::bitstream::BitstreamWriter;
use rust_mp3_encoder::quantization::GranuleInfo;

#[test]
fn test_huffman_count1_all_zeros() {
    println!("\n🔍 Testing Huffman count1 encoding with all zeros");
    
    let huffman = HuffmanEncoder::new();
    let mut writer = BitstreamWriter::new(100);
    
    // Test with all-zero quantized coefficients
    let quantized = [0i32; 576];
    let mut granule_info = GranuleInfo::default();
    
    // Simulate what the quantization loop would set for all zeros
    granule_info.big_values = 0;
    granule_info.count1 = 144; // 576 / 4 = 144 quadruples
    granule_info.count1table_select = false; // Use table A
    
    println!("Granule info:");
    println!("  big_values: {}", granule_info.big_values);
    println!("  count1: {}", granule_info.count1);
    println!("  count1table_select: {}", granule_info.count1table_select);
    
    let result = huffman.encode_count1(&quantized, &granule_info, &mut writer);
    match result {
        Ok(bits) => {
            println!("Count1 encoding succeeded: {} bits", bits);
            
            let data = writer.flush();
            println!("Output: {} bytes", data.len());
            
            // Analyze the output
            let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
            let ff_percentage = if data.len() > 0 { (ff_count as f32 / data.len() as f32) * 100.0 } else { 0.0 };
            
            println!("0xFF bytes: {} ({:.1}%)", ff_count, ff_percentage);
            
            if data.len() >= 16 {
                print!("First 16 bytes: ");
                for i in 0..16 {
                    print!("{:02X} ", data[i]);
                }
                println!();
            }
            
            if ff_percentage > 50.0 {
                println!("❌ PROBLEM: Too many 0xFF bytes in count1 encoding");
            } else {
                println!("✓ Count1 encoding looks reasonable");
            }
        },
        Err(e) => {
            println!("❌ Count1 encoding failed: {:?}", e);
        }
    }
}

#[test]
fn test_huffman_count1_single_quadruple() {
    println!("\n🔍 Testing Huffman count1 encoding with single quadruple");
    
    let huffman = HuffmanEncoder::new();
    
    // Test different quadruple patterns
    let test_cases = [
        ("All zeros", [0, 0, 0, 0]),
        ("Single 1", [1, 0, 0, 0]),
        ("All 1s", [1, 1, 1, 1]),
        ("Mixed", [1, 0, 1, 0]),
    ];
    
    for (name, pattern) in test_cases.iter() {
        println!("\n--- Testing pattern: {} {:?} ---", name, pattern);
        
        let mut writer = BitstreamWriter::new(100);
        let mut quantized = [0i32; 576];
        
        // Set the pattern in the first quadruple
        quantized[0] = pattern[0];
        quantized[1] = pattern[1];
        quantized[2] = pattern[2];
        quantized[3] = pattern[3];
        
        let mut granule_info = GranuleInfo::default();
        granule_info.big_values = 0;
        granule_info.count1 = 1; // Just one quadruple
        granule_info.count1table_select = false; // Use table A
        
        let result = huffman.encode_count1(&quantized, &granule_info, &mut writer);
        match result {
            Ok(bits) => {
                println!("  Bits used: {}", bits);
                
                let data = writer.flush();
                println!("  Output bytes: {}", data.len());
                
                if !data.is_empty() {
                    print!("  Bytes: ");
                    for &byte in data {
                        print!("{:02X} ", byte);
                    }
                    println!();
                    
                    let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
                    if ff_count > 0 {
                        println!("  ⚠ Contains {} 0xFF bytes", ff_count);
                    }
                }
            },
            Err(e) => {
                println!("  ❌ Failed: {:?}", e);
            }
        }
    }
}

#[test]
fn test_huffman_big_values_all_zeros() {
    println!("\n🔍 Testing Huffman big values encoding with all zeros");
    
    let huffman = HuffmanEncoder::new();
    let mut writer = BitstreamWriter::new(100);
    
    // Test with all-zero quantized coefficients
    let quantized = [0i32; 576];
    let mut granule_info = GranuleInfo::default();
    
    // For all zeros, big_values should be 0
    granule_info.big_values = 0;
    granule_info.table_select = [0, 0, 0]; // Table 0 for all-zero regions
    granule_info.address1 = 0;
    granule_info.address2 = 0;
    granule_info.address3 = 0;
    
    println!("Testing big values with all zeros...");
    let result = huffman.encode_big_values(&quantized, &granule_info, &mut writer);
    match result {
        Ok(bits) => {
            println!("Big values encoding: {} bits", bits);
            
            let data = writer.flush();
            println!("Output: {} bytes", data.len());
            
            if !data.is_empty() {
                let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
                println!("0xFF bytes: {}", ff_count);
                
                if ff_count > 0 {
                    println!("❌ PROBLEM: Big values encoding produced 0xFF bytes");
                } else {
                    println!("✓ Big values encoding looks clean");
                }
            } else {
                println!("✓ Big values encoding produced no output (expected for all zeros)");
            }
        },
        Err(e) => {
            println!("❌ Big values encoding failed: {:?}", e);
        }
    }
}

#[test]
fn test_count1_table_lookup() {
    println!("\n🔍 Testing count1 table lookup directly");
    
    use rust_mp3_encoder::tables::COUNT1_TABLES;
    
    println!("Count1 tables available: {}", COUNT1_TABLES.len());
    
    for (table_idx, table) in COUNT1_TABLES.iter().enumerate() {
        println!("\nTable {}: {} codes, {} lengths", table_idx, table.codes.len(), table.lengths.len());
        
        // Test the all-zero case (index 0)
        if !table.codes.is_empty() && !table.lengths.is_empty() {
            let code = table.codes[0];
            let length = table.lengths[0];
            println!("  All-zero quadruple: code=0x{:X}, length={} bits", code, length);
            
            // Check if this would produce 0xFF when written
            if length == 8 && code == 0xFF {
                println!("  ⚠ WARNING: All-zero quadruple produces 0xFF byte");
            }
        }
        
        // Show first few entries
        println!("  First 8 entries:");
        for i in 0..8.min(table.codes.len()) {
            let code = table.codes[i];
            let length = table.lengths[i];
            println!("    [{}]: code=0x{:X}, length={}", i, code, length);
        }
    }
}

#[test]
fn test_manual_count1_encoding() {
    println!("\n🔍 Testing manual count1 encoding step by step");
    
    use rust_mp3_encoder::tables::COUNT1_TABLES;
    
    let table = COUNT1_TABLES[0]; // Table A
    let mut writer = BitstreamWriter::new(100);
    
    // Manually encode a few all-zero quadruples
    let num_quadruples = 5;
    
    for i in 0..num_quadruples {
        println!("Encoding quadruple {}: [0, 0, 0, 0]", i);
        
        // For all zeros: v=0, w=0, x=0, y=0
        // Table index = v*8 + w*4 + x*2 + y = 0
        let table_idx = 0;
        
        let code = table.codes[table_idx] as u32;
        let length = table.lengths[table_idx];
        
        println!("  Using code=0x{:X}, length={}", code, length);
        
        writer.write_bits(code, length);
        
        // No sign bits needed for all zeros
    }
    
    let data = writer.flush();
    println!("\nManual encoding result:");
    println!("  {} quadruples encoded", num_quadruples);
    println!("  Output: {} bytes", data.len());
    
    if !data.is_empty() {
        print!("  Bytes: ");
        for &byte in data {
            print!("{:02X} ", byte);
        }
        println!();
        
        let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
        println!("  0xFF bytes: {}", ff_count);
        
        if ff_count as f32 / data.len() as f32 > 0.5 {
            println!("  ❌ PROBLEM: Manual encoding also produces too many 0xFF bytes");
            
            // Let's see what the table actually contains
            println!("  Table A entry 0: code=0x{:X}, length={}", table.codes[0], table.lengths[0]);
            
            if table.codes[0] == 1 && table.lengths[0] == 1 {
                println!("  This means all-zero quadruples are encoded as single '1' bit");
                println!("  Multiple '1' bits would create 0xFF patterns when packed");
            }
        } else {
            println!("  ✓ Manual encoding looks reasonable");
        }
    }
}