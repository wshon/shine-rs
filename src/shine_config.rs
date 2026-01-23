//! Shine global configuration structure
//!
//! This module implements the shine_global_config structure that mirrors
//! the original shine implementation exactly, maintaining all data structures
//! and their relationships as defined in ref/shine/src/lib/types.h

use crate::config::Config;
use crate::bitstream::BitstreamWriter;
use crate::quantization::GranuleInfo;
use crate::Result;

/// Maximum number of channels (from shine's MAX_CHANNELS)
pub const MAX_CHANNELS: usize = 2;

/// Maximum number of granules (from shine's MAX_GRANULES) 
pub const MAX_GRANULES: usize = 2;

/// Granule size in samples (from shine's GRANULE_SIZE)
pub const GRANULE_SIZE: usize = 576;

/// Number of subbands (from shine's SBLIMIT)
pub const SBLIMIT: usize = 32;

/// HAN window size (from shine's HAN_SIZE)
pub const HAN_SIZE: usize = 512;

/// L3 loop structure following shine's l3loop_t exactly
/// (ref/shine/src/lib/types.h:95-102)
#[derive(Debug)]
pub struct L3Loop {
    /// MDCT coefficients pointer (xr in shine)
    pub xr: *mut i32,
    /// Squared MDCT coefficients
    pub xrsq: [i32; GRANULE_SIZE],
    /// Absolute MDCT coefficients  
    pub xrabs: [i32; GRANULE_SIZE],
    /// Maximum absolute coefficient
    pub xrmax: i32,
    /// Quantization step table (floating point)
    pub steptab: [f64; 256],
    /// Integer quantization step table
    pub steptabi: [i32; 256],
    /// Integer to index lookup table
    pub int2idx: [i32; 10000],
}

impl Default for L3Loop {
    fn default() -> Self {
        Self {
            xr: std::ptr::null_mut(),
            xrsq: [0; GRANULE_SIZE],
            xrabs: [0; GRANULE_SIZE], 
            xrmax: 0,
            steptab: [0.0; 256],
            steptabi: [0; 256],
            int2idx: [0; 10000],
        }
    }
}

/// MDCT structure following shine's mdct_t exactly
/// (ref/shine/src/lib/types.h:104-106)
#[derive(Debug)]
pub struct Mdct {
    /// Cosine lookup table for MDCT
    pub cos_l: [[i32; 36]; 18],
}

impl Default for Mdct {
    fn default() -> Self {
        Self {
            cos_l: [[0; 36]; 18],
        }
    }
}

/// Subband structure following shine's subband_t exactly  
/// (ref/shine/src/lib/types.h:108-112)
#[derive(Debug)]
pub struct Subband {
    /// Channel offsets
    pub off: [i32; MAX_CHANNELS],
    /// Filter coefficients
    pub fl: [[i32; 64]; SBLIMIT],
    /// Sample history buffer
    pub x: [[i32; HAN_SIZE]; MAX_CHANNELS],
}

impl Default for Subband {
    fn default() -> Self {
        Self {
            off: [0; MAX_CHANNELS],
            fl: [[0; 64]; SBLIMIT],
            x: [[0; HAN_SIZE]; MAX_CHANNELS],
        }
    }
}

/// Side information structure following shine's shine_side_info_t exactly
/// (ref/shine/src/lib/types.h:135-144)
#[derive(Debug, Clone)]
pub struct ShineSideInfo {
    /// Private bits
    pub private_bits: u32,
    /// Reservoir drain
    pub resv_drain: i32,
    /// Scale factor selection information
    pub scfsi: [[u32; 4]; MAX_CHANNELS],
    /// Granule information
    pub gr: [[GranuleChannel; MAX_CHANNELS]; MAX_GRANULES],
}

#[derive(Debug, Clone)]
pub struct GranuleChannel {
    pub tt: GranuleInfo,
}

impl Default for GranuleChannel {
    fn default() -> Self {
        Self {
            tt: GranuleInfo::default(),
        }
    }
}

impl Default for ShineSideInfo {
    fn default() -> Self {
        Self {
            private_bits: 0,
            resv_drain: 0,
            scfsi: [[0; 4]; MAX_CHANNELS],
            gr: [[GranuleChannel::default(); MAX_CHANNELS]; MAX_GRANULES],
        }
    }
}

/// Psychoacoustic ratio structure following shine's shine_psy_ratio_t exactly
/// (ref/shine/src/lib/types.h:146-148)
#[derive(Debug)]
pub struct ShinePsyRatio {
    /// Psychoacoustic ratios [granule][channel][scalefactor_band]
    pub l: [[[f64; 21]; MAX_CHANNELS]; MAX_GRANULES],
}

