//! MP3 encoder implementation
//!
//! This module implements the main MP3 encoding functions exactly as defined
//! in shine's layer3.c. It provides the primary interface for MP3 encoding
//! including initialization, configuration, and encoding operations.

use crate::error::{EncodingError, EncodingResult};
use crate::types::{ShineGlobalConfig, ShineSideInfo, GRANULE_SIZE};
use crate::tables::{SAMPLERATES, BITRATES};
use crate::bitstream::BitstreamWriter;

/// Buffer size for bitstream (matches shine BUFFER_SIZE)
/// (ref/shine/src/lib/bitstream.h:19)
const BUFFER_SIZE: i32 = 4096;

/// MPEG version constants (matches shine's mpeg_versions enum)
/// (ref/shine/src/lib/layer3.h:10)
const MPEG_I: i32 = 3;
const MPEG_II: i32 = 2;
const MPEG_25: i32 = 0;

/// MPEG layer constants (matches shine's mpeg_layers enum)
/// (ref/shine/src/lib/layer3.h:13)
const LAYER_III: i32 = 1;

/// Emphasis constants (matches shine's emph enum)
/// (ref/shine/src/lib/layer3.h:25)
const NONE: i32 = 0;

/// Granules per frame for different MPEG versions (matches shine's granules_per_frame)
/// (ref/shine/src/lib/layer3.c:9-14)
static GRANULES_PER_FRAME: [i32; 4] = [
    1,  // MPEG 2.5
    -1, // Reserved
    1,  // MPEG II
    2,  // MPEG I
];

/// Public wave configuration (matches shine_wave_t)
/// (ref/shine/src/lib/layer3.h:16-19)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShineWave {
    pub channels: i32,
    pub samplerate: i32,
}

/// Public MPEG configuration (matches shine_mpeg_t)
/// (ref/shine/src/lib/layer3.h:21-34)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShineMpeg {
    pub mode: i32,
    pub bitr: i32,
    pub emph: i32,
    pub copyright: i32,
    pub original: i32,
}

/// Public configuration structure (matches shine_config_t)
/// (ref/shine/src/lib/layer3.h:36-38)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShineConfig {
    pub wave: ShineWave,
    pub mpeg: ShineMpeg,
}

/// Set default values for important vars (matches shine_set_config_mpeg_defaults)
/// (ref/shine/src/lib/layer3.c:16-21)
pub fn shine_set_config_mpeg_defaults(mpeg: &mut ShineMpeg) {
    mpeg.bitr = 128;
    mpeg.emph = NONE;
    mpeg.copyright = 0;
    mpeg.original = 1;
}

/// Pick mpeg version according to samplerate index (matches shine_mpeg_version)
/// (ref/shine/src/lib/layer3.c:23-33)
pub fn shine_mpeg_version(samplerate_index: i32) -> i32 {
    if samplerate_index < 3 {
        // First 3 samplerates are for MPEG-I
        MPEG_I
    } else if samplerate_index < 6 {
        // Then it's MPEG-II
        MPEG_II
    } else {
        // Finally, MPEG-2.5
        MPEG_25
    }
}

/// Find samplerate index (matches shine_find_samplerate_index)
/// (ref/shine/src/lib/layer3.c:35-43)
pub fn shine_find_samplerate_index(freq: i32) -> i32 {
    for i in 0..9 {
        if freq == SAMPLERATES[i] {
            return i as i32;
        }
    }
    -1 // error - not a valid samplerate for encoder
}

/// Find bitrate index (matches shine_find_bitrate_index)
/// (ref/shine/src/lib/layer3.c:45-53)
pub fn shine_find_bitrate_index(bitr: i32, mpeg_version: i32) -> i32 {
    for i in 0..16 {
        if bitr == BITRATES[i][mpeg_version as usize] {
            return i as i32;
        }
    }
    -1 // error - not a valid bitrate for encoder
}

/// Check configuration validity (matches shine_check_config)
/// (ref/shine/src/lib/layer3.c:55-69)
pub fn shine_check_config(freq: i32, bitr: i32) -> i32 {
    let samplerate_index = shine_find_samplerate_index(freq);
    if samplerate_index < 0 {
        return -1;
    }

    let mpeg_version = shine_mpeg_version(samplerate_index);

    let bitrate_index = shine_find_bitrate_index(bitr, mpeg_version);
    if bitrate_index < 0 {
        return -1;
    }

    mpeg_version
}

