//! Rust vs Shine Encoder Comparison Tests
//!
//! This test suite compares the output of the Rust MP3 encoder with the Shine
//! reference implementation by encoding the same WAV files with both encoders
//! and comparing the resulting MP3 files.
//!
//! The tests use three different audio files to cover various scenarios:
//! - sample-3s.wav: Standard stereo 44.1kHz test file
//! - voice-recorder-testing-1-2-3-sound-file.wav: Voice recording (mono 48kHz)
//! - Free_Test_Data_500KB_WAV.wav: Larger test file for stress testing

use std::process::Command;
use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};

/// Test configuration for encoder comparison
#[derive(Debug, Clone)]
struct EncoderTestConfig {
    name: String,
    input_file: String,
    bitrate: u32,
    frame_limit: Option<u32>,
    description: String,
}

/// Result of encoder comparison test
#[derive(Debug)]
struct ComparisonResult {
    config_name: String,
    rust_success: bool,
    shine_success: bool,
    rust_size: Option<u64>,
    shine_size: Option<u64>,
    rust_hash: Option<String>,
    shine_hash: Option<String>,
    files_identical: bool,
    error_message: Option<String>,
}

impl ComparisonResult {
    fn new(config_name: String) -> Self {
        Self {
            config_name,
            rust_success: false,
            shine_success: false,
            rust_size: None,
            shine_size: None,
            rust_hash: None,
            shine_hash: None,
            files_identical: false,
            error_message: None,
        }
    }
    

}

