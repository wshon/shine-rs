//! Debug binary search for step size
//!
//! This tool traces the binary search process to see why it's not finding
//! the correct step size.

use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

fn main() {
    println!("Binary Search Debug");
    println!("==================");
    
    let quantizer = QuantizationLoop::new();
    
    // Test with a simple input that should work
    let mut test_input = [0i32; GRANULE_SIZE];
    test_input[0] = 1000;
    test_input[1] = 500;
    test_input[100] = 100;
    
    println!("Input: {} non-zero values", test_input.iter().filter(|&&x| x != 0).count());
    
    // Manual binary search with debug output
    let desired_rate = 2000;
    let mut next = -120;
    let mut count = 120;
    let mut iteration = 0;
    
    println!("\n=== Manual Binary Search ===");
    println!("Target rate: {} bits", desired_rate);
    
    loop {
        let half = count / 2;
        iteration += 1;
        
        println!("\nIteration {}: next={}, count={}, half={}", iteration, next, count, half);
        
        if half == 0 {
            println!("Search complete: final stepsize = {}", next);
            break;
        }
        
        let test_stepsize = next + half;
        println!("  Testing stepsize: {}", test_stepsize);
        
        // Test quantization
        let mut temp_coeffs = [0i32; GRANULE_SIZE];
        let max_quantized = quantizer.quantize(&mut temp_coeffs, test_stepsize, &test_input);
        
        println!("  max_quantized: {}", max_quantized);
        
        let bit = if max_quantized > 8192 {
            println!("  -> FAIL: max_quantized > 8192");
            100000
        } else {
            // Calculate bit count
            let mut temp_info = GranuleInfo::default();
            temp_info.quantizer_step_size = test_stepsize;
            
            quantizer.calculate_run_length(&temp_coeffs, &mut temp_info);
            println!("  -> big_values: {}, count1: {}", temp_info.big_values, temp_info.count1);
            
            // For now, just return a simple bit count based on non-zero values
            let non_zero_count = temp_coeffs.iter().filter(|&&x| x != 0).count();
            let estimated_bits = non_zero_count * 4; // Rough estimate
            
            println!("  -> estimated_bits: {}", estimated_bits);
            estimated_bits as i32
        };
        
        println!("  bit count: {}", bit);
        
        if bit < desired_rate {
            println!("  -> bit < desired_rate, reducing count");
            count = half;
        } else {
            println!("  -> bit >= desired_rate, increasing next");
            next += half;
            count -= half;
        }
        
        if iteration > 10 {
            println!("Too many iterations, stopping");
            break;
        }
    }
    
    // Test the final stepsize
    println!("\n=== Testing Final Stepsize ===");
    let mut final_coeffs = [0i32; GRANULE_SIZE];
    let max_quantized = quantizer.quantize(&mut final_coeffs, next, &test_input);
    
    let mut final_info = GranuleInfo::default();
    quantizer.calculate_run_length(&final_coeffs, &mut final_info);
    
    println!("Final stepsize: {}", next);
    println!("Final max_quantized: {}", max_quantized);
    println!("Final big_values: {}", final_info.big_values);
    println!("Final count1: {}", final_info.count1);
}