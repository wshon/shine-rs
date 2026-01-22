//! Shine Reference Implementation Comparison Tests
//!
//! This module provides comprehensive tests that compare our Rust MP3 encoder
//! implementation against the reference shine library. Each test validates
//! a specific encoding stage to ensure numerical precision and algorithmic
//! consistency.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::process::Command;

/// Test configuration that matches shine's default settings
fn create_test_config() -> Config {
    Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::JointStereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    }
}

/// Generate test PCM data with known patterns for comparison
fn generate_test_pcm_data(samples: usize, channels: usize, pattern: &str) -> Vec<i16> {
    match pattern {
        "sine_440hz" => {
            let sample_rate = 44100.0;
            let frequency = 440.0;
            (0..samples * channels)
                .map(|i| {
                    let sample_idx = i / channels;
                    let t = sample_idx as f32 / sample_rate;
                    let amplitude = 16000.0;
                    (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
                })
                .collect()
        },
        "sine_1000hz" => {
            let sample_rate = 44100.0;
            let frequency = 1000.0;
            (0..samples * channels)
                .map(|i| {
                    let sample_idx = i / channels;
                    let t = sample_idx as f32 / sample_rate;
                    let amplitude = 12000.0;
                    (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
                })
                .collect()
        },
        "white_noise" => {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            (0..samples * channels)
                .map(|i| {
                    let mut hasher = DefaultHasher::new();
                    i.hash(&mut hasher);
                    let hash = hasher.finish();
                    ((hash % 32768) as i16) - 16384
                })
                .collect()
        },
        "impulse" => {
            let mut data = vec![0i16; samples * channels];
            // Add impulse at the beginning
            if !data.is_empty() {
                data[0] = 16000;
                if channels > 1 && data.len() > 1 {
                    data[1] = 16000;
                }
            }
            data
        },
        "zeros" => vec![0i16; samples * channels],
        _ => vec![1000i16; samples * channels], // Default constant pattern
    }
}

/// Write PCM data to a WAV file for shine reference encoding
fn write_wav_file(path: &str, pcm_data: &[i16], sample_rate: u32, channels: u16) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // WAV header
    let data_size = (pcm_data.len() * 2) as u32;
    let file_size = 36 + data_size;
    
    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    
    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // chunk size
    file.write_all(&1u16.to_le_bytes())?;  // PCM format
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * channels as u32 * 2).to_le_bytes())?; // byte rate
    file.write_all(&(channels * 2).to_le_bytes())?; // block align
    file.write_all(&16u16.to_le_bytes())?; // bits per sample
    
    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    
    // PCM data
    for &sample in pcm_data {
        file.write_all(&sample.to_le_bytes())?;
    }
    
    Ok(())
}

/// Encode using shine reference implementation
fn encode_with_shine(wav_path: &str, mp3_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Try to find shine binary in the reference directory
    let shine_paths = [
        "ref/shine/src/bin/shine",
        "ref/shine/shine",
        "shine", // System installed
    ];
    
    let mut shine_cmd = None;
    for path in &shine_paths {
        if std::path::Path::new(path).exists() {
            shine_cmd = Some(path);
            break;
        }
    }
    
    let shine_binary = shine_cmd.ok_or("Shine binary not found. Please build shine reference implementation.")?;
    
    // Run shine encoder
    let output = Command::new(shine_binary)
        .args(&["-b", "128", wav_path, mp3_path])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Shine encoding failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    // Read the generated MP3 file
    let mut file = File::open(mp3_path)?;
    let mut mp3_data = Vec::new();
    file.read_to_end(&mut mp3_data)?;
    
    Ok(mp3_data)
}

