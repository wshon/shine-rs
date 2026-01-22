//! Real audio file encoding tests
//!
//! These tests verify the encoder's ability to handle real audio files
//! and produce valid MP3 output that can be decoded by standard players.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::fs::{File, remove_file};
use std::io::{Read, Write};

/// Clean up old MP3 files before running tests
fn cleanup_output_files() {
    let output_dir = "tests/output";
    if let Ok(entries) = std::fs::read_dir(output_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "mp3" {
                        let _ = remove_file(&path);
                        println!("Cleaned up: {:?}", path);
                    }
                }
            }
        }
    }
}

/// Read WAV file and extract PCM data
/// This is a simplified WAV reader for testing purposes
fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32, u16), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Simple WAV header parsing (assumes standard format)
    if &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
        return Err("Invalid WAV file format".into());
    }
    
    // Find fmt chunk
    let mut pos = 12;
    while pos < buffer.len() - 8 {
        let chunk_id = &buffer[pos..pos+4];
        let chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
        
        if chunk_id == b"fmt " {
            let sample_rate = u32::from_le_bytes([buffer[pos+12], buffer[pos+13], buffer[pos+14], buffer[pos+15]]);
            let channels = u16::from_le_bytes([buffer[pos+10], buffer[pos+11]]);
            pos += 8 + chunk_size as usize;
            
            // Find data chunk
            while pos < buffer.len() - 8 {
                let data_chunk_id = &buffer[pos..pos+4];
                let data_chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
                
                if data_chunk_id == b"data" {
                    let data_start = pos + 8;
                    let data_end = data_start + data_chunk_size as usize;
                    
                    // Convert bytes to i16 samples
                    let mut samples = Vec::new();
                    for i in (data_start..data_end).step_by(2) {
                        if i + 1 < buffer.len() {
                            let sample = i16::from_le_bytes([buffer[i], buffer[i+1]]);
                            samples.push(sample);
                        }
                    }
                    
                    return Ok((samples, sample_rate, channels));
                }
                
                pos += 8 + data_chunk_size as usize;
            }
            break;
        }
        
        pos += 8 + chunk_size as usize;
    }
    
    Err("Could not parse WAV file".into())
}

#[test]
fn test_encode_real_wav_file() {
    cleanup_output_files();
    
    // Read the test WAV file
    let (pcm_data, sample_rate, channels) = read_wav_file("tests/input/sample-12s.wav")
        .expect("Failed to read WAV file");
    
    println!("WAV file info: {} Hz, {} channels, {} samples", 
             sample_rate, channels, pcm_data.len());
    
    // Create encoder configuration matching the WAV file
    let config = Config {
        wave: WaveConfig {
            channels: if channels == 1 { Channels::Mono } else { Channels::Stereo },
            sample_rate,
        },
        mpeg: MpegConfig {
            mode: if channels == 1 { StereoMode::Mono } else { StereoMode::JointStereo },
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    
    // Encode the audio in chunks
    let samples_per_frame = encoder.samples_per_frame();
    let frame_size = samples_per_frame * channels as usize;
    let mut mp3_data = Vec::new();
    
    println!("Encoding {} frames of {} samples each", 
             pcm_data.len() / frame_size, samples_per_frame);
    
    // Process complete frames
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            match encoder.encode_frame_interleaved(chunk) {
                Ok(frame_data) => {
                    mp3_data.extend_from_slice(frame_data);
                    println!("Encoded frame: {} bytes", frame_data.len());
                },
                Err(e) => {
                    panic!("Failed to encode frame: {:?}", e);
                }
            }
        } else {
            // Handle partial frame with encode_samples
            match encoder.encode_samples(chunk) {
                Ok(Some(frame_data)) => {
                    mp3_data.extend_from_slice(frame_data);
                    println!("Encoded partial frame: {} bytes", frame_data.len());
                },
                Ok(None) => {
                    println!("Partial frame buffered: {} samples", chunk.len());
                },
                Err(e) => {
                    panic!("Failed to encode partial frame: {:?}", e);
                }
            }
        }
    }
    
    // Flush any remaining data
    match encoder.flush() {
        Ok(final_data) => {
            if !final_data.is_empty() {
                mp3_data.extend_from_slice(final_data);
                println!("Flushed final data: {} bytes", final_data.len());
            }
        },
        Err(e) => {
            panic!("Failed to flush encoder: {:?}", e);
        }
    }
    
    println!("Total MP3 data: {} bytes", mp3_data.len());
    
    // Verify the MP3 data is not empty and has reasonable size
    assert!(!mp3_data.is_empty(), "MP3 output should not be empty");
    assert!(mp3_data.len() > 1000, "MP3 output should be substantial");
    
    // Verify MP3 sync words in the output
    let mut sync_count = 0;
    for i in 0..mp3_data.len().saturating_sub(1) {
        let sync = ((mp3_data[i] as u16) << 3) | ((mp3_data[i+1] as u16) >> 5);
        if sync == 0x7FF {
            sync_count += 1;
        }
    }
    
    println!("Found {} MP3 sync words", sync_count);
    assert!(sync_count > 0, "Should find at least one MP3 sync word");
    
    // Write output file for manual verification
    let mut output_file = File::create("tests/output/encoded_output.mp3")
        .expect("Failed to create output file");
    output_file.write_all(&mp3_data)
        .expect("Failed to write MP3 data");
    
    println!("MP3 file written to tests/output/encoded_output.mp3");
}

