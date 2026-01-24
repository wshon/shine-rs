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
    
    // Encode only the first frame
    let frame_data = &pcm_data[0..frame_size];
    let data_ptr = frame_data.as_ptr();
    
    let (mp3_data, written) = shine_encode_buffer_interleaved(&mut encoder, data_ptr)
        .expect("Failed to encode frame");
    
    println!("First frame: {} bytes", written);
    println!("First 32 bytes: {:02X?}", &mp3_data[..32.min(written)]);
    
    // Check frame header
    if written >= 4 {
        let header_bytes = [mp3_data[0], mp3_data[1], mp3_data[2], mp3_data[3]];
        println!("Frame header bytes: {:02X} {:02X} {:02X} {:02X}", 
                 header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3]);
        
        let header = u32::from_be_bytes(header_bytes);
        println!("Frame header: 0x{:08X}", header);
        
        // Check if it starts with sync word
        let sync = (header >> 21) & 0x7FF;
        if sync == 0x7FF {
            println!("✓ Correct sync word found");
        } else {
            println!("✗ Invalid sync word: 0x{:X}", sync);
        }
    }
}