//! WAV to MP3 converter command line tool
//!
//! This tool converts WAV files to MP3 format using the shine-rs library.
//! It supports various sample rates, mono/stereo configurations, and bitrates.

use shine_rs::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_flush, shine_close, shine_set_config_mpeg_defaults};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;
use shine_rs_cli::util::read_wav_file;

/// Stereo mode constants (matches shine's stereo modes)
const STEREO_MONO: i32 = 3;
const STEREO_STEREO: i32 = 0;
const STEREO_JOINT_STEREO: i32 = 1;
const STEREO_DUAL_CHANNEL: i32 = 2;

/// Command line arguments structure
struct Args {
    input_file: String,
    output_file: String,
    bitrate: i32,
    stereo_mode: i32,
    verbose: bool,
    max_frames: Option<usize>,
}

impl Args {
    /// Parse command line arguments
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 3 {
            return Err(format!(
                "Usage: {} <input.wav> <output.mp3> [bitrate] [stereo_mode] [--verbose] [--max-frames N]\n\
                 \n\
                 Arguments:\n\
                   input.wav    - Input WAV file path\n\
                   output.mp3   - Output MP3 file path\n\
                   bitrate      - MP3 bitrate in kbps (default: 128)\n\
                   stereo_mode  - Stereo mode: mono, stereo, joint_stereo, dual_channel (default: auto)\n\
                   --verbose    - Enable verbose output with frame details\n\
                   --max-frames N - Limit encoding to N frames (debug mode only)\n\
                 \n\
                 Examples:\n\
                   {} input.wav output.mp3\n\
                   {} input.wav output.mp3 192\n\
                   {} input.wav output.mp3 128 joint_stereo\n\
                   {} input.wav output.mp3 128 joint_stereo --verbose\n\
                   {} input.wav output.mp3 128 stereo --max-frames 10",
                args[0], args[0], args[0], args[0], args[0], args[0]
            ));
        }
        
        let input_file = args[1].clone();
        let output_file = args[2].clone();
        
        // Check for verbose flag
        let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
        
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
        
