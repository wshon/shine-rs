//! Lookup tables and constants for MP3 encoding
//!
//! This module contains all the static lookup tables and constants
//! required for MP3 encoding, including sample rate tables, bitrate tables,
//! subband filter coefficients, MDCT cosine tables, and Huffman code tables.

// Placeholder for tables module - will be implemented in later tasks
// This module will contain:
// - Sample rate and bitrate tables
// - Subband filter coefficients
// - MDCT cosine tables
// - Quantization step tables
// - Huffman code tables

/// Sample rate table for different MPEG versions
pub const SAMPLE_RATES: [[u32; 3]; 4] = [
    [44100, 48000, 32000], // MPEG-1
    [22050, 24000, 16000], // MPEG-2
    [11025, 12000, 8000],  // MPEG-2.5
    [0, 0, 0],             // Reserved
];

/// Bitrate table for different MPEG versions and layers
pub const BITRATES: [[[u32; 15]; 3]; 4] = [
    // MPEG-1
    [
        [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448], // Layer I
        [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384],    // Layer II
        [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320],     // Layer III
    ],
    // MPEG-2
    [
        [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256], // Layer I
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],      // Layer II
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],      // Layer III
    ],
    // MPEG-2.5 (same as MPEG-2)
    [
        [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256], // Layer I
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],      // Layer II
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],      // Layer III
    ],
    // Reserved
    [
        [0; 15], // Layer I
        [0; 15], // Layer II
        [0; 15], // Layer III
    ],
];