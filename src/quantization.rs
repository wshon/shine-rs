//! Quantization and rate control for MP3 encoding
//!
//! This module implements the quantization loop that controls the
//! trade-off between audio quality and bitrate by adjusting quantization
//! step sizes and managing the bit reservoir.

use crate::error::{EncodingError, EncodingResult};
use crate::reservoir::BitReservoir;

/// Number of MDCT coefficients per granule
pub const GRANULE_SIZE: usize = 576;

/// Quantization loop for rate control and quality management
pub struct QuantizationLoop {
    /// Quantization step table (floating point)
    step_table: [f32; 256],
    /// Integer version of step table for fixed-point arithmetic
    step_table_i32: [i32; 256],
    /// Integer to index lookup table for quantization
    int2idx: [u32; 10000],
    /// Bit reservoir for rate control
    #[allow(dead_code)]
    reservoir: BitReservoir,
}

/// Granule information structure
#[derive(Debug, Clone)]
pub struct GranuleInfo {
    /// Length of part2_3 data in bits
    pub part2_3_length: u32,
    /// Number of big values
    pub big_values: u32,
    /// Global gain value
    pub global_gain: u32,
    /// Scale factor compression
    pub scalefac_compress: u32,
    /// Huffman table selection
    pub table_select: [u32; 3],
    /// Region 0 count
    pub region0_count: u32,
    /// Region 1 count
    pub region1_count: u32,
    /// Pre-emphasis flag
    pub preflag: bool,
    /// Scale factor scale
    pub scalefac_scale: bool,
    /// Count1 table selection
    pub count1table_select: bool,
    /// Quantizer step size
    pub quantizer_step_size: i32,
    /// Number of count1 quadruples
    pub count1: u32,
    /// Part2 length in bits
    pub part2_length: u32,
    /// Region addresses for Huffman coding
    pub address1: u32,
    pub address2: u32,
    pub address3: u32,
}

impl Default for GranuleInfo {
    fn default() -> Self {
        Self {
            part2_3_length: 0,
            big_values: 0,
            global_gain: 210,
            scalefac_compress: 0,
            table_select: [1, 1, 1], // Use table 1 as default (table 0 doesn't exist)
            region0_count: 0,
            region1_count: 0,
            preflag: false,
            scalefac_scale: false,
            count1table_select: false,
            quantizer_step_size: 0,
            count1: 0,
            part2_length: 0,
            address1: 0,
            address2: 0,
            address3: 0,
        }
    }
}



impl QuantizationLoop {
    /// Create a new quantization loop
    pub fn new() -> Self {
        let mut quantizer = Self {
            step_table: [0.0; 256],
            step_table_i32: [0; 256],
            int2idx: [0; 10000],
            reservoir: BitReservoir::new(7680), // Maximum reservoir size for Layer III
        };
        
        quantizer.initialize_tables();
        quantizer
    }
    
    /// Initialize quantization lookup tables
    fn initialize_tables(&mut self) {
        // Initialize step table: 2^(-stepsize/4)
        // The table is inverted (negative power) from the equation given
        // in the spec because it is quicker to do x*y than x/y.
        for i in 0..256 {
            self.step_table[i] = (2.0_f32).powf((127 - i as i32) as f32 / 4.0);
            
            // Convert to fixed point with extra bit of accuracy
            // The table is multiplied by 2 to give an extra bit of accuracy.
            if (self.step_table[i] * 2.0) > 0x7fffffff as f32 {
                self.step_table_i32[i] = 0x7fffffff;
            } else {
                self.step_table_i32[i] = ((self.step_table[i] * 2.0) + 0.5) as i32;
            }
        }
        
        // Initialize int2idx table: quantization index lookup
        // The 0.5 is for rounding, the 0.0946 comes from the spec.
        for i in 0..10000 {
            let val = (i as f64).sqrt().sqrt() * (i as f64).sqrt() - 0.0946 + 0.5;
            self.int2idx[i] = val.max(0.0) as u32;
        }
    }
    
