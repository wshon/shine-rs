//! Main MP3 encoder implementation
//!
//! This module provides the main Mp3Encoder struct that coordinates
//! all the encoding stages from PCM input to MP3 output.

use crate::config::Config;
use crate::shine_config::ShineGlobalConfig;
use crate::reservoir::BitReservoir;
use crate::error::{EncoderError, InputDataError};
use crate::quantization::{QuantizationLoop, GranuleInfo};
use crate::pcm_utils;
use crate::Result;

/// Main MP3 encoder structure following shine's architecture
#[allow(dead_code)]
pub struct Mp3Encoder {
    /// Internal global configuration containing all encoding state
    global_config: ShineGlobalConfig,
    /// Public API configuration
    config: Config,
    /// Output frame buffer
    frame_buffer: Vec<u8>,
    /// Samples accumulated in buffer
    samples_in_buffer: usize,
    /// Frame size calculation parameters
    whole_slots_per_frame: usize,
    frac_slots_per_frame: f64,
    slot_lag: f64,
    /// Bit reservoir for rate control
    reservoir: BitReservoir,
    /// Quantization loop for rate control
    quantization_loop: QuantizationLoop,
    /// Granule info for current frame
    current_granule_info: Vec<GranuleInfo>,
}

impl Mp3Encoder {
    /// Create a new MP3 encoder with the specified configuration
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let config = config.clone();
        let mut shine_config = ShineGlobalConfig::new(config.clone())?;
        shine_config.initialize()?;

        let channels: usize = config.wave.channels.into();
        let bitrate_kbps = config.mpeg.bitrate;
        let sample_rate = config.wave.sample_rate;
        let granule_size = 576;
        let bits_per_slot = 8;

        let granules_per_frame = match config.mpeg_version() {
            crate::config::MpegVersion::Mpeg1 => 2,
            crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 1,
        };

        // Calculate frame size parameters following shine's logic
        let avg_slots_per_frame = ((granules_per_frame * granule_size) as f64 / sample_rate as f64)
            * (1000.0 * bitrate_kbps as f64 / bits_per_slot as f64);

        let whole_slots_per_frame = avg_slots_per_frame as usize;
        let frac_slots_per_frame = avg_slots_per_frame - whole_slots_per_frame as f64;
        let slot_lag = -frac_slots_per_frame;

