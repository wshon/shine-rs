//! Test data collection and validation for MP3 encoding
//!
//! This module provides functionality to collect key encoding parameters
//! during the encoding process and save them to JSON for later validation.
//! 
//! This module is only available when the "diagnostics" feature is enabled.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::collections::HashMap;
use std::thread;
use lazy_static::lazy_static;

lazy_static! {
    /// Thread-local frame counters for debugging consistency across modules
    static ref THREAD_FRAME_COUNTERS: Mutex<HashMap<std::thread::ThreadId, i32>> = Mutex::new(HashMap::new());
}
lazy_static! {
    /// Global test data collector - now supports multiple threads
    static ref TEST_DATA_COLLECTORS: Mutex<HashMap<std::thread::ThreadId, TestDataCollector>> = Mutex::new(HashMap::new());
}

/// Reset the frame counter for current thread (for testing)
pub fn reset_frame_counter() {
    let thread_id = thread::current().id();
    let mut counters = THREAD_FRAME_COUNTERS.lock().unwrap();
    counters.insert(thread_id, 0);
    
    // Also reset TestDataCollector if diagnostics feature is enabled
    TestDataCollector::reset();
}

/// Get the current frame number and increment the counter for current thread
pub fn get_next_frame_number() -> i32 {
    let thread_id = thread::current().id();
    let mut counters = THREAD_FRAME_COUNTERS.lock().unwrap();
    let counter = counters.entry(thread_id).or_insert(0);
    *counter += 1;
    *counter
}

/// Get the current frame number without incrementing for current thread
pub fn get_current_frame_number() -> i32 {
    let thread_id = thread::current().id();
    let counters = THREAD_FRAME_COUNTERS.lock().unwrap();
    *counters.get(&thread_id).unwrap_or(&0)
}


/// Frame-specific encoding data
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdctData {
    /// MDCT coefficients before aliasing reduction [k=17, k=16, k=15]
    /// These are the raw MDCT transform results
    pub coefficients_before_aliasing: Vec<i32>,
    
    /// MDCT coefficients after aliasing reduction [k=17, k=16, k=15]
    /// These are the final coefficients used in quantization
    pub coefficients_after_aliasing: Vec<i32>,
    
    /// Saved l3_sb_sample values for verification
    pub l3_sb_sample: Vec<i32>,
}

/// Quantization data
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
pub type TestDataSet = TestCaseData;

/// Test metadata
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



/// Test data collector implementation
#[derive(Debug)]
pub struct TestDataCollector {
    pub test_case: TestCaseData,
    pub current_frame: i32,
}

impl TestDataCollector {
    /// Initialize the test data collector for current thread
    pub fn initialize(metadata: TestMetadata, config: EncodingConfig) {
        let thread_id = thread::current().id();
        let collector = TestDataCollector {
            test_case: TestCaseData {
                metadata,
                config,
                frames: Vec::new(),
            },
            current_frame: 0,
        };
        
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        guard.insert(thread_id, collector);
    }
    
    /// Start collecting data for a new frame in current thread
    pub fn start_frame(frame_number: i32) {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get_mut(&thread_id) {
            collector.current_frame = frame_number;
            
            // Initialize frame data if this is a new frame within our collection range
            let frame_exists = collector.test_case.frames.iter().any(|f| f.frame_number == frame_number);
            
            if frame_number <= 6 && !frame_exists {
                let frame_data = FrameData {
                    frame_number,
                    mdct_coefficients: MdctData {
                        coefficients_before_aliasing: Vec::new(),
                        coefficients_after_aliasing: Vec::new(),
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
    
    /// Record MDCT coefficient before aliasing reduction for current thread
    pub fn record_mdct_coefficient_before_aliasing(k: usize, value: i32) {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get_mut(&thread_id) {
            if collector.current_frame <= 6 && k >= 15 && k <= 17 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    // Ensure the vector has the right size
                    if frame.mdct_coefficients.coefficients_before_aliasing.len() < 3 {
                        frame.mdct_coefficients.coefficients_before_aliasing.resize(3, 0);
                    }
                    // Store in order: k=17 at index 0, k=16 at index 1, k=15 at index 2
                    let index = 17 - k;
                    if index < 3 {
                        frame.mdct_coefficients.coefficients_before_aliasing[index] = value;
                    }
                }
            }
        }
    }
    
    /// Record MDCT coefficient after aliasing reduction for current thread
    pub fn record_mdct_coefficient_after_aliasing(k: usize, value: i32) {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get_mut(&thread_id) {
            if collector.current_frame <= 6 && k >= 15 && k <= 17 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    // Ensure the vector has the right size
                    if frame.mdct_coefficients.coefficients_after_aliasing.len() < 3 {
                        frame.mdct_coefficients.coefficients_after_aliasing.resize(3, 0);
                    }
                    // Store in order: k=17 at index 0, k=16 at index 1, k=15 at index 2
                    let index = 17 - k;
                    if index < 3 {
                        frame.mdct_coefficients.coefficients_after_aliasing[index] = value;
                    }
                }
            }
        }
    }
    
    /// Record l3_sb_sample value for current thread
    pub fn record_l3_sb_sample(ch: usize, value: i32) {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get_mut(&thread_id) {
            if collector.current_frame <= 6 && ch == 0 {
                if let Some(frame) = collector.test_case.frames.iter_mut()
                    .find(|f| f.frame_number == collector.current_frame) {
                    frame.mdct_coefficients.l3_sb_sample.push(value);
                }
            }
        }
    }
    
    /// Record quantization data for current thread
    pub fn record_quantization(xrmax: i32, max_bits: i32, part2_3_length: u32, quantizer_step_size: i32, global_gain: u32) {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get_mut(&thread_id) {
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
    
    /// Record bitstream data for current thread
    pub fn record_bitstream(padding: i32, bits_per_frame: i32, written: usize, slot_lag: f64) {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get_mut(&thread_id) {
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
    
    /// Save collected data to JSON file for current thread
    pub fn save_to_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let thread_id = thread::current().id();
        let guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get(&thread_id) {
            let json = serde_json::to_string_pretty(&collector.test_case)?;
            let mut file = File::create(filename)?;
            file.write_all(json.as_bytes())?;
            log::info!("Test data saved to: {}", filename);
            Ok(())
        } else {
            Err("No test data collector initialized for current thread".into())
        }
    }
    
    /// Load test data from JSON file
    pub fn load_from_file(filename: &str) -> Result<TestCaseData, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(filename)?;
        let test_case: TestCaseData = serde_json::from_str(&content)?;
        Ok(test_case)
    }
    
    /// Get current frame data from the collector for current thread
    pub fn get_current_frame_data() -> Option<FrameData> {
        let thread_id = thread::current().id();
        let guard = TEST_DATA_COLLECTORS.lock().unwrap();
        if let Some(collector) = guard.get(&thread_id) {
            if let Some(frame) = collector.test_case.frames.iter()
                .find(|f| f.frame_number == collector.current_frame) {
                return Some(frame.clone());
            }
        }
        None
    }
    
    /// Check if collection is enabled for current thread (for performance)
    pub fn is_collecting() -> bool {
        let thread_id = thread::current().id();
        let guard = TEST_DATA_COLLECTORS.lock().unwrap();
        guard.contains_key(&thread_id)
    }
    
    /// Reset the test data collector for current thread (for testing)
    pub fn reset() {
        let thread_id = thread::current().id();
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        guard.remove(&thread_id);
    }
    
    /// Reset all test data collectors (for global cleanup)
    pub fn reset_all() {
        let mut guard = TEST_DATA_COLLECTORS.lock().unwrap();
        guard.clear();
    }
}

pub fn start_frame_collection(frame_number: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::start_frame(frame_number);
    }
}

pub fn record_mdct_coeff_before_aliasing(k: usize, value: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_mdct_coefficient_before_aliasing(k, value);
    }
}

pub fn record_mdct_coeff_after_aliasing(k: usize, value: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_mdct_coefficient_after_aliasing(k, value);
    }
}

