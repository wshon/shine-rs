//! Type definitions for MP3 encoding
//!
//! This module contains all the type definitions that correspond exactly
//! to shine's types.h, maintaining binary compatibility and data layout.

use crate::bitstream::BitstreamWriter;

/// Constants from shine (matches types.h exactly)
pub const GRANULE_SIZE: usize = 576;
pub const PI: f64 = std::f64::consts::PI;
pub const PI4: f64 = std::f64::consts::FRAC_PI_4;
pub const PI12: f64 = 0.26179938779915;
pub const PI36: f64 = 0.087266462599717;
pub const PI64: f64 = 0.049087385212;
pub const SQRT2: f64 = std::f64::consts::SQRT_2;
pub const LN2: f64 = std::f64::consts::LN_2;
pub const LN_TO_LOG10: f64 = 0.2302585093;
pub const BLKSIZE: usize = 1024;
pub const HAN_SIZE: usize = 512; // for loop unrolling, require that HAN_SIZE%8==0
pub const SCALE_BLOCK: i32 = 12;
pub const SCALE_RANGE: i32 = 64;
pub const SCALE: i32 = 32768;
pub const SBLIMIT: usize = 32;
pub const MAX_CHANNELS: usize = 2;
pub const MAX_GRANULES: usize = 2;

/// SWAB32 macro implementation (matches shine's SWAB32)
#[inline]
pub fn swab32(x: u32) -> u32 {
    (x >> 24) | ((x >> 8) & 0xff00) | ((x & 0xff00) << 8) | (x << 24)
}

/// Private shine wave configuration (matches priv_shine_wave_t)
/// (ref/shine/src/lib/types.h:60-63)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PrivShineWave {
    pub channels: i32,
    pub samplerate: i32,
}

/// Private shine MPEG configuration (matches priv_shine_mpeg_t)
/// (ref/shine/src/lib/types.h:65-87)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PrivShineMpeg {
    pub version: i32,
    pub layer: i32,
    pub granules_per_frame: i32,
    pub mode: i32, // Stereo mode
    pub bitr: i32, // Must conform to known bitrate
    pub emph: i32, // De-emphasis
    pub padding: i32,
    pub bits_per_frame: i32,
    pub bits_per_slot: i32,
    pub frac_slots_per_frame: f64,
    pub slot_lag: f64,
    pub whole_slots_per_frame: i32,
    pub bitrate_index: i32,    // See Main.c and Layer3.c
    pub samplerate_index: i32, // See Main.c and Layer3.c
    pub crc: i32,
    pub ext: i32,
    pub mode_ext: i32,
    pub copyright: i32,
    pub original: i32,
}
/// L3 loop structure (matches l3loop_t)
/// (ref/shine/src/lib/types.h:89-101)
#[repr(C)]
#[derive(Debug)]
pub struct L3Loop {
    /// Magnitudes of the spectral values
    pub xr: *mut i32,
    /// xr squared
    pub xrsq: Box<[i32; GRANULE_SIZE]>, // Move to heap
    /// xr absolute
    pub xrabs: Box<[i32; GRANULE_SIZE]>, // Move to heap
    /// Maximum of xrabs array
    pub xrmax: i32,
    /// Total energy per granule
    pub en_tot: [i32; MAX_GRANULES],
    /// Energy per scalefactor band [granule][sfb]
    pub en: [[i32; 21]; MAX_GRANULES],
    /// Masking threshold per scalefactor band [granule][sfb]
    pub xm: [[i32; 21]; MAX_GRANULES],
    /// Maximum per granule
    pub xrmaxl: [i32; MAX_GRANULES],
    /// 2**(-x/4) for x = -127..0
    pub steptab: [f64; 128],
    /// 2**(-x/4) for x = -127..0 (integer version)
    pub steptabi: [i32; 128],
    /// x**(3/4) for x = 0..9999
    pub int2idx: Box<[i32; 10000]>, // Move to heap
}

impl Default for L3Loop {
    fn default() -> Self {
        Self {
            xr: std::ptr::null_mut(),
            xrsq: Box::new([0; GRANULE_SIZE]),
            xrabs: Box::new([0; GRANULE_SIZE]),
            xrmax: 0,
            en_tot: [0; MAX_GRANULES],
            en: [[0; 21]; MAX_GRANULES],
            xm: [[0; 21]; MAX_GRANULES],
            xrmaxl: [0; MAX_GRANULES],
            steptab: [0.0; 128],
            steptabi: [0; 128],
            int2idx: Box::new([0; 10000]),
        }
    }
}

