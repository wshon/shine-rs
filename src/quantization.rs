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
            reservoir: BitReservoir::new(128, 44100, 2), // Default values for testing
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
    /// Following shine's quantize function exactly (ref/shine/src/lib/l3loop.c:365-420)
    /// 
    /// Original shine signature: int quantize(int ix[GRANULE_SIZE], int stepsize, shine_global_config *config)
    /// - ix: int array (quantized output) - corresponds to our output parameter
    /// - stepsize: int (quantization step size) - matches our stepsize parameter
    /// - config: shine_global_config* (contains xr array and tables) - corresponds to our mdct_coeffs and internal tables
    /// - return: int (maximum quantized value) - matches our return type
    /// 
    /// CRITICAL: This function modifies the ix array in-place, just like shine
    pub fn quantize(&self, ix: &mut [i32; GRANULE_SIZE], stepsize: i32, xr: &[i32; GRANULE_SIZE]) -> i32 {
        let mut max_value = 0;
        
        // Get the step size from the table - following shine's logic exactly
        // Original shine: scalei = config->l3loop.steptabi[stepsize + 127]; /* 2**(-stepsize/4) */
        let step_index = (stepsize + 127).clamp(0, 255) as usize;
        let scalei = self.step_table_i32[step_index];
        
        // Find maximum absolute value for quick check - following shine's xrmax logic
        // Original shine: xrmax is calculated beforehand and stored in config->l3loop.xrmax
        let xrmax = xr.iter().map(|&x| x.abs()).max().unwrap_or(0);
        
        // Quick check to see if ixmax will be less than 8192 - following shine's logic exactly
        // Original shine: if ((mulr(config->l3loop.xrmax, scalei)) > 165140) /* 8192**(4/3) */
        if Self::multiply_and_round(xrmax, scalei) > 165140 {
            return 16384; // Following shine: no point in continuing, stepsize not big enough
        }
        
        // Main quantization loop - following shine's logic exactly
        // Original shine: for (i = 0, max = 0; i < GRANULE_SIZE; i++)
        for i in 0..GRANULE_SIZE {
            let abs_coeff = xr[i].abs();
            
            if abs_coeff == 0 {
                ix[i] = 0;
                continue;
            }
            
            // Following shine's calculation exactly:
            // Original shine: ln = mulr(labs(config->l3loop.xr[i]), scalei);
            let ln = Self::multiply_and_round(abs_coeff, scalei);
            
            let quantized = if ln < 10000 {
                // Following shine: ln < 10000 catches most values
                // Original shine: ix[i] = config->l3loop.int2idx[ln]; /* quick look up method */
                self.int2idx[ln as usize] as i32
            } else {
                // Following shine: outside table range so have to do it using floats
                // Original shine: scale = config->l3loop.steptab[stepsize + 127]; /* 2**(-stepsize/4) */
                let scale = self.step_table[step_index];
                // Original shine: dbl = ((double)config->l3loop.xrabs[i]) * scale * 4.656612875e-10; /* 0x7fffffff */
                let dbl = (abs_coeff as f64) * (scale as f64) * (1.0 / 0x7fffffff as f64);
                (dbl.sqrt().sqrt() * dbl.sqrt()) as i32 // dbl^(3/4)
            };
            
            // Following shine's comment: "note. ix cannot be negative"
            // Store only absolute values, signs are handled separately in MP3
            ix[i] = quantized;
            
            // Following shine: calculate ixmax while we're here
            // Original shine: if (max < ix[i]) max = ix[i];
            if quantized > max_value {
                max_value = quantized;
            }
        }
        
        max_value
    }
    
    /// Multiply two integers with rounding (fixed-point arithmetic)
    /// Following shine's mulr macro exactly: (int32_t)(((((int64_t)a) * ((int64_t)b)) + 0x80000000LL) >> 32)
    /// 
    /// Original shine signature: #define mulr(a, b) (int32_t)(((((int64_t)a) * ((int64_t)b)) + 0x80000000LL) >> 32)
    /// - a: int32_t input parameter
    /// - b: int32_t input parameter  
    /// - return: int32_t result
    fn multiply_and_round(a: i32, b: i32) -> i32 {
        let result = (a as i64) * (b as i64); // Cast to i64 matches shine's (int64_t) cast
        ((result + 0x80000000i64) >> 32) as i32 // Round and shift like shine's mulr, cast back to i32
    }
    
    /// Calculate quantization step size for given coefficients
    /// Following shine's bin_search_StepSize exactly (ref/shine/src/lib/l3loop.c:774-810)
    /// 
    /// Original shine signature: int bin_search_StepSize(int desired_rate, int ix[GRANULE_SIZE], gr_info *cod_info, shine_global_config *config)
    /// - desired_rate: int (target bit rate)
    /// - ix: int array (quantized coefficients) - corresponds to our temp_output
    /// - cod_info: gr_info* (granule info structure) - corresponds to our granule_info
    /// - return: int (optimal step size)
    pub fn calculate_step_size(&self, mdct_coeffs: &[i32; GRANULE_SIZE], desired_rate: i32, granule_info: &mut GranuleInfo, sample_rate: u32) -> i32 {
        // Binary search for optimal step size
        let mut low = -120;
        let mut high = 120;
        let mut best_step = 0;
        
        while low <= high {
            let mid = (low + high) / 2;
            let mut temp_output = [0i32; GRANULE_SIZE];
            let max_quantized = self.quantize(&mut temp_output, mid, mdct_coeffs);
            
            if max_quantized > 8192 {
                // Step size too small, increase it
                low = mid + 1;
            } else {
                // Calculate exact bit count following shine's bin_search_StepSize logic
                let calculated_bits = self.calculate_exact_bits(&temp_output, granule_info, sample_rate);
                
                if calculated_bits <= desired_rate as usize {
                    best_step = mid;
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }
        
        best_step
    }
    
    /// Calculate exact bit count following shine's bin_search_StepSize logic
    /// This implements the complete shine algorithm for bit counting
    /// 
    /// Original shine sequence in bin_search_StepSize:
    /// calc_runlen(ix, cod_info);           /* rzero,count1,big_values */
    /// bit = count1_bitcount(ix, cod_info); /* count1_table selection */
    /// subdivide(cod_info, config);         /* bigvalues sfb division */
    /// bigv_tab_select(ix, cod_info);       /* codebook selection */
    /// bit += bigv_bitcount(ix, cod_info);  /* bit count */
    /// 
    /// Returns: usize (converted from shine's int for interface compatibility)
    fn calculate_exact_bits(&self, quantized: &[i32; GRANULE_SIZE], granule_info: &mut GranuleInfo, sample_rate: u32) -> usize {
        // Following shine's bin_search_StepSize: calc_runlen -> count1_bitcount -> subdivide -> bigv_tab_select -> bigv_bitcount
        
        // Step 1: Calculate run length (rzero, count1, big_values)
        self.calculate_run_length(quantized, granule_info);
        
        // Step 2: Count1 table selection and bit count
        let count1_bits = self.count1_bitcount(quantized, granule_info);
        
        // Step 3: Subdivide big values region
        self.subdivide_big_values(granule_info, sample_rate);
        
        // Step 4: Select Huffman tables for big values regions
        self.select_big_values_tables(quantized, granule_info);
        
        // Step 5: Count big values bits
        let bigv_bits = self.big_values_bitcount(quantized, granule_info);
        
        // Check for encoding failure
        if bigv_bits == i32::MAX {
            return usize::MAX; // Cannot encode - return max value to indicate failure
        }
        
        // Return total bit count - convert from shine's int to usize for interface
        (count1_bits + bigv_bits) as usize
    }
    
    /// Quantize MDCT coefficients and encode them
    /// Following shine's shine_outer_loop exactly (ref/shine/src/lib/l3loop.c:72-98)
    /// max_bits: int (shine type)
    pub fn quantize_and_encode(
        &mut self,
        mdct_coeffs: &[i32; GRANULE_SIZE],
        max_bits: i32,
        side_info: &mut GranuleInfo,
        output: &mut [i32; GRANULE_SIZE],
        sample_rate: u32
    ) -> EncodingResult<i32> {
        // Use outer loop to find optimal quantization and encoding
        let total_bits = self.outer_loop(mdct_coeffs, max_bits, side_info, sample_rate);
        
        // CRITICAL FIX: The outer_loop already calculated the optimal quantization
        // and updated side_info accordingly. We need to quantize with the final
        // step size to get the coefficients that match the side_info.
        let max_quantized = self.quantize(output, side_info.quantizer_step_size, mdct_coeffs);
        
        if max_quantized > 8192 {
            return Err(EncodingError::QuantizationFailed);
        }
        
        // CRITICAL FIX: DO NOT recalculate run length here!
        // The outer_loop -> inner_loop already calculated the correct run length
        // and all other side_info parameters. Recalculating here would overwrite
        // the correct values with potentially different ones.
        // Following shine's logic: outer_loop calls inner_loop which calls calc_runlen,
        // and that's the final result we should use.
        
        // Set global gain (quantizer step size + 210 as per MP3 spec)
        side_info.global_gain = (side_info.quantizer_step_size + 210) as u32;
        
        // Return total bit count
        Ok(total_bits)
    }
    
    /// Calculate run length encoding information 
    /// Following shine's calc_runlen exactly (ref/shine/src/lib/l3loop.c:429-450)
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
            if quantized[i - 1] <= 1 && quantized[i - 2] <= 1 && quantized[i - 3] <= 1 && quantized[i - 4] <= 1 {
                side_info.count1 += 1;
                i -= 4;
            } else {
                break;
            }
        }
        
        // Set big values count - following shine's logic exactly
        // cod_info->big_values = i >> 1;
        side_info.big_values = (i >> 1) as u32;
    }
    
    /// Inner loop: find optimal Huffman table selection
    /// Returns the number of bits needed for encoding
    /// Following shine's shine_inner_loop exactly (ref/shine/src/lib/l3loop.c:45-70)
    /// max_bits: int (shine type)
    fn inner_loop(&self, original_coeffs: &[i32; GRANULE_SIZE], quantized_coeffs: &mut [i32; GRANULE_SIZE], max_bits: i32, info: &mut GranuleInfo, sample_rate: u32) -> i32 {
        let mut bits;
        
        // Following shine's logic exactly:
        // if (max_bits < 0) cod_info->quantizerStepSize--;
        if max_bits < 0 {
            info.quantizer_step_size = info.quantizer_step_size.saturating_sub(1);
        }
        
        // Main quantization loop - following shine's do-while structure exactly
        // do {
        loop {
            // while (quantize(ix, ++cod_info->quantizerStepSize, config) > 8192)
            //   ; /* within table range? */
            loop {
                info.quantizer_step_size += 1;
                let max_quantized = self.quantize(quantized_coeffs, info.quantizer_step_size, original_coeffs);
                if max_quantized <= 8192 {
                    break;
                }
            }
            
            // Following shine's exact sequence:
            // calc_runlen(ix, cod_info);                     /* rzero,count1,big_values*/
            self.calculate_run_length(quantized_coeffs, info);
            
            // bits = c1bits = count1_bitcount(ix, cod_info); /* count1_table selection*/
            let c1bits = self.count1_bitcount(quantized_coeffs, info);
            bits = c1bits;
            
            // subdivide(cod_info, config);                   /* bigvalues sfb division */
            self.subdivide_big_values(info, sample_rate);
            
            // bigv_tab_select(ix, cod_info);                 /* codebook selection*/
            self.select_big_values_tables(quantized_coeffs, info);
            
            // bits += bvbits = bigv_bitcount(ix, cod_info);  /* bit count */
            let bvbits = self.big_values_bitcount(quantized_coeffs, info);
            bits += bvbits;
            
            // } while (bits > max_bits);
            if max_bits < 0 || bits <= max_bits {
                break;
            }
        }
        
        // return bits;
        bits
    }
    
    /// Outer loop: adjust quantization step size for optimal quality
    /// Returns the total number of bits used
    /// Following shine's shine_outer_loop exactly (ref/shine/src/lib/l3loop.c:72-98)
    /// max_bits: int (shine type)
    fn outer_loop(&self, mdct_coeffs: &[i32; GRANULE_SIZE], max_bits: i32, info: &mut GranuleInfo, sample_rate: u32) -> i32 {
        // Following shine's logic exactly:
        // cod_info->quantizerStepSize = bin_search_StepSize(max_bits, ix, cod_info, config);
        info.quantizer_step_size = self.binary_search_step_size(mdct_coeffs, max_bits, info, sample_rate);
        
        // Following shine's logic: cod_info->part2_length = part2_length(gr, ch, config);
        info.part2_length = self.calculate_part2_length(info);
        
        // Following shine's logic: huff_bits = max_bits - cod_info->part2_length;
        let huffman_bits = max_bits - info.part2_length as i32;
        
        // Quantize coefficients with the selected step size
        let mut quantized = [0i32; GRANULE_SIZE];
        self.quantize(&mut quantized, info.quantizer_step_size, mdct_coeffs);
        
        // Following shine's logic: bits = shine_inner_loop(ix, huff_bits, cod_info, gr, ch, config);
        let bits = self.inner_loop(mdct_coeffs, &mut quantized, huffman_bits, info, sample_rate);
        
        // Following shine's logic: cod_info->part2_3_length = cod_info->part2_length + bits;
        info.part2_3_length = info.part2_length + bits as u32;
        
        info.part2_3_length as i32
    }
    
    /// Binary search for optimal quantization step size
    /// Following shine's bin_search_StepSize exactly (ref/shine/src/lib/l3loop.c:780-810)
    /// desired_rate: int (shine type)
    fn binary_search_step_size(&self, mdct_coeffs: &[i32; GRANULE_SIZE], desired_rate: i32, info: &mut GranuleInfo, sample_rate: u32) -> i32 {
        let mut next = -120;
        let mut count = 120;
        
        // Following shine's binary search algorithm exactly
        // do {
        //   int half = count / 2;
        loop {
            let half = count / 2;
            
            if half == 0 {
                break;
            }
            
            let mut temp_coeffs = [0i32; GRANULE_SIZE];
            
            // Following shine's logic exactly:
            // if (quantize(ix, next + half, config) > 8192)
            //   bit = 100000; /* fail */
            let max_quantized = self.quantize(&mut temp_coeffs, next + half, mdct_coeffs);
            
            let bit = if max_quantized > 8192 {
                100000 // fail - following shine's logic exactly
            } else {
                // Calculate bit count for this step size - following shine's exact sequence
                let mut temp_info = info.clone();
                temp_info.quantizer_step_size = next + half;
                
                // Following shine's sequence exactly:
                // calc_runlen(ix, cod_info);           /* rzero,count1,big_values */
                // bit = count1_bitcount(ix, cod_info); /* count1_table selection */
                // subdivide(cod_info, config);         /* bigvalues sfb division */
                // bigv_tab_select(ix, cod_info);       /* codebook selection */
                // bit += bigv_bitcount(ix, cod_info);  /* bit count */
                self.calculate_run_length(&temp_coeffs, &mut temp_info);
                let c1bits = self.count1_bitcount(&temp_coeffs, &mut temp_info);
                self.subdivide_big_values(&mut temp_info, sample_rate);
                self.select_big_values_tables(&temp_coeffs, &mut temp_info);
                let bvbits = self.big_values_bitcount(&temp_coeffs, &temp_info);
                
                c1bits + bvbits
            };
            
            // Following shine's binary search logic exactly:
            // if (bit < desired_rate)
            //   count = half;
            // else {
            //   next += half;
            //   count -= half;
            // }
            if bit < desired_rate {
                count = half;
            } else {
                next += half;
                count -= half;
            }
        }
        // } while (count > 1);
        
        // return next;
        next
    }
    
    /// Get maximum quantized value in coefficients array
    #[allow(dead_code)]
    fn max_quantized_value(&self, coeffs: &[i32; GRANULE_SIZE]) -> i32 {
        coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0)
    }
    
    /// Count bits needed for count1 region (quadruples)
    /// Following shine's count1_bitcount exactly (ref/shine/src/lib/l3loop.c:452-490)
    /// shine signature: int count1_bitcount(int ix[GRANULE_SIZE], gr_info *cod_info)
    /// Returns: int (shine type)
    fn count1_bitcount(&self, coeffs: &[i32; GRANULE_SIZE], info: &mut GranuleInfo) -> i32 {
        use crate::tables::COUNT1_TABLES;
        
        let mut sum0 = 0i32;  // Following shine's int type
        let mut sum1 = 0i32;  // Following shine's int type
        
        // Following shine's exact loop:
        // for (i = cod_info->big_values << 1, k = 0; k < cod_info->count1; i += 4, k++)
        let mut i = (info.big_values << 1) as usize; // Convert to usize for array indexing
        for _k in 0..info.count1 {
            if i + 3 >= GRANULE_SIZE {
                break;
            }
            
            let v = coeffs[i];
            let w = coeffs[i + 1];
            let x = coeffs[i + 2];
            let y = coeffs[i + 3];
            
            // Following shine's pattern calculation:
            // p = v + (w << 1) + (x << 2) + (y << 3);
            let p = (v.abs().min(1) + (w.abs().min(1) << 1) + (x.abs().min(1) << 2) + (y.abs().min(1) << 3)) as usize;
            
            // Count sign bits - following shine's logic exactly
            let mut signbits = 0i32;  // Following shine's int type
            if v != 0 { signbits += 1; }
            if w != 0 { signbits += 1; }
            if x != 0 { signbits += 1; }
            if y != 0 { signbits += 1; }
            
            sum0 += signbits;
            sum1 += signbits;
            
            // Add Huffman table bits - following shine's logic
            if p < COUNT1_TABLES[0].lengths.len() {
                sum0 += COUNT1_TABLES[0].lengths[p] as i32;
            }
            if p < COUNT1_TABLES[1].lengths.len() {
                sum1 += COUNT1_TABLES[1].lengths[p] as i32;
            }
            
            i += 4;
        }
        
        // Following shine's table selection logic exactly:
        // if (sum0 < sum1) {
        //   cod_info->count1table_select = 0;
        //   return sum0;
        // } else {
        //   cod_info->count1table_select = 1;
        //   return sum1;
        // }
        if sum0 < sum1 {
            info.count1table_select = false;
            sum0
        } else {
            info.count1table_select = true;
            sum1
        }
    }

    
    /// Subdivide big values region into sub-regions 
    /// Following shine's subdivide exactly (ref/shine/src/lib/l3loop.c:492-570)
    /// shine signature: void subdivide(gr_info *cod_info, shine_global_config *config)
    fn subdivide_big_values(&self, info: &mut GranuleInfo, sample_rate: u32) {
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
        // for (thiscount = subdv_table[scfb_anz].region0_count; thiscount; thiscount--) {
        //   if (scalefac_band_long[thiscount + 1] <= bigvalues_region)
        //     break;
        // }
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
        
        // Calculate region1_count - following shine's pointer offset logic exactly:
        // scalefac_band_long += cod_info->region0_count + 1;
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
        
        let region1_index = region0_offset + thiscount as usize + 1;
        info.address2 = if region1_index < scalefac_band_long.len() {
            scalefac_band_long[region1_index] as u32
        } else {
            bigvalues_region as u32
        };
        
        info.address3 = bigvalues_region as u32;
    }
    
    /// Select optimal Huffman tables for big values regions
    /// Following shine's bigv_tab_select exactly (ref/shine/src/lib/l3loop.c:572-590)
    /// shine signature: void bigv_tab_select(int ix[GRANULE_SIZE], gr_info *cod_info)
    fn select_big_values_tables(&self, coeffs: &[i32; GRANULE_SIZE], info: &mut GranuleInfo) {
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
            info.table_select[0] = self.new_choose_table(coeffs, 0, info.address1) as u32;
        }
        
        // if (cod_info->address2 > cod_info->address1)
        //   cod_info->table_select[1] = new_choose_table(ix, cod_info->address1, cod_info->address2);
        if info.address2 > info.address1 {
            info.table_select[1] = self.new_choose_table(coeffs, info.address1, info.address2) as u32;
        }
        
        // if (cod_info->big_values << 1 > cod_info->address2)
        //   cod_info->table_select[2] = new_choose_table(ix, cod_info->address2, cod_info->big_values << 1);
        if (info.big_values << 1) > info.address2 {
            info.table_select[2] = self.new_choose_table(coeffs, info.address2, info.big_values << 1) as u32;
        }
    }
    
    /// Choose the Huffman table 
    /// Following shine's new_choose_table exactly (ref/shine/src/lib/l3loop.c:592-690)
    /// begin: unsigned int (shine type), end: unsigned int (shine type)
    /// Returns: int (shine type)
    fn new_choose_table(&self, coeffs: &[i32; GRANULE_SIZE], begin: u32, end: u32) -> i32 {
        use crate::tables::HUFFMAN_TABLES;
        
        if begin >= end || begin >= GRANULE_SIZE as u32 {
            return 0;
        }
        
        let actual_end = std::cmp::min(end, GRANULE_SIZE as u32);
        let begin_idx = begin as usize;
        let end_idx = actual_end as usize;
        
        // Following shine's ix_max function exactly
        let mut max = 0;
        for i in begin_idx..end_idx {
            if coeffs[i].abs() > max {
                max = coeffs[i].abs();
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
                sum[0] = self.count_bit_region(coeffs, begin, actual_end, choice[0] as u32);
            }
            
            // Following shine's switch statement exactly
            match choice[0] {
                2 => {
                    if let Some(_) = &HUFFMAN_TABLES[3] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 3);
                        if sum[1] <= sum[0] {
                            choice[0] = 3;
                        }
                    }
                },
                5 => {
                    if let Some(_) = &HUFFMAN_TABLES[6] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 6);
                        if sum[1] <= sum[0] {
                            choice[0] = 6;
                        }
                    }
                },
                7 => {
                    if let Some(_) = &HUFFMAN_TABLES[8] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 8);
                        if sum[1] <= sum[0] {
                            choice[0] = 8;
                            sum[0] = sum[1];
                        }
                    }
                    if let Some(_) = &HUFFMAN_TABLES[9] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 9);
                        if sum[1] <= sum[0] {
                            choice[0] = 9;
                        }
                    }
                },
                10 => {
                    if let Some(_) = &HUFFMAN_TABLES[11] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 11);
                        if sum[1] <= sum[0] {
                            choice[0] = 11;
                            sum[0] = sum[1];
                        }
                    }
                    if let Some(_) = &HUFFMAN_TABLES[12] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 12);
                        if sum[1] <= sum[0] {
                            choice[0] = 12;
                        }
                    }
                },
                13 => {
                    if let Some(_) = &HUFFMAN_TABLES[15] {
                        sum[1] = self.count_bit_region(coeffs, begin, actual_end, 15);
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
                sum[0] = self.count_bit_region(coeffs, begin, actual_end, choice[0] as u32);
            }
            if choice[1] > 0 {
                sum[1] = self.count_bit_region(coeffs, begin, actual_end, choice[1] as u32);
            }
            
            // Following shine's logic: if (sum[1] < sum[0]) choice[0] = choice[1];
            if sum[1] < sum[0] {
                choice[0] = choice[1];
            }
        }
        
        choice[0]
    }
    
    /// Count bits needed for big values regions
    /// Following shine's bigv_bitcount exactly (ref/shine/src/lib/l3loop.c:693-710)
    /// shine signature: int bigv_bitcount(int ix[GRANULE_SIZE], gr_info *gi)
    /// Returns: int (shine type)
    fn big_values_bitcount(&self, coeffs: &[i32; GRANULE_SIZE], info: &GranuleInfo) -> i32 {
        let mut bits = 0i32;  // Following shine's int type
        
        // Following shine's logic exactly:
        // if ((table = gi->table_select[0])) bits += count_bit(ix, 0, gi->address1, table);
        if info.table_select[0] != 0 {
            let region_bits = self.count_bit_region(coeffs, 0, info.address1, info.table_select[0]);
            if region_bits == usize::MAX {
                return i32::MAX; // Cannot encode with selected table
            }
            bits += region_bits as i32;
        }
        
        // if ((table = gi->table_select[1])) bits += count_bit(ix, gi->address1, gi->address2, table);
        if info.table_select[1] != 0 {
            let region_bits = self.count_bit_region(coeffs, info.address1, info.address2, info.table_select[1]);
            if region_bits == usize::MAX {
                return i32::MAX; // Cannot encode with selected table
            }
            bits += region_bits as i32;
        }
        
        // if ((table = gi->table_select[2])) bits += count_bit(ix, gi->address2, gi->big_values << 1, table);
        if info.table_select[2] != 0 {
            let region_bits = self.count_bit_region(coeffs, info.address2, info.big_values << 1, info.table_select[2]);
            if region_bits == usize::MAX {
                return i32::MAX; // Cannot encode with selected table
            }
            bits += region_bits as i32;
        }
        
        bits
    }
    
    /// Count bits for a specific region using Huffman table 
    /// Following shine's count_bit exactly (ref/shine/src/lib/l3loop.c:712-778)
    /// start: unsigned int (shine type), end: unsigned int (shine type), table: unsigned int (shine type)
    fn count_bit_region(&self, coeffs: &[i32; GRANULE_SIZE], start: u32, end: u32, table: u32) -> usize {
        use crate::tables::HUFFMAN_TABLES;
        
        // Following shine's logic: if (!table) return 0;
        if table == 0 || table as usize >= HUFFMAN_TABLES.len() {
            return 0;
        }
        
        let huffman_table = match &HUFFMAN_TABLES[table as usize] {
            Some(table) => table,
            None => return 0,
        };
        
        let mut bits = 0usize;
        let mut i = start as usize; // Convert to usize for array indexing
        
        // Following shine's logic exactly:
        // ylen = h->ylen;
        // linbits = h->linbits;
        let ylen = huffman_table.ylen;
        let linbits = huffman_table.linbits;
        
        // Process pairs of coefficients (following shine's logic)
        // if (table > 15) { /* ESC-table is used */
        if table > 15 {
            // for (i = start; i < end; i += 2) {
            while i + 1 < end as usize && i + 1 < GRANULE_SIZE {
                let mut x = coeffs[i].abs();
                let mut y = coeffs[i + 1].abs();
                
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
                if coeffs[i] != 0 { 
                    bits += 1; 
                }
                if coeffs[i + 1] != 0 { 
                    bits += 1; 
                }
                
                i += 2;
            }
        } else {
            // } else { /* No ESC-words */
            while i + 1 < end as usize && i + 1 < GRANULE_SIZE {
                let x = coeffs[i].abs();
                let y = coeffs[i + 1].abs();
                
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
                if coeffs[i] != 0 { 
                    bits += 1; 
                }
                if coeffs[i + 1] != 0 { 
                    bits += 1; 
                }
                
                i += 2;
            }
        }
        
        bits
    }


    
    /// Calculate part2 length (scale factors) following shine's part2_length function
    fn calculate_part2_length(&self, info: &GranuleInfo) -> u32 {
        // Following shine's part2_length function in l3loop.c
        use crate::tables::{SLEN1_TAB, SLEN2_TAB};
        
        let slen1 = SLEN1_TAB[info.scalefac_compress as usize % SLEN1_TAB.len()];
        let slen2 = SLEN2_TAB[info.scalefac_compress as usize % SLEN2_TAB.len()];
        
        let mut bits = 0;
        
        // For MPEG-1, we need to consider SCFSI (Scale Factor Selection Information)
        // For now, assume no SCFSI (gr == 0 or scfsi[band] == false)
        // This matches shine's logic when !gr || !(scfsi[ch][band])
        
        bits += 6 * slen1;  // scalefactor band 0 (6 scalefactors)
        bits += 5 * slen1;  // scalefactor band 1 (5 scalefactors)  
        bits += 5 * slen2;  // scalefactor band 2 (5 scalefactors)
        bits += 5 * slen2;  // scalefactor band 3 (5 scalefactors)
        
        bits as u32
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
    /// Following shine's int type for max_bits
    fn target_bits_strategy() -> impl Strategy<Value = i32> {
        100i32..10000i32
    }

    /// Strategy for generating perceptual entropy values
    fn perceptual_entropy_strategy() -> impl Strategy<Value = f64> {
        0.0f64..1000.0f64
    }

    /// Strategy for generating channel counts
    /// Following shine's int type for channels
    fn channels_strategy() -> impl Strategy<Value = u8> {
        1u8..=2u8
    }

    /// Strategy for generating mean bits per granule
    /// Following shine's int type for mean_bits
    fn mean_bits_strategy() -> impl Strategy<Value = i32> {
        100i32..5000i32
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
            let max1 = quantizer.quantize(&mut output1, step_size, &mdct_coeffs);
            let max2 = quantizer.quantize(&mut output2, step_size + 4, &mdct_coeffs);
            
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
            
            quantizer.quantize(&mut output, step_size, &mdct_coeffs);
            
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
            
            quantizer.quantize(&mut output, step_size, &mdct_coeffs);
            
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
            _mean_bits in mean_bits_strategy(),
            channels in channels_strategy(),
            perceptual_entropy in perceptual_entropy_strategy()
        ) {
            setup_panic_hook();
            
            use crate::reservoir::BitReservoir;
            use crate::bitstream::SideInfo;
            
            let mut reservoir = BitReservoir::new(128, 44100, channels);
            
            // Property 1: Max reservoir bits should be reasonable
            let max_bits = reservoir.max_reservoir_bits(perceptual_entropy, channels);
            prop_assert!(max_bits <= 4095, "Max bits should not exceed 4095");
            prop_assert!(max_bits > 0, "Max bits should be positive");
            
            // Property 2: Frame operations should maintain consistency
            let _side_info = SideInfo::default();
            let stuffing_bits = reservoir.frame_end(channels)?;
            
            // Stuffing bits should be reasonable
            prop_assert!(stuffing_bits >= 0, "Stuffing bits should be non-negative");
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
            
            let max_val = quantizer.quantize(&mut output, 0, &zero_coeffs);
            
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
            
            let mut granule_info = GranuleInfo::default();
            let step_size = quantizer.calculate_step_size(&test_coeffs, 1000, &mut granule_info, 44100);
            
            // Should return a reasonable step size
            assert!(step_size >= -120);
            assert!(step_size <= 120);
        }
    }
}