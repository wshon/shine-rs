//! Huffman encoding for MP3 quantized coefficients
//!
//! This module implements Huffman encoding using the standard MP3
//! Huffman code tables for lossless compression of quantized coefficients.
//! 
//! Following shine's huffman.c and l3loop.c implementation exactly

use crate::bitstream::BitstreamWriter;
use crate::quantization::GranuleInfo;
use crate::error::{EncodingResult, EncodingError};
use crate::tables::{HUFFMAN_TABLES, COUNT1_TABLES, HuffmanTable};

/// Calculate run length encoding information (following shine's calc_runlen)
/// 
/// This mirrors shine's calc_runlen function (ref/shine/src/lib/l3loop.c:429-450)
/// Partitions quantized coefficients into big values, quadruples and zeros.
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients (ix array in shine)
/// * `info` - Granule information to be updated
/// 
/// shine signature: void calc_runlen(int ix[GRANULE_SIZE], gr_info *cod_info)
pub fn calc_runlen(quantized: &[i32; 576], info: &mut GranuleInfo) {
    let mut i = 576;
    
    // Count trailing zero pairs - following shine's logic exactly
    // for (i = GRANULE_SIZE; i > 1; i -= 2)
    //   if (!ix[i - 1] && !ix[i - 2])
    //     rzero++;
    //   else
    //     break;
    while i > 1 {
        if quantized[i - 1] == 0 && quantized[i - 2] == 0 {
            i -= 2;
        } else {
            break;
        }
    }
    
    // Count quadruples (count1 region) - following shine's logic exactly
    // cod_info->count1 = 0;
    // for (; i > 3; i -= 4)
    //   if (ix[i - 1] <= 1 && ix[i - 2] <= 1 && ix[i - 3] <= 1 && ix[i - 4] <= 1)
    //     cod_info->count1++;
    //   else
    //     break;
    info.count1 = 0;
    while i > 3 {
        if quantized[i - 1] <= 1 && quantized[i - 2] <= 1 && quantized[i - 3] <= 1 && quantized[i - 4] <= 1 {
            info.count1 += 1;
            i -= 4;
        } else {
            break;
        }
    }
    
    // Set big values count - following shine's logic exactly
    // cod_info->big_values = i >> 1;
    let calculated_big_values = (i >> 1) as u32;
    
    // CRITICAL: MP3 standard requires big_values <= 288 (576 coefficients / 2)
    if calculated_big_values > 288 {
        info.big_values = 288; // Clamp to maximum allowed
    } else {
        info.big_values = calculated_big_values;
    }
}

