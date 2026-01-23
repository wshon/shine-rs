//! Data flow monitoring and validation for MP3 encoding pipeline
//!
//! This module provides comprehensive monitoring and validation of the MP3 encoding
//! pipeline to detect issues like all-zero main data, incorrect quantization, and
//! other encoding problems. It implements the data flow validation requirements
//! from task 4.1.

use crate::quantization::GranuleInfo;
use crate::error::{EncodingError, EncodingResult};
use std::collections::HashMap;

/// Data flow validation thresholds based on task 4.1 requirements
pub struct ValidationThresholds {
    /// PCM input: non-zero sample ratio > 1%
    pub pcm_nonzero_ratio_min: f32,
    /// Subband output: non-zero coefficient ratio > 10%
    pub subband_nonzero_ratio_min: f32,
    /// MDCT coefficients: non-zero coefficient ratio (reasonable distribution)
    pub mdct_nonzero_ratio_min: f32,
    /// Quantized coefficients: non-zero coefficient ratio > 5%
    pub quantized_nonzero_ratio_min: f32,
    /// Quantized coefficients: big_values must be < 288
    pub big_values_max: u32,
    /// Huffman encoding: minimum bits generated for non-zero coefficients
    pub huffman_min_bits_per_nonzero: f32,
    /// Bitstream: main data non-zero bytes > 50%
    pub bitstream_main_data_nonzero_min: f32,
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            pcm_nonzero_ratio_min: 0.01,      // 1%
            subband_nonzero_ratio_min: 0.10,   // 10%
            mdct_nonzero_ratio_min: 0.05,      // 5%
            quantized_nonzero_ratio_min: 0.05, // 5%
            big_values_max: 288,               // MP3 standard limit
            huffman_min_bits_per_nonzero: 1.0, // At least 1 bit per non-zero coefficient
            bitstream_main_data_nonzero_min: 0.50, // 50%
        }
    }
}

/// Statistics for each encoding stage
#[derive(Debug, Clone, Default)]
pub struct StageStats {
    pub total_samples: usize,
    pub nonzero_samples: usize,
    pub max_value: i32,
    pub min_value: i32,
    pub energy: f64,
    pub dynamic_range: f32,
}

impl StageStats {
    pub fn nonzero_ratio(&self) -> f32 {
        if self.total_samples == 0 {
            0.0
        } else {
            self.nonzero_samples as f32 / self.total_samples as f32
        }
    }
    
    pub fn calculate_dynamic_range(&self) -> f32 {
        if self.max_value == self.min_value {
            0.0
        } else {
            (self.max_value - self.min_value) as f32
        }
    }
}

/// Validation issues detected during encoding
#[derive(Debug, Clone)]
pub enum ValidationIssue {
    /// PCM input validation issues
    PcmInputIssue {
        issue: String,
        nonzero_ratio: f32,
        expected_min: f32,
    },
    /// Subband filter output issues
    SubbandIssue {
        issue: String,
        nonzero_ratio: f32,
        expected_min: f32,
    },
    /// MDCT transform output issues
    MdctIssue {
        issue: String,
        nonzero_ratio: f32,
        low_freq_energy: f64,
    },
    /// Quantization issues
    QuantizationIssue {
        issue: String,
        nonzero_ratio: f32,
        big_values: u32,
        global_gain: u32,
    },
    /// Huffman encoding issues
    HuffmanIssue {
        issue: String,
        bits_generated: usize,
        nonzero_coeffs: usize,
        bits_per_coeff: f32,
    },
    /// Bitstream writing issues
    BitstreamIssue {
        issue: String,
        main_data_size: usize,
        nonzero_bytes: usize,
        nonzero_ratio: f32,
    },
}

/// Comprehensive data flow monitor for the MP3 encoding pipeline
pub struct DataFlowMonitor {
    /// Validation thresholds
    thresholds: ValidationThresholds,
    /// Enable detailed logging
    verbose: bool,
    /// Statistics for each stage
    pcm_stats: StageStats,
    subband_stats: StageStats,
    mdct_stats: StageStats,
    quantized_stats: StageStats,
    huffman_stats: HashMap<String, usize>,
    bitstream_stats: StageStats,
    /// Detected validation issues
    issues: Vec<ValidationIssue>,
    /// Frame counter
    frame_count: usize,
}

impl DataFlowMonitor {
    /// Create a new data flow monitor
    pub fn new(verbose: bool) -> Self {
        Self {
            thresholds: ValidationThresholds::default(),
            verbose,
            pcm_stats: StageStats::default(),
            subband_stats: StageStats::default(),
            mdct_stats: StageStats::default(),
            quantized_stats: StageStats::default(),
            huffman_stats: HashMap::new(),
            bitstream_stats: StageStats::default(),
            issues: Vec::new(),
            frame_count: 0,
        }
    }
    
