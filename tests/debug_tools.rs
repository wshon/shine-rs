//! Debug tools for MP3 encoder testing
//!
//! This module provides comprehensive debugging utilities for analyzing
//! MP3 encoding issues, particularly big_values problems and frame structure.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::process::Command;

/// Configuration for debug tests
#[derive(Clone)]
pub struct DebugConfig {
    pub sample_rate: u32,
    pub channels: Channels,
    pub mode: StereoMode,
    pub bitrate: u32,
    pub duration: f32,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: Channels::Stereo,
            mode: StereoMode::Stereo,
            bitrate: 128,
            duration: 0.1,
        }
    }
}

/// Signal generator for test patterns
pub struct SignalGenerator;

impl SignalGenerator {
    /// Generate sine wave signal
    pub fn sine_wave(config: &DebugConfig, frequency: f32, amplitude: f32) -> Vec<i16> {
        let samples_count = (config.sample_rate as f32 * config.duration) as usize;
        let channels = match config.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };
        
        let mut pcm_data = Vec::with_capacity(samples_count * channels);
        
        for i in 0..samples_count {
            let t = i as f32 / config.sample_rate as f32;
            let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin() * amplitude;
            let sample_i16 = sample as i16;
            
            pcm_data.push(sample_i16);
            if channels == 2 {
                pcm_data.push(sample_i16); // Duplicate for stereo
            }
        }
        
        pcm_data
    }
    
    /// Generate mixed frequency signal (might cause big_values issues)
    pub fn mixed_frequencies(config: &DebugConfig) -> Vec<i16> {
        let samples_count = (config.sample_rate as f32 * config.duration) as usize;
        let channels = match config.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };
        
        let mut pcm_data = Vec::with_capacity(samples_count * channels);
        
        for i in 0..samples_count {
            let t = i as f32 / config.sample_rate as f32;
            
            // Mix frequencies that might cause large MDCT coefficients
            let low_freq = (t * 100.0 * 2.0 * std::f32::consts::PI).sin() * 15000.0;
            let mid_freq = (t * 1000.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
            let high_freq = (t * 8000.0 * 2.0 * std::f32::consts::PI).sin() * 5000.0;
            
            let mixed = (low_freq + mid_freq + high_freq) as i16;
            
            pcm_data.push(mixed);
            if channels == 2 {
                pcm_data.push(mixed);
            }
        }
        
        pcm_data
    }
    
    /// Generate quiet signal for testing low-amplitude cases
    pub fn quiet_signal(config: &DebugConfig) -> Vec<i16> {
        Self::sine_wave(config, 440.0, 100.0) // Very quiet 440Hz tone
    }
    
    /// Generate silence
    pub fn silence(config: &DebugConfig) -> Vec<i16> {
        let samples_count = (config.sample_rate as f32 * config.duration) as usize;
        let channels = match config.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };
        
        vec![0i16; samples_count * channels]
    }
    
    /// Generate constant value signal for testing
    pub fn constant_value(config: &DebugConfig, value: i16) -> Vec<i16> {
        let samples_count = (config.sample_rate as f32 * config.duration) as usize;
        let channels = match config.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };
        
        vec![value; samples_count * channels]
    }
}

/// MP3 frame analyzer
pub struct FrameAnalyzer;

impl FrameAnalyzer {
    /// Analyze MP3 frame structure and detect issues
    pub fn analyze_frame(frame_data: &[u8]) -> FrameAnalysis {
        let mut analysis = FrameAnalysis::default();
        analysis.frame_size = frame_data.len();
        
        if frame_data.len() < 4 {
            analysis.errors.push("Frame too small".to_string());
            return analysis;
        }
        
        // Check sync word
        let sync = ((frame_data[0] as u16) << 3) | ((frame_data[1] as u16) >> 5);
        analysis.sync_word = sync;
        
        if sync != 0x7FF {
            analysis.errors.push(format!("Invalid sync word: 0x{:03X}", sync));
            return analysis;
        }
        
        // Parse frame header
        analysis.version = (frame_data[1] >> 3) & 0x03;
        analysis.layer = (frame_data[1] >> 1) & 0x03;
        analysis.bitrate_index = (frame_data[2] >> 4) & 0x0F;
        analysis.sample_rate_index = (frame_data[2] >> 2) & 0x03;
        analysis.padding = (frame_data[2] >> 1) & 0x01;
        analysis.channel_mode = (frame_data[3] >> 6) & 0x03;
        
        // Analyze side info for MPEG-1 stereo
        if frame_data.len() >= 36 && analysis.version == 3 && analysis.channel_mode != 3 {
            analysis.side_info_analysis = Some(Self::analyze_side_info(&frame_data[4..36]));
        }
        
        analysis
    }
    
