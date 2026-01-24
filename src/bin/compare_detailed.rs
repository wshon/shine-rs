use std::fs::File;
use std::io::{Read, BufReader};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file1.mp3> <file2.mp3>", args[0]);
        return Ok(());
    }

    let file1_path = &args[1];
    let file2_path = &args[2];

    // Read both files
    let mut file1 = BufReader::new(File::open(file1_path)?);
    let mut file2 = BufReader::new(File::open(file2_path)?);

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();

    file1.read_to_end(&mut buf1)?;
    file2.read_to_end(&mut buf2)?;

    println!("File 1 ({}) size: {} bytes", file1_path, buf1.len());
    println!("File 2 ({}) size: {} bytes", file2_path, buf2.len());
    println!("Size difference: {} bytes", buf1.len() as i32 - buf2.len() as i32);

    // Find first difference
    let min_len = std::cmp::min(buf1.len(), buf2.len());
    let mut first_diff = None;
    for i in 0..min_len {
        if buf1[i] != buf2[i] {
            first_diff = Some(i);
            break;
        }
    }

    if let Some(diff_pos) = first_diff {
        println!("First difference at byte {}: 0x{:02X} vs 0x{:02X}", diff_pos, buf1[diff_pos], buf2[diff_pos]);
        
        // Show context around first difference
        let start = diff_pos.saturating_sub(16);
        let end = std::cmp::min(diff_pos + 16, min_len);
        
        println!("\nContext around first difference:");
        println!("File 1:");
        for i in (start..end).step_by(16) {
            print!("{:04X}: ", i);
            for j in 0..16 {
                if i + j < buf1.len() && i + j < end {
                    if i + j == diff_pos {
                        print!("[{:02X}] ", buf1[i + j]);
                    } else {
                        print!("{:02X} ", buf1[i + j]);
                    }
                } else {
                    print!("   ");
                }
            }
            println!();
        }
        
        println!("File 2:");
        for i in (start..end).step_by(16) {
            print!("{:04X}: ", i);
            for j in 0..16 {
                if i + j < buf2.len() && i + j < end {
                    if i + j == diff_pos {
                        print!("[{:02X}] ", buf2[i + j]);
                    } else {
                        print!("{:02X} ", buf2[i + j]);
                    }
                } else {
                    print!("   ");
                }
            }
            println!();
        }
    } else {
        println!("Files are identical up to {} bytes", min_len);
        
        // Check if one file is longer
        if buf1.len() != buf2.len() {
            let (shorter, longer, shorter_name, longer_name) = if buf1.len() < buf2.len() {
                (&buf1, &buf2, file1_path, file2_path)
            } else {
                (&buf2, &buf1, file2_path, file1_path)
            };

            println!("\n{} is {} bytes longer than {}", longer_name, longer.len() - shorter.len(), shorter_name);
            println!("Extra bytes in {}:", longer_name);
            
            let extra_start = shorter.len();
            let extra_end = std::cmp::min(extra_start + 64, longer.len());
            
            for i in (extra_start..extra_end).step_by(16) {
                print!("{:04X}: ", i);
                for j in 0..16 {
                    if i + j < longer.len() {
                        print!("{:02X} ", longer[i + j]);
                    } else {
                        print!("   ");
                    }
                }
                println!();
            }
            
            // Check if extra bytes are all zeros
            let extra_bytes = &longer[extra_start..];
            let all_zeros = extra_bytes.iter().all(|&b| b == 0);
            println!("Extra bytes are all zeros: {}", all_zeros);
        } else {
            println!("Files are completely identical!");
        }
    }

    Ok(())
}