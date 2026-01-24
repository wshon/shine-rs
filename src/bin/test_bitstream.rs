use rust_mp3_encoder::bitstream::BitstreamWriter;

fn main() {
    let mut bs = BitstreamWriter::new(1024);

    // Write some bits that would trigger the cache write
    bs.put_bits(0x7ff, 11).unwrap(); // Sync word
    bs.put_bits(3, 2); // Version (3 = MPEG-1)
    bs.put_bits(1, 2); // Layer (1 = Layer III)
    bs.put_bits(1, 1); // CRC
    bs.put_bits(9, 4); // Bitrate index
    bs.put_bits(0, 2); // Sample rate
    bs.put_bits(1, 1); // Padding
    bs.put_bits(0, 1); // Extension
    bs.put_bits(1, 2); // Mode
    bs.put_bits(0, 2); // Mode extension
    bs.put_bits(0, 1); // Copyright
    bs.put_bits(1, 1); // Original
    bs.put_bits(0, 2); // Emphasis

    // Write 0 for 9 bits (main data begin)
    bs.put_bits(0, 9).unwrap();

    // Flush and get data
    bs.flush().unwrap();
    let data = bs.get_data();

    println!("Data length: {}", data.len());
    println!("Data (hex): {:02X?}", &data[..std::cmp::min(16, data.len())]);

    // Check the first 4 bytes
    if data.len() >= 4 {
        println!(
            "First 4 bytes: {:02X} {:02X} {:02X} {:02X}",
            data[0], data[1], data[2], data[3]
        );
    }

    // Expected: FF FB 92 44 (for 128 kbps, 44100 Hz, joint stereo, padding=1)
    println!("\nExpected first 4 bytes: FF FB 92 44");
}
