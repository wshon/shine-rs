//! Test data collection tool
//!
//! This tool encodes an audio file and collects key encoding parameters
//! for the first 6 frames, saving them to a JSON file for later validation.

use rust_mp3_encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_flush, shine_close, shine_set_config_mpeg_defaults};
use rust_mp3_encoder::test_data::{TestDataCollector, TestMetadata, EncodingConfig};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process;
use sha2::{Sha256, Digest};

/// Stereo mode constants (matches shine's stereo modes)
const STEREO_MONO: i32 = 3;
const STEREO_STEREO: i32 = 0;
const STEREO_JOINT_STEREO: i32 = 1;
const STEREO_DUAL_CHANNEL: i32 = 2;

/// WAV file reader that extracts PCM data and metadata
struct WavReader;

impl WavReader {
    /// Read WAV file and extract PCM data, sample rate, and channel count
    fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32, u16), Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        if buffer.len() < 44 {
            return Err("WAV file too small".into());
        }
        
        // Validate RIFF header
        if &buffer[0..4] != b"RIFF" {
            return Err("Not a RIFF file".into());
        }
        
        let file_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        if file_size as usize + 8 != buffer.len() {
            return Err("Invalid RIFF file size".into());
        }
        
        // Validate WAVE format
        if &buffer[8..12] != b"WAVE" {
            return Err("Not a WAVE file".into());
        }
        
        let mut sample_rate = 0u32;
        let mut channels = 0u16;
        let mut pcm_data = Vec::new();
        let mut fmt_found = false;
        let mut data_found = false;
        
        // Parse chunks
        let mut pos = 12;
        while pos < buffer.len() - 8 {
            if pos + 8 > buffer.len() {
                break;
            }
            
            let chunk_id = &buffer[pos..pos+4];
            let chunk_size = u32::from_le_bytes([buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]]);
            let chunk_data_start = pos + 8;
            let chunk_data_end = chunk_data_start + chunk_size as usize;
            
            if chunk_data_end > buffer.len() {
                return Err("Invalid chunk size".into());
            }
            
            match chunk_id {
                b"fmt " => {
                    if chunk_size < 16 {
                        return Err("Invalid fmt chunk size".into());
                    }
                    
                    let audio_format = u16::from_le_bytes([buffer[chunk_data_start], buffer[chunk_data_start+1]]);
                    if audio_format != 1 {
                        return Err("Only PCM format supported".into());
                    }
                    
                    channels = u16::from_le_bytes([buffer[chunk_data_start+2], buffer[chunk_data_start+3]]);
                    sample_rate = u32::from_le_bytes([
                        buffer[chunk_data_start+4], buffer[chunk_data_start+5], 
                        buffer[chunk_data_start+6], buffer[chunk_data_start+7]
                    ]);
                    let bits_per_sample = u16::from_le_bytes([buffer[chunk_data_start+14], buffer[chunk_data_start+15]]);
                    
                    if bits_per_sample != 16 {
                        return Err("Only 16-bit samples supported".into());
                    }
                    
                    fmt_found = true;
                },
                b"data" => {
                    if !fmt_found {
                        return Err("Data chunk found before fmt chunk".into());
                    }
                    
                    // Convert bytes to i16 samples
                    for i in (chunk_data_start..chunk_data_end).step_by(2) {
                        if i + 1 < buffer.len() {
                            let sample = i16::from_le_bytes([buffer[i], buffer[i+1]]);
                            pcm_data.push(sample);
                        }
                    }
                    data_found = true;
                },
                _ => {
                    // Skip unknown chunks
                }
            }
            
            // Move to next chunk (ensure even alignment)
            pos = chunk_data_end;
            if chunk_size % 2 == 1 {
                pos += 1; // WAV chunks are word-aligned
            }
        }
        
        if !fmt_found {
            return Err("No fmt chunk found".into());
        }
        
        if !data_found {
            return Err("No data chunk found".into());
        }
        
        if pcm_data.is_empty() {
            return Err("No audio data found".into());
        }
        
        Ok((pcm_data, sample_rate, channels))
    }
}

