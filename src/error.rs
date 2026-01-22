//! Error types for the MP3 encoder
//!
//! This module defines all error types used throughout the encoder,
//! providing detailed error information for different failure scenarios.

use thiserror::Error;

/// Main error type for the MP3 encoder
#[derive(Debug, Error)]
pub enum EncoderError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    /// Input data validation errors
    #[error("Input data error: {0}")]
    InputData(#[from] InputDataError),
    
    /// Encoding process errors
    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),
    
    /// Memory allocation failures
    #[error("Memory allocation error")]
    Memory,
    
    /// Internal state consistency errors
    #[error("Internal state error: {0}")]
    InternalState(String),
}

/// Configuration validation errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Unsupported sample rate
    #[error("Unsupported sample rate: {0} Hz")]
    UnsupportedSampleRate(u32),
    
    /// Unsupported bitrate
    #[error("Unsupported bitrate: {0} kbps")]
    UnsupportedBitrate(u32),
    
    /// Invalid channel configuration
    #[error("Invalid channel configuration")]
    InvalidChannels,
    
    /// Incompatible sample rate and bitrate combination
    #[error("Incompatible sample rate ({sample_rate} Hz) and bitrate ({bitrate} kbps) combination")]
    IncompatibleRateCombination { sample_rate: u32, bitrate: u32 },
    
    /// Invalid stereo mode for channel count
    #[error("Invalid stereo mode {mode:?} for {channels} channels")]
    InvalidStereoMode { mode: String, channels: u8 },
}

/// Input data validation errors
#[derive(Debug, Error)]
pub enum InputDataError {
    /// Invalid PCM data length
    #[error("Invalid PCM data length: expected {expected} samples, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    
    /// Invalid channel count in PCM data
    #[error("Invalid channel count in PCM data: expected {expected}, got {actual}")]
    InvalidChannelCount { expected: usize, actual: usize },
    
    /// PCM data contains invalid samples
    #[error("PCM data contains invalid samples")]
    InvalidSamples,
    
    /// Empty input data
    #[error("Empty input data provided")]
    EmptyInput,
}

/// Encoding process errors
#[derive(Debug, Error)]
pub enum EncodingError {
    /// Quantization loop failed to converge
    #[error("Quantization loop failed to converge within maximum iterations")]
    QuantizationFailed,
    
    /// Huffman encoding error
    #[error("Huffman encoding error: {0}")]
    HuffmanError(String),
    
    /// Bitstream writing error
    #[error("Bitstream writing error: {0}")]
    BitstreamError(String),
    
    /// MDCT transform error
    #[error("MDCT transform error: {0}")]
    MdctError(String),
    
    /// Subband filter error
    #[error("Subband filter error: {0}")]
    SubbandError(String),
    
    /// Invalid input length for processing
    #[error("Invalid input length: expected {expected} samples, got {actual}")]
    InvalidInputLength { expected: usize, actual: usize },
    
    /// Invalid data length for processing
    #[error("Invalid data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
    
    /// Invalid channel index
    #[error("Invalid channel index {channel}: maximum supported channels is {max_channels}")]
    InvalidChannelIndex { channel: usize, max_channels: usize },
    
    /// Bit reservoir overflow
    #[error("Bit reservoir overflow: attempted to use {requested} bits, only {available} available")]
    BitReservoirOverflow { requested: usize, available: usize },
}

/// Specialized result types for different modules
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type InputResult<T> = std::result::Result<T, InputDataError>;
pub type EncodingResult<T> = std::result::Result<T, EncodingError>;