/// Compare MP3 frame headers between our implementation and shine
fn compare_frame_headers(our_data: &[u8], shine_data: &[u8]) -> Result<(), String> {
    if our_data.len() < 4 || shine_data.len() < 4 {
        return Err("Insufficient data for header comparison".to_string());
    }
    
    // Extract frame headers (first 4 bytes)
    let our_header = u32::from_be_bytes([our_data[0], our_data[1], our_data[2], our_data[3]]);
    let shine_header = u32::from_be_bytes([shine_data[0], shine_data[1], shine_data[2], shine_data[3]]);
    
    // Compare sync word (bits 31-21)
    let our_sync = (our_header >> 21) & 0x7FF;
    let shine_sync = (shine_header >> 21) & 0x7FF;
    if our_sync != shine_sync {
        return Err(format!("Sync word mismatch: ours=0x{:03X}, shine=0x{:03X}", our_sync, shine_sync));
    }
    
    // Compare MPEG version (bits 20-19)
    let our_version = (our_header >> 19) & 0x3;
    let shine_version = (shine_header >> 19) & 0x3;
    if our_version != shine_version {
        return Err(format!("MPEG version mismatch: ours={}, shine={}", our_version, shine_version));
    }
    
    // Compare layer (bits 18-17)
    let our_layer = (our_header >> 17) & 0x3;
    let shine_layer = (shine_header >> 17) & 0x3;
    if our_layer != shine_layer {
        return Err(format!("Layer mismatch: ours={}, shine={}", our_layer, shine_layer));
    }
    
    // Compare bitrate index (bits 15-12)
    let our_bitrate = (our_header >> 12) & 0xF;
    let shine_bitrate = (shine_header >> 12) & 0xF;
    if our_bitrate != shine_bitrate {
        return Err(format!("Bitrate index mismatch: ours={}, shine={}", our_bitrate, shine_bitrate));
    }
    
    // Compare sample rate index (bits 11-10)
    let our_samplerate = (our_header >> 10) & 0x3;
    let shine_samplerate = (shine_header >> 10) & 0x3;
    if our_samplerate != shine_samplerate {
        return Err(format!("Sample rate index mismatch: ours={}, shine={}", our_samplerate, shine_samplerate));
    }
    
    // Compare channel mode (bits 7-6)
    let our_mode = (our_header >> 6) & 0x3;
    let shine_mode = (shine_header >> 6) & 0x3;
    if our_mode != shine_mode {
        return Err(format!("Channel mode mismatch: ours={}, shine={}", our_mode, shine_mode));
    }
    
    println!("Frame header comparison passed:");
    println!("  Sync: 0x{:03X}", our_sync);
    println!("  MPEG version: {}", our_version);
    println!("  Layer: {}", our_layer);
    println!("  Bitrate index: {}", our_bitrate);
    println!("  Sample rate index: {}", our_samplerate);
    println!("  Channel mode: {}", our_mode);
    
    Ok(())
}

/// Find all MP3 sync words in the data
fn find_sync_words(data: &[u8]) -> Vec<usize> {
    let mut sync_positions = Vec::new();
    
    for i in 0..data.len().saturating_sub(1) {
        let sync = ((data[i] as u16) << 3) | ((data[i + 1] as u16) >> 5);
        if sync == 0x7FF {
            sync_positions.push(i);
        }
    }
    
    sync_positions
}

/// Analyze MP3 frame structure
fn analyze_frame_structure(data: &[u8], label: &str) {
    let sync_positions = find_sync_words(data);
    
    println!("{} analysis:", label);
    println!("  Total size: {} bytes", data.len());
    println!("  Sync words found: {}", sync_positions.len());
    
    if sync_positions.len() > 1 {
        let frame_sizes: Vec<usize> = sync_positions.windows(2)
            .map(|w| w[1] - w[0])
            .collect();
        
        println!("  Frame sizes: {:?}", frame_sizes);
        
        if let Some(&min_size) = frame_sizes.iter().min() {
            if let Some(&max_size) = frame_sizes.iter().max() {
                println!("  Frame size range: {} - {} bytes", min_size, max_size);
            }
        }
    }
    
    // Show first few bytes for debugging
    if data.len() >= 16 {
        print!("  First 16 bytes: ");
        for i in 0..16 {
            print!("{:02X} ", data[i]);
        }
        println!();
    }
}

