//! Test quantization tables against shine reference
//!
//! This tool compares our quantization table initialization with shine's expected values.

use rust_mp3_encoder::quantization::QuantizationLoop;

fn main() {
    println!("Quantization Table Test");
    println!("======================");
    
    let quantizer = QuantizationLoop::new();
    
    // Test a simple case that should work
    println!("\n=== Testing Simple Quantization ===");
    
    // Create a simple input with just a few non-zero values
    let mut simple_input = [0i32; 576];
    simple_input[0] = 1000;   // One strong coefficient
    simple_input[1] = 500;    // One medium coefficient
    simple_input[100] = 100;  // One weak coefficient
    
    println!("Input: {} non-zero values", simple_input.iter().filter(|&&x| x != 0).count());
    
    // Test quantization with different step sizes
    for step_size in [0, 10, 20, 30, 40, 50] {
        let mut quantized = [0i32; 576];
        let max_quantized = quantizer.quantize(&mut quantized, step_size, &simple_input);
        
        let non_zero_count = quantized.iter().filter(|&&x| x != 0).count();
        
        println!("Step size {}: max_quantized={}, non_zero_count={}", 
                step_size, max_quantized, non_zero_count);
        
        if max_quantized <= 8192 {
            let mut side_info = rust_mp3_encoder::quantization::GranuleInfo::default();
            quantizer.calculate_run_length(&quantized, &mut side_info);
            
            println!("  -> big_values={}, count1={}", side_info.big_values, side_info.count1);
            
            if side_info.big_values <= 288 {
                println!("  -> ✅ big_values within limits");
            } else {
                println!("  -> ❌ big_values too large");
            }
        } else {
            println!("  -> ❌ max_quantized too large");
        }
    }
    
    // Test the complete quantization and encoding process
    println!("\n=== Testing Complete Quantization Process ===");
    
    let mut quantizer = QuantizationLoop::new();
    let mut side_info = rust_mp3_encoder::quantization::GranuleInfo::default();
    let mut output = [0i32; 576];
    
    match quantizer.quantize_and_encode(&simple_input, 2000, &mut side_info, &mut output, 44100) {
        Ok(bits_used) => {
            println!("✅ Complete quantization successful");
            println!("  Bits used: {}", bits_used);
            println!("  big_values: {}", side_info.big_values);
            println!("  global_gain: {}", side_info.global_gain);
            println!("  quantizer_step_size: {}", side_info.quantizer_step_size);
            
            if side_info.big_values <= 288 {
                println!("  ✅ big_values within limits");
            } else {
                println!("  ❌ big_values too large: {}", side_info.big_values);
            }
        },
        Err(e) => {
            println!("❌ Complete quantization failed: {:?}", e);
        }
    }
}