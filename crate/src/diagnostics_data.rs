//! Test data collection and validation for MP3 encoding
//!
//! This module provides functionality to collect key encoding parameters
//! during the encoding process and save them to JSON for later validation.
//! 
//! This module is only available when the "diagnostics" feature is enabled.

#[cfg(feature = "diagnostics")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "diagnostics")]
use std::fs::File;
#[cfg(feature = "diagnostics")]
use std::io::Write;
#[cfg(feature = "diagnostics")]
use std::sync::Mutex;

/// Frame-specific encoding data
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    /// Frame number (1-based)
    pub frame_number: i32,
    
    /// MDCT coefficients for key positions
    pub mdct_coefficients: MdctData,
    
    /// Quantization parameters
    pub quantization: QuantizationData,
    
    /// Bitstream parameters
    pub bitstream: BitstreamData,
}

/// MDCT coefficient data
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdctData {
    /// Key MDCT coefficients [ch][gr][band][k] for verification
    /// Only stores coefficients at positions [0][0][0][15], [0][0][0][16], [0][0][0][17]
    pub coefficients: Vec<i32>,
    
    /// Saved l3_sb_sample values for verification
    pub l3_sb_sample: Vec<i32>,
}

/// Quantization data
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationData {
    /// Maximum spectral value (xrmax)
    pub xrmax: i32,
    
    /// Maximum bits available
    pub max_bits: i32,
    
    /// Part2_3_length after quantization
    pub part2_3_length: u32,
    
    /// Quantizer step size
    pub quantizer_step_size: i32,
    
    /// Global gain
    pub global_gain: u32,
}

/// Bitstream data
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitstreamData {
    /// Padding bit
    pub padding: i32,
    
    /// Bits per frame
    pub bits_per_frame: i32,
    
    /// Bytes written for this frame
    pub written: usize,
    
    /// Slot lag value
    pub slot_lag: f64,
}

/// Complete test case data
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseData {
    /// Test case metadata
    pub metadata: TestMetadata,
    
    /// Encoding configuration
    pub config: EncodingConfig,
    
    /// Frame data for first 6 frames
    pub frames: Vec<FrameData>,
}

/// Type alias for backward compatibility with integration tests
#[cfg(feature = "diagnostics")]
pub type TestDataSet = TestCaseData;

/// Test metadata
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Test case name
    pub name: String,
    
    /// Input audio file path
    pub input_file: String,
    
    /// Expected output file size in bytes
    pub expected_output_size: usize,
    
    /// Expected SHA256 hash of output
    pub expected_hash: String,
    
    /// Creation timestamp
    pub created_at: String,
    
    /// Description
    pub description: String,
}

/// Encoding configuration
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    /// Sample rate in Hz
    pub sample_rate: i32,
    
    /// Number of channels
    pub channels: i32,
    
    /// Bitrate in kbps
    pub bitrate: i32,
    
    /// Stereo mode
    pub stereo_mode: i32,
    
    /// MPEG version
    pub mpeg_version: i32,
}

/// Global test data collector
#[cfg(feature = "diagnostics")]
static TEST_DATA_COLLECTOR: Mutex<Option<TestDataCollector>> = Mutex::new(None);

/// Test data collector implementation
#[cfg(feature = "diagnostics")]
#[derive(Debug)]
pub struct TestDataCollector {
    pub test_case: TestCaseData,
    pub current_frame: i32,
}

#[cfg(feature = "diagnostics")]
impl TestDataCollector {
    /// Initialize the test data collector
    pub fn initialize(metadata: TestMetadata, config: EncodingConfig) {
        let collector = TestDataCollector {
            test_case: TestCaseData {
                metadata,
                config,
                frames: Vec::new(),
            },
            current_frame: 0,
        };
        
        let mut guard = TEST_DATA_COLLECTOR.lock().unwrap();
        *guard = Some(collector);
    }
    
    /// Start collecting data for a new frame
    pub fn start_frame(frame_number: i32) {
        let mut guard = TEST_DATA_COLLECTOR.lock().unwrap();
        if let Some(collector) = guard.as_mut() {
            collector.current_frame = frame_number;
            
            // Initialize frame data if this is a new frame within our collection range
            if frame_number <= 6 && !collector.test_case.frames.iter().any(|f| f.frame_number == frame_number) {
                let frame_data = FrameData {
                    frame_number,
                    mdct_coefficients: MdctData {
                        coefficients: Vec::new(),
                        l3_sb_sample: Vec::new(),
                    },
                    quantization: QuantizationData {
                        xrmax: 0,
                        max_bits: 0,
                        part2_3_length: 0,
                        quantizer_step_size: 0,
                        global_gain: 0,
                    },
                    bitstream: BitstreamData {
                        padding: 0,
                        bits_per_frame: 0,
                        written: 0,
                        slot_lag: 0.0,
                    },
                };
                collector.test_case.frames.push(frame_data);
            }
        }
    }
    