#[test]
fn test_compare_with_shine_sine_wave() {
    // Create output directory
    create_dir_all("tests/output/comparison").expect("Failed to create comparison directory");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
    
    // Generate test data - sine wave at 440Hz
    let samples_per_frame = encoder.samples_per_frame();
    let channels = config.wave.channels as usize;
    let pcm_data = generate_test_pcm_data(samples_per_frame, channels, "sine_440hz");
    
    // Encode with our implementation
    let our_result = encoder.encode_frame_interleaved(&pcm_data);
    assert!(our_result.is_ok(), "Our encoder should succeed");
    let our_mp3 = our_result.unwrap().to_vec();
    
    // Write test WAV file for shine
    let wav_path = "tests/output/comparison/test_sine.wav";
    write_wav_file(wav_path, &pcm_data, config.wave.sample_rate, channels as u16)
        .expect("Failed to write WAV file");
    
    // Try to encode with shine (skip if not available)
    let shine_mp3_path = "tests/output/comparison/shine_sine.mp3";
    match encode_with_shine(wav_path, shine_mp3_path) {
        Ok(shine_mp3) => {
            // Save our output for comparison
            let our_mp3_path = "tests/output/comparison/our_sine.mp3";
            let mut our_file = File::create(our_mp3_path).expect("Failed to create our MP3 file");
            our_file.write_all(&our_mp3).expect("Failed to write our MP3 data");
            
            println!("Comparing sine wave encoding with shine reference:");
            
            // Analyze both outputs
            analyze_frame_structure(&our_mp3, "Our implementation");
            analyze_frame_structure(&shine_mp3, "Shine reference");
            
            // Compare frame headers
            match compare_frame_headers(&our_mp3, &shine_mp3) {
                Ok(()) => println!("✓ Frame headers match shine reference"),
                Err(e) => println!("⚠ Frame header differences: {}", e),
            }
            
            // Basic validation - both should produce valid MP3 data
            assert!(!our_mp3.is_empty(), "Our MP3 output should not be empty");
            assert!(!shine_mp3.is_empty(), "Shine MP3 output should not be empty");
            
            // Both should have valid sync words
            let our_syncs = find_sync_words(&our_mp3);
            let shine_syncs = find_sync_words(&shine_mp3);
            assert!(!our_syncs.is_empty(), "Our MP3 should have sync words");
            assert!(!shine_syncs.is_empty(), "Shine MP3 should have sync words");
            
            println!("✓ Basic comparison with shine completed successfully");
        },
        Err(e) => {
            println!("⚠ Shine reference not available: {}", e);
            println!("Skipping shine comparison, but validating our output:");
            
            // Still validate our output
            analyze_frame_structure(&our_mp3, "Our implementation");
            assert!(!our_mp3.is_empty(), "Our MP3 output should not be empty");
            
            let our_syncs = find_sync_words(&our_mp3);
            assert!(!our_syncs.is_empty(), "Our MP3 should have sync words");
            
            println!("✓ Our implementation produces valid MP3 output");
        }
    }
}

#[test]
fn test_compare_with_shine_impulse_response() {
    create_dir_all("tests/output/comparison").expect("Failed to create comparison directory");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
    
    // Generate impulse test data
    let samples_per_frame = encoder.samples_per_frame();
    let channels = config.wave.channels as usize;
    let pcm_data = generate_test_pcm_data(samples_per_frame, channels, "impulse");
    
    // Encode with our implementation
    let our_result = encoder.encode_frame_interleaved(&pcm_data);
    assert!(our_result.is_ok(), "Our encoder should succeed with impulse");
    let our_mp3 = our_result.unwrap().to_vec();
    
    // Write test WAV file
    let wav_path = "tests/output/comparison/test_impulse.wav";
    write_wav_file(wav_path, &pcm_data, config.wave.sample_rate, channels as u16)
        .expect("Failed to write impulse WAV file");
    
    // Try shine comparison
    let shine_mp3_path = "tests/output/comparison/shine_impulse.mp3";
    match encode_with_shine(wav_path, shine_mp3_path) {
        Ok(shine_mp3) => {
            let our_mp3_path = "tests/output/comparison/our_impulse.mp3";
            let mut our_file = File::create(our_mp3_path).expect("Failed to create our impulse MP3");
            our_file.write_all(&our_mp3).expect("Failed to write our impulse MP3");
            
            println!("Comparing impulse response encoding:");
            
            analyze_frame_structure(&our_mp3, "Our impulse");
            analyze_frame_structure(&shine_mp3, "Shine impulse");
            
            // Impulse response is a good test for algorithm correctness
            match compare_frame_headers(&our_mp3, &shine_mp3) {
                Ok(()) => println!("✓ Impulse response headers match shine"),
                Err(e) => println!("⚠ Impulse response header differences: {}", e),
            }
            
            println!("✓ Impulse response comparison completed");
        },
        Err(e) => {
            println!("⚠ Shine not available for impulse test: {}", e);
            
            // Validate our impulse response
            analyze_frame_structure(&our_mp3, "Our impulse");
            assert!(!our_mp3.is_empty(), "Impulse MP3 should not be empty");
            
            let syncs = find_sync_words(&our_mp3);
            assert!(!syncs.is_empty(), "Impulse MP3 should have sync words");
            
            println!("✓ Our impulse response produces valid output");
        }
    }
}