    /// Analyze side information for big_values issues
    fn analyze_side_info(side_info: &[u8]) -> SideInfoAnalysis {
        let mut analysis = SideInfoAnalysis::default();
        
        // Parse main data begin (9 bits)
        analysis.main_data_begin = ((side_info[0] as u16) << 1) | ((side_info[1] as u16) >> 7);
        
        // Parse granule info for MPEG-1 stereo (2 granules × 2 channels)
        let mut bit_offset = 20; // After main_data_begin + private_bits + SCFSI
        
        for gr in 0..2 {
            for ch in 0..2 {
                if bit_offset + 29 <= side_info.len() * 8 {
                    let part2_3_length = Self::extract_bits(side_info, bit_offset, 12);
                    bit_offset += 12;
                    
                    let big_values = Self::extract_bits(side_info, bit_offset, 9);
                    bit_offset += 9;
                    
                    let global_gain = Self::extract_bits(side_info, bit_offset, 8);
                    bit_offset += 8;
                    
                    let granule_info = GranuleAnalysis {
                        granule: gr,
                        channel: ch,
                        part2_3_length,
                        big_values,
                        global_gain,
                        is_valid: big_values <= 288,
                    };
                    
                    analysis.granules.push(granule_info);
                    
                    if big_values > 288 {
                        analysis.errors.push(format!(
                            "Granule {} Channel {}: big_values {} exceeds maximum 288",
                            gr, ch, big_values
                        ));
                    }
                    
                    // Skip remaining fields
                    bit_offset += 28;
                }
            }
        }
        
        analysis
    }
    
    /// Extract bits from byte array
    fn extract_bits(data: &[u8], bit_offset: usize, num_bits: usize) -> u16 {
        let mut result = 0u16;
        
        for i in 0..num_bits {
            let byte_index = (bit_offset + i) / 8;
            let bit_index = 7 - ((bit_offset + i) % 8);
            
            if byte_index < data.len() {
                let bit = (data[byte_index] >> bit_index) & 1;
                result = (result << 1) | (bit as u16);
            }
        }
        
        result
    }
}

/// Frame analysis results
#[derive(Debug, Default)]
pub struct FrameAnalysis {
    pub frame_size: usize,
    pub sync_word: u16,
    pub version: u8,
    pub layer: u8,
    pub bitrate_index: u8,
    pub sample_rate_index: u8,
    pub padding: u8,
    pub channel_mode: u8,
    pub side_info_analysis: Option<SideInfoAnalysis>,
    pub errors: Vec<String>,
}

/// Side info analysis results
#[derive(Debug, Default)]
pub struct SideInfoAnalysis {
    pub main_data_begin: u16,
    pub granules: Vec<GranuleAnalysis>,
    pub errors: Vec<String>,
}

/// Granule analysis results
#[derive(Debug)]
pub struct GranuleAnalysis {
    pub granule: usize,
    pub channel: usize,
    pub part2_3_length: u16,
    pub big_values: u16,
    pub global_gain: u16,
    pub is_valid: bool,
}

/// Pipeline analyzer for detailed debugging
pub struct PipelineAnalyzer;

