//! WAV to MP3 converter command line tool
//!
//! This tool converts WAV files to MP3 format using the rust-mp3-encoder library.
//! It supports various sample rates, mono/stereo configurations, and bitrates.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process;

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

/// Command line arguments structure
struct Args {
    input_file: String,
    output_file: String,
    bitrate: u32,
    stereo_mode: StereoMode,
}

impl Args {
    /// Parse command line arguments
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 3 {
            return Err(format!(
                "Usage: {} <input.wav> <output.mp3> [bitrate] [stereo_mode]\n\
                 \n\
                 Arguments:\n\
                   input.wav    - Input WAV file path\n\
                   output.mp3   - Output MP3 file path\n\
                   bitrate      - MP3 bitrate in kbps (default: 128)\n\
                   stereo_mode  - Stereo mode: mono, stereo, joint_stereo, dual_channel (default: auto)\n\
                 \n\
                 Examples:\n\
                   {} input.wav output.mp3\n\
                   {} input.wav output.mp3 192\n\
                   {} input.wav output.mp3 128 joint_stereo",
                args[0], args[0], args[0], args[0]
            ));
        }
        
        let input_file = args[1].clone();
        let output_file = args[2].clone();
        
        // Parse bitrate (default: 128)
        let bitrate = if args.len() > 3 {
            args[3].parse::<u32>()
                .map_err(|_| format!("Invalid bitrate: {}", args[3]))?
        } else {
            128
        };
        
        // Validate bitrate
        if ![32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320].contains(&bitrate) {
            return Err(format!("Unsupported bitrate: {}. Supported: 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320", bitrate));
        }
        
        // Parse stereo mode (default: auto-detect based on channels)
        let stereo_mode = if args.len() > 4 {
            match args[4].to_lowercase().as_str() {
                "mono" => StereoMode::Mono,
                "stereo" => StereoMode::Stereo,
                "joint_stereo" => StereoMode::JointStereo,
                "dual_channel" => StereoMode::DualChannel,
                _ => return Err(format!("Invalid stereo mode: {}. Supported: mono, stereo, joint_stereo, dual_channel", args[4])),
            }
        } else {
            StereoMode::JointStereo // Will be adjusted based on input channels
        };
        
        Ok(Args {
            input_file,
            output_file,
            bitrate,
            stereo_mode,
        })
    }
}

/// Convert WAV file to MP3
fn convert_wav_to_mp3(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading WAV file: {}", args.input_file);
    
    // Read WAV file
    let (pcm_data, sample_rate, channels) = WavReader::read_wav_file(&args.input_file)?;
    
    println!("WAV info: {} Hz, {} channels, {} samples", 
             sample_rate, channels, pcm_data.len());
    
    // Determine channel configuration
    let wave_channels = if channels == 1 { 
        Channels::Mono 
    } else { 
        Channels::Stereo 
    };
    
    // Adjust stereo mode based on input channels
    let stereo_mode = if channels == 1 {
        StereoMode::Mono
    } else {
        args.stereo_mode
    };
    
    // Create encoder configuration
    let config = Config {
        wave: WaveConfig {
            channels: wave_channels,
            sample_rate,
        },
        mpeg: MpegConfig {
            mode: stereo_mode,
            bitrate: args.bitrate,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    println!("Encoding with: {} kbps, {:?} mode", args.bitrate, stereo_mode);
    
    let mut encoder = Mp3Encoder::new(config)?;
    
    // Encode audio in chunks
    let samples_per_frame = encoder.samples_per_frame();
    let frame_size = samples_per_frame * channels as usize;
    let mut mp3_data = Vec::new();
    
    let total_frames = pcm_data.len() / frame_size;
    println!("Encoding {} frames of {} samples each", total_frames, samples_per_frame);
    
    // Process complete frames
    let mut frame_count = 0;
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            let frame_data = if channels == 1 {
                encoder.encode_frame(chunk)?
            } else {
                encoder.encode_frame_interleaved(chunk)?
            };
            
            mp3_data.extend_from_slice(frame_data);
            frame_count += 1;
            
            if frame_count % 100 == 0 {
                println!("Encoded {} / {} frames", frame_count, total_frames);
            }
        } else {
            // Handle partial frame
            match encoder.encode_samples(chunk)? {
                Some(frame_data) => {
                    mp3_data.extend_from_slice(frame_data);
                    println!("Encoded partial frame: {} samples", chunk.len());
                },
                None => {
                    println!("Partial frame buffered: {} samples", chunk.len());
                }
            }
        }
    }
    
    // Flush any remaining data
    let final_data = encoder.flush()?;
    if !final_data.is_empty() {
        mp3_data.extend_from_slice(final_data);
        println!("Flushed final data: {} bytes", final_data.len());
    }
    
    println!("Total MP3 data: {} bytes", mp3_data.len());
    
    // Write MP3 file
    println!("Writing MP3 file: {}", args.output_file);
    let mut output_file = File::create(&args.output_file)?;
    output_file.write_all(&mp3_data)?;
    
    // Calculate compression ratio
    let input_size = pcm_data.len() * 2; // 16-bit samples
    let compression_ratio = input_size as f64 / mp3_data.len() as f64;
    
    println!("✅ Conversion completed successfully!");
    println!("   Input size:  {} bytes", input_size);
    println!("   Output size: {} bytes", mp3_data.len());
    println!("   Compression: {:.1}:1", compression_ratio);
    
    // Calculate duration
    let duration = pcm_data.len() as f64 / (sample_rate as f64 * channels as f64);
    println!("   Duration:    {:.2} seconds", duration);
    
    Ok(())
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
    
    // Perform conversion
    if let Err(err) = convert_wav_to_mp3(args) {
        eprintln!("Conversion failed: {}", err);
        process::exit(1);
    }
}