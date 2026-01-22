//! FFmpeg validation tests
//!
//! These tests verify that the generated MP3 files can be correctly decoded
//! by FFmpeg and other standard MP3 decoders.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::process::Command;

/// Generate a test MP3 file and validate it with FFmpeg
fn create_and_validate_mp3(filename: &str, config: Config, duration_seconds: f32) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure output directory exists
    create_dir_all("tests/output")?;
    
    // Generate test audio data
    let sample_rate = config.wave.sample_rate;
    let channels = config.wave.channels as usize;
    let samples_count = (sample_rate as f32 * duration_seconds) as usize;
    
    let mut pcm_data = Vec::with_capacity(samples_count * channels);
    
    // Generate a mix of frequencies for better testing
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        // Mix of 440Hz and 880Hz sine waves
        let sample1 = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
        let sample2 = (t * 880.0 * 2.0 * std::f32::consts::PI).sin() * 4000.0;
        let mixed_sample = (sample1 + sample2) as i16;
        
        if channels == 1 {
            pcm_data.push(mixed_sample);
        } else {
            pcm_data.push(mixed_sample); // Left channel
            pcm_data.push(mixed_sample); // Right channel
        }
    }
    
    // Create encoder and encode the audio
    let mut encoder = Mp3Encoder::new(config)?;
    let samples_per_frame = encoder.samples_per_frame();
    let frame_size = samples_per_frame * channels;
    let mut mp3_data = Vec::new();
    
    println!("Encoding {} samples in frames of {} samples each", 
             pcm_data.len(), frame_size);
    
    // Encode complete frames
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            let frame_data = if channels == 1 {
                encoder.encode_frame(chunk)?
            } else {
                encoder.encode_frame_interleaved(chunk)?
            };
            mp3_data.extend_from_slice(frame_data);
        }
    }
    
    // Flush remaining data
    let final_data = encoder.flush()?;
    mp3_data.extend_from_slice(final_data);
    
    // Write MP3 file
    let filepath = format!("tests/output/{}", filename);
    let mut file = File::create(&filepath)?;
    file.write_all(&mp3_data)?;
    
    println!("Created MP3 file: {} ({} bytes)", filepath, mp3_data.len());
    
    // Validate with FFmpeg
    validate_mp3_with_ffmpeg(&filepath)?;
    
    Ok(())
}

/// Validate MP3 file using FFmpeg
fn validate_mp3_with_ffmpeg(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating {} with FFmpeg...", filepath);
    
    // First, try to get basic info about the file
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration,bit_rate,format_name",
            "-show_entries", "stream=codec_name,sample_rate,channels,bit_rate",
            "-of", "csv=p=0",
            filepath
        ])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                let info = String::from_utf8_lossy(&result.stdout);
                println!("FFprobe info: {}", info.trim());
            } else {
                let error = String::from_utf8_lossy(&result.stderr);
                println!("FFprobe error: {}", error);
                return Err(format!("FFprobe failed: {}", error).into());
            }
        },
        Err(e) => {
            println!("FFprobe not available: {}", e);
            // Continue with ffmpeg decode test
        }
    }
    
    // Try to decode the MP3 file to WAV to verify it's valid
    let output_wav = format!("{}.decoded.wav", filepath);
    let decode_result = Command::new("ffmpeg")
        .args(&[
            "-y", // Overwrite output file
            "-v", "error", // Only show errors
            "-i", filepath,
            "-f", "wav",
            &output_wav
        ])
        .output();
    
    match decode_result {
        Ok(result) => {
            if result.status.success() {
                println!("✓ FFmpeg successfully decoded {} to {}", filepath, output_wav);
                
                // Clean up the decoded WAV file
                let _ = std::fs::remove_file(&output_wav);
                
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&result.stderr);
                println!("✗ FFmpeg decode error: {}", error);
                
                // Check for specific error patterns
                if error.contains("big_values") {
                    return Err("FFmpeg reports big_values error - Huffman encoding issue".into());
                } else if error.contains("Invalid data found") {
                    return Err("FFmpeg reports invalid data - bitstream format issue".into());
                } else if error.contains("Header missing") {
                    return Err("FFmpeg reports missing header - frame header issue".into());
                } else {
                    return Err(format!("FFmpeg decode failed: {}", error).into());
                }
            }
        },
        Err(e) => {
            println!("FFmpeg not available: {}", e);
            // If FFmpeg is not available, we can't validate but shouldn't fail the test
            println!("⚠ Skipping FFmpeg validation (FFmpeg not installed)");
            Ok(())
        }
    }
}

#[test]
fn test_ffmpeg_validation_mono_128kbps() {
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
    
    create_and_validate_mp3("ffmpeg_test_mono_128.mp3", config, 2.0)
        .expect("Failed to create and validate mono 128kbps MP3");
}

#[test]
fn test_ffmpeg_validation_stereo_128kbps() {
    let config = Config {
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
    };
    
    create_and_validate_mp3("ffmpeg_test_stereo_128.mp3", config, 2.0)
        .expect("Failed to create and validate stereo 128kbps MP3");
}

#[test]
fn test_ffmpeg_validation_joint_stereo() {
    let config = Config {
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
    };
    
    create_and_validate_mp3("ffmpeg_test_joint_stereo.mp3", config, 2.0)
        .expect("Failed to create and validate joint stereo MP3");
}