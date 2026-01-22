//! Main MP3 encoder implementation
//!
//! This module provides the main Mp3Encoder struct that coordinates
//! all the encoding stages from PCM input to MP3 output.

use crate::config::Config;
use crate::subband::SubbandFilter;
use crate::mdct::MdctTransform;
use crate::quantization::{QuantizationLoop, GranuleInfo};
use crate::huffman::HuffmanEncoder;
use crate::bitstream::{BitstreamWriter, SideInfo};
use crate::error::{EncoderError, InputDataError};
use crate::Result;

/// Main MP3 encoder structure
#[allow(dead_code)]
pub struct Mp3Encoder {
    /// Encoder configuration
    config: Config,
    /// Subband filter
    subband: SubbandFilter,
    /// MDCT transform
    mdct: MdctTransform,
    /// Quantization loop
    quantizer: QuantizationLoop,
    /// Huffman encoder
    huffman: HuffmanEncoder,
    /// Bitstream writer
    bitstream: BitstreamWriter,
    /// Input buffer for each channel
    buffer: Vec<Vec<i16>>,
    /// Output frame buffer
    frame_buffer: Vec<u8>,
    /// Samples accumulated in buffer
    samples_in_buffer: usize,
}

impl Mp3Encoder {
    /// Create a new MP3 encoder with the specified configuration
    pub fn new(config: Config) -> Result<Self> {
        // Validate configuration
        config.validate()?;
        
        let channels = config.wave.channels as usize;
        let samples_per_frame = config.samples_per_frame();
        
        Ok(Self {
            subband: SubbandFilter::new(channels),
            mdct: MdctTransform::new(),
            quantizer: QuantizationLoop::new(),
            huffman: HuffmanEncoder::new(),
            bitstream: BitstreamWriter::new(2048), // Typical MP3 frame size
            buffer: vec![Vec::with_capacity(samples_per_frame); channels],
            frame_buffer: Vec::with_capacity(2048),
            samples_in_buffer: 0,
            config,
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
        let channels = self.config.wave.channels as usize;
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
        self.bitstream.reset();
        
        // De-interleave PCM data into channel buffers
        self.deinterleave_pcm(pcm_data, channels, samples_per_frame);
        
        // Encode the frame through the complete pipeline
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Encode a frame of interleaved PCM data
    /// 
    /// # Arguments
    /// * `pcm_data` - Interleaved PCM samples [L, R, L, R, ...]
    /// 
    /// # Returns
    /// * `Ok(&[u8])` - Encoded MP3 frame data
    /// * `Err(EncoderError)` - Encoding error
    pub fn encode_frame_interleaved(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        let channels = self.config.wave.channels as usize;
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
        self.bitstream.reset();
        
        // De-interleave PCM data into channel buffers
        self.deinterleave_pcm_interleaved(pcm_data, channels, samples_per_frame);
        
        // Encode the frame through the complete pipeline
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Flush any remaining data and finalize encoding
    /// 
    /// # Returns
    /// * `Ok(&[u8])` - Final MP3 frame data (may be empty)
    /// * `Err(EncoderError)` - Encoding error
    pub fn flush(&mut self) -> Result<&[u8]> {
        // For now, just return empty buffer since we don't buffer partial frames
        // In a full implementation, this would handle any remaining buffered samples
        self.frame_buffer.clear();
        Ok(&self.frame_buffer)
    }
    
    /// Get the number of samples per frame for this configuration
    pub fn samples_per_frame(&self) -> usize {
        self.config.samples_per_frame()
    }
    
    /// Get the encoder configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Reset the encoder state
    pub fn reset(&mut self) {
        self.subband.reset();
        self.quantizer = QuantizationLoop::new();
        self.bitstream.reset();
        for channel_buffer in &mut self.buffer {
            channel_buffer.clear();
        }
        self.frame_buffer.clear();
        self.samples_in_buffer = 0;
    }
    
    /// De-interleave non-interleaved PCM data into channel buffers
    /// For non-interleaved data: [ch0_sample0, ch0_sample1, ..., ch1_sample0, ch1_sample1, ...]
    fn deinterleave_pcm(&mut self, pcm_data: &[i16], channels: usize, samples_per_frame: usize) {
        for ch in 0..channels {
            self.buffer[ch].clear();
            self.buffer[ch].reserve(samples_per_frame);
            
            let channel_start = ch * samples_per_frame;
            let channel_end = channel_start + samples_per_frame;
            
            for sample_idx in channel_start..channel_end {
                if sample_idx < pcm_data.len() {
                    self.buffer[ch].push(pcm_data[sample_idx]);
                }
            }
        }
    }
    
    /// De-interleave interleaved PCM data into channel buffers
    /// For interleaved data: [L, R, L, R, L, R, ...]
    fn deinterleave_pcm_interleaved(&mut self, pcm_data: &[i16], channels: usize, samples_per_frame: usize) {
        for ch in 0..channels {
            self.buffer[ch].clear();
            self.buffer[ch].reserve(samples_per_frame);
        }
        
        for sample_idx in 0..samples_per_frame {
            for ch in 0..channels {
                let interleaved_idx = sample_idx * channels + ch;
                if interleaved_idx < pcm_data.len() {
                    self.buffer[ch].push(pcm_data[interleaved_idx]);
                }
            }
        }
    }
    
    /// Main encoding pipeline that processes PCM data through all stages
    fn encode_frame_pipeline(&mut self, channels: usize, samples_per_frame: usize) -> Result<&[u8]> {
        // Step 1: Write MP3 frame header
        self.bitstream.write_frame_header(&self.config, false); // No padding for now
        
        // Step 2: Prepare side information structure
        let mut side_info = SideInfo::default();
        self.prepare_side_info(&mut side_info, channels);
        
        // Step 3: Process each channel through the encoding pipeline
        for ch in 0..channels {
            self.encode_channel(ch, samples_per_frame, &mut side_info)?;
        }
        
        // Step 4: Write side information
        self.bitstream.write_side_info(&side_info, &self.config);
        
        // Step 5: Flush bitstream and copy to frame buffer
        let encoded_data = self.bitstream.flush();
        self.frame_buffer.clear();
        self.frame_buffer.extend_from_slice(encoded_data);
        
        Ok(&self.frame_buffer)
    }
    
    /// Prepare side information structure for the frame
    fn prepare_side_info(&self, side_info: &mut SideInfo, channels: usize) {
        use crate::config::MpegVersion;
        
        // Set private bits (unused for now)
        side_info.private_bits = 0;
        
        // Initialize SCFSI (Scale Factor Selection Information) - all false for now
        side_info.scfsi = [[false; 4]; 2];
        
        // Determine number of granules based on MPEG version
        let granules_per_frame = match self.config.mpeg_version() {
            MpegVersion::Mpeg1 => 2,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 1,
        };
        
        // Initialize granule information for each granule*channel combination
        side_info.granules.clear();
        for _gr in 0..granules_per_frame {
            for _ch in 0..channels {
                let mut granule_info = GranuleInfo::default();
                
                // Set default values - these will be updated during encoding
                granule_info.part2_3_length = 0;
                granule_info.big_values = 0;
                granule_info.global_gain = 210; // Default global gain
                granule_info.scalefac_compress = 0;
                granule_info.table_select = [1, 1, 1]; // Use table 1 as default (table 0 doesn't exist)
                granule_info.region0_count = 0;
                granule_info.region1_count = 0;
                granule_info.preflag = false;
                granule_info.scalefac_scale = false;
                granule_info.count1table_select = false;
                
                side_info.granules.push(granule_info);
            }
        }
    }
    
    /// Encode a single channel through the complete pipeline
    fn encode_channel(&mut self, channel: usize, samples_per_frame: usize, side_info: &mut SideInfo) -> Result<()> {
        use crate::config::MpegVersion;
        
        // Determine number of granules based on MPEG version
        let granules_per_frame = match self.config.mpeg_version() {
            MpegVersion::Mpeg1 => 2,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 1,
        };
        
        let samples_per_granule = samples_per_frame / granules_per_frame;
        
        // Process each granule for this channel
        for granule in 0..granules_per_frame {
            let granule_index = granule * (self.config.wave.channels as usize) + channel;
            
            if granule_index < side_info.granules.len() {
                self.encode_granule(channel, granule, samples_per_granule, 
                                  &mut side_info.granules[granule_index])?;
            }
        }
        
        Ok(())
    }
    
    /// Encode a single granule (portion of a channel's data)
    fn encode_granule(&mut self, channel: usize, granule: usize, samples_per_granule: usize, 
                     granule_info: &mut GranuleInfo) -> Result<()> {
        // Step 1: Extract PCM samples for this granule
        let granule_start = granule * samples_per_granule;
        let granule_end = granule_start + samples_per_granule;
        
        if granule_end > self.buffer[channel].len() {
            return Err(EncoderError::InputData(InputDataError::InvalidLength {
                expected: granule_end,
                actual: self.buffer[channel].len(),
            }));
        }
        
        let granule_samples = &self.buffer[channel][granule_start..granule_end];
        
        // Step 2: Subband filtering (32 samples at a time)
        let mut subband_samples = [[0i32; 32]; 36]; // 36 granules of 32 subbands each
        
        for (i, chunk) in granule_samples.chunks(32).enumerate() {
            if i >= 36 { break; } // Safety check
            
            let mut chunk_32 = [0i16; 32];
            for (j, &sample) in chunk.iter().enumerate() {
                if j < 32 {
                    chunk_32[j] = sample;
                }
            }
            
            // Pad with zeros if chunk is smaller than 32
            if chunk.len() < 32 {
                for j in chunk.len()..32 {
                    chunk_32[j] = 0;
                }
            }
            
            self.subband.filter(&chunk_32, &mut subband_samples[i], channel)?;
        }
        
        // Step 3: MDCT transform
        let mut mdct_coeffs = [0i32; 576];
        self.mdct.transform(&subband_samples, &mut mdct_coeffs)?;
        
        // Step 4: Apply aliasing reduction
        self.mdct.apply_aliasing_reduction(&mut mdct_coeffs)?;
        
        // Step 5: Quantization and rate control
        let max_bits = 1000; // Simplified bit allocation for now
        let mut quantized_coeffs = [0i32; 576];
        
        let _bits_used = self.quantizer.quantize_and_encode(
            &mdct_coeffs, 
            max_bits, 
            granule_info, 
            &mut quantized_coeffs
        )?;
        
        // Step 6: Huffman encoding (write directly to bitstream)
        let _big_values_bits = self.huffman.encode_big_values(
            &quantized_coeffs, 
            granule_info, 
            &mut self.bitstream
        )?;
        
        let _count1_bits = self.huffman.encode_count1(
            &quantized_coeffs, 
            granule_info, 
            &mut self.bitstream
        )?;
        
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
        assert_eq!(encoder.config().wave.channels, Channels::Stereo);
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
    fn test_mp3_encoder_encode_frame_basic() {
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
        assert!(flushed_data.is_empty(), "Flush should return empty data for now");
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
            44100u32, 48000, 32000,  // MPEG-1
            22050, 24000, 16000,     // MPEG-2
            11025, 12000, 8000,      // MPEG-2.5
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
        fn valid_emphasis()(emphasis in prop::sample::select(&[
            Emphasis::None, Emphasis::Emphasis50_15, Emphasis::CcittJ17
        ])) -> Emphasis {
            emphasis
        }
    }

    fn compatible_config() -> impl Strategy<Value = Config> {
        (valid_sample_rate(), valid_channels(), valid_emphasis(), any::<bool>(), any::<bool>())
            .prop_flat_map(|(sample_rate, channels, emphasis, copyright, original)| {
                let bitrate_strategy = match sample_rate {
                    44100 | 48000 | 32000 => prop::sample::select(vec![32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320]),
                    22050 | 24000 | 16000 => prop::sample::select(vec![8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160]),
                    11025 | 12000 | 8000 => prop::sample::select(vec![8, 16, 24, 32, 40, 48, 56, 64]),
                    _ => prop::sample::select(vec![128]), // fallback
                };
                
                let mode_strategy = match channels {
                    Channels::Mono => prop::sample::select(vec![StereoMode::Mono]),
                    Channels::Stereo => prop::sample::select(vec![StereoMode::Stereo, StereoMode::JointStereo, StereoMode::DualChannel]),
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
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_encoder_initialization_and_basic_functionality(config in compatible_config()) {
            setup_clean_errors();
            
            // For any valid encoding configuration, encoder should successfully initialize
            let encoder_result = Mp3Encoder::new(config.clone());
            prop_assert!(encoder_result.is_ok(), "Encoder initialization failed");
            
            let encoder = encoder_result.unwrap();
            
            // Verify encoder properties match configuration
            prop_assert_eq!(encoder.config().wave.channels, config.wave.channels, "Channel configuration mismatch");
            prop_assert_eq!(encoder.config().wave.sample_rate, config.wave.sample_rate, "Sample rate mismatch");
            prop_assert_eq!(encoder.config().mpeg.bitrate, config.mpeg.bitrate, "Bitrate mismatch");
            
            // Verify samples per frame calculation
            let expected_samples = match config.mpeg_version() {
                crate::config::MpegVersion::Mpeg1 => 1152,
                crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 576,
            };
            prop_assert_eq!(encoder.samples_per_frame(), expected_samples, "Samples per frame mismatch");
        }

        #[test]
        fn test_encoder_basic_functionality_with_valid_pcm(
            config in compatible_config(),
        ) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            // Generate valid PCM data for this configuration
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            let total_samples = samples_per_frame * channels;
            
            // Create PCM data with appropriate size (use simple pattern for deterministic testing)
            let pcm_data: Vec<i16> = (0..total_samples)
                .map(|i| ((i % 1000) as i16) * 32)
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
                .map(|i| ((i % 2000) as i16) * 16)
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
    }
}