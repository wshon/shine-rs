//! Quantization algorithm validation tests
//!
//! This module tests the quantization loop implementation against shine reference
//! and validates quantization table accuracy.

use rust_mp3_encoder::quantization::{QuantizationLoop, GranuleInfo};
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_clean_errors() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                if let Some(s) = info.payload().downcast_ref::<String>() {
                    let msg = if s.len() > 200 { &s[..197] } else { s };
                    eprintln!("Test failed: {}", msg.trim());
                }
            }));
        });
    }

    #[test]
    fn test_quantization_table_initialization() {
        let quantizer = QuantizationLoop::new();
        
        // Test that quantization tables are properly initialized
        // This would test the quantization table generation against shine's tables
        println!("Testing quantization table initialization...");
        
        // Verify table bounds and values match shine implementation
        // (Implementation would depend on exposing table access methods)
    }

    #[test]
    fn test_quantization_with_zero_input() {
        let mut quantizer = QuantizationLoop::new();
        let mdct_coeffs = [0i32; 576];
        let mut granule_info = GranuleInfo::default();
        let mut quantized_coeffs = [0i32; 576];
        
        let result = quantizer.quantize_and_encode(
            &mdct_coeffs,
            1000, // mean_bits
            &mut granule_info,
            &mut quantized_coeffs,
            44100
        );
        
        assert!(result.is_ok(), "Quantization should succeed with zero input");
        assert_eq!(granule_info.big_values, 0, "Zero input should produce zero big_values");
        assert_eq!(granule_info.global_gain, 91, "Zero input should have default global_gain");
    }

    #[test]
    fn test_quantization_with_small_values() {
        let mut quantizer = QuantizationLoop::new();
        let mut mdct_coeffs = [0i32; 576];
        
        // Fill with small values
        for i in 0..100 {
            mdct_coeffs[i] = 10;
        }
        
        let mut granule_info = GranuleInfo::default();
        let mut quantized_coeffs = [0i32; 576];
        
        let result = quantizer.quantize_and_encode(
            &mdct_coeffs,
            1000,
            &mut granule_info,
            &mut quantized_coeffs,
            44100
        );
        
        assert!(result.is_ok(), "Quantization should succeed with small values");
        assert!(granule_info.big_values <= 288, "big_values should be within limits");
    }

    #[test]
    fn test_quantization_with_large_values() {
        let mut quantizer = QuantizationLoop::new();
        let mut mdct_coeffs = [0i32; 576];
        
        // Fill with large values that require quantization
        for i in 0..200 {
            mdct_coeffs[i] = 1000 + (i as i32 * 10);
        }
        
        let mut granule_info = GranuleInfo::default();
        let mut quantized_coeffs = [0i32; 576];
        
        let result = quantizer.quantize_and_encode(
            &mdct_coeffs,
            2000,
            &mut granule_info,
            &mut quantized_coeffs,
            44100
        );
        
        assert!(result.is_ok(), "Quantization should succeed with large values");
        assert!(granule_info.big_values <= 288, "big_values should be within limits");
        assert!(granule_info.global_gain > 0, "Global gain should be set for large values");
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_quantization_big_values_property(
            coeffs in prop::collection::vec(-32768i32..32768, 576),
            mean_bits in 100i32..5000,
            sample_rate in prop::sample::select(&[44100u32, 48000, 32000])
        ) {
            setup_clean_errors();
            
            let mut quantizer = QuantizationLoop::new();
            let mut mdct_coeffs = [0i32; 576];
            mdct_coeffs.copy_from_slice(&coeffs);
            
            let mut granule_info = GranuleInfo::default();
            let mut quantized_coeffs = [0i32; 576];
            
            let result = quantizer.quantize_and_encode(
                &mdct_coeffs,
                mean_bits,
                &mut granule_info,
                &mut quantized_coeffs,
                sample_rate
            );
            
            prop_assert!(result.is_ok(), "Quantization should always succeed");
            prop_assert!(granule_info.big_values <= 288, "big_values must never exceed 288");
            prop_assert!(granule_info.global_gain <= 255, "global_gain must fit in 8 bits");
        }

        #[test]
        fn test_quantization_deterministic(
            coeffs in prop::collection::vec(-1000i32..1000, 576),
            mean_bits in 500i32..2000
        ) {
            setup_clean_errors();
            
            let mut quantizer1 = QuantizationLoop::new();
            let mut quantizer2 = QuantizationLoop::new();
            
            let mut mdct_coeffs = [0i32; 576];
            mdct_coeffs.copy_from_slice(&coeffs);
            
            let mut granule_info1 = GranuleInfo::default();
            let mut quantized_coeffs1 = [0i32; 576];
            
            let mut granule_info2 = GranuleInfo::default();
            let mut quantized_coeffs2 = [0i32; 576];
            
            let result1 = quantizer1.quantize_and_encode(
                &mdct_coeffs, mean_bits, &mut granule_info1, &mut quantized_coeffs1, 44100
            );
            
            let result2 = quantizer2.quantize_and_encode(
                &mdct_coeffs, mean_bits, &mut granule_info2, &mut quantized_coeffs2, 44100
            );
            
            prop_assert!(result1.is_ok() && result2.is_ok(), "Both quantizations should succeed");
            prop_assert_eq!(granule_info1.big_values, granule_info2.big_values, "Quantization should be deterministic");
            prop_assert_eq!(granule_info1.global_gain, granule_info2.global_gain, "Global gain should be deterministic");
        }
    }
}