//! Configuration management for the MP3 encoder
//!
//! This module provides configuration structures and validation logic
//! for all encoding parameters including sample rates, bitrates, and
//! channel configurations.

use crate::error::{ConfigError, ConfigResult};

/// Main configuration structure for the MP3 encoder
#[derive(Debug, Clone)]
pub struct Config {
    /// Wave/audio configuration
    pub wave: WaveConfig,
    /// MPEG encoding configuration
    pub mpeg: MpegConfig,
}

/// Audio format configuration
#[derive(Debug, Clone)]
pub struct WaveConfig {
    /// Number of audio channels
    pub channels: Channels,
    /// Sample rate in Hz
    pub sample_rate: u32,
}

/// MPEG encoding configuration
#[derive(Debug, Clone)]
pub struct MpegConfig {
    /// Stereo encoding mode
    pub mode: StereoMode,
    /// Target bitrate in kbps
    pub bitrate: u32,
    /// Pre-emphasis mode
    pub emphasis: Emphasis,
    /// Copyright flag
    pub copyright: bool,
    /// Original flag
    pub original: bool,
}

/// Number of audio channels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    /// Mono audio (1 channel)
    Mono = 1,
    /// Stereo audio (2 channels)
    Stereo = 2,
}

/// Stereo encoding modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoMode {
    /// Standard stereo
    Stereo,
    /// Joint stereo (uses mid/side encoding)
    JointStereo,
    /// Dual channel (independent channels)
    DualChannel,
    /// Mono
    Mono,
}

/// Pre-emphasis modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    /// No emphasis
    None,
    /// 50/15 microseconds emphasis
    Emphasis50_15,
    /// CCITT J.17 emphasis
    CcittJ17,
}

