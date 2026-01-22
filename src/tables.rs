//! Lookup tables and constants for MP3 encoding
//!
//! This module contains all the static lookup tables and constants
//! required for MP3 encoding, including sample rate tables, bitrate tables,
//! subband filter coefficients, MDCT cosine tables, and Huffman code tables.

/// Sample rates for different MPEG versions (matches shine samplerates array)
pub const SAMPLE_RATES: [u32; 9] = [
    44100, 48000, 32000, // MPEG-1
    22050, 24000, 16000, // MPEG-2
    11025, 12000, 8000,  // MPEG-2.5
];

/// Bitrate table for different MPEG versions (matches shine bitrates array)
/// Index: [bitrate_index][mpeg_version] where mpeg_version: 0=2.5, 1=reserved, 2=II, 3=I
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

/// Scale factor band indices for different sample rates (matches shine_scale_fact_band_index)
/// Index: [sample_rate_index][band] where sample_rate_index corresponds to SAMPLE_RATES
pub const SCALE_FACT_BAND_INDEX: [[i32; 23]; 9] = [
    // MPEG-1
    // 44.1 kHz (Table B.8.b)
    [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 52, 62, 74, 90, 110, 134, 162, 196, 238, 288, 342, 418, 576],
    // 48 kHz (Table B.8.c)
    [0, 4, 8, 12, 16, 20, 24, 30, 36, 42, 50, 60, 72, 88, 106, 128, 156, 190, 230, 276, 330, 384, 576],
    // 32 kHz (Table B.8.a)
    [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 54, 66, 82, 102, 126, 156, 194, 240, 296, 364, 448, 550, 576],
    
    // MPEG-2
    // 22.05 kHz (Table B.2.b)
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
    // 24 kHz (Table B.2.c)
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 114, 136, 162, 194, 232, 278, 330, 394, 464, 540, 576],
    // 16 kHz (Table B.2.a)
    [0, 6, 12, 18, 24, 30, 36, 44, 45, 66, 80, 96, 116, 140, 168, 200, 238, 248, 336, 396, 464, 522, 576],
    
    // MPEG-2.5
    // 11.025 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
    // 12 kHz
    [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
    // 8 kHz
    [0, 12, 24, 36, 48, 60, 72, 88, 108, 132, 160, 192, 232, 280, 336, 400, 476, 566, 568, 570, 572, 574, 576],
];

/// Scale factor length tables (matches shine_slen1_tab and shine_slen2_tab)
pub const SLEN1_TAB: [i32; 16] = [0, 0, 0, 0, 3, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4];
pub const SLEN2_TAB: [i32; 16] = [0, 1, 2, 3, 0, 1, 2, 3, 1, 2, 3, 1, 2, 3, 2, 3];

/// Subband filter window coefficients (matches shine_enwindow)
/// These are the analysis window coefficients for the polyphase filterbank
/// Scaled and converted to fixed point (i32) from the original floating point values
/// Note: 0.035781 is shine_enwindow maximum value
/// Scale and convert to fixed point before storing (matches SHINE_EW macro)
const fn shine_ew(x: f64) -> i32 {
    (x * 0x7fffffff as f64) as i32
}

