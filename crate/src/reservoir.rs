//! Bit reservoir implementation for MP3 encoding
//!
//! Layer3 bit reservoir: Described in C.1.5.4.2.2 of the IS
//! This module implements shine's reservoir.c functions exactly

use crate::types::{ShineGlobalConfig, GrInfo};

/// Get maximum reservoir bits for current granule
/// Corresponds to shine_max_reservoir_bits() in reservoir.c
/// 
/// Called at the beginning of each granule to get the max bit
/// allowance for the current granule based on reservoir size
/// and perceptual entropy.
pub fn shine_max_reservoir_bits(pe: &f64, config: &ShineGlobalConfig) -> i32 {
    let more_bits: i32;
    let mut max_bits: i32;
    let mut add_bits: i32;
    let over_bits: i32;
    let mut mean_bits = config.mean_bits;

    mean_bits /= config.wave.channels;
    max_bits = mean_bits;

    if max_bits > 4095 {
        max_bits = 4095;
    }
    if config.resv_max == 0 {
        return max_bits;
    }

    more_bits = (*pe * 3.1) as i32 - mean_bits;
    add_bits = 0;
    if more_bits > 100 {
        let frac = (config.resv_size * 6) / 10;

        if frac < more_bits {
            add_bits = frac;
        } else {
            add_bits = more_bits;
        }
    }
    over_bits = config.resv_size - ((config.resv_max << 3) / 10) - add_bits;
    if over_bits > 0 {
        add_bits += over_bits;
    }

    max_bits += add_bits;
    if max_bits > 4095 {
        max_bits = 4095;
    }
    max_bits
}

/// Adjust reservoir after granule encoding
/// Corresponds to shine_ResvAdjust() in reservoir.c
/// 
/// Called after a granule's bit allocation. Readjusts the size of
/// the reservoir to reflect the granule's usage.
pub fn shine_resv_adjust(gi: &GrInfo, config: &mut ShineGlobalConfig) {
    config.resv_size += (config.mean_bits / config.wave.channels) - gi.part2_3_length as i32;
}

/// Finalize reservoir at frame end
/// Corresponds to shine_ResvFrameEnd() in reservoir.c
/// 
/// Called after all granules in a frame have been allocated. Makes sure
/// that the reservoir size is within limits, possibly by adding stuffing
/// bits. Note that stuffing bits are added by increasing a granule's
/// part2_3_length. The bitstream formatter will detect this and write the
/// appropriate stuffing bits to the bitstream.
pub fn shine_resv_frame_end(config: &mut ShineGlobalConfig) {
    let ancillary_pad = 0;
    let mut stuffing_bits: i32;
    let mut over_bits: i32;
    let l3_side = &mut config.side_info;

    // just in case mean_bits is odd, this is necessary...
    if (config.wave.channels == 2) && (config.mean_bits & 1) != 0 {
        config.resv_size += 1;
    }

    over_bits = config.resv_size - config.resv_max;
    if over_bits < 0 {
        over_bits = 0;
    }

    config.resv_size -= over_bits;
    stuffing_bits = over_bits + ancillary_pad;

    // we must be byte aligned
    over_bits = config.resv_size % 8;
    if over_bits != 0 {
        stuffing_bits += over_bits;
        config.resv_size -= over_bits;
    }

    if stuffing_bits != 0 {
        /*
         * plan a: put all into the first granule
         * This was preferred by someone designing a
         * real-time decoder...
         */
        let gi = &mut l3_side.gr[0].ch[0].tt;

        if gi.part2_3_length + (stuffing_bits as u32) < 4095 {
            gi.part2_3_length += stuffing_bits as u32;
        } else {
            // plan b: distribute throughout the granules
            for gr in 0..config.mpeg.granules_per_frame {
                for ch in 0..config.wave.channels {
                    if stuffing_bits == 0 {
                        break;
                    }
                    let gi = &mut l3_side.gr[gr as usize].ch[ch as usize].tt;
                    let extra_bits = 4095 - gi.part2_3_length as i32;
                    let bits_this_gr = if extra_bits < stuffing_bits { 
                        extra_bits 
                    } else { 
                        stuffing_bits 
                    };
                    gi.part2_3_length += bits_this_gr as u32;
                    stuffing_bits -= bits_this_gr;
                }
                if stuffing_bits == 0 {
                    break;
                }
            }
            /*
             * If any stuffing bits remain, we elect to spill them
             * into ancillary data. The bitstream formatter will do this if
             * l3side->resvDrain is set
             */
            l3_side.resv_drain = stuffing_bits;
        }
    }
}