    /// Create monitor with custom thresholds
    pub fn with_thresholds(thresholds: ValidationThresholds, verbose: bool) -> Self {
        Self {
            thresholds,
            verbose,
            pcm_stats: StageStats::default(),
            subband_stats: StageStats::default(),
            mdct_stats: StageStats::default(),
            quantized_stats: StageStats::default(),
            huffman_stats: HashMap::new(),
            bitstream_stats: StageStats::default(),
            issues: Vec::new(),
            frame_count: 0,
        }
    }
    
    /// Monitor PCM input data
    pub fn monitor_pcm_input(&mut self, pcm_data: &[i16]) -> EncodingResult<()> {
        self.frame_count += 1;
        
        let mut stats = StageStats::default();
        stats.total_samples = pcm_data.len();
        stats.nonzero_samples = pcm_data.iter().filter(|&&x| x != 0).count();
        stats.max_value = pcm_data.iter().map(|&x| x as i32).max().unwrap_or(0);
        stats.min_value = pcm_data.iter().map(|&x| x as i32).min().unwrap_or(0);
        stats.energy = pcm_data.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / pcm_data.len() as f64;
        stats.dynamic_range = stats.calculate_dynamic_range();
        
        if self.verbose {
            println!("PCM Input [Frame {}]: {} samples, {:.1}% non-zero, range: {} to {}, energy: {:.2}", 
                     self.frame_count, stats.total_samples, stats.nonzero_ratio() * 100.0, 
                     stats.min_value, stats.max_value, stats.energy);
        }
        
        // Validate PCM input according to task 4.1 requirements
        let nonzero_ratio = stats.nonzero_ratio();
        if nonzero_ratio < self.thresholds.pcm_nonzero_ratio_min {
            let issue = ValidationIssue::PcmInputIssue {
                issue: format!("PCM input has insufficient non-zero samples: {:.1}% < {:.1}%", 
                              nonzero_ratio * 100.0, self.thresholds.pcm_nonzero_ratio_min * 100.0),
                nonzero_ratio,
                expected_min: self.thresholds.pcm_nonzero_ratio_min,
            };
            self.issues.push(issue);
        }
        
        // Check dynamic range
        if stats.dynamic_range < 100.0 && nonzero_ratio > 0.01 {
            let issue = ValidationIssue::PcmInputIssue {
                issue: format!("PCM input has very low dynamic range: {:.1}", stats.dynamic_range),
                nonzero_ratio,
                expected_min: self.thresholds.pcm_nonzero_ratio_min,
            };
            self.issues.push(issue);
        }
        
        self.pcm_stats = stats;
        Ok(())
    }
    
    /// Monitor subband filter output
    pub fn monitor_subband_output(&mut self, subband_data: &[i32; 32], channel: usize) -> EncodingResult<()> {
        let mut stats = StageStats::default();
        stats.total_samples = subband_data.len();
        stats.nonzero_samples = subband_data.iter().filter(|&&x| x != 0).count();
        stats.max_value = *subband_data.iter().max().unwrap_or(&0);
        stats.min_value = *subband_data.iter().min().unwrap_or(&0);
        stats.energy = subband_data.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / subband_data.len() as f64;
        stats.dynamic_range = stats.calculate_dynamic_range();
        
        if self.verbose {
            println!("Subband Output [Frame {}, Ch {}]: {} coeffs, {:.1}% non-zero, range: {} to {}, energy: {:.2}", 
                     self.frame_count, channel, stats.total_samples, stats.nonzero_ratio() * 100.0, 
                     stats.min_value, stats.max_value, stats.energy);
        }
        
        // Validate subband output according to task 4.1 requirements
        let nonzero_ratio = stats.nonzero_ratio();
        if nonzero_ratio < self.thresholds.subband_nonzero_ratio_min && self.pcm_stats.nonzero_ratio() > 0.01 {
            let issue = ValidationIssue::SubbandIssue {
                issue: format!("Subband output has insufficient non-zero coefficients: {:.1}% < {:.1}%", 
                              nonzero_ratio * 100.0, self.thresholds.subband_nonzero_ratio_min * 100.0),
                nonzero_ratio,
                expected_min: self.thresholds.subband_nonzero_ratio_min,
            };
            self.issues.push(issue);
        }
        
        // Check energy distribution (low frequencies should typically have more energy)
        let low_freq_energy: f64 = subband_data[0..8].iter().map(|&x| (x as f64).powi(2)).sum();
        let total_energy: f64 = subband_data.iter().map(|&x| (x as f64).powi(2)).sum();
        
        if total_energy > 0.0 && low_freq_energy / total_energy < 0.1 && self.pcm_stats.energy > 1000.0 {
            let issue = ValidationIssue::SubbandIssue {
                issue: format!("Subband output has unusual energy distribution: low freq {:.1}% of total", 
                              (low_freq_energy / total_energy) * 100.0),
                nonzero_ratio,
                expected_min: self.thresholds.subband_nonzero_ratio_min,
            };
            self.issues.push(issue);
        }
        
        self.subband_stats = stats;
        Ok(())
    }
    