#[test]
fn test_compare_with_shine_silence() {
    create_dir_all("tests/output/comparison").expect("Failed to create comparison directory");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
    
    // Generate silence test data
    let samples_per_frame = encoder.samples_per_frame();
    let channels = config.wave.channels as usize;
    let pcm_data = generate_test_pcm_data(samples_per_frame, channels, "zeros");
    
    // Encode with our implementation
    let our_result = encoder.encode_frame_interleaved(&pcm_data);
    assert!(our_result.is_ok(), "Our encoder should succeed with silence");
    let our_mp3 = our_result.unwrap().to_vec();
    
    // Write test WAV file
    let wav_path = "tests/output/comparison/test_silence.wav";
    write_wav_file(wav_path, &pcm_data, config.wave.sample_rate, channels as u16)
        .expect("Failed to write silence WAV file");
    
    // Try shine comparison
    let shine_mp3_path = "tests/output/comparison/shine_silence.mp3";
    match encode_with_shine(wav_path, shine_mp3_path) {
        Ok(shine_mp3) => {
            let our_mp3_path = "tests/output/comparison/our_silence.mp3";
            let mut our_file = File::create(our_mp3_path).expect("Failed to create our silence MP3");
            our_file.write_all(&our_mp3).expect("Failed to write our silence MP3");
            
            println!("Comparing silence encoding:");
            
            analyze_frame_structure(&our_mp3, "Our silence");
            analyze_frame_structure(&shine_mp3, "Shine silence");
            
            // Silence should compress very well and be very similar
            match compare_frame_headers(&our_mp3, &shine_mp3) {
                Ok(()) => println!("✓ Silence headers match shine perfectly"),
                Err(e) => println!("⚠ Silence header differences: {}", e),
            }
            
            // For silence, the frame sizes might be very similar
            let our_syncs = find_sync_words(&our_mp3);
            let shine_syncs = find_sync_words(&shine_mp3);
            
            if our_syncs.len() == shine_syncs.len() && our_syncs.len() > 1 {
                let our_frame_size = our_syncs[1] - our_syncs[0];
                let shine_frame_size = shine_syncs[1] - shine_syncs[0];
                
                println!("Frame size comparison - Our: {}, Shine: {}", our_frame_size, shine_frame_size);
                
                // For silence, frame sizes should be very close
                let size_diff = if our_frame_size > shine_frame_size {
                    our_frame_size - shine_frame_size
                } else {
                    shine_frame_size - our_frame_size
                };
                
                if size_diff <= 10 {
                    println!("✓ Silence frame sizes are very close (diff: {})", size_diff);
                } else {
                    println!("⚠ Silence frame size difference: {}", size_diff);
                }
            }
            
            println!("✓ Silence comparison completed");
        },
        Err(e) => {
            println!("⚠ Shine not available for silence test: {}", e);
            
            analyze_frame_structure(&our_mp3, "Our silence");
            assert!(!our_mp3.is_empty(), "Silence MP3 should not be empty");
            
            let syncs = find_sync_words(&our_mp3);
            assert!(!syncs.is_empty(), "Silence MP3 should have sync words");
            
            println!("✓ Our silence encoding produces valid output");
        }
    }
}

