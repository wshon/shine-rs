//! Lookup tables and constants for MP3 encoding
//!
//! Here are MPEG1 Table B.8 and MPEG2 Table B.1 -- Layer III scalefactor bands.
//! This module contains all the static lookup tables and constants
//! required for MP3 encoding, following shine's tables.c exactly.

/// Scale factor length tables (matches shine's shine_slen1_tab and shine_slen2_tab)
pub const SHINE_SLEN1_TAB: [i32; 16] = [0, 0, 0, 0, 3, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4];
pub const SHINE_SLEN2_TAB: [i32; 16] = [0, 1, 2, 3, 0, 1, 2, 3, 1, 2, 3, 1, 2, 3, 2, 3];

/// Valid samplerates (matches shine's samplerates array)
pub const SAMPLERATES: [i32; 9] = [
    44100, 48000, 32000, // MPEG-I
    22050, 24000, 16000, // MPEG-II
    11025, 12000, 8000,  // MPEG-2.5
];

/// Bitrate table for different MPEG versions (matches shine's bitrates array)
/// Index: [bitrate_index][mpeg_version] where mpeg_version: 2.5, reserved, II, I
pub const BITRATES: [[i32; 4]; 16] = [
    [-1, -1, -1, -1],   // 0000
    [8, -1, 8, 32],     // 0001
    [16, -1, 16, 40],   // 0010
    [24, -1, 24, 48],   // 0011
    [32, -1, 32, 56],   // 0100
    [40, -1, 40, 64],   // 0101
    [48, -1, 48, 80],   // 0110
    [56, -1, 56, 96],   // 0111
    [64, -1, 64, 112],  // 1000
    [-1, -1, 80, 128],  // 1001
    [-1, -1, 96, 160],  // 1010
    [-1, -1, 112, 192], // 1011
    [-1, -1, 128, 224], // 1100
    [-1, -1, 144, 256], // 1101
    [-1, -1, 160, 320], // 1110
    [-1, -1, -1, -1],   // 1111
];

/// Scale factor band indices for different sample rates (matches shine's shine_scale_fact_band_index)
/// Index: [sample_rate_index][band] where sample_rate_index corresponds to SAMPLERATES
pub const SHINE_SCALE_FACT_BAND_INDEX: [[i32; 23]; 9] = [
    // MPEG-I
    // Table B.8.b: 44.1 kHz
    [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 52, 62, 74, 90, 110, 134, 162, 196, 238, 288, 342, 418, 576],
    // Table B.8.c: 48 kHz
    [0, 4, 8, 12, 16, 20, 24, 30, 36, 42, 50, 60, 72, 88, 106, 128, 156, 190, 230, 276, 330, 384, 576],
    // Table B.8.a: 32 kHz
    [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 54, 66, 82, 102, 126, 156, 194, 240, 296, 364, 448, 550, 576],
    
    // MPEG-II
    // Table B.2.b: 22.05 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
    // Table B.2.c: 24 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 114, 136, 162, 194, 232, 278, 330, 394, 464, 540, 576],
    // Table B.2.a: 16 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 45, 66, 80, 96, 116, 140, 168, 200, 238, 248, 336, 396, 464, 522, 576],
    
    // MPEG-2.5
    // 11.025 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
    // 12 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
    // MPEG-2.5 8 kHz
    [0, 12, 24, 36, 48, 60, 72, 88, 108, 132, 160, 192, 232, 280, 336, 400, 476, 566, 568, 570, 572, 574, 576],
];
/// Subband filter window coefficients (matches shine's shine_enwindow)
/// These are the analysis window coefficients for the polyphase filterbank
/// Scaled and converted to fixed point (i32) from the original floating point values
/// Note: 0.035781 is shine_enwindow maximum value
/// Scale and convert to fixed point before storing (matches SHINE_EW macro)
const fn shine_ew(x: f64) -> i32 {
    (x * 0x7fffffff as f64) as i32
}