    /// Record MDCT coefficient
    pub fn record_mdct_coefficient(k: usize, value: i32) {
        let mut guard = TEST_DATA_COLLECTOR.lock().unwrap();
        if let Some(collector) = guard.as_mut() {
            if collector.current_frame <= 6 && k >= 15 && k <= 17 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    frame.mdct_coefficients.coefficients.push(value);
                }
            }
        }
    }
    
    /// Record l3_sb_sample value
    pub fn record_l3_sb_sample(ch: usize, value: i32) {
        let mut guard = TEST_DATA_COLLECTOR.lock().unwrap();
        if let Some(collector) = guard.as_mut() {
            if collector.current_frame <= 6 && ch == 0 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    frame.mdct_coefficients.l3_sb_sample.push(value);
                }
            }
        }
    }
    
    /// Record quantization data
    pub fn record_quantization(xrmax: i32, max_bits: i32, part2_3_length: u32, quantizer_step_size: i32, global_gain: u32) {
        let mut guard = TEST_DATA_COLLECTOR.lock().unwrap();
        if let Some(collector) = guard.as_mut() {
            if collector.current_frame <= 6 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    frame.quantization.xrmax = xrmax;
                    frame.quantization.max_bits = max_bits;
                    frame.quantization.part2_3_length = part2_3_length;
                    frame.quantization.quantizer_step_size = quantizer_step_size;
                    frame.quantization.global_gain = global_gain;
                }
            }
        }
    }
    
    /// Record bitstream data
    pub fn record_bitstream(padding: i32, bits_per_frame: i32, written: usize, slot_lag: f64) {
        let mut guard = TEST_DATA_COLLECTOR.lock().unwrap();
        if let Some(collector) = guard.as_mut() {
            if collector.current_frame <= 6 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    frame.bitstream.padding = padding;
                    frame.bitstream.bits_per_frame = bits_per_frame;
                    frame.bitstream.written = written;
                    frame.bitstream.slot_lag = slot_lag;
                }
            }
        }
    }
    
    /// Save collected data to JSON file
    pub fn save_to_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let guard = TEST_DATA_COLLECTOR.lock().unwrap();
        if let Some(collector) = guard.as_ref() {
            let json = serde_json::to_string_pretty(&collector.test_case)?;
            let mut file = File::create(filename)?;
            file.write_all(json.as_bytes())?;
            log::info!("Test data saved to: {}", filename);
            Ok(())
        } else {
            Err("No test data collector initialized".into())
        }
    }
    
    /// Load test data from JSON file
    pub fn load_from_file(filename: &str) -> Result<TestCaseData, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(filename)?;
        let test_case: TestCaseData = serde_json::from_str(&content)?;
        Ok(test_case)
    }
    
    /// Check if collection is enabled (for performance)
    pub fn is_collecting() -> bool {
        let guard = TEST_DATA_COLLECTOR.lock().unwrap();
        guard.is_some()
    }
}

/// Convenience functions for recording data
#[cfg(feature = "diagnostics")]
pub fn start_frame_collection(frame_number: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::start_frame(frame_number);
    }
}

#[cfg(feature = "diagnostics")]
pub fn record_mdct_coeff(k: usize, value: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_mdct_coefficient(k, value);
    }
}

#[cfg(feature = "diagnostics")]
pub fn record_sb_sample(ch: usize, value: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_l3_sb_sample(ch, value);
    }
}

#[cfg(feature = "diagnostics")]
pub fn record_quant_data(xrmax: i32, max_bits: i32, part2_3_length: u32, quantizer_step_size: i32, global_gain: u32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_quantization(xrmax, max_bits, part2_3_length, quantizer_step_size, global_gain);
    }
}

#[cfg(feature = "diagnostics")]
pub fn record_bitstream_data(padding: i32, bits_per_frame: i32, written: usize, slot_lag: f64) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_bitstream(padding, bits_per_frame, written, slot_lag);
    }
}

// High-level encoder interface for integration testing
#[cfg(feature = "diagnostics")]
use crate::error::EncodingResult;
#[cfg(feature = "diagnostics")]
use crate::encoder::{shine_initialise, shine_encode_buffer_interleaved, ShineConfig, ShineWave, ShineMpeg};
#[cfg(feature = "diagnostics")]
use crate::types::ShineGlobalConfig;

