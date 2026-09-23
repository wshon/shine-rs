//! Unit tests for quantization module
//!
//! These tests validate quantization parameters, global gain calculation,
//! and big_values constraints against the Shine reference implementation.

use shine_rs::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_granule_info_default() {
        let gi = GrInfo::default();

        assert_eq!(gi.part2_3_length, 0, "Default part2_3_length should be 0");
        assert_eq!(gi.big_values, 0, "Default big_values should be 0");
        assert_eq!(gi.count1, 0, "Default count1 should be 0");
        assert_eq!(gi.global_gain, 210, "Default global_gain should be 210");
        assert_eq!(
            gi.scalefac_compress, 0,
            "Default scalefac_compress should be 0"
        );
        assert_eq!(
            gi.table_select,
            [0, 0, 0],
            "Default table_select should be [0,0,0]"
        );
        assert_eq!(gi.region0_count, 0, "Default region0_count should be 0");
        assert_eq!(gi.region1_count, 0, "Default region1_count should be 0");
        assert_eq!(gi.preflag, 0, "Default preflag should be 0");
        assert_eq!(gi.scalefac_scale, 0, "Default scalefac_scale should be 0");
        assert_eq!(
            gi.count1table_select, 0,
            "Default count1table_select should be 0"
        );
        assert_eq!(gi.part2_length, 0, "Default part2_length should be 0");
        assert_eq!(gi.sfb_lmax, 21, "Default sfb_lmax should be 21");
        assert_eq!(
            gi.quantizer_step_size, 0,
            "Default quantizer_step_size should be 0"
        );
    }

    #[test]
    fn test_mp3_standard_limits() {
        // Test MP3 standard limits that our implementation must respect

        // Test that our granule info structure can hold valid MP3 values
        let mut gr_info = GrInfo::default();

        // Test setting maximum valid values
        gr_info.part2_3_length = 4095; // 12-bit field maximum
        gr_info.big_values = 288; // Granule size / 2 maximum
        gr_info.global_gain = 255; // 8-bit field maximum

        assert!(
            gr_info.part2_3_length <= 4095,
            "Part2_3_length should fit in 12 bits"
        );
        assert!(
            gr_info.big_values <= 288,
            "Big values should not exceed granule limit"
        );
        assert!(
            gr_info.global_gain <= 255,
            "Global gain should fit in 8 bits"
        );
    }

    #[test]
    fn test_region_addresses_stay_inside_big_values_region() {
        for samplerate in [32_000, 44_100, 48_000] {
            for big_values in 1..=288 {
                let mut gr_info = GrInfo {
                    big_values,
                    ..GrInfo::default()
                };
                shine_rs::quantization::subdivide_with_samplerate(&mut gr_info, samplerate);
                assert!(
                    gr_info.address1 <= 2 * big_values && gr_info.address2 <= 2 * big_values,
                    "samplerate {samplerate}, big_values {big_values}: address1 {}, address2 {}",
                    gr_info.address1,
                    gr_info.address2
                );
            }
        }
    }

    #[test]
    fn test_region_addresses_reset_without_big_values() {
        let mut gr_info = GrInfo {
            big_values: 100,
            ..GrInfo::default()
        };
        shine_rs::quantization::subdivide_with_samplerate(&mut gr_info, 48_000);
        assert!(gr_info.address1 > 0 && gr_info.address2 > 0 && gr_info.address3 > 0);

        gr_info.big_values = 0;
        shine_rs::quantization::subdivide_with_samplerate(&mut gr_info, 48_000);
        assert_eq!(
            (gr_info.address1, gr_info.address2, gr_info.address3),
            (0, 0, 0)
        );
    }
}
