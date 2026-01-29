//! WAV to MP3 converter command line tool
//!
//! This tool converts WAV files to MP3 format using the shine-rs library.
//! It supports various sample rates, mono/stereo configurations, and bitrates.
//! Command line interface matches the original shine encoder.

use shine_rs::{
    shine_close, shine_encode_buffer_interleaved, shine_flush, shine_initialise,
    shine_set_config_mpeg_defaults, ShineConfig, ShineMpeg, ShineWave,
};
use shine_rs_cli::util::read_wav_file;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;

/// Stereo mode constants (matches shine's stereo modes)
const STEREO: i32 = 0; // stereo
const JOINT_STEREO: i32 = 1; // joint-stereo
const DUAL_CHANNEL: i32 = 2; // dual-channel
const MONO: i32 = 3; // mono

/// Command line arguments structure
struct Args {
    input_file: String,
    output_file: String,
    bitrate: i32,
    stereo_mode: i32,
    force_mono: bool,
    copyright: bool,
    quiet: bool,
    verbose: bool,
}

impl Args {
    /// Parse command line arguments (matches shine's argument parsing)
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 3 {
            return Err("".to_string()); // Empty error triggers usage display
        }

        let mut bitrate = 128; // Default bitrate
        let mut stereo_mode = STEREO; // Default stereo mode
        let mut force_mono = false;
        let mut copyright = false;
        let mut quiet = false;
        let mut verbose = false;

        let mut i = 1;

        // Parse options (flags starting with -)
        while i < args.len() && args[i].starts_with('-') && args[i] != "-" {
            let arg = &args[i];

            if arg.len() < 2 {
                return Err(format!("Invalid option: {}", arg));
            }

            match arg.chars().nth(1).unwrap() {
                'b' => {
                    // Bitrate option
                    i += 1;
                    if i >= args.len() {
                        return Err("Option -b requires a bitrate value".to_string());
                    }
                    bitrate = args[i]
                        .parse::<i32>()
                        .map_err(|_| format!("Invalid bitrate: {}", args[i]))?;
                }
                'm' => {
                    // Force mono
                    force_mono = true;
                }
                'j' => {
                    // Joint stereo
                    stereo_mode = JOINT_STEREO;
                }
                'd' => {
                    // Dual channel
                    stereo_mode = DUAL_CHANNEL;
                }
                'c' => {
                    // Copyright flag
                    copyright = true;
                }
                'q' => {
                    // Quiet mode
                    quiet = true;
                    verbose = false;
                }
                'v' => {
                    // Verbose mode
                    verbose = true;
                    quiet = false;
                }
                'h' => {
                    // Help
                    return Err("".to_string()); // Empty error triggers usage display
                }
                _ => {
                    return Err(format!("Unknown option: {}", arg));
                }
            }
            i += 1;
        }

        // Parse input and output files
        if i + 1 >= args.len() {
            return Err("".to_string()); // Empty error triggers usage display
        }

        let input_file: String = args[i].clone();
        let output_file: String = args[i + 1].clone();

        // Validate bitrate (matches shine's supported bitrates)
        if ![
            8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
        ]
        .contains(&bitrate)
        {
            return Err(format!(
                "Unsupported bitrate: {}. Supported: 8-320 kbps",
                bitrate
            ));
        }

        Ok(Args {
            input_file,
            output_file,
            bitrate,
            stereo_mode,
            force_mono,
            copyright,
            quiet,
            verbose,
        })
    }
}

/// Print usage information (matches shine's usage format)
fn print_usage() {
    println!("Usage: shineenc [options] <infile> <outfile>");
    println!();
    println!("Use \"-\" for standard input or output.");
    println!();
    println!("Options:");
    println!(" -h            this help message");
    println!(" -b <bitrate>  set the bitrate [8-320], default 128kbit");
    println!(" -m            force encoder to operate in mono");
    println!(" -c            set copyright flag, default off");
    println!(" -j            encode in joint stereo (stereo data only)");
    println!(" -d            encode in dual-channel (stereo data only)");
    println!(" -q            quiet mode");
    println!(" -v            verbose mode");
}

/// Print program name (matches shine's output)
fn print_name() {
    println!("shineenc (Rust version)");
}

