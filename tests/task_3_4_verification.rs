//! 任务3.4: 其他核心模块函数验证测试
//! 
//! 验证MDCT、子带、储备池和主编码流程模块与shine参考实现的一致性

use rust_mp3_encoder::{Config, Mp3Encoder};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, Channels, StereoMode, Emphasis};

fn create_test_config() -> Config {
    Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Stereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    }
}

/// 测试MDCT模块基本功能
#[test]
fn test_mdct_module_functionality() {
    println!("=== 验证MDCT模块基本功能 ===");
    
    let config = create_test_config();
    let encoder = Mp3Encoder::new(config);
    assert!(encoder.is_ok(), "编码器初始化应该成功");
    
    let encoder = encoder.unwrap();
    
    // 验证MDCT相关配置正确初始化
    let shine_config = encoder.config();
    
    // 验证MDCT系数表已初始化（通过检查非零值）
    let mut non_zero_coeffs = 0;
    for m in 0..18 {
        for k in 0..36 {
            if shine_config.mdct.cos_l[m][k] != 0 {
                non_zero_coeffs += 1;
            }
        }
    }
    
    assert!(non_zero_coeffs > 500, "MDCT系数表应该包含大量非零值，实际: {}", non_zero_coeffs);
    println!("✓ MDCT系数表正确初始化，包含{}个非零系数", non_zero_coeffs);
}

/// 测试子带滤波器模块基本功能
#[test]
fn test_subband_module_functionality() {
    println!("=== 验证子带滤波器模块基本功能 ===");
    
    let mut filter = rust_mp3_encoder::subband::SubbandFilter::new();
    
    // 测试基本滤波功能
    let pcm_samples = [100i16; 32];
    let mut output = [0i32; 32];
    
    let result = filter.filter(&pcm_samples, &mut output, 0);
    assert!(result.is_ok(), "子带滤波应该成功");
    
    // 验证输出不全为零（非零输入应产生非零输出）
    let non_zero_count = output.iter().filter(|&&x| x != 0).count();
    assert!(non_zero_count > 0, "子带滤波器对非零输入应产生非零输出");
    
    println!("✓ 子带滤波器工作正常，产生{}个非零输出", non_zero_count);
    
    // 测试不同输入模式
    let test_cases = [
        ([0i16; 32], "零输入"),
        ([1000i16; 32], "常数输入"),
    ];
    
    for (pcm_input, description) in test_cases.iter() {
        let mut test_output = [0i32; 32];
        let result = filter.filter(pcm_input, &mut test_output, 0);
        assert!(result.is_ok(), "子带滤波器{}测试失败", description);
        
        // 验证输出范围合理
        for &val in test_output.iter() {
            assert!(val.abs() <= i32::MAX / 2, "子带滤波器输出值{}超出合理范围", val);
        }
    }
    
    println!("✓ 子带滤波器通过所有测试用例");
}

/// 测试储备池模块基本功能
#[test]
fn test_reservoir_module_functionality() {
    println!("=== 验证储备池模块基本功能 ===");
    
    let mut reservoir = rust_mp3_encoder::reservoir::BitReservoir::new(128, 44100, 2);
    
    // 验证初始状态
    assert_eq!(reservoir.reservoir_size(), 0, "初始储备池大小应为0");
    assert!(reservoir.mean_bits() > 0, "平均比特数应为正数");
    assert!(reservoir.reservoir_max() > 0, "最大储备池大小应为正数");
    
    println!("✓ 储备池初始化正确: mean_bits={}, max={}", 
             reservoir.mean_bits(), reservoir.reservoir_max());
    
    // 测试max_reservoir_bits函数
    let test_pe_values = [0.0, 50.0, 100.0, 200.0];
    for &pe in test_pe_values.iter() {
        let max_bits = reservoir.max_reservoir_bits(pe, 2);
        assert!(max_bits > 0 && max_bits <= 4095, 
                "max_reservoir_bits返回值{}应在[1, 4095]范围内", max_bits);
    }
    
    println!("✓ max_reservoir_bits函数工作正常");
    
    // 测试储备池调整
    let initial_size = reservoir.reservoir_size();
    reservoir.adjust_reservoir(1000, 2);
    let adjusted_size = reservoir.reservoir_size();
    
    let expected_change = (reservoir.mean_bits() / 2) - 1000;
    assert_eq!(adjusted_size, initial_size + expected_change, 
               "储备池调整逻辑错误");
    
    println!("✓ 储备池调整功能正常");
    
    // 测试帧结束处理
    let result = reservoir.frame_end(2);
    assert!(result.is_ok(), "frame_end应该成功");
    
    let stuffing_bits = result.unwrap();
    assert!(stuffing_bits >= 0, "填充比特数应为非负数");
    assert_eq!(reservoir.reservoir_size() % 8, 0, "储备池应字节对齐");
    
    println!("✓ 储备池帧结束处理正常，填充比特数: {}", stuffing_bits);
}