        let reservoir = BitReservoir::new(bitrate_kbps, sample_rate, channels as u8);
        let quantization_loop = QuantizationLoop::new();

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
    pub fn encode_frame(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let expected_samples = samples_per_frame * channels;
        
        if pcm_data.len() != expected_samples {
            return Err(EncoderError::InputData(InputDataError::InvalidLength {
                expected: expected_samples,
                actual: pcm_data.len(),
            }));
        }
        
        self.prepare_frame();
        pcm_utils::deinterleave_pcm_non_interleaved(
            pcm_data, 
            channels, 
            samples_per_frame, 
            &mut self.global_config.buffer[..channels]
        );
        self.samples_in_buffer = 0;
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Encode a frame of interleaved PCM data (L, R, L, R, ...)
    pub fn encode_frame_interleaved(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let expected_samples = samples_per_frame * channels;
        
        if pcm_data.len() != expected_samples {
            return Err(EncoderError::InputData(InputDataError::InvalidLength {
                expected: expected_samples,
                actual: pcm_data.len(),
            }));
        }
        
        self.prepare_frame();
        pcm_utils::deinterleave_pcm_interleaved(
            pcm_data, 
            channels, 
            samples_per_frame, 
            &mut self.global_config.buffer[..channels]
        );
        self.samples_in_buffer = 0;
        self.encode_frame_pipeline(channels, samples_per_frame)
    }
    
    /// Encode samples incrementally, buffering until a complete frame is available
    pub fn encode_samples(&mut self, pcm_data: &[i16]) -> Result<Option<&[u8]>> {
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        let samples_per_channel = pcm_data.len() / channels;
        
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
        
        if self.samples_in_buffer >= samples_per_frame {
            self.prepare_frame();
            self.encode_frame_pipeline(channels, samples_per_frame)?;
            
            // Remove encoded samples from buffer
            for ch in 0..channels {
                self.global_config.buffer[ch].drain(0..samples_per_frame);
            }
            self.samples_in_buffer -= samples_per_frame;
            
            Ok(Some(&self.frame_buffer))
        } else {
            Ok(None)
        }
    }
    
    /// Encode a frame of interleaved PCM data (alias for encode_frame_interleaved)
    pub fn encode(&mut self, pcm_data: &[i16]) -> Result<&[u8]> {
        self.encode_frame_interleaved(pcm_data)
    }
    
    /// Flush any remaining data and finalize encoding
    pub fn flush(&mut self) -> Result<&[u8]> {
        if self.samples_in_buffer == 0 {
            self.frame_buffer.clear();
            return Ok(&self.frame_buffer);
        }
        
        let channels = self.global_config.wave.channels as usize;
        let samples_per_frame = self.config.samples_per_frame();
        
        // Pad partial data to complete frame
        if self.samples_in_buffer < samples_per_frame {
            for ch in 0..channels {
                while self.global_config.buffer[ch].len() < samples_per_frame {
                    self.global_config.buffer[ch].push(0);
                }
            }
        }
        
        self.prepare_frame();
        self.encode_frame_pipeline(channels, samples_per_frame)?;
        
        // Clear buffer after flushing
        self.samples_in_buffer = 0;
        for channel_buffer in &mut self.global_config.buffer {
            channel_buffer.clear();
        }
        
        Ok(&self.frame_buffer)
    }
    
    /// Get the number of samples per frame for this configuration
    pub fn samples_per_frame(&self) -> usize {
        self.config.samples_per_frame()
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
        for ch in 0..self.global_config.wave.channels as usize {
            self.global_config.buffer[ch].clear();
        }
        self.frame_buffer.clear();
        self.samples_in_buffer = 0;
        self.global_config.side_info = crate::shine_config::ShineSideInfo::default();
        self.global_config.bs.reset();
    }
    
    /// Prepare frame buffer and bitstream for new frame
    fn prepare_frame(&mut self) {
        self.frame_buffer.clear();
        self.global_config.bs.reset();
    }
    
    /// Main encoding pipeline following shine's encode_buffer_internal
    fn encode_frame_pipeline(&mut self, channels: usize, _samples_per_frame: usize) -> Result<&[u8]> {
        // Step 1: Padding calculation
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
        
        // Step 2: Calculate frame size
        let bits_per_frame = 8 * (self.whole_slots_per_frame + if padding { 1 } else { 0 });
        let target_frame_bytes = bits_per_frame / 8;
        
        // Step 3: Calculate mean_bits
        let granules_per_frame = match self.config.mpeg_version() {
            crate::config::MpegVersion::Mpeg1 => 2,
            crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 1,
        };
        let sideinfo_len = if self.config.mpeg_version() == crate::config::MpegVersion::Mpeg1 {
            8 * if channels == 1 { 4 + 17 } else { 4 + 32 }
        } else {
            8 * if channels == 1 { 4 + 9 } else { 4 + 17 }
        };
        let _mean_bits = (bits_per_frame - sideinfo_len) / granules_per_frame;
        
        // Step 4: Apply MDCT transform
        crate::mdct::shine_mdct_sub(&mut self.global_config, 1);
        
        // Step 5: Bit and noise allocation
        let (granule_info, quantized_coeffs) = self.quantization_loop.encode_granules(
            &self.global_config.mdct_freq,
            channels,
            granules_per_frame,
            self.global_config.wave.sample_rate,
            &mut self.reservoir,
            self.config.mpeg_version(),
        )?;
        
        // Store results
        self.current_granule_info = granule_info;
        for ch in 0..channels {
            for gr in 0..granules_per_frame {
                self.global_config.l3_enc[ch][gr] = quantized_coeffs[ch][gr];
            }
        }
        
        // Step 6: Format bitstream
        self.global_config.bs.format_frame(
            &self.config,
            &self.current_granule_info,
            padding,
            target_frame_bytes,
        )?;
        
        // Step 7: Return encoded data
        let encoded_data = self.global_config.bs.flush();
        self.frame_buffer.clear();
        self.frame_buffer.extend_from_slice(encoded_data);
        self.global_config.bs.reset();
        
        Ok(&self.frame_buffer)
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
    fn test_encoder_creation() {
        let config = Config::default();
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_ok(), "Encoder creation should succeed");
        
        let encoder = encoder.unwrap();
        assert_eq!(encoder.samples_per_frame(), 1152);
        assert_eq!(encoder.public_config().wave.channels, Channels::Stereo);
    }

    #[test]
    fn test_invalid_config() {
        let mut config = Config::default();
        config.mpeg.bitrate = 999;
        
        let encoder = Mp3Encoder::new(config);
        assert!(encoder.is_err(), "Invalid config should be rejected");
    }

    #[test]
    fn test_frame_encoding() {
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
        let pcm_data = vec![0i16; 1152];
        
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_ok(), "Frame encoding should succeed");
        
        let encoded_frame = result.unwrap();
        assert!(!encoded_frame.is_empty(), "Frame should not be empty");
        
        // Check MP3 sync word
        let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "Frame should start with MP3 sync word");
    }

    #[test]
    fn test_interleaved_encoding() {
        let config = Config::default();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        let pcm_data = vec![0i16; 1152 * 2];
        
        let result = encoder.encode_frame_interleaved(&pcm_data);
        assert!(result.is_ok(), "Interleaved encoding should succeed");
        
        let encoded_frame = result.unwrap();
        assert!(!encoded_frame.is_empty(), "Frame should not be empty");
    }

    #[test]
    fn test_invalid_input_length() {
        let config = Config::default();
        let mut encoder = Mp3Encoder::new(config).unwrap();
        let pcm_data = vec![0i16; 100];
        
        let result = encoder.encode_frame(&pcm_data);
        assert!(result.is_err(), "Wrong input length should be rejected");
        
        if let Err(EncoderError::InputData(InputDataError::InvalidLength { expected, actual })) = result {
            assert_eq!(expected, 1152 * 2);
            assert_eq!(actual, 100);
        } else {
            panic!("Should return InvalidLength error");
        }
    }

    #[test]
    fn test_incremental_encoding() {
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
        
        // Add partial samples
        let partial_samples = vec![100i16; 500];
        let result = encoder.encode_samples(&partial_samples);
        assert!(result.is_ok(), "Partial encoding should succeed");
        assert!(result.unwrap().is_none(), "Partial frame should return None");
        
        // Complete the frame
        let remaining_samples = vec![200i16; 652];
        let result = encoder.encode_samples(&remaining_samples);
        assert!(result.is_ok(), "Completing frame should succeed");
        
        let encoded_frame = result.unwrap();
        assert!(encoded_frame.is_some(), "Complete frame should return Some");
        
        let frame_data = encoded_frame.unwrap();
        let sync = ((frame_data[0] as u16) << 3) | ((frame_data[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "Frame should have valid sync word");
    }

    #[test]
    fn test_flush_functionality() {
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
        
        // Add partial samples
        let partial_samples = vec![300i16; 800];
        let result = encoder.encode_samples(&partial_samples);
        assert!(result.is_ok(), "Partial encoding should succeed");
        assert!(result.unwrap().is_none(), "Partial frame should be buffered");
        
        // Flush should encode remaining data
        let flush_result = encoder.flush();
        assert!(flush_result.is_ok(), "Flush should succeed");
        
        let flushed_data = flush_result.unwrap();
        assert!(!flushed_data.is_empty(), "Flush should return encoded frame");
        
        let sync = ((flushed_data[0] as u16) << 3) | ((flushed_data[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "Flushed frame should have valid sync word");
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
        
        let pcm_data1: Vec<i16> = (0..1152).map(|i| (i % 100) as i16 * 10).collect();
        let pcm_data2: Vec<i16> = (0..1152).map(|i| (i % 200) as i16 * 20).collect();
        
        let result1 = encoder.encode_frame(&pcm_data1);
        assert!(result1.is_ok(), "First encoding should succeed");
        let encoded1 = result1.unwrap().to_vec();
        
        encoder.reset();
        
        let result2 = encoder.encode_frame(&pcm_data2);
        assert!(result2.is_ok(), "Second encoding should succeed");
        let encoded2 = result2.unwrap().to_vec();
        
        assert_ne!(encoded1, encoded2, "Different inputs should produce different outputs");
        
        // Both should be valid MP3 frames
        let sync1 = ((encoded1[0] as u16) << 3) | ((encoded1[1] as u16) >> 5);
        let sync2 = ((encoded2[0] as u16) << 3) | ((encoded2[1] as u16) >> 5);
        assert_eq!(sync1, 0x7FF, "First frame should have valid sync word");
        assert_eq!(sync2, 0x7FF, "Second frame should have valid sync word");
    }

    // Property test generators
    prop_compose! {
        fn valid_sample_rate()(rate in prop::sample::select(&[44100u32, 22050, 11025])) -> u32 {
            rate
        }
    }

    prop_compose! {
        fn valid_bitrate()(rate in prop::sample::select(&[128u32, 192, 320])) -> u32 {
            rate
        }
    }

    prop_compose! {
        fn valid_channels()(channels in prop::sample::select(&[Channels::Mono, Channels::Stereo])) -> Channels {
            channels
        }
    }

    fn compatible_config() -> impl Strategy<Value = Config> {
        (valid_sample_rate(), valid_channels(), any::<bool>(), any::<bool>())
            .prop_map(|(sample_rate, channels, copyright, original)| {
                let bitrate = match sample_rate {
                    44100 => 128,
                    22050 => 64,
                    11025 => 32,
                    _ => 128,
                };
                
                let mode = match channels {
                    Channels::Mono => StereoMode::Mono,
                    Channels::Stereo => StereoMode::Stereo,
                };
                
                Config {
                    wave: WaveConfig {
                        channels,
                        sample_rate,
                    },
                    mpeg: MpegConfig {
                        mode,
                        bitrate,
                        emphasis: Emphasis::None,
                        copyright,
                        original,
                    },
                }
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 20,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_encoder_initialization(config in compatible_config()) {
            setup_clean_errors();
            
            let encoder_result = Mp3Encoder::new(config.clone());
            prop_assert!(encoder_result.is_ok(), "Encoder initialization failed");
            
            let encoder = encoder_result.unwrap();
            prop_assert_eq!(encoder.public_config().wave.channels, config.wave.channels, "Channel mismatch");
            prop_assert_eq!(encoder.public_config().wave.sample_rate, config.wave.sample_rate, "Sample rate mismatch");
        }

        #[test]
        fn test_frame_encoding_property(config in compatible_config()) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            let total_samples = samples_per_frame * channels;
            
            let pcm_data: Vec<i16> = (0..total_samples)
                .map(|i| (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * i as f64 / 44100.0).sin()) as i16)
                .collect();
            
            let encode_result = encoder.encode_frame(&pcm_data);
            prop_assert!(encode_result.is_ok(), "Frame encoding failed");
            
            let encoded_frame = encode_result.unwrap();
            prop_assert!(!encoded_frame.is_empty(), "Frame should not be empty");
            
            // Verify MP3 sync word
            prop_assert!(encoded_frame.len() >= 4, "Frame should be at least 4 bytes");
            let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Frame should start with MP3 sync word");
        }

        #[test]
        fn test_flush_completeness(
            config in compatible_config(),
            partial_samples_count in 1usize..100,
        ) {
            setup_clean_errors();
            
            let mut encoder = Mp3Encoder::new(config.clone()).unwrap();
            
            let samples_per_frame = config.samples_per_frame();
            let channels = config.wave.channels as usize;
            let partial_count = partial_samples_count % samples_per_frame;
            
            if partial_count == 0 {
                return Ok(());
            }
            
            let partial_pcm: Vec<i16> = (0..partial_count * channels)
                .map(|i| (1000.0 * (2.0 * std::f64::consts::PI * 220.0 * i as f64 / 44100.0).sin()) as i16)
                .collect();
            
            let partial_result = encoder.encode_samples(&partial_pcm);
            prop_assert!(partial_result.is_ok(), "Partial encoding should succeed");
            prop_assert!(partial_result.unwrap().is_none(), "Partial samples should not produce output");
            
            let flush_result = encoder.flush();
            prop_assert!(flush_result.is_ok(), "Flush should succeed");
            
            let flushed_data = flush_result.unwrap();
            prop_assert!(!flushed_data.is_empty(), "Flush should return encoded data");
            
            let sync = ((flushed_data[0] as u16) << 3) | ((flushed_data[1] as u16) >> 5);
            prop_assert_eq!(sync, 0x7FF, "Flushed frame should have valid sync word");
        }
    }
}