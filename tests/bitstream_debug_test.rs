//! Bitstream debugging tests
//!
//! This module contains tests specifically designed to debug and fix
//! the bitstream format issues we're seeing in the comparison tests.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

/// Analyze the structure of MP3 data to identify issues
fn analyze_mp3_structure(data: &[u8], label: &str) {
    println!("\n=== {} Analysis ===", label);
    println!("Total size: {} bytes", data.len());
    
    if data.is_empty() {
        println!("Data is empty!");
        return;
    }
    
    // Show first 32 bytes in hex
    println!("First 32 bytes:");
    for (i, chunk) in data.chunks(16).take(2).enumerate() {
        print!("{:04X}: ", i * 16);
        for byte in chunk {
            print!("{:02X} ", byte);
        }
        println!();
    }
    
    // Find all potential sync words
    let mut sync_positions = Vec::new();
    for i in 0..data.len().saturating_sub(1) {
        let sync = ((data[i] as u16) << 3) | ((data[i + 1] as u16) >> 5);
        if sync == 0x7FF {
            sync_positions.push(i);
        }
    }
    
    println!("Potential sync words found: {}", sync_positions.len());
    if sync_positions.len() <= 10 {
        for (idx, &pos) in sync_positions.iter().enumerate() {
            println!("  Sync {}: position {}", idx + 1, pos);
            
            // Analyze the frame header at this position
            if pos + 3 < data.len() {
                let header = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                analyze_frame_header(header, pos);
            }
        }
    } else {
        println!("Too many sync words - showing first 10:");
        for (idx, &pos) in sync_positions.iter().take(10).enumerate() {
            println!("  Sync {}: position {}", idx + 1, pos);
        }
    }
    
    // Check for patterns that might indicate issues
    let mut ff_count = 0;
    let mut zero_count = 0;
    for &byte in data {
        if byte == 0xFF {
            ff_count += 1;
        } else if byte == 0x00 {
            zero_count += 1;
        }
    }
    
    println!("Byte patterns:");
    println!("  0xFF bytes: {} ({:.1}%)", ff_count, (ff_count as f32 / data.len() as f32) * 100.0);
    println!("  0x00 bytes: {} ({:.1}%)", zero_count, (zero_count as f32 / data.len() as f32) * 100.0);
    
    if ff_count as f32 / data.len() as f32 > 0.5 {
        println!("⚠ WARNING: Too many 0xFF bytes - possible encoding issue");
    }
    
    // Look for frame size consistency
    if sync_positions.len() > 1 {
        let frame_sizes: Vec<usize> = sync_positions.windows(2)
            .map(|w| w[1] - w[0])
            .collect();
        
        println!("Frame sizes: {:?}", frame_sizes);
        
        if let (Some(&min_size), Some(&max_size)) = (frame_sizes.iter().min(), frame_sizes.iter().max()) {
            println!("Frame size range: {} - {} bytes", min_size, max_size);
            
            if min_size < 10 {
                println!("⚠ WARNING: Very small frames detected - possible false sync words");
            }
            
            if max_size > 2000 {
                println!("⚠ WARNING: Very large frames detected - possible missing sync words");
            }
        }
    }
}

/// Analyze a single MP3 frame header
fn analyze_frame_header(header: u32, position: usize) {
    println!("    Frame at {}: 0x{:08X}", position, header);
    
    // Extract fields
    let sync = (header >> 21) & 0x7FF;
    let version = (header >> 19) & 0x3;
    let layer = (header >> 17) & 0x3;
    let protection = (header >> 16) & 0x1;
    let bitrate_index = (header >> 12) & 0xF;
    let samplerate_index = (header >> 10) & 0x3;
    let padding = (header >> 9) & 0x1;
    let private_bit = (header >> 8) & 0x1;
    let mode = (header >> 6) & 0x3;
    let mode_ext = (header >> 4) & 0x3;
    let copyright = (header >> 3) & 0x1;
    let original = (header >> 2) & 0x1;
    let emphasis = header & 0x3;
    
    println!("      Sync: 0x{:03X} {}", sync, if sync == 0x7FF { "✓" } else { "✗" });
    println!("      Version: {} ({})", version, match version {
        3 => "MPEG-1",
        2 => "MPEG-2",
        0 => "MPEG-2.5",
        _ => "Reserved",
    });
    println!("      Layer: {} ({})", layer, match layer {
        1 => "Layer III",
        2 => "Layer II", 
        3 => "Layer I",
        _ => "Reserved",
    });
    println!("      Protection: {} ({})", protection, if protection == 1 { "No CRC" } else { "CRC" });
    println!("      Bitrate index: {}", bitrate_index);
    println!("      Sample rate index: {}", samplerate_index);
    println!("      Padding: {}", padding);
    println!("      Mode: {} ({})", mode, match mode {
        0 => "Stereo",
        1 => "Joint stereo",
        2 => "Dual channel",
        3 => "Mono",
        _ => "Unknown",
    });
    
    // Validate header
    let mut issues = Vec::new();
    if sync != 0x7FF {
        issues.push("Invalid sync word");
    }
    if version == 1 {
        issues.push("Reserved MPEG version");
    }
    if layer == 0 {
        issues.push("Reserved layer");
    }
    if bitrate_index == 0 || bitrate_index == 15 {
        issues.push("Invalid bitrate index");
    }
    if samplerate_index == 3 {
        issues.push("Reserved sample rate");
    }
    if emphasis == 2 {
        issues.push("Reserved emphasis");
    }
    
    if !issues.is_empty() {
        println!("      ⚠ Issues: {}", issues.join(", "));
    } else {
        println!("      ✓ Header appears valid");
    }
}

