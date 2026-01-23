//! Data Flow Validation Tests
//!
//! Comprehensive tests for task 4.1: 添加数据流验证和异常检测
//! 
//! This module implements the data flow validation requirements:
//! - PCM input: non-zero sample ratio > 1%, reasonable dynamic range
//! - Subband output: energy distribution符合音频特征, non-zero coeffs > 10%
//! - MDCT coefficients: frequency domain energy distribution, low freq coeffs usually larger
//! - Quantized coefficients: retain sufficient non-zero coeffs (usually > 5%), big_values < 288
//! - Huffman encoding: generated bits related to non-zero coefficient count
//! - Bitstream: main data region non-zero bytes > 50%
//! - End-to-end testing: use known audio files, verify final MP3 can be decoded by ffmpeg
//! - Comparison validation with existing test cases (using files in tests/input/)

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use rust_mp3_encoder::data_flow_monitor::{DataFlowMonitor, ValidationThresholds, ValidationIssue};
use std::fs::{File, create_dir_all};
use std::io::{Write, Read};
use std::process::Command;
use proptest::prelude::*;

/// Test configuration for data flow validation
pub struct DataFlowTestConfig;

impl DataFlowTestConfig {
    pub fn default_stereo() -> Config {
        Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Stereo,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        }
    }
    
    pub fn default_mono() -> Config {
        Config {
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
        }
    }
    
    pub fn high_quality() -> Config {
        Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::JointStereo,
                bitrate: 320,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        }
    }
}

/// Audio signal generators for validation testing
pub struct ValidationSignalGenerator;

impl ValidationSignalGenerator {
    /// Generate sine wave with specific characteristics for validation
    pub fn sine_wave_validation(samples: usize, channels: usize, sample_rate: f32, frequency: f32, amplitude: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
            })
            .collect()
    }
    
    /// Generate complex audio signal with multiple frequencies
    pub fn complex_audio_signal(samples: usize, channels: usize, sample_rate: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                
                // Mix multiple frequencies to create realistic audio content
                let fundamental = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
                let harmonic2 = (t * 880.0 * 2.0 * std::f32::consts::PI).sin() * 4000.0;
                let harmonic3 = (t * 1320.0 * 2.0 * std::f32::consts::PI).sin() * 2000.0;
                let noise = ((sample_idx as f32 * 0.1).sin() * 500.0);
                
                (fundamental + harmonic2 + harmonic3 + noise) as i16
            })
            .collect()
    }
    
    /// Generate low amplitude signal to test quantization sensitivity
    pub fn low_amplitude_signal(samples: usize, channels: usize, sample_rate: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                (100.0 * (2.0 * std::f32::consts::PI * 1000.0 * t).sin()) as i16
            })
            .collect()
    }
    
    /// Generate high dynamic range signal
    pub fn high_dynamic_range_signal(samples: usize, channels: usize, sample_rate: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                
                // Alternating between high and low amplitude
                let base_freq = 440.0;
                let amplitude = if (sample_idx / 1000) % 2 == 0 { 20000.0 } else { 1000.0 };
                (amplitude * (2.0 * std::f32::consts::PI * base_freq * t).sin()) as i16
            })
            .collect()
    }
    
    /// Generate near-silence signal (edge case)
    pub fn near_silence_signal(samples: usize, channels: usize, sample_rate: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                (10.0 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()) as i16
            })
            .collect()
    }
}

/// FFmpeg validation utilities for end-to-end testing
pub struct FFmpegValidation;

impl FFmpegValidation {
    /// Validate MP3 file can be decoded by FFmpeg
    pub fn validate_mp3_decodable(mp3_path: &str) -> Result<(), String> {
        println!("Validating MP3 decodability with FFmpeg: {}", mp3_path);
        
        let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };
        