    /// Quantize MDCT coefficients using non-linear quantization
    /// Returns the maximum quantized value
    pub fn quantize(&self, mdct_coeffs: &[i32; GRANULE_SIZE], stepsize: i32, output: &mut [i32; GRANULE_SIZE]) -> i32 {
        let mut max_value = 0;
        
        // Get the step size from the table
        let step_index = (stepsize + 127).clamp(0, 255) as usize;
        let scalei = self.step_table_i32[step_index];
        
        // Find maximum absolute value for quick check
        let xrmax = mdct_coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0);
        
        // Quick check to see if max quantized value will be less than 8192
        // This speeds up the early calls to binary search
        if Self::multiply_and_round(xrmax, scalei) > 165140 { // 8192^(4/3)
            return 16384; // No point in continuing, stepsize not big enough
        }
        
        for i in 0..GRANULE_SIZE {
            let abs_coeff = mdct_coeffs[i].abs();
            
            if abs_coeff == 0 {
                output[i] = 0;
                continue;
            }
            
            // Multiply coefficient by step size
            let ln = Self::multiply_and_round(abs_coeff, scalei);
            
            let quantized = if ln < 10000 {
                // Use lookup table for fast quantization
                self.int2idx[ln as usize] as i32
            } else {
                // Outside table range, use floating point calculation
                let scale = self.step_table[step_index];
                let dbl = (abs_coeff as f64) * (scale as f64) * 4.656612875e-10; // 1.0 / 0x7fffffff
                (dbl.sqrt().sqrt() * dbl.sqrt()) as i32 // dbl^(3/4)
            };
            
            // CRITICAL FIX: Store only absolute values, signs are handled separately in MP3
            output[i] = quantized;
            
            // Track maximum value
            if quantized > max_value {
                max_value = quantized;
            }
        }
        
