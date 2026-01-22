//! Subband filtering for MP3 encoding
//!
//! This module implements the polyphase subband filter that decomposes
//! PCM audio into 32 frequency subbands for further processing.
//! 
//! The implementation follows the shine library's approach, using a 
//! polyphase filterbank with 512-point history buffer and analysis window.

use crate::error::EncodingResult;
use crate::tables::ENWINDOW;
use std::f64::consts::PI;

/// Number of subbands in the filterbank
const SBLIMIT: usize = 32;
/// Size of the history buffer (must be power of 2 for efficient modulo)
const HAN_SIZE: usize = 512;

/// Subband filter for decomposing PCM audio into frequency bands
/// 
/// This implements the polyphase analysis filterbank as specified in the MP3 standard.
/// The filter decomposes PCM audio into 32 frequency subbands using a 512-point
/// analysis window and maintains separate history buffers for each channel.
pub struct SubbandFilter {
    /// Polyphase filter coefficients [subband][coefficient]
    /// These are the analysis filterbank coefficients calculated from cosine functions
    filter_coeffs: [[i32; 64]; SBLIMIT],
    /// History buffer for each channel [channel][sample]
    /// Circular buffer storing the last 512 samples for windowing
    history: Vec<[i32; HAN_SIZE]>,
    /// Current offset in history buffer for each channel
    /// Used for circular buffer indexing
    offset: Vec<usize>,
}

impl SubbandFilter {
    /// Create a new subband filter for the specified number of channels
    /// 
    /// Initializes the polyphase filter coefficients and allocates history buffers.
    /// The filter coefficients are calculated using the same method as the shine library.
    pub fn new(channels: usize) -> Self {
        let mut filter = Self {
            filter_coeffs: [[0; 64]; SBLIMIT],
            history: vec![[0; HAN_SIZE]; channels],
            offset: vec![0; channels],
        };
        
        filter.initialize_coefficients();
        filter
    }
    
    /// Initialize the polyphase filter coefficients
    /// 
    /// Calculates the analysis filterbank coefficients using the same formula as shine:
    /// filter[i][j] = cos((2*i + 1) * (16 - j) * PI/64)
    /// 
    /// The coefficients are scaled and converted to fixed point (i32) for efficiency.
    fn initialize_coefficients(&mut self) {
        const PI64: f64 = PI / 64.0;
        
        for i in 0..SBLIMIT {
            for j in 0..64 {
                // Calculate the cosine coefficient as in shine
                let angle = (2 * i + 1) as f64 * (16_i32 - j as i32) as f64 * PI64;
                let filter_val = angle.cos();
                
                // Round to 9th decimal place accuracy as in shine
                let scaled = filter_val * 1e9;
                let rounded = if scaled >= 0.0 {
                    (scaled + 0.5).floor()
                } else {
                    (scaled - 0.5).ceil()
                };
                
                // Scale and convert to fixed point (matches shine's scaling)
                self.filter_coeffs[i][j] = (rounded * (0x7fffffff as f64 * 1e-9)) as i32;
            }
        }
    }
    
    /// Filter PCM samples into subband samples
    /// 
    /// This implements the polyphase analysis filterbank algorithm:
    /// 1. Add new PCM samples to the history buffer
    /// 2. Apply the analysis window (ENWINDOW) to get windowed samples
    /// 3. Apply the polyphase filter matrix to produce 32 subband samples
    /// 
    /// # Arguments
    /// * `pcm_samples` - Input PCM samples (32 samples)
    /// * `output` - Output subband samples (32 subbands)
    /// * `channel` - Channel index (0 for mono/left, 1 for right)
    pub fn filter(&mut self, pcm_samples: &[i16], output: &mut [i32; 32], channel: usize) -> EncodingResult<()> {
        if pcm_samples.len() != 32 {
            return Err(crate::error::EncodingError::InvalidInputLength {
                expected: 32,
                actual: pcm_samples.len(),
            }.into());
        }
        
        if channel >= self.history.len() {
            return Err(crate::error::EncodingError::InvalidChannelIndex {
                channel,
                max_channels: self.history.len(),
            }.into());
        }
        
        // Step 1: Replace 32 oldest samples with 32 new samples
        // Convert to fixed point (shift left by 16 bits) as in shine
        for (i, &sample) in pcm_samples.iter().enumerate() {
            let index = (self.offset[channel] + i) & (HAN_SIZE - 1);
            self.history[channel][index] = (sample as i32) << 16;
        }
        
        // Step 2: Apply analysis window to produce windowed samples
        let mut windowed = [0i32; 64];
        for i in 0..64 {
            let mut sum = 0i64;
            
            // Apply windowing with 8 overlapping sections as in shine
            for section in 0..8 {
                let history_index = (self.offset[channel] + i + (section << 6)) & (HAN_SIZE - 1);
                let window_index = i + (section << 6);
                
                // Multiply history sample by window coefficient
                let history_val = self.history[channel][history_index] as i64;
                let window_val = ENWINDOW[window_index] as i64;
                sum += (history_val * window_val) >> 32; // Fixed point multiplication
            }
            
            windowed[i] = sum as i32;
        }
        
        // Update offset for next frame (move by 480 samples, equivalent to 32 new + 448 shift)
        self.offset[channel] = (self.offset[channel] + 480) & (HAN_SIZE - 1);
        
        // Step 3: Apply polyphase filter matrix to produce subband samples
        for i in 0..SBLIMIT {
            let mut sum = 0i64;
            
            // Multiply windowed samples by filter coefficients
            // Start with the last coefficient
            sum += (self.filter_coeffs[i][63] as i64) * (windowed[63] as i64);
            
            // Process remaining coefficients in groups of 7 (unrolled loop as in shine)
            let mut j = 63;
            while j > 0 {
                let end = if j >= 7 { j - 7 } else { 0 };
                for k in (end..j).rev() {
                    sum += (self.filter_coeffs[i][k] as i64) * (windowed[k] as i64);
                }
                if j >= 7 {
                    j -= 7;
                } else {
                    break;
                }
            }
            
            // Convert back from fixed point
            output[i] = (sum >> 32) as i32;
        }
        
        Ok(())
    }
    
    /// Reset the filter state
    /// 
    /// Clears all history buffers and resets offsets to zero.
    /// This should be called when starting to encode a new audio stream.
    pub fn reset(&mut self) {
        for channel_history in &mut self.history {
            channel_history.fill(0);
        }
        self.offset.fill(0);
    }
    
    /// Get the number of channels supported by this filter
    pub fn channels(&self) -> usize {
        self.history.len()
    }
    
    /// Get the current offset for a channel (for debugging/testing)
    pub fn get_offset(&self, channel: usize) -> Option<usize> {
        self.offset.get(channel).copied()
    }
}