//! Simple test to verify big_values limiting works correctly

use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

fn main() {
    println!("Simple Big Values Test");
    println!("=====================");
    
    let mut quantizer = QuantizationLoop::new();
    
    // Create test MDCT coefficients that would normally produce large big_values
    let mut test_coeffs = [0i32; GRANULE_SIZE];
    
    // Fill with small non-zero values that should produce many quantized coefficients
    for i in 0..GRANULE_SIZE {
        test_coeffs[i] = 100; // Small constant value
    }
    
    let mut output = [0i32; GRANULE_SIZE];
    let mut side_info = GranuleInfo::default();
    
    println!("Testing with constant value 100 across all {} coefficients", GRANULE_SIZE);
    
    // Try to quantize and encode with a reasonable bit budget
    match quantizer.quantize_and_encode(&test_coeffs, 2000, &mut side_info, &mut output, 44100) {
        Ok(bits_used) => {
            println!("SUCCESS: Quantization completed");
            println!("  Bits used: {}", bits_used);
            println!("  Big values: {} (limit: 288)", side_info.big_values);
            println!("  Count1: {}", side_info.count1);
            println!("  Global gain: {}", side_info.global_gain);
            println!("  Quantizer step size: {}", side_info.quantizer_step_size);
            
            if side_info.big_values > 288 {
                println!("  ❌ BIG_VALUES EXCEEDS LIMIT!");
            } else {
                println!("  ✅ big_values within limits");
            }
            
            // Count non-zero quantized coefficients
            let non_zero_count = output.iter().filter(|&&x| x != 0).count();
            println!("  Non-zero quantized coefficients: {}", non_zero_count);
        },
        Err(e) => {
            println!("ERROR: Quantization failed: {:?}", e);
        }
    }
    
    // Test with even larger values
    println!("\nTesting with constant value 1000 across all {} coefficients", GRANULE_SIZE);
    
    for i in 0..GRANULE_SIZE {
        test_coeffs[i] = 1000; // Larger constant value
    }
    
    let mut side_info2 = GranuleInfo::default();
    
    match quantizer.quantize_and_encode(&test_coeffs, 2000, &mut side_info2, &mut output, 44100) {
        Ok(bits_used) => {
            println!("SUCCESS: Quantization completed");
            println!("  Bits used: {}", bits_used);
            println!("  Big values: {} (limit: 288)", side_info2.big_values);
            println!("  Count1: {}", side_info2.count1);
            println!("  Global gain: {}", side_info2.global_gain);
            println!("  Quantizer step size: {}", side_info2.quantizer_step_size);
            
            if side_info2.big_values > 288 {
                println!("  ❌ BIG_VALUES EXCEEDS LIMIT!");
            } else {
                println!("  ✅ big_values within limits");
            }
            
            // Count non-zero quantized coefficients
            let non_zero_count = output.iter().filter(|&&x| x != 0).count();
            println!("  Non-zero quantized coefficients: {}", non_zero_count);
        },
        Err(e) => {
            println!("ERROR: Quantization failed: {:?}", e);
        }
    }
}