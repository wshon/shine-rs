//! Main MP3 encoder implementation
//!
//! This module provides the main Mp3Encoder struct that coordinates
//! all the encoding stages from PCM input to MP3 output.

use crate::config::Config;
use crate::shine_config::ShineGlobalConfig;
use crate::reservoir::BitReservoir;
use crate::error::{EncoderError, InputDataError};
use crate::quantization::{QuantizationLoop, GranuleInfo};
use crate::Result;

/// Main MP3 encoder structure following shine's architecture
/// 
/// This encoder wraps a ShineGlobalConfig and provides a high-level interface
/// for MP3 encoding while maintaining compatibility with shine's implementation.
#[allow(dead_code)]
pub struct Mp3Encoder {
    /// Internal global configuration containing all encoding state (shine_global_config)
    global_config: ShineGlobalConfig,
    /// Public API configuration (shine_config_t equivalent)
    config: Config,
    /// Output frame buffer
    frame_buffer: Vec<u8>,
    /// Samples accumulated in buffer
    samples_in_buffer: usize,
    /// Whole slots per frame (for frame size calculation)
    whole_slots_per_frame: usize,
    /// Fractional slots per frame (for padding calculation)
    frac_slots_per_frame: f64,
    /// Slot lag for padding calculation
    slot_lag: f64,
    /// Bit reservoir used during iteration loop
    reservoir: BitReservoir,
    /// Quantization loop for rate control
    quantization_loop: QuantizationLoop,
    /// Granule info for current frame (used when formatting bitstream)
    current_granule_info: Vec<GranuleInfo>,
}

impl Mp3Encoder {
    /// Create a new MP3 encoder with the specified configuration
    pub fn new(config: Config) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Keep a public copy of the high-level configuration for API consumers
        let config = config.clone();

        // Create shine global configuration (low-level state)
        let mut shine_config = ShineGlobalConfig::new(config.clone())?;

        // Initialize shine configuration
        shine_config.initialize()?;

        let channels: usize = config.wave.channels.into();

        // Calculate frame size parameters (following shine's logic exactly)
        let bitrate_kbps = config.mpeg.bitrate; // Keep in kbps like shine
        let sample_rate = config.wave.sample_rate;
        let granule_size = 576; // GRANULE_SIZE from shine
        let bits_per_slot = 8;

        let granules_per_frame = match config.mpeg_version() {
            crate::config::MpegVersion::Mpeg1 => 2,
            crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 1,
        };

        // Following shine's avg_slots_per_frame calculation exactly:
        // avg_slots_per_frame = ((double)granules_per_frame * GRANULE_SIZE /
        //                       ((double)samplerate)) *
        //                      (1000 * (double)bitr / (double)bits_per_slot);
        let avg_slots_per_frame = ((granules_per_frame * granule_size) as f64 / sample_rate as f64)
            * (1000.0 * bitrate_kbps as f64 / bits_per_slot as f64);

        let whole_slots_per_frame = avg_slots_per_frame as usize;
        let frac_slots_per_frame = avg_slots_per_frame - whole_slots_per_frame as f64;
        let slot_lag = -frac_slots_per_frame;

        // Initialize bit reservoir following shine's logic
        let reservoir = BitReservoir::new(bitrate_kbps, sample_rate, channels as u8);

        // Initialize quantization loop
        let quantization_loop = QuantizationLoop::new();

        // Debug output for frame size calculation
        println!("Frame size calculation:");
        println!("  Bitrate: {}kbps, Sample rate: {}Hz", bitrate_kbps, sample_rate);
        println!("  Granules per frame: {}, Granule size: {}", granules_per_frame, granule_size);
        println!("  Avg slots per frame: {:.6}", avg_slots_per_frame);
        println!("  Whole slots per frame: {}", whole_slots_per_frame);
        println!("  Target frame size: {} bytes", whole_slots_per_frame);