    /// Monitor MDCT transform output
    pub fn monitor_mdct_output(&mut self, mdct_coeffs: &[i32; 576], channel: usize, granule: usize) -> EncodingResult<()> {
        let mut stats = StageStats::default();
        stats.total_samples = mdct_coeffs.len();
        stats.nonzero_samples = mdct_coeffs.iter().filter(|&&x| x != 0).count();
        stats.max_value = *mdct_coeffs.iter().max().unwrap_or(&0);
        stats.min_value = *mdct_coeffs.iter().min().unwrap_or(&0);
        stats.energy = mdct_coeffs.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / mdct_coeffs.len() as f64;
        stats.dynamic_range = stats.calculate_dynamic_range();
        
        // Calculate low frequency energy (first 64 coefficients)
        let low_freq_energy: f64 = mdct_coeffs[0..64.min(mdct_coeffs.len())].iter()
            .map(|&x| (x as f64).powi(2)).sum();
        
        if self.verbose {
            println!("MDCT Output [Frame {}, Ch {}, Gr {}]: {} coeffs, {:.1}% non-zero, range: {} to {}, energy: {:.2}, low_freq: {:.2}", 
                     self.frame_count, channel, granule, stats.total_samples, stats.nonzero_ratio() * 100.0, 
                     stats.min_value, stats.max_value, stats.energy, low_freq_energy);
        }
        
        // Validate MDCT output according to task 4.1 requirements
        let nonzero_ratio = stats.nonzero_ratio();
        if nonzero_ratio < self.thresholds.mdct_nonzero_ratio_min && self.subband_stats.nonzero_ratio() > 0.05 {
            let issue = ValidationIssue::MdctIssue {
                issue: format!("MDCT output has insufficient non-zero coefficients: {:.1}% < {:.1}%", 
                              nonzero_ratio * 100.0, self.thresholds.mdct_nonzero_ratio_min * 100.0),
                nonzero_ratio,
                low_freq_energy,
            };
            self.issues.push(issue);
        }
        
        // Check frequency domain energy distribution
        if stats.energy > 0.0 && low_freq_energy / (stats.energy * stats.total_samples as f64) < 0.1 && self.subband_stats.energy > 1000.0 {
            let issue = ValidationIssue::MdctIssue {
                issue: format!("MDCT output has unusual frequency distribution: low freq energy too low"),
                nonzero_ratio,
                low_freq_energy,
            };
            self.issues.push(issue);
        }
        
        self.mdct_stats = stats;
        Ok(())
    }
    
    /// Monitor quantization output
    pub fn monitor_quantization_output(&mut self, quantized: &[i32; 576], info: &GranuleInfo, channel: usize, granule: usize) -> EncodingResult<()> {
        let mut stats = StageStats::default();
        stats.total_samples = quantized.len();
        stats.nonzero_samples = quantized.iter().filter(|&&x| x != 0).count();
        stats.max_value = *quantized.iter().max().unwrap_or(&0);
        stats.min_value = *quantized.iter().min().unwrap_or(&0);
        stats.energy = quantized.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / quantized.len() as f64;
        stats.dynamic_range = stats.calculate_dynamic_range();
        
        if self.verbose {
            println!("Quantization [Frame {}, Ch {}, Gr {}]: {} coeffs, {:.1}% non-zero, big_values: {}, global_gain: {}, range: {} to {}", 
                     self.frame_count, channel, granule, stats.total_samples, stats.nonzero_ratio() * 100.0, 
                     info.big_values, info.global_gain, stats.min_value, stats.max_value);
        }
        
        // Validate quantization output according to task 4.1 requirements
        let nonzero_ratio = stats.nonzero_ratio();
        
        // Check non-zero coefficient ratio
        if nonzero_ratio < self.thresholds.quantized_nonzero_ratio_min && self.mdct_stats.nonzero_ratio() > 0.05 {
            let issue = ValidationIssue::QuantizationIssue {
                issue: format!("Quantized coefficients have insufficient non-zero values: {:.1}% < {:.1}%", 
                              nonzero_ratio * 100.0, self.thresholds.quantized_nonzero_ratio_min * 100.0),
                nonzero_ratio,
                big_values: info.big_values,
                global_gain: info.global_gain,
            };
            self.issues.push(issue);
        }
        
        // Check big_values limit (critical MP3 standard requirement)
        if info.big_values > self.thresholds.big_values_max {
            let issue = ValidationIssue::QuantizationIssue {
                issue: format!("big_values exceeds MP3 standard limit: {} > {}", 
                              info.big_values, self.thresholds.big_values_max),
                nonzero_ratio,
                big_values: info.big_values,
                global_gain: info.global_gain,
            };
            self.issues.push(issue);
        }
        
        // Check for excessive quantization (all coefficients quantized to zero)
        if nonzero_ratio == 0.0 && self.mdct_stats.nonzero_ratio() > 0.1 {
            let issue = ValidationIssue::QuantizationIssue {
                issue: format!("All coefficients quantized to zero despite non-zero MDCT input (global_gain: {})", 
                              info.global_gain),
                nonzero_ratio,
                big_values: info.big_values,
                global_gain: info.global_gain,
            };
            self.issues.push(issue);
        }
        
        self.quantized_stats = stats;
        Ok(())
    }
    