/// MDCT structure (matches mdct_t)
/// (ref/shine/src/lib/types.h:103-105)
#[repr(C)]
#[derive(Debug)]
pub struct Mdct {
    pub cos_l: [[i32; 36]; 18],
}

impl Default for Mdct {
    fn default() -> Self {
        Self {
            cos_l: [[0; 36]; 18],
        }
    }
}

/// Subband structure (matches subband_t)
/// (ref/shine/src/lib/types.h:107-111)
#[repr(C)]
#[derive(Debug)]
pub struct Subband {
    pub off: [i32; MAX_CHANNELS],
    pub fl: [[i32; 64]; SBLIMIT],
    pub x: Box<[[i32; HAN_SIZE]; MAX_CHANNELS]>, // Move to heap
}

impl Default for Subband {
    fn default() -> Self {
        Self {
            off: [0; MAX_CHANNELS],
            fl: [[0; 64]; SBLIMIT],
            x: Box::new([[0; HAN_SIZE]; MAX_CHANNELS]),
        }
    }
}
/// Granule information (matches gr_info)
/// (ref/shine/src/lib/types.h:114-133)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GrInfo {
    pub part2_3_length: u32,
    pub big_values: u32,
    pub count1: u32,
    pub global_gain: u32,
    pub scalefac_compress: u32,
    pub table_select: [u32; 3],
    pub region0_count: u32,
    pub region1_count: u32,
    pub preflag: u32,
    pub scalefac_scale: u32,
    pub count1table_select: u32,
    pub part2_length: u32,
    pub sfb_lmax: u32,
    pub address1: u32,
    pub address2: u32,
    pub address3: u32,
    pub quantizer_step_size: i32,
    pub slen: [u32; 4],
}

impl Default for GrInfo {
    fn default() -> Self {
        Self {
            part2_3_length: 0,
            big_values: 0,
            count1: 0,
            global_gain: 210,
            scalefac_compress: 0,
            table_select: [0, 0, 0],
            region0_count: 0,
            region1_count: 0,
            preflag: 0,
            scalefac_scale: 0,
            count1table_select: 0,
            part2_length: 0,
            sfb_lmax: 21,
            address1: 0,
            address2: 0,
            address3: 0,
            quantizer_step_size: 0,
            slen: [0, 0, 0, 0],
        }
    }
}

/// Channel information within a granule
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct GranuleChannel {
    pub tt: GrInfo,
}

/// Granule structure
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Granule {
    pub ch: [GranuleChannel; MAX_CHANNELS],
}
/// Side information structure (matches shine_side_info_t)
/// (ref/shine/src/lib/types.h:135-144)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShineSideInfo {
    pub private_bits: u32,
    pub resv_drain: i32, // matches resvDrain in shine
    pub scfsi: [[u32; 4]; MAX_CHANNELS],
    pub gr: [Granule; MAX_GRANULES],
}

impl Default for ShineSideInfo {
    fn default() -> Self {
        Self {
            private_bits: 0,
            resv_drain: 0,
            scfsi: [[0; 4]; MAX_CHANNELS],
            gr: [Granule::default(), Granule::default()],
        }
    }
}

/// Psychoacoustic ratio structure (matches shine_psy_ratio_t)
/// (ref/shine/src/lib/types.h:146-148)
#[repr(C)]
#[derive(Debug)]
pub struct ShinePsyRatio {
    pub l: Box<[[[f64; 21]; MAX_CHANNELS]; MAX_GRANULES]>, // Move to heap
}

impl Default for ShinePsyRatio {
    fn default() -> Self {
        Self {
            l: Box::new([[[0.0; 21]; MAX_CHANNELS]; MAX_GRANULES]),
        }
    }
}

/// Psychoacoustic minimum structure (matches shine_psy_xmin_t)
/// (ref/shine/src/lib/types.h:150-152)
#[repr(C)]
#[derive(Debug)]
pub struct ShinePsyXmin {
    pub l: Box<[[[f64; 21]; MAX_CHANNELS]; MAX_GRANULES]>, // Move to heap
}