pub const ENWINDOW: [i32; 512] = [
    // First 10 values
    shine_ew(0.000000), shine_ew(-0.000000), shine_ew(-0.000000), shine_ew(-0.000000), shine_ew(-0.000000), shine_ew(-0.000000),
    shine_ew(-0.000000), shine_ew(-0.000001), shine_ew(-0.000001), shine_ew(-0.000001),
    // Second 10 values
    shine_ew(-0.000001), shine_ew(-0.000001), shine_ew(-0.000001), shine_ew(-0.000002), shine_ew(-0.000002), shine_ew(-0.000002),
    shine_ew(-0.000002), shine_ew(-0.000003), shine_ew(-0.000003), shine_ew(-0.000003),
    // Third 10 values
    shine_ew(-0.000004), shine_ew(-0.000004), shine_ew(-0.000005), shine_ew(-0.000005), shine_ew(-0.000006), shine_ew(-0.000007),
    shine_ew(-0.000008), shine_ew(-0.000008), shine_ew(-0.000009), shine_ew(-0.000010),
    // Fourth 10 values
    shine_ew(-0.000011), shine_ew(-0.000012), shine_ew(-0.000014), shine_ew(-0.000015), shine_ew(-0.000017), shine_ew(-0.000018),
    shine_ew(-0.000020), shine_ew(-0.000021), shine_ew(-0.000023), shine_ew(-0.000025),
    // Fifth 10 values
    shine_ew(-0.000028), shine_ew(-0.000030), shine_ew(-0.000032), shine_ew(-0.000035), shine_ew(-0.000038), shine_ew(-0.000041),
    shine_ew(-0.000043), shine_ew(-0.000046), shine_ew(-0.000050), shine_ew(-0.000053),
    // Sixth 10 values
    shine_ew(-0.000056), shine_ew(-0.000060), shine_ew(-0.000063), shine_ew(-0.000066), shine_ew(-0.000070), shine_ew(-0.000073),
    shine_ew(-0.000077), shine_ew(-0.000081), shine_ew(-0.000084), shine_ew(-0.000087),
    // Seventh 10 values
    shine_ew(-0.000091), shine_ew(-0.000093), shine_ew(-0.000096), shine_ew(-0.000099), shine_ew(0.000102), shine_ew(0.000104),
    shine_ew(0.000106), shine_ew(0.000107), shine_ew(0.000108), shine_ew(0.000109),
    // Eighth 10 values
    shine_ew(0.000109), shine_ew(0.000108), shine_ew(0.000107), shine_ew(0.000105), shine_ew(0.000103), shine_ew(0.000099),
    shine_ew(0.000095), shine_ew(0.000090), shine_ew(0.000084), shine_ew(0.000078),
    // Ninth 10 values
    shine_ew(0.000070), shine_ew(0.000061), shine_ew(0.000051), shine_ew(0.000040), shine_ew(0.000027), shine_ew(0.000014),
    shine_ew(-0.000001), shine_ew(-0.000017), shine_ew(-0.000034), shine_ew(-0.000053),
    // Tenth 10 values
    shine_ew(-0.000073), shine_ew(-0.000094), shine_ew(-0.000116), shine_ew(-0.000140), shine_ew(-0.000165), shine_ew(-0.000191),
    shine_ew(-0.000219), shine_ew(-0.000247), shine_ew(-0.000277), shine_ew(-0.000308),
    // Eleventh 10 values
    shine_ew(-0.000339), shine_ew(-0.000371), shine_ew(-0.000404), shine_ew(-0.000438), shine_ew(-0.000473), shine_ew(-0.000507),
    shine_ew(-0.000542), shine_ew(-0.000577), shine_ew(-0.000612), shine_ew(-0.000647),
    // Twelfth 10 values
    shine_ew(-0.000681), shine_ew(-0.000714), shine_ew(-0.000747), shine_ew(-0.000779), shine_ew(-0.000810), shine_ew(-0.000839),
    shine_ew(-0.000866), shine_ew(-0.000892), shine_ew(-0.000915), shine_ew(-0.000936),
    // Thirteenth 10 values
    shine_ew(-0.000954), shine_ew(-0.000969), shine_ew(-0.000981), shine_ew(-0.000989), shine_ew(-0.000994), shine_ew(-0.000995),
    shine_ew(-0.000992), shine_ew(-0.000984), shine_ew(0.000971), shine_ew(0.000954),
    // Fourteenth 10 values
    shine_ew(0.000931), shine_ew(0.000903), shine_ew(0.000869), shine_ew(0.000829), shine_ew(0.000784), shine_ew(0.000732),
    shine_ew(0.000674), shine_ew(0.000610), shine_ew(0.000539), shine_ew(0.000463),
    // Fifteenth 10 values
    shine_ew(0.000379), shine_ew(0.000288), shine_ew(0.000192), shine_ew(0.000088), shine_ew(-0.000021), shine_ew(-0.000137),
    shine_ew(-0.000260), shine_ew(-0.000388), shine_ew(-0.000522), shine_ew(-0.000662),
    // Sixteenth 10 values
    shine_ew(-0.000807), shine_ew(-0.000957), shine_ew(-0.001111), shine_ew(-0.001270), shine_ew(-0.001432), shine_ew(-0.001598),
    shine_ew(-0.001767), shine_ew(-0.001937), shine_ew(-0.002110), shine_ew(-0.002283),
    // Seventeenth 10 values
    shine_ew(-0.002457), shine_ew(-0.002631), shine_ew(-0.002803), shine_ew(-0.002974), shine_ew(-0.003142), shine_ew(-0.003307),
    shine_ew(-0.003467), shine_ew(-0.003623), shine_ew(-0.003772), shine_ew(-0.003914),
    // Eighteenth 10 values
    shine_ew(-0.004049), shine_ew(-0.004175), shine_ew(-0.004291), shine_ew(-0.004396), shine_ew(-0.004490), shine_ew(-0.004570),
    shine_ew(-0.004638), shine_ew(-0.004691), shine_ew(-0.004728), shine_ew(-0.004749),
    // Nineteenth 10 values
    shine_ew(-0.004752), shine_ew(-0.004737), shine_ew(-0.004703), shine_ew(-0.004649), shine_ew(-0.004574), shine_ew(-0.004477),
    shine_ew(-0.004358), shine_ew(-0.004215), shine_ew(-0.004049), shine_ew(-0.003859),
    // Twentieth 10 values
    shine_ew(-0.003643), shine_ew(-0.003402), shine_ew(0.003135), shine_ew(0.002841), shine_ew(0.002522), shine_ew(0.002175),
    shine_ew(0.001801), shine_ew(0.001400), shine_ew(0.000971), shine_ew(0.000516),
    // Twenty-first 10 values
    shine_ew(0.000033), shine_ew(-0.000476), shine_ew(-0.001012), shine_ew(-0.001574), shine_ew(-0.002162), shine_ew(-0.002774),
    shine_ew(-0.003411), shine_ew(-0.004072), shine_ew(-0.004756), shine_ew(-0.005462),
    // Twenty-second 10 values
    shine_ew(-0.006189), shine_ew(-0.006937), shine_ew(-0.007703), shine_ew(-0.008487), shine_ew(-0.009288), shine_ew(-0.010104),
    shine_ew(-0.010933), shine_ew(-0.011775), shine_ew(-0.012628), shine_ew(-0.013489),
    // Twenty-third 10 values
    shine_ew(-0.014359), shine_ew(-0.015234), shine_ew(-0.016113), shine_ew(-0.016994), shine_ew(-0.017876), shine_ew(-0.018757),
    shine_ew(-0.019634), shine_ew(-0.020507), shine_ew(-0.021372), shine_ew(-0.022229),
    // Twenty-fourth 10 values
    shine_ew(-0.023074), shine_ew(-0.023907), shine_ew(-0.024725), shine_ew(-0.025527), shine_ew(-0.026311), shine_ew(-0.027074),
    shine_ew(-0.027815), shine_ew(-0.028533), shine_ew(-0.029225), shine_ew(-0.029890),
    // Twenty-fifth 10 values
    shine_ew(-0.030527), shine_ew(-0.031133), shine_ew(-0.031707), shine_ew(-0.032248), shine_ew(-0.032755), shine_ew(-0.033226),
    shine_ew(-0.033660), shine_ew(-0.034056), shine_ew(-0.034413), shine_ew(-0.034730),
    // Twenty-sixth 10 values (center point with maximum value)
    shine_ew(-0.035007), shine_ew(-0.035242), shine_ew(-0.035435), shine_ew(-0.035586), shine_ew(-0.035694), shine_ew(-0.035759),
    shine_ew(0.035781), shine_ew(0.035759), shine_ew(0.035694), shine_ew(0.035586),
    // Twenty-seventh 10 values (symmetric part begins)
    shine_ew(0.035435), shine_ew(0.035242), shine_ew(0.035007), shine_ew(0.034730), shine_ew(0.034413), shine_ew(0.034056),
    shine_ew(0.033660), shine_ew(0.033226), shine_ew(0.032755), shine_ew(0.032248),
    // Twenty-eighth 10 values
    shine_ew(0.031707), shine_ew(0.031133), shine_ew(0.030527), shine_ew(0.029890), shine_ew(0.029225), shine_ew(0.028533),
    shine_ew(0.027815), shine_ew(0.027074), shine_ew(0.026311), shine_ew(0.025527),
    // Twenty-ninth 10 values
    shine_ew(0.024725), shine_ew(0.023907), shine_ew(0.023074), shine_ew(0.022229), shine_ew(0.021372), shine_ew(0.020507),
    shine_ew(0.019634), shine_ew(0.018757), shine_ew(0.017876), shine_ew(0.016994),
    // Thirtieth 10 values
    shine_ew(0.016113), shine_ew(0.015234), shine_ew(0.014359), shine_ew(0.013489), shine_ew(0.012628), shine_ew(0.011775),
    shine_ew(0.010933), shine_ew(0.010104), shine_ew(0.009288), shine_ew(0.008487),
    // Thirty-first 10 values
    shine_ew(0.007703), shine_ew(0.006937), shine_ew(0.006189), shine_ew(0.005462), shine_ew(0.004756), shine_ew(0.004072),
    shine_ew(0.003411), shine_ew(0.002774), shine_ew(0.002162), shine_ew(0.001574),
    // Thirty-second 10 values
    shine_ew(0.001012), shine_ew(0.000476), shine_ew(-0.000033), shine_ew(-0.000516), shine_ew(-0.000971), shine_ew(-0.001400),
    shine_ew(-0.001801), shine_ew(-0.002175), shine_ew(-0.002522), shine_ew(-0.002841),
    // Thirty-third 10 values
    shine_ew(0.003135), shine_ew(0.003402), shine_ew(0.003643), shine_ew(0.003859), shine_ew(0.004049), shine_ew(0.004215),
    shine_ew(0.004358), shine_ew(0.004477), shine_ew(0.004574), shine_ew(0.004649),
    // Thirty-fourth 10 values
    shine_ew(0.004703), shine_ew(0.004737), shine_ew(0.004752), shine_ew(0.004749), shine_ew(0.004728), shine_ew(0.004691),
    shine_ew(0.004638), shine_ew(0.004570), shine_ew(0.004490), shine_ew(0.004396),
    // Thirty-fifth 10 values
    shine_ew(0.004291), shine_ew(0.004175), shine_ew(0.004049), shine_ew(0.003914), shine_ew(0.003772), shine_ew(0.003623),
    shine_ew(0.003467), shine_ew(0.003307), shine_ew(0.003142), shine_ew(0.002974),
    // Thirty-sixth 10 values
    shine_ew(0.002803), shine_ew(0.002631), shine_ew(0.002457), shine_ew(0.002283), shine_ew(0.002110), shine_ew(0.001937),
    shine_ew(0.001767), shine_ew(0.001598), shine_ew(0.001432), shine_ew(0.001270),
    // Thirty-seventh 10 values
    shine_ew(0.001111), shine_ew(0.000957), shine_ew(0.000807), shine_ew(0.000662), shine_ew(0.000522), shine_ew(0.000388),
    shine_ew(0.000260), shine_ew(0.000137), shine_ew(0.000021), shine_ew(-0.000088),
    // Thirty-eighth 10 values
    shine_ew(-0.000192), shine_ew(-0.000288), shine_ew(-0.000379), shine_ew(-0.000463), shine_ew(-0.000539), shine_ew(-0.000610),
    shine_ew(-0.000674), shine_ew(-0.000732), shine_ew(-0.000784), shine_ew(-0.000829),
    // Thirty-ninth 10 values
    shine_ew(-0.000869), shine_ew(-0.000903), shine_ew(-0.000931), shine_ew(-0.000954), shine_ew(0.000971), shine_ew(0.000984),
    shine_ew(0.000992), shine_ew(0.000995), shine_ew(0.000994), shine_ew(0.000989),
    // Fortieth 10 values
    shine_ew(0.000981), shine_ew(0.000969), shine_ew(0.000954), shine_ew(0.000936), shine_ew(0.000915), shine_ew(0.000892),
    shine_ew(0.000866), shine_ew(0.000839), shine_ew(0.000810), shine_ew(0.000779),
    // Forty-first 10 values
    shine_ew(0.000747), shine_ew(0.000714), shine_ew(0.000681), shine_ew(0.000647), shine_ew(0.000612), shine_ew(0.000577),
    shine_ew(0.000542), shine_ew(0.000507), shine_ew(0.000473), shine_ew(0.000438),
    // Forty-second 10 values
    shine_ew(0.000404), shine_ew(0.000371), shine_ew(0.000339), shine_ew(0.000308), shine_ew(0.000277), shine_ew(0.000247),
    shine_ew(0.000219), shine_ew(0.000191), shine_ew(0.000165), shine_ew(0.000140),
    // Forty-third 10 values
    shine_ew(0.000116), shine_ew(0.000094), shine_ew(0.000073), shine_ew(0.000053), shine_ew(0.000034), shine_ew(0.000017),
    shine_ew(0.000001), shine_ew(-0.000014), shine_ew(-0.000027), shine_ew(-0.000040),
    // Forty-fourth 10 values
    shine_ew(-0.000051), shine_ew(-0.000061), shine_ew(-0.000070), shine_ew(-0.000078), shine_ew(-0.000084), shine_ew(-0.000090),
    shine_ew(-0.000095), shine_ew(-0.000099), shine_ew(-0.000103), shine_ew(-0.000105),
    // Forty-fifth 10 values
    shine_ew(-0.000107), shine_ew(-0.000108), shine_ew(-0.000109), shine_ew(-0.000109), shine_ew(-0.000108), shine_ew(-0.000107),
    shine_ew(-0.000106), shine_ew(-0.000104), shine_ew(0.000102), shine_ew(0.000099),
    // Forty-sixth 10 values
    shine_ew(0.000096), shine_ew(0.000093), shine_ew(0.000091), shine_ew(0.000087), shine_ew(0.000084), shine_ew(0.000081),
    shine_ew(0.000077), shine_ew(0.000073), shine_ew(0.000070), shine_ew(0.000066),
    // Forty-seventh 10 values
    shine_ew(0.000063), shine_ew(0.000060), shine_ew(0.000056), shine_ew(0.000053), shine_ew(0.000050), shine_ew(0.000046),
    shine_ew(0.000043), shine_ew(0.000041), shine_ew(0.000038), shine_ew(0.000035),
    // Forty-eighth 10 values
    shine_ew(0.000032), shine_ew(0.000030), shine_ew(0.000028), shine_ew(0.000025), shine_ew(0.000023), shine_ew(0.000021),
    shine_ew(0.000020), shine_ew(0.000018), shine_ew(0.000017), shine_ew(0.000015),
    // Forty-ninth 10 values
    shine_ew(0.000014), shine_ew(0.000012), shine_ew(0.000011), shine_ew(0.000010), shine_ew(0.000009), shine_ew(0.000008),
    shine_ew(0.000008), shine_ew(0.000007), shine_ew(0.000006), shine_ew(0.000005),
    // Fiftieth 10 values
    shine_ew(0.000005), shine_ew(0.000004), shine_ew(0.000004), shine_ew(0.000003), shine_ew(0.000003), shine_ew(0.000003),
    shine_ew(0.000002), shine_ew(0.000002), shine_ew(0.000002), shine_ew(0.000002),
    // Fifty-first 10 values
    shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001), shine_ew(0.000001),
    shine_ew(0.000000), shine_ew(0.000000), shine_ew(0.000000), shine_ew(0.000000),
    // Final 2 values (512 total)
    shine_ew(0.000000), shine_ew(0.000000)
];
/// MDCT cosine tables for different block types
/// These are precomputed cosine values for the MDCT transform
#[allow(clippy::excessive_precision)]
pub const MDCT_COS_TABLE: [[f32; 36]; 18] = [
    // Block type 0 (long blocks)
    [
        0.50190991877167369479, 0.50547095989754365998, 0.51213975715725043878,
        0.52191893391251657818, 0.53481482808326839967, 0.55082972903270762692,
        0.56996905566805595843, 0.59223893077681003726, 0.61764538699621229852,
        0.64619397662556434544, 0.67788456080286535588, 0.71272260728279436413,
        0.75071413042652648703, 0.79186394098711283508, 0.83617747073791821279,
        0.88366026910506238129, 0.93431734905808781532, 0.98815462227369399104,
        1.04517751321065688781, 1.10539169756337744319, 1.16880315886582681295,
        1.23541788906253310421, 1.30524188068711928833, 1.37828113635150621171,
        1.45454166206309244149, 1.53402946781909378840, 1.61675056264149239425,
        1.70270096274896663532, 1.79188668777166851934, 1.88431375266317467619,
        1.97998816738197424484, 2.07891594193989637947, 2.18110309226978806542,
        2.28655563322661699817, 2.39527957481127598311, 2.50728093703231462618,
    ],
    // Additional block types would be added here...
    // For now, we'll use the same values for all block types as a placeholder
    [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36],
    [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36], [0.0; 36],
];

