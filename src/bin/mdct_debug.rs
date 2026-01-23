//! MDCT debugging tool
//!
//! This tool analyzes the MDCT transform output to see if it's producing
//! reasonable values that should not cause big_values to exceed limits.

use rust_mp3_encoder::mdct::MdctTransform;
use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

fn main() {
    println!("MDCT Debug Tool");
    println!("===============");
    
    let mdct = MdctTransform::new();
    let quantizer = QuantizationLoop::new();
    
    // Test with small constant input (similar to our problematic test case)
    println!("\n=== Testing Small Constant Input ===");
    test_mdct_and_quantization(&mdct, &quantizer, 1, "small_constant");
    
    // Test with larger constant input
    println!("\n=== Testing Large Constant Input ===");
    test_mdct_and_quantization(&mdct, &quantizer, 1000, "large_constant");
    
    // Test with zero input
    println!("\n=== Testing Zero Input ===");
    test_mdct_and_quantization(&mdct, &quantizer, 0, "zero_input");
}

fn test_mdct_and_quantization(mdct: &MdctTransform, quantizer: &QuantizationLoop, 
                             input_value: i16, name: &str) {
    println!("\n--- Testing {} (input value: {}) ---", name, input_value);
    
    // Create subband samples (simulate what would come from polyphase filter)
    // For constant input, all subband samples should be similar
    let mut subband_samples = [[0i32; 32]; 36];
    
    // Fill with constant values (scaled appropriately for subband domain)
    // In a real encoder, these would come from the polyphase filter
    let scaled_value = (input_value as i32) * 1000; // Scale up for subband domain
    for granule in 0..36 {
        for band in 0..32 {
            subband_samples[granule][band] = scaled_value;
        }
    }
    
    // Perform MDCT transform
    let mut mdct_coeffs = [0i32; GRANULE_SIZE];
    match mdct.transform(&subband_samples, &mut mdct_coeffs) {
        Ok(()) => {
            println!("✅ MDCT transform successful");
            
            // Analyze MDCT coefficients
            let non_zero_count = mdct_coeffs.iter().filter(|&&x| x != 0).count();
            let max_abs = mdct_coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0);
            let min_abs = mdct_coeffs.iter().filter(|&&x| x != 0).map(|&x| x.abs()).min().unwrap_or(0);
            
            println!("MDCT coefficients analysis:");
            println!("  Non-zero count: {}", non_zero_count);
            println!("  Max absolute value: {}", max_abs);
            println!("  Min absolute value: {}", min_abs);
            
            // Show first few non-zero coefficients
            println!("  First 10 coefficients: {:?}", &mdct_coeffs[0..10]);
            
            // Test quantization with these MDCT coefficients
            println!("\nQuantization analysis:");
            
            // Try different step sizes
            for step_size in [-10, 0, 10, 20, 30, 40, 50] {
                let mut quantized = [0i32; GRANULE_SIZE];
                let max_quantized = quantizer.quantize(&mut quantized, step_size, &mdct_coeffs);
                
                if max_quantized <= 8192 {
                    let mut side_info = GranuleInfo::default();
                    quantizer.calculate_run_length(&quantized, &mut side_info);
                    
                    println!("  Step size {}: max_quantized={}, big_values={}, count1={}", 
                            step_size, max_quantized, side_info.big_values, side_info.count1);
                    
                    if side_info.big_values <= 288 {
                        println!("    ✅ big_values within limits");
                        break;
                    } else {
                        println!("    ❌ big_values too large: {} > 288", side_info.big_values);
                    }
                } else {
                    println!("  Step size {}: max_quantized={} (too large)", step_size, max_quantized);
                }
            }
            
        },
        Err(e) => {
            println!("❌ MDCT transform failed: {:?}", e);
        }
    }
}