/// Convert WAV file to MP3
fn convert_wav_to_mp3(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Determine if we should use quiet mode
    let quiet = args.quiet || args.output_file == "-";

    // Print header (matches shine output)
    if !quiet {
        print_name();
    }

    // Read WAV file
    let (pcm_data, sample_rate_i32, channels_i32) =
        read_wav_file(&args.input_file).map_err(|e| format!("Could not open WAVE file: {}", e))?;

    let sample_rate = sample_rate_i32 as u32;
    let channels = channels_i32 as u16;

    // Calculate duration (high precision floating point calculation)
    let data_chunk_length = pcm_data.len() * 2; // Convert samples to bytes (16-bit = 2 bytes per sample)
    let byte_rate = sample_rate * channels as u32 * 2; // fmt_chunk.byte_rate
    let duration = data_chunk_length as f64 / byte_rate as f64; // High precision calculation

    // Print WAV info (matches shine format - this happens in wave_open)
    if !quiet {
        let channel_str = if channels == 1 { "mono" } else { "stereo" };
        println!(
            "WAVE PCM Data, {} {}Hz 16bit, duration: {:02}:{:02}:{:02}",
            channel_str,
            sample_rate,
            (duration as u32) / 3600,
            ((duration as u32) % 3600) / 60,
            (duration as u32) % 60
        );
    }

    // Create encoder configuration
    let mut config = ShineConfig {
        wave: ShineWave {
            channels: channels as i32,
            samplerate: sample_rate as i32,
        },
        mpeg: ShineMpeg {
            mode: args.stereo_mode,
            bitr: args.bitrate,
            emph: 0,
            copyright: if args.copyright { 1 } else { 0 },
            original: 1,
        },
    };

    // Set default MPEG values
    shine_set_config_mpeg_defaults(&mut config.mpeg);
    config.mpeg.bitr = args.bitrate; // Override default bitrate

    // Force mono if requested
    if args.force_mono {
        config.wave.channels = 1;
    }

    // Set stereo mode based on channels (matches shine logic)
    if config.wave.channels > 1 {
        config.mpeg.mode = args.stereo_mode;
    } else {
        config.mpeg.mode = MONO;
    }

    let mut encoder = shine_initialise(&config)?;

    // Print some info about the file about to be created (matches shine's check_config)
    if !quiet {
        let version_names = ["2.5", "reserved", "II", "I"];
        let mode_names = ["stereo", "joint-stereo", "dual-channel", "mono"];
        let demp_names = ["none", "50/15us", "", "CITT"];

        // For now, assume MPEG-I (version 3 in array)
        println!(
            "MPEG-{} layer III, {}  Psychoacoustic Model: Shine",
            version_names[3], mode_names[config.mpeg.mode as usize]
        );
        println!(
            "Bitrate: {} kbps  De-emphasis: {}   {} {}",
            config.mpeg.bitr,
            demp_names[config.mpeg.emph as usize],
            if config.mpeg.original != 0 {
                "Original"
            } else {
                ""
            },
            if config.mpeg.copyright != 0 {
                "(C)"
            } else {
                ""
            }
        );
        println!(
            "Encoding \"{}\" to \"{}\"",
            args.input_file, args.output_file
        );
    }

    let start_time = std::time::Instant::now();

    // Open output file (matches shine's file handling)
    let mut output_file: Box<dyn Write> = if args.output_file == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(File::create(&args.output_file)?)
    };

    // Calculate samples per frame
    let samples_per_frame = 1152; // MPEG Layer III frame size
    let frame_size = samples_per_frame * channels as usize;
    let mut mp3_data = Vec::new();

    if args.verbose {
        println!();
        println!("=== Verbose Mode: Frame-by-Frame Encoding Details ===");
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
        frame_buffer[..current_frame_size]
            .copy_from_slice(&pcm_data[processed_samples..processed_samples + current_frame_size]);

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

                    output_file.write_all(&frame_data[..written])?;
                    mp3_data.extend_from_slice(&frame_data[..written]);
                    mp3_offset += written;
                } else if args.verbose {
                    println!(
                        "[Frame {}] PCM {}-{}, MP3 buffered",
                        frame_count + 1,
                        pcm_start,
                        pcm_end
                    );
                }

                frame_count += 1;
                processed_samples += current_frame_size;
            }
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
            println!(
                "[Flush] MP3 {} bytes @ 0x{:04X}-0x{:04X}, CRC32: 0x{:08X}",
                final_written,
                mp3_offset,
                mp3_offset + final_written - 1,
                final_checksum
            );
        }
        output_file.write_all(&final_data[..final_written])?;
        mp3_data.extend_from_slice(&final_data[..final_written]);
    }

    // Close encoder
    shine_close(encoder);

    let elapsed = start_time.elapsed();
    let realtime_factor = if elapsed.as_secs_f64() > 0.0 {
        duration / elapsed.as_secs_f64()
    } else {
        f64::INFINITY
    };

    // Print completion message (matches shine format)
    if !quiet {
        if realtime_factor.is_infinite() {
            println!(
                "Finished in {:02}:{:02}:{:02} (infx realtime)",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() % 3600) / 60,
                elapsed.as_secs() % 60
            );
        } else {
            println!(
                "Finished in {:02}:{:02}:{:02} ({:.1}x realtime)",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() % 3600) / 60,
                elapsed.as_secs() % 60,
                realtime_factor
            );
        }
    }

    if args.verbose {
        println!();
        println!("=== Additional Statistics ===");
        println!("Total frames encoded: {}", frame_count);
        println!(
            "Total MP3 bytes: {} (hex: 0x{:04X})",
            mp3_data.len(),
            mp3_data.len()
        );
        println!(
            "Average bytes per frame: {:.1}",
            mp3_data.len() as f64 / frame_count as f64
        );

        // Show first few bytes of MP3 data (header info)
        if mp3_data.len() >= 4 {
            println!(
                "MP3 header bytes: {:02X} {:02X} {:02X} {:02X} (at offset 0x0000)",
                mp3_data[0], mp3_data[1], mp3_data[2], mp3_data[3]
            );
        }

        // Calculate compression ratio (use data_chunk_length to match Shine's wave.length)
        let input_size = data_chunk_length; // This matches wave.length in Shine
        let compression_ratio = input_size as f64 / mp3_data.len() as f64;
        println!("Input size:  {} bytes", input_size);
        println!("Output size: {} bytes", mp3_data.len());
        println!("Compression: {:.1}:1", compression_ratio);
        println!(
            "Actual bitrate: {:.1} kbps",
            (mp3_data.len() as f64 * 8.0) / (duration * 1000.0)
        );
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
            if err.is_empty() {
                // Empty error means show usage
                print_usage();
            } else {
                eprintln!("Error: {}", err);
            }
            process::exit(1);
        }
    };

    // Check if input file exists (unless it's stdin)
    if args.input_file != "-" && !Path::new(&args.input_file).exists() {
        eprintln!("Could not open WAVE file");
        process::exit(1);
    }

    // Perform conversion
    if let Err(err) = convert_wav_to_mp3(args) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
