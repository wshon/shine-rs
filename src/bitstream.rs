//! Bitstream writing functionality for MP3 frames
//!
//! This module provides the BitstreamWriter for writing MP3 frame data,
//! including frame headers, side information, and encoded audio data.

use crate::config::{Config, MpegVersion, StereoMode, Emphasis};

/// Bitstream writer for MP3 frame data
pub struct BitstreamWriter {
    /// Output buffer
    buffer: Vec<u8>,
    /// Bit cache for sub-byte operations
    cache: u32,
    /// Number of bits in cache
    cache_bits: u8,
    /// Current write position in buffer
    position: usize,
}

impl BitstreamWriter {
    /// Create a new bitstream writer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            cache: 0,
            cache_bits: 0,
            position: 0,
        }
    }
    
    /// Write bits to the bitstream
    /// 
    /// This method accumulates bits in a cache and writes complete bytes
    /// to the buffer when the cache is full.
    pub fn write_bits(&mut self, value: u32, bits: u8) {
        if bits == 0 || bits > 32 {
            return; // Invalid bit count
        }
        
        // For large bit counts, split into smaller chunks to avoid overflow
        if bits > 24 || (self.cache_bits as u32 + bits as u32) > 32 {
            // Split into two writes to avoid overflow
            if bits > 16 {
                let high_bits = bits - 16;
                let high_value = value >> 16;
                let low_value = value & 0xFFFF;
                
                self.write_bits(high_value, high_bits);
                self.write_bits(low_value, 16);
            } else {
                // Write in chunks that won't cause overflow
                let available_space = 32 - self.cache_bits;
                if bits > available_space {
                    let first_chunk_bits = available_space;
                    let remaining_bits = bits - first_chunk_bits;
                    
                    let first_chunk = value >> remaining_bits;
                    let second_chunk = value & ((1u32 << remaining_bits) - 1);
                    
                    self.write_bits(first_chunk, first_chunk_bits);
                    self.write_bits(second_chunk, remaining_bits);
                } else {
                    // Safe to write directly
                    self.write_bits_direct(value, bits);
                }
            }
        } else {
            // Safe to write directly
            self.write_bits_direct(value, bits);
        }
    }
    
    /// Internal method to write bits directly without overflow checking
    fn write_bits_direct(&mut self, value: u32, bits: u8) {
        // Mask the value to ensure only the specified number of bits are used
        let masked_value = if bits == 32 {
            value
        } else {
            value & ((1u32 << bits) - 1)
        };
        
        // Add bits to cache
        if self.cache_bits == 0 {
            self.cache = masked_value;
        } else {
            self.cache = (self.cache << bits) | masked_value;
        }
        self.cache_bits += bits;
        
        // Write complete bytes to buffer
        while self.cache_bits >= 8 {
            let byte = (self.cache >> (self.cache_bits - 8)) as u8;
            
            // Ensure buffer has enough capacity
            if self.position >= self.buffer.len() {
                self.buffer.push(byte);
            } else {
                self.buffer[self.position] = byte;
            }
            
            self.position += 1;
            self.cache_bits -= 8;
            
            if self.cache_bits > 0 {
                self.cache &= (1u32 << self.cache_bits) - 1; // Clear written bits
            } else {
                self.cache = 0;
            }
        }
    }
    
    /// Write MP3 frame header
    /// 
    /// Writes the standard MP3 frame header according to the ISO/IEC 11172-3 specification.
    /// The frame header contains sync word, version, layer, bitrate, sample rate, and other flags.
    pub fn write_frame_header(&mut self, config: &Config, padding: bool) {
        // Sync word (11 bits) - always 0x7FF
        self.write_bits(0x7FF, 11);
        
        // MPEG version (2 bits)
        let version_bits = match config.mpeg_version() {
            MpegVersion::Mpeg1 => 3,   // 11
            MpegVersion::Mpeg2 => 2,   // 10  
            MpegVersion::Mpeg25 => 0,  // 00
        };
        self.write_bits(version_bits, 2);
        
        // Layer (2 bits) - always Layer III (01)
        self.write_bits(1, 2);
        
        // Protection bit (1 bit) - 1 = no CRC, 0 = CRC present
        self.write_bits(1, 1); // No CRC for now
        
        // Bitrate index (4 bits)
        let bitrate_index = self.get_bitrate_index(config);
        self.write_bits(bitrate_index, 4);
        
        // Sample rate index (2 bits)
        let samplerate_index = self.get_samplerate_index(config);
        self.write_bits(samplerate_index, 2);
        
        // Padding bit (1 bit)
        self.write_bits(if padding { 1 } else { 0 }, 1);
        
        // Private bit (1 bit) - unused, set to 0
        self.write_bits(0, 1);
        
        // Channel mode (2 bits)
        let mode_bits = match config.mpeg.mode {
            StereoMode::Stereo => 0,      // 00
            StereoMode::JointStereo => 1, // 01
            StereoMode::DualChannel => 2, // 10
            StereoMode::Mono => 3,        // 11
        };
        self.write_bits(mode_bits, 2);
        
        // Mode extension (2 bits) - only used for joint stereo
        self.write_bits(0, 2); // No joint stereo extensions for now
        
        // Copyright bit (1 bit)
        self.write_bits(if config.mpeg.copyright { 1 } else { 0 }, 1);
        
        // Original bit (1 bit)
        self.write_bits(if config.mpeg.original { 1 } else { 0 }, 1);
        
        // Emphasis (2 bits)
        let emphasis_bits = match config.mpeg.emphasis {
            Emphasis::None => 0,         // 00
            Emphasis::Emphasis50_15 => 1, // 01
            Emphasis::CcittJ17 => 3,     // 11
        };
        self.write_bits(emphasis_bits, 2);
    }
    
    /// Get bitrate index for the frame header
    fn get_bitrate_index(&self, config: &Config) -> u32 {
        let bitrates = match config.mpeg_version() {
            MpegVersion::Mpeg1 => &[
                0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0
            ][..],
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => &[
                0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0
            ][..],
        };
        
        for (index, &rate) in bitrates.iter().enumerate() {
            if rate == config.mpeg.bitrate {
                return index as u32;
            }
        }
        
        // Default to index 9 (128 kbps for MPEG-1, 64 kbps for MPEG-2/2.5)
        9
    }
    
    /// Get sample rate index for the frame header
    fn get_samplerate_index(&self, config: &Config) -> u32 {
        match config.wave.sample_rate {
            44100 => 0,
            48000 => 1,
            32000 => 2,
            22050 => 0, // MPEG-2
            24000 => 1, // MPEG-2
            16000 => 2, // MPEG-2
            11025 => 0, // MPEG-2.5
            12000 => 1, // MPEG-2.5
            8000 => 2,  // MPEG-2.5
            _ => 0,     // Default
        }
    }
    
    /// Write side information
    /// 
    /// Writes the side information section that contains encoding parameters
    /// for each granule and channel.
    pub fn write_side_info(&mut self, side_info: &SideInfo, config: &Config) {
        let channels = config.wave.channels as usize;
        let granules_per_frame = match config.mpeg_version() {
            MpegVersion::Mpeg1 => 2,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 1,
        };
        
        // Main data begin pointer and private bits
        if config.mpeg_version() == MpegVersion::Mpeg1 {
            // MPEG-1: 9 bits for main data begin
            self.write_bits(0, 9); // No bit reservoir for now
            
            // Private bits
            if channels == 2 {
                self.write_bits(side_info.private_bits, 3);
            } else {
                self.write_bits(side_info.private_bits, 5);
            }
        } else {
            // MPEG-2/2.5: 8 bits for main data begin
            self.write_bits(0, 8); // No bit reservoir for now
            
            // Private bits
            if channels == 2 {
                self.write_bits(side_info.private_bits, 2);
            } else {
                self.write_bits(side_info.private_bits, 1);
            }
        }
        
        // SCFSI (Scale Factor Selection Information) - only for MPEG-1
        if config.mpeg_version() == MpegVersion::Mpeg1 {
            for ch in 0..channels {
                for band in 0..4 {
                    self.write_bits(if side_info.scfsi[ch][band] { 1 } else { 0 }, 1);
                }
            }
        }
        
        // Granule information
        for gr in 0..granules_per_frame {
            for ch in 0..channels {
                let granule_index = gr * channels + ch;
                if granule_index < side_info.granules.len() {
                    let gi = &side_info.granules[granule_index];
                    
                    // Part 2+3 length (12 bits)
                    self.write_bits(gi.part2_3_length, 12);
                    
                    // Big values (9 bits)
                    self.write_bits(gi.big_values, 9);
                    
                    // Global gain (8 bits)
                    self.write_bits(gi.global_gain, 8);
                    
                    // Scale factor compress (4 bits for MPEG-1, 9 bits for MPEG-2/2.5)
                    if config.mpeg_version() == MpegVersion::Mpeg1 {
                        self.write_bits(gi.scalefac_compress, 4);
                    } else {
                        self.write_bits(gi.scalefac_compress, 9);
                    }
                    
                    // Window switching flag (1 bit) - always 0 for long blocks
                    self.write_bits(0, 1);
                    
                    // Table select (3 * 5 bits)
                    for region in 0..3 {
                        self.write_bits(gi.table_select[region], 5);
                    }
                    
                    // Region count (4 + 3 bits)
                    self.write_bits(gi.region0_count, 4);
                    self.write_bits(gi.region1_count, 3);
                    
                    // Pre-flag (1 bit) - only for MPEG-1
                    if config.mpeg_version() == MpegVersion::Mpeg1 {
                        self.write_bits(if gi.preflag { 1 } else { 0 }, 1);
                    }
                    
                    // Scale factor scale (1 bit)
                    self.write_bits(if gi.scalefac_scale { 1 } else { 0 }, 1);
                    
                    // Count1 table select (1 bit)
                    self.write_bits(if gi.count1table_select { 1 } else { 0 }, 1);
                }
            }
        }
    }
    
    /// Calculate CRC-16 for the frame header and side info
    /// 
    /// This implements the CRC-16 polynomial used in MP3: x^16 + x^15 + x^2 + 1
    pub fn calculate_crc(&self, data: &[u8], start_byte: usize, length_bits: usize) -> u16 {
        let mut crc: u16 = 0xFFFF;
        let polynomial: u16 = 0x8005; // CRC-16 polynomial
        
        let end_byte = start_byte + (length_bits + 7) / 8;
        let end_byte = std::cmp::min(end_byte, data.len());
        
        for byte_idx in start_byte..end_byte {
            let mut byte = data[byte_idx];
            
            // Handle partial last byte
            if byte_idx == end_byte - 1 {
                let remaining_bits = length_bits % 8;
                if remaining_bits != 0 {
                    byte &= 0xFF << (8 - remaining_bits);
                }
            }
            
            crc ^= (byte as u16) << 8;
            
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ polynomial;
                } else {
                    crc <<= 1;
                }
            }
        }
        
        crc
    }
    
    /// Flush remaining bits and return the buffer
    /// 
    /// This method writes any remaining bits in the cache to the buffer,
    /// padding with zeros if necessary to complete the last byte.
    pub fn flush(&mut self) -> &[u8] {
        // Write any remaining bits in cache
        if self.cache_bits > 0 {
            // Pad with zeros to complete the byte
            let padding_bits = 8 - self.cache_bits;
            self.cache <<= padding_bits;
            
            let byte = (self.cache >> (self.cache_bits + padding_bits - 8)) as u8;
            
            // Ensure buffer has enough capacity
            if self.position >= self.buffer.len() {
                self.buffer.push(byte);
            } else {
                self.buffer[self.position] = byte;
            }
            
            self.position += 1;
            self.cache = 0;
            self.cache_bits = 0;
        }
        
        // Ensure buffer is exactly the right size
        self.buffer.truncate(self.position);
        &self.buffer
    }
    
    /// Byte-align the bitstream by padding with zeros if necessary
    pub fn byte_align(&mut self) {
        if self.cache_bits > 0 {
            let padding_bits = 8 - self.cache_bits;
            self.write_bits(0, padding_bits);
        }
    }
    
    /// Reset the writer for reuse
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cache = 0;
        self.cache_bits = 0;
        self.position = 0;
    }
    
    /// Get the number of bits written
    pub fn bits_written(&self) -> usize {
        self.position * 8 + self.cache_bits as usize
    }
    
    /// Get the current buffer contents without flushing
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[..self.position]
    }
    
    /// Get the number of bytes written (complete bytes only)
    pub fn bytes_written(&self) -> usize {
        self.position
    }
}