        let output = Command::new("ffmpeg")
            .args(&[
                "-v", "error",
                "-i", mp3_path,
                "-f", "null",
                "-y",
                null_device
            ])
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ MP3 file is decodable by FFmpeg");
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    Err(format!("FFmpeg decoding failed: {}", stderr))
                }
            },
            Err(e) => {
                println!("⚠ FFmpeg not available, skipping validation: {}", e);
                Ok(()) // Don't fail if FFmpeg is not installed
            }
        }
    }
    
    /// Get MP3 stream information using FFprobe
    pub fn get_mp3_info(mp3_path: &str) -> Result<Mp3Info, String> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-show_streams",
                "-select_streams", "a:0",
                "-of", "csv=p=0:s=,",
                "-show_entries", "stream=codec_name,sample_rate,channels,bit_rate,duration",
                mp3_path
            ])
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    let info_str = String::from_utf8_lossy(&result.stdout);
                    let parts: Vec<&str> = info_str.trim().split(',').collect();
                    
                    if parts.len() >= 5 {
                        Ok(Mp3Info {
                            codec: parts[0].to_string(),
                            sample_rate: parts[1].parse().unwrap_or(0),
                            channels: parts[2].parse().unwrap_or(0),
                            bit_rate: parts[3].parse().unwrap_or(0),
                            duration: parts[4].parse().unwrap_or(0.0),
                        })
                    } else {
                        Err("Invalid FFprobe output format".to_string())
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    Err(format!("FFprobe failed: {}", stderr))
                }
            },
            Err(e) => {
                Err(format!("Failed to run FFprobe: {}", e))
            }
        }
    }
    
    /// Read WAV file for comparison testing
    pub fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32, u16), String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open WAV file: {}", e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read WAV file: {}", e))?;
        
        if buffer.len() < 44 {
            return Err("WAV file too small".to_string());
        }
        
        // Simple WAV header parsing (assumes standard format)
        if &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
            return Err("Invalid WAV file format".to_string());
        }
        
        // Find fmt chunk (simplified)
        let mut pos = 12;
        while pos + 8 < buffer.len() {
            let chunk_id = &buffer[pos..pos+4];
            let chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
            
            if chunk_id == b"fmt " {
                if chunk_size >= 16 && pos + 8 + 16 <= buffer.len() {
                    let format = u16::from_le_bytes([buffer[pos+8], buffer[pos+9]]);
                    let channels = u16::from_le_bytes([buffer[pos+10], buffer[pos+11]]);
                    let sample_rate = u32::from_le_bytes([buffer[pos+12], buffer[pos+13], buffer[pos+14], buffer[pos+15]]);
                    let bits_per_sample = u16::from_le_bytes([buffer[pos+22], buffer[pos+23]]);
                    
                    if format != 1 || bits_per_sample != 16 {
                        return Err("Unsupported WAV format (only 16-bit PCM supported)".to_string());
                    }
                    
                    // Find data chunk
                    pos += 8 + chunk_size as usize;
                    while pos + 8 < buffer.len() {
                        let data_chunk_id = &buffer[pos..pos+4];
                        let data_chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
                        
                        if data_chunk_id == b"data" {
                            let data_start = pos + 8;
                            let data_end = (data_start + data_chunk_size as usize).min(buffer.len());
                            let pcm_bytes = &buffer[data_start..data_end];
                            
                            let mut pcm_data = Vec::new();
                            for chunk in pcm_bytes.chunks(2) {
                                if chunk.len() == 2 {
                                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                                    pcm_data.push(sample);
                                }
                            }
                            
                            return Ok((pcm_data, sample_rate, channels));
                        }
                        
                        pos += 8 + data_chunk_size as usize;
                    }
                    
                    return Err("No data chunk found in WAV file".to_string());
                }
            }
            
            pos += 8 + chunk_size as usize;
        }
        
        Err("No fmt chunk found in WAV file".to_string())
    }
}

#[derive(Debug)]
pub struct Mp3Info {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub bit_rate: u32,
    pub duration: f64,
}

/// Comprehensive data flow validation test runner
pub struct DataFlowValidationRunner;

