use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo, GRANULE_SIZE};

#[test]
fn debug_quantization_issue() {
    let mut quantizer = QuantizationLoop::new();
    
    // Test with simple input - all zeros
    let zero_coeffs = [0i32; GRANULE_SIZE];
    let mut zero_output = [0i32; GRANULE_SIZE];
    let mut zero_info = GranuleInfo::default();
    
    println!("Testing with all zeros...");
    let result = quantizer.quantize_and_encode(&zero_coeffs, 1000, &mut zero_info, &mut zero_output, 44100);
    println!("Zero coeffs result: {:?}", result);
    println!("Zero big_values: {}", zero_info.big_values);
    println!("Zero count1: {}", zero_info.count1);
    println!("Zero quantizer_step_size: {}", zero_info.quantizer_step_size);
    
    // Test with small non-zero values
    let mut small_coeffs = [0i32; GRANULE_SIZE];
    for i in 0..10 {
        small_coeffs[i] = 100; // Small values
    }
    let mut small_output = [0i32; GRANULE_SIZE];
    let mut small_info = GranuleInfo::default();
    
    println!("\nTesting with small values...");
    let result = quantizer.quantize_and_encode(&small_coeffs, 1000, &mut small_info, &mut small_output, 44100);
    println!("Small coeffs result: {:?}", result);
    println!("Small big_values: {}", small_info.big_values);
    println!("Small count1: {}", small_info.count1);
    println!("Small quantizer_step_size: {}", small_info.quantizer_step_size);
    
    // Check if big_values is reasonable
    assert!(small_info.big_values <= 288, "big_values {} exceeds maximum 288", small_info.big_values);
    
    // Test with larger values
    let mut large_coeffs = [0i32; GRANULE_SIZE];
    for i in 0..100 {
        large_coeffs[i] = 1000; // Larger values
    }
    let mut large_output = [0i32; GRANULE_SIZE];
    let mut large_info = GranuleInfo::default();
    
    println!("\nTesting with large values...");
    let result = quantizer.quantize_and_encode(&large_coeffs, 1000, &mut large_info, &mut large_output, 44100);
    println!("Large coeffs result: {:?}", result);
    println!("Large big_values: {}", large_info.big_values);
    println!("Large count1: {}", large_info.count1);
    println!("Large quantizer_step_size: {}", large_info.quantizer_step_size);
    
    // Check if big_values is reasonable
    assert!(large_info.big_values <= 288, "big_values {} exceeds maximum 288", large_info.big_values);
    
    // Check quantized output values
    let max_quantized = large_output.iter().map(|&x| x.abs()).max().unwrap_or(0);
    println!("Max quantized value: {}", max_quantized);
    assert!(max_quantized <= 8192, "Quantized value {} exceeds maximum 8192", max_quantized);
}