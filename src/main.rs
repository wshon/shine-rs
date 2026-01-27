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
use log::{info, debug, error};
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
    info!("Reading WAV file: {}", args.input_file);
    
    // Set max frames environment variable if specified
    if let Some(max_frames) = args.max_frames {
        std::env::set_var("RUST_MP3_MAX_FRAMES", max_frames.to_string());
        info!("Frame limit set to: {} frames", max_frames);
    }
    
    // Read WAV file
    let (pcm_data, sample_rate_i32, channels_i32) = read_wav_file(&args.input_file)
        .map_err(|e| format!("Failed to read WAV file: {}", e))?;
    
    let sample_rate = sample_rate_i32 as u32;
    let channels = channels_i32 as u16;
    
    info!("WAV info: {} Hz, {} channels, {} samples", 
             sample_rate, channels, pcm_data.len());
    
    // Adjust stereo mode based on input channels
    let stereo_mode = if channels == 1 {
        STEREO_MONO
    } else {
        args.stereo_mode
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
    
    info!("Encoding with: {} kbps, mode {}", args.bitrate, stereo_mode);
    
    let mut encoder = shine_initialise(&config)?;
    
    // Calculate samples per frame
    let samples_per_frame = 1152; // MPEG Layer III frame size
    let frame_size = samples_per_frame * channels as usize;
    let mut mp3_data = Vec::new();
    
    let total_frames = pcm_data.len() / frame_size;
    info!("Encoding {} frames of {} samples each", total_frames, samples_per_frame);
    
    if args.verbose {
        info!("\n=== Verbose Mode: Frame-by-Frame Encoding Details ===");
        info!("Format: [Frame #] PCM samples, MP3 bytes @ hex offset");
        info!("─────────────────────────────────────────────────────────────────");
    }
    
    // Process complete frames
    let mut frame_count = 0;
    let mut mp3_offset = 0;
    
    for chunk in pcm_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            // Convert to raw pointer for shine API
            let data_ptr = chunk.as_ptr();
            
            let pcm_start = frame_count * frame_size;
            let pcm_end = pcm_start + frame_size - 1;
            
            match unsafe { shine_encode_buffer_interleaved(&mut encoder, data_ptr) } {
                Ok((frame_data, written)) => {
                    if written > 0 {
                        if args.verbose {
                            debug!("[Frame {}] PCM {}-{}, MP3 {} bytes @ 0x{:04X}-0x{:04X}",
                                     frame_count + 1,
                                     pcm_start,
                                     pcm_end,
                                     written,
                                     mp3_offset,
                                     mp3_offset + written - 1);
                        }
                        
                        mp3_data.extend_from_slice(&frame_data[..written]);
                        mp3_offset += written;
                    } else if args.verbose {
                        debug!("[Frame {}] PCM {}-{}, MP3 buffered",
                                 frame_count + 1,
                                 pcm_start,
                                 pcm_end);
                    }
                    
                    frame_count += 1;
                    
                    if !args.verbose && frame_count % 100 == 0 {
                        info!("Encoded {} / {} frames", frame_count, total_frames);
                    }
                },
                #[cfg(debug_assertions)]
                Err(shine_rs::error::EncodingError::StopAfterFrames) => {
                    // This is expected when we stop after a certain number of frames in debug mode
                    info!("Stopped encoding after {} frames as requested", frame_count);
                    break;
                },
                Err(e) => return Err(e.into()),
            }
        }
    }
    
    if args.verbose {
        info!("─────────────────────────────────────────────────────────────────");
    }
    
    // Flush any remaining data
    let (final_data, final_written) = shine_flush(&mut encoder);
    if final_written > 0 {
        if args.verbose {
            debug!("[Flush] MP3 {} bytes @ 0x{:04X}-0x{:04X}",
                     final_written,
                     mp3_offset,
                     mp3_offset + final_written - 1);
        }
        mp3_data.extend_from_slice(&final_data[..final_written]);
        info!("Flushed final data: {} bytes", final_written);
    }
    
    // Close encoder
    shine_close(encoder);
    
    info!("Total MP3 data: {} bytes", mp3_data.len());
    
    if args.verbose {
        info!("\n=== MP3 File Structure Summary ===");
        info!("Total frames encoded: {}", frame_count);
        info!("Total MP3 bytes: {} (hex: 0x{:04X})", mp3_data.len(), mp3_data.len());
        info!("Average bytes per frame: {:.1}", mp3_data.len() as f64 / frame_count as f64);
        
        // Show first few bytes of MP3 data (header info)
        if mp3_data.len() >= 4 {
            info!("MP3 header bytes: {:02X} {:02X} {:02X} {:02X} (at offset 0x0000)", 
                     mp3_data[0], mp3_data[1], mp3_data[2], mp3_data[3]);
        }
        
        // Show file structure breakdown
        if mp3_data.len() > 0 {
            info!("File range: 0x0000 - 0x{:04X}", mp3_data.len() - 1);
        }
    }
    
    // Write MP3 file
    info!("Writing MP3 file: {}", args.output_file);
    let mut output_file = File::create(&args.output_file)?;
    output_file.write_all(&mp3_data)?;
    
    if args.verbose {
        info!("Successfully wrote {} bytes (0x0000-0x{:04X}) to {}", 
                 mp3_data.len(), mp3_data.len() - 1, args.output_file);
    }
    
    // Calculate compression ratio
    let input_size = pcm_data.len() * 2; // 16-bit samples
    let compression_ratio = input_size as f64 / mp3_data.len() as f64;
    
    // Calculate duration
    let duration = pcm_data.len() as f64 / (sample_rate as f64 * channels as f64);
    
    // Final success message and statistics (always show to user)
    println!("✅ Conversion completed successfully!");
    println!("   Input size:  {} bytes", input_size);
    println!("   Output size: {} bytes", mp3_data.len());
    println!("   Compression: {:.1}:1", compression_ratio);
    println!("   Duration:    {:.2} seconds", duration);
    
    if args.verbose {
        info!("   Bitrate:     {} kbps (configured)", args.bitrate);
        info!("   Actual rate: {:.1} kbps (calculated)", 
                 (mp3_data.len() as f64 * 8.0) / (duration * 1000.0));
    }
    
    Ok(())
}

fn main() {
    // Initialize logger with configurable level
    // Set RUST_LOG=debug to see debug messages, RUST_LOG=info for info level (default)
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();
    
    // Parse command line arguments
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            error!("Error: {}", err);
            process::exit(1);
        }
    };
    
    // Check if input file exists
    if !Path::new(&args.input_file).exists() {
        error!("Input file '{}' does not exist", args.input_file);
        process::exit(1);
    }
    
    // Perform conversion
    if let Err(err) = convert_wav_to_mp3(args) {
        error!("Conversion failed: {}", err);
        process::exit(1);
    }
}