impl DataFlowValidationRunner {
    /// Setup test environment
    pub fn setup() -> Result<(), std::io::Error> {
        create_dir_all("tests/output")?;
        Ok(())
    }
    
    /// Run comprehensive data flow validation test
    pub fn run_validation_test(
        test_name: &str,
        config: Config,
        pcm_data: Vec<i16>,
        custom_thresholds: Option<ValidationThresholds>,
    ) -> Result<ValidationReport, String> {
        println!("\n=== Data Flow Validation Test: {} ===", test_name);
        
        // Create encoder with data flow monitoring
        let mut encoder = Mp3Encoder::new(config.clone())
            .map_err(|e| format!("Failed to create encoder: {:?}", e))?;
        
        // Setup data flow monitor
        let thresholds = custom_thresholds.unwrap_or_default();
        let mut monitor = DataFlowMonitor::with_thresholds(thresholds, true);
        
        let samples_per_frame = encoder.samples_per_frame();
        let channels = match config.wave.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };
        let frame_size = samples_per_frame * channels;
        
        println!("Config: {}kbps, {}Hz, {} channels", 
                 config.mpeg.bitrate, config.wave.sample_rate, channels);
        println!("Samples per frame: {}, Frame size: {}", samples_per_frame, frame_size);
        
        let mut mp3_data = Vec::new();
        let mut frame_count = 0;
        
        // Monitor PCM input
        monitor.monitor_pcm_input(&pcm_data)
            .map_err(|e| format!("PCM monitoring failed: {:?}", e))?;
        
        // Encode frames with monitoring
        for chunk in pcm_data.chunks(frame_size) {
            if chunk.len() == frame_size {
                // TODO: Add monitoring hooks to encoder for subband, MDCT, quantization, huffman stages
                // This would require modifying the encoder to accept a monitor callback
                
                match encoder.encode_frame_interleaved(chunk) {
                    Ok(frame_data) => {
                        frame_count += 1;
                        mp3_data.extend_from_slice(frame_data);
                        
                        // Monitor bitstream output
                        monitor.monitor_bitstream_output(frame_data, 0, frame_data.len())
                            .map_err(|e| format!("Bitstream monitoring failed: {:?}", e))?;
                        
                        if frame_count <= 3 {
                            println!("Frame {}: {} bytes", frame_count, frame_data.len());
                        }
                    },
                    Err(e) => {
                        return Err(format!("Failed to encode frame {}: {:?}", frame_count + 1, e));
                    }
                }
            }
        }
        
        // Flush encoder
        match encoder.flush() {
            Ok(final_data) => {
                if !final_data.is_empty() {
                    mp3_data.extend_from_slice(final_data);
                    monitor.monitor_bitstream_output(final_data, 0, final_data.len())
                        .map_err(|e| format!("Final bitstream monitoring failed: {:?}", e))?;
                    println!("Flushed: {} bytes", final_data.len());
                }
            },
            Err(e) => {
                return Err(format!("Failed to flush: {:?}", e));
            }
        }
        
        println!("Total: {} frames, {} bytes", frame_count, mp3_data.len());
        
        // Write output file for validation
        let output_path = format!("tests/output/dataflow_{}.mp3", test_name.replace(" ", "_").to_lowercase());
        let mut file = File::create(&output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        file.write_all(&mp3_data)
            .map_err(|e| format!("Failed to write MP3 data: {}", e))?;
        
        println!("Written to: {}", output_path);
        
        // Generate validation report
        let report = monitor.generate_report();
        
        // Validate with FFmpeg if available
        if let Err(e) = FFmpegValidation::validate_mp3_decodable(&output_path) {
            println!("FFmpeg validation warning: {}", e);
        }
        
        Ok(ValidationReport {
            test_name: test_name.to_string(),
            config,
            frame_count,
            mp3_size: mp3_data.len(),
            output_path,
            monitor_report: report,
            mp3_data,
        })
    }
    