/// Get samples per pass (matches shine_samples_per_pass)
/// (ref/shine/src/lib/layer3.c:71-73)
pub fn shine_samples_per_pass(config: &ShineGlobalConfig) -> i32 {
    config.mpeg.granules_per_frame * GRANULE_SIZE as i32
}

/// Compute default encoding values (matches shine_initialise)
/// (ref/shine/src/lib/layer3.c:75-134)
pub fn shine_initialise(pub_config: &ShineConfig) -> EncodingResult<Box<ShineGlobalConfig>> {
    if shine_check_config(pub_config.wave.samplerate, pub_config.mpeg.bitr) < 0 {
        return Err(EncodingError::ValidationError("Invalid configuration".to_string()));
    }

    let mut config = Box::new(ShineGlobalConfig::default());

    // Initialize submodules
    crate::subband::shine_subband_initialise(&mut config.subband);
    crate::mdct::shine_mdct_initialise(&mut config);
    crate::quantization::shine_loop_initialise(&mut config);

    // Copy public config
    config.wave.channels = pub_config.wave.channels;
    config.wave.samplerate = pub_config.wave.samplerate;
    config.mpeg.mode = pub_config.mpeg.mode;
    config.mpeg.bitr = pub_config.mpeg.bitr;
    config.mpeg.emph = pub_config.mpeg.emph;
    config.mpeg.copyright = pub_config.mpeg.copyright;
    config.mpeg.original = pub_config.mpeg.original;

    // Set default values
    config.resv_max = 0;
    config.resv_size = 0;
    config.mpeg.layer = LAYER_III;
    config.mpeg.crc = 0;
    config.mpeg.ext = 0;
    config.mpeg.mode_ext = 0;
    config.mpeg.bits_per_slot = 8;

    config.mpeg.samplerate_index = shine_find_samplerate_index(config.wave.samplerate);
    config.mpeg.version = shine_mpeg_version(config.mpeg.samplerate_index);
    config.mpeg.bitrate_index = shine_find_bitrate_index(config.mpeg.bitr, config.mpeg.version);
    config.mpeg.granules_per_frame = GRANULES_PER_FRAME[config.mpeg.version as usize];

    // Figure average number of 'slots' per frame
    let avg_slots_per_frame = (config.mpeg.granules_per_frame as f64 * GRANULE_SIZE as f64 / 
                              config.wave.samplerate as f64) *
                              (1000.0 * config.mpeg.bitr as f64 / config.mpeg.bits_per_slot as f64);

    config.mpeg.whole_slots_per_frame = avg_slots_per_frame as i32;

    config.mpeg.frac_slots_per_frame = avg_slots_per_frame - config.mpeg.whole_slots_per_frame as f64;
    config.mpeg.slot_lag = -config.mpeg.frac_slots_per_frame;

    if config.mpeg.frac_slots_per_frame == 0.0 {
        config.mpeg.padding = 0;
    }

    config.bs = BitstreamWriter::new(BUFFER_SIZE);

    // Clear side info (matches memset in shine)
    config.side_info = ShineSideInfo::default();

    // Determine the mean bitrate for main data
    if config.mpeg.granules_per_frame == 2 {
        // MPEG 1
        config.sideinfo_len = 8 * if config.wave.channels == 1 { 4 + 17 } else { 4 + 32 };
    } else {
        // MPEG 2
        config.sideinfo_len = 8 * if config.wave.channels == 1 { 4 + 9 } else { 4 + 17 };
    }

    Ok(config)
}