/// Quantization step size table
/// These are the quantization step sizes for different global gain values
#[allow(clippy::excessive_precision)]
pub const QUANTIZATION_STEP_TABLE: [f32; 120] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    1.0, 1.0905077326652577, 1.1892071150027210, 1.2968395546510096,
    std::f32::consts::SQRT_2, 1.542_210_8, 1.681_792_9, 1.834_008_1,
    2.0, 2.1810154653305155, 2.3784142300054421, 2.5936791093020192,
    2.8284271247461903, 3.0844216508158816, 3.3635856610148581, 3.6680161728186848,
    4.0, 4.3620309306610311, 4.7568284600108841, 5.1873582186040384,
    5.6568542494923806, 6.1688433016317632, 6.7271713220297162, 7.3360323456373696,
    8.0, 8.7240618613220622, 9.5136569200217682, 10.374716437208077,
    11.313708498984761, 12.337686603263526, 13.454342644059432, 14.672064691274739,
    16.0, 17.448123722644124, 19.027313840043536, 20.749432874416154,
    22.627416997969522, 24.675373206527053, 26.908685288118865, 29.344129382549479,
    32.0, 34.896247445288249, 38.054627680087073, 41.498865748832308,
    45.254833995939045, 49.350746413054105, 53.817370576237730, 58.688258765098958,
    64.0, 69.792494890576498, 76.109255360174146, 82.997731497664616,
    90.509667991878090, 98.701492826108211, 107.634741152475460, 117.376517530197916,
    128.0, 139.584989781152996, 152.218510720348292, 165.995462995329232,
    181.019335983756180, 197.402985652216422, 215.269482304950920, 234.753035060395832,
    256.0, 279.169979562305992, 304.437021440696584, 331.990925990658464,
    362.038671967512360, 394.805971304432844, 430.538964609901840, 469.506070120791664,
    512.0, 558.339959124611984, 608.874042881393168, 663.981851981316928,
    724.077343935024720, 789.611942608865688, 861.077929219803680, 939.012140241583328,
    1024.0, 1116.679918249223968, 1217.748085762786336, 1327.963703962633856,
    1448.154687870049440, 1579.223885217731376, 1722.155858439607360, 1878.024280483166656,
];