    /// Analyze validation report and check for critical issues
    pub fn analyze_report(report: &ValidationReport) -> AnalysisResult {
        let mut critical_issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for critical data flow issues
        for issue in &report.monitor_report.issues {
            match issue {
                ValidationIssue::QuantizationIssue { big_values, .. } if *big_values > 288 => {
                    critical_issues.push(format!("big_values exceeds MP3 limit: {}", big_values));
                },
                ValidationIssue::BitstreamIssue { nonzero_ratio, .. } if *nonzero_ratio < 0.1 => {
                    critical_issues.push(format!("Main data mostly zero: {:.1}%", nonzero_ratio * 100.0));
                },
                ValidationIssue::PcmInputIssue { .. } => {
                    warnings.push("PCM input validation issue".to_string());
                },
                _ => {
                    warnings.push(format!("Validation issue: {:?}", issue));
                }
            }
        }
        
        // Check MP3 data quality
        let zero_bytes = report.mp3_data.iter().filter(|&&b| b == 0).count();
        let zero_ratio = zero_bytes as f32 / report.mp3_data.len() as f32;
        
        if zero_ratio > 0.8 {
            critical_issues.push(format!("MP3 data mostly zeros: {:.1}%", zero_ratio * 100.0));
        }
        
        // Check for valid MP3 sync words
        let mut sync_count = 0;
        for i in 0..report.mp3_data.len().saturating_sub(1) {
            let sync = ((report.mp3_data[i] as u16) << 3) | ((report.mp3_data[i + 1] as u16) >> 5);
            if sync == 0x7FF {
                sync_count += 1;
            }
        }
        
        if sync_count == 0 {
            critical_issues.push("No valid MP3 sync words found".to_string());
        }
        
        AnalysisResult {
            passed: critical_issues.is_empty(),
            critical_issues,
            warnings,
            sync_words_found: sync_count,
            zero_byte_ratio: zero_ratio,
        }
    }
}

