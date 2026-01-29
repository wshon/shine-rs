//! Bitstream writing functionality for MP3 encoding
//!
//! This module implements the bitstream writing functions exactly as defined
//! in shine's bitstream.c and l3bitstream.c. It provides functions to write
//! MP3 frame headers, side information, and main data to the output bitstream.

use crate::error::{EncodingError, EncodingResult};
use crate::huffman::{HuffCodeTab, SHINE_HUFFMAN_TABLE};
use crate::tables::{SHINE_SCALE_FACT_BAND_INDEX, SHINE_SLEN1_TAB, SHINE_SLEN2_TAB};
use crate::types::{GrInfo, ShineGlobalConfig, GRANULE_SIZE};

/// Bitstream writer structure (matches shine's bitstream_t exactly)
/// (ref/shine/src/lib/bitstream.h:4-10)
#[derive(Debug)]
pub struct BitstreamWriter {
    /// Processed data
    pub data: Box<[u8]>,
    /// Total data size
    pub data_size: i32,
    /// Data position
    pub data_position: i32,
    /// Bit stream cache
    pub cache: u32,
    /// Free bits in cache
    pub cache_bits: i32,
}

impl BitstreamWriter {
    /// Open the bitstream for writing (matches shine_open_bit_stream)
    /// (ref/shine/src/lib/bitstream.c:15-22)
    pub fn new(size: i32) -> Self {
        Self {
            data: vec![0u8; size as usize].into_boxed_slice(),
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
    pub fn put_bits(&mut self, val: u32, n: i32) -> EncodingResult<()> {
        #[cfg(debug_assertions)]
        {
            if n > 32 {
                return Err(EncodingError::BitstreamError(
                    "Cannot write more than 32 bits at a time".to_string(),
                ));
            }
            if n < 0 {
                return Err(EncodingError::BitstreamError(
                    "Cannot write negative number of bits".to_string(),
                ));
            }
            if n < 32 && (val >> n) != 0 {
                return Err(EncodingError::BitstreamError(format!(
                    "Upper bits are not all zeros: val=0x{:X}, n={}, val>>n=0x{:X}",
                    val,
                    n,
                    val >> n
                )));
            }
        }

        // Handle the special case where n=0 (no bits to write)
        if n == 0 {
            return Ok(());
        }

        if self.cache_bits > n {
            // Cache has enough space for the new bits
            self.cache_bits -= n;

            // Add safety check to prevent overflow
            if self.cache_bits >= 0 && self.cache_bits < 32 {
                let shifted_val = val << self.cache_bits;
                self.cache |= shifted_val;
            } else {
                return Err(EncodingError::BitstreamError(format!(
                    "Invalid cache_bits: {}",
                    self.cache_bits
                )));
            }
        } else {
            // Cache doesn't have enough space, need to flush and write to buffer
            // Ensure we have enough space in the buffer
            if self.data_position + 4 >= self.data_size {
                let new_size = self.data_size + (self.data_size / 2);
                let mut new_buffer = vec![0u8; new_size as usize];
                new_buffer[..self.data_position as usize]
                    .copy_from_slice(&self.data[..self.data_position as usize]);
                self.data = new_buffer.into_boxed_slice();
                self.data_size = new_size;
            }

            // Match shine's logic exactly
            let remaining_n = n - self.cache_bits;
            self.cache |= val >> remaining_n;

            // Write cache to buffer using SWAB32 equivalent (byte swap on little-endian)
            let cache_bytes = self.cache.to_be_bytes();
            self.data[self.data_position as usize..self.data_position as usize + 4]
                .copy_from_slice(&cache_bytes);

            self.data_position += 4;
            self.cache_bits = 32 - remaining_n;

            // Match Shine's exact logic for setting new cache value
            // Prevent overflow when remaining_n is 0 or cache_bits is 0
            if remaining_n != 0 && self.cache_bits > 0 && self.cache_bits < 32 {
                let new_cache = val << self.cache_bits;
                self.cache = new_cache;
            } else {
                self.cache = 0;
            }
        }

        Ok(())
    }

    /// Get the current bit count (matches shine_get_bits_count exactly)
    /// (ref/shine/src/lib/bitstream.c:60-62)
    pub fn get_bits_count(&self) -> i32 {
        self.data_position * 8 + (32 - self.cache_bits)
    }

    /// Get the output data
    pub fn get_data(&self) -> &[u8] {
        &self.data[..self.data_position as usize]
    }

    /// Flush any remaining bits in the cache
    /// This matches shine's behavior when there are remaining bits in cache
    pub fn flush(&mut self) -> EncodingResult<()> {
        // Only flush if there are bits in the cache (cache_bits < 32)
        if self.cache_bits < 32 {
            // Calculate how many bytes we need to write
            let bits_in_cache = 32 - self.cache_bits;
            let bytes_to_write = (bits_in_cache + 7) / 8; // Round up to nearest byte

            // Ensure we have enough space
            if self.data_position + bytes_to_write >= self.data_size {
                let new_size = self.data_size + (self.data_size / 2);
                let mut new_buffer = vec![0u8; new_size as usize];
                new_buffer[..self.data_position as usize]
                    .copy_from_slice(&self.data[..self.data_position as usize]);
                self.data = new_buffer.into_boxed_slice();
                self.data_size = new_size;
            }

            // Write the cache bytes in big-endian format (matches shine's SWAB32)
            let cache_bytes = self.cache.to_be_bytes();
            self.data[self.data_position as usize
                ..self.data_position as usize + bytes_to_write as usize]
                .copy_from_slice(&cache_bytes[..bytes_to_write as usize]);
            self.data_position += bytes_to_write;

            // Clear the cache
            self.cache = 0;
            self.cache_bits = 32;
        }
        Ok(())
    }

    /// Align to byte boundary by flushing partial bytes
    /// This matches shine's byte alignment behavior
    pub fn byte_align(&mut self) -> EncodingResult<()> {
        let bits_in_cache = 32 - self.cache_bits;
        if bits_in_cache > 0 {
            let bytes_to_flush = (bits_in_cache + 7) / 8;
            let bits_to_flush = bytes_to_flush * 8;

            if bits_to_flush > bits_in_cache {
                // Need to add padding bits to reach byte boundary
                let padding_bits = bits_to_flush - bits_in_cache;
                self.put_bits(0, padding_bits)?;
            }

            // Now flush the cache to align to byte boundary
            if self.cache_bits < 32 {
                // Ensure we have enough space
                if self.data_position + 4 >= self.data_size {
                    let new_size = self.data_size + (self.data_size / 2);
                    let mut new_buffer = vec![0u8; new_size as usize];
                    new_buffer[..self.data_position as usize]
                        .copy_from_slice(&self.data[..self.data_position as usize]);
                    self.data = new_buffer.into_boxed_slice();
                    self.data_size = new_size;
                }

                let cache_bytes = self.cache.to_be_bytes();
                self.data[self.data_position as usize..self.data_position as usize + 4]
                    .copy_from_slice(&cache_bytes);
                self.data_position += 4;
                self.cache = 0;
                self.cache_bits = 32;
            }
        }
        Ok(())
    }
}

impl Default for BitstreamWriter {
    fn default() -> Self {
        Self::new(8192) // Default buffer size
    }
}

/// Format the bitstream for a complete frame (matches shine_format_bitstream exactly)
/// (ref/shine/src/lib/l3bitstream.c:25-44)
///
/// This is called after a frame of audio has been quantized and coded.
/// It will write the encoded audio to the bitstream.
pub fn format_bitstream(config: &mut ShineGlobalConfig) -> EncodingResult<()> {
    // Apply sign correction to quantized values (matches shine exactly)
    (0..config.wave.channels as usize).for_each(|ch| {
        (0..config.mpeg.granules_per_frame as usize).for_each(|gr| {
            let pi = &mut config.l3_enc[ch][gr];
            let pr = &config.mdct_freq[ch][gr];

            pi.iter_mut()
                .zip(pr.iter())
                .take(GRANULE_SIZE)
                .for_each(|(pi_val, &pr_val)| {
                    if pr_val < 0 && *pi_val > 0 {
                        *pi_val *= -1;
                    }
                });
        });
    });

    encode_side_info(config)?;
    encode_main_data(config)?;

    Ok(())
}

/// Encode the main data section (matches encodeMainData exactly)
/// (ref/shine/src/lib/l3bitstream.c:46-71)
fn encode_main_data(config: &mut ShineGlobalConfig) -> EncodingResult<()> {
    for gr in 0..config.mpeg.granules_per_frame as usize {
        for ch in 0..config.wave.channels as usize {
            // Extract values we need before borrowing config mutably
            let scalefac_compress = config.side_info.gr[gr].ch[ch].tt.scalefac_compress;
            let scfsi = config.side_info.scfsi[ch];
            let slen1 = SHINE_SLEN1_TAB[scalefac_compress as usize];
            let slen2 = SHINE_SLEN2_TAB[scalefac_compress as usize];

            // Write scale factors
            if gr == 0 || scfsi[0] == 0 {
                (0..6).try_for_each(|sfb| {
                    let sf_val = config.scalefactor.l[gr][ch][sfb];
                    config.bs.put_bits(sf_val as u32, slen1)
                })?;
            }
            if gr == 0 || scfsi[1] == 0 {
                (6..11).try_for_each(|sfb| {
                    let sf_val = config.scalefactor.l[gr][ch][sfb];
                    config.bs.put_bits(sf_val as u32, slen1)
                })?;
            }
            if gr == 0 || scfsi[2] == 0 {
                (11..16).try_for_each(|sfb| {
                    let sf_val = config.scalefactor.l[gr][ch][sfb];
                    config.bs.put_bits(sf_val as u32, slen2)
                })?;
            }
            if gr == 0 || scfsi[3] == 0 {
                (16..21).try_for_each(|sfb| {
                    let sf_val = config.scalefactor.l[gr][ch][sfb];
                    config.bs.put_bits(sf_val as u32, slen2)
                })?;
            }

            // Copy the granule info to avoid borrowing conflicts
            let gi = config.side_info.gr[gr].ch[ch].tt.clone();
            let ix = config.l3_enc[ch][gr];
            huffman_code_bits(config, &ix, &gi)?;
        }
    }

    Ok(())
}

/// Encode the side information (matches encodeSideInfo exactly)
/// (ref/shine/src/lib/l3bitstream.c:73-120)
fn encode_side_info(config: &mut ShineGlobalConfig) -> EncodingResult<()> {
    let si = &config.side_info;

    // Write frame header
    config.bs.put_bits(0x7ff, 11)?; // Sync word
    config.bs.put_bits(config.mpeg.version as u32, 2)?;
    config.bs.put_bits(config.mpeg.layer as u32, 2)?;
    config
        .bs
        .put_bits(if config.mpeg.crc == 0 { 1 } else { 0 }, 1)?;
    config.bs.put_bits(config.mpeg.bitrate_index as u32, 4)?;
    config
        .bs
        .put_bits((config.mpeg.samplerate_index % 3) as u32, 2)?;
    config.bs.put_bits(config.mpeg.padding as u32, 1)?;
    config.bs.put_bits(config.mpeg.ext as u32, 1)?;
    config.bs.put_bits(config.mpeg.mode as u32, 2)?;
    config.bs.put_bits(config.mpeg.mode_ext as u32, 2)?;
    config.bs.put_bits(config.mpeg.copyright as u32, 1)?;
    config.bs.put_bits(config.mpeg.original as u32, 1)?;
    config.bs.put_bits(config.mpeg.emph as u32, 2)?;

    // Write side information
    if config.mpeg.version == 3 {
        // MPEG_I = 3
        config.bs.put_bits(0, 9)?; // Main data begin
        if config.wave.channels == 2 {
            config.bs.put_bits(si.private_bits, 3)?;
        } else {
            config.bs.put_bits(si.private_bits, 5)?;
        }
    } else {
        config.bs.put_bits(0, 8)?; // Main data begin
        if config.wave.channels == 2 {
            config.bs.put_bits(si.private_bits, 2)?;
        } else {
            config.bs.put_bits(si.private_bits, 1)?;
        }
    }

    // Write SCFSI (only for MPEG-I)
    if config.mpeg.version == 3 {
        (0..config.wave.channels as usize).try_for_each(|ch| {
            (0..4).try_for_each(|scfsi_band| config.bs.put_bits(si.scfsi[ch][scfsi_band], 1))
        })?;
    }

    // Write granule information
    for gr in 0..config.mpeg.granules_per_frame as usize {
        for ch in 0..config.wave.channels as usize {
            let gi = &si.gr[gr].ch[ch].tt;

            config.bs.put_bits(gi.part2_3_length, 12)?;
            config.bs.put_bits(gi.big_values, 9)?;
            config.bs.put_bits(gi.global_gain, 8)?;

            if config.mpeg.version == 3 {
                // MPEG_I = 3
                config.bs.put_bits(gi.scalefac_compress, 4)?;
            } else {
                config.bs.put_bits(gi.scalefac_compress, 9)?;
            }

            config.bs.put_bits(0, 1)?; // Window switching flag (always 0 for long blocks)

            (0..3).try_for_each(|region| config.bs.put_bits(gi.table_select[region], 5))?;

            config.bs.put_bits(gi.region0_count, 4)?;
            config.bs.put_bits(gi.region1_count, 3)?;

            if config.mpeg.version == 3 {
                // MPEG_I = 3
                config.bs.put_bits(gi.preflag, 1)?;
            }
            config.bs.put_bits(gi.scalefac_scale, 1)?;
            config.bs.put_bits(gi.count1table_select, 1)?;
        }
    }

    Ok(())
}

/// Huffman encode the quantized values (matches Huffmancodebits exactly)
/// (ref/shine/src/lib/l3bitstream.c:123-165)
fn huffman_code_bits(
    config: &mut ShineGlobalConfig,
    ix: &[i32],
    gi: &GrInfo,
) -> EncodingResult<()> {
    let scalefac = &SHINE_SCALE_FACT_BAND_INDEX[config.mpeg.samplerate_index as usize];
    let bits_start = config.bs.get_bits_count();

    // 1: Write the bigvalues
    let bigvalues = (gi.big_values << 1) as usize;

    let scalefac_index = gi.region0_count + 1;
    let region1_start = scalefac[scalefac_index as usize] as usize;
    let scalefac_index = scalefac_index + gi.region1_count + 1;
    let region2_start = scalefac[scalefac_index as usize] as usize;

    let mut i = 0;
    while i < bigvalues {
        // Get table pointer
        let idx = if i >= region1_start { 1 } else { 0 } + if i >= region2_start { 1 } else { 0 };
        let table_index = gi.table_select[idx];

        // Get huffman code
        if table_index != 0 {
            let x = ix[i];
            let y = ix[i + 1];

            huffman_code(&mut config.bs, table_index as usize, x, y)?;
        }
        i += 2;
    }

    // 2: Write count1 area
    let h = &SHINE_HUFFMAN_TABLE[(gi.count1table_select + 32) as usize];
    let count1_end = bigvalues + ((gi.count1 << 2) as usize);

    let mut i = bigvalues;
    while i < count1_end {
        let v = ix[i];
        let w = ix[i + 1];
        let x = ix[i + 2];
        let y = ix[i + 3];

        huffman_coder_count1(&mut config.bs, h, v, w, x, y)?;
        i += 4;
    }

    // 3: Pad with stuffing bits if necessary
    let bits_used = config.bs.get_bits_count() - bits_start;
    let bits_available = gi.part2_3_length as i32 - gi.part2_length as i32;
    let stuffing_bits = bits_available - bits_used;

    if stuffing_bits > 0 {
        let stuffing_words = stuffing_bits / 32;
        let remaining_bits = stuffing_bits % 32;

        // Due to the nature of the Huffman code tables, we will pad with ones
        for _ in 0..stuffing_words {
            config.bs.put_bits(0xffffffff, 32)?;
        }
        if remaining_bits > 0 {
            config
                .bs
                .put_bits((1u32 << remaining_bits) - 1, remaining_bits)?;
        }
    }

    Ok(())
}

/// Huffman encode count1 region (matches shine_huffman_coder_count1 exactly)
/// (ref/shine/src/lib/l3bitstream.c:174-200)
fn huffman_coder_count1(
    bs: &mut BitstreamWriter,
    h: &HuffCodeTab,
    v: i32,
    w: i32,
    x: i32,
    y: i32,
) -> EncodingResult<()> {
    let mut v = v;
    let mut w = w;
    let mut x = x;
    let mut y = y;

    let signv = abs_and_sign(&mut v);
    let signw = abs_and_sign(&mut w);
    let signx = abs_and_sign(&mut x);
    let signy = abs_and_sign(&mut y);

    let p = v + (w << 1) + (x << 2) + (y << 3);

    if let (Some(table), Some(hlen)) = (h.hb, h.hlen) {
        bs.put_bits(table[p as usize] as u32, hlen[p as usize] as i32)?;

        let mut code = 0u32;
        let mut cbits = 0u32;

        if v != 0 {
            code = signv;
            cbits = 1;
        }
        if w != 0 {
            code = (code << 1) | signw;
            cbits += 1;
        }
        if x != 0 {
            code = (code << 1) | signx;
            cbits += 1;
        }
        if y != 0 {
            code = (code << 1) | signy;
            cbits += 1;
        }

        if cbits > 0 {
            bs.put_bits(code, cbits as i32)?;
        }
    }

    Ok(())
}

/// Huffman encode a pair of values (matches shine_HuffmanCode exactly)
/// (ref/shine/src/lib/l3bitstream.c:203-250)
fn huffman_code(
    bs: &mut BitstreamWriter,
    table_select: usize,
    x: i32,
    y: i32,
) -> EncodingResult<()> {
    let mut x = x;
    let mut y = y;

    let signx = abs_and_sign(&mut x);
    let signy = abs_and_sign(&mut y);

    let h = &SHINE_HUFFMAN_TABLE[table_select];
    let ylen = h.ylen as usize;

    if let (Some(table), Some(hlen)) = (h.hb, h.hlen) {
        if table_select > 15 {
            // ESC-table is used
            let mut linbitsx = 0u32;
            let mut linbitsy = 0u32;
            let linbits = h.linbits;

            if x > 14 {
                linbitsx = (x - 15) as u32;
                x = 15;
            }
            if y > 14 {
                linbitsy = (y - 15) as u32;
                y = 15;
            }

            let idx = (x as usize * ylen) + y as usize;
            let code = table[idx] as u32;
            let cbits = hlen[idx] as u32;

            let mut ext = 0u32;
            let mut xbits = 0u32;

            if x > 14 {
                ext |= linbitsx;
                xbits += linbits;
            }
            if x != 0 {
                ext <<= 1;
                ext |= signx;
                xbits += 1;
            }
            if y > 14 {
                ext <<= linbits;
                ext |= linbitsy;
                xbits += linbits;
            }
            if y != 0 {
                ext <<= 1;
                ext |= signy;
                xbits += 1;
            }

            bs.put_bits(code, cbits as i32)?;
            if xbits > 0 {
                bs.put_bits(ext, xbits as i32)?;
            }
        } else {
            // No ESC-words
            let idx = (x as usize * ylen) + y as usize;
            let mut code = table[idx] as u32;
            let mut cbits = hlen[idx] as u32;

            if x != 0 {
                code <<= 1;
                code |= signx;
                cbits += 1;
            }
            if y != 0 {
                code <<= 1;
                code |= signy;
                cbits += 1;
            }

            bs.put_bits(code, cbits as i32)?;
        }
    }

    Ok(())
}
/// Get absolute value and sign bit (matches shine_abs_and_sign exactly)
/// (ref/shine/src/lib/l3bitstream.c:167-172)
#[inline]
pub fn abs_and_sign(x: &mut i32) -> u32 {
    if *x > 0 {
        0
    } else {
        *x = -*x;
        1
    }
}
