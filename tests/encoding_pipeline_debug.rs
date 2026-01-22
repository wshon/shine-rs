//! Encoding pipeline debugging tests
//!
//! This module tests each stage of the encoding pipeline individually
//! to isolate where the 0xFF bytes are being generated.

use rust_mp3_encoder::bitstream::{BitstreamWriter, SideInfo};
use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo};
use rust_mp3_encoder::huffman::HuffmanEncoder;
use rust_mp3_encoder::config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

/// Test configuration
fn test_config() -> Config {
    Config {
        wave: WaveConfig {
            channels: Channels::Mono,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Mono,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    }
}

/// Analyze data for 0xFF patterns
fn analyze_ff_pattern(data: &[u8], label: &str) {
    let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
    let total = data.len();
    let ff_percentage = if total > 0 { (ff_count as f32 / total as f32) * 100.0 } else { 0.0 };
    
    println!("{}: {} bytes, {} 0xFF bytes ({:.1}%)", label, total, ff_count, ff_percentage);
    
    if ff_percentage > 50.0 {
        println!("  ⚠ WARNING: High 0xFF content");
        
        // Show pattern of 0xFF bytes
        let mut consecutive_ff = 0;
        let mut max_consecutive = 0;
        for &byte in data {
            if byte == 0xFF {
                consecutive_ff += 1;
                max_consecutive = max_consecutive.max(consecutive_ff);
            } else {
                consecutive_ff = 0;
            }
        }
        println!("  Max consecutive 0xFF: {}", max_consecutive);
        
        // Show first few bytes
        if data.len() >= 16 {
            print!("  First 16 bytes: ");
            for i in 0..16 {
                print!("{:02X} ", data[i]);
            }
            println!();
        }
    }
}

#[test]
fn test_debug_quantization_output() {
    println!("\n🔍 Testing quantization loop output");
    
    let config = test_config();
    let mut quantizer = QuantizationLoop::new();
    
    // Test with all-zero MDCT coefficients (should compress well)
    let mdct_coeffs = [0i32; 576];
    let max_bits = 1000;
    let mut side_info = GranuleInfo::default();
    let mut quantized = [0i32; 576];
    
    let result = quantizer.quantize_and_encode(&mdct_coeffs, max_bits, &mut side_info, &mut quantized);
    
    match result {
        Ok(bits_used) => {
            println!("Quantization succeeded: {} bits used", bits_used);
            println!("Big values: {}", side_info.big_values);
            println!("Global gain: {}", side_info.global_gain);
            println!("Table select: {:?}", side_info.table_select);
            
            // Check quantized coefficients
            let non_zero_count = quantized.iter().filter(|&&x| x != 0).count();
            println!("Non-zero quantized coefficients: {}", non_zero_count);
            
            if non_zero_count > 0 {
                let max_abs = quantized.iter().map(|&x| x.abs()).max().unwrap_or(0);
                println!("Max absolute quantized value: {}", max_abs);
            }
        },
        Err(e) => {
            println!("Quantization failed: {:?}", e);
        }
    }
}

#[test]
fn test_debug_huffman_encoding() {
    println!("\n🔍 Testing Huffman encoding output");
    
    let config = test_config();
    let huffman = HuffmanEncoder::new();
    let mut writer = BitstreamWriter::new(200);
    
    // Test with all-zero quantized coefficients
    let quantized = [0i32; 576];
    let mut granule_info = GranuleInfo::default();
    
    // Set up granule info for all-zero case
    granule_info.big_values = 0;
    granule_info.table_select = [0, 0, 0]; // Table 0 for all-zero regions
    granule_info.address1 = 0;
    granule_info.address2 = 0;
    granule_info.address3 = 0;
    
    println!("Testing big values encoding...");
    let big_values_result = huffman.encode_big_values(&quantized, &granule_info, &mut writer);
    match big_values_result {
        Ok(bits) => {
            println!("Big values encoding: {} bits", bits);
        },
        Err(e) => {
            println!("Big values encoding failed: {:?}", e);
        }
    }
    
    println!("Testing count1 encoding...");
    let count1_result = huffman.encode_count1(&quantized, &granule_info, &mut writer);
    match count1_result {
        Ok(bits) => {
            println!("Count1 encoding: {} bits", bits);
        },
        Err(e) => {
            println!("Count1 encoding failed: {:?}", e);
        }
    }
    
    let huffman_data = writer.flush();
    analyze_ff_pattern(huffman_data, "Huffman output");
}

#[test]
fn test_debug_complete_frame_construction() {
    println!("\n🔍 Testing complete frame construction step by step");
    
    let config = test_config();
    let mut writer = BitstreamWriter::new(200);
    
    // Step 1: Write frame header
    println!("Step 1: Writing frame header");
    writer.write_frame_header(&config, false);
    let header_data = writer.buffer().to_vec();
    analyze_ff_pattern(&header_data, "Frame header");
    
    // Step 2: Write side info
    println!("Step 2: Writing side info");
    let mut side_info = SideInfo::default();
    
    // Add granules for MPEG-1 mono (2 granules)
    for _ in 0..2 {
        let mut granule = GranuleInfo::default();
        granule.big_values = 0; // All zeros
        granule.table_select = [0, 0, 0];
        side_info.granules.push(granule);
    }
    
    writer.write_side_info(&side_info, &config);
    let after_side_info = writer.buffer().to_vec();
    let side_info_only = &after_side_info[header_data.len()..];
    analyze_ff_pattern(side_info_only, "Side info");
    
    // Step 3: Simulate Huffman data (should be minimal for all zeros)
    println!("Step 3: Adding minimal Huffman data");
    // For all-zero coefficients, we should write very little data
    // Let's just add a few bits to simulate the minimal case
    writer.write_bits(0, 4); // Some padding bits
    
    let final_data = writer.flush();
    let huffman_only = &final_data[after_side_info.len()..];
    analyze_ff_pattern(huffman_only, "Huffman data");
    analyze_ff_pattern(final_data, "Complete frame");
    
    // Check if the complete frame has the expected structure
    let sync_count = count_sync_words(final_data);
    println!("Complete frame sync words: {}", sync_count);
    
    if sync_count == 1 {
        println!("✓ Frame construction looks correct");
    } else {
        println!("⚠ Frame construction has issues");
    }
}

#[test]
fn test_debug_encoder_pipeline_isolation() {
    println!("\n🔍 Testing encoder pipeline with isolation");
    
    use rust_mp3_encoder::{Mp3Encoder, Config};
    
    let config = test_config();
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    
    // Test with a very simple pattern that should not produce many 0xFF bytes
    let samples_per_frame = encoder.samples_per_frame();
    
    // Try different simple patterns
    let patterns = [
        ("All zeros", vec![0i16; samples_per_frame]),
        ("Small values", vec![1i16; samples_per_frame]),
        ("Alternating 0,1", (0..samples_per_frame).map(|i| (i % 2) as i16).collect()),
    ];
    
    for (name, pcm_data) in patterns.iter() {
        println!("\n--- Testing pattern: {} ---", name);
        
        // Reset encoder for each test
        encoder.reset();
        
        let result = encoder.encode_frame(pcm_data);
        match result {
            Ok(mp3_data) => {
                analyze_ff_pattern(mp3_data, name);
                
                let sync_count = count_sync_words(mp3_data);
                println!("Sync words: {}", sync_count);
                
                if sync_count == 1 {
                    println!("✓ Pattern {} encoded correctly", name);
                } else {
                    println!("⚠ Pattern {} has {} sync words", name, sync_count);
                    
                    // If this pattern fails, the issue is in the core pipeline
                    if *name == "All zeros" {
                        println!("❌ CRITICAL: All-zero pattern should encode cleanly");
                        
                        // Let's see what the frame looks like
                        if mp3_data.len() >= 16 {
                            print!("First 16 bytes: ");
                            for i in 0..16 {
                                print!("{:02X} ", mp3_data[i]);
                            }
                            println!();
                        }
                    }
                }
            },
            Err(e) => {
                println!("❌ Pattern {} failed to encode: {:?}", name, e);
            }
        }
    }
}

/// Count sync words in data
fn count_sync_words(data: &[u8]) -> usize {
    let mut count = 0;
    for i in 0..data.len().saturating_sub(1) {
        let sync = ((data[i] as u16) << 3) | ((data[i + 1] as u16) >> 5);
        if sync == 0x7FF {
            count += 1;
        }
    }
    count
}

#[test]
fn test_debug_bitstream_writer_patterns() {
    println!("\n🔍 Testing bitstream writer with different bit patterns");
    
    let mut writer = BitstreamWriter::new(100);
    
    // Test writing different patterns to see if any produce 0xFF
    let test_cases = [
        ("All zeros", vec![(0u32, 8u8); 10]),
        ("All ones", vec![(0xFFu32, 8u8); 10]),
        ("Alternating", {
            let mut pattern = Vec::new();
            for _ in 0..5 {
                pattern.push((0xAAu32, 8u8));
                pattern.push((0x55u32, 8u8));
            }
            pattern
        }),
        ("Mixed bits", vec![(0x12u32, 8u8), (0x34u32, 8u8), (0x56u32, 8u8), (0x78u32, 8u8)]),
    ];
    
    for (name, bit_patterns) in test_cases.iter() {
        writer.reset();
        
        for &(value, bits) in bit_patterns {
            writer.write_bits(value, bits);
        }
        
        let data = writer.flush();
        analyze_ff_pattern(data, name);
    }
}

#[test]
fn test_debug_side_info_content() {
    println!("\n🔍 Testing side info content generation");
    
    let config = test_config();
    let mut writer = BitstreamWriter::new(100);
    
    // Test different side info configurations
    let test_cases = [
        ("Default granule", GranuleInfo::default()),
        ("High global gain", {
            let mut gi = GranuleInfo::default();
            gi.global_gain = 255;
            gi
        }),
        ("Non-zero big values", {
            let mut gi = GranuleInfo::default();
            gi.big_values = 100;
            gi.table_select = [1, 2, 3];
            gi
        }),
    ];
    
    for (name, granule_info) in test_cases.iter() {
        writer.reset();
        
        let mut side_info = SideInfo::default();
        side_info.granules.push(granule_info.clone());
        side_info.granules.push(granule_info.clone()); // MPEG-1 needs 2 granules
        
        writer.write_side_info(&side_info, &config);
        let data = writer.flush();
        
        analyze_ff_pattern(data, &format!("Side info: {}", name));
    }
}