impl PipelineAnalyzer {
    /// Analyze 0xFF byte patterns in data
    pub fn analyze_ff_pattern(data: &[u8], label: &str) {
        let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
        let total = data.len();
        let ff_percentage = if total > 0 { (ff_count as f32 / total as f32) * 100.0 } else { 0.0 };
        
        println!("{}: {} bytes, {} 0xFF bytes ({:.1}%)", label, total, ff_count, ff_percentage);
        
        if ff_percentage > 50.0 {
            println!("  ⚠ WARNING: High 0xFF content");
            
            // Show pattern of consecutive 0xFF bytes
            let mut consecutive_ff = 0;
            let mut max_consecutive = 0;
            for &byte in data {
                if byte == 0xFF {
                    consecutive_ff += 1;
                    max_consecutive = max_consecutive.max(consecutive_ff);
                } else {
                    consecutive_ff = 0;
                }
            }
            println!("  Max consecutive 0xFF: {}", max_consecutive);
            
            // Show first few bytes for pattern analysis
            if data.len() >= 16 {
                print!("  First 16 bytes: ");
                for i in 0..16 {
                    print!("{:02X} ", data[i]);
                }
                println!();
            }
        }
    }
    
    /// Count sync words in MP3 data
    pub fn count_sync_words(data: &[u8]) -> usize {
        let mut count = 0;
        for i in 0..data.len().saturating_sub(1) {
            let sync = ((data[i] as u16) << 3) | ((data[i + 1] as u16) >> 5);
            if sync == 0x7FF {
                count += 1;
            }
        }
        count
    }
    
    /// Analyze MP3 file structure
    pub fn analyze_mp3_structure(data: &[u8], label: &str) {
        println!("\n=== {} Structure Analysis ===", label);
        println!("Total size: {} bytes", data.len());
        
        if data.is_empty() {
            println!("Data is empty!");
            return;
        }
        
        // Show first 32 bytes in hex
        println!("First 32 bytes:");
        for (i, chunk) in data.chunks(16).take(2).enumerate() {
            print!("{:04X}: ", i * 16);
            for byte in chunk {
                print!("{:02X} ", byte);
            }
            println!();
        }
        
        // Find sync words and analyze frames
        let mut pos = 0;
        let mut frame_count = 0;
        
        while pos < data.len().saturating_sub(4) {
            let sync = ((data[pos] as u16) << 3) | ((data[pos + 1] as u16) >> 5);
            
            if sync == 0x7FF {
                frame_count += 1;
                println!("\n--- Frame {} at position {} ---", frame_count, pos);
                
                if pos + 4 < data.len() {
                    let header = ((data[pos] as u32) << 24) |
                                ((data[pos + 1] as u32) << 16) |
                                ((data[pos + 2] as u32) << 8) |
                                (data[pos + 3] as u32);
                    
                    let version = (header >> 19) & 0x3;
                    let bitrate_index = (header >> 12) & 0xF;
                    let sample_rate_index = (header >> 10) & 0x3;
                    let padding = (header >> 9) & 0x1;
                    let channel_mode = (header >> 6) & 0x3;
                    
                    println!("Header: {:08X}", header);
                    println!("Version: {}, Bitrate: {}, Sample rate: {}", 
                             version, bitrate_index, sample_rate_index);
                    println!("Padding: {}, Channel mode: {}", padding, channel_mode);
                    
                    // Calculate expected frame size
                    let frame_size = Self::calculate_frame_size(bitrate_index, sample_rate_index, padding, version);
                    println!("Expected frame size: {} bytes", frame_size);
                    
                    if frame_size > 0 && frame_size < 2000 {
                        pos += frame_size;
                    } else {
                        pos += 400; // Default skip
                    }
                } else {
                    pos += 1;
                }
                
                // Only analyze first few frames
                if frame_count >= 3 {
                    break;
                }
            } else {
                pos += 1;
            }
        }
        
        println!("\nTotal frames found: {}", frame_count);
    }
    