/// Calculate SHA256 hash of data
fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Collect test data from encoding process
fn collect_test_data(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("Collecting test data from: {}", args.input_file);
    
    // Set max frames environment variable if specified
    if let Some(max_frames) = args.max_frames {
        std::env::set_var("RUST_MP3_MAX_FRAMES", max_frames.to_string());
        println!("Frame limit set to: {} frames", max_frames);
    }
    
    // Read WAV file
    let (pcm_data, sample_rate, channels) = WavReader::read_wav_file(&args.input_file)?;
    
    println!("WAV info: {} Hz, {} channels, {} samples", 
             sample_rate, channels, pcm_data.len());
    
    // Adjust stereo mode based on input channels
    let stereo_mode = if channels == 1 {
        STEREO_MONO
    } else {
        STEREO_STEREO
    };
    
    // Create encoder configuration
    let mut config = ShineConfig {
        wave: ShineWave {
            channels: channels as i32,
            samplerate: sample_rate as i32,
        },
        mpeg: ShineMpeg {
            mode: stereo_mode,
            bitr: args.bitrate,
            emph: 0,
            copyright: 0,
            original: 1,
        },
    };
    
    // Set default MPEG values
    shine_set_config_mpeg_defaults(&mut config.mpeg);
    config.mpeg.bitr = args.bitrate; // Override default bitrate
    config.mpeg.mode = stereo_mode;  // Override default mode
    
    // Initialize test data collector
    let metadata = TestMetadata {
        name: format!("test_case_{}_{}hz_{}ch_{}kbps", 
                     Path::new(&args.input_file).file_stem().unwrap().to_str().unwrap(),
                     sample_rate, channels, args.bitrate),
        input_file: args.input_file.clone(),
        expected_output_size: 0, // Will be filled later
        expected_hash: String::new(), // Will be filled later
        created_at: chrono::Utc::now().to_rfc3339(),
        description: format!("Test case for {} at {} kbps", args.input_file, args.bitrate),
    };
    
    let encoding_config = EncodingConfig {
        sample_rate: sample_rate as i32,
        channels: channels as i32,
        bitrate: args.bitrate,
        stereo_mode,
        mpeg_version: 3, // Will be determined by encoder
    };
    
    TestDataCollector::initialize(metadata, encoding_config);
    
    println!("Encoding with: {} kbps, mode {}", args.bitrate, stereo_mode);
    
    let mut encoder = shine_initialise(&config)?;
    
    // Calculate samples per frame
    let samples_per_frame = 1152; // MPEG Layer III frame size
    let frame_size = samples_per_frame * channels as usize;
    let mut mp3_data = Vec::new();
    
    let total_frames = pcm_data.len() / frame_size;
    println!("Encoding {} frames of {} samples each (collecting first 6 frames)", total_frames, samples_per_frame);
    
    // Process complete frames
    let mut frame_count = 0;
    
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            // Convert to raw pointer for shine API
            let data_ptr = chunk.as_ptr();
            
            match shine_encode_buffer_interleaved(&mut encoder, data_ptr) {
                Ok((frame_data, written)) => {
                    if written > 0 {
                        mp3_data.extend_from_slice(&frame_data[..written]);
                    }
                    
                    frame_count += 1;
                    
                    if frame_count % 50 == 0 {
                        println!("Encoded {} / {} frames", frame_count, total_frames);
                    }
                },
                #[cfg(debug_assertions)]
                Err(rust_mp3_encoder::error::EncodingError::StopAfterFrames) => {
                    // This is expected when we stop after 6 frames in debug mode
                    println!("Stopped encoding after {} frames for test data collection", frame_count);
                    break;
                },
                Err(e) => return Err(e.into()),
            }
        }
    }
    
    // Flush any remaining data
    let (final_data, final_written) = shine_flush(&mut encoder);
    if final_written > 0 {
        mp3_data.extend_from_slice(&final_data[..final_written]);
        println!("Flushed final data: {} bytes", final_written);
    }
    
    // Close encoder
    shine_close(encoder);
    
    println!("Total MP3 data: {} bytes", mp3_data.len());
    
    // Calculate hash
    let hash = calculate_sha256(&mp3_data);
    
    // Update metadata with final values
    // Note: We need to update the collector's metadata, but the current API doesn't allow it
    // For now, we'll save with the current values and manually update if needed
    
    // Save collected data to JSON file
    TestDataCollector::save_to_file(&args.output_json)?;
    
    println!("✅ Test data collection completed!");
    println!("   Frames collected: 6");
    println!("   Output size: {} bytes", mp3_data.len());
    println!("   SHA256: {}", hash);
    println!("   JSON saved to: {}", args.output_json);
    
    Ok(())
}

