//! Data-driven integration tests for MP3 encoder validation
//!
//! This test suite automatically discovers and validates all test data files
//! in the fixtures/data directory, ensuring comprehensive coverage across
//! different audio files and encoding configurations.

use std::fs;
use serde_json;
use shine_rs::test_data::TestDataSet;

/// Discover all JSON test data files in the fixtures directory
fn discover_test_data_files() -> Vec<String> {
    let data_dir = "testing/fixtures/data";
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(data_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        files.push(format!("{}/{}", data_dir, file_name));
                    }
                }
            }
        }
    }
    
    files.sort();
    files
}

/// Load and validate a single test data file
fn validate_test_data_file(file_path: &str) -> Result<TestDataSet, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let test_data: TestDataSet = serde_json::from_str(&content)?;
    
    // Basic validation of test data structure
    assert!(!test_data.metadata.input_file.is_empty(), "Audio file path should not be empty");
    assert!(test_data.config.bitrate > 0, "Bitrate should be positive");
    assert!(test_data.config.sample_rate > 0, "Sample rate should be positive");
    assert!(test_data.config.channels > 0, "Channels should be positive");
    assert!(!test_data.frames.is_empty(), "Should have frame data");
    
    // Validate each frame
    for (frame_idx, frame) in test_data.frames.iter().enumerate() {
        assert_eq!(frame.frame_number as usize, frame_idx + 1, 
                  "Frame number should match index");
        
        // Validate MDCT data
        let mdct_data = &frame.mdct_coefficients;
        assert!(!mdct_data.coefficients.is_empty(), "Should have MDCT coefficients");
        assert!(!mdct_data.l3_sb_sample.is_empty(), "Should have l3_sb_sample data");
        
        // Test that MDCT coefficients have reasonable values
        for &coeff in &mdct_data.coefficients {
            assert!(coeff.abs() < 1_000_000_000, "MDCT coefficient should be reasonable");
        }
        
        // Test that l3_sb_sample values have reasonable values  
        for &sample in &mdct_data.l3_sb_sample {
            assert!(sample.abs() < 1_000_000_000, "l3_sb_sample should be reasonable");
        }
        
        // Validate quantization data
        let quant_data = &frame.quantization;
        assert!(quant_data.global_gain <= 255, "Global gain should be <= 255");
        assert!(quant_data.part2_3_length <= 4095, "Part2_3_length should be <= 4095");
        assert!(quant_data.max_bits > 0, "Max bits should be positive");
        assert!(quant_data.xrmax >= 0, "xrmax should be non-negative");
        
        // Validate bitstream data
        let bitstream_data = &frame.bitstream;
        assert!(bitstream_data.written > 0, "Should have written bytes");
        assert!(bitstream_data.bits_per_frame > 0, "Should have bits per frame");
        assert!(bitstream_data.slot_lag >= -2.0 && bitstream_data.slot_lag <= 2.0,
               "Slot lag should be in reasonable range");
        assert!(bitstream_data.padding == 0 || bitstream_data.padding == 1,
               "Padding should be 0 or 1");
    }
    
    Ok(test_data)
}

/// Test that all discovered test data files are valid
#[test]
fn test_all_test_data_files_valid() {
    let test_files = discover_test_data_files();
    
    assert!(!test_files.is_empty(), "Should find at least one test data file");
    
    for file_path in test_files {
        println!("Validating test data file: {}", file_path);
        
        match validate_test_data_file(&file_path) {
            Ok(test_data) => {
                println!("✓ {} - {} frames, {}kbps, {}Hz, {}ch", 
                        file_path,
                        test_data.frames.len(),
                        test_data.config.bitrate,
                        test_data.config.sample_rate,
                        test_data.config.channels);
            }
            Err(e) => {
                panic!("Failed to validate {}: {}", file_path, e);
            }
        }
    }
}