/// Subdivide big values region (following shine's subdivide)
/// 
/// This mirrors shine's subdivide function (ref/shine/src/lib/l3loop.c:500-570)
/// Subdivides the bigvalue region which will use separate Huffman tables.
/// 
/// # Arguments
/// * `info` - Granule information to be updated
/// * `sample_rate` - Sample rate for scale factor band calculation
/// 
/// shine signature: void subdivide(gr_info *cod_info, shine_global_config *config)
pub fn subdivide(info: &mut GranuleInfo, sample_rate: u32) {
    use crate::tables::SCALE_FACT_BAND_INDEX;
    
    // Subdivision table from shine (matches subdv_table in l3loop.c exactly)
    const SUBDV_TABLE: [(u32, u32); 23] = [
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 1), (1, 1), (1, 1), 
        (1, 2), (2, 2), (2, 3), (2, 3), (3, 4), (3, 4), (3, 4), (4, 5), 
        (4, 5), (4, 6), (5, 6), (5, 6), (5, 7), (6, 7), (6, 7),
    ];
    
    // Following shine's logic exactly:
    // if (!cod_info->big_values) { /* no big_values region */
    if info.big_values == 0 {
        info.region0_count = 0;
        info.region1_count = 0;
        info.address1 = 0;
        info.address2 = 0;
        info.address3 = 0;
        return;
    }
    
    // Following shine's samplerate_index calculation
    let samplerate_index = match sample_rate {
        44100 => 0, 48000 => 1, 32000 => 2, 22050 => 3, 24000 => 4,
        16000 => 5, 11025 => 6, 12000 => 7, 8000 => 8, _ => 0,
    };
    
    let scalefac_band_long = &SCALE_FACT_BAND_INDEX[samplerate_index];
    let bigvalues_region = (info.big_values * 2) as i32;
    
    // Calculate scfb_anz - following shine's logic exactly:
    // scfb_anz = 0;
    // while (scalefac_band_long[scfb_anz] < bigvalues_region)
    //   scfb_anz++;
    let mut scfb_anz = 0;
    while scfb_anz < scalefac_band_long.len() - 1 && scalefac_band_long[scfb_anz] < bigvalues_region {
        scfb_anz += 1;
    }
    
    if scfb_anz >= SUBDV_TABLE.len() {
        scfb_anz = SUBDV_TABLE.len() - 1;
    }
    
    // Calculate region0_count - following shine's logic exactly:
    let mut thiscount = SUBDV_TABLE[scfb_anz].0;
    while thiscount > 0 {
        if (thiscount as usize + 1) < scalefac_band_long.len() &&
           scalefac_band_long[thiscount as usize + 1] <= bigvalues_region {
            break;
        }
        thiscount -= 1;
    }
    info.region0_count = thiscount;
    
    // Ensure address1 doesn't exceed bigvalues_region
    let calculated_address1 = if (thiscount as usize + 1) < scalefac_band_long.len() {
        scalefac_band_long[thiscount as usize + 1] as u32
    } else {
        bigvalues_region as u32
    };
    info.address1 = std::cmp::min(calculated_address1, bigvalues_region as u32);
    
    // Calculate region1_count - following shine's pointer offset logic exactly:
    let region0_offset = (info.region0_count + 1) as usize;
    let mut thiscount = SUBDV_TABLE[scfb_anz].1;
    while thiscount > 0 {
        let index = region0_offset + thiscount as usize + 1;
        if index < scalefac_band_long.len() &&
           scalefac_band_long[index] <= bigvalues_region {
            break;
        }
        thiscount -= 1;
    }
    info.region1_count = thiscount;
    
    // Ensure address2 doesn't exceed bigvalues_region
    let region1_index = region0_offset + thiscount as usize + 1;
    let calculated_address2 = if region1_index < scalefac_band_long.len() {
        scalefac_band_long[region1_index] as u32
    } else {
        bigvalues_region as u32
    };
    info.address2 = std::cmp::min(calculated_address2, bigvalues_region as u32);
    
    // Ensure address2 >= address1
    info.address2 = std::cmp::max(info.address2, info.address1);
    
    info.address3 = bigvalues_region as u32;
}

/// Select Huffman tables for big values regions (following shine's bigv_tab_select)
/// 
/// This mirrors shine's bigv_tab_select function (ref/shine/src/lib/l3loop.c:572-590)
/// Selects huffman code tables for bigvalues regions.
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients (ix array in shine)
/// * `info` - Granule information to be updated
/// 
/// shine signature: void bigv_tab_select(int ix[GRANULE_SIZE], gr_info *cod_info)
pub fn bigv_tab_select(quantized: &[i32; 576], info: &mut GranuleInfo) {
    // Following shine's initialization exactly:
    // cod_info->table_select[0] = 0;
    // cod_info->table_select[1] = 0;
    // cod_info->table_select[2] = 0;
    info.table_select[0] = 0;
    info.table_select[1] = 0;
    info.table_select[2] = 0;
    
    // Following shine's logic exactly:
    // if (cod_info->address1 > 0)
    //   cod_info->table_select[0] = new_choose_table(ix, 0, cod_info->address1);
    if info.address1 > 0 {
        info.table_select[0] = new_choose_table(quantized, 0, info.address1) as u32;
    }
    
    // if (cod_info->address2 > cod_info->address1)
    //   cod_info->table_select[1] = new_choose_table(ix, cod_info->address1, cod_info->address2);
    if info.address2 > info.address1 {
        info.table_select[1] = new_choose_table(quantized, info.address1, info.address2) as u32;
    }
    
    // if (cod_info->big_values << 1 > cod_info->address2)
    //   cod_info->table_select[2] = new_choose_table(ix, cod_info->address2, cod_info->big_values << 1);
    if (info.big_values << 1) > info.address2 {
        info.table_select[2] = new_choose_table(quantized, info.address2, info.big_values << 1) as u32;
    }
}