    /// Calculate MP3 frame size
    fn calculate_frame_size(bitrate_index: u32, sample_rate_index: u32, padding: u32, version: u32) -> usize {
        let bitrates = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
        let sample_rates = match version {
            3 => [44100, 48000, 32000, 0], // MPEG-1
            2 => [22050, 24000, 16000, 0], // MPEG-2
            0 => [11025, 12000, 8000, 0],  // MPEG-2.5
            _ => return 0,
        };
        
        if bitrate_index == 0 || bitrate_index == 15 || sample_rate_index == 3 {
            return 0;
        }
        
        let bitrate = bitrates[bitrate_index as usize] * 1000;
        let sample_rate = sample_rates[sample_rate_index as usize];
        
        if bitrate == 0 || sample_rate == 0 {
            return 0;
        }
        
        let samples_per_frame = if version == 3 { 1152 } else { 576 };
        let frame_size = (samples_per_frame * bitrate / sample_rate / 8) + padding as usize;
        
        frame_size
    }
}

/// Debug test runner
pub struct DebugRunner;

impl DebugRunner {
    /// Run comprehensive debug test
    pub fn run_debug_test(name: &str, config: DebugConfig, pcm_data: Vec<i16>) -> Result<(), String> {
        println!("=== Debug Test: {} ===", name);
        
        // Create output directory
        create_dir_all("tests/output").map_err(|e| format!("Failed to create output dir: {}", e))?;
        
        // Create encoder config
        let encoder_config = Config {
            wave: WaveConfig {
                channels: config.channels,
                sample_rate: config.sample_rate,
            },
            mpeg: MpegConfig {
                mode: config.mode,
                bitrate: config.bitrate,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(encoder_config)
            .map_err(|e| format!("Failed to create encoder: {:?}", e))?;
        
        let samples_per_frame = encoder.samples_per_frame();
        let channels = match config.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };
        let frame_size = samples_per_frame * channels;
        
        println!("Config: {}kbps, {}Hz, {} channels", 
                 config.bitrate, config.sample_rate, channels);
        println!("Samples per frame: {}, Frame size: {}", samples_per_frame, frame_size);
        println!("Sample range: {} to {}", 
                 pcm_data.iter().min().unwrap_or(&0), 
                 pcm_data.iter().max().unwrap_or(&0));
        
        let mut mp3_data = Vec::new();
        let mut frame_count = 0;
        
        // Encode frames
        for chunk in pcm_data.chunks(frame_size) {
            if chunk.len() == frame_size {
                match encoder.encode_frame_interleaved(chunk) {
                    Ok(frame_data) => {
                        frame_count += 1;
                        println!("Frame {}: {} bytes", frame_count, frame_data.len());
                        
                        // Analyze first frame in detail
                        if frame_count == 1 {
                            let analysis = FrameAnalyzer::analyze_frame(frame_data);
                            Self::print_frame_analysis(&analysis);
                            
                            // Additional pipeline analysis
                            PipelineAnalyzer::analyze_ff_pattern(frame_data, &format!("Frame {}", frame_count));
                        }
                        
                        mp3_data.extend_from_slice(frame_data);
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
        let output_path = format!("tests/output/debug_{}.mp3", name.replace(" ", "_").to_lowercase());
        let mut file = File::create(&output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        file.write_all(&mp3_data)
            .map_err(|e| format!("Failed to write MP3 data: {}", e))?;
        
        println!("Written to: {}", output_path);
        
        // Comprehensive analysis of the output
        PipelineAnalyzer::analyze_ff_pattern(&mp3_data, "Complete MP3");
        PipelineAnalyzer::analyze_mp3_structure(&mp3_data, "Output");
        
        let sync_count = PipelineAnalyzer::count_sync_words(&mp3_data);
        println!("Sync words found: {}", sync_count);
        
        // Validate with FFmpeg if available
        Self::validate_with_ffmpeg(&output_path);
        
        Ok(())
    }
    
    /// Run pipeline isolation test
    pub fn run_pipeline_isolation_test(name: &str) -> Result<(), String> {
        println!("=== Pipeline Isolation Test: {} ===", name);
        
        // Test different simple patterns to isolate pipeline issues
        let patterns = [
            ("silence", vec![0i16; 2304]),
            ("small_values", vec![1i16; 2304]),
            ("alternating", (0..2304).map(|i| (i % 2) as i16).collect()),
            ("constant_100", vec![100i16; 2304]),
        ];
        
        for (pattern_name, pcm_data) in patterns.iter() {
            println!("\n--- Testing pattern: {} ---", pattern_name);
            
            let config = DebugConfig {
                channels: Channels::Stereo,
                mode: StereoMode::Stereo,
                duration: 0.05, // Short duration
                ..Default::default()
            };
            
            let test_name = format!("{}_{}", name, pattern_name);
            let result = Self::run_debug_test(&test_name, config, pcm_data.clone());
            
            match result {
                Ok(()) => println!("✅ Pattern {} encoded successfully", pattern_name),
                Err(e) => {
                    println!("❌ Pattern {} failed: {}", pattern_name, e);
                    if *pattern_name == "silence" {
                        return Err(format!("Critical: Silence pattern should always work: {}", e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Print frame analysis results
    fn print_frame_analysis(analysis: &FrameAnalysis) {
        println!("\n--- Frame Analysis ---");
        println!("Size: {} bytes", analysis.frame_size);
        println!("Sync: 0x{:03X}", analysis.sync_word);
        println!("Version: {}, Layer: {}, Mode: {}", 
                 analysis.version, analysis.layer, analysis.channel_mode);
        
        if !analysis.errors.is_empty() {
            println!("Errors:");
            for error in &analysis.errors {
                println!("  ❌ {}", error);
            }
        }
        
        if let Some(ref side_info) = analysis.side_info_analysis {
            println!("\nSide Info:");
            println!("Main data begin: {}", side_info.main_data_begin);
            
            for granule in &side_info.granules {
                println!("Granule {} Ch {}: big_values={}, global_gain={}, part2_3_length={}", 
                         granule.granule, granule.channel, granule.big_values, 
                         granule.global_gain, granule.part2_3_length);
                
                if !granule.is_valid {
                    println!("  ❌ Invalid big_values: {}", granule.big_values);
                }
            }
            
            if !side_info.errors.is_empty() {
                for error in &side_info.errors {
                    println!("  ❌ {}", error);
                }
            }
        }
    }
    
    /// Validate MP3 file with FFmpeg
    fn validate_with_ffmpeg(mp3_path: &str) {
        println!("\n--- FFmpeg Validation ---");
        
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
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    println!("❌ FFmpeg validation failed:");
                    println!("{}", stderr);
                }
            },
            Err(e) => {
                println!("⚠ FFmpeg not available: {}", e);
            }
        }
    }
    
    /// Calculate frame size parameters
    pub fn debug_frame_size(config: &Config) {
        println!("=== Frame Size Debug ===");
        
        let bitrate = config.mpeg.bitrate * 1000;
        let sample_rate = config.wave.sample_rate;
        let granules_per_frame = match config.mpeg_version() {
            rust_mp3_encoder::config::MpegVersion::Mpeg1 => 2,
            rust_mp3_encoder::config::MpegVersion::Mpeg2 | 
            rust_mp3_encoder::config::MpegVersion::Mpeg25 => 1,
        };
        let granule_size = 576;
        let bits_per_slot = 8;
        
        let avg_slots_per_frame = ((granules_per_frame * granule_size) as f64 / sample_rate as f64) *
                                 (1000.0 * bitrate as f64 / bits_per_slot as f64);
        
        let whole_slots_per_frame = avg_slots_per_frame as usize;
        let frac_slots_per_frame = avg_slots_per_frame - whole_slots_per_frame as f64;
        
        println!("Config: {}kbps, {}Hz, {} channels", 
                 config.mpeg.bitrate, config.wave.sample_rate, config.wave.channels as u8);
        println!("Granules per frame: {}", granules_per_frame);
        println!("Granule size: {}", granule_size);
        println!("Avg slots per frame: {:.6}", avg_slots_per_frame);
        println!("Whole slots per frame: {}", whole_slots_per_frame);
        println!("Frac slots per frame: {:.6}", frac_slots_per_frame);
        println!("Target frame size: {} bytes ({} bits)", whole_slots_per_frame, whole_slots_per_frame * 8);
    }
}