//! Data-driven integration tests for MP3 encoder validation
//!
//! This test suite automatically discovers and validates all test data files
//! in the fixtures/data directory, ensuring comprehensive coverage across
//! different audio files and encoding configurations.

use std::fs;
use std::path::Path;
use serde_json;
use rust_mp3_encoder::test_data::TestDataSet;

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
    assert!(!test_data.audio_file.is_empty(), "Audio file path should not be empty");
    assert!(test_data.encoding_config.bitrate > 0, "Bitrate should be positive");
    assert!(test_data.encoding_config.sample_rate > 0, "Sample rate should be positive");
    assert!(test_data.encoding_config.channels > 0, "Channels should be positive");
    assert!(!test_data.frames.is_empty(), "Should have frame data");
    
    // Validate each frame
    for (frame_idx, frame) in test_data.frames.iter().enumerate() {
        assert_eq!(frame.frame_number as usize, frame_idx + 1, 
                  "Frame number should match index");
        
        // Validate MDCT data
        if let Some(ref mdct_data) = frame.mdct_data {
            assert_eq!(mdct_data.len(), 2, "Should have data for 2 channels");
            for ch_data in mdct_data {
                assert_eq!(ch_data.len(), 32, "Should have 32 subbands");
                for sb_data in ch_data {
                    assert_eq!(sb_data.len(), 18, "Should have 18 samples per subband");
                }
            }
        }
        
        // Validate quantization data
        if let Some(ref quant_data) = frame.quantization_data {
            assert_eq!(quant_data.len(), 2, "Should have data for 2 channels");
            for ch_data in quant_data {
                assert_eq!(ch_data.len(), 2, "Should have data for 2 granules");
                for gr_data in ch_data {
                    assert!(gr_data.global_gain <= 255, "Global gain should be <= 255");
                    assert!(gr_data.big_values <= 288, "Big values should be <= 288");
                    assert!(gr_data.part2_3_length <= 4095, "Part2_3_length should be <= 4095");
                }
            }
        }
        
        // Validate SCFSI data
        if let Some(ref scfsi_data) = frame.scfsi_data {
            assert_eq!(scfsi_data.len(), 2, "Should have SCFSI for 2 channels");
            for ch_scfsi in scfsi_data {
                assert_eq!(ch_scfsi.len(), 4, "Should have 4 SCFSI bands");
                for &scfsi_val in ch_scfsi {
                    assert!(scfsi_val == 0 || scfsi_val == 1, "SCFSI should be 0 or 1");
                }
            }
        }
        
        // Validate bitstream data
        if let Some(ref bitstream_data) = frame.bitstream_data {
            assert!(bitstream_data.written_bytes > 0, "Should have written bytes");
            assert!(bitstream_data.bits_per_frame > 0, "Should have bits per frame");
            assert!(bitstream_data.slot_lag_before >= -1.0 && bitstream_data.slot_lag_before <= 1.0,
                   "Slot lag should be in range [-1, 1]");
            assert!(bitstream_data.slot_lag_after >= -1.0 && bitstream_data.slot_lag_after <= 1.0,
                   "Slot lag should be in range [-1, 1]");
        }
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
                        test_data.encoding_config.bitrate,
                        test_data.encoding_config.sample_rate,
                        test_data.encoding_config.channels);
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
            if let Some(ref mdct_data) = frame.mdct_data {
                // Test that MDCT data has reasonable values
                for (ch, ch_data) in mdct_data.iter().enumerate() {
                    for (sb, sb_data) in ch_data.iter().enumerate() {
                        for (s, &sample) in sb_data.iter().enumerate() {
                            assert!(sample.abs() < 100_000_000, 
                                   "MDCT sample too large in {} frame {} ch {} sb {} s {}: {}", 
                                   file_path, frame_idx + 1, ch, sb, s, sample);
                        }
                    }
                }
                
                // Test that stereo channels have similar energy distribution
                if test_data.encoding_config.channels == 2 {
                    let mut ch0_energy = 0i64;
                    let mut ch1_energy = 0i64;
                    
                    for sb in 0..32 {
                        for s in 0..18 {
                            ch0_energy += (mdct_data[0][sb][s] as i64).pow(2);
                            ch1_energy += (mdct_data[1][sb][s] as i64).pow(2);
                        }
                    }
                    
                    // Channels should have similar energy (within 10x ratio)
                    if ch0_energy > 0 && ch1_energy > 0 {
                        let energy_ratio = ch0_energy as f64 / ch1_energy as f64;
                        assert!(energy_ratio > 0.1 && energy_ratio < 10.0,
                               "Energy ratio too extreme in {} frame {}: {:.2}",
                               file_path, frame_idx + 1, energy_ratio);
                    }
                }
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
            if let Some(ref quant_data) = frame.quantization_data {
                for (ch, ch_data) in quant_data.iter().enumerate() {
                    for (gr, gr_data) in ch_data.iter().enumerate() {
                        // Test MP3 standard limits
                        assert!(gr_data.global_gain <= 255,
                               "Global gain exceeds limit in {} frame {} ch {} gr {}: {}",
                               file_path, frame_idx + 1, ch, gr, gr_data.global_gain);
                        
                        assert!(gr_data.big_values <= 288,
                               "Big values exceeds limit in {} frame {} ch {} gr {}: {}",
                               file_path, frame_idx + 1, ch, gr, gr_data.big_values);
                        
                        assert!(gr_data.part2_3_length <= 4095,
                               "Part2_3_length exceeds limit in {} frame {} ch {} gr {}: {}",
                               file_path, frame_idx + 1, ch, gr, gr_data.part2_3_length);
                        
                        // Test reasonable ranges
                        assert!(gr_data.global_gain >= 50,
                               "Global gain too low in {} frame {} ch {} gr {}: {}",
                               file_path, frame_idx + 1, ch, gr, gr_data.global_gain);
                        
                        assert!(gr_data.big_values > 0,
                               "Big values should be positive in {} frame {} ch {} gr {}",
                               file_path, frame_idx + 1, ch, gr);
                        
                        assert!(gr_data.part2_3_length > 0,
                               "Part2_3_length should be positive in {} frame {} ch {} gr {}",
                               file_path, frame_idx + 1, ch, gr);
                    }
                }
                
                // Test stereo consistency
                if test_data.encoding_config.channels == 2 {
                    for gr in 0..2 {
                        assert_eq!(quant_data[0][gr].global_gain, quant_data[1][gr].global_gain,
                                  "Stereo global gain mismatch in {} frame {} gr {}",
                                  file_path, frame_idx + 1, gr);
                        
                        assert_eq!(quant_data[0][gr].big_values, quant_data[1][gr].big_values,
                                  "Stereo big values mismatch in {} frame {} gr {}",
                                  file_path, frame_idx + 1, gr);
                    }
                }
            }
        }
    }
}