/// Count bits for big values region (following shine's bigv_bitcount)
/// 
/// This mirrors shine's bigv_bitcount function (ref/shine/src/lib/l3loop.c:693-710)
/// Counts the number of bits necessary to code the bigvalues region.
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients (ix array in shine)
/// * `info` - Granule information with table selections and addresses
/// 
/// # Returns
/// * Number of bits required (int in shine)
/// 
/// shine signature: int bigv_bitcount(int ix[GRANULE_SIZE], gr_info *gi)
pub fn bigv_bitcount(quantized: &[i32; 576], info: &GranuleInfo) -> i32 {
    let mut bits = 0i32;  // Following shine's int type
    
    // Following shine's logic exactly:
    // if ((table = gi->table_select[0])) bits += count_bit(ix, 0, gi->address1, table);
    if info.table_select[0] != 0 {
        let region_bits = count_bit(quantized, 0, info.address1 as usize, info.table_select[0] as usize);
        if region_bits == usize::MAX {
            return i32::MAX; // Cannot encode with selected table
        }
        bits += region_bits as i32;
    }
    
    // if ((table = gi->table_select[1])) bits += count_bit(ix, gi->address1, gi->address2, table);
    if info.table_select[1] != 0 {
        let region_bits = count_bit(quantized, info.address1 as usize, info.address2 as usize, info.table_select[1] as usize);
        if region_bits == usize::MAX {
            return i32::MAX; // Cannot encode with selected table
        }
        bits += region_bits as i32;
    }
    
    // if ((table = gi->table_select[2])) bits += count_bit(ix, gi->address2, gi->big_values << 1, table);
    if info.table_select[2] != 0 {
        let region_bits = count_bit(quantized, info.address2 as usize, (info.big_values << 1) as usize, info.table_select[2] as usize);
        if region_bits == usize::MAX {
            return i32::MAX; // Cannot encode with selected table
        }
        bits += region_bits as i32;
    }
    
    bits
}

/// Count bits for count1 region (following shine's count1_bitcount)
/// 
/// This mirrors shine's count1_bitcount function (ref/shine/src/lib/l3loop.c:452-490)
/// Determines the number of bits to encode the quadruples.
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients (ix array in shine)
/// * `info` - Granule information to be updated
/// 
/// # Returns
/// * Number of bits required (int in shine)
/// 
/// shine signature: int count1_bitcount(int ix[GRANULE_SIZE], gr_info *cod_info)
pub fn count1_bitcount(quantized: &[i32; 576], info: &mut GranuleInfo) -> i32 {
    let mut sum0 = 0i32;
    let mut sum1 = 0i32;
    
    // Following shine's logic exactly
    let big_values_end = (info.big_values << 1) as usize;
    
    for k in 0..info.count1 {
        let i = big_values_end + (k as usize * 4);
        if i + 3 >= 576 { break; }
        
        let v = quantized[i];
        let w = quantized[i + 1];
        let x = quantized[i + 2];
        let y = quantized[i + 3];
        
        let p = (if v != 0 { 1 } else { 0 }) +
                (if w != 0 { 2 } else { 0 }) +
                (if x != 0 { 4 } else { 0 }) +
                (if y != 0 { 8 } else { 0 });
        
        let mut signbits = 0;
        if v != 0 { signbits += 1; }
        if w != 0 { signbits += 1; }
        if x != 0 { signbits += 1; }
        if y != 0 { signbits += 1; }
        
        sum0 += signbits;
        sum1 += signbits;
        
        // Add table bits (tables 32 and 33 are count1 tables)
        if (p as usize) < COUNT1_TABLES[0].lengths.len() {
            sum0 += COUNT1_TABLES[0].lengths[p as usize] as i32;
        }
        
        if (p as usize) < COUNT1_TABLES[1].lengths.len() {
            sum1 += COUNT1_TABLES[1].lengths[p as usize] as i32;
        }
    }
    
    // Select the better table
    if sum0 < sum1 {
        info.count1table_select = 0;
        sum0
    } else {
        info.count1table_select = 1;
        sum1
    }
}

