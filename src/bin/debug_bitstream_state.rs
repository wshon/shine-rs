use rust_mp3_encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_set_config_mpeg_defaults};
use std::fs::File;
use std::io::Read;

fn main() {
    // Read WAV file
    let mut file = File::open("../../tests/input/sample-3s.wav").expect("Failed to open WAV file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read WAV file");
    
    // Skip WAV header and get PCM data (simplified)
    let pcm_start = 44; // Standard WAV header size
    let pcm_data: Vec<i16> = buffer[pcm_start..]
        .chunks(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    
    // Create encoder configuration
    let mut config = ShineConfig {
        wave: ShineWave {
            channels: 2,
            samplerate: 44100,
        },
        mpeg: ShineMpeg {
            mode: 0, // Stereo mode
            bitr: 128,
            emph: 0,
            copyright: 0,
            original: 1,
        },
    };
    
    shine_set_config_mpeg_defaults(&mut config.mpeg);
    config.mpeg.bitr = 128;
    config.mpeg.mode = 0;
    
    let mut encoder = shine_initialise(&config).expect("Failed to initialize encoder");
    
    let samples_per_frame = 1152;
    let frame_size = samples_per_frame * 2; // stereo
    
    // Encode first 3 frames and track bitstream state
    for frame_num in 1..=3 {
        println!("\n=== Frame {} ===", frame_num);
        
        // Print bitstream state before encoding
        println!("Before encoding:");
        println!("  data_position: {}", encoder.bs.data_position);
        println!("  cache_bits: {}", encoder.bs.cache_bits);
        println!("  cache: 0x{:08X}", encoder.bs.cache);
        println!("  bits_count: {}", encoder.bs.get_bits_count());
        
        let start_idx = (frame_num - 1) * frame_size;
        let end_idx = start_idx + frame_size;
        
        if end_idx <= pcm_data.len() {
            let frame_data = &pcm_data[start_idx..end_idx];
            let data_ptr = frame_data.as_ptr();
            
            // Store state before encoding
            let before_data_pos = encoder.bs.data_position;
            let before_cache_bits = encoder.bs.cache_bits;
            let before_cache = encoder.bs.cache;
            let before_bits_count = encoder.bs.get_bits_count();
            
            let (mp3_data, written) = shine_encode_buffer_interleaved(&mut encoder, data_ptr)
                .expect("Failed to encode frame");
            
            // Copy the MP3 data to avoid borrowing issues
            let mp3_copy: Vec<u8> = mp3_data.to_vec();
            
            // Print bitstream state after encoding
            println!("After encoding:");
            println!("  data_position: {} -> {}", before_data_pos, encoder.bs.data_position);
            println!("  cache_bits: {} -> {}", before_cache_bits, encoder.bs.cache_bits);
            println!("  cache: 0x{:08X} -> 0x{:08X}", before_cache, encoder.bs.cache);
            println!("  bits_count: {} -> {}", before_bits_count, encoder.bs.get_bits_count());
            println!("  written: {} bytes", written);
            
            // Check frame header
            if written >= 4 {
                let header_bytes = [mp3_copy[0], mp3_copy[1], mp3_copy[2], mp3_copy[3]];
                let header = u32::from_be_bytes(header_bytes);
                let sync = (header >> 21) & 0x7FF;
                
                if sync == 0x7FF {
                    println!("  ✓ Valid header: {:02X} {:02X} {:02X} {:02X}", 
                             header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3]);
                } else {
                    println!("  ✗ Invalid header: {:02X} {:02X} {:02X} {:02X}, sync=0x{:X}", 
                             header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3], sync);
                }
            }
        }
    }
}