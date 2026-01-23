//! Detailed quantization debugging tool
//!
//! This tool provides step-by-step analysis of the quantization process
//! to identify where big_values becomes too large.

use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

fn main() {
    println!("Quantization Debug Tool");
    println!("======================");
    
    let quantizer = QuantizationLoop::new();
    
    // Test with small constant input (similar to test case 2)
    println!("\n=== Testing Small Constant Input ===");
    let small_constant = [1i32; GRANULE_SIZE];
    test_quantization_steps(&quantizer, &small_constant, "small_constant");
    
    // Test with larger constant input (similar to test case 3)
    println!("\n=== Testing Large Constant Input ===");
    let large_constant = [1000i32; GRANULE_SIZE];
    test_quantization_steps(&quantizer, &large_constant, "large_constant");
    
    // Test with zero input (should work correctly)
    println!("\n=== Testing Zero Input ===");
    let zero_input = [0i32; GRANULE_SIZE];
    test_quantization_steps(&quantizer, &zero_input, "zero_input");
}

fn test_quantization_steps(quantizer: &QuantizationLoop, input: &[i32; GRANULE_SIZE], name: &str) {
    println!("\n--- Testing {} ---", name);
    
    // Step 1: Test different quantization step sizes
    println!("Step 1: Testing quantization with different step sizes");
    for step_size in [-10, 0, 10, 20, 30, 40, 50] {
        let mut quantized = [0i32; GRANULE_SIZE];
        let max_quantized = quantizer.quantize(&mut quantized, step_size, input);
        
        // Count non-zero values
        let non_zero_count = quantized.iter().filter(|&&x| x != 0).count();
        
        println!("  Step size {}: max_quantized={}, non_zero_count={}", 
                step_size, max_quantized, non_zero_count);
        
        if max_quantized <= 8192 {
            // Step 2: Calculate run length for this quantization
            let mut side_info = GranuleInfo::default();
            quantizer.calculate_run_length(&quantized, &mut side_info);
            
            println!("    -> big_values={}, count1={}", 
                    side_info.big_values, side_info.count1);
            
            if side_info.big_values <= 288 {
                println!("    -> ✅ big_values within limits");
                break;
            } else {
                println!("    -> ❌ big_values too large: {} > 288", side_info.big_values);
            }
        } else {
            println!("    -> ❌ max_quantized too large: {} > 8192", max_quantized);
        }
    }
    
    // Step 3: Test binary search step size calculation
    println!("\nStep 2: Testing binary search for optimal step size");
    let mut side_info = GranuleInfo::default();
    let target_bits = 2000; // Reasonable target
    
    let optimal_step = quantizer.calculate_step_size(input, target_bits, &mut side_info, 44100);
    println!("  Optimal step size: {}", optimal_step);
    
    // Test the optimal step size
    let mut quantized = [0i32; GRANULE_SIZE];
    let max_quantized = quantizer.quantize(&mut quantized, optimal_step, input);
    
    let mut final_side_info = GranuleInfo::default();
    quantizer.calculate_run_length(&quantized, &mut final_side_info);
    
    println!("  Final results:");
    println!("    max_quantized: {}", max_quantized);
    println!("    big_values: {}", final_side_info.big_values);
    println!("    count1: {}", final_side_info.count1);
    
    if final_side_info.big_values <= 288 {
        println!("    ✅ Final big_values within limits");
    } else {
        println!("    ❌ Final big_values too large: {} > 288", final_side_info.big_values);
    }
}