        // Filter out verbose and max-frames flags for other parsing
        let filtered_args: Vec<String> = args.iter()
            .enumerate()
            .filter(|(i, arg)| {
                *arg != "--verbose" && *arg != "-v" && *arg != "--max-frames" &&
                (*i == 0 || args[*i - 1] != "--max-frames")
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
        
        // Parse stereo mode (default: auto-detect based on channels)
        let stereo_mode = if filtered_args.len() > 4 {
            match filtered_args[4].to_lowercase().as_str() {
                "mono" => STEREO_MONO,
                "stereo" => STEREO_STEREO,
                "joint_stereo" => STEREO_JOINT_STEREO,
                "dual_channel" => STEREO_DUAL_CHANNEL,
                _ => return Err(format!("Invalid stereo mode: {}. Supported: mono, stereo, joint_stereo, dual_channel", filtered_args[4])),
            }
        } else {
            STEREO_STEREO // Default to stereo mode (matches shine default)
        };
        
        Ok(Args {
            input_file,
            output_file,
            bitrate,
            stereo_mode,
            verbose,
            max_frames,
        })
    }
}

/// Convert WAV file to MP3
fn convert_wav_to_mp3(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Print header (matches shine output)
    println!("shineenc (Rust version)");
    
    // Set max frames environment variable if specified
    if let Some(max_frames) = args.max_frames {
        std::env::set_var("RUST_MP3_MAX_FRAMES", max_frames.to_string());
    }
    
    // Read WAV file
    let (pcm_data, sample_rate_i32, channels_i32) = read_wav_file(&args.input_file)
        .map_err(|e| format!("Failed to read WAV file: {}", e))?;
    
    let sample_rate = sample_rate_i32 as u32;
    let channels = channels_i32 as u16;
    
    // Calculate duration (high precision floating point calculation)
    let data_chunk_length = pcm_data.len() * 2; // Convert samples to bytes (16-bit = 2 bytes per sample)
    let byte_rate = sample_rate * channels as u32 * 2; // fmt_chunk.byte_rate
    let duration = data_chunk_length as f64 / byte_rate as f64; // High precision calculation
    let duration_minutes = (duration / 60.0) as u32;
    let duration_seconds = (duration % 60.0) as u32;
    
    // Print WAV info (matches shine format - this happens in wave_open)
    let channel_str = if channels == 1 { "mono" } else { "stereo" };
    println!("WAVE PCM Data, {} {}Hz 16bit, duration: {:02}:{:02}:{:02}", 
             channel_str, sample_rate, duration_minutes / 60, duration_minutes % 60, duration_seconds);
    
    // Adjust stereo mode based on input channels
    let stereo_mode = if channels == 1 {
        STEREO_MONO
    } else {
        args.stereo_mode
    };
    
    // Print MPEG info (matches shine format - this happens in check_config)
    let mpeg_mode_str = match stereo_mode {
        STEREO_MONO => "mono",
        STEREO_STEREO => "stereo",
        STEREO_JOINT_STEREO => "joint stereo",
        STEREO_DUAL_CHANNEL => "dual channel",
        _ => "stereo",
    };
    println!("MPEG-I layer III, {}  Psychoacoustic Model: Shine", mpeg_mode_str);
    println!("Bitrate: {} kbps  De-emphasis: none   Original", args.bitrate);
    println!("Encoding \"{}\" to \"{}\"", args.input_file, args.output_file);
    
    let start_time = std::time::Instant::now();
    
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
    
    let mut encoder = shine_initialise(&config)?;
    
    // Calculate samples per frame
    let samples_per_frame = 1152; // MPEG Layer III frame size
    let frame_size = samples_per_frame * channels as usize;
    let mut mp3_data = Vec::new();
    
    let _total_frames = pcm_data.len() / frame_size;
    
    if args.verbose {
        println!("\n=== Verbose Mode: Frame-by-Frame Encoding Details ===");
        println!("Format: [Frame #] PCM samples, MP3 bytes @ hex offset, CRC32 checksum");
        println!("-------------------------------------------------------------------------------");
    }
    
    // Process complete frames
    let mut frame_count = 0;
    let mut mp3_offset = 0;
    let mut processed_samples = 0;
    
    // Process all data, including incomplete last frame (matches Shine behavior)
    while processed_samples < pcm_data.len() {
        let remaining_samples = pcm_data.len() - processed_samples;
        let current_frame_size = std::cmp::min(frame_size, remaining_samples);
        
        // Create buffer for this frame, pad with zeros if incomplete (matches Shine)
        let mut frame_buffer = vec![0i16; frame_size];
        frame_buffer[..current_frame_size].copy_from_slice(&pcm_data[processed_samples..processed_samples + current_frame_size]);
        
        // Convert to raw pointer for shine API
        let data_ptr = frame_buffer.as_ptr();
        
        // Calculate PCM range (matches Shine's samples_per_pass calculation)
        let pcm_start = frame_count * samples_per_frame;
        let pcm_end = pcm_start + samples_per_frame - 1;
        
        match unsafe { shine_encode_buffer_interleaved(&mut encoder, data_ptr) } {
            Ok((frame_data, written)) => {
                if written > 0 {
                    // Calculate frame checksum (CRC32)
                    let frame_checksum = crc32fast::hash(&frame_data[..written]);
                    
                    if args.verbose {
                        println!("[Frame {}] PCM {}-{}, MP3 {} bytes @ 0x{:04X}-0x{:04X}, CRC32: 0x{:08X}",
                                 frame_count + 1,
                                 pcm_start,
                                 pcm_end,
                                 written,
                                 mp3_offset,
                                 mp3_offset + written - 1,
                                 frame_checksum);
                    }
                    
                    mp3_data.extend_from_slice(&frame_data[..written]);
                    mp3_offset += written;
                } else if args.verbose {
                    println!("[Frame {}] PCM {}-{}, MP3 buffered",
                             frame_count + 1,
                             pcm_start,
                             pcm_end);
                }
                
                frame_count += 1;
                processed_samples += current_frame_size;
            },
            #[cfg(debug_assertions)]
            Err(shine_rs::error::EncodingError::StopAfterFrames) => {
                // This is expected when we stop after a certain number of frames in debug mode
                if args.verbose {
                    println!("Stopped encoding after {} frames as requested", frame_count);
                }
                break;
            },
            Err(e) => return Err(e.into()),
        }
    }
    
    if args.verbose {
        println!("-------------------------------------------------------------------------------");
    }
    
    // Flush any remaining data
    let (final_data, final_written) = shine_flush(&mut encoder);
    if final_written > 0 {
        if args.verbose {
            let final_checksum = crc32fast::hash(&final_data[..final_written]);
            println!("[Flush] MP3 {} bytes @ 0x{:04X}-0x{:04X}, CRC32: 0x{:08X}",
                     final_written,
                     mp3_offset,
                     mp3_offset + final_written - 1,
                     final_checksum);
        }
        mp3_data.extend_from_slice(&final_data[..final_written]);
    }
    
    // Close encoder
    shine_close(encoder);
    
    // Write MP3 file
    let mut output_file = File::create(&args.output_file)?;
    output_file.write_all(&mp3_data)?;
    
    let elapsed = start_time.elapsed();
    let realtime_factor = if elapsed.as_secs_f64() > 0.0 {
        duration / elapsed.as_secs_f64()
    } else {
        f64::INFINITY
    };
    
    // Print completion message (matches shine format)
    if realtime_factor.is_infinite() {
        println!("Finished in {:02}:{:02}:{:02} (infx realtime)", 
                 elapsed.as_secs() / 3600, 
                 (elapsed.as_secs() % 3600) / 60, 
                 elapsed.as_secs() % 60);
    } else {
        println!("Finished in {:02}:{:02}:{:02} ({:.1}x realtime)", 
                 elapsed.as_secs() / 3600, 
                 (elapsed.as_secs() % 3600) / 60, 
                 elapsed.as_secs() % 60,
                 realtime_factor);
    }
    
    if args.verbose {
        println!("\n=== Additional Statistics ===");
        println!("Total frames encoded: {}", frame_count);
        println!("Total MP3 bytes: {} (hex: 0x{:04X})", mp3_data.len(), mp3_data.len());
        println!("Average bytes per frame: {:.1}", mp3_data.len() as f64 / frame_count as f64);
        
        // Show first few bytes of MP3 data (header info)
        if mp3_data.len() >= 4 {
            println!("MP3 header bytes: {:02X} {:02X} {:02X} {:02X} (at offset 0x0000)", 
                     mp3_data[0], mp3_data[1], mp3_data[2], mp3_data[3]);
        }
        
        // Calculate compression ratio (use data_chunk_length to match Shine's wave.length)
        let input_size = data_chunk_length; // This matches wave.length in Shine
        let compression_ratio = input_size as f64 / mp3_data.len() as f64;
        println!("Input size:  {} bytes", input_size);
        println!("Output size: {} bytes", mp3_data.len());
        println!("Compression: {:.1}:1", compression_ratio);
        println!("Actual bitrate: {:.1} kbps", 
                 (mp3_data.len() as f64 * 8.0) / (duration * 1000.0));
    }
    
    Ok(())
}

fn main() {
    // Initialize logger with minimal output (only errors by default)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error) // Only show errors by default
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();
    
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
        eprintln!("Input file '{}' does not exist", args.input_file);
        process::exit(1);
    }
    
    // Perform conversion
    if let Err(err) = convert_wav_to_mp3(args) {
        eprintln!("Conversion failed: {}", err);
        process::exit(1);
    }
}