impl Default for ShinePsyRatio {
    fn default() -> Self {
        Self {
            l: [[[0.0; 21]; MAX_CHANNELS]; MAX_GRANULES],
        }
    }
}

/// Psychoacoustic minimum structure following shine's shine_psy_xmin_t exactly  
/// (ref/shine/src/lib/types.h:150-152)
#[derive(Debug)]
pub struct ShinePsyXmin {
    /// Psychoacoustic minimums [granule][channel][scalefactor_band]
    pub l: [[[f64; 21]; MAX_CHANNELS]; MAX_GRANULES],
}

impl Default for ShinePsyXmin {
    fn default() -> Self {
        Self {
            l: [[[0.0; 21]; MAX_CHANNELS]; MAX_GRANULES],
        }
    }
}

/// Scale factor structure following shine's shine_scalefac_t exactly
/// (ref/shine/src/lib/types.h:154-157)
#[derive(Debug)]
pub struct ShineScalefac {
    /// Long block scale factors [granule][channel][scalefactor_band]
    pub l: [[[i32; 22]; MAX_CHANNELS]; MAX_GRANULES],
    /// Short block scale factors [granule][channel][scalefactor_band][window]
    pub s: [[[[i32; 3]; 13]; MAX_CHANNELS]; MAX_GRANULES],
}

impl Default for ShineScalefac {
    fn default() -> Self {
        Self {
            l: [[[0; 22]; MAX_CHANNELS]; MAX_GRANULES],
            s: [[[[0; 3]; 13]; MAX_CHANNELS]; MAX_GRANULES],
        }
    }
}

/// Wave configuration from shine (priv_shine_wave_t)
#[derive(Debug, Clone)]
pub struct ShineWave {
    pub channels: i32,
    pub sample_rate: u32,
}

/// MPEG configuration from shine (priv_shine_mpeg_t)  
#[derive(Debug, Clone)]
pub struct ShineMpeg {
    pub mode: i32,
    pub bitrate: u32,
    pub emphasis: i32,
    pub copyright: bool,
    pub original: bool,
    pub version: i32,
    pub granules_per_frame: i32,
}

