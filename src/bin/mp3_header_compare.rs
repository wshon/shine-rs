use std::env;
use std::fs::File;
use std::io::{Read, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        std::process::exit(1);
    }

    let file1_path = &args[1];
    let file2_path = &args[2];

    let mut file1 = BufReader::new(File::open(file1_path)?);
    let mut file2 = BufReader::new(File::open(file2_path)?);

    let mut buf1 = [0u8; 32];
    let mut buf2 = [0u8; 32];

    file1.read_exact(&mut buf1)?;
    file2.read_exact(&mut buf2)?;

    println!("First 32 bytes comparison:");
    println!("Offset  File1   File2   Diff");
    println!("------  ------  ------  ----");
    
    for i in 0..32 {
        let diff = if buf1[i] == buf2[i] { " " } else { "*" };
        println!("0x{:04X}  0x{:02X}    0x{:02X}    {}", i, buf1[i], buf2[i], diff);
    }

    // Check MP3 frame header
    if buf1.len() >= 4 && buf2.len() >= 4 {
        println!("\nMP3 Frame Header Analysis:");
        
        // Frame 1 header
        let header1 = u32::from_be_bytes([buf1[0], buf1[1], buf1[2], buf1[3]]);
        let header2 = u32::from_be_bytes([buf2[0], buf2[1], buf2[2], buf2[3]]);
        
        println!("File1 header: 0x{:08X}", header1);
        println!("File2 header: 0x{:08X}", header2);
        
        if header1 == header2 {
            println!("Frame headers are identical");
        } else {
            println!("Frame headers differ!");
        }
    }

    Ok(())
}