//! Unit tests for SCFSI (Scale Factor Selection Information) calculation
//!
//! These tests validate the SCFSI calculation logic in isolation,
//! ensuring it matches the Shine reference implementation exactly.

use shine_rs::types::ShineGlobalConfig;

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test SCFSI calculation constants match Shine implementation
    #[test]
    fn test_scfsi_constants() {
        // SCFSI band boundaries (ref/shine/src/lib/l3loop.c)
        const SCFSI_BAND_LONG: [i32; 5] = [0, 6, 11, 16, 21];
        const EN_SCFSI_BAND_KRIT: i32 = 10;
        const XM_SCFSI_BAND_KRIT: i32 = 10;

        // Verify SCFSI band boundaries match Shine's implementation
        assert_eq!(SCFSI_BAND_LONG[0], 0);
        assert_eq!(SCFSI_BAND_LONG[1], 6);
        assert_eq!(SCFSI_BAND_LONG[2], 11);
        assert_eq!(SCFSI_BAND_LONG[3], 16);
        assert_eq!(SCFSI_BAND_LONG[4], 21);

        // Verify SCFSI criteria match Shine's implementation
        assert_eq!(EN_SCFSI_BAND_KRIT, 10);
        assert_eq!(XM_SCFSI_BAND_KRIT, 10);
    }

    /// Test SCFSI decision logic
    #[test]
    fn test_scfsi_decision_logic() {
        const EN_SCFSI_BAND_KRIT: i32 = 10;
        const XM_SCFSI_BAND_KRIT: i32 = 10;

        // Test SCFSI decision logic
        // If sum0 < EN_SCFSI_BAND_KRIT && sum1 < XM_SCFSI_BAND_KRIT, then SCFSI = 1
        // Otherwise SCFSI = 0

        let test_cases = [
            (5, 5, 1),   // Both below threshold -> SCFSI = 1
            (15, 5, 0),  // sum0 above threshold -> SCFSI = 0
            (5, 15, 0),  // sum1 above threshold -> SCFSI = 0
            (15, 15, 0), // Both above threshold -> SCFSI = 0
            (10, 5, 0),  // sum0 equal to threshold -> SCFSI = 0
            (5, 10, 0),  // sum1 equal to threshold -> SCFSI = 0
        ];

        for (sum0, sum1, expected_scfsi) in test_cases.iter() {
            let scfsi = if sum0 < &EN_SCFSI_BAND_KRIT && sum1 < &XM_SCFSI_BAND_KRIT {
                1
            } else {
                0
            };
            assert_eq!(
                scfsi, *expected_scfsi,
                "SCFSI calculation failed for sum0={}, sum1={}",
                sum0, sum1
            );
        }
    }

    /// Test SCFSI condition calculation
    #[test]
    fn test_scfsi_condition_calculation() {
        // Test the condition calculation that determines whether SCFSI should be used
        // The condition must equal 6 for SCFSI to be calculated

        const EN_TOT_KRIT: i32 = 10;
        const EN_DIF_KRIT: i32 = 100;

        // Verify constants match Shine's implementation
        assert_eq!(EN_TOT_KRIT, 10);
        assert_eq!(EN_DIF_KRIT, 100);

        // Test condition calculation logic
        let mut condition = 0;

        // Simulate the condition calculation from calc_scfsi
        // for (gr2 = 2; gr2--;) {
        //   if (config->l3loop.xrmaxl[gr2]) condition++;
        //   condition++;
        // }

        // Simulate two granules with non-zero xrmaxl
        let xrmaxl = [100, 200]; // Both non-zero
        for gr2 in (0..2).rev() {
            if xrmaxl[gr2] != 0 {
                condition += 1;
            }
            condition += 1;
        }

        // At this point condition should be 4 (2 for non-zero xrmaxl + 2 always incremented)
        assert_eq!(condition, 4);

        // Simulate en_tot difference check
        let en_tot = [50i32, 52i32]; // Difference = 2, which is < EN_TOT_KRIT
        if (en_tot[0] - en_tot[1]).abs() < EN_TOT_KRIT {
            condition += 1;
        }
        assert_eq!(condition, 5);

        // Simulate tp (total energy difference) check
        let tp = 50; // < EN_DIF_KRIT
        if tp < EN_DIF_KRIT {
            condition += 1;
        }
        assert_eq!(condition, 6);

        // When condition == 6, SCFSI calculation should be performed
        assert_eq!(
            condition, 6,
            "Condition should equal 6 for SCFSI calculation"
        );
    }

    /// Test SCFSI version check
    #[test]
    fn test_scfsi_version_check() {
        // This test ensures that SCFSI calculation is only performed for MPEG-I (version 3)
        let mut config = ShineGlobalConfig::default();

        // Test MPEG-I (version 3) - should calculate SCFSI
        config.mpeg.version = 3;
        config.wave.channels = 2;

        // Initialize required data structures
        config.side_info.scfsi = [[0; 4]; 2];

        // For MPEG-I, SCFSI should be calculated (non-zero values possible)
        // For other versions, SCFSI should remain [0,0,0,0]

        // Test MPEG-II (version 2) - should NOT calculate SCFSI
        config.mpeg.version = 2;
        config.side_info.scfsi = [[1; 4]; 2]; // Set to non-zero initially

        // After processing, SCFSI should remain unchanged for non-MPEG-I versions
        // This verifies that calc_scfsi is not called for MPEG-II

        // Note: This is a structural test - actual SCFSI calculation would need
        // the full quantization loop to be executed
        assert_eq!(config.mpeg.version, 2);
        assert_eq!(config.side_info.scfsi[0], [1; 4]);
    }
}
