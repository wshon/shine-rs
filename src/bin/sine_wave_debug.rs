//! Debug sine wave encoding to understand why big_values exceeds 288
//!
//! This tool analyzes the sine wave encoding process step by step.

use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};
use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

fn main() {
    println!("Sine Wave Debug Tool");
    println!("===================");
    
    // Create the same sine wave as in big_values_debug
    let samples_per_frame = 1152;
    let sine_data: Vec<i16> = (0..samples_per_frame)
        .map(|i| {
            let t = i as f64 / 44100.0;
            (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()) as i16
        })
        .collect();
    
    println!("Generated sine wave: {} samples", sine_data.len());
    println!("Sample range: {} to {}", 
             sine_data.iter().min().unwrap(), 
             sine_data.iter().max().unwrap());
    
    // Test different quantization step sizes to see the effect on big_values
    println!("\n=== Testing Different Quantization Step Sizes ===");
    
    // Create a simple MDCT coefficient array for testing
    // (In real encoding, this would come from MDCT transform)
    let mut test_coeffs = [0i32; GRANULE_SIZE];
    
    // Simulate what a sine wave might look like in MDCT domain
    // A 440Hz sine wave should have energy concentrated around certain bins
    let fundamental_bin = (440.0 * GRANULE_SIZE as f64 / 44100.0) as usize;
    println!("Fundamental frequency bin: {}", fundamental_bin);
    
    // Add energy at fundamental and harmonics
    test_coeffs[fundamental_bin] = 10000;
    test_coeffs[fundamental_bin * 2] = 5000;
    test_coeffs[fundamental_bin * 3] = 2500;
    test_coeffs[fundamental_bin * 4] = 1250;
    
    // Add some spread around each harmonic (realistic spectral leakage)
    for harmonic in 1..=4 {
        let bin = fundamental_bin * harmonic;
        if bin < GRANULE_SIZE - 2 {
            test_coeffs[bin - 1] = test_coeffs[bin] / 4;
            test_coeffs[bin + 1] = test_coeffs[bin] / 4;
        }
    }
    
    let non_zero_count = test_coeffs.iter().filter(|&&x| x != 0).count();
    println!("Test coefficients: {} non-zero values", non_zero_count);
    
    let quantizer = QuantizationLoop::new();
    
    // Test quantization with different step sizes
    for step_size in [-120, -100, -80, -60, -40, -20, 0, 20] {
        let mut quantized = [0i32; GRANULE_SIZE];
        let max_quantized = quantizer.quantize(&mut quantized, step_size, &test_coeffs);
        
        if max_quantized <= 8192 {
            let mut side_info = GranuleInfo::default();
            quantizer.calculate_run_length(&quantized, &mut side_info);
            
            let quantized_non_zero = quantized.iter().filter(|&&x| x != 0).count();
            
            println!("Step size {}: max_quantized={}, quantized_non_zero={}, big_values={}, count1={}", 
                    step_size, max_quantized, quantized_non_zero, side_info.big_values, side_info.count1);
            
            if side_info.big_values <= 288 {
                println!("  ✅ big_values within limits");
            } else {
                println!("  ❌ big_values too large: {} > 288", side_info.big_values);
            }
        } else {
            println!("Step size {}: max_quantized={} (too large)", step_size, max_quantized);
        }
    }
    
    // Test the complete quantization process
    println!("\n=== Testing Complete Quantization Process ===");
    
    let mut quantizer = QuantizationLoop::new();
    let mut side_info = GranuleInfo::default();
    let mut output = [0i32; GRANULE_SIZE];
    
    // Try different target bit rates
    for target_bits in [1000, 1500, 2000, 2500, 3000] {
        let mut temp_side_info = GranuleInfo::default();
        let mut temp_output = [0i32; GRANULE_SIZE];
        
        match quantizer.quantize_and_encode(&test_coeffs, target_bits, &mut temp_side_info, &mut temp_output, 44100) {
            Ok(bits_used) => {
                println!("Target bits {}: bits_used={}, big_values={}, global_gain={}, quantizer_step_size={}", 
                        target_bits, bits_used, temp_side_info.big_values, temp_side_info.global_gain, temp_side_info.quantizer_step_size);
                
                if temp_side_info.big_values <= 288 {
                    println!("  ✅ big_values within limits");
                } else {
                    println!("  ❌ big_values too large: {} > 288", temp_side_info.big_values);
                }
            },
            Err(e) => {
                println!("Target bits {}: FAILED - {:?}", target_bits, e);
            }
        }
    }
}