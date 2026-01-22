use rust_mp3_encoder::config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use rust_mp3_encoder::encoder::Mp3Encoder;

fn main() {
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
    
    // Create simple test data - all zeros first
    let samples_per_frame = 1152;
    let channels = 2;
    let total_samples = samples_per_frame * channels;
    let pcm_data = vec![0i16; total_samples];
    
    println!("Encoding frame with all zeros...");
    let result = encoder.encode_frame(&pcm_data);
    match result {
        Ok(frame) => {
            println!("Success! Frame size: {} bytes", frame.len());
            
            // Check sync word
            if frame.len() >= 4 {
                let sync = ((frame[0] as u16) << 3) | ((frame[1] as u16) >> 5);
                println!("Sync word: 0x{:03X} (expected: 0x7FF)", sync);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    
    // Now try with small non-zero values
    let pcm_data = vec![100i16; total_samples];
    
    println!("\nEncoding frame with small values (100)...");
    let result = encoder.encode_frame(&pcm_data);
    match result {
        Ok(frame) => {
            println!("Success! Frame size: {} bytes", frame.len());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}