/// Internal encoding function (matches shine_encode_buffer_internal)
/// (ref/shine/src/lib/layer3.c:136-158)
fn shine_encode_buffer_internal(config: &mut ShineGlobalConfig, stride: i32) -> EncodingResult<(&[u8], usize)> {
    #[cfg(debug_assertions)]
    let frame_num = crate::get_next_frame_number();
    #[cfg(not(debug_assertions))]
    let _frame_num = crate::get_next_frame_number();
    
    // Start frame data collection
    #[cfg(debug_assertions)]
    crate::test_data::start_frame_collection(frame_num);
    
    // Dynamic padding calculation (matches shine exactly)
    if config.mpeg.frac_slots_per_frame != 0.0 {
        config.mpeg.padding = if config.mpeg.slot_lag <= (config.mpeg.frac_slots_per_frame - 1.0) { 1 } else { 0 };
        config.mpeg.slot_lag += config.mpeg.padding as f64 - config.mpeg.frac_slots_per_frame;
    }

    config.mpeg.bits_per_frame = 8 * (config.mpeg.whole_slots_per_frame + config.mpeg.padding);
    config.mean_bits = (config.mpeg.bits_per_frame - config.sideinfo_len) / config.mpeg.granules_per_frame;

    // Apply mdct to the polyphase output
    crate::mdct::shine_mdct_sub(config, stride);

    // Bit and noise allocation
    crate::quantization::shine_iteration_loop(config);

    // Write the frame to the bitstream
    crate::bitstream::format_bitstream(config)?;

    // Return data exactly as shine does: return current data_position and reset it
    let written = config.bs.data_position as usize;
    config.bs.data_position = 0;

    // Print key parameters for verification (debug mode only)
    #[cfg(debug_assertions)]
    println!("[RUST F{}] pad={}, bits={}, written={}, slot_lag={:.6}", 
             frame_num, config.mpeg.padding, config.mpeg.bits_per_frame, written, config.mpeg.slot_lag);

    // Record bitstream data for test collection
    #[cfg(debug_assertions)]
    crate::test_data::record_bitstream_data(
        config.mpeg.padding,
        config.mpeg.bits_per_frame,
        written,
        config.mpeg.slot_lag
    );

    // Stop after specified frames for debugging (debug mode only)
    #[cfg(debug_assertions)]
    {
        // Check for frame limit from environment variable or default to unlimited
        if let Ok(max_frames_str) = std::env::var("RUST_MP3_MAX_FRAMES") {
            if let Ok(max_frames) = max_frames_str.parse::<i32>() {
                if frame_num > max_frames {
                    println!("[RUST] Stopping after {} frames for comparison", max_frames);
                    // Return a special error to indicate we should stop encoding but still write the file
                    return Err(EncodingError::StopAfterFrames);
                }
            }
        }
    }

    Ok((&config.bs.data[..written], written))
}

/// Encode buffer with separate channel arrays (matches shine_encode_buffer)
/// (ref/shine/src/lib/layer3.c:160-167)
pub fn shine_encode_buffer<'a>(config: &'a mut ShineGlobalConfig, data: &[*const i16]) -> EncodingResult<(&'a [u8], usize)> {
    config.buffer[0] = data[0] as *mut i16;
    if config.wave.channels == 2 {
        config.buffer[1] = data[1] as *mut i16;
    }

    shine_encode_buffer_internal(config, 1)
}

/// Encode buffer with interleaved channels (matches shine_encode_buffer_interleaved)
/// (ref/shine/src/lib/layer3.c:169-176)
pub fn shine_encode_buffer_interleaved<'a>(config: &'a mut ShineGlobalConfig, data: *const i16) -> EncodingResult<(&'a [u8], usize)> {
    config.buffer[0] = data as *mut i16;
    if config.wave.channels == 2 {
        unsafe {
            config.buffer[1] = data.offset(1) as *mut i16;
        }
    }

    shine_encode_buffer_internal(config, config.wave.channels)
}

/// Flush remaining data (matches shine_flush)
/// (ref/shine/src/lib/layer3.c:178-183)
pub fn shine_flush(config: &mut ShineGlobalConfig) -> (&[u8], usize) {
    // Shine's flush function simply returns current data_position without any bitstream flush
    // *written = config->bs.data_position;
    // config->bs.data_position = 0;
    let written = config.bs.data_position as usize;
    config.bs.data_position = 0;

    (&config.bs.data[..written], written)
}

/// Close encoder and free resources (matches shine_close)
/// (ref/shine/src/lib/layer3.c:185-188)
pub fn shine_close(_config: Box<ShineGlobalConfig>) {
    // shine_close_bit_stream(&config->bs);
    // free(config);
    // In Rust, the Box will be automatically dropped
}

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