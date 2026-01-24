//! Test put_bits behavior to debug the 1-byte difference

use rust_mp3_encoder::bitstream::BitstreamWriter;

fn main() {
    println!("Testing put_bits behavior...\n");
    
    // Test 1: Write bits one at a time
    println!("Test 1: Write 8 bits (0xFF) one at a time");
    let mut bs = BitstreamWriter::new(1024);
    for i in 0..8 {
        bs.put_bits(1, 1).unwrap();
        println!("  After bit {}: cache_bits={}, data_position={}, cache=0x{:08X}", 
                 i+1, bs.cache_bits, bs.data_position, bs.cache);
    }
    println!("  Final: data_position={}, bits_count={}\n", bs.data_position, bs.get_bits_count());
    
    // Test 2: Write 32 bits at once
    println!("Test 2: Write 32 bits (0xFFFFFFFF) at once");
    let mut bs = BitstreamWriter::new(1024);
    bs.put_bits(0xFFFFFFFF, 32).unwrap();
    println!("  After: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    println!("  Final: data_position={}, bits_count={}\n", bs.data_position, bs.get_bits_count());
    
    // Test 3: Write 11 bits (sync word)
    println!("Test 3: Write 11 bits (0x7FF - sync word)");
    let mut bs = BitstreamWriter::new(1024);
    bs.put_bits(0x7FF, 11).unwrap();
    println!("  After: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    println!("  Final: data_position={}, bits_count={}\n", bs.data_position, bs.get_bits_count());
    
    // Test 4: Write multiple values to fill a 32-bit word
    println!("Test 4: Write values to fill exactly 32 bits");
    let mut bs = BitstreamWriter::new(1024);
    bs.put_bits(0x7FF, 11).unwrap();  // 11 bits
    println!("  After 11 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(3, 2).unwrap();       // 2 bits (total 13)
    println!("  After 13 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(1, 1).unwrap();       // 1 bit (total 14)
    println!("  After 14 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0xF, 4).unwrap();     // 4 bits (total 18)
    println!("  After 18 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 20)
    println!("  After 20 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 22)
    println!("  After 22 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 24)
    println!("  After 24 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 26)
    println!("  After 26 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 28)
    println!("  After 28 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 30)
    println!("  After 30 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    bs.put_bits(0, 2).unwrap();       // 2 bits (total 32)
    println!("  After 32 bits: cache_bits={}, data_position={}", bs.cache_bits, bs.data_position);
    
    println!("  Final: data_position={}, bits_count={}\n", bs.data_position, bs.get_bits_count());
    
    // Test 5: Write 33 bits (should trigger second branch)
    println!("Test 5: Write 33 bits (should trigger cache flush)");
    let mut bs = BitstreamWriter::new(1024);
    bs.put_bits(0xFFFFFFFF, 32).unwrap();
    bs.put_bits(1, 1).unwrap();
    println!("  After: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    println!("  Final: data_position={}, bits_count={}\n", bs.data_position, bs.get_bits_count());
    
    // Test 6: Simulate frame header writing
    println!("Test 6: Simulate MP3 frame header writing");
    let mut bs = BitstreamWriter::new(1024);
    
    // Frame header (32 bits total)
    bs.put_bits(0x7ff, 11).unwrap();  // Sync word
    bs.put_bits(3, 2).unwrap();       // MPEG version
    bs.put_bits(1, 2).unwrap();       // Layer
    bs.put_bits(1, 1).unwrap();       // Protection bit
    bs.put_bits(9, 4).unwrap();       // Bitrate index
    bs.put_bits(0, 2).unwrap();       // Sample rate index
    bs.put_bits(0, 1).unwrap();       // Padding
    bs.put_bits(0, 1).unwrap();       // Private bit
    bs.put_bits(1, 2).unwrap();       // Channel mode
    bs.put_bits(0, 2).unwrap();       // Mode extension
    bs.put_bits(0, 1).unwrap();       // Copyright
    bs.put_bits(1, 1).unwrap();       // Original
    bs.put_bits(0, 2).unwrap();       // Emphasis
    
    println!("  After frame header: cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    // Side info start (9 bits for main_data_begin in MPEG-I)
    bs.put_bits(0, 9).unwrap();
    println!("  After main_data_begin: cache_bits={}, data_position={}, bits_count={}", 
             bs.cache_bits, bs.data_position, bs.get_bits_count());
    
    println!("\nAll tests completed!");
}
