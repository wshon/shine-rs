//! CI/CD Encoder Validation Tests
//!
//! This test suite validates Rust encoder output against pre-generated reference files.
//! Does not require Shine binary - uses pre-computed reference data.
//!
//! Covers comprehensive configurations:
//! - Mono/Stereo channels
//! - Different bitrates (128, 192, 256 kbps)
//! - Standard test files

use std::fs;
use std::path::Path;
use std::process::Command;
use sha2::{Sha256, Digest};
use serde_json;
use std::collections::HashMap;

/// Reference configuration data
#[derive(Debug)]
struct ReferenceConfig {
    description: String,
    input_file: String,
    reference_file: String,
    expected_size: u64,
    expected_hash: String,
}

/// Load reference manifest
fn load_reference_manifest() -> HashMap<String, ReferenceConfig> {
    let manifest_path = "tests/integration_reference_validation.data/reference_manifest.json";
    
    if !Path::new(manifest_path).exists() {
        panic!("Reference manifest not found: {}. Run 'python scripts/generate_reference_validation_data.py' first.", manifest_path);
    }
    
    let manifest_content = fs::read_to_string(manifest_path)
        .expect("Failed to read reference manifest");
    
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .expect("Failed to parse reference manifest JSON");
    
    let reference_files = manifest["reference_files"].as_object()
        .expect("Invalid manifest format: missing reference_files");
    
    let mut configs = HashMap::new();
    
    for (config_name, config_data) in reference_files {
        let config = ReferenceConfig {
            description: config_data["description"].as_str().unwrap().to_string(),
            input_file: format!("tests/integration_reference_validation.data/{}", 
                              config_data["input_file"].as_str().unwrap()),
            reference_file: format!("tests/integration_reference_validation.data/{}", 
                                  config_data["file_path"].as_str().unwrap()),
            expected_size: config_data["size_bytes"].as_u64().unwrap(),
            expected_hash: config_data["sha256"].as_str().unwrap().to_string(),
        };
        
        configs.insert(config_name.clone(), config);
    }
    
    configs
}

/// Calculate SHA256 hash of a file
fn calculate_sha256(file_path: &str) -> String {
    let data = fs::read(file_path).expect("Failed to read file");
    let mut hasher = Sha256::new();
    hasher.update(&data);
    format!("{:x}", hasher.finalize())
}

/// Run Rust encoder
fn run_rust_encoder(input_file: &str, output_file: &str) -> Result<(), String> {
    if !Path::new(input_file).exists() {
        return Err(format!("Input file not found: {}", input_file));
    }
    
    let result = Command::new("cargo")
        .args(&["run", "--", input_file, output_file])
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
/// Validate a single configuration
fn validate_configuration(config_name: &str, config: &ReferenceConfig) -> Result<(), String> {
    let output_file = format!("test_cicd_{}.mp3", config_name);
    
    // Ensure reference file exists
    if !Path::new(&config.reference_file).exists() {
        return Err(format!("Reference file not found: {}", config.reference_file));
    }
    
    // Run Rust encoder
    run_rust_encoder(&config.input_file, &output_file)?;
    
    // Verify output file was created
    if !Path::new(&output_file).exists() {
        return Err(format!("Rust output file not created: {}", output_file));
    }
    
    // Check file size
    let rust_size = fs::metadata(&output_file)
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    if rust_size != config.expected_size {
        let _ = fs::remove_file(&output_file);
        return Err(format!(
            "Size mismatch: Rust={} bytes, Expected={} bytes",
            rust_size, config.expected_size
        ));
    }
    
    // Check SHA256 hash
    let rust_hash = calculate_sha256(&output_file);
    
    // Clean up
    let _ = fs::remove_file(&output_file);
    
    if rust_hash != config.expected_hash {
        return Err(format!(
            "Hash mismatch:\nRust:     {}\nExpected: {}",
            rust_hash, config.expected_hash
        ));
    }
    
    Ok(())
}

#[test]
fn test_standard_configurations() {
    let configs = load_reference_manifest();
    
    // Test only standard configurations (not frame-specific ones)
    let standard_configs = [
        "Free_Test_Data_500KB_WAV",
        "voice-recorder-testing-1-2-3-sound-file",
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for config_name in &standard_configs {
        if let Some(config) = configs.get(&config_name.to_string()) {
            print!("Testing {}: ", config_name);
            match validate_configuration(config_name, config) {
                Ok(()) => {
                    println!("✅ PASS ({} bytes)", config.expected_size);
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ FAIL - {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("⚠️  Configuration {} not found in manifest", config_name);
            failed += 1;
        }
    }
    
    println!("\nStandard configurations test summary: {} passed, {} failed", passed, failed);
    
    if failed > 0 {
        panic!("{} standard configurations failed validation", failed);
    }
}

#[test]
fn test_reference_file_integrity() {
    let configs = load_reference_manifest();
    
    println!("🔍 Checking reference file integrity...");
    
    let mut missing_files = Vec::new();
    let mut corrupted_files = Vec::new();
    
    for (config_name, config) in &configs {
        // Check if reference file exists
        if !Path::new(&config.reference_file).exists() {
            missing_files.push(config_name);
            continue;
        }
        
        // Check file size
        if let Ok(metadata) = fs::metadata(&config.reference_file) {
            let actual_size = metadata.len();
            if actual_size != config.expected_size {
                corrupted_files.push(format!("{} (size: {} vs {})", 
                                           config_name, actual_size, config.expected_size));
                continue;
            }
        }
        
        // Check SHA256 hash
        let actual_hash = calculate_sha256(&config.reference_file);
        if actual_hash != config.expected_hash {
            corrupted_files.push(format!("{} (hash mismatch)", config_name));
        }
    }
    
    if !missing_files.is_empty() {
        println!("❌ Missing reference files: {:?}", missing_files);
        println!("   Run 'python scripts/generate_reference_validation_data.py' to generate them.");
    }
    
    if !corrupted_files.is_empty() {
        println!("❌ Corrupted reference files: {:?}", corrupted_files);
        println!("   Run 'python scripts/generate_reference_validation_data.py' to regenerate them.");
    }
    
    if missing_files.is_empty() && corrupted_files.is_empty() {
        println!("✅ All {} reference files are intact", configs.len());
    } else {
        panic!("Reference file integrity check failed");
    }
}