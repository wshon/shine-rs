use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

fn main() {
    println!("Testing stack overflow issue...");
    
    // Create a minimal configuration
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::JointStereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    println!("Creating encoder...");
    let mut encoder = Mp3Encoder::new(config).expect("Failed to create encoder");
    
    println!("Creating test data...");
    // Create a small amount of test data (1 frame)
    let samples_per_frame = encoder.samples_per_frame();
    let channels = 2; // stereo
    let frame_size = samples_per_frame.saturating_mul(channels); // Use saturating_mul to prevent overflow
    println!("Frame size calculation:");
    println!("  Samples per frame: {}", samples_per_frame);
    println!("  Channels: {}", channels);
    println!("  Frame size (total): {}", frame_size);
    let test_data: Vec<i16> = (0..frame_size).map(|i| ((i % 1000) as i16).saturating_mul(10)).collect();
    
    println!("Encoding single frame with {} samples...", test_data.len());
    
    // Try to encode just one frame
    match encoder.encode_frame_interleaved(&test_data) {
        Ok(mp3_data) => {
            println!("✅ Successfully encoded {} bytes", mp3_data.len());
        },
        Err(e) => {
            println!("❌ Encoding failed: {}", e);
        }
    }
    
    println!("Test completed");
}