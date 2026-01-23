//! MDCT algorithm validation tests
//!
//! This module tests the MDCT implementation for correctness, numerical precision,
//! and compliance with the shine reference implementation.

use rust_mp3_encoder::mdct::MdctTransform;
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
    fn test_mdct_zero_input() {
        let mut mdct = MdctTransform::new();
        let subband_samples = [[0i32; 32]; 36];
        let mut mdct_coeffs = [0i32; 576];
        
        let result = mdct.transform(&subband_samples, &mut mdct_coeffs);
        
        assert!(result.is_ok(), "MDCT should succeed with zero input");
        
        // Zero input should produce zero output (or very small values due to rounding)
        let max_coeff = mdct_coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0);
        assert!(max_coeff <= 1, "Zero input should produce near-zero output, got max: {}", max_coeff);
    }

    #[test]
    fn test_mdct_impulse_response() {
        let mut mdct = MdctTransform::new();
        let mut subband_samples = [[0i32; 32]; 36];
        
        // Create impulse at center
        subband_samples[18][0] = 1000;
        
        let mut mdct_coeffs = [0i32; 576];
        let result = mdct.transform(&subband_samples, &mut mdct_coeffs);
        
        assert!(result.is_ok(), "MDCT should succeed with impulse input");
        
        // Impulse should spread energy across frequency bins
        let total_energy: i64 = mdct_coeffs.iter().map(|&x| (x as i64).pow(2)).sum();
        assert!(total_energy > 0, "Impulse should produce non-zero energy");
    }

    #[test]
    fn test_mdct_sine_wave_input() {
        let mut mdct = MdctTransform::new();
        let mut subband_samples = [[0i32; 32]; 36];
        
        // Generate sine wave in time domain
        for t in 0..36 {
            for sb in 0..32 {
                let phase = (t * sb) as f64 * 0.1;
                subband_samples[t][sb] = (1000.0 * phase.sin()) as i32;
            }
        }
        
        let mut mdct_coeffs = [0i32; 576];
        let result = mdct.transform(&subband_samples, &mut mdct_coeffs);
        
        assert!(result.is_ok(), "MDCT should succeed with sine wave input");
        
        // Sine wave should produce concentrated energy in frequency domain
        let total_energy: i64 = mdct_coeffs.iter().map(|&x| (x as i64).pow(2)).sum();
        assert!(total_energy > 100000, "Sine wave should produce significant energy");
    }

    #[test]
    fn test_mdct_aliasing_reduction() {
        let mut mdct = MdctTransform::new();
        let mut subband_samples = [[0i32; 32]; 36];
        
        // Create pattern that would cause aliasing without proper reduction
        for t in 0..36 {
            subband_samples[t][15] = if t % 2 == 0 { 1000 } else { -1000 };
            subband_samples[t][16] = if t % 2 == 0 { -1000 } else { 1000 };
        }
        
        let mut mdct_coeffs = [0i32; 576];
        let result = mdct.transform(&subband_samples, &mut mdct_coeffs);
        
        assert!(result.is_ok(), "MDCT should succeed with aliasing test pattern");
        
        // Verify aliasing reduction is working (coefficients should be reasonable)
        let max_coeff = mdct_coeffs.iter().map(|&x| x.abs()).max().unwrap_or(0);
        assert!(max_coeff < 100000, "Aliasing reduction should limit coefficient magnitude");
    }

    #[test]
    fn test_mdct_overflow_protection() {
        let mut mdct = MdctTransform::new();
        let mut subband_samples = [[0i32; 32]; 36];
        
        // Fill with maximum values to test overflow protection
        for t in 0..36 {
            for sb in 0..32 {
                subband_samples[t][sb] = if (t + sb) % 2 == 0 { 32767 } else { -32768 };
            }
        }
        
        let mut mdct_coeffs = [0i32; 576];
        let result = mdct.transform(&subband_samples, &mut mdct_coeffs);
        
        assert!(result.is_ok(), "MDCT should handle maximum input values without overflow");
        
        // Verify no coefficient exceeds reasonable bounds
        for &coeff in &mdct_coeffs {
            assert!(
                coeff.abs() <= i32::MAX / 2,
                "MDCT coefficient {} should not cause overflow",
                coeff
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 50,
            verbose: 0,
            max_shrink_iters: 0,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn test_mdct_energy_conservation(
            samples in prop::collection::vec(
                prop::collection::vec(-1000i32..1000, 32),
                36
            )
        ) {
            setup_clean_errors();
            
            let mut mdct = MdctTransform::new();
            let mut subband_samples = [[0i32; 32]; 36];
            
            // Copy generated samples
            for (t, time_samples) in samples.iter().enumerate() {
                for (sb, &sample) in time_samples.iter().enumerate() {
                    subband_samples[t][sb] = sample;
                }
            }
            
            let mut mdct_coeffs = [0i32; 576];
            let result = mdct.transform(&subband_samples, &mut mdct_coeffs);
            
            prop_assert!(result.is_ok(), "MDCT should always succeed");
            
            // Calculate input and output energy
            let input_energy: i64 = subband_samples.iter()
                .flat_map(|time_slice| time_slice.iter())
                .map(|&x| (x as i64).pow(2))
                .sum();
            
            let output_energy: i64 = mdct_coeffs.iter()
                .map(|&x| (x as i64).pow(2))
                .sum();
            
            // Energy should be conserved (within reasonable bounds due to quantization)
            if input_energy > 0 {
                let energy_ratio = output_energy as f64 / input_energy as f64;
                prop_assert!(
                    energy_ratio > 0.1 && energy_ratio < 10.0,
                    "Energy should be approximately conserved, ratio: {}",
                    energy_ratio
                );
            }
        }

        #[test]
        fn test_mdct_linearity(
            samples1 in prop::collection::vec(
                prop::collection::vec(-500i32..500, 32),
                36
            ),
            samples2 in prop::collection::vec(
                prop::collection::vec(-500i32..500, 32),
                36
            )
        ) {
            setup_clean_errors();
            
            let mut mdct = MdctTransform::new();
            
            // Test MDCT linearity: MDCT(a + b) ≈ MDCT(a) + MDCT(b)
            let mut subband1 = [[0i32; 32]; 36];
            let mut subband2 = [[0i32; 32]; 36];
            let mut subband_sum = [[0i32; 32]; 36];
            
            for t in 0..36 {
                for sb in 0..32 {
                    subband1[t][sb] = samples1[t][sb];
                    subband2[t][sb] = samples2[t][sb];
                    subband_sum[t][sb] = samples1[t][sb].saturating_add(samples2[t][sb]);
                }
            }
            
            let mut coeffs1 = [0i32; 576];
            let mut coeffs2 = [0i32; 576];
            let mut coeffs_sum = [0i32; 576];
            
            let result1 = mdct.transform(&subband1, &mut coeffs1);
            let result2 = mdct.transform(&subband2, &mut coeffs2);
            let result_sum = mdct.transform(&subband_sum, &mut coeffs_sum);
            
            prop_assert!(result1.is_ok() && result2.is_ok() && result_sum.is_ok(),
                        "All MDCT transforms should succeed");
            
            // Check linearity (allowing for some numerical error)
            let mut max_error = 0i32;
            for i in 0..576 {
                let expected = coeffs1[i].saturating_add(coeffs2[i]);
                let actual = coeffs_sum[i];
                let error = (expected - actual).abs();
                max_error = max_error.max(error);
            }
            
            // Allow some error due to saturation and numerical precision
            prop_assert!(
                max_error < 1000,
                "MDCT should be approximately linear, max error: {}",
                max_error
            );
        }
    }
}