        max_value
    }
    
    /// Multiply two integers with rounding (fixed-point arithmetic)
    fn multiply_and_round(a: i32, b: i32) -> i32 {
        let result = (a as i64) * (b as i64);
        ((result + (1 << 30)) >> 31) as i32 // Round and shift
    }
    
    /// Calculate quantization step size for given coefficients
    pub fn calculate_step_size(&self, mdct_coeffs: &[i32; GRANULE_SIZE], target_bits: usize) -> i32 {
        // Binary search for optimal step size
        let mut low = -120;
        let mut high = 120;
        let mut best_step = 0;
        
        while low <= high {
            let mid = (low + high) / 2;
            let mut temp_output = [0i32; GRANULE_SIZE];
            let max_quantized = self.quantize(mdct_coeffs, mid, &mut temp_output);
            
            if max_quantized > 8192 {
                // Step size too small, increase it
                low = mid + 1;
            } else {
                // Calculate approximate bit count (simplified)
                let estimated_bits = self.estimate_bits(&temp_output);
                
                if estimated_bits <= target_bits {
                    best_step = mid;
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }
        
        best_step
    }
    
    /// Estimate the number of bits needed for quantized coefficients
    /// This is a simplified estimation for the binary search
    fn estimate_bits(&self, quantized: &[i32; GRANULE_SIZE]) -> usize {
        let mut bits = 0;
        
        // Count non-zero coefficients and estimate bits
        for &coeff in quantized.iter() {
            if coeff != 0 {
                let abs_val = coeff.abs();
                if abs_val == 1 {
                    bits += 2; // Rough estimate for small values
                } else if abs_val <= 15 {
                    bits += 4; // Rough estimate for medium values
                } else {
                    bits += 8; // Rough estimate for large values
                }
            }
        }
        
        bits
    }
    
    /// Quantize MDCT coefficients and encode them
    pub fn quantize_and_encode(
        &mut self,
        mdct_coeffs: &[i32; GRANULE_SIZE],
        max_bits: usize,
        side_info: &mut GranuleInfo,
        output: &mut [i32; GRANULE_SIZE],
        sample_rate: u32
    ) -> EncodingResult<usize> {
        // Use outer loop to find optimal quantization and encoding
        let total_bits = self.outer_loop(mdct_coeffs, max_bits, side_info, sample_rate);
        
        // Quantize coefficients with the selected step size
        let max_quantized = self.quantize(mdct_coeffs, side_info.quantizer_step_size, output);
        
        if max_quantized > 8192 {
            return Err(EncodingError::QuantizationFailed);
        }
        
        // Set global gain (quantizer step size + 210 as per MP3 spec)
        side_info.global_gain = (side_info.quantizer_step_size + 210) as u32;
        
        // Return total bit count
        Ok(total_bits)
    }
    
    /// Calculate run length encoding information (following shine's calc_runlen exactly)
    fn calculate_run_length(&self, quantized: &[i32; GRANULE_SIZE], side_info: &mut GranuleInfo) {
        let mut i = GRANULE_SIZE;
        
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
        side_info.count1 = 0;
        while i > 3 {
            // CRITICAL FIX: Follow shine's exact logic
            // In shine, quantized coefficients are non-negative (ix cannot be negative)
            // So we check if values are <= 1, which means 0 or 1 only
            if quantized[i - 1] <= 1 && quantized[i - 2] <= 1 &&
               quantized[i - 3] <= 1 && quantized[i - 4] <= 1 {
                side_info.count1 += 1;
                i -= 4;
            } else {
                break;
            }
        }
        
        // Set big values count - KEY FIX: use right shift like shine
        // cod_info->big_values = i >> 1;
        side_info.big_values = (i >> 1) as u32;
    }
    
    /// Inner loop: find optimal Huffman table selection
    /// Returns the number of bits needed for encoding
    fn inner_loop(&self, coeffs: &mut [i32; GRANULE_SIZE], max_bits: usize, info: &mut GranuleInfo, sample_rate: u32) -> usize {
        let mut bits;
        
        // Ensure quantized values are within table range
        while self.max_quantized_value(coeffs) > 8192 {
            info.quantizer_step_size += 1;
            self.quantize_coefficients(coeffs, info.quantizer_step_size);
        }
        
        // Calculate run length encoding info
        self.calculate_run_length(coeffs, info);
        
        // Count bits for count1 region (quadruples)
        let c1bits = self.count1_bitcount(coeffs, info);
        bits = c1bits;
        
        // Subdivide big values region
        self.subdivide_big_values(info, sample_rate);
        
        // Select optimal Huffman tables for big values
        self.select_big_values_tables(coeffs, info);
        
        // Count bits for big values region
        let bvbits = self.big_values_bitcount(coeffs, info);
        bits += bvbits;
        
        // Continue until we're within the bit limit
        while bits > max_bits {
            info.quantizer_step_size += 1;
            self.quantize_coefficients(coeffs, info.quantizer_step_size);
            
            if self.max_quantized_value(coeffs) > 8192 {
                continue;
            }
            
            self.calculate_run_length(coeffs, info);
            bits = self.count1_bitcount(coeffs, info);
            self.subdivide_big_values(info, sample_rate);
            self.select_big_values_tables(coeffs, info);
            bits += self.big_values_bitcount(coeffs, info);
        }
        
        bits
    }
    
    /// Outer loop: adjust quantization step size for optimal quality
    /// Returns the total number of bits used
    fn outer_loop(&self, mdct_coeffs: &[i32; GRANULE_SIZE], max_bits: usize, info: &mut GranuleInfo, sample_rate: u32) -> usize {
        // Binary search for optimal quantization step size
        info.quantizer_step_size = self.binary_search_step_size(mdct_coeffs, max_bits, info, sample_rate);
        
        // Calculate part2 length (scale factors)
        info.part2_length = self.calculate_part2_length(info);
        
        // Calculate available bits for Huffman coding
        let huffman_bits = max_bits.saturating_sub(info.part2_length as usize);
        
        // Quantize coefficients with the selected step size
        let mut quantized = [0i32; GRANULE_SIZE];
        self.quantize(mdct_coeffs, info.quantizer_step_size, &mut quantized);
        
        // Run inner loop to optimize Huffman coding
        let bits = self.inner_loop(&mut quantized, huffman_bits, info, sample_rate);
        
        // Set total part2_3 length
        info.part2_3_length = info.part2_length + bits as u32;
        
        info.part2_3_length as usize
    }
    
    /// Binary search for optimal quantization step size
    fn binary_search_step_size(&self, mdct_coeffs: &[i32; GRANULE_SIZE], desired_rate: usize, info: &mut GranuleInfo, sample_rate: u32) -> i32 {
        let mut low = -120;
        let mut high = 120;
        let mut best_step = 0;
        
        while low <= high {
            let mid = (low + high) / 2;
            let mut temp_coeffs = [0i32; GRANULE_SIZE];
            let max_quantized = self.quantize(mdct_coeffs, mid, &mut temp_coeffs);
            
            if max_quantized > 8192 {
                // Step size too small, need larger step
                low = mid + 1;
            } else {
                // Calculate bit count for this step size
                let mut temp_info = info.clone();
                temp_info.quantizer_step_size = mid;
                
                self.calculate_run_length(&temp_coeffs, &mut temp_info);
                let c1bits = self.count1_bitcount(&temp_coeffs, &temp_info);
                self.subdivide_big_values(&mut temp_info, sample_rate);
                self.select_big_values_tables(&temp_coeffs, &mut temp_info);
                let bvbits = self.big_values_bitcount(&temp_coeffs, &temp_info);
                let total_bits = c1bits + bvbits;
                
                if total_bits <= desired_rate {
                    best_step = mid;
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }
        
        best_step
    }
    
    /// Get maximum quantized value in coefficients array
    fn max_quantized_value(&self, coeffs: &[i32; GRANULE_SIZE]) -> i32 {
        coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0)
    }
    
    /// Quantize coefficients in place
    fn quantize_coefficients(&self, coeffs: &mut [i32; GRANULE_SIZE], step_size: i32) {
        // This is a simplified version - in practice would use the full quantize method
        let step_index = (step_size + 127).clamp(0, 255) as usize;
        let scale = self.step_table[step_index];
        
        for coeff in coeffs.iter_mut() {
            if *coeff != 0 {
                let abs_val = coeff.abs();
                let quantized = ((abs_val as f32) * scale).round() as i32;
                // CRITICAL FIX: Store only absolute values, signs are handled separately in MP3
                *coeff = quantized;
            }
        }
    }
    
    /// Count bits needed for count1 region (quadruples)
    fn count1_bitcount(&self, coeffs: &[i32; GRANULE_SIZE], info: &GranuleInfo) -> usize {
        let mut bits = 0;
        let start_index = (info.big_values * 2) as usize;
        let mut i = start_index;
        
        // Count quadruples in count1 region
        for _ in 0..info.count1 {
            if i + 3 < GRANULE_SIZE {
                let v = coeffs[i].abs().min(1);
                let w = coeffs[i + 1].abs().min(1);
                let x = coeffs[i + 2].abs().min(1);
                let y = coeffs[i + 3].abs().min(1);
                
                // Simplified bit counting for quadruples
                let pattern = v + (w << 1) + (x << 2) + (y << 3);
                bits += self.estimate_quadruple_bits(pattern);
                
                // Add sign bits
                if coeffs[i] != 0 { bits += 1; }
                if coeffs[i + 1] != 0 { bits += 1; }
                if coeffs[i + 2] != 0 { bits += 1; }
                if coeffs[i + 3] != 0 { bits += 1; }
                
                i += 4;
            }
        }
        
        bits
    }
    
    /// Estimate bits needed for a quadruple pattern
    fn estimate_quadruple_bits(&self, pattern: i32) -> usize {
        // Simplified estimation - in practice would use actual Huffman tables
        match pattern {
            0 => 1,      // All zeros
            1..=7 => 3,  // Simple patterns
            _ => 5,      // Complex patterns
        }
    }
    
    /// Subdivide big values region into sub-regions (following shine's subdivide function exactly)
    fn subdivide_big_values(&self, info: &mut GranuleInfo, sample_rate: u32) {
        use crate::tables::SCALE_FACT_BAND_INDEX;
        
        // Subdivision table from shine (matches subdv_table in l3loop.c)
        const SUBDV_TABLE: [(u32, u32); 23] = [
            (0, 0), // 0 bands
            (0, 0), // 1 bands
            (0, 0), // 2 bands
            (0, 0), // 3 bands
            (0, 0), // 4 bands
            (0, 1), // 5 bands
            (1, 1), // 6 bands
            (1, 1), // 7 bands
            (1, 2), // 8 bands
            (2, 2), // 9 bands
            (2, 3), // 10 bands
            (2, 3), // 11 bands
            (3, 4), // 12 bands
            (3, 4), // 13 bands
            (3, 4), // 14 bands
            (4, 5), // 15 bands
            (4, 5), // 16 bands
            (4, 6), // 17 bands
            (5, 6), // 18 bands
            (5, 6), // 19 bands
            (5, 7), // 20 bands
            (6, 7), // 21 bands
            (6, 7), // 22 bands
        ];
        
        if info.big_values == 0 {
            // No big_values region
            info.region0_count = 0;
            info.region1_count = 0;
            info.address1 = 0;
            info.address2 = 0;
            info.address3 = 0;
            return;
        }
        
        // Following shine's logic exactly - get correct samplerate_index
        let samplerate_index = match sample_rate {
            44100 => 0,
            48000 => 1, 
            32000 => 2,
            22050 => 3,
            24000 => 4,
            16000 => 5,
            11025 => 6,
            12000 => 7,
            8000 => 8,
            _ => 0, // Default fallback
        };
        let scalefac_band_long = &SCALE_FACT_BAND_INDEX[samplerate_index];
        
        let bigvalues_region = (info.big_values * 2) as i32;
        
        // Calculate scfb_anz (scale factor band count)
        let mut scfb_anz = 0;
        while scfb_anz < 22 && scalefac_band_long[scfb_anz] < bigvalues_region {
            scfb_anz += 1;
        }
        
        // Ensure scfb_anz is within bounds for SUBDV_TABLE
        if scfb_anz >= SUBDV_TABLE.len() {
            scfb_anz = SUBDV_TABLE.len() - 1;
        }
        
        // Calculate region0_count
        let mut thiscount = SUBDV_TABLE[scfb_anz].0;
        while thiscount > 0 {
            if (thiscount as usize + 1) < scalefac_band_long.len() &&
               scalefac_band_long[thiscount as usize + 1] <= bigvalues_region {
                break;
            }
            thiscount -= 1;
        }
        info.region0_count = thiscount;
        info.address1 = if (thiscount as usize + 1) < scalefac_band_long.len() {
            scalefac_band_long[thiscount as usize + 1] as u32
        } else {
            bigvalues_region as u32
        };
        
        // Calculate region1_count
        let region0_offset = (info.region0_count + 1) as usize;
        let mut thiscount = SUBDV_TABLE[scfb_anz].1;
        while thiscount > 0 {
            if (region0_offset + thiscount as usize + 1) < scalefac_band_long.len() &&
               scalefac_band_long[region0_offset + thiscount as usize + 1] <= bigvalues_region {
                break;
            }
            thiscount -= 1;
        }
        info.region1_count = thiscount;
        info.address2 = if (region0_offset + thiscount as usize + 1) < scalefac_band_long.len() {
            scalefac_band_long[region0_offset + thiscount as usize + 1] as u32
        } else {
            bigvalues_region as u32
        };
        
        info.address3 = bigvalues_region as u32;
    }
    
    /// Select optimal Huffman tables for big values regions
    /// Following shine's bigv_tab_select function exactly
    fn select_big_values_tables(&self, coeffs: &[i32; GRANULE_SIZE], info: &mut GranuleInfo) {
        use crate::huffman::HuffmanEncoder;
        
        // Initialize all table selections to 0 (following shine's logic)
        info.table_select[0] = 0;
        info.table_select[1] = 0;
        info.table_select[2] = 0;
        
        let encoder = HuffmanEncoder::new();
        
        // Region 0 - following shine's logic: if (cod_info->address1 > 0)
        if info.address1 > 0 {
            info.table_select[0] = encoder.select_table(coeffs, 0, info.address1 as usize) as u32;
        }
        
        // Region 1 - following shine's logic: if (cod_info->address2 > cod_info->address1)
        if info.address2 > info.address1 {
            info.table_select[1] = encoder.select_table(coeffs, info.address1 as usize, info.address2 as usize) as u32;
        }
        
        // Region 2 - following shine's logic: if (cod_info->big_values << 1 > cod_info->address2)
        if (info.big_values << 1) > info.address2 {
            info.table_select[2] = encoder.select_table(coeffs, info.address2 as usize, (info.big_values << 1) as usize) as u32;
        }
    }
    
    /// Count bits needed for big values regions
    fn big_values_bitcount(&self, coeffs: &[i32; GRANULE_SIZE], info: &GranuleInfo) -> usize {
        let mut bits = 0;
        
        // Region 0 - following shine's logic: skip if table is 0
        if info.table_select[0] != 0 && info.address1 > 0 {
            bits += self.count_region_bits(coeffs, 0, info.address1 as usize, info.table_select[0]);
        }
        
        // Region 1 - following shine's logic: skip if table is 0
        if info.table_select[1] != 0 && info.address2 > info.address1 {
            bits += self.count_region_bits(coeffs, info.address1 as usize, info.address2 as usize, info.table_select[1]);
        }
        
        // Region 2 - following shine's logic: skip if table is 0
        if info.table_select[2] != 0 && info.address3 > info.address2 {
            bits += self.count_region_bits(coeffs, info.address2 as usize, info.address3 as usize, info.table_select[2]);
        }
        
        bits
    }
    
    /// Count bits for a specific region with given Huffman table
    fn count_region_bits(&self, coeffs: &[i32; GRANULE_SIZE], start: usize, end: usize, table: u32) -> usize {
        // Following shine's logic: return 0 if table is 0
        if table == 0 {
            return 0;
        }
        
        let mut bits = 0;
        let mut i = start;
        
        while i + 1 < end && i + 1 < GRANULE_SIZE {
            let x = coeffs[i].abs();
            let y = coeffs[i + 1].abs();
            
            // Estimate bits based on table and values
            bits += self.estimate_pair_bits(x, y, table);
            
            // Add sign bits
            if coeffs[i] != 0 { bits += 1; }
            if coeffs[i + 1] != 0 { bits += 1; }
            
            i += 2;
        }
        
        bits
    }
    
    /// Estimate bits needed for a coefficient pair
    fn estimate_pair_bits(&self, x: i32, y: i32, table: u32) -> usize {
        // Following shine's logic: return 0 if table is 0
        if table == 0 {
            return 0;
        }
        
        // Simplified estimation - in practice would use actual Huffman tables
        let max_val = x.max(y);
        
        match table {
            1..=3 => if max_val <= 1 { 2 } else { 100 }, // Small value tables (skip table 4)
            5..=9 => if max_val <= 3 { 4 } else { 100 }, // Medium value tables
            10..=13 => if max_val <= 7 { 6 } else { 100 }, // Large value tables (skip table 14)
            15..=31 => {
                // Tables with escape sequences
                let base_bits = 8;
                let escape_bits = if max_val > 15 { (max_val - 15) * 2 } else { 0 };
                base_bits + escape_bits as usize
            },
            _ => 100, // Invalid table
        }
    }
    
    /// Calculate part2 length (scale factors)
    fn calculate_part2_length(&self, _info: &GranuleInfo) -> u32 {
        // Simplified calculation - in practice would calculate actual scale factor bits
        // This depends on scale factor compression and SCFSI
        42 // Typical value for scale factors
    }
}

impl Default for QuantizationLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Set up custom panic hook to avoid verbose parameter output
    fn setup_panic_hook() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|_| {
                eprintln!("Test failed: Property test assertion failed");
            }));
        });
    }

    /// Strategy for generating valid MDCT coefficients
    fn mdct_coeffs_strategy() -> impl Strategy<Value = [i32; GRANULE_SIZE]> {
        prop::collection::vec(-32768i32..32768i32, GRANULE_SIZE)
            .prop_map(|v| {
                let mut arr = [0i32; GRANULE_SIZE];
                arr.copy_from_slice(&v);
                arr
            })
    }

    /// Strategy for generating valid step sizes
    fn step_size_strategy() -> impl Strategy<Value = i32> {
        -120i32..120i32
    }

    /// Strategy for generating target bit counts
    fn target_bits_strategy() -> impl Strategy<Value = usize> {
        100usize..10000usize
    }

    /// Strategy for generating perceptual entropy values
    fn perceptual_entropy_strategy() -> impl Strategy<Value = f64> {
        0.0f64..1000.0f64
    }

    /// Strategy for generating channel counts
    fn channels_strategy() -> impl Strategy<Value = usize> {
        1usize..=2usize
    }

    /// Strategy for generating mean bits per granule
    fn mean_bits_strategy() -> impl Strategy<Value = usize> {
        100usize..5000usize
    }

    // Feature: rust-mp3-encoder, Property 7: 量化和比特率控制
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_quantization_and_bitrate_control(
            mdct_coeffs in mdct_coeffs_strategy(),
            target_bits in target_bits_strategy()
        ) {
            setup_panic_hook();
            
            let mut quantizer = QuantizationLoop::new();
            let mut output = [0i32; GRANULE_SIZE];
            let mut side_info = GranuleInfo::default();
            
            // Test quantization process
            let result = quantizer.quantize_and_encode(&mdct_coeffs, target_bits, &mut side_info, &mut output, 44100);
            
            // Property 1: Quantization should succeed for valid inputs
            prop_assert!(result.is_ok(), "Quantization should succeed");
            
            // Property 2: Quantized values should be within valid range
            for &val in output.iter() {
                prop_assert!(val.abs() <= 8192, "Quantized values should be within range");
            }
            
            // Property 3: Global gain should be reasonable
            prop_assert!(side_info.global_gain >= 90, "Global gain too low");
            prop_assert!(side_info.global_gain <= 330, "Global gain too high");
            
            // Property 4: Big values count should be reasonable
            prop_assert!(side_info.big_values <= 288, "Big values count too high");
        }

        #[test]
        fn test_quantization_step_size_adjustment(
            mdct_coeffs in mdct_coeffs_strategy(),
            step_size in step_size_strategy()
        ) {
            setup_panic_hook();
            
            let quantizer = QuantizationLoop::new();
            let mut output1 = [0i32; GRANULE_SIZE];
            let mut output2 = [0i32; GRANULE_SIZE];
            
            // Test with two different step sizes
            let max1 = quantizer.quantize(&mdct_coeffs, step_size, &mut output1);
            let max2 = quantizer.quantize(&mdct_coeffs, step_size + 4, &mut output2);
            
            // Property: Larger step size should generally produce smaller quantized values
            if max1 > 0 && max2 > 0 {
                prop_assert!(max2 <= max1, "Larger step size should produce smaller values");
            }
            
            // Property: All quantized values should be within valid range
            prop_assert!(max1 <= 16384, "Max quantized value should be within range");
            prop_assert!(max2 <= 16384, "Max quantized value should be within range");
        }

        #[test]
        fn test_quantization_preserves_zero_coefficients(
            mdct_coeffs in mdct_coeffs_strategy(),
            step_size in step_size_strategy()
        ) {
            setup_panic_hook();
            
            let quantizer = QuantizationLoop::new();
            let mut output = [0i32; GRANULE_SIZE];
            
            quantizer.quantize(&mdct_coeffs, step_size, &mut output);
            
            // Property: Zero input coefficients should produce zero output
            for i in 0..GRANULE_SIZE {
                if mdct_coeffs[i] == 0 {
                    prop_assert_eq!(output[i], 0, "Zero coefficients should remain zero");
                }
            }
        }

        #[test]
        fn test_quantization_absolute_values_only(
            mdct_coeffs in mdct_coeffs_strategy(),
            step_size in step_size_strategy()
        ) {
            setup_panic_hook();
            
            let quantizer = QuantizationLoop::new();
            let mut output = [0i32; GRANULE_SIZE];
            
            quantizer.quantize(&mdct_coeffs, step_size, &mut output);
            
            // Property: Quantized values should be absolute values only (MP3 standard)
            for i in 0..GRANULE_SIZE {
                if output[i] != 0 {
                    prop_assert!(output[i] >= 0, "Quantized values should be non-negative");
                }
            }
        }

        // Feature: rust-mp3-encoder, Property 8: 比特储备池机制
        #[test]
        fn test_bit_reservoir_integration(
            mean_bits in mean_bits_strategy(),
            channels in channels_strategy(),
            perceptual_entropy in perceptual_entropy_strategy()
        ) {
            setup_panic_hook();
            
            use crate::reservoir::BitReservoir;
            
            let mut reservoir = BitReservoir::new(7680);
            reservoir.frame_begin(mean_bits);
            
            // Property 1: Max reservoir bits should be reasonable
            let max_bits = reservoir.max_reservoir_bits(perceptual_entropy, channels);
            prop_assert!(max_bits <= 4095, "Max bits should not exceed 4095");
            prop_assert!(max_bits > 0, "Max bits should be positive");
            
            // Property 2: Frame operations should maintain consistency
            let initial_bits = reservoir.available_bits();
            let (stuffing_bits, _drain_bits) = reservoir.frame_end(channels);
            
            // Stuffing bits should be reasonable
            prop_assert!(stuffing_bits <= initial_bits + mean_bits, "Stuffing bits should be reasonable");
        }
    }

    #[cfg(test)]
    mod unit_tests {
        use super::*;

        #[test]
        fn test_quantization_loop_creation() {
            let quantizer = QuantizationLoop::new();
            
            // Test that tables are initialized
            assert_ne!(quantizer.step_table[0], 0.0);
            assert_ne!(quantizer.step_table_i32[0], 0);
            assert_ne!(quantizer.int2idx[100], 0);
        }

        #[test]
        fn test_quantization_with_zero_input() {
            let quantizer = QuantizationLoop::new();
            let zero_coeffs = [0i32; GRANULE_SIZE];
            let mut output = [0i32; GRANULE_SIZE];
            
            let max_val = quantizer.quantize(&zero_coeffs, 0, &mut output);
            
            assert_eq!(max_val, 0);
            assert!(output.iter().all(|&x| x == 0));
        }

        #[test]
        fn test_run_length_calculation() {
            let quantizer = QuantizationLoop::new();
            let mut side_info = GranuleInfo::default();
            
            // Test with some specific patterns
            let mut test_coeffs = [0i32; GRANULE_SIZE];
            test_coeffs[0] = 100;
            test_coeffs[1] = 50;
            test_coeffs[2] = 1;
            test_coeffs[3] = 1;
            
            quantizer.calculate_run_length(&test_coeffs, &mut side_info);
            
            assert!(side_info.big_values > 0);
        }

        #[test]
        fn test_step_size_calculation() {
            let quantizer = QuantizationLoop::new();
            let test_coeffs = [100i32; GRANULE_SIZE];
            
            let step_size = quantizer.calculate_step_size(&test_coeffs, 1000);
            
            // Should return a reasonable step size
            assert!(step_size >= -120);
            assert!(step_size <= 120);
        }
    }
}