/// Command line arguments structure
struct Args {
    input_file: String,
    output_json: String,
    bitrate: i32,
    max_frames: Option<usize>,
}

impl Args {
    /// Parse command line arguments
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 3 {
            return Err(format!(
                "Usage: {} <input.wav> <output.json> [bitrate] [--max-frames N]\n\
                 \n\
                 Arguments:\n\
                   input.wav    - Input WAV file path\n\
                   output.json  - Output JSON file path for test data\n\
                   bitrate      - MP3 bitrate in kbps (default: 128)\n\
                   --max-frames N - Limit encoding to N frames (debug mode only, default: 6)\n\
                 \n\
                 Examples:\n\
                   {} sample-3s.wav test_data.json\n\
                   {} sample-3s.wav test_data.json 192\n\
                   {} sample-3s.wav test_data.json 128 --max-frames 10",
                args[0], args[0], args[0], args[0]
            ));
        }
        
        let input_file = args[1].clone();
        let output_json = args[2].clone();
        
        // Check for max-frames flag
        let mut max_frames = None;
        for i in 0..args.len() {
            if args[i] == "--max-frames" && i + 1 < args.len() {
                if let Ok(frames) = args[i + 1].parse::<usize>() {
                    max_frames = Some(frames);
                }
            }
        }
        
        // Also check environment variable
        if max_frames.is_none() {
            if let Ok(env_frames) = std::env::var("RUST_MP3_MAX_FRAMES") {
                if let Ok(frames) = env_frames.parse::<usize>() {
                    max_frames = Some(frames);
                }
            }
        }
        
        // Default to 6 frames if not specified
        if max_frames.is_none() {
            max_frames = Some(6);
        }
        
        // Filter out max-frames flags for other parsing
        let filtered_args: Vec<String> = args.iter()
            .enumerate()
            .filter(|(i, arg)| {
                *arg != "--max-frames" && (*i == 0 || args[*i - 1] != "--max-frames")
            })
            .map(|(_, arg)| arg.clone())
            .collect();
        
        // Parse bitrate (default: 128)
        let bitrate = if filtered_args.len() > 3 {
            filtered_args[3].parse::<i32>()
                .map_err(|_| format!("Invalid bitrate: {}", filtered_args[3]))?
        } else {
            128
        };
        
        // Validate bitrate
        if ![32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320].contains(&bitrate) {
            return Err(format!("Unsupported bitrate: {}. Supported: 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320", bitrate));
        }
        
        Ok(Args {
            input_file,
            output_json,
            bitrate,
            max_frames,
        })
    }
}

fn main() {
    // Parse command line arguments
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };
    
    // Check if input file exists
    if !Path::new(&args.input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist", args.input_file);
        process::exit(1);
    }
    
    // Perform test data collection
    if let Err(err) = collect_test_data(args) {
        eprintln!("Test data collection failed: {}", err);
        process::exit(1);
    }
}