/// Main shine global configuration structure following shine_global_config exactly
/// (ref/shine/src/lib/types.h:159-178)
/// 
/// This structure contains all the state needed for MP3 encoding and mirrors
/// the original shine implementation's global configuration structure.
#[derive(Debug)]
pub struct ShineGlobalConfig {
    /// Wave format configuration
    pub wave: ShineWave,
    /// MPEG encoding configuration  
    pub mpeg: ShineMpeg,
    /// Bitstream writer
    pub bs: BitstreamWriter,
    /// Side information
    pub side_info: ShineSideInfo,
    /// Side information length in bits
    pub sideinfo_len: i32,
    /// Mean bits per granule
    pub mean_bits: i32,
    /// Psychoacoustic ratios
    pub ratio: ShinePsyRatio,
    /// Scale factors
    pub scalefactor: ShineScalefac,
    /// Input buffers for each channel
    pub buffer: [Vec<i16>; MAX_CHANNELS],
    /// Perceptual entropy [channel][granule]
    pub pe: [[f64; MAX_GRANULES]; MAX_CHANNELS],
    /// Quantized coefficients [channel][granule][coefficient]
    pub l3_enc: [[[i32; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS],
    /// Subband samples [channel][granule+1][time][subband]
    pub l3_sb_sample: [[[[i32; SBLIMIT]; 18]; MAX_GRANULES + 1]; MAX_CHANNELS],
    /// MDCT frequency coefficients [channel][granule][coefficient]
    pub mdct_freq: [[[i32; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS],
    /// Bit reservoir size
    pub resv_size: i32,
    /// Maximum reservoir size
    pub resv_max: i32,
    /// L3 loop state
    pub l3loop: L3Loop,
    /// MDCT state
    pub mdct: Mdct,
    /// Subband state
    pub subband: Subband,
}

impl ShineGlobalConfig {
    /// Create a new shine global configuration from a Config
    pub fn new(config: Config) -> Result<Self> {
        // Convert Config to shine format
        let wave = ShineWave {
            channels: config.wave.channels as i32,
            sample_rate: config.wave.sample_rate,
        };
        
        let mpeg = ShineMpeg {
            mode: config.mpeg.mode as i32,
            bitrate: config.mpeg.bitrate,
            emphasis: config.mpeg.emphasis as i32,
            copyright: config.mpeg.copyright,
            original: config.mpeg.original,
            version: config.mpeg_version() as i32,
            granules_per_frame: match config.mpeg_version() {
                crate::config::MpegVersion::Mpeg1 => 2,
                crate::config::MpegVersion::Mpeg2 | crate::config::MpegVersion::Mpeg25 => 1,
            },
        };
        
        // Calculate side info length
        let sideinfo_len = if config.mpeg_version() == crate::config::MpegVersion::Mpeg1 {
            8 * if wave.channels == 1 { 4 + 17 } else { 4 + 32 }
        } else {
            8 * if wave.channels == 1 { 4 + 9 } else { 4 + 17 }
        };
        
        // Initialize buffers
        let mut buffer = [Vec::new(), Vec::new()];
        let samples_per_frame = config.samples_per_frame();
        for ch in 0..wave.channels as usize {
            buffer[ch] = Vec::with_capacity(samples_per_frame);
        }
        
        Ok(Self {
            wave,
            mpeg,
            bs: BitstreamWriter::new(2048),
            side_info: ShineSideInfo::default(),
            sideinfo_len,
            mean_bits: 0,
            ratio: ShinePsyRatio::default(),
            scalefactor: ShineScalefac::default(),
            buffer,
            pe: [[0.0; MAX_GRANULES]; MAX_CHANNELS],
            l3_enc: [[[0; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS],
            l3_sb_sample: [[[[0; SBLIMIT]; 18]; MAX_GRANULES + 1]; MAX_CHANNELS],
            mdct_freq: [[[0; GRANULE_SIZE]; MAX_GRANULES]; MAX_CHANNELS],
            resv_size: 0,
            resv_max: 0,
            l3loop: L3Loop::default(),
            mdct: Mdct::default(),
            subband: Subband::default(),
        })
    }
    
    /// Initialize the shine configuration following shine's initialization
    pub fn initialize(&mut self) -> Result<()> {
        // Initialize L3 loop tables
        self.shine_loop_initialise()?;
        
        // Initialize MDCT
        self.shine_mdct_initialise()?;
        
        // Initialize subband filter
        self.shine_subband_initialise()?;
        
        Ok(())
    }
    
    /// Initialize L3 loop tables following shine's shine_loop_initialise
    /// (ref/shine/src/lib/l3loop.c:325-350)
    fn shine_loop_initialise(&mut self) -> Result<()> {
        // quantize: stepsize conversion, fourth root of 2 table.
        // The table is inverted (negative power) from the equation given
        // in the spec because it is quicker to do x*y than x/y.
        // The 0.5 is for rounding.
        for i in 0..128 {
            self.l3loop.steptab[i] = (2.0_f64).powf((127 - i) as f64 / 4.0);
            if (self.l3loop.steptab[i] * 2.0) > 0x7fffffff as f64 {
                self.l3loop.steptabi[i] = 0x7fffffff;
            } else {
                // The table is multiplied by 2 to give an extra bit of accuracy.
                // In quantize, the long multiply does not shift its result left one
                // bit to compensate.
                self.l3loop.steptabi[i] = (self.l3loop.steptab[i] * 2.0 + 0.5) as i32;
            }
        }
        
        // Initialize int2idx table for x^(3/4) calculation
        for i in 0..10000 {
            self.l3loop.int2idx[i] = ((i as f64).powf(0.75) + 0.5) as i32;
        }
        
        Ok(())
    }
    
    /// Initialize MDCT following shine's shine_mdct_initialise
    /// (ref/shine/src/lib/l3mdct.c:30-50)
    fn shine_mdct_initialise(&mut self) -> Result<()> {
        // Initialize MDCT cosine table
        for i in 0..18 {
            for j in 0..36 {
                let angle = std::f64::consts::PI / 36.0 * (j as f64 + 0.5) * (i as f64 + 0.5);
                self.mdct.cos_l[i][j] = (angle.cos() * (1 << 15) as f64) as i32;
            }
        }
        
        Ok(())
    }
    
    /// Initialize subband filter following shine's shine_subband_initialise  
    /// (ref/shine/src/lib/l3subband.c:50-100)
    fn shine_subband_initialise(&mut self) -> Result<()> {
        // Initialize subband filter coefficients
        // This would load the actual filter coefficients from tables
        // For now, initialize to zero (will be implemented with proper tables)
        
        for ch in 0..MAX_CHANNELS {
            self.subband.off[ch] = 0;
            for i in 0..HAN_SIZE {
                self.subband.x[ch][i] = 0;
            }
        }
        
        // Initialize filter bank coefficients (simplified for now)
        for sb in 0..SBLIMIT {
            for i in 0..64 {
                self.subband.fl[sb][i] = 0;
            }
        }
        
        Ok(())
    }
}