impl Default for ShinePsyXmin {
    fn default() -> Self {
        Self {
            l: Box::new([[[0.0; 21]; MAX_CHANNELS]; MAX_GRANULES]),
        }
    }
}

/// Scale factor structure (matches shine_scalefac_t)
/// (ref/shine/src/lib/types.h:154-157)
#[repr(C)]
#[derive(Debug)]
pub struct ShineScalefac {
    /// Long block scale factors [granule][channel][scalefactor_band]
    pub l: Box<[[[i32; 22]; MAX_CHANNELS]; MAX_GRANULES]>, // Move to heap
    /// Short block scale factors [granule][channel][scalefactor_band][window]
    pub s: Box<[[[[i32; 3]; 13]; MAX_CHANNELS]; MAX_GRANULES]>, // Move to heap
}

impl Default for ShineScalefac {
    fn default() -> Self {
        Self {
            l: Box::new([[[0; 22]; MAX_CHANNELS]; MAX_GRANULES]),
            s: Box::new([[[[0; 3]; 13]; MAX_CHANNELS]; MAX_GRANULES]),
        }
    }
}
/// Global configuration structure (matches shine_global_config)
/// (ref/shine/src/lib/types.h:159-180)
#[repr(C)]
#[derive(Debug)]
pub struct ShineGlobalConfig {
    pub wave: PrivShineWave,
    pub mpeg: PrivShineMpeg,
    pub bs: BitstreamWriter,
    pub side_info: ShineSideInfo,
    pub sideinfo_len: i32,
    pub mean_bits: i32,
    pub ratio: ShinePsyRatio,
    pub scalefactor: Box<ShineScalefac>, // Move to heap
    pub buffer: [*mut i16; MAX_CHANNELS],
    pub pe: Box<[[f64; MAX_GRANULES]; MAX_CHANNELS]>, // Move to heap
    pub l3_enc: Box<[[[i32; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS]>, // Move to heap
    pub l3_sb_sample: Box<[[[[i32; SBLIMIT]; 18]; MAX_GRANULES + 1]; MAX_CHANNELS]>, // Move to heap
    pub mdct_freq: Box<[[[i32; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS]>, // Move to heap
    pub resv_size: i32,
    pub resv_max: i32,
    pub l3loop: L3Loop,
    pub mdct: Mdct,
    pub subband: Subband,
}

impl ShineGlobalConfig {
    /// Create a new global configuration
    pub fn new() -> Self {
        Self {
            wave: PrivShineWave {
                channels: 2,
                samplerate: 44100,
            },
            mpeg: PrivShineMpeg {
                version: 3, // MPEG_I = 3
                layer: 1,   // LAYER_III = 1
                granules_per_frame: 2,
                mode: 1,
                bitr: 128,
                emph: 0,
                padding: 0,
                bits_per_frame: 0,
                bits_per_slot: 8,
                frac_slots_per_frame: 0.0,
                slot_lag: 0.0,
                whole_slots_per_frame: 0,
                bitrate_index: 9,
                samplerate_index: 0,
                crc: 0,
                ext: 0,
                mode_ext: 0,
                copyright: 0,
                original: 1,
            },
            bs: BitstreamWriter::default(),
            side_info: ShineSideInfo::default(),
            sideinfo_len: 0,
            mean_bits: 0,
            ratio: ShinePsyRatio::default(),
            scalefactor: Box::new(ShineScalefac::default()), // Allocate on heap
            buffer: [std::ptr::null_mut(); MAX_CHANNELS],
            pe: Box::new([[0.0; MAX_GRANULES]; MAX_CHANNELS]), // Allocate on heap
            l3_enc: Box::new([[[0; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS]), // Allocate on heap
            l3_sb_sample: Box::new([[[[0; SBLIMIT]; 18]; MAX_GRANULES + 1]; MAX_CHANNELS]), // Allocate on heap
            mdct_freq: Box::new([[[0; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS]), // Allocate on heap
            resv_size: 0,
            resv_max: 0,
            l3loop: L3Loop::default(),
            mdct: Mdct::default(),
            subband: Subband::default(),
        }
    }
}

impl Default for ShineGlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}
