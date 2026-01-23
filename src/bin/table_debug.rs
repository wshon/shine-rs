//! Debug quantization tables
//!
//! This tool prints out the quantization table values to verify they are correct.

use rust_mp3_encoder::quantization::QuantizationLoop;

fn main() {
    println!("Quantization Table Debug");
    println!("=======================");
    
    let quantizer = QuantizationLoop::new();
    
    // Test step size calculation for common values
    println!("\n=== Step Size Index Calculation ===");
    for stepsize in [-120, -50, -10, 0, 10, 50, 120] {
        let step_index = (stepsize + 127).clamp(0, 255) as usize;
        println!("stepsize: {}, step_index: {}", stepsize, step_index);
    }
    
    // Test a simple quantization with debug output
    println!("\n=== Simple Quantization Debug ===");
    let test_input = [1000i32; 576];
    let mut quantized = [0i32; 576];
    
    for stepsize in [-10, 0, 10] {
        println!("\nTesting stepsize: {}", stepsize);
        let step_index = (stepsize + 127).clamp(0, 255) as usize;
        println!("  step_index: {}", step_index);
        
        // We can't access private fields, so let's just test the quantize function
        let max_quantized = quantizer.quantize(&mut quantized, stepsize, &test_input);
        let non_zero_count = quantized.iter().filter(|&&x| x != 0).count();
        
        println!("  max_quantized: {}", max_quantized);
        println!("  non_zero_count: {}", non_zero_count);
        println!("  first 10 quantized: {:?}", &quantized[0..10]);
    }
    
    // Test with very negative stepsize (should produce large quantized values)
    println!("\n=== Testing Very Negative Stepsize ===");
    for stepsize in [-120, -100, -80, -60] {
        let mut quantized = [0i32; 576];
        let max_quantized = quantizer.quantize(&mut quantized, stepsize, &test_input);
        let non_zero_count = quantized.iter().filter(|&&x| x != 0).count();
        
        println!("stepsize: {}, max_quantized: {}, non_zero_count: {}", 
                stepsize, max_quantized, non_zero_count);
    }
}