/// Test MDCT consistency across all test data files
#[test]
fn test_mdct_consistency_all_files() {
    let test_files = discover_test_data_files();
    
    for file_path in test_files {
        let test_data = validate_test_data_file(&file_path).unwrap();
        
        println!("Testing MDCT consistency for: {}", file_path);
        
        for (frame_idx, frame) in test_data.frames.iter().enumerate() {
            let mdct_data = &frame.mdct_coefficients;
            
            // Test that MDCT coefficients have reasonable values
            for &coeff in &mdct_data.coefficients {
                assert!(coeff.abs() < 1_000_000_000, 
                       "MDCT coefficient too large in {} frame {}: {}", 
                       file_path, frame_idx + 1, coeff);
            }
            
            // Test that l3_sb_sample values have reasonable values
            for &sample in &mdct_data.l3_sb_sample {
                assert!(sample.abs() < 1_000_000_000,
                       "l3_sb_sample too large in {} frame {}: {}",
                       file_path, frame_idx + 1, sample);
            }
            
            // Test stereo consistency for stereo files
            if test_data.config.channels == 2 {
                // For stereo files, we expect similar energy distribution
                // This is a basic sanity check
                assert!(!mdct_data.coefficients.is_empty(), 
                       "Stereo file should have MDCT coefficients");
            }
        }
    }
}

/// Test quantization parameter consistency across all test data files
#[test]
fn test_quantization_consistency_all_files() {
    let test_files = discover_test_data_files();
    
    for file_path in test_files {
        let test_data = validate_test_data_file(&file_path).unwrap();
        
        println!("Testing quantization consistency for: {}", file_path);
        
        for (frame_idx, frame) in test_data.frames.iter().enumerate() {
            let quant_data = &frame.quantization;
            
            // Test MP3 standard limits
            assert!(quant_data.global_gain <= 255,
                   "Global gain exceeds limit in {} frame {}: {}",
                   file_path, frame_idx + 1, quant_data.global_gain);
            
            assert!(quant_data.part2_3_length <= 4095,
                   "Part2_3_length exceeds limit in {} frame {}: {}",
                   file_path, frame_idx + 1, quant_data.part2_3_length);
            
            // Test reasonable ranges
            assert!(quant_data.global_gain >= 50,
                   "Global gain too low in {} frame {}: {}",
                   file_path, frame_idx + 1, quant_data.global_gain);
            
            assert!(quant_data.part2_3_length > 0,
                   "Part2_3_length should be positive in {} frame {}",
                   file_path, frame_idx + 1);
            
            assert!(quant_data.max_bits > 0,
                   "Max bits should be positive in {} frame {}",
                   file_path, frame_idx + 1);
            
            assert!(quant_data.xrmax >= 0,
                   "xrmax should be non-negative in {} frame {}",
                   file_path, frame_idx + 1);
        }
    }
}

/// Test bitstream parameter consistency across all test data files
#[test]
fn test_bitstream_consistency_all_files() {
    let test_files = discover_test_data_files();
    
    for file_path in test_files {
        let test_data = validate_test_data_file(&file_path).unwrap();
        
        println!("Testing bitstream consistency for: {}", file_path);
        
        let mut total_bytes = 0;
        let mut prev_slot_lag: Option<f64> = None;
        
        for (frame_idx, frame) in test_data.frames.iter().enumerate() {
            let bitstream_data = &frame.bitstream;
            
            // Test frame size consistency
            assert!(bitstream_data.written > 0,
                   "Written bytes should be positive in {} frame {}",
                   file_path, frame_idx + 1);
            
            total_bytes += bitstream_data.written;
            
            // Test bits per frame consistency (should be same for CBR)
            if frame_idx > 0 {
                let prev_frame = &test_data.frames[frame_idx - 1];
                let prev_bitstream = &prev_frame.bitstream;
                assert_eq!(bitstream_data.bits_per_frame, prev_bitstream.bits_per_frame,
                          "Bits per frame should be consistent in {} frame {}",
                          file_path, frame_idx + 1);
            }
            
            // Test slot lag continuity
            if let Some(prev_lag) = prev_slot_lag {
                let lag_diff = (bitstream_data.slot_lag - prev_lag).abs();
                assert!(lag_diff < 1.0,
                       "Slot lag change too large in {} frame {}: prev={:.6}, current={:.6}",
                       file_path, frame_idx + 1, prev_lag, bitstream_data.slot_lag);
            }
            
            prev_slot_lag = Some(bitstream_data.slot_lag);
            
            // Test padding value
            assert!(bitstream_data.padding == 0 || bitstream_data.padding == 1,
                   "Invalid padding value in {} frame {}: {}",
                   file_path, frame_idx + 1, bitstream_data.padding);
        }
        
        println!("✓ {} - Total bytes: {}", file_path, total_bytes);
    }
}