/// Channel mode enumeration
#[derive(Debug, Clone)]
pub enum ChannelMode {
    Mono,
    Stereo,
}

/// Complete frame encoding result for validation
#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub mdct_data: MdctData,
    pub quantization_data: QuantizationData,
    pub bitstream_data: BitstreamData,
    pub frame_data: Vec<u8>,
}

/// High-level MP3 encoder for integration testing
#[cfg(feature = "diagnostics")]
pub struct Encoder {
    config: Box<ShineGlobalConfig>,
}

#[cfg(feature = "diagnostics")]
impl Encoder {
    /// Create a new encoder with the given configuration
    pub fn new(encoding_config: EncodingConfig) -> EncodingResult<Self> {
        // Create shine configuration
        let shine_config = ShineConfig {
            wave: ShineWave {
                channels: encoding_config.channels,
                samplerate: encoding_config.sample_rate,
            },
            mpeg: ShineMpeg {
                mode: if encoding_config.channels == 1 { 3 } else { 1 }, // MPG_MD_MONO or MPG_MD_STEREO
                bitr: encoding_config.bitrate,
                emph: 0,
                copyright: 0,
                original: 1,
            },
        };

        // Initialize encoder
        let config = shine_initialise(&shine_config)?;

        Ok(Self { config })
    }

    /// Encode a frame and capture intermediate data
    pub fn encode_frame(&mut self, samples: &[i16]) -> EncodingResult<EncodedFrame> {
        // Prepare sample data
        let sample_ptr = samples.as_ptr();

        // Encode frame and immediately copy the data to avoid borrow issues
        let (frame_data_slice, written) = unsafe { shine_encode_buffer_interleaved(&mut self.config, sample_ptr)? };
        let frame_data = frame_data_slice.to_vec(); // Copy immediately

        // Now we can safely access self.config again
        let mdct_data = self.capture_mdct_data();
        let quantization_data = self.capture_quantization_data();

        // Create bitstream data
        let bitstream_data = BitstreamData {
            written,
            bits_per_frame: self.config.mpeg.bits_per_frame,
            slot_lag: self.config.mpeg.slot_lag,
            padding: self.config.mpeg.padding,
        };

        Ok(EncodedFrame {
            mdct_data,
            quantization_data,
            bitstream_data,
            frame_data,
        })
    }

    /// Capture MDCT coefficients from the encoder state
    fn capture_mdct_data(&self) -> MdctData {
        let mut coefficients = Vec::new();
        let mut l3_sb_sample = Vec::new();

        // Extract key MDCT coefficients (positions 17, 16, 15 from first channel/granule/band)
        // MDCT coefficients are stored as mdct_freq[ch][gr][band*18 + k]
        // We want band=0, k=17,16,15 from ch=0, gr=0 (in that order to match test data)
        if self.config.mpeg.granules_per_frame > 0 {
            for k in [17, 16, 15] {
                let index = 0 * 18 + k; // band=0, k=17,16,15
                if index < self.config.mdct_freq[0][0].len() {
                    let coeff = self.config.mdct_freq[0][0][index];
                    coefficients.push(coeff);
                    #[cfg(any(debug_assertions, feature = "diagnostics"))]
                    println!("[RUST DEBUG] Captured MDCT coeff k={}: {}", k, coeff);
                }
            }
        }

        // Extract l3_sb_sample data from first channel
        for gr in 0..std::cmp::min(self.config.mpeg.granules_per_frame as usize, 1) {
            for sb in 0..std::cmp::min(18, 3) { // Limit to first few subbands
                for i in 0..std::cmp::min(crate::types::SBLIMIT, 8) { // Limit samples
                    l3_sb_sample.push(self.config.l3_sb_sample[0][gr][sb][i]);
                }
            }
        }

        MdctData {
            coefficients,
            l3_sb_sample,
        }
    }

    /// Capture quantization parameters from the encoder state
    fn capture_quantization_data(&self) -> QuantizationData {
        // Get data from the first granule and channel
        let gr_info = &self.config.side_info.gr[0].ch[0].tt;

        QuantizationData {
            global_gain: gr_info.global_gain,
            part2_3_length: gr_info.part2_3_length,
            max_bits: self.config.mean_bits,
            xrmax: self.config.l3loop.xrmax,
            quantizer_step_size: gr_info.quantizer_step_size,
        }
    }
}