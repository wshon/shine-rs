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
    
    let mut total_bytes = 0;
    
    // Encode first 5 frames
    for frame_num in 1..=5 {
        let start_idx = (frame_num - 1) * frame_size;
        let end_idx = start_idx + frame_size;
        
        if end_idx <= pcm_data.len() {
            let frame_data = &pcm_data[start_idx..end_idx];
            let data_ptr = frame_data.as_ptr();
            
            let (mp3_data, written) = shine_encode_buffer_interleaved(&mut encoder, data_ptr)
                .expect("Failed to encode frame");
            
            println!("Frame {}: {} bytes, offset: 0x{:04X}-0x{:04X}", 
                     frame_num, written, total_bytes, total_bytes + written - 1);
            
            // Check frame header
            if written >= 4 {
                let header_bytes = [mp3_data[0], mp3_data[1], mp3_data[2], mp3_data[3]];
                let header = u32::from_be_bytes(header_bytes);
                let sync = (header >> 21) & 0x7FF;
                let padding = (header >> 9) & 0x1;
                
                if sync == 0x7FF {
                    println!("  ✓ Header: {:02X} {:02X} {:02X} {:02X}, padding={}", 
                             header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3], padding);
                } else {
                    println!("  ✗ Invalid header: {:02X} {:02X} {:02X} {:02X}, sync=0x{:X}", 
                             header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3], sync);
                }
            }
            
            total_bytes += written;
        }
    }
    
    println!("Total: {} bytes", total_bytes);
}