/// Side information structure for MP3 frames
/// 
/// Contains encoding parameters for each granule and channel,
/// including quantization settings, Huffman table selections,
/// and scale factor information.
#[derive(Debug, Clone)]
pub struct SideInfo {
    /// Private bits for encoder use
    pub private_bits: u32,
    /// Scale factor selection information [channel][band]
    pub scfsi: [[bool; 4]; 2],
    /// Granule information for each granule*channel
    pub granules: Vec<GranuleInfo>,
}

/// Information for a single granule
#[derive(Debug, Clone)]
pub struct GranuleInfo {
    /// Length of part 2 + part 3 in bits
    pub part2_3_length: u32,
    /// Number of big value pairs
    pub big_values: u32,
    /// Global gain for quantization
    pub global_gain: u32,
    /// Scale factor compression index
    pub scalefac_compress: u32,
    /// Huffman table selection for 3 regions
    pub table_select: [u32; 3],
    /// Region 0 count
    pub region0_count: u32,
    /// Region 1 count
    pub region1_count: u32,
    /// Pre-emphasis flag (MPEG-1 only)
    pub preflag: bool,
    /// Scale factor scale flag
    pub scalefac_scale: bool,
    /// Count1 table selection
    pub count1table_select: bool,
}

impl Default for SideInfo {
    fn default() -> Self {
        Self {
            private_bits: 0,
            scfsi: [[false; 4]; 2],
            granules: Vec::new(),
        }
    }
}