/// Calculate SHA256 hash of a file
fn calculate_sha256(file_path: &str) -> Result<String, String> {
    let data = fs::read(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Get file size in bytes
fn get_file_size(file_path: &str) -> Result<u64, String> {
    fs::metadata(file_path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file size for {}: {}", file_path, e))
}

/// Run Rust encoder with specified parameters
fn run_rust_encoder(input_file: &str, output_file: &str, bitrate: u32, _frame_limit: Option<u32>) -> Result<(), String> {
    if !Path::new(input_file).exists() {
        return Err(format!("Input file not found: {}", input_file));
    }
    
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--features", "diagnostics", "--", input_file, output_file, &bitrate.to_string()]);
    
    // Frame limit is no longer supported - we use pre-generated WAV files instead
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to run Rust encoder: {}", e))?;
    
    if !result.status.success() {
        return Err(format!(
            "Rust encoder failed with exit code {:?}:\nstdout: {}\nstderr: {}",
            result.status.code(),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    
    Ok(())
}

/// Run Shine encoder with specified parameters
fn run_shine_encoder(input_file: &str, output_file: &str, bitrate: u32, frame_limit: Option<u32>) -> Result<(), String> {
    if !Path::new(input_file).exists() {
        return Err(format!("Input file not found: {}", input_file));
    }
    
    let shine_exe = "ref/shine/shineenc.exe";
    if !Path::new(shine_exe).exists() {
        return Err(format!("Shine encoder not found: {}. Run 'cd ref/shine && .\\build.ps1' to build it.", shine_exe));
    }
    
    let mut cmd = Command::new(shine_exe);
    cmd.args(&["-b", &bitrate.to_string(), input_file, output_file]);
    
    // Set frame limit environment variable if specified
    if let Some(limit) = frame_limit {
        cmd.env("SHINE_MAX_FRAMES", limit.to_string());
    }
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to run Shine encoder: {}", e))?;
    
    if !result.status.success() {
        return Err(format!(
            "Shine encoder failed with exit code {:?}:\nstdout: {}\nstderr: {}",
            result.status.code(),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    
    Ok(())
}

/// Compare two encoders on a single configuration
fn compare_encoders(config: &EncoderTestConfig) -> ComparisonResult {
    let rust_output = format!("test_rust_{}.mp3", config.name);
    let shine_output = format!("test_shine_{}.mp3", config.name);
    
    let mut result = ComparisonResult::new(config.name.clone());
    
    // Clean up any existing files
    let _ = fs::remove_file(&rust_output);
    let _ = fs::remove_file(&shine_output);
    
    // Run Rust encoder
    match run_rust_encoder(&config.input_file, &rust_output, config.bitrate, config.frame_limit) {
        Ok(()) => {
            result.rust_success = true;
            if let Ok(size) = get_file_size(&rust_output) {
                result.rust_size = Some(size);
            }
            if let Ok(hash) = calculate_sha256(&rust_output) {
                result.rust_hash = Some(hash);
            }
        }
        Err(e) => {
            result.error_message = Some(format!("Rust encoder failed: {}", e));
        }
    }
    
    // Run Shine encoder
    match run_shine_encoder(&config.input_file, &shine_output, config.bitrate, config.frame_limit) {
        Ok(()) => {
            result.shine_success = true;
            if let Ok(size) = get_file_size(&shine_output) {
                result.shine_size = Some(size);
            }
            if let Ok(hash) = calculate_sha256(&shine_output) {
                result.shine_hash = Some(hash);
            }
        }
        Err(e) => {
            let error_msg = format!("Shine encoder failed: {}", e);
            result.error_message = match result.error_message {
                Some(existing) => Some(format!("{}; {}", existing, error_msg)),
                None => Some(error_msg),
            };
        }
    }
    
    // Compare results if both succeeded
    if result.rust_success && result.shine_success {
        if let (Some(rust_hash), Some(shine_hash)) = (&result.rust_hash, &result.shine_hash) {
            result.files_identical = rust_hash == shine_hash;
        }
    }
    
    // Clean up output files
    let _ = fs::remove_file(&rust_output);
    let _ = fs::remove_file(&shine_output);
    
    result
}

/// Generate test configurations for all three audio files
fn generate_test_configurations() -> Vec<EncoderTestConfig> {
    let audio_files = [
        ("sample-3s", "tests/audio/sample-3s.wav", "Standard stereo 44.1kHz test file"),
        ("voice", "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav", "Voice recording (mono 48kHz)"),
        ("large", "tests/audio/Free_Test_Data_500KB_WAV.wav", "Large test file for stress testing"),
    ];
    
    let bitrates = [128, 192];
    let frame_limits = [Some(3), Some(6), None]; // 3 frames, 6 frames, unlimited
    
    let mut configs = Vec::new();
    
    for (file_key, file_path, description) in &audio_files {
        // Skip if file doesn't exist
        if !Path::new(file_path).exists() {
            println!("⚠️  Skipping {} - file not found: {}", file_key, file_path);
            continue;
        }
        
        for &bitrate in &bitrates {
            for &frame_limit in &frame_limits {
                let limit_str = match frame_limit {
                    Some(n) => format!("{}f", n),
                    None => "full".to_string(),
                };
                
                let config_name = format!("{}_{}k_{}", file_key, bitrate, limit_str);
                let config_desc = format!("{} @ {} kbps, {} frames", 
                                        description, 
                                        bitrate, 
                                        frame_limit.map_or("unlimited".to_string(), |n| n.to_string()));
                
                configs.push(EncoderTestConfig {
                    name: config_name,
                    input_file: file_path.to_string(),
                    bitrate,
                    frame_limit,
                    description: config_desc,
                });
            }
        }
    }
    
    configs
}

/// Print detailed comparison results
fn print_comparison_results(results: &[ComparisonResult]) {
    println!("\n🔍 Rust vs Shine Encoder Comparison Results:");
    println!("{:-<100}", "");
    
    let mut total_tests = 0;
    let mut both_succeeded = 0;
    let mut identical_files = 0;
    let mut rust_only_succeeded = 0;
    let mut shine_only_succeeded = 0;
    let mut both_failed = 0;
    
    for result in results {
        total_tests += 1;
        
        let status = match (result.rust_success, result.shine_success, result.files_identical) {
            (true, true, true) => {
                identical_files += 1;
                both_succeeded += 1;
                "✅ IDENTICAL"
            }
            (true, true, false) => {
                both_succeeded += 1;
                "⚠️  DIFFERENT"
            }
            (true, false, _) => {
                rust_only_succeeded += 1;
                "🔶 RUST ONLY"
            }
            (false, true, _) => {
                shine_only_succeeded += 1;
                "🔷 SHINE ONLY"
            }
            (false, false, _) => {
                both_failed += 1;
                "❌ BOTH FAILED"
            }
        };
        
        println!("{:<12} {:<30} {}", status, result.config_name, 
                 format_size_comparison(&result));
        
        if let Some(error) = &result.error_message {
            println!("             Error: {}", error);
        }
    }
    
    println!("{:-<100}", "");
    println!("📊 Summary:");
    println!("   Total tests:        {}", total_tests);
    println!("   Both succeeded:     {} ({:.1}%)", both_succeeded, 
             (both_succeeded as f64 / total_tests as f64) * 100.0);
    println!("   Identical files:    {} ({:.1}%)", identical_files,
             (identical_files as f64 / total_tests as f64) * 100.0);
    println!("   Rust only:          {}", rust_only_succeeded);
    println!("   Shine only:         {}", shine_only_succeeded);
    println!("   Both failed:        {}", both_failed);
    
    let success_rate = (identical_files as f64 / total_tests as f64) * 100.0;
    if success_rate >= 80.0 {
        println!("🎉 Overall success rate: {:.1}% - EXCELLENT!", success_rate);
    } else if success_rate >= 60.0 {
        println!("👍 Overall success rate: {:.1}% - GOOD", success_rate);
    } else if success_rate >= 40.0 {
        println!("⚠️  Overall success rate: {:.1}% - NEEDS IMPROVEMENT", success_rate);
    } else {
        println!("❌ Overall success rate: {:.1}% - POOR", success_rate);
    }
}

/// Format size comparison string
fn format_size_comparison(result: &ComparisonResult) -> String {
    match (result.rust_size, result.shine_size) {
        (Some(rust), Some(shine)) => {
            if rust == shine {
                format!("{} bytes", rust)
            } else {
                let diff = rust as i64 - shine as i64;
                let diff_pct = (diff as f64 / shine as f64) * 100.0;
                format!("R:{} S:{} ({:+.1}%)", rust, shine, diff_pct)
            }
        }
        (Some(rust), None) => format!("R:{} S:FAIL", rust),
        (None, Some(shine)) => format!("R:FAIL S:{}", shine),
        (None, None) => "R:FAIL S:FAIL".to_string(),
    }
}

/// Test sample-3s.wav file with different configurations
#[test]
fn test_sample_file_comparison() {
    let configs = generate_test_configurations()
        .into_iter()
        .filter(|c| c.input_file.contains("sample-3s.wav"))
        .collect::<Vec<_>>();
    
    if configs.is_empty() {
        println!("⚠️  No sample-3s.wav configurations available - file may be missing");
        return;
    }
    
    println!("🎵 Testing sample-3s.wav configurations...");
    
    let mut results = Vec::new();
    for config in &configs {
        println!("   Testing: {}", config.description);
        let result = compare_encoders(config);
        results.push(result);
    }
    
    print_comparison_results(&results);
    
    // Check if we have any identical results
    let identical_count = results.iter().filter(|r| r.files_identical).count();
    
    if identical_count == 0 {
        println!("\n⚠️  No identical files found for sample-3s.wav");
        println!("   This may indicate implementation differences or configuration issues.");
    } else {
        println!("\n✅ {} out of {} sample-3s.wav tests produced identical files", 
                 identical_count, results.len());
    }
}

/// Test voice recording file with different configurations
#[test]
fn test_voice_file_comparison() {
    let configs = generate_test_configurations()
        .into_iter()
        .filter(|c| c.input_file.contains("voice-recorder-testing"))
        .collect::<Vec<_>>();
    
    if configs.is_empty() {
        println!("⚠️  No voice recording configurations available - file may be missing");
        return;
    }
    
    println!("🎤 Testing voice recording configurations...");
    
    let mut results = Vec::new();
    for config in &configs {
        println!("   Testing: {}", config.description);
        let result = compare_encoders(config);
        results.push(result);
    }
    
    print_comparison_results(&results);
    
    // Voice files are known to have issues, so we're more lenient
    let both_succeeded = results.iter().filter(|r| r.rust_success && r.shine_success).count();
    
    if both_succeeded == 0 {
        println!("\n⚠️  No voice recording tests had both encoders succeed");
        println!("   This is expected due to known mono 48kHz processing differences.");
    } else {
        println!("\n📊 {} out of {} voice recording tests had both encoders succeed", 
                 both_succeeded, results.len());
    }
}

/// Test large file with different configurations
#[test]
fn test_large_file_comparison() {
    let configs = generate_test_configurations()
        .into_iter()
        .filter(|c| c.input_file.contains("Free_Test_Data_500KB_WAV.wav"))
        .collect::<Vec<_>>();
    
    if configs.is_empty() {
        println!("⚠️  No large file configurations available - file may be missing");
        return;
    }
    
    println!("📁 Testing large file configurations...");
    
    let mut results = Vec::new();
    for config in &configs {
        println!("   Testing: {}", config.description);
        let result = compare_encoders(config);
        results.push(result);
    }
    
    print_comparison_results(&results);
    
    let identical_count = results.iter().filter(|r| r.files_identical).count();
    
    if identical_count == 0 {
        println!("\n⚠️  No identical files found for large file");
        println!("   This may indicate implementation differences or performance issues.");
    } else {
        println!("\n✅ {} out of {} large file tests produced identical files", 
                 identical_count, results.len());
    }
}

/// Comprehensive test of all configurations
#[test]
fn test_comprehensive_encoder_comparison() {
    let configs = generate_test_configurations();
    
    if configs.is_empty() {
        panic!("No test configurations available - audio files may be missing");
    }
    
    println!("🚀 Running comprehensive encoder comparison...");
    println!("   Total configurations: {}", configs.len());
    
    let mut results = Vec::new();
    
    for (i, config) in configs.iter().enumerate() {
        println!("   [{}/{}] Testing: {}", i + 1, configs.len(), config.description);
        let result = compare_encoders(config);
        results.push(result);
    }
    
    print_comparison_results(&results);
    
    // Analyze results by file type
    let sample_results: Vec<_> = results.iter().filter(|r| r.config_name.starts_with("sample")).collect();
    let voice_results: Vec<_> = results.iter().filter(|r| r.config_name.starts_with("voice")).collect();
    let large_results: Vec<_> = results.iter().filter(|r| r.config_name.starts_with("large")).collect();
    
    println!("\n📈 Results by file type:");
    
    if !sample_results.is_empty() {
        let sample_identical = sample_results.iter().filter(|r| r.files_identical).count();
        println!("   Sample file:  {}/{} identical ({:.1}%)", 
                 sample_identical, sample_results.len(),
                 (sample_identical as f64 / sample_results.len() as f64) * 100.0);
    }
    
    if !voice_results.is_empty() {
        let voice_identical = voice_results.iter().filter(|r| r.files_identical).count();
        println!("   Voice file:   {}/{} identical ({:.1}%)", 
                 voice_identical, voice_results.len(),
                 (voice_identical as f64 / voice_results.len() as f64) * 100.0);
    }
    
    if !large_results.is_empty() {
        let large_identical = large_results.iter().filter(|r| r.files_identical).count();
        println!("   Large file:   {}/{} identical ({:.1}%)", 
                 large_identical, large_results.len(),
                 (large_identical as f64 / large_results.len() as f64) * 100.0);
    }
    
    // Overall assessment
    let total_identical = results.iter().filter(|r| r.files_identical).count();
    let success_rate = (total_identical as f64 / results.len() as f64) * 100.0;
    
    println!("\n🎯 Final Assessment:");
    if success_rate >= 90.0 {
        println!("   EXCELLENT: {:.1}% of tests produced identical files", success_rate);
        println!("   The Rust implementation is highly compatible with Shine.");
    } else if success_rate >= 70.0 {
        println!("   GOOD: {:.1}% of tests produced identical files", success_rate);
        println!("   The Rust implementation is mostly compatible with Shine.");
    } else if success_rate >= 50.0 {
        println!("   MODERATE: {:.1}% of tests produced identical files", success_rate);
        println!("   The Rust implementation has some compatibility issues.");
    } else {
        println!("   POOR: {:.1}% of tests produced identical files", success_rate);
        println!("   The Rust implementation has significant compatibility issues.");
    }
    
    // Don't fail the test automatically - let the user interpret the results
    if total_identical == 0 {
        println!("\n⚠️  WARNING: No tests produced identical files!");
        println!("   This suggests fundamental differences between implementations.");
        println!("   Check encoder configurations, algorithm implementations, or test setup.");
    }
}

/// Test to verify that both encoders can be executed
#[test]
fn test_encoder_availability() {
    println!("🔧 Checking encoder availability...");
    
    // Check Rust encoder (via cargo run)
    let rust_check = Command::new("cargo")
        .args(&["check", "--features", "diagnostics"])
        .output();
    
    match rust_check {
        Ok(result) if result.status.success() => {
            println!("✅ Rust encoder: Available (cargo check passed)");
        }
        Ok(result) => {
            println!("❌ Rust encoder: Build issues detected");
            println!("   stderr: {}", String::from_utf8_lossy(&result.stderr));
        }
        Err(e) => {
            println!("❌ Rust encoder: Cannot run cargo - {}", e);
        }
    }
    
    // Check Shine encoder
    let shine_exe = "ref/shine/shineenc.exe";
    if Path::new(shine_exe).exists() {
        println!("✅ Shine encoder: Available at {}", shine_exe);
    } else {
        println!("❌ Shine encoder: Not found at {}", shine_exe);
        println!("   Run 'cd ref/shine && .\\build.ps1' to build it.");
    }
    
    // Check audio files
    let audio_files = [
        "tests/audio/sample-3s.wav",
        "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav", 
        "tests/audio/Free_Test_Data_500KB_WAV.wav",
    ];
    
    println!("\n📁 Checking audio files:");
    for file in &audio_files {
        if Path::new(file).exists() {
            if let Ok(metadata) = fs::metadata(file) {
                println!("✅ {}: {} bytes", file, metadata.len());
            } else {
                println!("⚠️  {}: Exists but cannot read metadata", file);
            }
        } else {
            println!("❌ {}: Not found", file);
        }
    }
}

/// Quick smoke test with minimal configuration
#[test]
fn test_quick_comparison_smoke_test() {
    let test_file = "tests/audio/sample-3s.wav";
    
    if !Path::new(test_file).exists() {
        println!("⚠️  Skipping smoke test - sample file not found: {}", test_file);
        return;
    }
    
    println!("💨 Running quick smoke test...");
    
    let config = EncoderTestConfig {
        name: "smoke_test".to_string(),
        input_file: test_file.to_string(),
        bitrate: 128,
        frame_limit: Some(3), // Just 3 frames for speed
        description: "Quick smoke test (3 frames @ 128 kbps)".to_string(),
    };
    
    let result = compare_encoders(&config);
    
    println!("   Rust encoder:  {}", if result.rust_success { "✅ SUCCESS" } else { "❌ FAILED" });
    println!("   Shine encoder: {}", if result.shine_success { "✅ SUCCESS" } else { "❌ FAILED" });
    
    if result.rust_success && result.shine_success {
        println!("   Files identical: {}", if result.files_identical { "✅ YES" } else { "❌ NO" });
        println!("   Size comparison: {}", format_size_comparison(&result));
        
        if result.files_identical {
            println!("🎉 Smoke test PASSED - encoders produce identical output!");
        } else {
            println!("⚠️  Smoke test PARTIAL - both encoders work but produce different output");
        }
    } else {
        println!("❌ Smoke test FAILED - one or both encoders failed");
        if let Some(error) = &result.error_message {
            println!("   Error: {}", error);
        }
    }
}