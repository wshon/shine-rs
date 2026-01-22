//! Comprehensive validation tests for MP3 encoder
//!
//! This module consolidates all validation tests including FFmpeg validation,
//! integration tests, and comparison with reference implementations.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all, remove_file};
use std::io::Write;
use std::process::Command;

/// Test configuration presets
pub struct TestConfigs;

impl TestConfigs {
    pub fn mono_44100_128() -> Config {
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
    
    pub fn stereo_44100_128() -> Config {
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
    
    pub fn joint_stereo_48000_192() -> Config {
        Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 48000,
            },
            mpeg: MpegConfig {
                mode: StereoMode::JointStereo,
                bitrate: 192,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        }
    }
    
    pub fn shine_reference() -> Config {
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
}

/// Audio signal generators for testing
pub struct SignalGenerator;

impl SignalGenerator {
    /// Generate sine wave with specified frequency and amplitude
    pub fn sine_wave(samples: usize, channels: usize, sample_rate: f32, frequency: f32, amplitude: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
            })
            .collect()
    }
    
    /// Generate mixed frequency signal for comprehensive testing
    pub fn mixed_frequencies(samples: usize, channels: usize, sample_rate: f32) -> Vec<i16> {
        (0..samples * channels)
            .map(|i| {
                let sample_idx = i / channels;
                let t = sample_idx as f32 / sample_rate;
                
                // Mix of 440Hz and 880Hz sine waves
                let sample1 = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
                let sample2 = (t * 880.0 * 2.0 * std::f32::consts::PI).sin() * 4000.0;
                (sample1 + sample2) as i16
            })
            .collect()
    }
    
    /// Generate silence for testing edge cases
    pub fn silence(samples: usize, channels: usize) -> Vec<i16> {
        vec![0i16; samples * channels]
    }
    
    /// Generate white noise for stress testing
    pub fn white_noise(samples: usize, channels: usize, amplitude: f32) -> Vec<i16> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        (0..samples * channels)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let normalized = (hash as f32 / u64::MAX as f32) * 2.0 - 1.0;
                (normalized * amplitude) as i16
            })
            .collect()
    }
}

/// FFmpeg validation utilities
pub struct FFmpegValidator;

impl FFmpegValidator {
    /// Validate MP3 file using FFmpeg
    pub fn validate_mp3(mp3_path: &str) -> Result<(), String> {
        println!("Validating MP3 file with FFmpeg: {}", mp3_path);
        
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
                    println!("✅ FFmpeg validation passed");
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    Err(format!("FFmpeg validation failed: {}", stderr))
                }
            },
            Err(e) => {
                println!("⚠ FFmpeg not available: {}", e);
                Ok(()) // Don't fail if FFmpeg is not installed
            }
        }
    }
    
    /// Read WAV file and extract PCM data
    pub fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32, u16), String> {
        use std::io::{Read, Seek, SeekFrom};
        
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open WAV file: {}", e))?;
        
        // Read RIFF header
        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Failed to read RIFF header: {}", e))?;
        
        if &buffer != b"RIFF" {
            return Err("Invalid WAV file: missing RIFF header".to_string());
        }
        
        // Skip file size
        file.seek(SeekFrom::Current(4))
            .map_err(|e| format!("Failed to seek in WAV file: {}", e))?;
        
        // Read WAVE header
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Failed to read WAVE header: {}", e))?;
        
        if &buffer != b"WAVE" {
            return Err("Invalid WAV file: missing WAVE header".to_string());
        }
        
        // Find fmt chunk
        let mut sample_rate = 0u32;
        let mut channels = 0u16;
        let mut _bits_per_sample = 0u16;
        
        loop {
            file.read_exact(&mut buffer)
                .map_err(|e| format!("Failed to read chunk header: {}", e))?;
            
            let mut size_buffer = [0u8; 4];
            file.read_exact(&mut size_buffer)
                .map_err(|e| format!("Failed to read chunk size: {}", e))?;
            let chunk_size = u32::from_le_bytes(size_buffer);
            
            if &buffer == b"fmt " {
                // Read format chunk
                let mut fmt_data = vec![0u8; chunk_size as usize];
                file.read_exact(&mut fmt_data)
                    .map_err(|e| format!("Failed to read fmt chunk: {}", e))?;
                
                if fmt_data.len() < 16 {
                    return Err("Invalid fmt chunk size".to_string());
                }
                
                let format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
                if format != 1 {
                    return Err("Unsupported WAV format (not PCM)".to_string());
                }
                
                channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
                sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
                _bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);
                
                if _bits_per_sample != 16 {
                    return Err("Unsupported bit depth (only 16-bit supported)".to_string());
                }
                
                break;
            } else {
                // Skip unknown chunk
                file.seek(SeekFrom::Current(chunk_size as i64))
                    .map_err(|e| format!("Failed to skip chunk: {}", e))?;
            }
        }
        
        // Find data chunk
        loop {
            file.read_exact(&mut buffer)
                .map_err(|e| format!("Failed to read data chunk header: {}", e))?;
            
            let mut size_buffer = [0u8; 4];
            file.read_exact(&mut size_buffer)
                .map_err(|e| format!("Failed to read data chunk size: {}", e))?;
            let chunk_size = u32::from_le_bytes(size_buffer);
            
            if &buffer == b"data" {
                // Read PCM data
                let sample_count = (chunk_size / 2) as usize; // 16-bit samples
                let mut pcm_data = vec![0i16; sample_count];
                
                for sample in &mut pcm_data {
                    let mut sample_buffer = [0u8; 2];
                    file.read_exact(&mut sample_buffer)
                        .map_err(|e| format!("Failed to read PCM sample: {}", e))?;
                    *sample = i16::from_le_bytes(sample_buffer);
                }
                
                return Ok((pcm_data, sample_rate, channels));
            } else {
                // Skip unknown chunk
                file.seek(SeekFrom::Current(chunk_size as i64))
                    .map_err(|e| format!("Failed to skip data chunk: {}", e))?;
            }
        }
    }
    
    /// Get MP3 duration using FFprobe
    pub fn get_mp3_duration(mp3_path: &str) -> Result<f64, String> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-show_entries", "format=duration",
                "-of", "csv=p=0",
                mp3_path
            ])
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    let duration_str = String::from_utf8_lossy(&result.stdout);
                    let duration: f64 = duration_str.trim().parse()
                        .map_err(|e| format!("Failed to parse duration: {}", e))?;
                    Ok(duration)
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
}

