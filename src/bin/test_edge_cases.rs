//! Test edge cases in put_bits implementation

use rust_mp3_encoder::bitstream::BitstreamWriter;

fn main() {
    println!("Testing put_bits edge cases...\n");
    
    // Test case: cache_bits exactly equals n
    println!("=== Test: cache_bits == n ===");
    let mut bs = BitstreamWriter::new(1024);
    
    // Fill cache with exactly 24 bits, leaving 8 bits free
    bs.put_bits(0xFFFFFF, 24).unwrap();
    println!("After 24 bits: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    // Now write exactly 8 bits (cache_bits == n)
    bs.put_bits(0xFF, 8).unwrap();
    println!("After 8 more bits: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    println!("Bits count: {}\n", bs.get_bits_count());
    
    // Test case: Write 1 bit when cache_bits == 1
    println!("=== Test: cache_bits == 1, write 1 bit ===");
    let mut bs = BitstreamWriter::new(1024);
    
    // Fill cache with 31 bits, leaving 1 bit free
    bs.put_bits(0x7FFFFFFF, 31).unwrap();
    println!("After 31 bits: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    // Write exactly 1 bit (cache_bits == n)
    bs.put_bits(1, 1).unwrap();
    println!("After 1 more bit: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    println!("Bits count: {}\n", bs.get_bits_count());
    
    // Test case: Write 0 bits
    println!("=== Test: Write 0 bits ===");
    let mut bs = BitstreamWriter::new(1024);
    bs.put_bits(0xFF, 8).unwrap();
    println!("Before 0 bits: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    bs.put_bits(0, 0).unwrap();
    println!("After 0 bits: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    // Test case: Exact 32-bit boundary
    println!("\n=== Test: Exact 32-bit boundary ===");
    let mut bs = BitstreamWriter::new(1024);
    
    bs.put_bits(0xFFFFFFFF, 32).unwrap();
    println!("After 32 bits: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    bs.put_bits(0x1, 1).unwrap();
    println!("After 1 more bit: cache_bits={}, data_position={}, cache=0x{:08X}", 
             bs.cache_bits, bs.data_position, bs.cache);
    
    println!("Final bits count: {}", bs.get_bits_count());
}