pub fn record_sb_sample(ch: usize, value: i32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_l3_sb_sample(ch, value);
    }
}

pub fn record_quant_data(xrmax: i32, max_bits: i32, part2_3_length: u32, quantizer_step_size: i32, global_gain: u32) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_quantization(xrmax, max_bits, part2_3_length, quantizer_step_size, global_gain);
    }
}

pub fn record_bitstream_data(padding: i32, bits_per_frame: i32, written: usize, slot_lag: f64) {
    if TestDataCollector::is_collecting() {
        TestDataCollector::record_bitstream(padding, bits_per_frame, written, slot_lag);
    }
}

// High-level encoder interface for integration testing
use crate::error::EncodingResult;
use crate::encoder::{shine_initialise, shine_encode_buffer_interleaved, ShineConfig, ShineWave, ShineMpeg};
use crate::types::ShineGlobalConfig;

/// Channel mode enumeration
#[derive(Debug, Clone)]
pub enum ChannelMode {
    Mono,
    Stereo,
}

/// Complete frame encoding result for validation
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub mdct_data: MdctData,
    pub quantization_data: QuantizationData,
    pub bitstream_data: BitstreamData,
    pub frame_data: Vec<u8>,
}

/// High-level MP3 encoder for integration testing
pub struct Encoder {
    config: Box<ShineGlobalConfig>,
}

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
        // Note: Frame collection is started in shine_encode_buffer_interleaved
        // No need to start it here to avoid duplicate calls

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
        // Try to get data from the global test data collector first
                {
            if let Some(frame_data) = TestDataCollector::get_current_frame_data() {
                return frame_data.mdct_coefficients;
            }
        }
        
        // Fallback: extract l3_sb_sample data from encoder state
        let mut l3_sb_sample = Vec::new();
        for gr in 0..std::cmp::min(self.config.mpeg.granules_per_frame as usize, 1) {
            for sb in 0..std::cmp::min(18, 3) { // Limit to first few subbands
                for i in 0..std::cmp::min(crate::types::SBLIMIT, 8) { // Limit samples
                    l3_sb_sample.push(self.config.l3_sb_sample[0][gr][sb][i]);
                }
            }
        }

        MdctData {
            coefficients_before_aliasing: Vec::new(),
            coefficients_after_aliasing: Vec::new(),
            l3_sb_sample,
        }
    }

    /// Capture quantization parameters from the encoder state
    fn capture_quantization_data(&self) -> QuantizationData {
        // Try to get data from the global test data collector first
                {
            if let Some(frame_data) = TestDataCollector::get_current_frame_data() {
                println!("[RUST DEBUG] Using TestDataCollector data: global_gain={}", frame_data.quantization.global_gain);
                return frame_data.quantization;
            } else {
                println!("[RUST DEBUG] TestDataCollector has no current frame data, using fallback");
            }
        }
        
        // Fallback: get data from the first granule and channel
        let gr_info = &self.config.side_info.gr[0].ch[0].tt;
        
        println!("[RUST DEBUG] Using fallback data: global_gain={}", gr_info.global_gain);

        QuantizationData {
            global_gain: gr_info.global_gain,
            part2_3_length: gr_info.part2_3_length,
            max_bits: self.config.mean_bits, // This is a fallback - should use collected data
            xrmax: self.config.l3loop.xrmax,
            quantizer_step_size: gr_info.quantizer_step_size,
        }
    }
}