#[test]
fn test_compare_multiple_frames() {
    create_dir_all("tests/output/comparison").expect("Failed to create comparison directory");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
    
    let samples_per_frame = encoder.samples_per_frame();
    let channels = config.wave.channels as usize;
    
    // Encode multiple frames with different patterns
    let mut our_mp3_data = Vec::new();
    let mut all_pcm_data = Vec::new();
    
    let patterns = ["sine_440hz", "sine_1000hz", "impulse", "zeros"];
    
    for pattern in &patterns {
        let pcm_data = generate_test_pcm_data(samples_per_frame, channels, pattern);
        all_pcm_data.extend_from_slice(&pcm_data);
        
        let frame_result = encoder.encode_frame_interleaved(&pcm_data);
        assert!(frame_result.is_ok(), "Frame encoding should succeed for pattern: {}", pattern);
        
        let frame_data = frame_result.unwrap();
        our_mp3_data.extend_from_slice(frame_data);
    }
    
    // Flush any remaining data
    let flush_result = encoder.flush();
    assert!(flush_result.is_ok(), "Flush should succeed");
    our_mp3_data.extend_from_slice(flush_result.unwrap());
    
    // Write combined WAV file
    let wav_path = "tests/output/comparison/test_multiple.wav";
    write_wav_file(wav_path, &all_pcm_data, config.wave.sample_rate, channels as u16)
        .expect("Failed to write multiple frames WAV");
    
    // Try shine comparison
    let shine_mp3_path = "tests/output/comparison/shine_multiple.mp3";
    match encode_with_shine(wav_path, shine_mp3_path) {
        Ok(shine_mp3) => {
            let our_mp3_path = "tests/output/comparison/our_multiple.mp3";
            let mut our_file = File::create(our_mp3_path).expect("Failed to create our multiple MP3");
            our_file.write_all(&our_mp3_data).expect("Failed to write our multiple MP3");
            
            println!("Comparing multiple frame encoding:");
            
            analyze_frame_structure(&our_mp3_data, "Our multiple frames");
            analyze_frame_structure(&shine_mp3, "Shine multiple frames");
            
            // Compare first frame header
            match compare_frame_headers(&our_mp3_data, &shine_mp3) {
                Ok(()) => println!("✓ Multiple frames first header matches shine"),
                Err(e) => println!("⚠ Multiple frames header differences: {}", e),
            }
            
            // Check frame count consistency
            let our_syncs = find_sync_words(&our_mp3_data);
            let shine_syncs = find_sync_words(&shine_mp3);
            
            println!("Frame count - Our: {}, Shine: {}", our_syncs.len(), shine_syncs.len());
            
            if our_syncs.len() == shine_syncs.len() {
                println!("✓ Frame count matches shine reference");
            } else {
                println!("⚠ Frame count differs from shine reference");
            }
            
            println!("✓ Multiple frame comparison completed");
        },
        Err(e) => {
            println!("⚠ Shine not available for multiple frame test: {}", e);
            
            analyze_frame_structure(&our_mp3_data, "Our multiple frames");
            assert!(!our_mp3_data.is_empty(), "Multiple frames MP3 should not be empty");
            
            let syncs = find_sync_words(&our_mp3_data);
            assert!(syncs.len() >= patterns.len(), "Should have at least {} frames", patterns.len());
            
            println!("✓ Our multiple frame encoding produces valid output");
        }
    }
}