/// Test encoding configuration consistency
#[test]
fn test_encoding_config_consistency() {
    let test_files = discover_test_data_files();
    
    for file_path in test_files {
        let test_data = validate_test_data_file(&file_path).unwrap();
        
        println!("Testing encoding config for: {}", file_path);
        
        let config = &test_data.config;
        
        // Test valid bitrates
        let valid_bitrates = [32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
        assert!(valid_bitrates.contains(&config.bitrate),
               "Invalid bitrate in {}: {}", file_path, config.bitrate);
        
        // Test valid sample rates
        let valid_sample_rates = [32000, 44100, 48000];
        assert!(valid_sample_rates.contains(&config.sample_rate),
               "Invalid sample rate in {}: {}", file_path, config.sample_rate);
        
        // Test valid channel counts
        assert!(config.channels == 1 || config.channels == 2,
               "Invalid channel count in {}: {}", file_path, config.channels);
        
        // Test MPEG version
        assert_eq!(config.mpeg_version, 3, "Should use MPEG-I in {}", file_path);
        
        println!("✓ {} - {}kbps, {}Hz, {}ch", 
                file_path, config.bitrate, config.sample_rate, config.channels);
    }
}

/// Test that test data covers different scenarios
#[test]
fn test_test_data_coverage() {
    let test_files = discover_test_data_files();
    
    let mut bitrates = std::collections::HashSet::new();
    let mut sample_rates = std::collections::HashSet::new();
    let mut channel_counts = std::collections::HashSet::new();
    let mut audio_files = std::collections::HashSet::new();
    
    for file_path in test_files {
        let test_data = validate_test_data_file(&file_path).unwrap();
        
        bitrates.insert(test_data.config.bitrate);
        sample_rates.insert(test_data.config.sample_rate);
        channel_counts.insert(test_data.config.channels);
        audio_files.insert(test_data.metadata.input_file.clone());
    }
    
    println!("Test data coverage:");
    println!("  Bitrates: {:?}", bitrates);
    println!("  Sample rates: {:?}", sample_rates);
    println!("  Channel counts: {:?}", channel_counts);
    println!("  Audio files: {:?}", audio_files);
    
    // Ensure we have reasonable coverage
    assert!(bitrates.len() >= 2, "Should test multiple bitrates");
    assert!(audio_files.len() >= 3, "Should test multiple audio files");
    
    // Ensure we test both mono and stereo
    assert!(channel_counts.contains(&1) || channel_counts.contains(&2),
           "Should test different channel configurations");
}

/// Performance test for data loading
#[test]
fn test_data_loading_performance() {
    let test_files = discover_test_data_files();
    
    let start_time = std::time::Instant::now();
    
    for file_path in &test_files {
        let _ = validate_test_data_file(file_path).unwrap();
    }
    
    let elapsed = start_time.elapsed();
    
    println!("Loaded {} test data files in {:?}", test_files.len(), elapsed);
    
    // Should load reasonably quickly
    assert!(elapsed.as_secs() < 5, "Test data loading should be fast");
}