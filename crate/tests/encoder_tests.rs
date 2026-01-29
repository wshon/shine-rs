//! Unit tests for encoder operations
//!
//! Tests the main encoder functionality including configuration validation,
//! initialization, and encoding parameter setup.

use shine_rs::encoder::*;
use shine_rs::types::*;

// Import constants from encoder module
use shine_rs::encoder::{LAYER_III, MPEG_25, MPEG_I, MPEG_II, NONE};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shine_mpeg_version() {
        // Test MPEG-I (first 3 samplerates)
        assert_eq!(shine_mpeg_version(0), MPEG_I);
        assert_eq!(shine_mpeg_version(1), MPEG_I);
        assert_eq!(shine_mpeg_version(2), MPEG_I);

        // Test MPEG-II (next 3 samplerates)
        assert_eq!(shine_mpeg_version(3), MPEG_II);
        assert_eq!(shine_mpeg_version(4), MPEG_II);
        assert_eq!(shine_mpeg_version(5), MPEG_II);

        // Test MPEG-2.5 (remaining samplerates)
        assert_eq!(shine_mpeg_version(6), MPEG_25);
        assert_eq!(shine_mpeg_version(7), MPEG_25);
        assert_eq!(shine_mpeg_version(8), MPEG_25);
    }

    #[test]
    fn test_shine_find_samplerate_index() {
        // Test valid samplerates
        assert_eq!(shine_find_samplerate_index(44100), 0);
        assert_eq!(shine_find_samplerate_index(48000), 1);
        assert_eq!(shine_find_samplerate_index(32000), 2);
        assert_eq!(shine_find_samplerate_index(22050), 3);
        assert_eq!(shine_find_samplerate_index(24000), 4);
        assert_eq!(shine_find_samplerate_index(16000), 5);
        assert_eq!(shine_find_samplerate_index(11025), 6);
        assert_eq!(shine_find_samplerate_index(12000), 7);
        assert_eq!(shine_find_samplerate_index(8000), 8);

        // Test invalid samplerate
        assert_eq!(shine_find_samplerate_index(96000), -1);
    }

    #[test]
    fn test_shine_find_bitrate_index() {
        // Test MPEG-I bitrates
        assert_eq!(shine_find_bitrate_index(128, MPEG_I), 9);
        assert_eq!(shine_find_bitrate_index(160, MPEG_I), 10);
        assert_eq!(shine_find_bitrate_index(192, MPEG_I), 11);

        // Test invalid bitrate
        assert_eq!(shine_find_bitrate_index(999, MPEG_I), -1);
    }

    #[test]
    fn test_shine_check_config() {
        // Test valid configuration
        assert!(shine_check_config(44100, 128) >= 0);

        // Test invalid samplerate
        assert_eq!(shine_check_config(96000, 128), -1);

        // Test invalid bitrate
        assert_eq!(shine_check_config(44100, 999), -1);
    }

    #[test]
    fn test_shine_set_config_mpeg_defaults() {
        let mut mpeg = ShineMpeg {
            mode: 0,
            bitr: 0,
            emph: 0,
            copyright: 0,
            original: 0,
        };

        shine_set_config_mpeg_defaults(&mut mpeg);

        assert_eq!(mpeg.bitr, 128);
        assert_eq!(mpeg.emph, NONE);
        assert_eq!(mpeg.copyright, 0);
        assert_eq!(mpeg.original, 1);
    }

    #[test]
    fn test_shine_samples_per_pass() {
        let mut config = Box::new(ShineGlobalConfig::default());
        config.mpeg.granules_per_frame = 2; // MPEG-I

        let samples = shine_samples_per_pass(&*config);
        assert_eq!(samples, 2 * GRANULE_SIZE as i32);
    }

    #[test]
    fn test_shine_initialise() {
        let pub_config = ShineConfig {
            wave: ShineWave {
                channels: 2,
                samplerate: 44100,
            },
            mpeg: ShineMpeg {
                mode: 0,
                bitr: 128,
                emph: NONE,
                copyright: 0,
                original: 1,
            },
        };

        let result = shine_initialise(&pub_config);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.wave.channels, 2);
        assert_eq!(config.wave.samplerate, 44100);
        assert_eq!(config.mpeg.bitr, 128);
        assert_eq!(config.mpeg.layer, LAYER_III);
        assert_eq!(config.mpeg.bits_per_slot, 8);
    }
}
