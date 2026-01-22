use rust_mp3_encoder::{Config, Mp3Encoder};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating minimal MP3 test...");
    
    // Create minimal config
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Mono,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Mono,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut encoder = Mp3Encoder::new(config)?;
    
    // Create minimal test data - all zeros (should be easiest to encode)
    let samples_per_frame = encoder.samples_per_frame();
    let pcm_data = vec![0i16; samples_per_frame];
    
    println!("Encoding {} samples of silence...", samples_per_frame);
    
    // Encode one frame
    let encoded_frame = encoder.encode_frame(&pcm_data)?;
    
    println!("Encoded frame size: {} bytes", encoded_frame.len());
    
    // Write to file
    std::fs::write("tests/output/minimal_test.mp3", encoded_frame)?;
    
    println!("✅ Minimal test file created: tests/output/minimal_test.mp3");
    
    Ok(())
}