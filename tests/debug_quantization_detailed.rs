use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

#[test]
fn debug_quantization_detailed() {
    let mut quantizer = QuantizationLoop::new();
    
    // Create test data similar to what would come from MDCT
    let mut test_coeffs = [0i32; GRANULE_SIZE];
    
    // Fill with a pattern that simulates MDCT output
    for i in 0..100 {
        test_coeffs[i] = 1000 + (i as i32 * 10); // Gradually increasing values
    }
    for i in 100..200 {
        test_coeffs[i] = 500; // Medium values
    }
    for i in 200..300 {
        test_coeffs[i] = 100; // Small values
    }
    // Rest remain zero
    
    let mut output = [0i32; GRANULE_SIZE];
    let mut info = GranuleInfo::default();
    
    println!("Testing quantization with realistic MDCT-like data...");
    
    // Test quantization directly first
    let max_quantized = quantizer.quantize(&test_coeffs, 0, &mut output);
    println!("Direct quantization with step_size=0:");
    println!("  Max quantized value: {}", max_quantized);
    
    // Count non-zero values
    let non_zero_count = output.iter().filter(|&&x| x != 0).count();
    println!("  Non-zero quantized values: {}", non_zero_count);
    
    // Calculate run length
    // quantizer.calculate_run_length(&output, &mut info);
    // println!("  big_values: {}", info.big_values);
    // println!("  count1: {}", info.count1);
    
    // Now test with full quantize_and_encode
    let mut full_output = [0i32; GRANULE_SIZE];
    let mut full_info = GranuleInfo::default();
    
    println!("\nTesting full quantize_and_encode...");
    let result = quantizer.quantize_and_encode(&test_coeffs, 1000, &mut full_info, &mut full_output, 44100);
    println!("Result: {:?}", result);
    println!("Full big_values: {}", full_info.big_values);
    println!("Full count1: {}", full_info.count1);
    println!("Full quantizer_step_size: {}", full_info.quantizer_step_size);
    
    // Check if big_values is reasonable
    if full_info.big_values > 288 {
        println!("ERROR: big_values {} exceeds maximum 288!", full_info.big_values);
        
        // Let's see what values are in the quantized output
        println!("Analyzing quantized values...");
        let mut value_counts = std::collections::HashMap::new();
        for &val in full_output.iter() {
            *value_counts.entry(val.abs()).or_insert(0) += 1;
        }
        
        println!("Value distribution:");
        let mut sorted_values: Vec<_> = value_counts.iter().collect();
        sorted_values.sort_by_key(|&(val, _)| val);
        for (val, count) in sorted_values.iter().take(10) {
            println!("  Value {}: {} occurrences", val, count);
        }
        
        // Find the last non-zero value
        let mut last_nonzero = 0;
        for i in (0..GRANULE_SIZE).rev() {
            if full_output[i] != 0 {
                last_nonzero = i;
                break;
            }
        }
        println!("Last non-zero value at index: {}", last_nonzero);
        println!("Expected big_values based on last non-zero: {}", (last_nonzero + 1) / 2);
    }
    
    assert!(full_info.big_values <= 288, "big_values {} exceeds maximum 288", full_info.big_values);
}