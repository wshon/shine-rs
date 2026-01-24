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

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();

    file1.read_to_end(&mut buf1)?;
    file2.read_to_end(&mut buf2)?;

    println!("File 1 size: {} bytes", buf1.len());
    println!("File 2 size: {} bytes", buf2.len());

    let min_len = buf1.len().min(buf2.len());
    let mut diff_count = 0;

    for i in 0..min_len {
        if buf1[i] != buf2[i] {
            if diff_count < 20 {  // Show first 20 differences
                println!("Difference at offset 0x{:04X}: 0x{:02X} vs 0x{:02X}", i, buf1[i], buf2[i]);
            }
            diff_count += 1;
        }
    }

    if buf1.len() != buf2.len() {
        println!("Files have different sizes: {} vs {}", buf1.len(), buf2.len());
    }

    if diff_count == 0 && buf1.len() == buf2.len() {
        println!("Files are identical");
    } else {
        println!("Total differences: {}", diff_count);
    }

    Ok(())
}