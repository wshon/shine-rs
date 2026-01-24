use rust_mp3_encoder::encoder::{ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_set_config_mpeg_defaults};
use rust_mp3_encoder::bitstream::BitstreamWriter;

fn main() {
    println!("🔍 调试帧头编码过程");
    
    // 创建配置
    let mut mpeg_config = ShineMpeg {
        mode: 0, // STEREO (not JOINT_STEREO)
        bitr: 128,
        emph: 0,
        copyright: 0,
        original: 1,
    };
    shine_set_config_mpeg_defaults(&mut mpeg_config);
    
    let config = ShineConfig {
        wave: ShineWave {
            channels: 2,
            samplerate: 44100,
        },
        mpeg: mpeg_config,
    };
    
    let global_config = shine_initialise(&config).expect("Failed to initialize");
    
    println!("📋 配置信息:");
    println!("  MPEG version: {} (应该是3=MPEG-I)", global_config.mpeg.version);
    println!("  MPEG layer: {} (应该是1=Layer III)", global_config.mpeg.layer);
    println!("  CRC: {} (应该是0)", global_config.mpeg.crc);
    println!("  Bitrate index: {} (应该是9对应128kbps)", global_config.mpeg.bitrate_index);
    println!("  Samplerate index: {} (应该是0对应44100Hz)", global_config.mpeg.samplerate_index);
    println!("  Padding: {} (可能是0或1)", global_config.mpeg.padding);
    println!("  Extension: {} (应该是0)", global_config.mpeg.ext);
    println!("  Mode: {} (应该是1=Joint stereo)", global_config.mpeg.mode);
    println!("  Mode ext: {} (应该是0)", global_config.mpeg.mode_ext);
    println!("  Copyright: {} (应该是0)", global_config.mpeg.copyright);
    println!("  Original: {} (应该是1)", global_config.mpeg.original);
    println!("  Emphasis: {} (应该是0)", global_config.mpeg.emph);
    
    println!("\n🔧 手动构建帧头:");
    let mut bs = BitstreamWriter::new(1024);
    
    // 按照shine的顺序写入帧头
    println!("写入 sync word (0x7ff, 11 bits)");
    bs.put_bits(0x7ff, 11).unwrap();
    
    println!("写入 version ({}, 2 bits)", global_config.mpeg.version);
    bs.put_bits(global_config.mpeg.version as u32, 2).unwrap();
    
    println!("写入 layer ({}, 2 bits)", global_config.mpeg.layer);
    bs.put_bits(global_config.mpeg.layer as u32, 2).unwrap();
    
    println!("写入 CRC protection ({}, 1 bit)", if global_config.mpeg.crc == 0 { 1 } else { 0 });
    bs.put_bits(if global_config.mpeg.crc == 0 { 1 } else { 0 }, 1).unwrap();
    
    println!("写入 bitrate index ({}, 4 bits)", global_config.mpeg.bitrate_index);
    bs.put_bits(global_config.mpeg.bitrate_index as u32, 4).unwrap();
    
    println!("写入 samplerate index ({}, 2 bits)", global_config.mpeg.samplerate_index % 3);
    bs.put_bits((global_config.mpeg.samplerate_index % 3) as u32, 2).unwrap();
    
    println!("写入 padding (1, 1 bit)"); // Test with padding=1
    bs.put_bits(1, 1).unwrap();
    
    println!("写入 extension ({}, 1 bit)", global_config.mpeg.ext);
    bs.put_bits(global_config.mpeg.ext as u32, 1).unwrap();
    
    println!("写入 mode ({}, 2 bits)", global_config.mpeg.mode);
    bs.put_bits(global_config.mpeg.mode as u32, 2).unwrap();
    
    println!("写入 mode extension ({}, 2 bits)", global_config.mpeg.mode_ext);
    bs.put_bits(global_config.mpeg.mode_ext as u32, 2).unwrap();
    
    println!("写入 copyright ({}, 1 bit)", global_config.mpeg.copyright);
    bs.put_bits(global_config.mpeg.copyright as u32, 1).unwrap();
    
    println!("写入 original ({}, 1 bit)", global_config.mpeg.original);
    bs.put_bits(global_config.mpeg.original as u32, 1).unwrap();
    
    println!("写入 emphasis ({}, 2 bits)", global_config.mpeg.emph);
    bs.put_bits(global_config.mpeg.emph as u32, 2).unwrap();
    
    // 刷新缓存
    bs.flush().unwrap();
    
    let data = bs.get_data();
    println!("\n📊 生成的帧头 (前4字节):");
    if data.len() >= 4 {
        println!("  0x{:02X} 0x{:02X} 0x{:02X} 0x{:02X}", data[0], data[1], data[2], data[3]);
        println!("  应该是: FF FB 92 04 (shine的输出)");
        println!("  我们的: {:02X} {:02X} {:02X} {:02X}", data[0], data[1], data[2], data[3]);
        
        // 分析每个字节
        println!("\n🔍 字节分析:");
        println!("  第1字节 0x{:02X}: sync word高8位 (应该是0xFF)", data[0]);
        println!("  第2字节 0x{:02X}: sync word低3位 + version + layer + CRC (应该是0xFB)", data[1]);
        println!("  第3字节 0x{:02X}: bitrate + samplerate + padding + ext (应该是0x92)", data[2]);
        println!("  第4字节 0x{:02X}: mode + mode_ext + copyright + original + emph (应该是0x04)", data[3]);
        
        // 详细分析第4字节
        let byte4 = data[3];
        let mode = (byte4 >> 6) & 0x03;
        let mode_ext = (byte4 >> 4) & 0x03;
        let copyright = (byte4 >> 3) & 0x01;
        let original = (byte4 >> 2) & 0x01;
        let emph = byte4 & 0x03;
        
        println!("\n  第4字节详细分析:");
        println!("    Mode: {} (期望: 1)", mode);
        println!("    Mode ext: {} (期望: 0)", mode_ext);
        println!("    Copyright: {} (期望: 0)", copyright);
        println!("    Original: {} (期望: 1)", original);
        println!("    Emphasis: {} (期望: 0)", emph);
        
        if byte4 != 0x04 {
            println!("  ❌ 第4字节不匹配! 0x{:02X} != 0x04", byte4);
        } else {
            println!("  ✅ 第4字节匹配!");
        }
    }
}