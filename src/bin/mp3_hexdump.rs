//! MP3 Hex Dump Tool
//!
//! A tool to display MP3 file content in hexadecimal format for debugging.

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args.len() > 4 {
        eprintln!("用法: {} <mp3文件路径> [起始位置] [字节数]", args[0]);
        eprintln!("示例: {} tests/output/encoded_output.mp3", args[0]);
        eprintln!("示例: {} tests/output/encoded_output.mp3 400 100", args[0]);
        std::process::exit(1);
    }

    let file_path = Path::new(&args[1]);
    let start_pos = if args.len() > 2 { 
        args[2].parse::<u64>().unwrap_or(0) 
    } else { 
        0 
    };
    let byte_count = if args.len() > 3 { 
        args[3].parse::<usize>().unwrap_or(512) 
    } else { 
        512 
    };

    if !file_path.exists() {
        eprintln!("❌ 错误: 文件不存在: {}", file_path.display());
        std::process::exit(1);
    }

    match dump_hex(file_path, start_pos, byte_count) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("❌ 错误: {}", e);
            std::process::exit(1);
        }
    }
}

fn dump_hex(file_path: &Path, start_pos: u64, byte_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    
    // Get file size
    let file_size = file.metadata()?.len();
    
    println!("📁 文件: {}", file_path.display());
    println!("📏 文件大小: {} 字节", file_size);
    println!("📍 起始位置: {}", start_pos);
    println!("📊 显示字节数: {}", byte_count);
    println!("{}", "=".repeat(80));

    // Seek to start position
    file.seek(SeekFrom::Start(start_pos))?;

    // Read bytes
    let mut buffer = vec![0u8; byte_count];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Display hex dump
    for (i, chunk) in buffer.chunks(16).enumerate() {
        let offset = start_pos + (i * 16) as u64;
        
        // Print offset
        print!("{:08X}: ", offset);
        
        // Print hex bytes
        for (j, &byte) in chunk.iter().enumerate() {
            print!("{:02X} ", byte);
            if j == 7 {
                print!(" ");
            }
        }
        
        // Pad if less than 16 bytes
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                print!("   ");
                if j == 7 {
                    print!(" ");
                }
            }
        }
        
        // Print ASCII representation
        print!(" |");
        for &byte in chunk {
            if byte >= 32 && byte <= 126 {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }

    // Analyze potential MP3 frame headers
    println!("\n🔍 分析潜在的 MP3 帧头:");
    analyze_mp3_headers(&buffer, start_pos);

    Ok(())
}

fn analyze_mp3_headers(buffer: &[u8], start_offset: u64) {
    for i in 0..buffer.len().saturating_sub(4) {
        let header = u32::from_be_bytes([
            buffer[i], 
            buffer[i+1], 
            buffer[i+2], 
            buffer[i+3]
        ]);
        
        let sync_word = (header >> 20) & 0xFFF;
        
        // Check if this looks like an MP3 sync word
        if sync_word >= 0xFFE {
            let position = start_offset + i as u64;
            let mpeg_version = (header >> 19) & 0x1;
            let layer = (header >> 17) & 0x3;
            let bitrate_index = (header >> 12) & 0xF;
            let sample_rate_index = (header >> 10) & 0x3;
            let channel_mode = (header >> 6) & 0x3;
            
            println!("🎵 位置 {}: 可能的帧头", position);
            println!("   同步字: 0x{:03X}", sync_word);
            println!("   MPEG版本: {} ({})", mpeg_version, if mpeg_version == 1 { "MPEG-1" } else { "MPEG-2/2.5" });
            println!("   层: {} ({})", layer, match layer {
                1 => "Layer III",
                2 => "Layer II", 
                3 => "Layer I",
                _ => "Reserved"
            });
            println!("   比特率索引: {}", bitrate_index);
            println!("   采样率索引: {}", sample_rate_index);
            println!("   声道模式: {} ({})", channel_mode, match channel_mode {
                0 => "Stereo",
                1 => "Joint Stereo",
                2 => "Dual Channel",
                3 => "Mono",
                _ => "Unknown"
            });
            println!();
        }
    }
}