#[test]
fn test_debug_simple_encoding() {
    println!("\n🔍 Testing simple encoding to debug bitstream issues");
    
    let config = Config {
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
    };
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    
    // Test with all zeros (should be very compressible)
    let samples_per_frame = encoder.samples_per_frame();
    println!("Samples per frame: {}", samples_per_frame);
    
    let pcm_data = vec![0i16; samples_per_frame];
    
    let result = encoder.encode_frame(&pcm_data);
    assert!(result.is_ok(), "Encoding should succeed");
    
    let mp3_data = result.unwrap();
    analyze_mp3_structure(mp3_data, "Simple Zero Encoding");
    
    // The output should be a single valid MP3 frame
    assert!(!mp3_data.is_empty(), "Output should not be empty");
    
    // Should have exactly one sync word at the beginning
    let sync_count = count_sync_words(mp3_data);
    if sync_count != 1 {
        println!("⚠ Expected 1 sync word, found {}", sync_count);
        
        // This is the main issue we're trying to fix
        // Let's see what's causing multiple sync words
        if sync_count > 1 {
            println!("Multiple sync words suggest either:");
            println!("1. Multiple frames being generated (should be 1)");
            println!("2. False sync words in the data (0xFF bytes)");
            println!("3. Bitstream corruption");
        }
    }
}

#[test]
fn test_debug_different_patterns() {
    println!("\n🔍 Testing different input patterns");
    
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Stereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    let samples_per_frame = encoder.samples_per_frame();
    let total_samples = samples_per_frame * 2; // Stereo
    
    // Test different patterns
    let patterns = [
        ("All zeros", vec![0i16; total_samples]),
        ("Small constant", vec![100i16; total_samples]),
        ("Max positive", vec![i16::MAX; total_samples]),
        ("Max negative", vec![i16::MIN; total_samples]),
        ("Alternating", (0..total_samples).map(|i| if i % 2 == 0 { 1000 } else { -1000 }).collect()),
    ];
    
    for (name, pcm_data) in patterns.iter() {
        println!("\n--- Testing pattern: {} ---", name);
        
        let result = encoder.encode_frame_interleaved(pcm_data);
        match result {
            Ok(mp3_data) => {
                analyze_mp3_structure(mp3_data, name);
                
                let sync_count = count_sync_words(mp3_data);
                println!("Sync words for {}: {}", name, sync_count);
                
                if sync_count != 1 {
                    println!("⚠ Pattern {} has {} sync words (expected 1)", name, sync_count);
                }
            },
            Err(e) => {
                println!("✗ Pattern {} failed to encode: {:?}", name, e);
            }
        }
    }
}

#[test]
fn test_debug_encoder_state() {
    println!("\n🔍 Testing encoder internal state");
    
    let config = Config {
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
    };
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    let samples_per_frame = encoder.samples_per_frame();
    
    // Test multiple frames to see if the issue compounds
    for frame_num in 1..=3 {
        println!("\n--- Frame {} ---", frame_num);
        
        // Use different data for each frame
        let pcm_data: Vec<i16> = (0..samples_per_frame)
            .map(|i| ((i * frame_num) % 1000) as i16)
            .collect();
        
        let result = encoder.encode_frame(&pcm_data);
        match result {
            Ok(mp3_data) => {
                let sync_count = count_sync_words(mp3_data);
                println!("Frame {}: {} bytes, {} sync words", frame_num, mp3_data.len(), sync_count);
                
                if sync_count != 1 {
                    analyze_mp3_structure(mp3_data, &format!("Frame {}", frame_num));
                }
            },
            Err(e) => {
                println!("Frame {} failed: {:?}", frame_num, e);
            }
        }
    }
    
    // Test flush
    println!("\n--- Flush ---");
    let flush_result = encoder.flush();
    match flush_result {
        Ok(flush_data) => {
            if !flush_data.is_empty() {
                let sync_count = count_sync_words(flush_data);
                println!("Flush: {} bytes, {} sync words", flush_data.len(), sync_count);
                
                if sync_count > 0 {
                    analyze_mp3_structure(flush_data, "Flush");
                }
            } else {
                println!("Flush: empty (expected)");
            }
        },
        Err(e) => {
            println!("Flush failed: {:?}", e);
        }
    }
}