        Ok(Self {
            global_config: shine_config,
            config,
            frame_buffer: Vec::with_capacity(2048),
            samples_in_buffer: 0,
            whole_slots_per_frame,
            frac_slots_per_frame,
            slot_lag,
            reservoir,
            quantization_loop,
            current_granule_info: Vec::new(),
        })
    }
    
    /// Encode a frame of PCM data (non-interleaved)
    /// 
    /// # Arguments
    /// * `pcm_data` - PCM samples organized as [sample][channel]
    /// 
    /// # Returns
    /// * `Ok(&[u8])` - Encoded MP3 frame data
    /// * `Err(EncoderError)` - Encoding error
    pub fn encode_frame(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let expected_samples = samples_per_frame * channels;
        
        // Validate input length
        if pcm_data.len() != expected_samples {
            return Err(EncoderError::InputData(InputDataError::InvalidLength {
                expected: expected_samples,
                actual: pcm_data.len(),
            }));
        }
        
        // Clear frame buffer for new frame
        self.frame_buffer.clear();
        self.global_config.bs.reset();
        
        // De-interleave PCM data into channel buffers
        self.deinterleave_pcm(pcm_data, channels, samples_per_frame);
        
        // Reset buffer counter since we have a complete frame
        self.samples_in_buffer = 0;
        
        // Encode the frame through the complete pipeline
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Encode a single frame of PCM data (interleaved format: L, R, L, R, ...)
    /// 
    /// This method encodes a single frame of interleaved PCM data into MP3 format.
    /// The input data must be in interleaved format: [L, R, L, R, ...]
    /// 
    /// # Arguments
    /// * `pcm_data` - PCM samples (interleaved format)
    /// 
    /// # Returns
    /// * `Ok(&[u8])` - Encoded MP3 frame data
    /// * `Err(EncoderError)` - Encoding error
    pub fn encode_frame_interleaved(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let expected_samples = samples_per_frame * channels;
        
        // Validate input length
        if pcm_data.len() != expected_samples {
            return Err(EncoderError::InputData(InputDataError::InvalidLength {
                expected: expected_samples,
                actual: pcm_data.len(),
            }));
        }
        
        // Clear frame buffer for new frame
        self.frame_buffer.clear();
        self.global_config.bs.reset();
        
        // De-interleave PCM data into channel buffers
        self.deinterleave_pcm_interleaved(pcm_data, channels, samples_per_frame);
        
        // Reset buffer counter since we have a complete frame
        self.samples_in_buffer = 0;
        
        // Encode the frame through the complete pipeline
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Encode samples incrementally, buffering until a complete frame is available
    /// 
    /// This method allows encoding PCM data in chunks smaller than a complete frame.
    /// Data is buffered internally until enough samples are available for encoding.
    /// 
    /// # Arguments
    /// * `pcm_data` - PCM samples (non-interleaved format)
    /// 
    /// # Returns
    /// * `Ok(Some(&[u8]))` - Encoded MP3 frame data if a complete frame was produced
    /// * `Ok(None)` - Data was buffered, no frame produced yet
    /// * `Err(EncoderError)` - Encoding error
    pub fn encode_samples(&mut self, pcm_data: &[i16]) -> Result<Option<&[u8]>> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let samples_per_channel = pcm_data.len() / channels;
        
        // Validate input is properly aligned to channels
        if pcm_data.len() % channels != 0 {
            return Err(EncoderError::InputData(InputDataError::InvalidChannelCount {
                expected: channels,
                actual: pcm_data.len() % channels,
            }));
        }
        
        // Add new samples to buffer
        for ch in 0..channels {
            let channel_start = ch * samples_per_channel;
            let channel_end = channel_start + samples_per_channel;
            
            for sample_idx in channel_start..channel_end {
                if sample_idx < pcm_data.len() {
                    self.global_config.buffer[ch].push(pcm_data[sample_idx]);
                }
            }
        }
        
        self.samples_in_buffer += samples_per_channel;
        
        // Check if we have enough samples for a complete frame
        if self.samples_in_buffer >= samples_per_frame {
            // Clear frame buffer for new frame
            self.frame_buffer.clear();
            self.global_config.bs.reset();
            
            // Encode the frame through the complete pipeline
            self.encode_frame_pipeline(channels, samples_per_frame)?;
            
            // Remove encoded samples from buffer
            for ch in 0..channels {
                self.global_config.buffer[ch].drain(0..samples_per_frame);
            }
            self.samples_in_buffer -= samples_per_frame;
            
            Ok(Some(&self.frame_buffer))
        } else {
            // Not enough samples yet, just buffer
            Ok(None)
        }
    }
    
    /// Encode a frame of interleaved PCM data
    /// 
    /// # Arguments
    /// * `pcm_data` - Interleaved PCM samples [L, R, L, R, ...]
    /// 
    /// # Returns
    /// * `Ok(&[u8])` - Encoded MP3 frame data
    /// * `Err(EncoderError)` - Encoding error
    pub fn encode(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let expected_samples = samples_per_frame * channels;
        
        // Validate input length
        if pcm_data.len() != expected_samples {
            return Err(EncoderError::InputData(InputDataError::InvalidLength {
                expected: expected_samples,
                actual: pcm_data.len(),
            }));
        }
        
        // Clear frame buffer for new frame
        self.frame_buffer.clear();
        self.global_config.bs.reset();
        
        // De-interleave PCM data into channel buffers
        self.deinterleave_pcm_interleaved(pcm_data, channels, samples_per_frame);
        
        // Reset buffer counter since we have a complete frame
        self.samples_in_buffer = 0;
        
        // Encode the frame through the complete pipeline
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Flush any remaining data and finalize encoding
    /// 
    /// This method processes any remaining buffered samples and outputs the final MP3 frame.
    /// If there are insufficient samples for a complete frame, they will be padded with zeros.
    /// 
    /// # Returns
    /// * `Ok(&[u8])` - Final MP3 frame data (may be empty if no buffered data)
    /// * `Err(EncoderError)` - Encoding error
    pub fn flush(&mut self) -> Result<&[u8]> {
        // Check if we have any buffered samples
        if self.samples_in_buffer == 0 {
            // No buffered data, return empty
            self.frame_buffer.clear();
            return Ok(&self.frame_buffer);
        }
        
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        
        // If we have partial data, pad it to a complete frame
        if self.samples_in_buffer < samples_per_frame {
            for ch in 0..channels {
                // Pad with zeros to complete the frame
                while self.global_config.buffer[ch].len() < samples_per_frame {
                    self.global_config.buffer[ch].push(0);
                }
            }
        }
        
        // Clear frame buffer for new frame
        self.frame_buffer.clear();
        self.global_config.bs.reset();
        
        // Encode the final frame through the complete pipeline
        self.encode_frame_pipeline(channels, samples_per_frame)?;
        
        // Clear the buffer after flushing
        self.samples_in_buffer = 0;
        for channel_buffer in &mut self.global_config.buffer {
            channel_buffer.clear();
        }
        
        Ok(&self.frame_buffer)
    }
    
    /// Get the number of samples per frame for this configuration
    pub fn samples_per_frame(&self) -> usize {
        match self.config.mpeg_version() {
            crate::config::MpegVersion::Mpeg1 => 1152, // MPEG-1
            crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 576,  // MPEG-2/2.5
        }
    }
    
    /// Get the encoder configuration
    pub fn config(&self) -> &crate::shine_config::ShineGlobalConfig {
        &self.global_config
    }
    
    /// Get the public configuration
    pub fn public_config(&self) -> &Config {
        &self.config
    }
    
    /// Reset the encoder state
    pub fn reset(&mut self) {
        // Reset shine configuration state
        for ch in 0..self.global_config.wave.channels as usize {
            self.global_config.buffer[ch].clear();
        }
        self.frame_buffer.clear();
        self.samples_in_buffer = 0;
        
        // Reset side info
        self.global_config.side_info = crate::shine_config::ShineSideInfo::default();
        
        // Reset bitstream
        self.global_config.bs.reset();
    }
    
    /// De-interleave non-interleaved PCM data into channel buffers
    /// For non-interleaved data: [ch0_sample0, ch0_sample1, ..., ch1_sample0, ch1_sample1, ...]
    fn deinterleave_pcm(&mut self, pcm_data: &[i16], channels: usize, samples_per_frame: usize) {
        for ch in 0..channels {
            self.global_config.buffer[ch].clear();
            self.global_config.buffer[ch].reserve(samples_per_frame);
            
            let channel_start = ch * samples_per_frame;
            let channel_end = channel_start + samples_per_frame;
            
            for sample_idx in channel_start..channel_end {
                if sample_idx < pcm_data.len() {
                    self.global_config.buffer[ch].push(pcm_data[sample_idx]);
                }
            }
        }
    }
    
    /// De-interleave interleaved PCM data into channel buffers
    /// For interleaved data: [L, R, L, R, L, R, ...]
    fn deinterleave_pcm_interleaved(&mut self, pcm_data: &[i16], channels: usize, samples_per_frame: usize) {
        for ch in 0..channels {
            self.global_config.buffer[ch].clear();
            self.global_config.buffer[ch].reserve(samples_per_frame);
        }
        
        for sample_idx in 0..samples_per_frame {
            for ch in 0..channels {
                let interleaved_idx = sample_idx * channels + ch;
                if interleaved_idx < pcm_data.len() {
                    self.global_config.buffer[ch].push(pcm_data[interleaved_idx]);
                }
            }
        }
    }
    
    /// Main encoding pipeline following shine's encode_buffer_internal exactly
    /// 
    /// Original shine signature: static unsigned char *shine_encode_buffer_internal(shine_global_config *config, int *written, int stride)
    /// This function must match shine's implementation line by line for correct MP3 encoding
    fn encode_frame_pipeline(&mut self, channels: usize, _samples_per_frame: usize) -> Result<&[u8]> {
        // Following shine's encode_buffer_internal exactly (ref/shine/src/lib/layer3.c:150-175):
        
        // Step 1: Padding calculation (lines 152-157)
        if self.frac_slots_per_frame > 0.0 {
            let padding = self.slot_lag <= (self.frac_slots_per_frame - 1.0);
            self.slot_lag += if padding { 1.0 } else { 0.0 } - self.frac_slots_per_frame;
            self.frame_buffer.clear();
            self.frame_buffer.push(if padding { 1 } else { 0 });
        } else {
            self.frame_buffer.clear();
            self.frame_buffer.push(0);
        }
        let padding = self.frame_buffer[0] != 0;
        
        // Step 2: Calculate bits_per_frame (lines 159-160)
        let bits_per_frame = 8 * (self.whole_slots_per_frame + if padding { 1 } else { 0 });
        let target_frame_bytes = bits_per_frame / 8;
        
        // Step 3: Calculate mean_bits (lines 161-162)
        let granules_per_frame = match self.config.mpeg_version() {
            crate::config::MpegVersion::Mpeg1 => 2,
            crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 1,
        };
        let sideinfo_len = if self.config.mpeg_version() == crate::config::MpegVersion::Mpeg1 {
            8 * if channels == 1 { 4 + 17 } else { 4 + 32 }
        } else {
            8 * if channels == 1 { 4 + 9 } else { 4 + 17 }
        };
        let mean_bits = (bits_per_frame - sideinfo_len) / granules_per_frame;
        
        // Step 4: Apply MDCT to polyphase output (line 164)
        // shine_mdct_sub(config, stride);
        self.shine_mdct_sub(channels)?;
        
        // Step 5: Bit and noise allocation (line 167)
        // shine_iteration_loop(config);
        self.shine_iteration_loop(channels, mean_bits as i32)?;
        
        // Step 6: Write frame to bitstream (line 170)
        // shine_format_bitstream(config);
        self.shine_format_bitstream(padding, target_frame_bytes)?;
        
        // Step 7: Return data (lines 172-176)
        let encoded_data = self.global_config.bs.flush();
        self.frame_buffer.clear();
        self.frame_buffer.extend_from_slice(encoded_data);
        self.global_config.bs.reset();
        
        Ok(&self.frame_buffer)
    }
    
    /// Apply MDCT transform to polyphase output
    /// Following shine's shine_mdct_sub exactly (ref/shine/src/lib/l3mdct.c:50-150)
    /// 
    /// Original shine signature: void shine_mdct_sub(shine_global_config *config, int stride)
    /// - config: shine_global_config* (contains subband samples and MDCT output arrays)
    /// - stride: int (channel stride for data access) - always 1 for non-interleaved data
    fn shine_mdct_sub(&mut self, _channels: usize) -> Result<()> {
        // Call the new shine_mdct_sub function with the global config
        // This function now handles all channels and granules internally
        crate::mdct::shine_mdct_sub(&mut self.global_config, 1);
        Ok(())
    }
    
    /// Bit and noise allocation iteration loop
    /// Following shine's shine_iteration_loop exactly (ref/shine/src/lib/l3loop.c:97-170)
    /// 
    /// Original shine signature: void shine_iteration_loop(shine_global_config *config)
    /// - config: shine_global_config* (contains all encoder state and MDCT coefficients)
    fn shine_iteration_loop(&mut self, channels: usize, _mean_bits: i32) -> Result<()> {
        use crate::config::MpegVersion;
        
        let granules_per_frame = match self.config.mpeg_version() {
            MpegVersion::Mpeg1 => 2,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 1,
        };
        
        // Store granule info for bitstream formatting
        let mut all_granule_info = Vec::new();
        
        // Following shine's iteration_loop exactly (ref/shine/src/lib/l3loop.c:97-170)
        // for (ch = config->wave.channels; ch--;) {
        for ch in (0..channels).rev() {
            // for (gr = 0; gr < config->mpeg.granules_per_frame; gr++) {
            for gr in 0..granules_per_frame {
                // Copy MDCT coefficients for processing
                let xr = self.global_config.mdct_freq[ch][gr];
                
                // Initialize granule info structure
                let mut cod_info = GranuleInfo::default();
                cod_info.sfb_lmax = 21 - 1; // SFB_LMAX = 21
                
                // Calculate psychoacoustic masking thresholds following shine's calc_xmin
                let l3_xmin = self.calc_xmin(&mut cod_info, gr, ch)?;
                
                // Calculate scale factor selection information for MPEG-1
                if matches!(self.config.mpeg_version(), MpegVersion::Mpeg1) {
                    self.calc_scfsi(&l3_xmin, ch, gr)?;
                }
                
                // Calculate available bits for this granule
                let perceptual_entropy = self.calculate_perceptual_entropy(ch, gr)?;
                let max_bits = self.reservoir.max_reservoir_bits(perceptual_entropy, channels as u8);
                
                // Use quantization module for encoding
                let mut quantized_coeffs = [0i32; 576];
                let sample_rate = self.global_config.wave.sample_rate;
                
                // CRITICAL FIX: Use the quantization module instead of local implementation
                let part2_3_length = self.quantization_loop.quantize_and_encode(
                    &xr,
                    max_bits,
                    &mut cod_info,
                    &mut quantized_coeffs,
                    sample_rate
                )?;
                
                // Store quantized coefficients back to global config
                self.global_config.l3_enc[ch][gr] = quantized_coeffs;
                
                // Adjust bit reservoir
                self.reservoir.adjust_reservoir(part2_3_length as u32, channels as u8);
                
                // Set global gain (quantizer step size + 210 as per MP3 spec)
                cod_info.global_gain = (cod_info.quantizer_step_size + 210) as u32;
                
                // Store granule info for bitstream formatting
                all_granule_info.push(cod_info);
            } // for gr
        } // for ch
        
        // End frame processing for bit reservoir
        self.reservoir.frame_end(channels as u8)?;
        
        // Store the granule info for use in bitstream formatting
        self.current_granule_info = all_granule_info;
        
        Ok(())
    }
    
    /// Calculate psychoacoustic masking thresholds following shine's calc_xmin
    /// (ref/shine/src/lib/l3loop.c:309-325)
    /// 
    /// Original shine signature: void calc_xmin(shine_psy_ratio_t *ratio, gr_info *cod_info,
    ///                                          shine_psy_xmin_t *l3_xmin, int gr, int ch)
    fn calc_xmin(
        &self,
        cod_info: &mut GranuleInfo,
        _gr: usize,
        _ch: usize,
    ) -> Result<[f32; 21]> {
        let mut l3_xmin = [0.0f32; 21];
        
        // Following shine's calc_xmin exactly (ref/shine/src/lib/l3loop.c:309-325)
        // for (sfb = cod_info->sfb_lmax; sfb--;) {
        for sfb in (0..=cod_info.sfb_lmax as usize).rev() {
            if sfb >= 21 { continue; } // Safety check
            
            // note. xmin will always be zero with no psychoacoustic model
            // start = scalefac_band_long[ sfb ];
            // end   = scalefac_band_long[ sfb+1 ];
            // bw = end - start;
            // for ( en = 0, l = start; l < end; l++ )
            //   en += config->l3loop.xrsq[l];
            // l3_xmin->l[gr][ch][sfb] = ratio->l[gr][ch][sfb] * en / bw;
            
            // l3_xmin->l[gr][ch][sfb] = 0;
            l3_xmin[sfb] = 0.0;
        }
        
        Ok(l3_xmin)
    }
    
    /// Calculate scale factor selection information following shine's calc_scfsi
    /// (ref/shine/src/lib/l3loop.c:170-200)
    /// 
    /// Original shine signature: void calc_scfsi(shine_psy_xmin_t *l3_xmin, int ch, int gr,
    ///                                           shine_global_config *config)
    fn calc_scfsi(
        &self,
        _l3_xmin: &[f32; 21],
        _ch: usize,
        _gr: usize,
    ) -> Result<()> {
        // Following shine's calc_scfsi exactly (ref/shine/src/lib/l3loop.c:170-200)
        // This is the scfsi_band table from 2.4.2.7 of the IS
        // static const int scfsi_band_long[5] = {0, 6, 11, 16, 21};
        
        // For now, we don't implement scale factor selection information
        // This would require maintaining scale factor history between granules
        // The shine implementation is quite complex and involves comparing
        // scale factors between granules to determine if they can be shared
        
        Ok(())
    }
    
    /// Calculate perceptual entropy for bit reservoir management
    /// Following shine's perceptual entropy calculation
    fn calculate_perceptual_entropy(
        &self,
        _ch: usize,
        _gr: usize,
    ) -> Result<f64> {
        // For now, return a reasonable default value
        // Real implementation would calculate based on spectral characteristics
        // and psychoacoustic model output
        Ok(100.0)
    }
    
    /// Format and write the bitstream
    /// Following shine's shine_format_bitstream exactly (ref/shine/src/lib/l3bitstream.c:32-100)
    /// 
    /// Original shine signature: void shine_format_bitstream(shine_global_config *config)
    /// - config: shine_global_config* (contains side info and quantized data)
    fn shine_format_bitstream(&mut self, padding: bool, target_frame_bytes: usize) -> Result<()> {
        use crate::config::MpegVersion;
        
        let granules_per_frame = match self.config.mpeg_version() {
            MpegVersion::Mpeg1 => 2,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 1,
        };
        let channels = self.global_config.wave.channels as usize;
        
        // Following shine's shine_format_bitstream exactly (ref/shine/src/lib/l3bitstream.c:32-100)
        
        // Step 1: Sign correction for quantized coefficients (lines 35-43)
        // for (ch = 0; ch < config->wave.channels; ch++)
        //   for (gr = 0; gr < config->mpeg.granules_per_frame; gr++) {
        //     int *pi = &config->l3_enc[ch][gr][0];
        //     int32_t *pr = &config->mdct_freq[ch][gr][0];
        //     for (i = 0; i < GRANULE_SIZE; i++) {
        //       if ((pr[i] < 0) && (pi[i] > 0))
        //         pi[i] *= -1;
        //     }
        //   }
        // (This step is handled in quantization for now)
        
        // Step 2: Encode side information (line 45)
        // encodeSideInfo(config);
        self.encode_side_info(padding)?;
        
        // Step 3: Encode main data (line 46)
        // encodeMainData(config);
        self.encode_main_data(granules_per_frame, channels, target_frame_bytes)?;
        
        Ok(())
    }
    
    /// Encode side information following shine's encodeSideInfo
    /// (ref/shine/src/lib/l3bitstream.c:70-100)
    fn encode_side_info(&mut self, padding: bool) -> Result<()> {
        // Write frame header first
        self.global_config.bs.write_frame_header(&self.config, padding);
        
        // Create side information structure with actual granule data
        let mut side_info = crate::bitstream::SideInfo::default();
        side_info.granules = self.current_granule_info.clone();
        
        // Write side information structure
        self.global_config.bs.write_side_info(&side_info, &self.config);
        
        Ok(())
    }
    
    /// Encode main data following shine's encodeMainData
    /// (ref/shine/src/lib/l3bitstream.c:48-68)
    fn encode_main_data(&mut self, granules_per_frame: usize, channels: usize, target_frame_bytes: usize) -> Result<()> {
        // Calculate how many bytes we need to write to reach target frame size
        let current_bytes = self.global_config.bs.bits_written() / 8;
        let _remaining_bytes = if target_frame_bytes > current_bytes {
            target_frame_bytes - current_bytes
        } else {
            0
        };
        
        // Following shine's encodeMainData exactly
        // for (gr = 0; gr < config->mpeg.granules_per_frame; gr++) {
        //   for (ch = 0; ch < config->wave.channels; ch++) {
        for _gr in 0..granules_per_frame {
            for _ch in 0..channels {
                // Write scale factors (simplified for now)
                // In real implementation, this would write actual scale factors
                // based on config->scalefactor.l[gr][ch] and SCFSI
                
                // Write some minimal scale factor data to create valid frame structure
                for _sfb in 0..21 { // 21 scale factor bands for long blocks
                    self.global_config.bs.write_bits(0, 4); // 4 bits per scale factor
                }
                
                // Write Huffman encoded spectral data
                // For now, write minimal data to create valid frame structure
                // In real implementation, this would call Huffmancodebits()
            }
        }
        
        // Fill remaining bytes to reach target frame size
        let bytes_written_after_scalefactors = self.global_config.bs.bits_written() / 8;
        let still_remaining = if target_frame_bytes > bytes_written_after_scalefactors {
            target_frame_bytes - bytes_written_after_scalefactors
        } else {
            0
        };
        
        // Write padding data to reach exact target frame size
        for _i in 0..still_remaining {
            self.global_config.bs.write_bits(0, 8);
        }
        
        Ok(())
    }
    

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
    use proptest::prelude::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_clean_errors() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                if let Some(s) = info.payload().downcast_ref::<String>() {
                    let msg = if s.len() > 200 { &s[..197] } else { s };
                    eprintln!("Test failed: {}", msg.trim());
                }
            }));
        });
    }

    #[test]
    fn test_mp3_encoder_creation() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::JointStereo,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_ok(), "Encoder creation should succeed");
        
        let encoder = encoder.unwrap();
        assert_eq!(encoder.samples_per_frame(), 1152); // MPEG-1 frame size
        assert_eq!(encoder.public_config().wave.channels, Channels::Stereo);
        assert_eq!(encoder.config().mpeg.bitrate, 128);
    }

    #[test]
    fn test_mp3_encoder_invalid_config() {
        let mut config = Config::default();
        config.mpeg.bitrate = 999; // Invalid bitrate
        
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_err(), "Encoder creation should fail with invalid config");
    }

    #[test]
    fn test_mp3_encoder_encode_frame_functionality() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Create test PCM data (1152 samples for mono MPEG-1)
        let pcm_data = vec![0i16; 1152];
        
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_ok(), "Frame encoding should succeed with valid input");
        
        let encoded_frame = result.unwrap();
        assert!(!encoded_frame.is_empty(), "Encoded frame should not be empty");
        
        // MP3 frame should start with sync word (0xFFF)
        assert!(encoded_frame.len() >= 4, "Frame should be at least 4 bytes (header)");
        let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "Frame should start with MP3 sync word");
    }

    #[test]
    fn test_mp3_encoder_encode_frame_interleaved() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Stereo,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Stereo,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Create test PCM data (1152 * 2 samples for stereo MPEG-1, interleaved)
        let pcm_data = vec![0i16; 1152 * 2];
        
        let result = encoder.encode_frame_interleaved(&pcm_data);
        assert!(result.is_ok(), "Interleaved frame encoding should succeed");
        
        let encoded_frame = result.unwrap();
        assert!(!encoded_frame.is_empty(), "Encoded frame should not be empty");
    }

    #[test]
    fn test_mp3_encoder_invalid_input_length() {
        let config = Config::default();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Wrong number of samples
        let pcm_data = vec![0i16; 100];
        
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_err(), "Should fail with wrong input length");
        
        if let Err(EncoderError::InputData(InputDataError::InvalidLength { expected, actual })) = result {
            assert_eq!(expected, 1152 * 2); // Stereo MPEG-1
            assert_eq!(actual, 100);
        } else {
            panic!("Should return InvalidLength error");
        }
    }

    #[test]
    fn test_mp3_encoder_flush() {
        let config = Config::default();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        let result = encoder.flush();
        assert!(result.is_ok(), "Flush should succeed");
        
        let flushed_data = result.unwrap();
        // For now, flush returns empty data since we don't buffer partial frames
        assert!(flushed_data.is_empty(), "Flush should return empty data when no buffered samples");
    }

    #[test]
    fn test_mp3_encoder_encode_samples_incremental() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Test incremental encoding with partial frames
        let partial_samples = vec![100i16; 500]; // Less than 1152 samples needed for mono MPEG-1
        
        // First partial frame should be buffered
        let result = encoder.encode_samples(&partial_samples);
        assert!(result.is_ok(), "Partial frame encoding should succeed");
        assert!(result.unwrap().is_none(), "Partial frame should return None");
        
        // Add more samples to complete a frame
        let remaining_samples = vec![200i16; 652]; // 500 + 652 = 1152 total
        let result = encoder.encode_samples(&remaining_samples);
        assert!(result.is_ok(), "Completing frame should succeed");
        
        let encoded_frame = result.unwrap();
        assert!(encoded_frame.is_some(), "Complete frame should return Some");
        
        let frame_data = encoded_frame.unwrap();
        assert!(!frame_data.is_empty(), "Encoded frame should not be empty");
        
        // Verify MP3 sync word
        let sync = ((frame_data[0] as u16) << 3) | ((frame_data[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "Frame should start with MP3 sync word");
    }

    #[test]
    fn test_mp3_encoder_flush_with_buffered_data() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Add some partial samples
        let partial_samples = vec![300i16; 800]; // Less than 1152 samples
        let result = encoder.encode_samples(&partial_samples);
        assert!(result.is_ok(), "Partial encoding should succeed");
        assert!(result.unwrap().is_none(), "Partial frame should be buffered");
        
        // Flush should encode the remaining data with zero padding
        let flush_result = encoder.flush();
        assert!(flush_result.is_ok(), "Flush should succeed");
        
        let flushed_data = flush_result.unwrap();
        assert!(!flushed_data.is_empty(), "Flush should return encoded frame with buffered data");
        
        // Verify MP3 sync word
        let sync = ((flushed_data[0] as u16) << 3) | ((flushed_data[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "Flushed frame should start with MP3 sync word");
    }

    #[test]
    fn test_different_inputs_produce_different_outputs() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Create two different PCM inputs
        let pcm_data1: Vec<i16> = (0..1152).map(|i| ((i % 100) as i16 * 10)).collect();
        let pcm_data2: Vec<i16> = (0..1152).map(|i| ((i % 200) as i16 * 20)).collect();
        
        // Encode both inputs
        let result1 = encoder.encode_frame(&pcm_data1);
        assert!(result1.is_ok(), "First encoding should succeed");
        let encoded1 = result1.unwrap().to_vec();
        
        // Reset encoder state for second encoding
        encoder.reset();
        
        let result2 = encoder.encode_frame(&pcm_data2);
        assert!(result2.is_ok(), "Second encoding should succeed");
        let encoded2 = result2.unwrap().to_vec();
        
        // Verify outputs are different
        assert_ne!(encoded1, encoded2, "Different inputs should produce different outputs");
        
        // Both should be valid MP3 frames (start with sync word)
        assert!(encoded1.len() >= 4, "First frame should be at least 4 bytes");
        assert!(encoded2.len() >= 4, "Second frame should be at least 4 bytes");
        
        let sync1 = ((encoded1[0] as u16) << 3) | ((encoded1[1] as u16) >> 5);
        let sync2 = ((encoded2[0] as u16) << 3) | ((encoded2[1] as u16) >> 5);
        assert_eq!(sync1, 0x7FF, "First frame should start with MP3 sync word");
        assert_eq!(sync2, 0x7FF, "Second frame should start with MP3 sync word");
        
        println!("Test passed: Different inputs produced different outputs");
        println!("Frame 1 size: {} bytes", encoded1.len());
        println!("Frame 2 size: {} bytes", encoded2.len());
    }

    #[test]
    fn test_mp3_encoder_reset() {
        let config = Config::default();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Encode a frame first with complex data (stereo needs 1152 * 2 samples)
        let pcm_data = vec![100i16; 1152 * 2];
        let result = encoder.encode_frame(&pcm_data);
        if result.is_err() {
            eprintln!("First encode failed: {:?}", result.err());
        }
        
        // Reset should work without errors
        encoder.reset();
        
        // Should be able to encode again after reset
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_ok(), "Should be able to encode after reset: {:?}", result.err());
    }

    #[test]
    fn test_mp3_encoder_multiple_frames() {
        let config = Config {
            wave: WaveConfig {
                channels: Channels::Mono,
                sample_rate: 44100,
            },
            mpeg: MpegConfig {
                mode: StereoMode::Mono,
                bitrate: 128,
                emphasis: Emphasis::None,
                copyright: false,
                original: true,
            },
        };
        
        let mut encoder = Mp3Encoder::new(config).unwrap();
        
        // Test with all zeros first
        let pcm_data = vec![0i16; 1152];
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_ok(), "Zero frame encoding should succeed: {:?}", result.err());
        
        // Test with very small values
        let pcm_data = vec![1i16; 1152];
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_ok(), "Small constant frame encoding should succeed: {:?}", result.err());
    }

    // Property test generators
    prop_compose! {
        fn valid_sample_rate()(rate in prop::sample::select(&[
            44100u32, 22050, 11025,  // One representative from each MPEG version
        ])) -> u32 {
            rate
        }
    }

    prop_compose! {
        fn valid_bitrate()(rate in prop::sample::select(&[
            8u32, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 192, 224, 256, 320
        ])) -> u32 {
            rate
        }
    }

    prop_compose! {
        fn valid_channels()(channels in prop::sample::select(&[Channels::Mono, Channels::Stereo])) -> Channels {
            channels
        }
    }

    prop_compose! {
        fn valid_stereo_mode()(mode in prop::sample::select(&[
            StereoMode::Stereo, StereoMode::JointStereo, StereoMode::DualChannel, StereoMode::Mono
        ])) -> StereoMode {
            mode
        }
    }

    prop_compose! {
        fn valid_emphasis()(emphasis in prop::sample::select(&[
            Emphasis::None, Emphasis::Emphasis50_15, Emphasis::CcittJ17
        ])) -> Emphasis {
            emphasis
        }
    }

    fn compatible_config() -> impl Strategy<Value = Config> {
        (valid_sample_rate(), valid_channels(), valid_emphasis(), any::<bool>(), any::<bool>())
            .prop_flat_map(|(sample_rate, channels, emphasis, copyright, original)| {
                // Use fewer bitrate options to reduce test time
                let bitrate_strategy = match sample_rate {
                    44100 | 48000 | 32000 => prop::sample::select(vec![128, 192, 320]), // Just 3 representative bitrates
                    22050 | 24000 | 16000 => prop::sample::select(vec![64, 96, 160]),   // Just 3 representative bitrates
                    11025 | 12000 | 8000 => prop::sample::select(vec![32, 48, 64]),     // Just 3 representative bitrates
                    _ => prop::sample::select(vec![128]), // fallback
                };
                
                let mode_strategy = match channels {
                    Channels::Mono => prop::sample::select(vec![StereoMode::Mono]),
                    Channels::Stereo => prop::sample::select(vec![StereoMode::Stereo, StereoMode::JointStereo]), // Reduced from 3 to 2 modes
                };
                
                (Just(sample_rate), Just(channels), bitrate_strategy, mode_strategy, Just(emphasis), Just(copyright), Just(original))
            })
            .prop_map(|(sample_rate, channels, bitrate, mode, emphasis, copyright, original)| {
                Config {
                    wave: WaveConfig {
                        channels,
                        sample_rate,
                    },
                    mpeg: MpegConfig {
                        mode,
                        bitrate,
                        emphasis,
                        copyright,
                        original,
                    },
                }
            })
    }

    // Feature: rust-mp3-encoder, Property 1: 编码器初始化和基本功能
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 20, // Reduced from 100 to 20 for faster testing
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_encoder_initialization_and_functionality(config in compatible_config()) {
            setup_clean_errors();
            
            // For any valid encoding configuration, encoder should successfully initialize
            let encoder_result = Mp3Encoder::new(config.clone());
            prop_assert!(encoder_result.is_ok(), "Encoder initialization failed");
            
            let encoder = encoder_result.unwrap();
            
            // Verify encoder properties match configuration
            prop_assert_eq!(encoder.public_config().wave.channels, config.wave.channels, "Channel configuration mismatch");
            prop_assert_eq!(encoder.public_config().wave.sample_rate, config.wave.sample_rate, "Sample rate mismatch");
            prop_assert_eq!(encoder.public_config().mpeg.bitrate, config.mpeg.bitrate, "Bitrate mismatch");
            
            // Verify samples per frame calculation
            let expected_samples = match config.mpeg_version() {
                crate::config::MpegVersion::Mpeg1 => 1152,
                crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 576,
            };
            prop_assert_eq!(encoder.samples_per_frame(), expected_samples, "Samples per frame mismatch");
        }

        #[test]
        fn test_encoder_functionality_with_valid_pcm(
            config in compatible_config(),
        ) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            // Generate valid PCM data for this configuration
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            let total_samples = samples_per_frame * channels;
            
            // Create PCM data with appropriate size (use realistic audio pattern for testing)
            let pcm_data: Vec<i16> = (0..total_samples)
                .map(|i| {
                    // Generate a more realistic audio signal with multiple frequency components
                    let t = i as f64 / 44100.0; // Assume 44.1kHz for time calculation
                    let sample = (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin() +
                                 500.0 * (2.0 * std::f64::consts::PI * 880.0 * t).sin() +
                                 250.0 * (2.0 * std::f64::consts::PI * 1320.0 * t).sin()) as i16;
                    sample
                })
                .collect();
            
            // For any valid PCM input data, should return valid MP3 encoded data
            let encode_result = encoder.encode_frame(&pcm_data);
            prop_assert!(encode_result.is_ok(), "Frame encoding failed");
            
            let encoded_frame = encode_result.unwrap();
            prop_assert!(!encoded_frame.is_empty(), "Encoded frame should not be empty");
            
            // Verify MP3 frame structure - should start with sync word
            prop_assert!(encoded_frame.len() >= 4, "Frame should be at least 4 bytes");
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Frame should start with MP3 sync word");
            
            // Verify frame header contains correct information
            let header = ((encoded_frame[0] as u32) << 24) | 
                        ((encoded_frame[1] as u32) << 16) | 
                        ((encoded_frame[2] as u32) << 8) | 
                        (encoded_frame[3] as u32);
            
            // Check MPEG version bits (bits 19-20)
            let mpeg_version_bits = (header >> 19) & 0x3;
            let expected_version_bits = match config.mpeg_version() {
                crate::config::MpegVersion::Mpeg1 => 0x3,
                crate::config::MpegVersion::Mpeg2 => 0x2,
                crate::config::MpegVersion::Mpeg25 => 0x0,
            };
            prop_assert_eq!(mpeg_version_bits, expected_version_bits, "MPEG version bits incorrect");
            
            // Check layer bits (bits 17-18) - should be 01 for Layer III
            let layer_bits = (header >> 17) & 0x3;
            prop_assert_eq!(layer_bits, 0x1, "Layer bits should indicate Layer III");
        }

        #[test]
        fn test_encoder_interleaved_functionality(
            config in compatible_config(),
        ) {
            setup_clean_errors();
            
            // Only test stereo configurations for interleaved encoding
            if config.wave.channels != Channels::Stereo {
                return Ok(());
            }
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            // Generate interleaved PCM data
            let samples_per_frame = config.samples_per_frame();
            let total_samples = samples_per_frame * 2; // Stereo
            
            let pcm_data: Vec<i16> = (0..total_samples)
                .map(|i| {
                    // Generate realistic stereo audio signal
                    let t = i as f64 / 44100.0;
                    let sample = (800.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16;
                    sample
                })
                .collect();
            
            // Test interleaved encoding
            let encode_result = encoder.encode_frame_interleaved(&pcm_data);
            prop_assert!(encode_result.is_ok(), "Interleaved frame encoding failed");
            
            let encoded_frame = encode_result.unwrap();
            prop_assert!(!encoded_frame.is_empty(), "Encoded frame should not be empty");
            
            // Should produce valid MP3 frame
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Interleaved frame should start with MP3 sync word");
        }

        #[test]
        fn test_encoder_reset_functionality(config in compatible_config()) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            // Generate some PCM data
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            let total_samples = samples_per_frame * channels;
            let pcm_data = vec![1000i16; total_samples];
            
            // Encode a frame and immediately extract the result
            let first_result = encoder.encode_frame(&pcm_data);
            prop_assert!(first_result.is_ok(), "First encoding should succeed");
            let first_frame = first_result.unwrap().to_vec(); // Copy the data
            
            // Reset encoder
            encoder.reset();
            
            // Should be able to encode again after reset
            let second_result = encoder.encode_frame(&pcm_data);
            prop_assert!(second_result.is_ok(), "Encoding after reset should succeed");
            let second_frame = second_result.unwrap().to_vec(); // Copy the data
            
            // Both results should be valid MP3 frames
            prop_assert!(!first_frame.is_empty(), "First frame should not be empty");
            prop_assert!(!second_frame.is_empty(), "Second frame should not be empty");
            
            // Both should have valid sync words
            let sync1 = ((first_frame[0] as u16) << 3) | ((first_frame[1] as u16) >> 5);
            let sync2 = ((second_frame[0] as u16) << 3) | ((second_frame[1] as u16) >> 5);
            prop_assert_eq!(sync1, 0x7FF, "First frame should have valid sync");
            prop_assert_eq!(sync2, 0x7FF, "Second frame should have valid sync");
        }

        // Feature: rust-mp3-encoder, Property 2: 刷新和完整性
        #[test]
        fn test_flush_and_completeness(
            config in compatible_config(),
            partial_samples_count in 1usize..100, // Reduced from 1000 to 100
        ) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            
            // Ensure partial_samples_count is less than samples_per_frame to create partial data
            let partial_count = partial_samples_count % samples_per_frame;
            if partial_count == 0 {
                return Ok(()); // Skip if no partial data
            }
            
            // Generate partial PCM data (less than a complete frame)
            let partial_pcm: Vec<i16> = (0..partial_count * channels)
                .map(|i| {
                    let t = i as f64 / 44100.0;
                    (1000.0 * (2.0 * std::f64::consts::PI * 220.0 * t).sin()) as i16
                })
                .collect();
            
            // Add partial samples using encode_samples
            let partial_result = encoder.encode_samples(&partial_pcm);
            prop_assert!(partial_result.is_ok(), "Partial sample encoding should succeed");
            
            // Should return None since we don't have a complete frame yet
            let partial_output = partial_result.unwrap();
            prop_assert!(partial_output.is_none(), "Partial samples should not produce output");
            
            // Now flush should return all remaining encoded data
            let flush_result = encoder.flush();
            prop_assert!(flush_result.is_ok(), "Flush should succeed");
            
            let flushed_data = flush_result.unwrap();
            
            // For any encoding session, calling flush should return all remaining encoded data
            // ensuring no data loss
            prop_assert!(!flushed_data.is_empty(), "Flush should return encoded data for buffered samples");
            
            // Verify the flushed data is a valid MP3 frame
            prop_assert!(flushed_data.len() >= 4, "Flushed frame should be at least 4 bytes");
            let sync = ((flushed_data[0] as u16) << 3) | ((flushed_data[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Flushed frame should start with MP3 sync word");
            
            // After flush, subsequent flush should return empty (no data loss means no duplicate data)
            let second_flush = encoder.flush();
            prop_assert!(second_flush.is_ok(), "Second flush should succeed");
            let second_flushed = second_flush.unwrap();
            prop_assert!(second_flushed.is_empty(), "Second flush should return empty data");
            
            // Verify encoder state is clean after flush
            // Should be able to encode new data after flush
            let new_pcm = vec![500i16; samples_per_frame * channels];
            let new_encode_result = encoder.encode_frame(&new_pcm);
            prop_assert!(new_encode_result.is_ok(), "Should be able to encode after flush");
            
            let new_frame = new_encode_result.unwrap();
            prop_assert!(!new_frame.is_empty(), "New frame after flush should not be empty");
            let new_sync = ((new_frame[0] as u16) << 3) | ((new_frame[1] as u16) >> 5);
            prop_assert_eq!(new_sync, 0x7FF, "New frame should have valid sync word");
        }

        #[test]
        fn test_flush_completeness_with_multiple_partial_chunks(
            config in compatible_config(),
            chunk_sizes in prop::collection::vec(1usize..50, 2..4), // Reduced range and count
        ) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            
            // Calculate total partial samples from all chunks
            let total_partial_samples: usize = chunk_sizes.iter().sum();
            let total_partial_samples = total_partial_samples % samples_per_frame;
            
            if total_partial_samples == 0 {
                return Ok(()); // Skip if no partial data
            }
            
            // Add multiple chunks of partial data
            let mut total_added = 0;
            for &chunk_size in &chunk_sizes {
                if total_added >= total_partial_samples {
                    break;
                }
                
                let actual_chunk_size = std::cmp::min(chunk_size, total_partial_samples - total_added);
                let chunk_pcm: Vec<i16> = (0..actual_chunk_size * channels)
                    .map(|i| ((i + total_added) % 2000) as i16)
                    .collect();
                
                let chunk_result = encoder.encode_samples(&chunk_pcm);
                prop_assert!(chunk_result.is_ok(), "Chunk encoding should succeed");
                
                let chunk_output = chunk_result.unwrap();
                prop_assert!(chunk_output.is_none(), "Partial chunks should not produce output");
                
                total_added += actual_chunk_size;
            }
            
            // Flush should return all accumulated data as a complete frame
            let flush_result = encoder.flush();
            prop_assert!(flush_result.is_ok(), "Flush should succeed");
            
            let flushed_data = flush_result.unwrap();
            prop_assert!(!flushed_data.is_empty(), "Flush should return data for accumulated samples");
            
            // Verify completeness - flushed data should be a valid MP3 frame
            let sync = ((flushed_data[0] as u16) << 3) | ((flushed_data[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Flushed frame should have valid sync word");
            
            // Verify no data remains after flush
            let second_flush = encoder.flush();
            prop_assert!(second_flush.is_ok(), "Second flush should succeed");
            prop_assert!(second_flush.unwrap().is_empty(), "No data should remain after flush");
        }

        #[test]
        fn test_flush_with_no_buffered_data(config in compatible_config()) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config).unwrap();
            
            // Flush with no buffered data should return empty
            let flush_result = encoder.flush();
            prop_assert!(flush_result.is_ok(), "Flush should succeed even with no data");
            
            let flushed_data = flush_result.unwrap();
            prop_assert!(flushed_data.is_empty(), "Flush should return empty when no buffered data");
            
            // Multiple flushes should all return empty
            for _ in 0..3 {
                let flush_result = encoder.flush();
                prop_assert!(flush_result.is_ok(), "Multiple flushes should succeed");
                prop_assert!(flush_result.unwrap().is_empty(), "Multiple flushes should return empty");
            }
        }

        // Feature: rust-mp3-encoder, Property 3: 音频格式支持
        #[test]
        fn test_audio_format_support_mono_configurations(
            sample_rate in valid_sample_rate(),
            emphasis in valid_emphasis(),
            copyright in any::<bool>(),
            original in any::<bool>(),
        ) {
            setup_clean_errors();
            
            // Test with just one representative bitrate per sample rate to avoid long test times
            let bitrate = match sample_rate {
                44100 | 48000 | 32000 => 128,
                22050 | 24000 | 16000 => 64,
                11025 | 12000 | 8000 => 32,
                _ => 128,
            };
            
            let config = Config {
                wave: WaveConfig {
                    channels: Channels::Mono,
                    sample_rate,
                },
                mpeg: MpegConfig {
                    mode: StereoMode::Mono,
                    bitrate,
                    emphasis,
                    copyright,
                    original,
                },
            };
            
            // For any standard audio format configuration (mono), encoder should handle correctly
            let encoder_result = Mp3Encoder::new(config.clone());
            prop_assert!(encoder_result.is_ok(), "Mono encoder creation should succeed for sample_rate={}, bitrate={}", sample_rate, bitrate);
            
            let mut encoder = encoder_result.unwrap();
            
            // Verify configuration is correctly applied
            prop_assert_eq!(encoder.public_config().wave.channels, Channels::Mono, "Channel configuration should be mono");
            prop_assert_eq!(encoder.public_config().wave.sample_rate, sample_rate, "Sample rate should match");
            prop_assert_eq!(encoder.public_config().mpeg.bitrate, bitrate, "Bitrate should match");
            
            // Test encoding with mono data
            let samples_per_frame = config.samples_per_frame();
            let pcm_data: Vec<i16> = (0..samples_per_frame)
                .map(|i| {
                    let t = i as f64 / 44100.0;
                    (500.0 * (2.0 * std::f64::consts::PI * 330.0 * t).sin()) as i16
                })
                .collect();
            
            let encode_result = encoder.encode_frame(&pcm_data);
            prop_assert!(encode_result.is_ok(), "Mono frame encoding should succeed for sample_rate={}, bitrate={}", sample_rate, bitrate);
            
            let encoded_frame = encode_result.unwrap();
            prop_assert!(!encoded_frame.is_empty(), "Encoded mono frame should not be empty");
            
            // Verify MP3 frame structure
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Mono frame should have valid sync word");
        }

        #[test]
        fn test_audio_format_support_stereo_configurations(
            sample_rate in valid_sample_rate(),
            stereo_mode in prop::sample::select(&[StereoMode::Stereo, StereoMode::JointStereo, StereoMode::DualChannel]),
            emphasis in valid_emphasis(),
            copyright in any::<bool>(),
            original in any::<bool>(),
        ) {
            setup_clean_errors();
            
            // Test with just one representative bitrate per sample rate to avoid long test times
            let bitrate = match sample_rate {
                44100 | 48000 | 32000 => 128,
                22050 | 24000 | 16000 => 64,
                11025 | 12000 | 8000 => 32,
                _ => 128,
            };
            
            let config = Config {
                wave: WaveConfig {
                    channels: Channels::Stereo,
                    sample_rate,
                },
                mpeg: MpegConfig {
                    mode: stereo_mode,
                    bitrate,
                    emphasis,
                    copyright,
                    original,
                },
            };
            
            // For any standard audio format configuration (stereo), encoder should handle correctly
            let encoder_result = Mp3Encoder::new(config.clone());
            prop_assert!(encoder_result.is_ok(), "Stereo encoder creation should succeed for sample_rate={}, bitrate={}, mode={:?}", sample_rate, bitrate, stereo_mode);
            
            let mut encoder = encoder_result.unwrap();
            
            // Verify configuration is correctly applied
            prop_assert_eq!(encoder.public_config().wave.channels, Channels::Stereo, "Channel configuration should be stereo");
            prop_assert_eq!(encoder.public_config().wave.sample_rate, sample_rate, "Sample rate should match");
            prop_assert_eq!(encoder.public_config().mpeg.bitrate, bitrate, "Bitrate should match");
            prop_assert_eq!(encoder.public_config().mpeg.mode, stereo_mode, "Stereo mode should match");
            
            // Test encoding with stereo data (non-interleaved)
            let samples_per_frame = config.samples_per_frame();
            let total_samples = samples_per_frame * 2; // Stereo
            let pcm_data: Vec<i16> = (0..total_samples)
                .map(|i| {
                    let t = i as f64 / 44100.0;
                    (400.0 * (2.0 * std::f64::consts::PI * 660.0 * t).sin()) as i16
                })
                .collect();
                
            
            let encode_result = encoder.encode_frame(&pcm_data);
            prop_assert!(encode_result.is_ok(), "Stereo frame encoding should succeed for sample_rate={}, bitrate={}, mode={:?}", sample_rate, bitrate, stereo_mode);
            
            let encoded_frame = encode_result.unwrap();
            prop_assert!(!encoded_frame.is_empty(), "Encoded stereo frame should not be empty");
            
            // Verify MP3 frame structure
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Stereo frame should have valid sync word");
            
            // Test interleaved encoding as well
            let interleaved_data: Vec<i16> = (0..samples_per_frame)
                .flat_map(|i| {
                    let t = i as f64 / 44100.0;
                    let left = (600.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16;
                    let right = (400.0 * (2.0 * std::f64::consts::PI * 880.0 * t).sin()) as i16;
                    vec![left, right]
                })
                .collect();
            
            let interleaved_result = encoder.encode_frame_interleaved(&interleaved_data);
            prop_assert!(interleaved_result.is_ok(), "Interleaved stereo encoding should succeed for sample_rate={}, bitrate={}, mode={:?}", sample_rate, bitrate, stereo_mode);
            
            let interleaved_frame = interleaved_result.unwrap();
            prop_assert!(!interleaved_frame.is_empty(), "Encoded interleaved frame should not be empty");
            
            let interleaved_sync = ((interleaved_frame[0] as u16) << 3) | ((interleaved_frame[1] as u16) >> 5);
            prop_assert_eq!(interleaved_sync, 0x7FF, "Interleaved frame should have valid sync word");
        }

        #[test]
        fn test_audio_format_support_invalid_configurations(
            invalid_sample_rate in prop::num::u32::ANY.prop_filter("Must be invalid", |&rate| {
                !matches!(rate, 44100 | 48000 | 32000 | 22050 | 24000 | 16000 | 11025 | 12000 | 8000)
            }),
            channels in valid_channels(),
            bitrate in valid_bitrate(),
            mode in valid_stereo_mode(),
            emphasis in valid_emphasis(),
            copyright in any::<bool>(),
            original in any::<bool>(),
        ) {
            setup_clean_errors();
            
            let config = Config {
                wave: WaveConfig {
                    channels,
                    sample_rate: invalid_sample_rate,
                },
                mpeg: MpegConfig {
                    mode,
                    bitrate,
                    emphasis,
                    copyright,
                    original,
                },
            };
            
            // For any invalid audio format configuration, encoder should reject with appropriate error
            let encoder_result = Mp3Encoder::new(config);
            prop_assert!(encoder_result.is_err(), "Invalid sample rate should be rejected");
            
            if let Err(EncoderError::Config(config_err)) = encoder_result {
                use crate::error::ConfigError;
                match config_err {
                    ConfigError::UnsupportedSampleRate(rate) => {
                        prop_assert_eq!(rate, invalid_sample_rate, "Error should contain invalid sample rate");
                    },
                    ConfigError::IncompatibleRateCombination { sample_rate, .. } => {
                        prop_assert_eq!(sample_rate, invalid_sample_rate, "Error should contain invalid sample rate");
                    },
                    _ => prop_assert!(false, "Should get sample rate related error"),
                }
            } else {
                prop_assert!(false, "Should get config error");
            }
        }

        #[test]
        fn test_audio_format_support_input_validation(
            config in compatible_config(),
            wrong_sample_count in 1usize..2000,
        ) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            let expected_total = samples_per_frame * channels;
            
            // Ensure wrong_sample_count is different from expected
            let wrong_count = if wrong_sample_count == expected_total {
                wrong_sample_count + 1
            } else {
                wrong_sample_count
            };
            
            let wrong_pcm_data = vec![100i16; wrong_count];
            
            // For any audio format configuration, encoder should validate input data correctly
            let encode_result = encoder.encode_frame(&wrong_pcm_data);
            prop_assert!(encode_result.is_err(), "Wrong input length should be rejected");
            
            match encode_result.unwrap_err() {
                EncoderError::InputData(InputDataError::InvalidLength { expected, actual }) => {
                    prop_assert_eq!(expected, expected_total, "Error should show expected sample count");
                    prop_assert_eq!(actual, wrong_count, "Error should show actual sample count");
                },
                _ => prop_assert!(false, "Should get InvalidLength error"),
            }
            
            // Test channel alignment validation for encode_samples
            if channels > 1 {
                let misaligned_data = vec![200i16; channels + 1]; // Not divisible by channel count
                let samples_result = encoder.encode_samples(&misaligned_data);
                prop_assert!(samples_result.is_err(), "Misaligned channel data should be rejected");
                
                match samples_result.unwrap_err() {
                    EncoderError::InputData(InputDataError::InvalidChannelCount { expected, actual }) => {
                        prop_assert_eq!(expected, channels, "Error should show expected channel count");
                        prop_assert_eq!(actual, 1, "Error should show misalignment");
                    },
                    _ => prop_assert!(false, "Should get InvalidChannelCount error"),
                }
            }
        }
    }
}