    /// Monitor Huffman encoding output
    pub fn monitor_huffman_output(&mut self, bits_generated: usize, nonzero_coeffs: usize, channel: usize, granule: usize) -> EncodingResult<()> {
        let key = format!("ch{}_gr{}", channel, granule);
        self.huffman_stats.insert(key.clone(), bits_generated);
        
        let bits_per_coeff = if nonzero_coeffs > 0 {
            bits_generated as f32 / nonzero_coeffs as f32
        } else {
            0.0
        };
        
        if self.verbose {
            println!("Huffman Encoding [Frame {}, Ch {}, Gr {}]: {} bits for {} non-zero coeffs ({:.2} bits/coeff)", 
                     self.frame_count, channel, granule, bits_generated, nonzero_coeffs, bits_per_coeff);
        }
        
        // Validate Huffman encoding according to task 4.1 requirements
        if nonzero_coeffs > 0 && bits_per_coeff < self.thresholds.huffman_min_bits_per_nonzero {
            let issue = ValidationIssue::HuffmanIssue {
                issue: format!("Huffman encoding generated insufficient bits: {:.2} bits/coeff < {:.2}", 
                              bits_per_coeff, self.thresholds.huffman_min_bits_per_nonzero),
                bits_generated,
                nonzero_coeffs,
                bits_per_coeff,
            };
            self.issues.push(issue);
        }
        
        // Check for no bits generated despite non-zero coefficients
        if bits_generated == 0 && nonzero_coeffs > 0 {
            let issue = ValidationIssue::HuffmanIssue {
                issue: format!("No bits generated despite {} non-zero coefficients", nonzero_coeffs),
                bits_generated,
                nonzero_coeffs,
                bits_per_coeff,
            };
            self.issues.push(issue);
        }
        
        Ok(())
    }
    
    /// Monitor bitstream writing output
    pub fn monitor_bitstream_output(&mut self, frame_data: &[u8], main_data_start: usize, main_data_len: usize) -> EncodingResult<()> {
        let mut stats = StageStats::default();
        
        // Analyze main data region
        if main_data_start < frame_data.len() {
            let main_data_end = (main_data_start + main_data_len).min(frame_data.len());
            let main_data = &frame_data[main_data_start..main_data_end];
            
            stats.total_samples = main_data.len();
            stats.nonzero_samples = main_data.iter().filter(|&&x| x != 0).count();
            stats.max_value = main_data.iter().map(|&x| x as i32).max().unwrap_or(0);
            stats.min_value = main_data.iter().map(|&x| x as i32).min().unwrap_or(0);
        }
        
        if self.verbose {
            println!("Bitstream Output [Frame {}]: {} total bytes, main data: {} bytes ({:.1}% non-zero)", 
                     self.frame_count, frame_data.len(), stats.total_samples, stats.nonzero_ratio() * 100.0);
        }
        
        // Validate bitstream output according to task 4.1 requirements
        let nonzero_ratio = stats.nonzero_ratio();
        if stats.total_samples > 0 && nonzero_ratio < self.thresholds.bitstream_main_data_nonzero_min {
            // Only flag as issue if we had significant Huffman encoding
            let total_huffman_bits: usize = self.huffman_stats.values().sum();
            if total_huffman_bits > 100 { // Threshold for significant encoding
                let issue = ValidationIssue::BitstreamIssue {
                    issue: format!("Main data region has insufficient non-zero bytes: {:.1}% < {:.1}%", 
                                  nonzero_ratio * 100.0, self.thresholds.bitstream_main_data_nonzero_min * 100.0),
                    main_data_size: stats.total_samples,
                    nonzero_bytes: stats.nonzero_samples,
                    nonzero_ratio,
                };
                self.issues.push(issue);
            }
        }
        
        // Check for completely empty main data
        if stats.total_samples > 0 && stats.nonzero_samples == 0 {
            let total_huffman_bits: usize = self.huffman_stats.values().sum();
            if total_huffman_bits > 0 {
                let issue = ValidationIssue::BitstreamIssue {
                    issue: format!("Main data is completely zero despite {} Huffman bits generated", total_huffman_bits),
                    main_data_size: stats.total_samples,
                    nonzero_bytes: stats.nonzero_samples,
                    nonzero_ratio,
                };
                self.issues.push(issue);
            }
        }
        
        self.bitstream_stats = stats;
        Ok(())
    }
    
