use rust_mp3_encoder::config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use rust_mp3_encoder::encoder::Mp3Encoder;

#[test]
fn debug_mp3_frame_structure() {
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
    
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    // Create simple test data - sine wave pattern
    let samples_per_frame = 1152;
    let channels = 2;
    let total_samples = samples_per_frame * channels;
    let pcm_data: Vec<i16> = (0..total_samples)
        .map(|i| ((i as f32 * 0.01).sin() * 1000.0) as i16)
        .collect();
    
    println!("Encoding frame with sine wave pattern...");
    let result = encoder.encode_frame(&pcm_data);
    
    match result {
        Ok(frame) => {
            println!("Success! Frame size: {} bytes", frame.len());
            
            if frame.len() >= 4 {
                // Parse MP3 frame header
                let header = ((frame[0] as u32) << 24) | 
                            ((frame[1] as u32) << 16) | 
                            ((frame[2] as u32) << 8) | 
                            (frame[3] as u32);
                
                let sync = (header >> 21) & 0x7FF;
                let version = (header >> 19) & 0x3;
                let layer = (header >> 17) & 0x3;
                let protection = (header >> 16) & 0x1;
                let bitrate_index = (header >> 12) & 0xF;
                let sample_rate_index = (header >> 10) & 0x3;
                let padding = (header >> 9) & 0x1;
                let private_bit = (header >> 8) & 0x1;
                let channel_mode = (header >> 6) & 0x3;
                let mode_extension = (header >> 4) & 0x3;
                let copyright = (header >> 3) & 0x1;
                let original = (header >> 2) & 0x1;
                let emphasis = header & 0x3;
                
                println!("MP3 Frame Header Analysis:");
                println!("  Sync: 0x{:03X} (expected: 0x7FF)", sync);
                println!("  Version: {} (3=MPEG1, 2=MPEG2, 0=MPEG2.5)", version);
                println!("  Layer: {} (1=Layer III)", layer);
                println!("  Protection: {} (0=CRC, 1=no CRC)", protection);
                println!("  Bitrate index: {}", bitrate_index);
                println!("  Sample rate index: {}", sample_rate_index);
                println!("  Padding: {}", padding);
                println!("  Private bit: {}", private_bit);
                println!("  Channel mode: {} (0=stereo, 1=joint, 2=dual, 3=mono)", channel_mode);
                println!("  Mode extension: {}", mode_extension);
                println!("  Copyright: {}", copyright);
                println!("  Original: {}", original);
                println!("  Emphasis: {}", emphasis);
                
                // Check if header is valid
                assert_eq!(sync, 0x7FF, "Invalid sync word");
                assert_eq!(version, 3, "Should be MPEG-1");
                assert_eq!(layer, 1, "Should be Layer III");
                
                // Print first few bytes of side info
                if frame.len() >= 36 {
                    println!("\nSide info bytes (first 32 after header):");
                    for i in 4..36 {
                        print!("{:02X} ", frame[i]);
                        if (i - 4) % 16 == 15 {
                            println!();
                        }
                    }
                    println!();
                }
                
                // Try to extract big_values from side info
                // For MPEG-1 stereo, side info starts at byte 4 and is 32 bytes long
                if frame.len() >= 36 {
                    // Granule 0, Channel 0 starts at byte 4
                    let granule_info_start = 4 + 9; // Skip main_data_begin and scfsi
                    
                    if frame.len() > granule_info_start + 12 {
                        // big_values is at bits 47-55 in granule info (9 bits)
                        let byte_offset = granule_info_start + 5; // Approximate position
                        if byte_offset + 1 < frame.len() {
                            let big_values_raw = ((frame[byte_offset] as u16) << 8) | (frame[byte_offset + 1] as u16);
                            println!("Raw bytes around big_values position: {:02X} {:02X}", frame[byte_offset], frame[byte_offset + 1]);
                            
                            // Extract 9 bits for big_values (this is approximate bit extraction)
                            let big_values = (big_values_raw >> 7) & 0x1FF; // Extract 9 bits
                            println!("Extracted big_values: {} (max allowed: 288)", big_values);
                            
                            if big_values > 288 {
                                println!("ERROR: big_values {} exceeds maximum 288!", big_values);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
            panic!("Encoding failed");
        }
    }
}