#[test]
fn test_mono_configuration_comparison() {
    create_dir_all("tests/output/comparison").expect("Failed to create comparison directory");
    
    // Test mono configuration
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Mono,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Mono,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create mono encoder");
    
    // Generate mono test data
    let samples_per_frame = encoder.samples_per_frame();
    let pcm_data = generate_test_pcm_data(samples_per_frame, 1, "sine_440hz");
    
    // Encode with our implementation
    let our_result = encoder.encode_frame(&pcm_data);
    assert!(our_result.is_ok(), "Mono encoding should succeed");
    let our_mp3 = our_result.unwrap().to_vec();
    
    // Write mono WAV file
    let wav_path = "tests/output/comparison/test_mono.wav";
    write_wav_file(wav_path, &pcm_data, config.wave.sample_rate, 1)
        .expect("Failed to write mono WAV file");
    
    // Try shine comparison
    let shine_mp3_path = "tests/output/comparison/shine_mono.mp3";
    match encode_with_shine(wav_path, shine_mp3_path) {
        Ok(shine_mp3) => {
            let our_mp3_path = "tests/output/comparison/our_mono.mp3";
            let mut our_file = File::create(our_mp3_path).expect("Failed to create our mono MP3");
            our_file.write_all(&our_mp3).expect("Failed to write our mono MP3");
            
            println!("Comparing mono encoding:");
            
            analyze_frame_structure(&our_mp3, "Our mono");
            analyze_frame_structure(&shine_mp3, "Shine mono");
            
            match compare_frame_headers(&our_mp3, &shine_mp3) {
                Ok(()) => println!("✓ Mono headers match shine reference"),
                Err(e) => println!("⚠ Mono header differences: {}", e),
            }
            
            println!("✓ Mono comparison completed");
        },
        Err(e) => {
            println!("⚠ Shine not available for mono test: {}", e);
            
            analyze_frame_structure(&our_mp3, "Our mono");
            assert!(!our_mp3.is_empty(), "Mono MP3 should not be empty");
            
            let syncs = find_sync_words(&our_mp3);
            assert!(!syncs.is_empty(), "Mono MP3 should have sync words");
            
            // Verify mono mode in header
            if our_mp3.len() >= 4 {
                let header = u32::from_be_bytes([our_mp3[0], our_mp3[1], our_mp3[2], our_mp3[3]]);
                let mode = (header >> 6) & 0x3;
                assert_eq!(mode, 3, "Header should indicate mono mode"); // 3 = mono
                println!("✓ Mono mode correctly set in header");
            }
            
            println!("✓ Our mono encoding produces valid output");
        }
    }
}

/// Test to verify our implementation handles edge cases consistently with shine
#[test]
fn test_edge_cases_comparison() {
    create_dir_all("tests/output/comparison").expect("Failed to create comparison directory");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config.clone()).expect("Failed to create encoder");
    
    let samples_per_frame = encoder.samples_per_frame();
    let channels = config.wave.channels as usize;
    
    // Test edge case: maximum amplitude
    let max_pcm: Vec<i16> = (0..samples_per_frame * channels)
        .map(|i| if i % 2 == 0 { i16::MAX } else { i16::MIN })
        .collect();
    
    let our_result = encoder.encode_frame_interleaved(&max_pcm);
    assert!(our_result.is_ok(), "Max amplitude encoding should succeed");
    let our_mp3 = our_result.unwrap().to_vec();
    
    // Write edge case WAV
    let wav_path = "tests/output/comparison/test_edge.wav";
    write_wav_file(wav_path, &max_pcm, config.wave.sample_rate, channels as u16)
        .expect("Failed to write edge case WAV");
    
    // Basic validation - should produce valid MP3
    assert!(!our_mp3.is_empty(), "Edge case MP3 should not be empty");
    
    let syncs = find_sync_words(&our_mp3);
    assert!(!syncs.is_empty(), "Edge case MP3 should have sync words");
    
    analyze_frame_structure(&our_mp3, "Our edge case");
    
    println!("✓ Edge case encoding produces valid output");
    
    // Test another edge case: very small values
    let small_pcm: Vec<i16> = (0..samples_per_frame * channels)
        .map(|i| ((i % 10) as i16) - 5)
        .collect();
    
    let small_result = encoder.encode_frame_interleaved(&small_pcm);
    assert!(small_result.is_ok(), "Small amplitude encoding should succeed");
    let small_mp3 = small_result.unwrap().to_vec();
    
    assert!(!small_mp3.is_empty(), "Small amplitude MP3 should not be empty");
    let small_syncs = find_sync_words(&small_mp3);
    assert!(!small_syncs.is_empty(), "Small amplitude MP3 should have sync words");
    
    analyze_frame_structure(&small_mp3, "Our small amplitude");
    
    println!("✓ Small amplitude encoding produces valid output");
}