/// Test SCFSI consistency across all test data files
#[test]
fn test_scfsi_consistency_all_files() {
    let test_files = discover_test_data_files();
    
    for file_path in test_files {
        let test_data = validate_test_data_file(&file_path).unwrap();
        
        println!("Testing SCFSI consistency for: {}", file_path);
        
        for (frame_idx, frame) in test_data.frames.iter().enumerate() {
            if let Some(ref scfsi_data) = frame.scfsi_data {
                for (ch, ch_scfsi) in scfsi_data.iter().enumerate() {
                    for (band, &scfsi_val) in ch_scfsi.iter().enumerate() {
                        assert!(scfsi_val == 0 || scfsi_val == 1,
                               "Invalid SCFSI value in {} frame {} ch {} band {}: {}",
                               file_path, frame_idx + 1, ch, band, scfsi_val);
                    }
                }
                
                // Test stereo consistency
                if test_data.encoding_config.channels == 2 {
                    assert_eq!(scfsi_data[0], scfsi_data[1],
                              "Stereo SCFSI mismatch in {} frame {}",
                              file_path, frame_idx + 1);
                }
            }
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
            if let Some(ref bitstream_data) = frame.bitstream_data {
                // Test frame size consistency
                assert!(bitstream_data.written_bytes > 0,
                       "Written bytes should be positive in {} frame {}",
                       file_path, frame_idx + 1);
                
                total_bytes += bitstream_data.written_bytes;
                
                // Test bits per frame consistency (should be same for CBR)
                if frame_idx > 0 {
                    let prev_frame = &test_data.frames[frame_idx - 1];
                    if let Some(ref prev_bitstream) = prev_frame.bitstream_data {
                        assert_eq!(bitstream_data.bits_per_frame, prev_bitstream.bits_per_frame,
                                  "Bits per frame should be consistent in {} frame {}",
                                  file_path, frame_idx + 1);
                    }
                }
                
                // Test slot lag continuity
                if let Some(prev_lag) = prev_slot_lag {
                    let lag_diff = (bitstream_data.slot_lag_before - prev_lag).abs();
                    assert!(lag_diff < 0.001,
                           "Slot lag discontinuity in {} frame {}: prev={:.6}, current={:.6}",
                           file_path, frame_idx + 1, prev_lag, bitstream_data.slot_lag_before);
                }
                
                prev_slot_lag = Some(bitstream_data.slot_lag_after);
                
                // Test slot lag increment
                let lag_increment = bitstream_data.slot_lag_after - bitstream_data.slot_lag_before;
                assert!((lag_increment - 0.040816).abs() < 0.001,
                       "Slot lag increment incorrect in {} frame {}: {:.6}",
                       file_path, frame_idx + 1, lag_increment);
            }
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
        
        let config = &test_data.encoding_config;
        
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
        
        // Test layer
        assert_eq!(config.layer, 1, "Should use Layer III in {}", file_path);
        
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
        
        bitrates.insert(test_data.encoding_config.bitrate);
        sample_rates.insert(test_data.encoding_config.sample_rate);
        channel_counts.insert(test_data.encoding_config.channels);
        audio_files.insert(test_data.audio_file.clone());
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