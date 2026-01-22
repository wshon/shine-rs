//! Quantization tracing test
//!
//! This test traces the quantization process to see exactly
//! what happens with calculate_run_length for all-zero inputs.

use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};
use rust_mp3_encoder::config::{Config, WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

#[test]
fn test_quantization_trace_all_zeros() {
    println!("\n🔍 Tracing quantization process with all-zero MDCT coefficients");
    
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
    
    let mut quantizer = QuantizationLoop::new();
    
    // Test with all-zero MDCT coefficients (this is what should come from MDCT of silence)
    let mdct_coeffs = [0i32; GRANULE_SIZE];
    let mut granule_info = GranuleInfo::default();
    let mut quantized_output = [0i32; GRANULE_SIZE];
    
    println!("Input: all-zero MDCT coefficients");
    
    // Check that input is really all zeros
    let non_zero_count = mdct_coeffs.iter().filter(|&&x| x != 0).count();
    println!("Non-zero MDCT coefficients: {}", non_zero_count);
    
    let max_bits = 1000;
    
    match quantizer.quantize_and_encode(&mdct_coeffs, max_bits, &mut granule_info, &mut quantized_output) {
        Ok(bits_used) => {
            println!("Quantization succeeded: {} bits used", bits_used);
            
            println!("Final granule info:");
            println!("  big_values: {}", granule_info.big_values);
            println!("  count1: {}", granule_info.count1);
            println!("  quantizer_step_size: {}", granule_info.quantizer_step_size);
            println!("  global_gain: {}", granule_info.global_gain);
            
            // Check the quantized output
            let non_zero_quantized = quantized_output.iter().filter(|&&x| x != 0).count();
            println!("Non-zero quantized coefficients: {}", non_zero_quantized);
            
            // This is the key test: for all-zero MDCT input, we should get count1=0
            if granule_info.count1 == 0 {
                println!("✓ Count1 is correctly 0 for all-zero input");
            } else {
                println!("❌ PROBLEM: Count1 is {} for all-zero input (should be 0)", granule_info.count1);
                
                // Let's manually verify what calculate_run_length should produce
                println!("\nManual verification of calculate_run_length logic:");
                
                let mut i = GRANULE_SIZE;
                
                // Count trailing zero pairs
                while i > 1 {
                    if quantized_output[i - 1] == 0 && quantized_output[i - 2] == 0 {
                        i -= 2;
                    } else {
                        break;
                    }
                }
                
                println!("After zero pair removal: i = {}", i);
                
                // Count quadruples
                let mut count1 = 0;
                while i > 3 {
                    if quantized_output[i - 1] <= 1 && quantized_output[i - 2] <= 1 &&
                       quantized_output[i - 3] <= 1 && quantized_output[i - 4] <= 1 {
                        count1 += 1;
                        i -= 4;
                    } else {
                        break;
                    }
                }
                
                let big_values = i >> 1;
                
                println!("Manual calculation results:");
                println!("  big_values: {}", big_values);
                println!("  count1: {}", count1);
                
                if count1 != granule_info.count1 {
                    println!("❌ Mismatch between manual calculation and actual result!");
                    println!("   Manual: count1={}, big_values={}", count1, big_values);
                    println!("   Actual: count1={}, big_values={}", granule_info.count1, granule_info.big_values);
                } else {
                    println!("✓ Manual calculation matches actual result");
                    println!("   The issue must be elsewhere in the pipeline");
                }
            }
            
            // Show some of the quantized coefficients for debugging
            println!("\nFirst 20 quantized coefficients:");
            for i in 0..20 {
                print!("{} ", quantized_output[i]);
            }
            println!();
            
            println!("Last 20 quantized coefficients:");
            for i in (GRANULE_SIZE-20)..GRANULE_SIZE {
                print!("{} ", quantized_output[i]);
            }
            println!();
        },
        Err(e) => {
            println!("❌ Quantization failed: {:?}", e);
        }
    }
}

#[test]
fn test_quantization_trace_small_values() {
    println!("\n🔍 Tracing quantization process with small MDCT coefficients");
    
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
    
    let mut quantizer = QuantizationLoop::new();
    
    // Test with small MDCT coefficients
    let mut mdct_coeffs = [0i32; GRANULE_SIZE];
    
    // Set some small values that might quantize to 0 or 1
    for i in 0..100 {
        mdct_coeffs[i] = 10; // Small but non-zero values
    }
    
    let mut granule_info = GranuleInfo::default();
    let mut quantized_output = [0i32; GRANULE_SIZE];
    
    println!("Input: small MDCT coefficients (10) in first 100 positions");
    
    let max_bits = 1000;
    
    match quantizer.quantize_and_encode(&mdct_coeffs, max_bits, &mut granule_info, &mut quantized_output) {
        Ok(bits_used) => {
            println!("Quantization succeeded: {} bits used", bits_used);
            
            println!("Final granule info:");
            println!("  big_values: {}", granule_info.big_values);
            println!("  count1: {}", granule_info.count1);
            println!("  quantizer_step_size: {}", granule_info.quantizer_step_size);
            
            // Check the quantized output
            let non_zero_quantized = quantized_output.iter().filter(|&&x| x != 0).count();
            println!("Non-zero quantized coefficients: {}", non_zero_quantized);
            
            // For small values, we expect some reasonable count1 value
            if granule_info.count1 < 144 && granule_info.count1 >= 0 {
                println!("✓ Count1 looks reasonable: {}", granule_info.count1);
            } else {
                println!("❌ Count1 looks suspicious: {}", granule_info.count1);
            }
            
            // Show some of the quantized coefficients
            println!("\nFirst 20 quantized coefficients:");
            for i in 0..20 {
                print!("{} ", quantized_output[i]);
            }
            println!();
        },
        Err(e) => {
            println!("❌ Quantization failed: {:?}", e);
        }
    }
}