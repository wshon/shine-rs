fn main() {
    // Analyze our header: FF FB 90 44
    let our_header = 0xFFFB9044u32;
    
    // Analyze expected header: FF FB 92 04  
    let expected_header = 0xFFFB9204u32;
    
    println!("=== Header Analysis ===");
    println!("Our header:      0x{:08X}", our_header);
    println!("Expected header: 0x{:08X}", expected_header);
    println!("Difference:      0x{:08X}", our_header ^ expected_header);
    
    println!("\n=== Bit-by-bit Analysis ===");
    
    // Extract fields from our header
    println!("Our header fields:");
    println!("  Sync:        0x{:03X} ({:011b})", (our_header >> 21) & 0x7FF, (our_header >> 21) & 0x7FF);
    println!("  Version:     {} ({:02b})", (our_header >> 19) & 0x3, (our_header >> 19) & 0x3);
    println!("  Layer:       {} ({:02b})", (our_header >> 17) & 0x3, (our_header >> 17) & 0x3);
    println!("  CRC:         {} ({:01b})", (our_header >> 16) & 0x1, (our_header >> 16) & 0x1);
    println!("  Bitrate:     {} ({:04b})", (our_header >> 12) & 0xF, (our_header >> 12) & 0xF);
    println!("  Sample rate: {} ({:02b})", (our_header >> 10) & 0x3, (our_header >> 10) & 0x3);
    println!("  Padding:     {} ({:01b})", (our_header >> 9) & 0x1, (our_header >> 9) & 0x1);
    println!("  Private:     {} ({:01b})", (our_header >> 8) & 0x1, (our_header >> 8) & 0x1);
    println!("  Mode:        {} ({:02b})", (our_header >> 6) & 0x3, (our_header >> 6) & 0x3);
    println!("  Mode ext:    {} ({:02b})", (our_header >> 4) & 0x3, (our_header >> 4) & 0x3);
    println!("  Copyright:   {} ({:01b})", (our_header >> 3) & 0x1, (our_header >> 3) & 0x1);
    println!("  Original:    {} ({:01b})", (our_header >> 2) & 0x1, (our_header >> 2) & 0x1);
    println!("  Emphasis:    {} ({:02b})", our_header & 0x3, our_header & 0x3);
    
    println!("\nExpected header fields:");
    println!("  Sync:        0x{:03X} ({:011b})", (expected_header >> 21) & 0x7FF, (expected_header >> 21) & 0x7FF);
    println!("  Version:     {} ({:02b})", (expected_header >> 19) & 0x3, (expected_header >> 19) & 0x3);
    println!("  Layer:       {} ({:02b})", (expected_header >> 17) & 0x3, (expected_header >> 17) & 0x3);
    println!("  CRC:         {} ({:01b})", (expected_header >> 16) & 0x1, (expected_header >> 16) & 0x1);
    println!("  Bitrate:     {} ({:04b})", (expected_header >> 12) & 0xF, (expected_header >> 12) & 0xF);
    println!("  Sample rate: {} ({:02b})", (expected_header >> 10) & 0x3, (expected_header >> 10) & 0x3);
    println!("  Padding:     {} ({:01b})", (expected_header >> 9) & 0x1, (expected_header >> 9) & 0x1);
    println!("  Private:     {} ({:01b})", (expected_header >> 8) & 0x1, (expected_header >> 8) & 0x1);
    println!("  Mode:        {} ({:02b})", (expected_header >> 6) & 0x3, (expected_header >> 6) & 0x3);
    println!("  Mode ext:    {} ({:02b})", (expected_header >> 4) & 0x3, (expected_header >> 4) & 0x3);
    println!("  Copyright:   {} ({:01b})", (expected_header >> 3) & 0x1, (expected_header >> 3) & 0x1);
    println!("  Original:    {} ({:01b})", (expected_header >> 2) & 0x1, (expected_header >> 2) & 0x1);
    println!("  Emphasis:    {} ({:02b})", expected_header & 0x3, expected_header & 0x3);
    
    println!("\n=== Differences ===");
    if ((our_header >> 21) & 0x7FF) != ((expected_header >> 21) & 0x7FF) {
        println!("  Sync differs!");
    }
    if ((our_header >> 19) & 0x3) != ((expected_header >> 19) & 0x3) {
        println!("  Version differs!");
    }
    if ((our_header >> 17) & 0x3) != ((expected_header >> 17) & 0x3) {
        println!("  Layer differs!");
    }
    if ((our_header >> 16) & 0x1) != ((expected_header >> 16) & 0x1) {
        println!("  CRC differs!");
    }
    if ((our_header >> 12) & 0xF) != ((expected_header >> 12) & 0xF) {
        println!("  Bitrate differs!");
    }
    if ((our_header >> 10) & 0x3) != ((expected_header >> 10) & 0x3) {
        println!("  Sample rate differs!");
    }
    if ((our_header >> 9) & 0x1) != ((expected_header >> 9) & 0x1) {
        println!("  Padding differs!");
    }
    if ((our_header >> 8) & 0x1) != ((expected_header >> 8) & 0x1) {
        println!("  Private differs!");
    }
    if ((our_header >> 6) & 0x3) != ((expected_header >> 6) & 0x3) {
        println!("  Mode differs!");
    }
    if ((our_header >> 4) & 0x3) != ((expected_header >> 4) & 0x3) {
        println!("  Mode ext differs!");
    }
    if ((our_header >> 3) & 0x1) != ((expected_header >> 3) & 0x1) {
        println!("  Copyright differs!");
    }
    if ((our_header >> 2) & 0x1) != ((expected_header >> 2) & 0x1) {
        println!("  Original differs!");
    }
    if (our_header & 0x3) != (expected_header & 0x3) {
        println!("  Emphasis differs!");
    }
}