/// Choose optimal Huffman table (following shine's new_choose_table)
/// 
/// This mirrors shine's new_choose_table function (ref/shine/src/lib/l3loop.c:600-690)
/// Chooses the Huffman table that will encode the region with the fewest bits.
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients (ix array in shine)
/// * `begin` - Start index (unsigned int in shine)
/// * `end` - End index (unsigned int in shine)
/// 
/// # Returns
/// * Table index (int in shine)
/// 
/// shine signature: int new_choose_table(int ix[GRANULE_SIZE], unsigned int begin, unsigned int end)
fn new_choose_table(quantized: &[i32; 576], begin: u32, end: u32) -> i32 {
    if begin >= end || begin >= 576 {
        return 0;
    }
    
    let actual_end = std::cmp::min(end, 576);
    let begin_idx = begin as usize;
    let end_idx = actual_end as usize;
    
    // Following shine's ix_max function exactly
    let mut max = 0;
    for i in begin_idx..end_idx {
        if quantized[i].abs() > max {
            max = quantized[i].abs();
        }
    }
    
    // Following shine's logic: if (!max) return 0;
    if max == 0 {
        return 0;
    }
    
    let mut choice = [0i32; 2];
    let mut sum = [usize::MAX; 2];
    
    // Following shine's logic exactly:
    // if (max < 15) {
    if max < 15 {
        // try tables with no linbits
        // for (i = 14; i--; ) if (shine_huffman_table[i].xlen > max)
        for i in (0..15).rev() {
            if i == 4 || i == 14 { continue; } // Skip non-existent tables
            
            if let Some(table) = &HUFFMAN_TABLES[i] {
                if table.xlen > max as u32 {
                    choice[0] = i as i32;
                    break;
                }
            }
        }
        
        if choice[0] > 0 {
            sum[0] = count_bit(quantized, begin_idx, end_idx, choice[0] as usize);
        }
        
        // Following shine's switch statement exactly
        match choice[0] {
            2 => {
                if HUFFMAN_TABLES[3].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 3);
                    if sum[1] <= sum[0] {
                        choice[0] = 3;
                    }
                }
            },
            5 => {
                if HUFFMAN_TABLES[6].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 6);
                    if sum[1] <= sum[0] {
                        choice[0] = 6;
                    }
                }
            },
            7 => {
                if HUFFMAN_TABLES[8].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 8);
                    if sum[1] <= sum[0] {
                        choice[0] = 8;
                        sum[0] = sum[1];
                    }
                }
                if HUFFMAN_TABLES[9].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 9);
                    if sum[1] <= sum[0] {
                        choice[0] = 9;
                    }
                }
            },
            10 => {
                if HUFFMAN_TABLES[11].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 11);
                    if sum[1] <= sum[0] {
                        choice[0] = 11;
                        sum[0] = sum[1];
                    }
                }
                if HUFFMAN_TABLES[12].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 12);
                    if sum[1] <= sum[0] {
                        choice[0] = 12;
                    }
                }
            },
            13 => {
                if HUFFMAN_TABLES[15].is_some() {
                    sum[1] = count_bit(quantized, begin_idx, end_idx, 15);
                    if sum[1] <= sum[0] {
                        choice[0] = 15;
                    }
                }
            },
            _ => {}
        }
    } else {
        // Following shine's logic: try tables with linbits
        // max -= 15;
        let max_linbits = max - 15;
        
        // for (i = 15; i < 24; i++) if (shine_huffman_table[i].linmax >= max)
        for i in 15..24 {
            if let Some(table) = &HUFFMAN_TABLES[i] {
                if table.linmax >= max_linbits as u32 {
                    choice[0] = i as i32;
                    break;
                }
            }
        }
        
        // for (i = 24; i < 32; i++) if (shine_huffman_table[i].linmax >= max)
        for i in 24..32 {
            if let Some(table) = &HUFFMAN_TABLES[i] {
                if table.linmax >= max_linbits as u32 {
                    choice[1] = i as i32;
                    break;
                }
            }
        }
        
        if choice[0] > 0 {
            sum[0] = count_bit(quantized, begin_idx, end_idx, choice[0] as usize);
        }
        if choice[1] > 0 {
            sum[1] = count_bit(quantized, begin_idx, end_idx, choice[1] as usize);
        }
        
        // Following shine's logic: if (sum[1] < sum[0]) choice[0] = choice[1];
        if sum[1] < sum[0] {
            choice[0] = choice[1];
        }
    }
    
    choice[0]
}