impl Default for GranuleInfo {
    fn default() -> Self {
        Self {
            part2_3_length: 0,
            big_values: 0,
            global_gain: 0,
            scalefac_compress: 0,
            table_select: [0; 3],
            region0_count: 0,
            region1_count: 0,
            preflag: false,
            scalefac_scale: false,
            count1table_select: false,
        }
    }
}

impl Default for BitstreamWriter {
    fn default() -> Self {
        Self::new(1024) // Default capacity for one MP3 frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bitstream_writer() {
        let writer = BitstreamWriter::new(512);
        assert_eq!(writer.bits_written(), 0);
        assert_eq!(writer.bytes_written(), 0);
        assert_eq!(writer.buffer().len(), 0);
    }

    #[test]
    fn test_write_single_byte() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 8 bits (one complete byte)
        writer.write_bits(0b10101010, 8);
        
        assert_eq!(writer.bits_written(), 8);
        assert_eq!(writer.bytes_written(), 1);
        assert_eq!(writer.buffer(), &[0b10101010]);
    }

    #[test]
    fn test_write_partial_bits() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 4 bits
        writer.write_bits(0b1010, 4);
        
        assert_eq!(writer.bits_written(), 4);
        assert_eq!(writer.bytes_written(), 0); // No complete bytes yet
        assert_eq!(writer.buffer().len(), 0);
        
        // Write 4 more bits to complete a byte
        writer.write_bits(0b0101, 4);
        
        assert_eq!(writer.bits_written(), 8);
        assert_eq!(writer.bytes_written(), 1);
        assert_eq!(writer.buffer(), &[0b10100101]);
    }

    #[test]
    fn test_write_multiple_bytes() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 16 bits (two bytes)
        writer.write_bits(0b1010101011110000, 16);
        
        assert_eq!(writer.bits_written(), 16);
        assert_eq!(writer.bytes_written(), 2);
        assert_eq!(writer.buffer(), &[0b10101010, 0b11110000]);
    }

    #[test]
    fn test_write_bits_across_byte_boundary() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 3 bits
        writer.write_bits(0b101, 3);
        assert_eq!(writer.bits_written(), 3);
        assert_eq!(writer.bytes_written(), 0);
        
        // Write 6 bits (total 9 bits, should produce 1 complete byte)
        writer.write_bits(0b110011, 6);
        assert_eq!(writer.bits_written(), 9);
        assert_eq!(writer.bytes_written(), 1);
        assert_eq!(writer.buffer(), &[0b10111001]);
        
        // Write 7 more bits (total 16 bits, should produce 2 complete bytes)
        writer.write_bits(0b1010101, 7);
        assert_eq!(writer.bits_written(), 16);
        assert_eq!(writer.bytes_written(), 2);
        assert_eq!(writer.buffer(), &[0b10111001, 0b11010101]);
    }

    #[test]
    fn test_flush_with_partial_bits() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 5 bits
        writer.write_bits(0b10101, 5);
        assert_eq!(writer.bits_written(), 5);
        assert_eq!(writer.bytes_written(), 0);
        
        // Flush should pad with zeros and complete the byte
        let result = writer.flush();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0b10101000); // Padded with 3 zeros
        assert_eq!(writer.bits_written(), 8);
        assert_eq!(writer.bytes_written(), 1);
    }

    #[test]
    fn test_flush_with_complete_bytes() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write exactly 8 bits
        writer.write_bits(0b11110000, 8);
        
        let result = writer.flush();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0b11110000);
    }

    #[test]
    fn test_flush_empty_writer() {
        let mut writer = BitstreamWriter::new(10);
        
        let result = writer.flush();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_byte_align() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 5 bits
        writer.write_bits(0b10101, 5);
        assert_eq!(writer.bits_written(), 5);
        
        // Byte align should pad to 8 bits
        writer.byte_align();
        assert_eq!(writer.bits_written(), 8);
        assert_eq!(writer.bytes_written(), 1);
        assert_eq!(writer.buffer(), &[0b10101000]);
    }

    #[test]
    fn test_byte_align_already_aligned() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write exactly 8 bits
        writer.write_bits(0b11110000, 8);
        assert_eq!(writer.bits_written(), 8);
        
        // Byte align should do nothing
        writer.byte_align();
        assert_eq!(writer.bits_written(), 8);
        assert_eq!(writer.bytes_written(), 1);
        assert_eq!(writer.buffer(), &[0b11110000]);
    }

    #[test]
    fn test_reset() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write some data
        writer.write_bits(0b10101010, 8);
        writer.write_bits(0b1111, 4);
        
        assert_eq!(writer.bits_written(), 12);
        assert_eq!(writer.bytes_written(), 1);
        
        // Reset should clear everything
        writer.reset();
        assert_eq!(writer.bits_written(), 0);
        assert_eq!(writer.bytes_written(), 0);
        assert_eq!(writer.buffer().len(), 0);
    }

    #[test]
    fn test_write_zero_bits() {
        let mut writer = BitstreamWriter::new(10);
        
        // Writing 0 bits should do nothing
        writer.write_bits(0b11111111, 0);
        assert_eq!(writer.bits_written(), 0);
        assert_eq!(writer.bytes_written(), 0);
    }

    #[test]
    fn test_write_invalid_bit_count() {
        let mut writer = BitstreamWriter::new(10);
        
        // Writing more than 32 bits should do nothing
        writer.write_bits(0b11111111, 33);
        assert_eq!(writer.bits_written(), 0);
        assert_eq!(writer.bytes_written(), 0);
    }

    #[test]
    fn test_value_masking() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write only the lower 4 bits of a larger value
        writer.write_bits(0b11111010, 4); // Should only write 0b1010
        writer.write_bits(0b11110101, 4); // Should only write 0b0101
        
        assert_eq!(writer.bits_written(), 8);
        assert_eq!(writer.bytes_written(), 1);
        assert_eq!(writer.buffer(), &[0b10100101]);
    }

    #[test]
    fn test_large_write() {
        let mut writer = BitstreamWriter::new(10);
        
        // Write 32 bits at once
        writer.write_bits(0xDEADBEEF, 32);
        
        assert_eq!(writer.bits_written(), 32);
        assert_eq!(writer.bytes_written(), 4);
        assert_eq!(writer.buffer(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_buffer_growth() {
        let mut writer = BitstreamWriter::new(1); // Small initial capacity
        
        // Write more data than initial capacity
        for i in 0..10 {
            writer.write_bits(i as u32, 8);
        }
        
        assert_eq!(writer.bits_written(), 80);
        assert_eq!(writer.bytes_written(), 10);
        assert_eq!(writer.buffer().len(), 10);
        
        // Verify the data is correct
        for (i, &byte) in writer.buffer().iter().enumerate() {
            assert_eq!(byte, i as u8);
        }
    }
}