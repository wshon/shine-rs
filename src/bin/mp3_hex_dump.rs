//! Simple hex dump tool for MP3 files to debug frame boundaries

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <mp3_file> [start_offset] [length]", args[0]);
        process::exit(1);
    }
    
    let filename = &args[1];
    let start_offset = if args.len() > 2 {
        args[2].parse::<u64>().unwrap_or(0)
    } else {
        0
    };
    let length = if args.len() > 3 {
        args[3].parse::<usize>().unwrap_or(512)
    } else {
        512
    };
    
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            process::exit(1);
        }
    };
    
    if let Err(e) = file.seek(SeekFrom::Start(start_offset)) {
        eprintln!("Error seeking to offset {}: {}", start_offset, e);
        process::exit(1);
    }
    
    let mut buffer = vec![0u8; length];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    };
    
    println!("Hex dump of {} starting at offset 0x{:04X} ({} bytes):", 
             filename, start_offset, bytes_read);
    println!("─────────────────────────────────────────────────────────────────────────────");
    
    for (i, chunk) in buffer[..bytes_read].chunks(16).enumerate() {
        let offset = start_offset + (i * 16) as u64;
        print!("{:08X}: ", offset);
        
        // Print hex bytes
        for (j, &byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02X} ", byte);
        }
        
        // Pad if less than 16 bytes
        for j in chunk.len()..16 {
            if j == 8 {
                print!(" ");
            }
            print!("   ");
        }
        
        print!(" |");
        
        // Print ASCII representation
        for &byte in chunk {
            if byte >= 32 && byte <= 126 {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        
        println!("|");
    }
    
    // Look for sync words
    println!("\n🔍 Searching for MP3 sync words (0xFFE0-0xFFFF):");
    for i in 0..bytes_read.saturating_sub(1) {
        let word = u16::from_be_bytes([buffer[i], buffer[i + 1]]);
        if word >= 0xFFE0 {
            println!("   Found sync word 0x{:04X} at offset 0x{:04X} ({})", 
                     word, start_offset + i as u64, start_offset + i as u64);
        }
    }
}