/// Count bits for a specific region using Huffman table (following shine's count_bit)
/// 
/// This mirrors shine's count_bit function (ref/shine/src/lib/l3loop.c:712-778)
/// Counts the number of bits necessary to code the subregion.
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients (ix array in shine)
/// * `start` - Start index (unsigned int in shine)
/// * `end` - End index (unsigned int in shine)  
/// * `table` - Huffman table index (unsigned int in shine)
/// 
/// # Returns
/// * Number of bits required (int in shine)
/// 
/// shine signature: int count_bit(int ix[GRANULE_SIZE], unsigned int start, unsigned int end, unsigned int table)
fn count_bit(quantized: &[i32; 576], start: usize, end: usize, table: usize) -> usize {
    // Following shine's logic: if (!table) return 0;
    if table == 0 || table >= HUFFMAN_TABLES.len() {
        return 0;
    }
    
    let huffman_table = match &HUFFMAN_TABLES[table] {
        Some(table) => table,
        None => return 0,
    };
    
    let mut bits = 0usize;
    let mut i = start; // Convert to usize for array indexing
    
    // Following shine's logic exactly:
    // ylen = h->ylen;
    // linbits = h->linbits;
    let ylen = huffman_table.ylen;
    let linbits = huffman_table.linbits;
    
    // Process pairs of coefficients (following shine's logic)
    // if (table > 15) { /* ESC-table is used */
    if table > 15 {
        // for (i = start; i < end; i += 2) {
        while i + 1 < end && i + 1 < 576 {
            let mut x = quantized[i].abs();
            let mut y = quantized[i + 1].abs();
            
            // Following shine's ESC logic exactly:
            // if (x > 14) { x = 15; sum += linbits; }
            // if (y > 14) { y = 15; sum += linbits; }
            if x > 14 {
                x = 15;
                bits += linbits as usize;
            }
            if y > 14 {
                y = 15;
                bits += linbits as usize;
            }
            
            // sum += h->hlen[(x * ylen) + y];
            let table_idx = (x as u32 * ylen + y as u32) as usize;
            if table_idx < huffman_table.lengths.len() {
                bits += huffman_table.lengths[table_idx] as usize;
            } else {
                return usize::MAX; // Invalid table index
            }
            
            // Add sign bits: if (x) sum++; if (y) sum++;
            if quantized[i] != 0 { 
                bits += 1; 
            }
            if quantized[i + 1] != 0 { 
                bits += 1; 
            }
            
            i += 2;
        }
    } else {
        // } else { /* No ESC-words */
        while i + 1 < end && i + 1 < 576 {
            let x = quantized[i].abs();
            let y = quantized[i + 1].abs();
            
            // Check if values are within table range
            if x > huffman_table.xlen as i32 || y > huffman_table.ylen as i32 {
                return usize::MAX; // Cannot encode with this table
            }
            
            // sum += h->hlen[(x * ylen) + y];
            let table_idx = (x as u32 * ylen + y as u32) as usize;
            if table_idx < huffman_table.lengths.len() {
                bits += huffman_table.lengths[table_idx] as usize;
            } else {
                return usize::MAX; // Invalid table index
            }
            
            // Add sign bits: if (x != 0) sum++; if (y != 0) sum++;
            if quantized[i] != 0 { 
                bits += 1; 
            }
            if quantized[i + 1] != 0 { 
                bits += 1; 
            }
            
            i += 2;
        }
    }
    
    bits
}

/// Encode big values region using Huffman tables (following shine's Huffmancodebits)
/// 
/// Encodes the big values region of quantized coefficients using
/// the appropriate Huffman tables. This mirrors the big values part of
/// shine's Huffmancodebits function (ref/shine/src/lib/l3bitstream.c:174-190)
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients with sign information (ix array in shine)
/// * `info` - Granule information with table selections and addresses
/// * `output` - Bitstream writer for output
pub fn encode_big_values(
    quantized: &[i32; 576],
    info: &GranuleInfo,
    output: &mut BitstreamWriter
) -> EncodingResult<usize> {
    let mut bits_written = 0;
    
    // Following shine's Huffmancodebits logic:
    // bigvalues = gi->big_values << 1;
    let big_values = (info.big_values as usize) << 1;
    
    // for (i = 0; i < bigvalues; i += 2) {
    //   /* get table pointer */
    //   int idx = (i >= region1Start) + (i >= region2Start);
    //   unsigned tableindex = gi->table_select[idx];
    //   /* get huffman code */
    //   if (tableindex) {
    //     x = ix[i];
    //     y = ix[i + 1];
    //     shine_HuffmanCode(&config->bs, tableindex, x, y);
    //   }
    // }
    
    let mut i = 0;
    while i < big_values && i + 1 < 576 {
        // Determine which region we're in
        let region_idx = if i >= info.address2 as usize {
            2
        } else if i >= info.address1 as usize {
            1
        } else {
            0
        };
        
        let table_index = info.table_select[region_idx] as usize;
        
        if table_index != 0 {
            let x = quantized[i];
            let y = quantized[i + 1];
            bits_written += encode_huffman_pair(x, y, table_index, output)?;
        }
        
        i += 2;
    }
    
    Ok(bits_written)
}

