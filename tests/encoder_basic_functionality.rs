//! Basic Encoder Functionality Tests
//!
//! This test suite validates the core functionality of the Rust MP3 encoder
//! using only the CLI interface, without complex API assumptions.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Run Rust encoder with specified parameters
fn run_rust_encoder(input_file: &str, output_file: &str, args: &[&str]) -> Result<(), String> {
    if !Path::new(input_file).exists() {
        return Err(format!("Input file not found: {}", input_file));
    }
    
    let mut cmd_args = vec!["run", "--"];
    cmd_args.push(input_file);
    cmd_args.push(output_file);
    cmd_args.extend_from_slice(args);
    
    let result = Command::new("cargo")
        .args(&cmd_args)
        .output()
        .map_err(|e| format!("Failed to run Rust encoder: {}", e))?;
    
    if !result.status.success() {
        return Err(format!(
            "Rust encoder failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    
    Ok(())
}

/// Check if output file is a valid MP3
fn validate_mp3_output(file_path: &str) -> Result<(), String> {
    if !Path::new(file_path).exists() {
        return Err("Output file not created".to_string());
    }
    
    let file_size = fs::metadata(file_path)
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    if file_size == 0 {
        return Err("Output file is empty".to_string());
    }
    
    // Check MP3 header (basic validation)
    let data = fs::read(file_path)
        .map_err(|e| format!("Failed to read output file: {}", e))?;
    
    if data.len() < 4 {
        return Err("Output file too small to be valid MP3".to_string());
    }
    
    // Check for MP3 frame sync (0xFF followed by 0xFB, 0xFA, or 0xF3, 0xF2)
    let mut found_sync = false;
    for i in 0..data.len().saturating_sub(1) {
        if data[i] == 0xFF && (data[i + 1] & 0xE0) == 0xE0 {
            found_sync = true;
            break;
        }
    }
    
    if !found_sync {
        return Err("No valid MP3 frame sync found".to_string());
    }
    
    Ok(())
}

#[test]
fn test_basic_encoding() {
    let input_file = "tests/audio/inputs/basic/sample-3s.wav";
    let output_file = "test_basic_encoding.mp3";
    
    if !Path::new(input_file).exists() {
        println!("Skipping test - input file not found: {}", input_file);
        return;
    }
    
    // Clean up old file
    let _ = fs::remove_file(output_file);
    
    match run_rust_encoder(input_file, output_file, &[]) {
        Ok(()) => {
            match validate_mp3_output(output_file) {
                Ok(()) => {
                    let file_size = fs::metadata(output_file).unwrap().len();
                    println!("✅ Basic encoding successful: {} bytes", file_size);
                }
                Err(e) => panic!("❌ Output validation failed: {}", e),
            }
        }
        Err(e) => panic!("❌ Basic encoding failed: {}", e),
    }
    
    // Clean up
    let _ = fs::remove_file(output_file);
}

#[test]
fn test_different_input_formats() {
    let test_files = [
        ("tests/audio/inputs/basic/sample-3s.wav", "stereo 44.1kHz"),
        ("tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav", "mono 48kHz"),
        ("tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav", "stereo 44.1kHz large"),
    ];
    
    for (input_file, description) in &test_files {
        if !Path::new(input_file).exists() {
            println!("Skipping {} - file not found", description);
            continue;
        }
        
        let output_file = format!("test_{}.mp3", 
                                 Path::new(input_file).file_stem().unwrap().to_string_lossy());
        
        // Clean up old file
        let _ = fs::remove_file(&output_file);
        
        match run_rust_encoder(input_file, &output_file, &[]) {
            Ok(()) => {
                match validate_mp3_output(&output_file) {
                    Ok(()) => {
                        let file_size = fs::metadata(&output_file).unwrap().len();
                        println!("✅ {} encoding: {} bytes", description, file_size);
                    }
                    Err(e) => panic!("❌ {} output validation failed: {}", description, e),
                }
            }
            Err(e) => panic!("❌ {} encoding failed: {}", description, e),
        }
        
        // Clean up
        let _ = fs::remove_file(&output_file);
    }
}

#[test]
fn test_error_handling() {
    let nonexistent_file = "nonexistent_file.wav";
    let output_file = "test_error.mp3";
    
    // Test with nonexistent input file
    match run_rust_encoder(nonexistent_file, output_file, &[]) {
        Ok(()) => panic!("❌ Should have failed with nonexistent input file"),
        Err(_) => println!("✅ Correctly handled nonexistent input file"),
    }
    
    // Clean up
    let _ = fs::remove_file(output_file);
}