    /// Get all detected validation issues
    pub fn get_issues(&self) -> &[ValidationIssue] {
        &self.issues
    }
    
    /// Check if any critical issues were detected
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|issue| match issue {
            ValidationIssue::QuantizationIssue { big_values, .. } => *big_values > 288,
            ValidationIssue::BitstreamIssue { nonzero_ratio, .. } => *nonzero_ratio < 0.1,
            _ => false,
        })
    }
    
    /// Generate a comprehensive validation report
    pub fn generate_report(&self) -> ValidationReport {
        ValidationReport {
            frame_count: self.frame_count,
            pcm_stats: self.pcm_stats.clone(),
            subband_stats: self.subband_stats.clone(),
            mdct_stats: self.mdct_stats.clone(),
            quantized_stats: self.quantized_stats.clone(),
            bitstream_stats: self.bitstream_stats.clone(),
            total_huffman_bits: self.huffman_stats.values().sum(),
            issues: self.issues.clone(),
            has_critical_issues: self.has_critical_issues(),
        }
    }
    
    /// Reset monitor for new encoding session
    pub fn reset(&mut self) {
        self.pcm_stats = StageStats::default();
        self.subband_stats = StageStats::default();
        self.mdct_stats = StageStats::default();
        self.quantized_stats = StageStats::default();
        self.huffman_stats.clear();
        self.bitstream_stats = StageStats::default();
        self.issues.clear();
        self.frame_count = 0;
    }
}

/// Comprehensive validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub frame_count: usize,
    pub pcm_stats: StageStats,
    pub subband_stats: StageStats,
    pub mdct_stats: StageStats,
    pub quantized_stats: StageStats,
    pub bitstream_stats: StageStats,
    pub total_huffman_bits: usize,
    pub issues: Vec<ValidationIssue>,
    pub has_critical_issues: bool,
}

impl ValidationReport {
    /// Print a formatted report to stdout
    pub fn print_report(&self) {
        println!("\n=== Data Flow Validation Report ===");
        println!("Frames processed: {}", self.frame_count);
        
        println!("\n--- Stage Statistics ---");
        println!("PCM Input:      {:.1}% non-zero, energy: {:.2}", 
                 self.pcm_stats.nonzero_ratio() * 100.0, self.pcm_stats.energy);
        println!("Subband:        {:.1}% non-zero, energy: {:.2}", 
                 self.subband_stats.nonzero_ratio() * 100.0, self.subband_stats.energy);
        println!("MDCT:           {:.1}% non-zero, energy: {:.2}", 
                 self.mdct_stats.nonzero_ratio() * 100.0, self.mdct_stats.energy);
        println!("Quantized:      {:.1}% non-zero, energy: {:.2}", 
                 self.quantized_stats.nonzero_ratio() * 100.0, self.quantized_stats.energy);
        println!("Bitstream:      {:.1}% non-zero main data, {} total Huffman bits", 
                 self.bitstream_stats.nonzero_ratio() * 100.0, self.total_huffman_bits);
        
        if !self.issues.is_empty() {
            println!("\n--- Validation Issues ({}) ---", self.issues.len());
            for (i, issue) in self.issues.iter().enumerate() {
                println!("{}. {:?}", i + 1, issue);
            }
        } else {
            println!("\n✅ No validation issues detected");
        }
        
        if self.has_critical_issues {
            println!("\n❌ CRITICAL ISSUES DETECTED - Encoding pipeline has serious problems");
        } else {
            println!("\n✅ No critical issues detected");
        }
    }
}