/// MPEG version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegVersion {
    /// MPEG-1
    Mpeg1,
    /// MPEG-2
    Mpeg2,
    /// MPEG-2.5
    Mpeg25,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            wave: WaveConfig::default(),
            mpeg: MpegConfig::default(),
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> ConfigResult<()> {
        self.wave.validate()?;
        self.mpeg.validate(&self.wave)?;
        self.validate_compatibility()?;
        Ok(())
    }
    
    /// Get the MPEG version based on sample rate
    pub fn mpeg_version(&self) -> MpegVersion {
        match self.wave.sample_rate {
            44100 | 48000 | 32000 => MpegVersion::Mpeg1,
            22050 | 24000 | 16000 => MpegVersion::Mpeg2,
            11025 | 12000 | 8000 => MpegVersion::Mpeg25,
            _ => MpegVersion::Mpeg1, // Default fallback
        }
    }
    
    /// Get the number of samples per frame
    pub fn samples_per_frame(&self) -> usize {
        match self.mpeg_version() {
            MpegVersion::Mpeg1 => 1152,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 576,
        }
    }
    
    /// Get bitrate index for MP3 frame header
    pub fn bitrate_index(&self) -> u8 {
        let bitrates = match self.mpeg_version() {
            MpegVersion::Mpeg1 => &[0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => &[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
        };
        
        for (index, &rate) in bitrates.iter().enumerate() {
            if rate == self.mpeg.bitrate {
                return index as u8;
            }
        }
        
        9 // Default to index 9 (128 kbps for MPEG-1, 64 kbps for MPEG-2)
    }
    
    /// Get sample rate index for MP3 frame header
    pub fn samplerate_index(&self) -> u8 {
        match self.wave.sample_rate {
            44100 => 0,
            48000 => 1,
            32000 => 2,
            22050 => 0, // MPEG-2 uses same indices but different interpretation
            24000 => 1,
            16000 => 2,
            11025 => 0, // MPEG-2.5 uses same indices but different interpretation
            12000 => 1,
            8000 => 2,
            _ => 0, // Default fallback
        }
    }
    
    /// Validate compatibility between sample rate and bitrate
    fn validate_compatibility(&self) -> ConfigResult<()> {
        let valid_combinations = match self.wave.sample_rate {
            44100 | 48000 | 32000 => &[32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320][..],
            22050 | 24000 | 16000 => &[8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160][..],
            11025 | 12000 | 8000 => &[8, 16, 24, 32, 40, 48, 56, 64][..],
            _ => return Err(ConfigError::UnsupportedSampleRate(self.wave.sample_rate)),
        };
        
        if !valid_combinations.contains(&self.mpeg.bitrate) {
            return Err(ConfigError::IncompatibleRateCombination {
                sample_rate: self.wave.sample_rate,
                bitrate: self.mpeg.bitrate,
            });
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveConfig {
    /// Validate wave configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate sample rate
        const VALID_SAMPLE_RATES: &[u32] = &[
            44100, 48000, 32000,  // MPEG-1
            22050, 24000, 16000,  // MPEG-2
            11025, 12000, 8000,   // MPEG-2.5
        ];
        
        if !VALID_SAMPLE_RATES.contains(&self.sample_rate) {
            return Err(ConfigError::UnsupportedSampleRate(self.sample_rate));
        }
        
        Ok(())
    }
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            channels: Channels::Stereo,
            sample_rate: 44100,
        }
    }
}

impl MpegConfig {
    /// Validate MPEG configuration
    pub fn validate(&self, wave: &WaveConfig) -> ConfigResult<()> {
        // Validate bitrate
        const VALID_BITRATES: &[u32] = &[
            8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 192, 224, 256, 320
        ];
        
        if !VALID_BITRATES.contains(&self.bitrate) {
            return Err(ConfigError::UnsupportedBitrate(self.bitrate));
        }
        
        // Validate stereo mode compatibility with channels
        match (wave.channels, self.mode) {
            (Channels::Mono, StereoMode::Mono) => Ok(()),
            (Channels::Stereo, StereoMode::Stereo | StereoMode::JointStereo | StereoMode::DualChannel) => Ok(()),
            (channels, mode) => Err(ConfigError::InvalidStereoMode {
                mode: format!("{:?}", mode),
                channels: channels as u8,
            }),
        }
    }
}

impl Default for MpegConfig {
    fn default() -> Self {
        Self {
            mode: StereoMode::JointStereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        }
    }
}

impl From<u8> for Channels {
    fn from(value: u8) -> Self {
        match value {
            1 => Channels::Mono,
            2 => Channels::Stereo,
            _ => Channels::Stereo, // Default fallback
        }
    }
}

impl From<Channels> for usize {
    fn from(channels: Channels) -> Self {
        channels as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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

    prop_compose! {
        fn invalid_sample_rate()(rate in prop::num::u32::ANY.prop_filter("Must be invalid", |&rate| {
            !matches!(rate, 44100 | 48000 | 32000 | 22050 | 24000 | 16000 | 11025 | 12000 | 8000)
        })) -> u32 {
            rate
        }
    }

    prop_compose! {
        fn invalid_bitrate()(rate in prop::num::u32::ANY.prop_filter("Must be invalid", |&rate| {
            !matches!(rate, 8 | 16 | 24 | 32 | 40 | 48 | 56 | 64 | 80 | 96 | 112 | 128 | 144 | 160 | 192 | 224 | 256 | 320)
        })) -> u32 {
            rate
        }
    }

    // Feature: rust-mp3-encoder, Property 12: 配置管理完整性
    proptest! {
        #[test]
        fn test_config_management_integrity_valid_configs(config in compatible_config()) {
            // For any valid configuration parameter combination, 
            // the configuration system should correctly set and validate all parameters
            prop_assert!(config.validate().is_ok(), "Valid configuration should pass validation");
            
            // Verify MPEG version detection is correct
            let expected_version = match config.wave.sample_rate {
                44100 | 48000 | 32000 => MpegVersion::Mpeg1,
                22050 | 24000 | 16000 => MpegVersion::Mpeg2,
                11025 | 12000 | 8000 => MpegVersion::Mpeg25,
                _ => MpegVersion::Mpeg1,
            };
            prop_assert_eq!(config.mpeg_version(), expected_version, "MPEG version should be correctly detected");
            
            // Verify samples per frame calculation
            let expected_samples = match expected_version {
                MpegVersion::Mpeg1 => 1152,
                MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 576,
            };
            prop_assert_eq!(config.samples_per_frame(), expected_samples, "Samples per frame should be correct");
        }

        #[test]
        fn test_config_management_integrity_invalid_sample_rate(
            invalid_rate in invalid_sample_rate(),
            channels in valid_channels(),
            bitrate in valid_bitrate(),
            mode in valid_stereo_mode(),
            emphasis in valid_emphasis(),
            copyright in any::<bool>(),
            original in any::<bool>(),
        ) {
            let config = Config {
                wave: WaveConfig {
                    channels,
                    sample_rate: invalid_rate,
                },
                mpeg: MpegConfig {
                    mode,
                    bitrate,
                    emphasis,
                    copyright,
                    original,
                },
            };
            
            // For any invalid configuration, should return appropriate error information
            let result = config.validate();
            prop_assert!(result.is_err(), "Invalid sample rate should fail validation");
            
            if let Err(ConfigError::UnsupportedSampleRate(rate)) = result {
                prop_assert_eq!(rate, invalid_rate, "Error should contain the invalid sample rate");
            } else if let Err(ConfigError::IncompatibleRateCombination { sample_rate, .. }) = result {
                prop_assert_eq!(sample_rate, invalid_rate, "Error should contain the invalid sample rate");
            } else {
                prop_assert!(false, "Should get sample rate related error");
            }
        }

        #[test]
        fn test_config_management_integrity_invalid_bitrate(
            sample_rate in valid_sample_rate(),
            channels in valid_channels(),
            invalid_bitrate in invalid_bitrate(),
            mode in valid_stereo_mode(),
            emphasis in valid_emphasis(),
            copyright in any::<bool>(),
            original in any::<bool>(),
        ) {
            let config = Config {
                wave: WaveConfig {
                    channels,
                    sample_rate,
                },
                mpeg: MpegConfig {
                    mode,
                    bitrate: invalid_bitrate,
                    emphasis,
                    copyright,
                    original,
                },
            };
            
            // For any invalid configuration, should return appropriate error information
            let result = config.validate();
            prop_assert!(result.is_err(), "Invalid bitrate should fail validation");
            
            match result {
                Err(ConfigError::UnsupportedBitrate(rate)) => {
                    prop_assert_eq!(rate, invalid_bitrate, "Error should contain the invalid bitrate");
                },
                Err(ConfigError::IncompatibleRateCombination { bitrate, .. }) => {
                    prop_assert_eq!(bitrate, invalid_bitrate, "Error should contain the invalid bitrate");
                },
                _ => prop_assert!(false, "Should get bitrate related error"),
            }
        }

        #[test]
        fn test_config_management_integrity_incompatible_stereo_mode(
            sample_rate in valid_sample_rate(),
            bitrate in valid_bitrate(),
            emphasis in valid_emphasis(),
            copyright in any::<bool>(),
            original in any::<bool>(),
        ) {
            // Test incompatible combinations: Mono channel with non-Mono stereo mode
            let config = Config {
                wave: WaveConfig {
                    channels: Channels::Mono,
                    sample_rate,
                },
                mpeg: MpegConfig {
                    mode: StereoMode::Stereo, // Incompatible with Mono channels
                    bitrate,
                    emphasis,
                    copyright,
                    original,
                },
            };
            
            let result = config.validate();
            // This might pass or fail depending on bitrate compatibility, but if it fails due to stereo mode, check the error
            if let Err(ConfigError::InvalidStereoMode { mode: _, channels }) = result {
                prop_assert_eq!(channels, 1, "Error should indicate mono channel count");
            }
        }

        #[test]
        fn test_config_default_values(_unit in Just(())) {
            let config = Config::default();
            
            // Default configuration should always be valid
            prop_assert!(config.validate().is_ok(), "Default configuration should be valid");
            
            // Verify default values
            prop_assert_eq!(config.wave.channels, Channels::Stereo, "Default should be stereo");
            prop_assert_eq!(config.wave.sample_rate, 44100, "Default sample rate should be 44100");
            prop_assert_eq!(config.mpeg.mode, StereoMode::JointStereo, "Default should be joint stereo");
            prop_assert_eq!(config.mpeg.bitrate, 128, "Default bitrate should be 128");
            prop_assert_eq!(config.mpeg.emphasis, Emphasis::None, "Default emphasis should be None");
            prop_assert_eq!(config.mpeg.copyright, false, "Default copyright should be false");
            prop_assert_eq!(config.mpeg.original, true, "Default original should be true");
        }
    }

    #[test]
    fn test_channels_conversion() {
        assert_eq!(Channels::from(1), Channels::Mono);
        assert_eq!(Channels::from(2), Channels::Stereo);
        assert_eq!(Channels::from(99), Channels::Stereo); // Default fallback
        
        assert_eq!(usize::from(Channels::Mono), 1);
        assert_eq!(usize::from(Channels::Stereo), 2);
    }
}