#[test]
fn test_debug_incremental_encoding() {
    println!("\n🔍 Testing incremental encoding");
    
    let config = Config {
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
    };
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    let samples_per_frame = encoder.samples_per_frame();
    
    // Add samples incrementally
    let chunk_size = samples_per_frame / 4;
    let mut total_output = Vec::new();
    
    for chunk_num in 1..=4 {
        println!("\n--- Adding chunk {} ---", chunk_num);
        
        let chunk_data: Vec<i16> = (0..chunk_size)
            .map(|i| ((i + chunk_num * 100) % 500) as i16)
            .collect();
        
        let result = encoder.encode_samples(&chunk_data);
        match result {
            Ok(Some(mp3_data)) => {
                println!("Chunk {} produced {} bytes", chunk_num, mp3_data.len());
                total_output.extend_from_slice(mp3_data);
                
                let sync_count = count_sync_words(mp3_data);
                if sync_count != 1 {
                    println!("⚠ Chunk {} has {} sync words", chunk_num, sync_count);
                    analyze_mp3_structure(mp3_data, &format!("Chunk {}", chunk_num));
                }
            },
            Ok(None) => {
                println!("Chunk {} buffered (expected for chunks 1-3)", chunk_num);
            },
            Err(e) => {
                println!("Chunk {} failed: {:?}", chunk_num, e);
            }
        }
    }
    
    // Final flush
    let flush_result = encoder.flush();
    match flush_result {
        Ok(flush_data) => {
            if !flush_data.is_empty() {
                total_output.extend_from_slice(flush_data);
                println!("Flush added {} bytes", flush_data.len());
            }
        },
        Err(e) => {
            println!("Flush failed: {:?}", e);
        }
    }
    
    if !total_output.is_empty() {
        println!("\n--- Total incremental output ---");
        analyze_mp3_structure(&total_output, "Incremental Total");
    }
}

/// Count sync words in MP3 data
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
fn test_debug_bitstream_writer_directly() {
    println!("\n🔍 Testing bitstream writer directly");
    
    use rust_mp3_encoder::bitstream::BitstreamWriter;
    use rust_mp3_encoder::config::Config;
    
    let config = Config {
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
    };
    
    let mut writer = BitstreamWriter::new(100);
    
    // Write just a frame header
    writer.write_frame_header(&config, false);
    let header_data = writer.flush();
    
    println!("Frame header only:");
    analyze_mp3_structure(header_data, "Header Only");
    
    // Should be exactly 4 bytes with one sync word
    assert_eq!(header_data.len(), 4, "Frame header should be 4 bytes");
    
    let sync_count = count_sync_words(header_data);
    assert_eq!(sync_count, 1, "Frame header should have exactly 1 sync word");
    
    println!("✓ Frame header test passed");
}

#[test]
fn test_debug_side_info_only() {
    println!("\n🔍 Testing side info writing");
    
    use rust_mp3_encoder::bitstream::{BitstreamWriter, SideInfo};
    use rust_mp3_encoder::quantization::GranuleInfo;
    
    let config = Config {
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
    };
    
    let mut writer = BitstreamWriter::new(100);
    
    // Create minimal side info
    let mut side_info = SideInfo::default();
    
    // Add granules for MPEG-1 mono (2 granules)
    for _ in 0..2 {
        side_info.granules.push(GranuleInfo::default());
    }
    
    writer.write_side_info(&side_info, &config);
    let side_info_data = writer.flush();
    
    println!("Side info only:");
    analyze_mp3_structure(side_info_data, "Side Info Only");
    
    // Side info should not contain sync words
    let sync_count = count_sync_words(side_info_data);
    if sync_count > 0 {
        println!("⚠ Side info contains {} sync words - this is unexpected", sync_count);
    } else {
        println!("✓ Side info contains no sync words (correct)");
    }
}