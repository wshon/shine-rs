use rust_mp3_encoder::config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use rust_mp3_encoder::encoder::Mp3Encoder;

#[test]
fn debug_side_info_writing() {
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
            
            if frame.len() >= 36 {
                // For MPEG-1 stereo, side info structure:
                // Bytes 4-5: main_data_begin (9 bits) + private_bits (3 bits) + scfsi ch0 (4 bits)
                // Byte 6: scfsi ch1 (4 bits) + granule 0 ch0 part2_3_length start (4 bits)
                // Then granule info continues...
                
                println!("\nDetailed side info analysis:");
                
                // Skip frame header (4 bytes)
                let side_info_start = 4;
                
                // main_data_begin (9 bits) + private_bits (3 bits) + scfsi (8 bits) = 20 bits total
                let mut bit_offset = side_info_start * 8 + 20;
                
                // Granule 0, Channel 0
                // Extract part2_3_length (12 bits)
                let part2_3_length = extract_bits(&frame, bit_offset, 12);
                bit_offset += 12;
                
                // Extract big_values (9 bits)
                let big_values = extract_bits(&frame, bit_offset, 9);
                bit_offset += 9;
                
                // Extract global_gain (8 bits)
                let global_gain = extract_bits(&frame, bit_offset, 8);
                
                println!("Granule 0, Channel 0:");
                println!("  part2_3_length: {}", part2_3_length);
                println!("  big_values: {} (max allowed: 288)", big_values);
                println!("  global_gain: {}", global_gain);
                
                if big_values > 288 {
                    println!("ERROR: big_values {} exceeds maximum 288!", big_values);
                    
                    // Let's also check the raw bytes
                    println!("Raw side info bytes:");
                    for i in side_info_start..std::cmp::min(side_info_start + 16, frame.len()) {
                        print!("{:02X} ", frame[i]);
                    }
                    println!();
                }
                
                // Also check other granules/channels - skip the rest of granule 0 ch0 info
                bit_offset += 4 + 1 + 5 + 4 + 3 + 1 + 1 + 1 + 2 + 1 + 1 + 2; // Skip remaining fields
                
                if bit_offset / 8 + 4 < frame.len() {
                    let part2_3_length_2 = extract_bits(&frame, bit_offset, 12);
                    bit_offset += 12;
                    let big_values_2 = extract_bits(&frame, bit_offset, 9);
                    
                    println!("Granule 0, Channel 1:");
                    println!("  part2_3_length: {}", part2_3_length_2);
                    println!("  big_values: {} (max allowed: 288)", big_values_2);
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
            panic!("Encoding failed");
        }
    }
}

fn extract_bits(data: &[u8], bit_offset: usize, num_bits: usize) -> u32 {
    let mut result = 0u32;
    
    for i in 0..num_bits {
        let byte_index = (bit_offset + i) / 8;
        let bit_index = 7 - ((bit_offset + i) % 8);
        
        if byte_index < data.len() {
            let bit = (data[byte_index] >> bit_index) & 1;
            result = (result << 1) | (bit as u32);
        }
    }
    
    result
}