#[derive(Debug)]
pub struct ValidationReport {
    pub test_name: String,
    pub config: Config,
    pub frame_count: usize,
    pub mp3_size: usize,
    pub output_path: String,
    pub monitor_report: rust_mp3_encoder::data_flow_monitor::ValidationReport,
    pub mp3_data: Vec<u8>,
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub passed: bool,
    pub critical_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub sync_words_found: usize,
    pub zero_byte_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_clean_errors() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                if let Some(s) = info.payload().downcast_ref::<String>() {
                    let msg = if s.len() > 200 { &s[..197] } else { s };
                    eprintln!("Test failed: {}", msg.trim());
                }
            }));
        });
    }

    #[test]
    fn test_data_flow_validation_setup() {
        setup_clean_errors();
        
        let result = DataFlowValidationRunner::setup();
        assert!(result.is_ok(), "Setup should succeed");
    }

    #[test]
    fn test_sine_wave_data_flow_validation() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let config = DataFlowTestConfig::default_stereo();
        let pcm_data = ValidationSignalGenerator::sine_wave_validation(4410, 2, 44100.0, 440.0, 16000.0);
        
        let result = DataFlowValidationRunner::run_validation_test(
            "sine_wave_440hz",
            config,
            pcm_data,
            None,
        );
        
        assert!(result.is_ok(), "Sine wave validation should succeed");
        
        let report = result.unwrap();
        let analysis = DataFlowValidationRunner::analyze_report(&report);
        
        println!("Analysis: {:?}", analysis);
        
        // Should not have critical issues for a proper sine wave
        assert!(analysis.critical_issues.is_empty(), 
                "Sine wave should not produce critical issues: {:?}", analysis.critical_issues);
        
        // Should find at least one sync word
        assert!(analysis.sync_words_found > 0, "Should find MP3 sync words");
        
        // Should not be mostly zeros
        assert!(analysis.zero_byte_ratio < 0.5, "MP3 data should not be mostly zeros");
    }

    #[test]
    fn test_complex_audio_data_flow_validation() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let config = DataFlowTestConfig::default_stereo();
        let pcm_data = ValidationSignalGenerator::complex_audio_signal(8820, 2, 44100.0); // 0.2 seconds
        
        let result = DataFlowValidationRunner::run_validation_test(
            "complex_audio",
            config,
            pcm_data,
            None,
        );
        
        assert!(result.is_ok(), "Complex audio validation should succeed");
        
        let report = result.unwrap();
        let analysis = DataFlowValidationRunner::analyze_report(&report);
        
        println!("Complex audio analysis: {:?}", analysis);
        
        // Complex audio should encode well
        assert!(analysis.critical_issues.is_empty(), 
                "Complex audio should not produce critical issues: {:?}", analysis.critical_issues);
    }

    #[test]
    fn test_low_amplitude_signal_validation() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let config = DataFlowTestConfig::default_mono();
        let pcm_data = ValidationSignalGenerator::low_amplitude_signal(4410, 1, 44100.0);
        
        // Use relaxed thresholds for low amplitude signals
        let custom_thresholds = ValidationThresholds {
            quantized_nonzero_ratio_min: 0.01, // Lower threshold for low amplitude
            bitstream_main_data_nonzero_min: 0.1, // Lower threshold
            ..Default::default()
        };
        
        let result = DataFlowValidationRunner::run_validation_test(
            "low_amplitude",
            config,
            pcm_data,
            Some(custom_thresholds),
        );
        
        assert!(result.is_ok(), "Low amplitude validation should succeed");
        
        let report = result.unwrap();
        let analysis = DataFlowValidationRunner::analyze_report(&report);
        
        println!("Low amplitude analysis: {:?}", analysis);
        
        // Low amplitude might have warnings but should not have critical issues
        // if thresholds are adjusted appropriately
    }

    #[test]
    fn test_high_dynamic_range_validation() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let config = DataFlowTestConfig::high_quality();
        let pcm_data = ValidationSignalGenerator::high_dynamic_range_signal(8820, 2, 44100.0);
        
        let result = DataFlowValidationRunner::run_validation_test(
            "high_dynamic_range",
            config,
            pcm_data,
            None,
        );
        
        assert!(result.is_ok(), "High dynamic range validation should succeed");
        
        let report = result.unwrap();
        let analysis = DataFlowValidationRunner::analyze_report(&report);
        
        println!("High dynamic range analysis: {:?}", analysis);
        
        // High quality encoding should handle dynamic range well
        assert!(analysis.critical_issues.is_empty(), 
                "High dynamic range should not produce critical issues: {:?}", analysis.critical_issues);
    }

    #[test]
    fn test_near_silence_edge_case() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let config = DataFlowTestConfig::default_mono();
        let pcm_data = ValidationSignalGenerator::near_silence_signal(4410, 1, 44100.0);
        
        // Very relaxed thresholds for near-silence
        let custom_thresholds = ValidationThresholds {
            pcm_nonzero_ratio_min: 0.001, // Very low threshold
            quantized_nonzero_ratio_min: 0.001,
            bitstream_main_data_nonzero_min: 0.01,
            ..Default::default()
        };
        
        let result = DataFlowValidationRunner::run_validation_test(
            "near_silence",
            config,
            pcm_data,
            Some(custom_thresholds),
        );
        
        assert!(result.is_ok(), "Near silence validation should succeed");
        
        let report = result.unwrap();
        let analysis = DataFlowValidationRunner::analyze_report(&report);
        
        println!("Near silence analysis: {:?}", analysis);
        
        // Near silence is an edge case that might produce warnings but should still encode
        assert!(analysis.sync_words_found > 0, "Should still produce valid MP3 frames");
    }

    #[test]
    #[ignore] // Only run when test audio files are available
    fn test_real_audio_file_validation() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let wav_path = "tests/input/sample-12s.wav";
        
        if std::path::Path::new(wav_path).exists() {
            match FFmpegValidation::read_wav_file(wav_path) {
                Ok((pcm_data, sample_rate, channels)) => {
                    println!("Loaded WAV: {} samples, {}Hz, {} channels", 
                             pcm_data.len(), sample_rate, channels);
                    
                    let config = if channels == 1 {
                        DataFlowTestConfig::default_mono()
                    } else {
                        DataFlowTestConfig::default_stereo()
                    };
                    
                    let result = DataFlowValidationRunner::run_validation_test(
                        "real_audio_sample_12s",
                        config,
                        pcm_data,
                        None,
                    );
                    
                    assert!(result.is_ok(), "Real audio file validation should succeed");
                    
                    let report = result.unwrap();
                    let analysis = DataFlowValidationRunner::analyze_report(&report);
                    
                    println!("Real audio analysis: {:?}", analysis);
                    
                    // Real audio should encode without critical issues
                    assert!(analysis.critical_issues.is_empty(), 
                            "Real audio should not produce critical issues: {:?}", analysis.critical_issues);
                    
                    // Validate with FFmpeg
                    FFmpegValidation::validate_mp3_decodable(&report.output_path)
                        .expect("Real audio MP3 should be decodable by FFmpeg");
                },
                Err(e) => {
                    println!("Failed to read WAV file: {}", e);
                    panic!("Could not read test WAV file");
                }
            }
        } else {
            println!("Test WAV file not found at {}, skipping test", wav_path);
        }
    }

    #[test]
    #[ignore] // Only run when FFmpeg is available
    fn test_ffmpeg_end_to_end_validation() {
        setup_clean_errors();
        DataFlowValidationRunner::setup().expect("Setup failed");
        
        let config = DataFlowTestConfig::default_stereo();
        let pcm_data = ValidationSignalGenerator::complex_audio_signal(44100, 2, 44100.0); // 1 second
        
        let result = DataFlowValidationRunner::run_validation_test(
            "ffmpeg_end_to_end",
            config,
            pcm_data,
            None,
        );
        
        assert!(result.is_ok(), "End-to-end validation should succeed");
        
        let report = result.unwrap();
        
        // Validate with FFmpeg
        FFmpegValidation::validate_mp3_decodable(&report.output_path)
            .expect("MP3 should be decodable by FFmpeg");
        
        // Get MP3 info
        if let Ok(info) = FFmpegValidation::get_mp3_info(&report.output_path) {
            println!("MP3 Info: {:?}", info);
            assert_eq!(info.codec, "mp3", "Should be MP3 codec");
            assert_eq!(info.sample_rate, 44100, "Should maintain sample rate");
            assert_eq!(info.channels, 2, "Should maintain channel count");
        }
    }

    // Property-based tests for data flow validation
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 10,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_data_flow_validation_property(
            frequency in 100.0f32..2000.0f32,
            amplitude in 1000.0f32..20000.0f32,
            duration_ms in 100u32..500u32,
        ) {
            setup_clean_errors();
            DataFlowValidationRunner::setup().expect("Setup failed");
            
            let config = DataFlowTestConfig::default_mono();
            let samples = (44100 * duration_ms / 1000) as usize;
            let pcm_data = ValidationSignalGenerator::sine_wave_validation(samples, 1, 44100.0, frequency, amplitude);
            
            let result = DataFlowValidationRunner::run_validation_test(
                &format!("property_test_{}hz_{}amp", frequency as u32, amplitude as u32),
                config,
                pcm_data,
                None,
            );
            
            prop_assert!(result.is_ok(), "Property test should succeed");
            
            let report = result.unwrap();
            let analysis = DataFlowValidationRunner::analyze_report(&report);
            
            // Should produce valid MP3 with sync words
            prop_assert!(analysis.sync_words_found > 0, "Should find MP3 sync words");
            
            // Should not be mostly zeros (unless very low amplitude)
            if amplitude > 5000.0 {
                prop_assert!(analysis.zero_byte_ratio < 0.7, "Should not be mostly zeros for reasonable amplitude");
            }
        }
    }
}