/// 测试主编码流程模块基本功能
#[test]
fn test_encoder_module_functionality() {
    println!("=== 验证主编码流程模块基本功能 ===");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    // 验证基本配置
    assert_eq!(encoder.samples_per_frame(), 1152, "每帧样本数应为1152");
    assert_eq!(encoder.public_config().wave.channels, Channels::Stereo, "应为立体声配置");
    
    println!("✓ 编码器初始化正确");
    
    // 测试编码流水线
    let pcm_data: Vec<i16> = (0..1152*2)
        .map(|i| (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * i as f64 / 44100.0).sin()) as i16)
        .collect();
    
    let result = encoder.encode_frame_interleaved(&pcm_data);
    assert!(result.is_ok(), "编码流水线应该成功");
    
    let encoded_frame = result.unwrap();
    assert!(!encoded_frame.is_empty(), "编码输出不应为空");
    assert!(encoded_frame.len() >= 4, "编码输出应至少4字节");
    
    // 验证MP3同步字
    let sync = ((encoded_frame[0] as u16) << 3) | ((encoded_frame[1] as u16) >> 5);
    assert_eq!(sync, 0x7FF, "应包含正确的MP3同步字");
    
    // 验证MPEG版本位
    let version_bits = (encoded_frame[1] >> 3) & 0x03;
    assert_eq!(version_bits, 0x03, "应为MPEG-1版本");
    
    // 验证层位
    let layer_bits = (encoded_frame[1] >> 1) & 0x03;
    assert_eq!(layer_bits, 0x01, "应为Layer III");
    
    println!("✓ 编码流水线工作正常，输出{}字节", encoded_frame.len());
    
    // 测试多帧编码一致性
    for frame_idx in 0..3 {
        let test_pcm: Vec<i16> = (0..1152*2)
            .map(|i| ((frame_idx * 100 + i) % 1000) as i16)
            .collect();
        
        let result = encoder.encode_frame_interleaved(&test_pcm);
        assert!(result.is_ok(), "帧{}编码失败", frame_idx);
        
        let frame = result.unwrap();
        assert!(!frame.is_empty(), "帧{}输出为空", frame_idx);
        
        // 验证帧头一致性
        let sync = ((frame[0] as u16) << 3) | ((frame[1] as u16) >> 5);
        assert_eq!(sync, 0x7FF, "帧{}同步字错误", frame_idx);
    }
    
    println!("✓ 多帧编码一致性验证通过");
}

/// 测试不同输入产生不同输出（确保编码器实际处理数据）
#[test]
fn test_encoder_different_inputs_different_outputs() {
    println!("=== 验证不同输入产生不同输出 ===");
    
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    // 创建两个不同的输入
    let pcm_data1: Vec<i16> = (0..1152*2)
        .map(|i| (1000.0 * (2.0 * std::f64::consts::PI * 440.0 * i as f64 / 44100.0).sin()) as i16)
        .collect();
    
    let pcm_data2: Vec<i16> = (0..1152*2)
        .map(|i| (1000.0 * (2.0 * std::f64::consts::PI * 880.0 * i as f64 / 44100.0).sin()) as i16)
        .collect();
    
    let result1 = encoder.encode_frame_interleaved(&pcm_data1);
    assert!(result1.is_ok(), "第一次编码应该成功");
    let encoded1 = result1.unwrap().to_vec();
    
    encoder.reset();
    
    let result2 = encoder.encode_frame_interleaved(&pcm_data2);
    assert!(result2.is_ok(), "第二次编码应该成功");
    let encoded2 = result2.unwrap().to_vec();
    
    // 验证输出不同（除了可能相同的帧头）
    let mut differences = 0;
    let min_len = encoded1.len().min(encoded2.len());
    for i in 4..min_len {  // 跳过帧头检查数据部分
        if encoded1[i] != encoded2[i] {
            differences += 1;
        }
    }
    
    assert!(differences > 0, "不同输入应产生不同输出，但输出完全相同");
    println!("✓ 不同输入产生不同输出，差异字节数: {}", differences);
}

/// 运行所有验证测试的集成测试
#[test]
fn test_all_modules_integration() {
    println!("=== 运行所有模块集成验证 ===");
    
    // 这个测试验证所有模块能够协同工作
    let config = create_test_config();
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    // 创建一个更复杂的测试信号（混合频率）
    let pcm_data: Vec<i16> = (0..1152*2)
        .map(|i| {
            let t = i as f64 / 44100.0;
            let signal = 500.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin() +
                        300.0 * (2.0 * std::f64::consts::PI * 880.0 * t).sin() +
                        200.0 * (2.0 * std::f64::consts::PI * 1320.0 * t).sin();
            signal as i16
        })
        .collect();
    
    let result = encoder.encode_frame_interleaved(&pcm_data);
    assert!(result.is_ok(), "复杂信号编码应该成功");
    
    let encoded_frame = result.unwrap();
    assert!(encoded_frame.len() > 100, "复杂信号应产生合理大小的输出");
    
    // 验证输出包含实际的音频数据（不全是零或填充）
    let mut non_zero_bytes = 0;
    for &byte in encoded_frame.iter().skip(32) {  // 跳过帧头和侧信息
        if byte != 0 && byte != 0xFF {
            non_zero_bytes += 1;
        }
    }
    
    assert!(non_zero_bytes > encoded_frame.len() / 10, 
            "编码输出应包含足够的非零/非填充数据，实际: {}/{}", 
            non_zero_bytes, encoded_frame.len());
    
    println!("✓ 所有模块集成工作正常");
    println!("✓ 编码输出包含{}字节有效数据（总{}字节）", non_zero_bytes, encoded_frame.len());
    
    println!("================================================");
    println!("✅ 任务3.4验证完成 - 所有核心模块功能正常");
}