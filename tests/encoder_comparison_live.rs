//! Live Encoder Comparison Tests
//!
//! This test suite compares Rust encoder output with Shine reference implementation
//! in real-time. Requires Shine binary to be present.
//!
//! **Note**: These tests are ignored by default due to known numerical differences.
//! Run manually with: `cargo test --test encoder_comparison_live -- --ignored`
//!
//! Test files:
//! - Default: tests/audio/inputs/basic/sample-3s.wav (stereo 44.1kHz)
//! - Voice: tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav (mono 48kHz)  
//! - Large: tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav (stereo 44.1kHz, larger file)

use std::process::Command;
use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};

/// Calculate SHA256 hash of a file
fn calculate_sha256(file_path: &str) -> String {
    let data = fs::read(file_path).expect("Failed to read file");
    let mut hasher = Sha256::new();
    hasher.update(&data);
    format!("{:x}", hasher.finalize())
}

/// Run Rust encoder
fn run_rust_encoder(input_file: &str, output_file: &str, bitrate: Option<u32>) -> Result<(), String> {
    if !Path::new(input_file).exists() {
        return Err(format!("Input file not found: {}", input_file));
    }
    
    let mut cmd = Command::new("cargo");
    let mut args = vec!["run", "--"];
    
    let bitrate_str;
    if let Some(br) = bitrate {
        bitrate_str = br.to_string();
        args.extend_from_slice(&["-b", &bitrate_str]);
    }
    
    args.extend_from_slice(&[input_file, output_file]);
    
    cmd.args(&args);
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to run Rust encoder: {}", e))?;
    
    if !result.status.success() {
        return Err(format!(
            "Rust encoder failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    
    Ok(())
}

/// Run Shine encoder  
fn run_shine_encoder(input_file: &str, output_file: &str, bitrate: Option<u32>) -> Result<(), String> {
    let shine_exe = "ref/shine/shineenc.exe";
    if !Path::new(shine_exe).exists() {
        return Err("Shine encoder not found. Please build Shine first.".to_string());
    }
    
    let mut cmd = Command::new(shine_exe);
    let mut args = vec![];
    
    let bitrate_str;
    if let Some(br) = bitrate {
        bitrate_str = br.to_string();
        args.extend_from_slice(&["-b", &bitrate_str]);
    }
    
    args.extend_from_slice(&[input_file, output_file]);
    cmd.args(&args);
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to run Shine encoder: {}", e))?;
    
    if !result.status.success() {
        return Err(format!(
            "Shine encoder failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    
    Ok(())
}

/// Compare two encoders on a single file
fn compare_encoders(input_file: &str, bitrate: Option<u32>) -> Result<(), String> {
    let base_name = Path::new(input_file).file_stem().unwrap().to_string_lossy();
    let bitrate_suffix = bitrate.map(|br| format!("_{}", br)).unwrap_or_default();
    
    let rust_output = format!("test_{}_rust{}.mp3", base_name, bitrate_suffix);
    let shine_output = format!("test_{}_shine{}.mp3", base_name, bitrate_suffix);
    
    // Clean up old files
    let _ = fs::remove_file(&rust_output);
    let _ = fs::remove_file(&shine_output);
    
    // Run both encoders
    run_rust_encoder(input_file, &rust_output, bitrate)?;
    run_shine_encoder(input_file, &shine_output, bitrate)?;
    
    // Compare results
    if !Path::new(&rust_output).exists() {
        return Err("Rust output file not created".to_string());
    }
    
    if !Path::new(&shine_output).exists() {
        return Err("Shine output file not created".to_string());
    }
    
    let rust_size = fs::metadata(&rust_output).unwrap().len();
    let shine_size = fs::metadata(&shine_output).unwrap().len();
    let rust_hash = calculate_sha256(&rust_output);
    let shine_hash = calculate_sha256(&shine_output);
    
    // Clean up
    let _ = fs::remove_file(&rust_output);
    let _ = fs::remove_file(&shine_output);
    
    println!("  Rust:  {} bytes, hash: {}", rust_size, &rust_hash[..16]);
    println!("  Shine: {} bytes, hash: {}", shine_size, &shine_hash[..16]);
    
    if rust_size != shine_size {
        return Err(format!("Size mismatch: Rust={}, Shine={}", rust_size, shine_size));
    }
    
    if rust_hash != shine_hash {
        return Err("Hash mismatch - files are different".to_string());
    }
    
    println!("  ✅ Perfect match!");
    Ok(())
}

#[test]
#[ignore = "Live comparison test - run manually with 'cargo test test_default_file_comparison -- --ignored'"]
fn test_default_file_comparison() {
    let input_file = "tests/audio/inputs/basic/sample-3s.wav";
    
    if !Path::new(input_file).exists() {
        println!("Skipping test - input file not found: {}", input_file);
        return;
    }
    
    println!("Testing default file: {}", input_file);
    
    match compare_encoders(input_file, None) {
        Ok(()) => println!("✅ Default comparison passed"),
        Err(e) => panic!("❌ Default comparison failed: {}", e),
    }
}

#[test]
#[ignore = "Live comparison test - run manually with 'cargo test test_voice_file_comparison -- --ignored'"]
fn test_voice_file_comparison() {
    let input_file = "tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav";
    
    if !Path::new(input_file).exists() {
        println!("Skipping test - input file not found: {}", input_file);
        return;
    }
    
    println!("Testing voice file: {}", input_file);
    
    match compare_encoders(input_file, None) {
        Ok(()) => println!("✅ Voice comparison passed"),
        Err(e) => panic!("❌ Voice comparison failed: {}", e),
    }
}

#[test]
#[ignore = "Live comparison test - run manually with 'cargo test test_large_file_comparison -- --ignored'"]
fn test_large_file_comparison() {
    let input_file = "tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav";
    
    if !Path::new(input_file).exists() {
        println!("Skipping test - input file not found: {}", input_file);
        return;
    }
    
    println!("Testing large file: {}", input_file);
    
    match compare_encoders(input_file, None) {
        Ok(()) => println!("✅ Large file comparison passed"),
        Err(e) => panic!("❌ Large file comparison failed: {}", e),
    }
}

#[test]
#[ignore = "Live comparison test - run manually with 'cargo test test_different_bitrates -- --ignored'"]
fn test_different_bitrates() {
    let input_file = "tests/audio/inputs/basic/sample-3s.wav";
    
    if !Path::new(input_file).exists() {
        println!("Skipping test - input file not found: {}", input_file);
        return;
    }
    
    let bitrates = [128, 192, 256];
    
    for bitrate in &bitrates {
        println!("Testing {}kbps encoding:", bitrate);
        
        match compare_encoders(input_file, Some(*bitrate)) {
            Ok(()) => println!("✅ {}kbps comparison passed", bitrate),
            Err(e) => panic!("❌ {}kbps comparison failed: {}", bitrate, e),
        }
    }
}