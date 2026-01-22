use rust_mp3_encoder::{Config};

fn main() {
    let config = Config::default();
    
    // Calculate frame size parameters (following shine's logic exactly)
    let bitrate = config.mpeg.bitrate * 1000; // Convert to bps
    let sample_rate = config.wave.sample_rate;
    let granules_per_frame = match config.mpeg_version() {
        rust_mp3_encoder::config::MpegVersion::Mpeg1 => 2,
        rust_mp3_encoder::config::MpegVersion::Mpeg2 | rust_mp3_encoder::config::MpegVersion::Mpeg25 => 1,
    };
    let granule_size = 576; // GRANULE_SIZE from shine
    let bits_per_slot = 8;
    
    // Following shine's avg_slots_per_frame calculation exactly:
    let avg_slots_per_frame = ((granules_per_frame * granule_size) as f64 / sample_rate as f64) *
                             (1000.0 * bitrate as f64 / bits_per_slot as f64);
    
    let whole_slots_per_frame = avg_slots_per_frame as usize;
    let frac_slots_per_frame = avg_slots_per_frame - whole_slots_per_frame as f64;
    
    println!("Config: {}kbps, {}Hz, {} channels", config.mpeg.bitrate, config.wave.sample_rate, config.wave.channels as u8);
    println!("Granules per frame: {}", granules_per_frame);
    println!("Granule size: {}", granule_size);
    println!("Avg slots per frame: {:.6}", avg_slots_per_frame);
    println!("Whole slots per frame: {}", whole_slots_per_frame);
    println!("Frac slots per frame: {:.6}", frac_slots_per_frame);
    println!("Target frame size (bytes): {}", whole_slots_per_frame);
    println!("Target frame size (bits): {}", whole_slots_per_frame * 8);
}