/// Encode count1 region using count1 tables (following shine's Huffmancodebits)
/// 
/// Encodes the count1 region (values of ±1 or 0) using the
/// specialized count1 Huffman tables. This mirrors the count1 part of
/// shine's Huffmancodebits function (ref/shine/src/lib/l3bitstream.c:192-200)
/// 
/// # Arguments
/// * `quantized` - Quantized coefficients with sign information (ix array in shine)
/// * `info` - Granule information with count1 settings
/// * `output` - Bitstream writer for output
pub fn encode_count1(
    quantized: &[i32; 576],
    info: &GranuleInfo,
    output: &mut BitstreamWriter
) -> EncodingResult<usize> {
    let mut bits_written = 0;
    
    // Following shine's Huffmancodebits logic:
    // h = &shine_huffman_table[gi->count1table_select + 32];
    let table_index = if info.count1table_select != 0 { 1 } else { 0 };
    let h = &COUNT1_TABLES[table_index];
    
    // bigvalues = gi->big_values << 1;
    let big_values = (info.big_values as usize) << 1;
    
    // count1End = bigvalues + (gi->count1 << 2);
    let count1_end = big_values + ((info.count1 as usize) << 2);
    
    // for (i = bigvalues; i < count1End; i += 4) {
    let mut i = big_values;
    while i < count1_end && i + 3 < 576 {
        let v = quantized[i];
        let w = quantized[i + 1];
        let x = quantized[i + 2];
        let y = quantized[i + 3];
        
        // shine_huffman_coder_count1(&config->bs, h, v, w, x, y);
        bits_written += encode_count1_quadruple(v, w, x, y, h, output)?;
        
        i += 4;
    }
    
    Ok(bits_written)
}

/// Encode a Huffman pair (following shine's shine_HuffmanCode function)
/// 
/// Implements the pseudocode of page 98 of the IS.
/// This mirrors shine's shine_HuffmanCode function (ref/shine/src/lib/l3bitstream.c:243-309)
/// 
/// # Arguments
/// * `x` - First coefficient (with sign)
/// * `y` - Second coefficient (with sign)
/// * `table_select` - Huffman table index
/// * `output` - Bitstream writer for output
fn encode_huffman_pair(x: i32, y: i32, table_select: usize, output: &mut BitstreamWriter) -> EncodingResult<usize> {
    if table_select >= HUFFMAN_TABLES.len() {
        return Err(EncodingError::HuffmanError(
            format!("Invalid Huffman table index: {}", table_select)
        ));
    }
    
    let h = match &HUFFMAN_TABLES[table_select] {
        Some(huffman_table) => huffman_table,
        None => return Err(EncodingError::HuffmanError(
            format!("Huffman table {} is not available", table_select)
        )),
    };
    
    // Following shine's shine_HuffmanCode implementation:
    let mut abs_x = x.unsigned_abs();
    let mut abs_y = y.unsigned_abs();
    let signx = if x < 0 { 1u32 } else { 0u32 };
    let signy = if y < 0 { 1u32 } else { 0u32 };
    
    let ylen = h.ylen;
    let mut bits_written = 0;
    
    if table_select > 15 {
        // ESC-table is used
        let mut linbitsx = 0u32;
        let mut linbitsy = 0u32;
        let linbits = h.linbits;
        
        // if (x > 14) { linbitsx = x - 15; x = 15; }
        if abs_x > 14 {
            linbitsx = abs_x - 15;
            abs_x = 15;
        }
        
        // if (y > 14) { linbitsy = y - 15; y = 15; }
        if abs_y > 14 {
            linbitsy = abs_y - 15;
            abs_y = 15;
        }
        
        // idx = (x * ylen) + y;
        let idx = (abs_x * ylen + abs_y) as usize;
        if idx >= h.codes.len() {
            return Err(EncodingError::HuffmanError(
                format!("Huffman code index {} out of bounds", idx)
            ));
        }
        
        // code = h->table[idx]; cbits = h->hlen[idx];
        let code = h.codes[idx] as u32;
        let cbits = h.lengths[idx];
        bits_written += cbits as usize;
        
        // shine_putbits(bs, code, cbits);
        output.write_bits(code, cbits);
        
        // Build extension bits following shine's exact logic
        let mut ext = 0u32;
        let mut xbits = 0u8;
        
        // if (x > 14) { ext |= linbitsx; xbits += linbits; }
        if x.abs() > 14 {
            ext |= linbitsx;
            xbits += linbits as u8;
        }
        
        // if (x != 0) { ext <<= 1; ext |= signx; xbits += 1; }
        if x != 0 {
            ext <<= 1;
            ext |= signx;
            xbits += 1;
        }
        
        // if (y > 14) { ext <<= linbits; ext |= linbitsy; xbits += linbits; }
        if y.abs() > 14 {
            ext <<= linbits as u8;
            ext |= linbitsy;
            xbits += linbits as u8;
        }
        
        // if (y != 0) { ext <<= 1; ext |= signy; xbits += 1; }
        if y != 0 {
            ext <<= 1;
            ext |= signy;
            xbits += 1;
        }
        
        // shine_putbits(bs, ext, xbits);
        if xbits > 0 {
            output.write_bits(ext, xbits);
            bits_written += xbits as usize;
        }
    } else {
        // No ESC-words
        // idx = (x * ylen) + y;
        let idx = (abs_x * ylen + abs_y) as usize;
        if idx >= h.codes.len() {
            return Err(EncodingError::HuffmanError(
                format!("Huffman code index {} out of bounds", idx)
            ));
        }
        
        // code = h->table[idx]; cbits = h->hlen[idx];
        let mut code = h.codes[idx] as u32;
        let mut cbits = h.lengths[idx];
        
        // if (x != 0) { code <<= 1; code |= signx; cbits += 1; }
        if x != 0 {
            code <<= 1;
            code |= signx;
            cbits += 1;
        }
        
        // if (y != 0) { code <<= 1; code |= signy; cbits += 1; }
        if y != 0 {
            code <<= 1;
            code |= signy;
            cbits += 1;
        }
        
        // shine_putbits(bs, code, cbits);
        output.write_bits(code, cbits);
        bits_written += cbits as usize;
    }
    
    Ok(bits_written)
}

