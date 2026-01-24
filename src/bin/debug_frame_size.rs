//! Debug frame size differences between our implementation and shine

use rust_mp3_encoder::bitstream::BitstreamWriter;

fn main() {
    println!("Debugging frame size differences...\n");
    
    // Simulate the exact sequence that would happen in MP3 frame encoding
    let mut bs = BitstreamWriter::new(1024);
    
    println!("=== Simulating MP3 Frame Header ===");
    
    // Frame header (32 bits total) - exactly as in MP3
    bs.put_bits(0x7ff, 11).unwrap();  // Sync word (11 bits)
    println!("After sync word: cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    bs.put_bits(3, 2).unwrap();       // MPEG version (2 bits)
    bs.put_bits(1, 2).unwrap();       // Layer (2 bits)  
    bs.put_bits(1, 1).unwrap();       // Protection bit (1 bit)
    bs.put_bits(9, 4).unwrap();       // Bitrate index (4 bits)
    bs.put_bits(0, 2).unwrap();       // Sample rate index (2 bits)
    bs.put_bits(0, 1).unwrap();       // Padding (1 bit)
    bs.put_bits(0, 1).unwrap();       // Private bit (1 bit)
    bs.put_bits(1, 2).unwrap();       // Channel mode (2 bits)
    bs.put_bits(0, 2).unwrap();       // Mode extension (2 bits)
    bs.put_bits(0, 1).unwrap();       // Copyright (1 bit)
    bs.put_bits(1, 1).unwrap();       // Original (1 bit)
    bs.put_bits(0, 2).unwrap();       // Emphasis (2 bits)
    
    println!("After frame header (32 bits): cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    println!("\n=== Simulating Side Info ===");
    
    // Side info for MPEG-1 stereo (256 bits total)
    bs.put_bits(0, 9).unwrap();       // main_data_begin (9 bits)
    bs.put_bits(0, 3).unwrap();       // private_bits (3 bits)
    
    // scfsi for both channels (4 bits each = 8 bits total)
    bs.put_bits(0, 4).unwrap();       // scfsi[0]
    bs.put_bits(0, 4).unwrap();       // scfsi[1]
    
    println!("After main_data_begin + private_bits + scfsi: cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    // Granule info for 2 granules, 2 channels (59 bits per granule per channel)
    for gr in 0..2 {
        for ch in 0..2 {
            println!("  Granule {} Channel {}: before granule info: cache_bits={}, data_position={}, bits_count={}", 
                     gr, ch, bs.cache_bits, bs.data_position, bs.get_bits_count());
            
            bs.put_bits(0, 12).unwrap();  // part2_3_length
            bs.put_bits(0, 9).unwrap();   // big_values
            bs.put_bits(0, 8).unwrap();   // global_gain
            bs.put_bits(0, 4).unwrap();   // scalefac_compress
            bs.put_bits(0, 1).unwrap();   // window_switching_flag
            bs.put_bits(0, 5).unwrap();   // block_type
            bs.put_bits(0, 1).unwrap();   // mixed_block_flag
            bs.put_bits(0, 5).unwrap();   // table_select[0]
            bs.put_bits(0, 5).unwrap();   // table_select[1]
            bs.put_bits(0, 4).unwrap();   // subblock_gain[0]
            bs.put_bits(0, 4).unwrap();   // subblock_gain[1]
            bs.put_bits(0, 4).unwrap();   // subblock_gain[2]
            bs.put_bits(0, 1).unwrap();   // preflag
            bs.put_bits(0, 1).unwrap();   // scalefac_scale
            bs.put_bits(0, 1).unwrap();   // count1table_select
            
            println!("    After granule info: cache_bits={}, data_position={}, bits_count={}", 
                     bs.cache_bits, bs.data_position, bs.get_bits_count());
        }
    }
    
    println!("\nAfter complete side info: cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    println!("\n=== Simulating Main Data ===");
    
    // Add some main data (this would be the actual audio data)
    // Let's add enough bits to see what happens
    for i in 0..100 {
        bs.put_bits(0, 8).unwrap();  // Add 8 bits of main data
        if i % 10 == 9 {
            println!("After {} bytes of main data: cache_bits={}, data_position={}, bits_count={}", 
                     i + 1, bs.cache_bits, bs.data_position, bs.get_bits_count());
        }
    }
    
    println!("\n=== Final State ===");
    println!("Final: cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    // Calculate frame size as shine would
    let frame_size_bytes = bs.data_position as usize;
    println!("Frame size (as shine would report): {} bytes", frame_size_bytes);
    
    // Show what would happen if we flushed
    let mut bs_copy = bs;
    bs_copy.flush().unwrap();
    println!("Frame size after flush: {} bytes", bs_copy.data_position);
    
    println!("\nDifference: {} bytes", bs_copy.data_position as usize - frame_size_bytes);
}