#[test]
fn test_encode_mono_configuration() {
    cleanup_output_files();
    
    // Create a simple mono test signal
    let sample_rate = 44100u32;
    let duration_seconds = 1.0;
    let samples_count = (sample_rate as f32 * duration_seconds) as usize;
    
    // Generate a 440Hz sine wave (A4 note)
    let mut pcm_data = Vec::with_capacity(samples_count);
    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 16000.0;
        pcm_data.push(sample as i16);
    }
    
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Mono,
            sample_rate,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Mono,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create mono encoder");
    
    // Encode in frames
    let samples_per_frame = encoder.samples_per_frame();
    let mut mp3_data = Vec::new();
    
    for chunk in pcm_data.chunks(samples_per_frame) {
        if chunk.len() == samples_per_frame {
            let frame_data = encoder.encode_frame(chunk)
                .expect("Failed to encode mono frame");
            mp3_data.extend_from_slice(frame_data);
        }
    }
    
    // Flush remaining data
    let final_data = encoder.flush().expect("Failed to flush mono encoder");
    mp3_data.extend_from_slice(final_data);
    
    assert!(!mp3_data.is_empty(), "Mono MP3 output should not be empty");
    
    // Write mono output
    let mut output_file = File::create("tests/output/mono_output.mp3")
        .expect("Failed to create mono output file");
    output_file.write_all(&mp3_data)
        .expect("Failed to write mono MP3 data");
    
    println!("Mono MP3 file written to tests/output/mono_output.mp3");
}

#[test]
fn test_different_sample_rates() {
    cleanup_output_files();
    
    let test_rates = vec![44100, 48000, 32000, 22050, 24000, 16000];
    
    for &sample_rate in &test_rates {
        println!("Testing sample rate: {} Hz", sample_rate);
        
        // Generate 0.5 second test signal
        let duration = 0.5;
        let samples_count = (sample_rate as f32 * duration) as usize;
        
        let mut pcm_data = Vec::with_capacity(samples_count * 2); // Stereo
        for i in 0..samples_count {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 1000.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
            let sample_i16 = sample as i16;
            pcm_data.push(sample_i16); // Left channel
            pcm_data.push(sample_i16); // Right channel
        }
        
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Stereo,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
        
        let samples_per_frame = encoder.samples_per_frame();
        let frame_size = samples_per_frame * 2; // Stereo
        let mut mp3_data = Vec::new();
        
        for chunk in pcm_data.chunks(frame_size) {
            if chunk.len() == frame_size {
                let frame_data = encoder.encode_frame_interleaved(chunk)
                    .expect("Failed to encode frame");
                mp3_data.extend_from_slice(frame_data);
            }
        }
        
        let final_data = encoder.flush().expect("Failed to flush");
        mp3_data.extend_from_slice(final_data);
        
        assert!(!mp3_data.is_empty(), "MP3 output should not be empty for {} Hz", sample_rate);
        
        // Write test output
        let filename = format!("tests/output/test_{}hz.mp3", sample_rate);
        let mut output_file = File::create(&filename)
            .expect("Failed to create test output file");
        output_file.write_all(&mp3_data)
            .expect("Failed to write test MP3 data");
        
        println!("Test file written: {}", filename);
    }
}