/// Encode count1 quadruple (following shine's shine_huffman_coder_count1)
/// 
/// This mirrors shine's shine_huffman_coder_count1 function 
/// (ref/shine/src/lib/l3bitstream.c:213-241)
fn encode_count1_quadruple(
    v: i32, w: i32, x: i32, y: i32,
    h: &HuffmanTable,
    output: &mut BitstreamWriter
) -> EncodingResult<usize> {
    // Following shine's shine_huffman_coder_count1 implementation:
    
    // Get absolute values and signs
    let abs_v = v.abs();
    let abs_w = w.abs();
    let abs_x = x.abs();
    let abs_y = y.abs();
    
    let signv = if v < 0 { 1u32 } else { 0u32 };
    let signw = if w < 0 { 1u32 } else { 0u32 };
    let signx = if x < 0 { 1u32 } else { 0u32 };
    let signy = if y < 0 { 1u32 } else { 0u32 };
    
    // Convert to count1 format (0 or 1)
    let v_bit = if abs_v != 0 { 1u32 } else { 0u32 };
    let w_bit = if abs_w != 0 { 1u32 } else { 0u32 };
    let x_bit = if abs_x != 0 { 1u32 } else { 0u32 };
    let y_bit = if abs_y != 0 { 1u32 } else { 0u32 };
    
    // p = v + (w << 1) + (x << 2) + (y << 3);
    let p = (v_bit + (w_bit << 1) + (x_bit << 2) + (y_bit << 3)) as usize;
    
    if p >= h.codes.len() {
        return Err(EncodingError::HuffmanError(
            format!("Count1 table index {} out of bounds", p)
        ));
    }
    
    // shine_putbits(bs, h->table[p], h->hlen[p]);
    let code = h.codes[p] as u32;
    let length = h.lengths[p];
    output.write_bits(code, length);
    let mut bits_written = length as usize;
    
    // Build sign bits following shine's logic
    let mut sign_code = 0u32;
    let mut sign_bits = 0u8;
    
    // if (v) { code = signv; cbits = 1; }
    if v != 0 {
        sign_code = signv;
        sign_bits = 1;
    }
    
    // if (w) { code = (code << 1) | signw; cbits++; }
    if w != 0 {
        sign_code = (sign_code << 1) | signw;
        sign_bits += 1;
    }
    
    // if (x) { code = (code << 1) | signx; cbits++; }
    if x != 0 {
        sign_code = (sign_code << 1) | signx;
        sign_bits += 1;
    }
    
    // if (y) { code = (code << 1) | signy; cbits++; }
    if y != 0 {
        sign_code = (sign_code << 1) | signy;
        sign_bits += 1;
    }
    
    // shine_putbits(bs, code, cbits);
    if sign_bits > 0 {
        output.write_bits(sign_code, sign_bits);
        bits_written += sign_bits as usize;
    }
    
    Ok(bits_written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitstream::BitstreamWriter;
    use crate::quantization::GranuleInfo;
    use proptest::prelude::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// 设置自定义 panic 钩子，只输出通用错误信息
    fn setup_panic_hook() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|_| {
                eprintln!("Test failed: Property test assertion failed");
            }));
        });
    }

    #[test]
    fn test_calc_runlen_basic() {
        let mut quantized = [0i32; 576];
        let mut info = GranuleInfo::default();
        
        // Test with all zeros
        calc_runlen(&quantized, &mut info);
        assert_eq!(info.big_values, 0);
        assert_eq!(info.count1, 0);
        
        // Test with some values
        quantized[0] = 5;
        quantized[1] = 3;
        calc_runlen(&quantized, &mut info);
        assert_eq!(info.big_values, 1);
    }

    #[test]
    fn test_subdivide_basic() {
        let mut info = GranuleInfo::default();
        info.big_values = 10;
        
        subdivide(&mut info, 44100);
        
        // Should have set addresses
        assert!(info.address3 > 0);
    }

    #[test]
    fn test_bigv_tab_select_basic() {
        let quantized = [0i32; 576];
        let mut info = GranuleInfo::default();
        info.big_values = 10;
        info.address1 = 10;
        info.address2 = 15;
        
        bigv_tab_select(&quantized, &mut info);
        
        // Should have selected tables
        assert!(info.table_select[0] <= 31);
        assert!(info.table_select[1] <= 31);
        assert!(info.table_select[2] <= 31);
    }

    #[test]
    fn test_bigv_bitcount_basic() {
        let quantized = [0i32; 576];
        let mut info = GranuleInfo::default();
        info.big_values = 5;
        info.table_select = [1, 2, 3];
        info.address1 = 5;
        info.address2 = 8;
        
        let bits = bigv_bitcount(&quantized, &info);
        assert!(bits >= 0);
    }

    #[test]
    fn test_count1_bitcount_basic() {
        let mut quantized = [0i32; 576];
        let mut info = GranuleInfo::default();
        
        // Set up count1 region
        quantized[100] = 1;
        quantized[101] = -1;
        quantized[102] = 0;
        quantized[103] = 1;
        
        info.big_values = 50;
        info.count1 = 5;
        
        let bits = count1_bitcount(&quantized, &mut info);
        assert!(bits >= 0);
        assert!(info.count1table_select <= 1);
    }

    #[test]
    fn test_encode_big_values_basic() {
        let quantized = [0i32; 576];
        let mut info = GranuleInfo::default();
        info.big_values = 5;
        info.table_select = [1, 2, 3];
        info.address1 = 5;
        info.address2 = 8;
        
        let mut output = BitstreamWriter::new(100);
        let result = encode_big_values(&quantized, &info, &mut output);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_encode_count1_basic() {
        let mut quantized = [0i32; 576];
        let mut info = GranuleInfo::default();
        
        // Set up count1 values
        quantized[100] = 1;
        quantized[101] = -1;
        quantized[102] = 0;
        quantized[103] = 1;
        
        info.big_values = 50;
        info.count1 = 5;
        info.count1table_select = 0;
        
        let mut output = BitstreamWriter::new(100);
        let result = encode_count1(&quantized, &info, &mut output);
        
        assert!(result.is_ok());
    }

    // Property-based tests
    prop_compose! {
        fn valid_quantized_coefficients()(
            coeffs in prop::collection::vec(-15i32..=15, 576)
        ) -> [i32; 576] {
            let mut result = [0i32; 576];
            for (i, &coeff) in coeffs.iter().enumerate() {
                result[i] = coeff;
            }
            result
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_calc_runlen_properties(
            quantized in valid_quantized_coefficients()
        ) {
            setup_panic_hook();
            
            let mut info = GranuleInfo::default();
            calc_runlen(&quantized, &mut info);
            
            // big_values should be within valid range
            prop_assert!(info.big_values <= 288, "big_values exceeds maximum");
            
            // count1 should be reasonable
            prop_assert!(info.count1 <= 144, "count1 exceeds reasonable limit");
        }

        #[test]
        fn test_subdivide_properties(
            big_values in 0u32..=288,
            sample_rate in prop::sample::select(vec![44100u32, 48000, 32000, 22050, 24000, 16000])
        ) {
            setup_panic_hook();
            
            let mut info = GranuleInfo::default();
            info.big_values = big_values;
            
            subdivide(&mut info, sample_rate);
            
            // Addresses should be in correct order
            prop_assert!(info.address1 <= info.address2, "address1 > address2");
            prop_assert!(info.address2 <= info.address3, "address2 > address3");
            
            // address3 should match big_values * 2
            prop_assert_eq!(info.address3, big_values * 2, "address3 mismatch");
        }

        #[test]
        fn test_huffman_encoding_stability(
            quantized in valid_quantized_coefficients()
        ) {
            setup_panic_hook();
            
            let mut info = GranuleInfo::default();
            calc_runlen(&quantized, &mut info);
            subdivide(&mut info, 44100);
            bigv_tab_select(&quantized, &mut info);
            
            // Functions should not panic with valid input
            let _bits = bigv_bitcount(&quantized, &info);
            let _count1_bits = count1_bitcount(&quantized, &mut info);
            
            // Encoding should not panic
            let mut output = BitstreamWriter::new(1000);
            let _result1 = encode_big_values(&quantized, &info, &mut output);
            let _result2 = encode_count1(&quantized, &info, &mut output);
        }
    }
}