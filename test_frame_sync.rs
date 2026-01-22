use rust_mp3_encoder::{Mp3Encoder, Config};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing MP3 frame synchronization fix...");
    
    // Create encoder with default config
    let config = Config::default();
    let mut encoder = Mp3Encoder::new(config)?;
    
    let samples_per_frame = encoder.samples_per_frame();
    let channels = 2; // Stereo
    
    println!("Samples per frame: {}, Channels: {}", samples_per_frame, channels);
    
    // Create output file
    let mut output_file = File::create("tests/output/frame_sync_test.mp3")?;
    
    // Generate and encode 3 frames with different patterns
    for frame_num in 0..3 {
        println!("Encoding frame {}...", frame_num + 1);
        
        // Generate different audio patterns for each frame
        let pcm_data: Vec<i16> = (0..samples_per_frame * channels)
            .map(|i| {
                let t = i as f64 / 44100.0;
                let freq = 440.0 * (frame_num + 1) as f64; // Different frequency for each frame
                (1000.0 * (2.0 * std::f64::consts::PI * freq * t).sin()) as i16
            })
            .collect();
        
        // Encode frame
        let encoded_frame = encoder.encode_frame_interleaved(&pcm_data)?;
        
        println!("Frame {} size: {} bytes", frame_num + 1, encoded_frame.len());
        
        // Verify frame starts with sync word
        if encoded_frame.len() >= 4 {
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            println!("Frame {} sync word: 0x{:03X}", frame_num + 1, sync);
            
            if sync != 0x7FF {
                eprintln!("ERROR: Frame {} has invalid sync word!", frame_num + 1);
                return Err("Invalid sync word".into());
            }
        }
        
        // Write frame to file
        output_file.write_all(encoded_frame)?;
    }
    
    println!("Successfully encoded 3 frames to tests/output/frame_sync_test.mp3");
    println!("Now validating the generated file...");
    
    Ok(())
}