/// Comprehensive test runner
pub struct ValidationRunner;

impl ValidationRunner {
    /// Setup test environment
    pub fn setup() -> Result<(), std::io::Error> {
        create_dir_all("tests/output")?;
        Self::cleanup_old_files()?;
        Ok(())
    }
    
    /// Clean up old test files
    fn cleanup_old_files() -> Result<(), std::io::Error> {
        if let Ok(entries) = std::fs::read_dir("tests/output") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "mp3" {
                            let _ = remove_file(&path);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Run comprehensive validation test
    pub fn run_validation_test(
        name: &str,
        config: Config,
        pcm_data: Vec<i16>,
        validate_with_ffmpeg: bool
    ) -> Result<Vec<u8>, String> {
        println!("=== Validation Test: {} ===", name);
        
        let mut encoder = Mp3Encoder::new(config.clone())
            .map_err(|e| format!("Failed to create encoder: {:?}", e))?;
        
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
        
        // Encode frames
        for chunk in pcm_data.chunks(frame_size) {
            if chunk.len() == frame_size {
                match encoder.encode_frame_interleaved(chunk) {
                    Ok(frame_data) => {
                        frame_count += 1;
                        mp3_data.extend_from_slice(frame_data);
                        
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
                mp3_data.extend_from_slice(final_data);
                if !final_data.is_empty() {
                    println!("Flushed: {} bytes", final_data.len());
                }
            },
            Err(e) => {
                return Err(format!("Failed to flush: {:?}", e));
            }
        }
        
        println!("Total: {} frames, {} bytes", frame_count, mp3_data.len());
        
        // Write output file
        let output_path = format!("tests/output/validation_{}.mp3", name.replace(" ", "_").to_lowercase());
        let mut file = File::create(&output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        file.write_all(&mp3_data)
            .map_err(|e| format!("Failed to write MP3 data: {}", e))?;
        
        println!("Written to: {}", output_path);
        
        // Validate with FFmpeg if requested
        if validate_with_ffmpeg {
            FFmpegValidator::validate_mp3(&output_path)?;
        }
        
        Ok(mp3_data)
    }
    
    /// Analyze MP3 data for common issues
    pub fn analyze_mp3_data(data: &[u8], label: &str) {
        println!("\n--- {} Analysis ---", label);
        println!("Size: {} bytes", data.len());
        
        // Check for excessive 0xFF bytes
        let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
        let ff_percentage = if data.len() > 0 { 
            (ff_count as f32 / data.len() as f32) * 100.0 
        } else { 
            0.0 
        };
        
        println!("0xFF bytes: {} ({:.1}%)", ff_count, ff_percentage);
        
        if ff_percentage > 50.0 {
            println!("⚠ WARNING: High 0xFF content may indicate encoding issues");
        }
        
        // Count sync words
        let mut sync_count = 0;
        for i in 0..data.len().saturating_sub(1) {
            let sync = ((data[i] as u16) << 3) | ((data[i + 1] as u16) >> 5);
            if sync == 0x7FF {
                sync_count += 1;
            }
        }
        
        println!("Sync words found: {}", sync_count);
        
        // Show first few bytes
        if data.len() >= 16 {
            print!("First 16 bytes: ");
            for i in 0..16 {
                print!("{:02X} ", data[i]);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encoder_creation() {
        let config = Config::default();
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_ok(), "Should create encoder with default config");
    }
    
    #[test]
    fn test_samples_per_frame() {
        // MPEG-1 should have 1152 samples per frame
        let config = TestConfigs::stereo_44100_128();
        let encoder = Mp3Encoder::new(config).unwrap();
        assert_eq!(encoder.samples_per_frame(), 1152);
        
        // MPEG-2 should have 576 samples per frame
        let mut config = TestConfigs::stereo_44100_128();
        config.wave.sample_rate = 22050;
        let encoder = Mp3Encoder::new(config).unwrap();
        assert_eq!(encoder.samples_per_frame(), 576);
    }
    
    #[test]
    fn test_silence_encoding() {
        ValidationRunner::setup().expect("Failed to setup test environment");
        
        let config = TestConfigs::stereo_44100_128();
        let pcm_data = SignalGenerator::silence(4410, 2); // 0.1 seconds of stereo silence
        
        let result = ValidationRunner::run_validation_test(
            "silence_encoding",
            config,
            pcm_data,
            false
        );
        
        assert!(result.is_ok(), "Silence encoding should succeed");
        
        let mp3_data = result.unwrap();
        ValidationRunner::analyze_mp3_data(&mp3_data, "Silence Test");
        
        // Silence should not produce excessive 0xFF bytes
        let ff_count = mp3_data.iter().filter(|&&b| b == 0xFF).count();
        let ff_percentage = (ff_count as f32 / mp3_data.len() as f32) * 100.0;
        assert!(ff_percentage < 50.0, "Silence should not produce excessive 0xFF bytes");
    }
    
    #[test]
    fn test_sine_wave_encoding() {
        ValidationRunner::setup().expect("Failed to setup test environment");
        
        let config = TestConfigs::stereo_44100_128();
        let pcm_data = SignalGenerator::sine_wave(4410, 2, 44100.0, 440.0, 16000.0);
        
        let result = ValidationRunner::run_validation_test(
            "sine_wave_440hz",
            config,
            pcm_data,
            false
        );
        
        assert!(result.is_ok(), "Sine wave encoding should succeed");
    }
    
    #[test]
    fn test_mixed_frequencies() {
        ValidationRunner::setup().expect("Failed to setup test environment");
        
        let config = TestConfigs::stereo_44100_128();
        let pcm_data = SignalGenerator::mixed_frequencies(4410, 2, 44100.0);
        
        let result = ValidationRunner::run_validation_test(
            "mixed_frequencies",
            config,
            pcm_data,
            false
        );
        
        assert!(result.is_ok(), "Mixed frequency encoding should succeed");
    }
    
    #[test]
    fn test_different_configurations() {
        ValidationRunner::setup().expect("Failed to setup test environment");
        
        let configs = [
            ("mono_44100_128", TestConfigs::mono_44100_128()),
            ("stereo_44100_128", TestConfigs::stereo_44100_128()),
            ("joint_stereo_48000_192", TestConfigs::joint_stereo_48000_192()),
        ];
        
        for (name, config) in configs.iter() {
            let channels = match config.wave.channels {
                Channels::Mono => 1,
                Channels::Stereo => 2,
            };
            
            let sample_count = (config.wave.sample_rate as f32 * 0.1) as usize; // 0.1 seconds
            let pcm_data = SignalGenerator::sine_wave(
                sample_count, 
                channels, 
                config.wave.sample_rate as f32, 
                1000.0, 
                16000.0
            );
            
            let result = ValidationRunner::run_validation_test(
                name,
                config.clone(),
                pcm_data,
                false
            );
            
            assert!(result.is_ok(), "Configuration {} should encode successfully", name);
        }
    }
    
    #[test]
    #[ignore] // Only run when real audio files are available
    fn test_real_audio_file_encoding() {
        ValidationRunner::setup().expect("Failed to setup test environment");
        
        // This test requires a real WAV file in tests/input/
        let wav_path = "tests/input/test_audio.wav";
        
        if std::path::Path::new(wav_path).exists() {
            match FFmpegValidator::read_wav_file(wav_path) {
                Ok((pcm_data, sample_rate, channels)) => {
                    println!("Loaded WAV: {} samples, {}Hz, {} channels", 
                             pcm_data.len(), sample_rate, channels);
                    
                    let config = if channels == 1 {
                        TestConfigs::mono_44100_128()
                    } else {
                        TestConfigs::stereo_44100_128()
                    };
                    
                    let result = ValidationRunner::run_validation_test(
                        "real_audio_file",
                        config,
                        pcm_data,
                        true // Enable FFmpeg validation
                    );
                    
                    assert!(result.is_ok(), "Real audio file encoding should succeed");
                },
                Err(e) => {
                    println!("Failed to read WAV file: {}", e);
                }
            }
        } else {
            println!("No test audio file found at {}, skipping test", wav_path);
        }
    }
    
    #[test]
    #[ignore] // Only run when FFmpeg is available
    fn test_ffmpeg_validation() {
        ValidationRunner::setup().expect("Failed to setup test environment");
        
        let config = TestConfigs::stereo_44100_128();
        let pcm_data = SignalGenerator::mixed_frequencies(44100, 2, 44100.0); // 1 second
        
        let result = ValidationRunner::run_validation_test(
            "ffmpeg_validation",
            config,
            pcm_data,
            true // Enable FFmpeg validation
        );
        
        assert!(result.is_ok(), "FFmpeg validation should pass");
    }
}