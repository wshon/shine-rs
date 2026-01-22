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