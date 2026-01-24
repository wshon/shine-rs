fn main() {
    // Analyze the side info bytes after the frame header
    // Our output:      00 00 00 00 (bytes 4-7)
    // Expected output: 00 00 03 93 (bytes 4-7)
    
    let our_sideinfo = [0x00u8, 0x00, 0x00, 0x00];
    let expected_sideinfo = [0x00u8, 0x00, 0x03, 0x93];
    
    println!("=== Side Information Analysis ===");
    println!("Our side info:      {:02X} {:02X} {:02X} {:02X}", 
             our_sideinfo[0], our_sideinfo[1], our_sideinfo[2], our_sideinfo[3]);
    println!("Expected side info: {:02X} {:02X} {:02X} {:02X}", 
             expected_sideinfo[0], expected_sideinfo[1], expected_sideinfo[2], expected_sideinfo[3]);
    
    // Convert to bit stream for analysis
    let our_bits = ((our_sideinfo[0] as u32) << 24) | 
                   ((our_sideinfo[1] as u32) << 16) | 
                   ((our_sideinfo[2] as u32) << 8) | 
                   (our_sideinfo[3] as u32);
                   
    let expected_bits = ((expected_sideinfo[0] as u32) << 24) | 
                        ((expected_sideinfo[1] as u32) << 16) | 
                        ((expected_sideinfo[2] as u32) << 8) | 
                        (expected_sideinfo[3] as u32);
    
    println!("\nOur bits:      0x{:08X} = {:032b}", our_bits, our_bits);
    println!("Expected bits: 0x{:08X} = {:032b}", expected_bits, expected_bits);
    
    println!("\n=== MPEG-I Side Info Structure ===");
    println!("Bit positions for MPEG-I, stereo:");
    println!("  Main data begin: bits 0-8 (9 bits)");
    println!("  Private bits: bits 9-11 (3 bits)");
    println!("  SCFSI ch0: bits 12-15 (4 bits)");
    println!("  SCFSI ch1: bits 16-19 (4 bits)");
    println!("  Then granule info starts at bit 20...");
    
    // Extract fields from expected
    println!("\nExpected fields:");
    println!("  Main data begin: {} (bits 0-8)", (expected_bits >> 23) & 0x1FF);
    println!("  Private bits: {} (bits 9-11)", (expected_bits >> 20) & 0x7);
    println!("  SCFSI ch0: {} (bits 12-15)", (expected_bits >> 16) & 0xF);
    println!("  SCFSI ch1: {} (bits 16-19)", (expected_bits >> 12) & 0xF);
    
    // The remaining bits are granule info
    let remaining_bits = expected_bits & 0xFFF; // Last 12 bits
    println!("  Remaining granule info: 0x{:03X} = {:012b}", remaining_bits, remaining_bits);
    
    // Analyze the 0x393 pattern
    println!("\n=== Analysis of 0x393 ===");
    println!("0x393 = {} = {:012b}", 0x393, 0x393);
    println!("This suggests some granule fields are non-zero in the expected output");
}