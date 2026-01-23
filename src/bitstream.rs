//! Bitstream writing functionality for MP3 encoding
//!
//! This module implements the bitstream writing functions exactly as defined
//! in shine's bitstream.c and l3bitstream.c. It provides functions to write
//! MP3 frame headers, side information, and main data to the output bitstream.

use crate::error::{EncodingError, EncodingResult};
use crate::huffman::{HuffCodeTab, SHINE_HUFFMAN_TABLE};
use crate::quantization::GranuleInfo;
use crate::tables::{SHINE_SLEN1_TAB, SHINE_SLEN2_TAB, SHINE_SCALE_FACT_BAND_INDEX};

/// Bitstream writer structure (matches shine's bitstream_t exactly)
/// (ref/shine/src/lib/bitstream.h:35-42)
#[derive(Debug)]
pub struct BitstreamWriter {
    /// Output data buffer
    pub data: Vec<u8>,
    /// Current data size
    pub data_size: usize,
    /// Current position in data buffer
    pub data_position: usize,
    /// Bit cache for accumulating bits
    pub cache: u32,
    /// Number of bits available in cache
    pub cache_bits: u32,
}

impl BitstreamWriter {
    /// Open the bitstream for writing (matches shine_open_bit_stream)
    /// (ref/shine/src/lib/bitstream.c:15-22)
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            data_size: size,
            data_position: 0,
            cache: 0,
            cache_bits: 32,
        }
    }

    /// Write N bits into the bit stream (matches shine_putbits exactly)
    /// (ref/shine/src/lib/bitstream.c:30-58)
    /// 
    /// # Arguments
    /// * `val` - value to write into the buffer
    /// * `n` - number of bits of val
    pub fn put_bits(&mut self, val: u32, n: u32) -> EncodingResult<()> {
        #[cfg(debug_assertions)]
        {
            if n > 32 {
                return Err(EncodingError::BitstreamError("Cannot write more than 32 bits at a time".to_string()));
            }
            if n < 32 && (val >> n) != 0 {
                return Err(EncodingError::BitstreamError(format!("Upper bits (higher than {}) are not all zeros", n)));
            }
        }

        if self.cache_bits > n {
            self.cache_bits -= n;
            self.cache |= val << self.cache_bits;
        } else {
            // Ensure we have enough space in the buffer
            if self.data_position + 4 >= self.data_size {
                let new_size = self.data_size + (self.data_size / 2);
                self.data.resize(new_size, 0);
                self.data_size = new_size;
            }

            let remaining_n = n - self.cache_bits;
            self.cache |= val >> remaining_n;

            // Write cache to buffer in big-endian format (matches SWAB32 in shine)
            let cache_bytes = self.cache.to_be_bytes();
            self.data[self.data_position..self.data_position + 4].copy_from_slice(&cache_bytes);
            
            self.data_position += 4;
            self.cache_bits = 32 - remaining_n;
            
            if remaining_n != 0 {
                self.cache = val << self.cache_bits;
            } else {
                self.cache = 0;
            }
        }
        
        Ok(())
    }

    /// Get the current bit count (matches shine_get_bits_count exactly)
    /// (ref/shine/src/lib/bitstream.c:60-62)
    pub fn get_bits_count(&self) -> i32 {
        (self.data_position * 8 + (32 - self.cache_bits) as usize) as i32
    }

    /// Get the output data
    pub fn get_data(&self) -> &[u8] {
        &self.data[..self.data_position]
    }

    /// Flush any remaining bits in the cache
    pub fn flush(&mut self) -> EncodingResult<()> {
        if self.cache_bits < 32 {
            // Ensure we have enough space
            if self.data_position + 4 >= self.data_size {
                let new_size = self.data_size + (self.data_size / 2);
                self.data.resize(new_size, 0);
                self.data_size = new_size;
            }

            let cache_bytes = self.cache.to_be_bytes();
            self.data[self.data_position..self.data_position + 4].copy_from_slice(&cache_bytes);
            self.data_position += 4;
            self.cache = 0;
            self.cache_bits = 32;
        }
        Ok(())
    }
}

impl Default for BitstreamWriter {
    fn default() -> Self {
        Self::new(8192) // Default buffer size
    }
}
/// Get absolute value and sign bit (matches shine_abs_and_sign exactly)
/// (ref/shine/src/lib/l3bitstream.c:167-172)
#[inline]
fn abs_and_sign(x: &mut i32) -> u32 {
    if *x > 0 {
        0
    } else {
        *x = -*x;
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..proptest::prelude::ProptestConfig::default()
        })]

        #[test]
        fn test_bitstream_writer_basic_operations(
            val in 0u32..0x10000,
            bits in 1u32..17
        ) {
            let mut bs = BitstreamWriter::new(1024);
            
            // Should be able to write bits without error
            prop_assert!(bs.put_bits(val & ((1 << bits) - 1), bits).is_ok(), "Writing bits should succeed");
            
            // Bit count should increase
            let count = bs.get_bits_count();
            prop_assert!(count >= bits as i32, "Bit count should increase");
        }

        #[test]
        fn test_bitstream_writer_buffer_expansion(
            values in prop::collection::vec(0u32..0x100, 100..200)
        ) {
            let mut bs = BitstreamWriter::new(16); // Small initial size
            
            // Should handle buffer expansion automatically
            for val in values {
                prop_assert!(bs.put_bits(val, 8).is_ok(), "Buffer expansion should work");
            }
            
            prop_assert!(bs.get_bits_count() > 0, "Should have written data");
        }

        #[test]
        fn test_abs_and_sign_function(x in -1000i32..1000) {
            let mut x_copy = x;
            let sign = abs_and_sign(&mut x_copy);
            
            if x >= 0 {
                prop_assert_eq!(sign, 0, "Positive numbers should have sign 0");
                prop_assert_eq!(x_copy, x, "Positive numbers should be unchanged");
            } else {
                prop_assert_eq!(sign, 1, "Negative numbers should have sign 1");
                prop_assert_eq!(x_copy, -x, "Negative numbers should be negated");
            }
        }
    }

    #[test]
    fn test_bitstream_writer_creation() {
        let bs = BitstreamWriter::new(1024);
        assert_eq!(bs.data_size, 1024);
        assert_eq!(bs.data_position, 0);
        assert_eq!(bs.cache, 0);
        assert_eq!(bs.cache_bits, 32);
    }

    #[test]
    fn test_bitstream_writer_simple_write() {
        let mut bs = BitstreamWriter::new(1024);
        
        // Write some bits
        assert!(bs.put_bits(0b1010, 4).is_ok());
        assert_eq!(bs.get_bits_count(), 4);
        
        assert!(bs.put_bits(0b11, 2).is_ok());
        assert_eq!(bs.get_bits_count(), 6);
    }

    #[test]
    fn test_bitstream_writer_flush() {
        let mut bs = BitstreamWriter::new(1024);
        
        bs.put_bits(0xff, 8).unwrap();
        bs.flush().unwrap();
        
        let data = bs.get_data();
        assert!(!data.is_empty());
    }
}