pub const SHINE_ENWINDOW: [i32; 512] = [
    // Values 0-9
    shine_ew(0.000000), shine_ew(-0.000000), shine_ew(-0.000000), shine_ew(-0.000000), shine_ew(-0.000000), 
    shine_ew(-0.000000), shine_ew(-0.000000), shine_ew(-0.000001), shine_ew(-0.000001), shine_ew(-0.000001),
    // Values 10-19
    shine_ew(-0.000001), shine_ew(-0.000001), shine_ew(-0.000001), shine_ew(-0.000002), shine_ew(-0.000002), 
    shine_ew(-0.000002), shine_ew(-0.000002), shine_ew(-0.000003), shine_ew(-0.000003), shine_ew(-0.000003),
    // Values 20-29
    shine_ew(-0.000004), shine_ew(-0.000004), shine_ew(-0.000005), shine_ew(-0.000005), shine_ew(-0.000006), 
    shine_ew(-0.000007), shine_ew(-0.000008), shine_ew(-0.000008), shine_ew(-0.000009), shine_ew(-0.000010),
    // Values 30-39
    shine_ew(-0.000011), shine_ew(-0.000012), shine_ew(-0.000014), shine_ew(-0.000015), shine_ew(-0.000017), 
    shine_ew(-0.000018), shine_ew(-0.000020), shine_ew(-0.000021), shine_ew(-0.000023), shine_ew(-0.000025),
    // Values 40-49
    shine_ew(-0.000028), shine_ew(-0.000030), shine_ew(-0.000032), shine_ew(-0.000035), shine_ew(-0.000038), 
    shine_ew(-0.000041), shine_ew(-0.000043), shine_ew(-0.000046), shine_ew(-0.000050), shine_ew(-0.000053),
    // Values 50-59
    shine_ew(-0.000056), shine_ew(-0.000060), shine_ew(-0.000063), shine_ew(-0.000066), shine_ew(-0.000070), 
    shine_ew(-0.000073), shine_ew(-0.000077), shine_ew(-0.000081), shine_ew(-0.000084), shine_ew(-0.000087),
    // Values 60-69
    shine_ew(-0.000091), shine_ew(-0.000093), shine_ew(-0.000096), shine_ew(-0.000099), shine_ew(0.000102), 
    shine_ew(0.000104), shine_ew(0.000106), shine_ew(0.000107), shine_ew(0.000108), shine_ew(0.000109),
    // Values 70-79
    shine_ew(0.000109), shine_ew(0.000108), shine_ew(0.000107), shine_ew(0.000105), shine_ew(0.000103), 
    shine_ew(0.000099), shine_ew(0.000095), shine_ew(0.000090), shine_ew(0.000084), shine_ew(0.000078),
    // Values 80-89
    shine_ew(0.000070), shine_ew(0.000061), shine_ew(0.000051), shine_ew(0.000040), shine_ew(0.000027), 
    shine_ew(0.000014), shine_ew(-0.000001), shine_ew(-0.000017), shine_ew(-0.000034), shine_ew(-0.000053),
    // Values 90-99
    shine_ew(-0.000073), shine_ew(-0.000094), shine_ew(-0.000116), shine_ew(-0.000140), shine_ew(-0.000165), 
    shine_ew(-0.000191), shine_ew(-0.000219), shine_ew(-0.000247), shine_ew(-0.000277), shine_ew(-0.000308),
    // Values 100-109
    shine_ew(-0.000339), shine_ew(-0.000371), shine_ew(-0.000404), shine_ew(-0.000438), shine_ew(-0.000473), 
    shine_ew(-0.000507), shine_ew(-0.000542), shine_ew(-0.000577), shine_ew(-0.000612), shine_ew(-0.000647),
    // Values 110-119
    shine_ew(-0.000681), shine_ew(-0.000714), shine_ew(-0.000747), shine_ew(-0.000779), shine_ew(-0.000810), 
    shine_ew(-0.000839), shine_ew(-0.000866), shine_ew(-0.000892), shine_ew(-0.000915), shine_ew(-0.000936),
    // Values 120-129
    shine_ew(-0.000954), shine_ew(-0.000969), shine_ew(-0.000981), shine_ew(-0.000989), shine_ew(-0.000994), 
    shine_ew(-0.000995), shine_ew(-0.000992), shine_ew(-0.000984), shine_ew(0.000971), shine_ew(0.000954),
    // Values 130-139
    shine_ew(0.000931), shine_ew(0.000903), shine_ew(0.000869), shine_ew(0.000829), shine_ew(0.000784), 
    shine_ew(0.000732), shine_ew(0.000674), shine_ew(0.000610), shine_ew(0.000539), shine_ew(0.000463),
    // Values 140-149
    shine_ew(0.000379), shine_ew(0.000288), shine_ew(0.000192), shine_ew(0.000088), shine_ew(-0.000021), 
    shine_ew(-0.000137), shine_ew(-0.000260), shine_ew(-0.000388), shine_ew(-0.000522), shine_ew(-0.000662),
    // Values 150-159
    shine_ew(-0.000807), shine_ew(-0.000957), shine_ew(-0.001111), shine_ew(-0.001270), shine_ew(-0.001432), 
    shine_ew(-0.001598), shine_ew(-0.001767), shine_ew(-0.001937), shine_ew(-0.002110), shine_ew(-0.002283),
    // Values 160-169
    shine_ew(-0.002457), shine_ew(-0.002631), shine_ew(-0.002803), shine_ew(-0.002974), shine_ew(-0.003142), 
    shine_ew(-0.003307), shine_ew(-0.003467), shine_ew(-0.003623), shine_ew(-0.003772), shine_ew(-0.003914),
    // Values 170-179
    shine_ew(-0.004049), shine_ew(-0.004175), shine_ew(-0.004291), shine_ew(-0.004396), shine_ew(-0.004490), 
    shine_ew(-0.004570), shine_ew(-0.004638), shine_ew(-0.004691), shine_ew(-0.004728), shine_ew(-0.004749),
    // Values 180-189
    shine_ew(-0.004752), shine_ew(-0.004737), shine_ew(-0.004703), shine_ew(-0.004649), shine_ew(-0.004574), 
    shine_ew(-0.004477), shine_ew(-0.004358), shine_ew(-0.004215), shine_ew(-0.004049), shine_ew(-0.003859),
    // Values 190-199
    shine_ew(-0.003643), shine_ew(-0.003402), shine_ew(0.003135), shine_ew(0.002841), shine_ew(0.002522), 
    shine_ew(0.002175), shine_ew(0.001801), shine_ew(0.001400), shine_ew(0.000971), shine_ew(0.000516),
    // Values 200-209
    shine_ew(0.000033), shine_ew(-0.000476), shine_ew(-0.001012), shine_ew(-0.001574), shine_ew(-0.002162), 
    shine_ew(-0.002774), shine_ew(-0.003411), shine_ew(-0.004072), shine_ew(-0.004756), shine_ew(-0.005462),
    // Values 210-219
    shine_ew(-0.006189), shine_ew(-0.006937), shine_ew(-0.007703), shine_ew(-0.008487), shine_ew(-0.009288), 
    shine_ew(-0.010104), shine_ew(-0.010933), shine_ew(-0.011775), shine_ew(-0.012628), shine_ew(-0.013489),
    // Values 220-229
    shine_ew(-0.014359), shine_ew(-0.015234), shine_ew(-0.016113), shine_ew(-0.016994), shine_ew(-0.017876), 
    shine_ew(-0.018757), shine_ew(-0.019634), shine_ew(-0.020507), shine_ew(-0.021372), shine_ew(-0.022229),
    // Values 230-239
    shine_ew(-0.023074), shine_ew(-0.023907), shine_ew(-0.024725), shine_ew(-0.025527), shine_ew(-0.026311), 
    shine_ew(-0.027074), shine_ew(-0.027815), shine_ew(-0.028533), shine_ew(-0.029225), shine_ew(-0.029890),
    // Values 240-249
    shine_ew(-0.030527), shine_ew(-0.031133), shine_ew(-0.031707), shine_ew(-0.032248), shine_ew(-0.032755), 
    shine_ew(-0.033226), shine_ew(-0.033660), shine_ew(-0.034056), shine_ew(-0.034413), shine_ew(-0.034730),
    // Values 250-259 (center point with maximum value)
    shine_ew(-0.035007), shine_ew(-0.035242), shine_ew(-0.035435), shine_ew(-0.035586), shine_ew(-0.035694), 
    shine_ew(-0.035759), shine_ew(0.035781), shine_ew(0.035759), shine_ew(0.035694), shine_ew(0.035586),
    // Values 260-269 (symmetric part begins)
    shine_ew(0.035435), shine_ew(0.035242), shine_ew(0.035007), shine_ew(0.034730), shine_ew(0.034413), 
    shine_ew(0.034056), shine_ew(0.033660), shine_ew(0.033226), shine_ew(0.032755), shine_ew(0.032248),
    // Values 270-279
    shine_ew(0.031707), shine_ew(0.031133), shine_ew(0.030527), shine_ew(0.029890), shine_ew(0.029225), 
    shine_ew(0.028533), shine_ew(0.027815), shine_ew(0.027074), shine_ew(0.026311), shine_ew(0.025527),
    // Values 280-289
    shine_ew(0.024725), shine_ew(0.023907), shine_ew(0.023074), shine_ew(0.022229), shine_ew(0.021372), 
    shine_ew(0.020507), shine_ew(0.019634), shine_ew(0.018757), shine_ew(0.017876), shine_ew(0.016994),
    // Values 290-299
    shine_ew(0.016113), shine_ew(0.015234), shine_ew(0.014359), shine_ew(0.013489), shine_ew(0.012628), 
    shine_ew(0.011775), shine_ew(0.010933), shine_ew(0.010104), shine_ew(0.009288), shine_ew(0.008487),
    // Values 300-309
    shine_ew(0.007703), shine_ew(0.006937), shine_ew(0.006189), shine_ew(0.005462), shine_ew(0.004756), 
    shine_ew(0.004072), shine_ew(0.003411), shine_ew(0.002774), shine_ew(0.002162), shine_ew(0.001574),
    // Values 310-319
    shine_ew(0.001012), shine_ew(0.000476), shine_ew(-0.000033), shine_ew(-0.000516), shine_ew(-0.000971), 
    shine_ew(-0.001400), shine_ew(-0.001801), shine_ew(-0.002175), shine_ew(-0.002522), shine_ew(-0.002841),
    // Values 320-329
    shine_ew(0.003135), shine_ew(0.003402), shine_ew(0.003643), shine_ew(0.003859), shine_ew(0.004049), 
    shine_ew(0.004215), shine_ew(0.004358), shine_ew(0.004477), shine_ew(0.004574), shine_ew(0.004649),
    // Values 330-339
    shine_ew(0.004703), shine_ew(0.004737), shine_ew(0.004752), shine_ew(0.004749), shine_ew(0.004728), 
    shine_ew(0.004691), shine_ew(0.004638), shine_ew(0.004570), shine_ew(0.004490), shine_ew(0.004396),
    // Values 340-349
    shine_ew(0.004291), shine_ew(0.004175), shine_ew(0.004049), shine_ew(0.003914), shine_ew(0.003772), 
    shine_ew(0.003623), shine_ew(0.003467), shine_ew(0.003307), shine_ew(0.003142), shine_ew(0.002974),
    // Values 350-359
    shine_ew(0.002803), shine_ew(0.002631), shine_ew(0.002457), shine_ew(0.002283), shine_ew(0.002110), 
    shine_ew(0.001937), shine_ew(0.001767), shine_ew(0.001598), shine_ew(0.001432), shine_ew(0.001270),
    // Values 360-369
    shine_ew(0.001111), shine_ew(0.000957), shine_ew(0.000807), shine_ew(0.000662), shine_ew(0.000522), 
    shine_ew(0.000388), shine_ew(0.000260), shine_ew(0.000137), shine_ew(0.000021), shine_ew(-0.000088),
    // Values 370-379
    shine_ew(-0.000192), shine_ew(-0.000288), shine_ew(-0.000379), shine_ew(-0.000463), shine_ew(-0.000539), 
    shine_ew(-0.000610), shine_ew(-0.000674), shine_ew(-0.000732), shine_ew(-0.000784), shine_ew(-0.000829),
    // Values 380-389
    shine_ew(-0.000869), shine_ew(-0.000903), shine_ew(-0.000931), shine_ew(-0.000954), shine_ew(0.000971), 
    shine_ew(0.000984), shine_ew(0.000992), shine_ew(0.000995), shine_ew(0.000994), shine_ew(0.000989),
    // Values 390-399
    shine_ew(0.000981), shine_ew(0.000969), shine_ew(0.000954), shine_ew(0.000936), shine_ew(0.000915), 
    shine_ew(0.000892), shine_ew(0.000866), shine_ew(0.000839), shine_ew(0.000810), shine_ew(0.000779),
    // Values 400-409
    shine_ew(0.000747), shine_ew(0.000714), shine_ew(0.000681), shine_ew(0.000647), shine_ew(0.000612), 
    shine_ew(0.000577), shine_ew(0.000542), shine_ew(0.000507), shine_ew(0.000473), shine_ew(0.000438),
    // Values 410-419
    shine_ew(0.000404), shine_ew(0.000371), shine_ew(0.000339), shine_ew(0.000308), shine_ew(0.000277), 
    shine_ew(0.000247), shine_ew(0.000219), shine_ew(0.000191), shine_ew(0.000165), shine_ew(0.000140),
    // Values 420-429
    shine_ew(0.000116), shine_ew(0.000094), shine_ew(0.000073), shine_ew(0.000053), shine_ew(0.000034), 
    shine_ew(0.000017), shine_ew(0.000001), shine_ew(-0.000014), shine_ew(-0.000027), shine_ew(-0.000040),
    // Values 430-439
    shine_ew(-0.000051), shine_ew(-0.000061), shine_ew(-0.000070), shine_ew(-0.000078), shine_ew(-0.000084), 
    shine_ew(-0.000090), shine_ew(-0.000095), shine_ew(-0.000099), shine_ew(-0.000103), shine_ew(-0.000105),
    // Values 440-449
    shine_ew(-0.000107), shine_ew(-0.000108), shine_ew(-0.000109), shine_ew(-0.000109), shine_ew(-0.000108), 
    shine_ew(-0.000107), shine_ew(-0.000106), shine_ew(-0.000104), shine_ew(0.000102), shine_ew(0.000099),
    // Values 450-459
    shine_ew(0.000096), shine_ew(0.000093), shine_ew(0.000091), shine_ew(0.000087), shine_ew(0.000084), 
    shine_ew(0.000081), shine_ew(0.000077), shine_ew(0.000073), shine_ew(0.000070), shine_ew(0.000066),
    // Values 460-469
    shine_ew(0.000063), shine_ew(0.000060), shine_ew(0.000056), shine_ew(0.000053), shine_ew(0.000050), 
    shine_ew(0.000046), shine_ew(0.000043), shine_ew(0.000041), shine_ew(0.000038), shine_ew(0.000035),
    // Values 470-479
    shine_ew(0.000032), shine_ew(0.000030), shine_ew(0.000028), shine_ew(0.000025), shine_ew(0.000023), 
    shine_ew(0.000021), shine_ew(0.000020), shine_ew(0.000018), shine_ew(0.000017), shine_ew(0.000015),
    // Values 480-489
    shine_ew(0.000014), shine_ew(0.000012), shine_ew(0.000011), shine_ew(0.000010), shine_ew(0.000009), 
    shine_ew(0.000008), shine_ew(0.000008), shine_ew(0.000007), shine_ew(0.000006), shine_ew(0.000005),
    // Values 490-499
    shine_ew(0.000005), shine_ew(0.000004), shine_ew(0.000004), shine_ew(0.000003), shine_ew(0.000003), 
    shine_ew(0.000003), shine_ew(0.000002), shine_ew(0.000002), shine_ew(0.000002), shine_ew(0.000002),
    // Values 500-509
    shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001), 
    shine_ew(0.000001), shine_ew(0.000000), shine_ew(0.000000), shine_ew(0.000000), shine_ew(0.000000),
    // Final 2 values (510-511)
    shine_ew(0.000000), shine_ew(0.000000)
];

/// Helper function to get sample rate index from sample rate value
pub fn get_sample_rate_index(sample_rate: i32) -> Option<usize> {
    SAMPLERATES.iter().position(|&sr| sr == sample_rate)
}

/// Helper function to get bitrate from bitrate index and MPEG version
pub fn get_bitrate(bitrate_index: usize, mpeg_version: usize) -> Option<i32> {
    if bitrate_index < 16 && mpeg_version < 4 {
        let bitrate = BITRATES[bitrate_index][mpeg_version];
        if bitrate > 0 {
            Some(bitrate)
        } else {
            None
        }
    } else {
        None
    }
}