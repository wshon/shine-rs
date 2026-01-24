use rust_mp3_encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved, shine_set_config_mpeg_defaults};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple test configuration
    let mut config = ShineConfig {
        wave: ShineWave {
            channels: 2,
            samplerate: 44100,
        },
        mpeg: ShineMpeg {
            mode: 0,
            bitr: 128,
            emph: 0,
            copyright: 0,
            original: 1,
        },
    };
    
    shine_set_config_mpeg_defaults(&mut config.mpeg);
    
    // Initialize encoder
    let mut encoder = shine_initialise(&config)?;
    
    println!("=== Initial Side Info State ===");
    println!("private_bits: {}", encoder.side_info.private_bits);
    println!("resv_drain: {}", encoder.side_info.resv_drain);
    
    for ch in 0..2 {
        println!("Channel {} SCFSI: {:?}", ch, encoder.side_info.scfsi[ch]);
    }
    
    for gr in 0..2 {
        for ch in 0..2 {
            let gi = &encoder.side_info.gr[gr].ch[ch].tt;
            println!("Granule {} Channel {} GrInfo:", gr, ch);
            println!("  part2_3_length: {}", gi.part2_3_length);
            println!("  big_values: {}", gi.big_values);
            println!("  count1: {}", gi.count1);
            println!("  global_gain: {}", gi.global_gain);
            println!("  scalefac_compress: {}", gi.scalefac_compress);
            println!("  table_select: {:?}", gi.table_select);
            println!("  region0_count: {}", gi.region0_count);
            println!("  region1_count: {}", gi.region1_count);
            println!("  preflag: {}", gi.preflag);
            println!("  scalefac_scale: {}", gi.scalefac_scale);
            println!("  count1table_select: {}", gi.count1table_select);
            println!("  part2_length: {}", gi.part2_length);
            println!("  sfb_lmax: {}", gi.sfb_lmax);
            println!("  quantizer_step_size: {}", gi.quantizer_step_size);
            println!("  slen: {:?}", gi.slen);
        }
    }
    
    // Create some test audio data (silence)
    let samples_per_pass = 2304; // 2 * 1152
    let buffer = vec![0i16; samples_per_pass];
    
    // Encode one frame
    println!("\n=== Encoding one frame ===");
    let (data, written) = shine_encode_buffer_interleaved(&mut encoder, buffer.as_ptr())?;
    println!("Encoded {} bytes", written);
    
    // Show first few bytes of encoded data
    println!("\n=== First 16 bytes of encoded data ===");
    for i in 0..std::cmp::min(16, written) {
        print!("{:02X} ", data[i]);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
    if written % 8 != 0 {
        println!();
    }
    
    println!("\n=== Side Info After Encoding ===");
    println!("private_bits: {}", encoder.side_info.private_bits);
    println!("resv_drain: {}", encoder.side_info.resv_drain);
    
    for gr in 0..2 {
        for ch in 0..2 {
            let gi = &encoder.side_info.gr[gr].ch[ch].tt;
            println!("Granule {} Channel {} GrInfo:", gr, ch);
            println!("  part2_3_length: {}", gi.part2_3_length);
            println!("  big_values: {}", gi.big_values);
            println!("  count1: {}", gi.count1);
            println!("  global_gain: {}", gi.global_gain);
            println!("  scalefac_compress: {}", gi.scalefac_compress);
            println!("  table_select: {:?}", gi.table_select);
            println!("  region0_count: {}", gi.region0_count);
            println!("  region1_count: {}", gi.region1_count);
            println!("  preflag: {}", gi.preflag);
            println!("  scalefac_scale: {}", gi.scalefac_scale);
            println!("  count1table_select: {}", gi.count1table_select);
            println!("  part2_length: {}", gi.part2_length);
            println!("  sfb_lmax: {}", gi.sfb_lmax);
            println!("  quantizer_step_size: {}", gi.quantizer_step_size);
            println!("  slen: {:?}", gi.slen);
        }
    }
    
    Ok(())
}