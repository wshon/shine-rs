fn main() {
    let value = 3056u32;
    println!("Value: {}", value);
    println!("Binary: {:b}", value);
    println!("Bits needed: {}", 32 - value.leading_zeros());
    println!("Max 12-bit value: {}", (1u32 << 12) - 1);
    println!("Value fits in 12 bits: {}", value <= (1u32 << 12) - 1);
    
    // Check if the issue is with the bit shifting
    println!("Value >> 12: {}", value >> 12);
    println!("Value & ((1<<12)-1): {}", value & ((1u32 << 12) - 1));
}