/// Helper function to get sample rate index from sample rate value
pub fn get_sample_rate_index(sample_rate: u32) -> Option<usize> {
    SAMPLE_RATES.iter().position(|&sr| sr == sample_rate)
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
/// Huffman code table structure (matches shine's huffcodetab)
#[derive(Debug, Clone, Copy)]
pub struct HuffmanTable {
    pub xlen: u32,          // max x-index
    pub ylen: u32,          // max y-index  
    pub linbits: u32,       // number of linbits
    pub linmax: u32,        // max number to be stored in linbits
    pub codes: &'static [u16],     // huffman codes
    pub lengths: &'static [u8],    // code lengths
}

// Huffman table data - codes and lengths for each table (from shine)
static T1_CODES: [u16; 4] = [1, 1, 1, 0];
static T1_LENGTHS: [u8; 4] = [1, 3, 2, 3];

static T2_CODES: [u16; 9] = [1, 2, 1, 3, 1, 1, 3, 2, 0];
static T2_LENGTHS: [u8; 9] = [1, 3, 6, 3, 3, 5, 5, 5, 6];

static T3_CODES: [u16; 9] = [3, 2, 1, 1, 1, 1, 3, 2, 0];
static T3_LENGTHS: [u8; 9] = [2, 2, 6, 3, 2, 5, 5, 5, 6];

static T5_CODES: [u16; 16] = [1, 2, 6, 5, 3, 1, 4, 4, 7, 5, 7, 1, 6, 1, 1, 0];
static T5_LENGTHS: [u8; 16] = [1, 3, 6, 7, 3, 3, 6, 7, 6, 6, 7, 8, 7, 6, 7, 8];

static T6_CODES: [u16; 16] = [7, 3, 5, 1, 6, 2, 3, 2, 5, 4, 4, 1, 3, 3, 2, 0];
static T6_LENGTHS: [u8; 16] = [3, 3, 5, 7, 3, 2, 4, 5, 4, 4, 5, 6, 6, 5, 6, 7];

static T7_CODES: [u16; 36] = [
    1, 2, 10, 19, 16, 10, 3, 3, 7, 10, 5, 3, 11, 4, 13, 17, 8, 4, 12, 11, 18, 15, 11, 2,
    7, 6, 9, 14, 3, 1, 6, 4, 5, 3, 2, 0
];
static T7_LENGTHS: [u8; 36] = [
    1, 3, 6, 8, 8, 9, 3, 4, 6, 7, 7, 8, 6, 5, 7, 8, 8, 9, 7, 7, 8, 9, 9, 9,
    7, 7, 8, 9, 9, 10, 8, 8, 9, 10, 10, 10
];

static T8_CODES: [u16; 36] = [
    3, 4, 6, 18, 12, 5, 5, 1, 2, 16, 9, 3, 7, 3, 5, 14, 7, 3, 19, 17, 15, 13, 10, 4,
    13, 5, 8, 11, 5, 1, 12, 4, 4, 1, 1, 0
];
static T8_LENGTHS: [u8; 36] = [
    2, 3, 6, 8, 8, 9, 3, 2, 4, 8, 8, 8, 6, 4, 6, 8, 8, 9, 8, 8, 8, 9, 9, 10,
    8, 7, 8, 9, 10, 10, 9, 8, 9, 9, 11, 11
];

static T9_CODES: [u16; 36] = [
    7, 5, 9, 14, 15, 7, 6, 4, 5, 5, 6, 7, 7, 6, 8, 8, 8, 5, 15, 6, 9, 10, 5, 1,
    11, 7, 9, 6, 4, 1, 14, 4, 6, 2, 6, 0
];
static T9_LENGTHS: [u8; 36] = [
    3, 3, 5, 6, 8, 9, 3, 3, 4, 5, 6, 8, 4, 4, 5, 6, 7, 8, 6, 5, 6, 7, 7, 8,
    7, 6, 7, 7, 8, 9, 8, 7, 8, 8, 9, 9
];

static T10_CODES: [u16; 64] = [
    1, 2, 10, 23, 35, 30, 12, 17, 3, 3, 8, 12, 18, 21, 12, 7,
    11, 9, 15, 21, 32, 40, 19, 6, 14, 13, 22, 34, 46, 23, 18, 7,
    20, 19, 33, 47, 27, 22, 9, 3, 31, 22, 41, 26, 21, 20, 5, 3,
    14, 13, 10, 11, 16, 6, 5, 1, 9, 8, 7, 8, 4, 4, 2, 0
];
static T10_LENGTHS: [u8; 64] = [
    1, 3, 6, 8, 9, 9, 9, 10, 3, 4, 6, 7, 8, 9, 8, 8,
    6, 6, 7, 8, 9, 10, 9, 9, 7, 7, 8, 9, 10, 10, 9, 10,
    8, 8, 9, 10, 10, 10, 10, 10, 9, 9, 10, 10, 11, 11, 10, 11,
    8, 8, 9, 10, 10, 10, 11, 11, 9, 8, 9, 10, 10, 11, 11, 11
];

static T11_CODES: [u16; 64] = [
    3, 4, 10, 24, 34, 33, 21, 15, 5, 3, 4, 10, 32, 17, 11, 10,
    11, 7, 13, 18, 30, 31, 20, 5, 25, 11, 19, 59, 27, 18, 12, 5,
    35, 33, 31, 58, 30, 16, 7, 5, 28, 26, 32, 19, 17, 15, 8, 14,
    14, 12, 9, 13, 14, 9, 4, 1, 11, 4, 6, 6, 6, 3, 2, 0
];
static T11_LENGTHS: [u8; 64] = [
    2, 3, 5, 7, 8, 9, 8, 9, 3, 3, 4, 6, 8, 8, 7, 8,
    5, 5, 6, 7, 8, 9, 8, 8, 7, 6, 7, 9, 8, 10, 8, 9,
    8, 8, 8, 9, 9, 10, 9, 10, 8, 8, 9, 10, 10, 11, 10, 11,
    8, 7, 7, 8, 9, 10, 10, 10, 8, 7, 8, 9, 10, 10, 10, 10
];

static T12_CODES: [u16; 64] = [
    9, 6, 16, 33, 41, 39, 38, 26, 7, 5, 6, 9, 23, 16, 26, 11,
    17, 7, 11, 14, 21, 30, 10, 7, 17, 10, 15, 12, 18, 28, 14, 5,
    32, 13, 22, 19, 18, 16, 9, 5, 40, 17, 31, 29, 17, 13, 4, 2,
    27, 12, 11, 15, 10, 7, 4, 1, 27, 12, 8, 12, 6, 3, 1, 0
];
static T12_LENGTHS: [u8; 64] = [
    4, 3, 5, 7, 8, 9, 9, 9, 3, 3, 4, 5, 7, 7, 8, 8,
    5, 4, 5, 6, 7, 8, 7, 8, 6, 5, 6, 6, 7, 8, 8, 8,
    7, 6, 7, 7, 8, 8, 8, 9, 8, 7, 8, 8, 8, 9, 8, 9,
    8, 7, 7, 8, 8, 9, 9, 10, 9, 8, 8, 9, 9, 9, 9, 10
];

static T13_CODES: [u16; 256] = [
    1, 5, 14, 21, 34, 51, 46, 71, 42, 52, 68, 52, 67, 44, 43, 19, 3, 4,
    12, 19, 31, 26, 44, 33, 31, 24, 32, 24, 31, 35, 22, 14, 15, 13, 23, 36,
    59, 49, 77, 65, 29, 40, 30, 40, 27, 33, 42, 16, 22, 20, 37, 61, 56, 79,
    73, 64, 43, 76, 56, 37, 26, 31, 25, 14, 35, 16, 60, 57, 97, 75, 114, 91,
    54, 73, 55, 41, 48, 53, 23, 24, 58, 27, 50, 96, 76, 70, 93, 84, 77, 58,
    79, 29, 74, 49, 41, 17, 47, 45, 78, 74, 115, 94, 90, 79, 69, 83, 71, 50,
    59, 38, 36, 15, 72, 34, 56, 95, 92, 85, 91, 90, 86, 73, 77, 65, 51, 44,
    43, 42, 43, 20, 30, 44, 55, 78, 72, 87, 78, 61, 46, 54, 37, 30, 20, 16,
    53, 25, 41, 37, 44, 59, 54, 81, 66, 76, 57, 54, 37, 18, 39, 11, 35, 33,
    31, 57, 42, 82, 72, 80, 47, 58, 55, 21, 22, 26, 38, 22, 53, 25, 23, 38,
    70, 60, 51, 36, 55, 26, 34, 23, 27, 14, 9, 7, 34, 32, 28, 39, 49, 75,
    30, 52, 48, 40, 52, 28, 18, 17, 9, 5, 45, 21, 34, 64, 56, 50, 49, 45,
    31, 19, 12, 15, 10, 7, 6, 3, 48, 23, 20, 39, 36, 35, 53, 21, 16, 23,
    13, 10, 6, 1, 4, 2, 16, 15, 17, 27, 25, 20, 29, 11, 17, 12, 16, 8,
    1, 1, 0, 1
];
static T13_LENGTHS: [u8; 256] = [
    1, 4, 6, 7, 8, 9, 9, 10, 9, 10, 11, 11, 12, 12, 13, 13, 3, 4, 6,
    7, 8, 8, 9, 9, 9, 9, 10, 10, 11, 12, 12, 12, 6, 6, 7, 8, 9, 9,
    10, 10, 9, 10, 10, 11, 11, 12, 13, 13, 7, 7, 8, 9, 9, 10, 10, 10, 10,
    11, 11, 11, 11, 12, 13, 13, 8, 7, 9, 9, 10, 10, 11, 11, 10, 11, 11, 12,
    12, 13, 13, 14, 9, 8, 9, 10, 10, 10, 11, 11, 11, 11, 12, 11, 13, 13, 14,
    14, 9, 9, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 13, 13, 14, 14, 10, 9,
    10, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 14, 16, 16, 9, 8, 9, 10, 10,
    11, 11, 12, 12, 12, 12, 13, 13, 14, 15, 15, 10, 9, 10, 10, 11, 11, 11, 13,
    12, 13, 13, 14, 14, 14, 16, 15, 10, 10, 10, 11, 11, 12, 12, 13, 12, 13, 14,
    13, 14, 15, 16, 17, 11, 10, 10, 11, 12, 12, 12, 12, 13, 13, 13, 14, 15, 15,
    15, 16, 11, 11, 11, 12, 12, 13, 12, 13, 14, 14, 15, 15, 15, 16, 16, 16, 12,
    11, 12, 13, 13, 13, 14, 14, 14, 14, 14, 15, 16, 15, 16, 16, 13, 12, 12, 13,
    13, 13, 15, 14, 14, 17, 15, 15, 15, 17, 16, 16, 12, 12, 13, 14, 14, 14, 15,
    14, 15, 15, 16, 16, 19, 18, 19, 16
];

static T15_CODES: [u16; 256] = [
    7, 12, 18, 53, 47, 76, 124, 108, 89, 123, 108, 119, 107, 81, 122, 63,
    13, 5, 16, 27, 46, 36, 61, 51, 42, 70, 52, 83, 65, 41, 59, 36,
    19, 17, 15, 24, 41, 34, 59, 48, 40, 64, 50, 78, 62, 80, 56, 33,
    29, 28, 25, 43, 39, 63, 55, 93, 76, 59, 93, 72, 54, 75, 50, 29,
    52, 22, 42, 40, 67, 57, 95, 79, 72, 57, 89, 69, 49, 66, 46, 27,
    77, 37, 35, 66, 58, 52, 91, 74, 62, 48, 79, 63, 90, 62, 40, 38,
    125, 32, 60, 56, 50, 92, 78, 65, 55, 87, 71, 51, 73, 51, 70, 30,
    109, 53, 49, 94, 88, 75, 66, 122, 91, 73, 56, 42, 64, 44, 21, 25,
    90, 43, 41, 77, 73, 63, 56, 92, 77, 66, 47, 67, 48, 53, 36, 20,
    71, 34, 67, 60, 58, 49, 88, 76, 67, 106, 71, 54, 38, 39, 23, 15,
    109, 53, 51, 47, 90, 82, 58, 57, 48, 72, 57, 41, 23, 27, 62, 9,
    86, 42, 40, 37, 70, 64, 52, 43, 70, 55, 42, 25, 29, 18, 11, 11,
    118, 68, 30, 55, 50, 46, 74, 65, 49, 39, 24, 16, 22, 13, 14, 7,
    91, 44, 39, 38, 34, 63, 52, 45, 31, 52, 28, 19, 14, 8, 9, 3,
    123, 60, 58, 53, 47, 43, 32, 22, 37, 24, 17, 12, 15, 10, 2, 1,
    71, 37, 34, 30, 28, 20, 17, 26, 21, 16, 10, 6, 8, 6, 2, 0
];
static T15_LENGTHS: [u8; 256] = [
    3, 4, 5, 7, 7, 8, 9, 9, 9, 10, 10, 11, 11, 11, 12, 13, 4, 3, 5,
    6, 7, 7, 8, 8, 8, 9, 9, 10, 10, 10, 11, 11, 5, 5, 5, 6, 7, 7,
    8, 8, 8, 9, 9, 10, 10, 11, 11, 11, 6, 6, 6, 7, 7, 8, 8, 9, 9,
    9, 10, 10, 10, 11, 11, 11, 7, 6, 7, 7, 8, 8, 9, 9, 9, 9, 10, 10,
    10, 11, 11, 11, 8, 7, 7, 8, 8, 8, 9, 9, 9, 9, 10, 10, 11, 11, 11,
    12, 9, 7, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 11, 11, 12, 12, 9, 8,
    8, 9, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11, 11, 12, 9, 8, 8, 9, 9,
    9, 9, 10, 10, 10, 10, 11, 11, 12, 12, 12, 9, 8, 9, 9, 9, 9, 10, 10,
    10, 11, 11, 11, 11, 12, 12, 12, 10, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11,
    11, 11, 12, 13, 12, 10, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 12, 12,
    12, 13, 11, 10, 9, 10, 10, 10, 11, 11, 11, 11, 11, 11, 12, 12, 13, 13, 11,
    10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 12, 11, 11, 11,
    11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 12, 13, 12, 11, 11, 11, 11, 11, 11,
    12, 12, 12, 12, 12, 13, 13, 13, 13
];

static T16_CODES: [u16; 256] = [
    1, 5, 14, 44, 74, 63, 110, 93, 172, 149, 138, 242, 225, 195, 376, 17,
    3, 4, 12, 20, 35, 62, 53, 47, 83, 75, 68, 119, 201, 107, 207, 9,
    15, 13, 23, 38, 67, 58, 103, 90, 161, 72, 127, 117, 110, 209, 206, 16,
    45, 21, 39, 69, 64, 114, 99, 87, 158, 140, 252, 212, 199, 387, 365, 26,
    75, 36, 68, 65, 115, 101, 179, 164, 155, 264, 246, 226, 395, 382, 362, 9,
    66, 30, 59, 56, 102, 185, 173, 265, 142, 253, 232, 400, 388, 378, 445, 16,
    111, 54, 52, 100, 184, 178, 160, 133, 257, 244, 228, 217, 385, 366, 715, 10,
    98, 48, 91, 88, 165, 157, 148, 261, 248, 407, 397, 372, 380, 889, 884, 8,
    85, 84, 81, 159, 156, 143, 260, 249, 427, 401, 392, 383, 727, 713, 708, 7,
    154, 76, 73, 141, 131, 256, 245, 426, 406, 394, 384, 735, 359, 710, 352, 11,
    139, 129, 67, 125, 247, 233, 229, 219, 393, 743, 737, 720, 885, 882, 439, 4,
    243, 120, 118, 115, 227, 223, 396, 746, 742, 736, 721, 712, 706, 223, 436, 6,
    202, 224, 222, 218, 216, 389, 386, 381, 364, 888, 443, 707, 440, 437, 1728, 4,
    747, 211, 210, 208, 370, 379, 734, 723, 714, 1735, 883, 877, 876, 3459, 865, 2,
    377, 369, 102, 187, 726, 722, 358, 711, 709, 866, 1734, 871, 3458, 870, 434, 0,
    12, 10, 7, 11, 10, 17, 11, 9, 13, 12, 10, 7, 5, 3, 1, 3
];
static T16_LENGTHS: [u8; 256] = [
    1, 4, 6, 8, 9, 9, 10, 10, 11, 11, 11, 12, 12, 12, 13, 9, 3, 4, 6,
    7, 8, 9, 9, 9, 10, 10, 10, 11, 12, 11, 12, 8, 6, 6, 7, 8, 9, 9,
    10, 10, 11, 10, 11, 11, 11, 12, 12, 9, 8, 7, 8, 9, 9, 10, 10, 10, 11,
    11, 12, 12, 12, 13, 13, 10, 9, 8, 9, 9, 10, 10, 11, 11, 11, 12, 12, 12,
    13, 13, 13, 9, 9, 8, 9, 9, 10, 11, 11, 12, 11, 12, 12, 13, 13, 13, 14,
    10, 10, 9, 9, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 14, 10, 10, 9,
    10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 13, 15, 15, 10, 10, 10, 10, 11, 11,
    11, 12, 12, 13, 13, 13, 13, 14, 14, 14, 10, 11, 10, 10, 11, 11, 12, 12, 13,
    13, 13, 13, 14, 13, 14, 13, 11, 11, 11, 10, 11, 12, 12, 12, 12, 13, 14, 14,
    14, 15, 15, 14, 10, 12, 11, 11, 11, 12, 12, 13, 14, 14, 14, 14, 14, 14, 13,
    14, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 15, 14, 14, 14, 14, 16, 11, 14,
    12, 12, 12, 13, 13, 14, 14, 14, 16, 15, 15, 15, 17, 15, 11, 13, 13, 11, 12,
    14, 14, 13, 14, 14, 15, 16, 15, 17, 15, 14, 11, 9, 8, 8, 9, 9, 10, 10,
    10, 11, 11, 11, 11, 11, 11, 11, 8
];

static T24_CODES: [u16; 256] = [
    15, 13, 46, 80, 146, 262, 248, 434, 426, 669, 653, 649, 621, 517, 1032, 88,
    14, 12, 21, 38, 71, 130, 122, 216, 209, 198, 327, 345, 319, 297, 279, 42,
    47, 22, 41, 74, 68, 128, 120, 221, 207, 194, 182, 340, 315, 295, 541, 18,
    81, 39, 75, 70, 134, 125, 116, 220, 204, 190, 178, 325, 311, 293, 271, 16,
    147, 72, 69, 135, 127, 118, 112, 210, 200, 188, 352, 323, 306, 285, 540, 14,
    263, 66, 129, 126, 119, 114, 214, 202, 192, 180, 341, 317, 301, 281, 262, 12,
    249, 123, 121, 117, 113, 215, 206, 195, 185, 347, 330, 308, 291, 272, 520, 10,
    435, 115, 111, 109, 211, 203, 196, 187, 353, 332, 313, 298, 283, 531, 381, 17,
    427, 212, 208, 205, 201, 193, 186, 177, 169, 320, 303, 286, 268, 514, 377, 16,
    335, 199, 197, 191, 189, 181, 174, 333, 321, 305, 289, 275, 521, 379, 371, 11,
    668, 184, 183, 179, 175, 344, 331, 314, 304, 290, 277, 530, 383, 373, 366, 10,
    652, 346, 171, 168, 164, 318, 309, 299, 287, 276, 263, 513, 375, 368, 362, 6,
    648, 322, 316, 312, 307, 302, 292, 284, 269, 261, 512, 376, 370, 364, 359, 4,
    620, 300, 296, 294, 288, 282, 273, 266, 515, 380, 374, 369, 365, 361, 357, 2,
    1033, 280, 278, 274, 267, 264, 259, 382, 378, 372, 367, 363, 360, 358, 356, 0,
    43, 20, 19, 17, 15, 13, 11, 9, 7, 6, 4, 7, 5, 3, 1, 3
];
static T24_LENGTHS: [u8; 256] = [
    4, 4, 6, 7, 8, 9, 9, 10, 10, 11, 11, 11, 11, 11, 12, 9, 4, 4, 5,
    6, 7, 8, 8, 9, 9, 9, 10, 10, 10, 10, 10, 8, 6, 5, 6, 7, 7, 8,
    8, 9, 9, 9, 9, 10, 10, 10, 11, 7, 7, 6, 7, 7, 8, 8, 8, 9, 9,
    9, 9, 10, 10, 10, 10, 7, 8, 7, 7, 8, 8, 8, 8, 9, 9, 9, 10, 10,
    10, 10, 11, 7, 9, 7, 8, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 10,
    7, 9, 8, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 10, 11, 7, 10, 8,
    8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11, 8, 10, 9, 9, 9, 9,
    9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 8, 10, 9, 9, 9, 9, 9, 9, 10,
    10, 10, 10, 10, 11, 11, 11, 8, 11, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10,
    11, 11, 11, 11, 8, 11, 10, 9, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 11,
    11, 8, 11, 10, 10, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 8, 11,
    10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 8, 12, 10, 10, 10,
    10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11, 8, 8, 7, 7, 7, 7, 7, 7,
    7, 7, 7, 7, 8, 8, 8, 8, 4
];

// Count1 tables (Table A and Table B)
static T32_CODES: [u16; 16] = [1, 5, 4, 5, 6, 5, 4, 4, 7, 3, 6, 0, 7, 2, 3, 1];
static T32_LENGTHS: [u8; 16] = [1, 4, 4, 5, 4, 6, 5, 6, 4, 5, 5, 6, 5, 6, 6, 6];

static T33_CODES: [u16; 16] = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
static T33_LENGTHS: [u8; 16] = [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];

/// All Huffman tables (0-33) - matches shine's huffman_table array
/// Tables 0, 4, and 14 are not used in MP3 encoding
pub const HUFFMAN_TABLES: [Option<HuffmanTable>; 34] = [
    None, // Table 0 - not used
    Some(HuffmanTable { xlen: 2, ylen: 2, linbits: 0, linmax: 0, codes: &T1_CODES, lengths: &T1_LENGTHS }),
    Some(HuffmanTable { xlen: 3, ylen: 3, linbits: 0, linmax: 0, codes: &T2_CODES, lengths: &T2_LENGTHS }),
    Some(HuffmanTable { xlen: 3, ylen: 3, linbits: 0, linmax: 0, codes: &T3_CODES, lengths: &T3_LENGTHS }),
    None, // Table 4 - not used
    Some(HuffmanTable { xlen: 4, ylen: 4, linbits: 0, linmax: 0, codes: &T5_CODES, lengths: &T5_LENGTHS }),
    Some(HuffmanTable { xlen: 4, ylen: 4, linbits: 0, linmax: 0, codes: &T6_CODES, lengths: &T6_LENGTHS }),
    Some(HuffmanTable { xlen: 6, ylen: 6, linbits: 0, linmax: 0, codes: &T7_CODES, lengths: &T7_LENGTHS }),
    Some(HuffmanTable { xlen: 6, ylen: 6, linbits: 0, linmax: 0, codes: &T8_CODES, lengths: &T8_LENGTHS }),
    Some(HuffmanTable { xlen: 6, ylen: 6, linbits: 0, linmax: 0, codes: &T9_CODES, lengths: &T9_LENGTHS }),
    Some(HuffmanTable { xlen: 8, ylen: 8, linbits: 0, linmax: 0, codes: &T10_CODES, lengths: &T10_LENGTHS }),
    Some(HuffmanTable { xlen: 8, ylen: 8, linbits: 0, linmax: 0, codes: &T11_CODES, lengths: &T11_LENGTHS }),
    Some(HuffmanTable { xlen: 8, ylen: 8, linbits: 0, linmax: 0, codes: &T12_CODES, lengths: &T12_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 0, linmax: 0, codes: &T13_CODES, lengths: &T13_LENGTHS }),
    None, // Table 14 - not used
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 0, linmax: 0, codes: &T15_CODES, lengths: &T15_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 1, linmax: 1, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 2, linmax: 3, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 3, linmax: 7, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 4, linmax: 15, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 6, linmax: 63, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 8, linmax: 255, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 10, linmax: 1023, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 13, linmax: 8191, codes: &T16_CODES, lengths: &T16_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 4, linmax: 15, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 5, linmax: 31, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 6, linmax: 63, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 7, linmax: 127, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 8, linmax: 255, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 9, linmax: 511, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 11, linmax: 2047, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 16, ylen: 16, linbits: 13, linmax: 8191, codes: &T24_CODES, lengths: &T24_LENGTHS }),
    Some(HuffmanTable { xlen: 1, ylen: 16, linbits: 0, linmax: 0, codes: &T32_CODES, lengths: &T32_LENGTHS }),
    Some(HuffmanTable { xlen: 1, ylen: 16, linbits: 0, linmax: 0, codes: &T33_CODES, lengths: &T33_LENGTHS }),
];

/// Count1 tables (Table A and Table B)
pub const COUNT1_TABLES: [&HuffmanTable; 2] = [
    &HuffmanTable { xlen: 1, ylen: 16, linbits: 0, linmax: 0, codes: &T32_CODES, lengths: &T32_LENGTHS }, // Table A
    &HuffmanTable { xlen: 1, ylen: 16, linbits: 0, linmax: 0, codes: &T33_CODES, lengths: &T33_LENGTHS }, // Table B
];