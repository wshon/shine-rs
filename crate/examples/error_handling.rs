//! 错误处理示例
//!
//! 这个示例展示了如何处理MP3编码器的各种错误情况

use shine_rs::mp3_encoder::{Mp3Encoder, Mp3EncoderConfig, StereoMode};
use shine_rs::error::{EncoderError, ConfigError, InputDataError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MP3编码器错误处理示例");

    // 示例1: 不支持的采样率
    println!("\n1. 测试不支持的采样率");
    test_unsupported_sample_rate();

    // 示例2: 不支持的比特率
    println!("\n2. 测试不支持的比特率");
    test_unsupported_bitrate();

    // 示例3: 不兼容的采样率和比特率组合
    println!("\n3. 测试不兼容的组合");
    test_incompatible_combinations();

    // 示例4: 声道配置错误
    println!("\n4. 测试声道配置错误");
    test_channel_configuration_errors();

    // 示例5: 输入数据错误
    println!("\n5. 测试输入数据错误");
    test_input_data_errors();

    println!("\n所有错误处理示例完成！");
    Ok(())
}

fn test_unsupported_sample_rate() {
    let config = Mp3EncoderConfig::new().sample_rate(12345);
    
    match Mp3Encoder::new(config) {
        Err(EncoderError::Config(ConfigError::UnsupportedSampleRate(rate))) => {
            println!("✓ 正确捕获不支持的采样率错误: {} Hz", rate);
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }
}

fn test_unsupported_bitrate() {
    let config = Mp3EncoderConfig::new().bitrate(999);
    
    match Mp3Encoder::new(config) {
        Err(EncoderError::Config(ConfigError::UnsupportedBitrate(bitrate))) => {
            println!("✓ 正确捕获不支持的比特率错误: {} kbps", bitrate);
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }
}
fn test_incompatible_combinations() {
    // MPEG-2.5 with high bitrate
    let config = Mp3EncoderConfig::new()
        .sample_rate(8000)   // MPEG-2.5
        .bitrate(320);       // Too high for MPEG-2.5
    
    match Mp3Encoder::new(config) {
        Err(EncoderError::Config(ConfigError::IncompatibleRateCombination { 
            sample_rate, bitrate, reason 
        })) => {
            println!("✓ 正确捕获不兼容组合错误:");
            println!("  采样率: {} Hz", sample_rate);
            println!("  比特率: {} kbps", bitrate);
            println!("  原因: {}", reason);
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }

    // MPEG-1 with low bitrate
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)  // MPEG-1
        .bitrate(16);        // Too low for MPEG-1
    
    match Mp3Encoder::new(config) {
        Err(EncoderError::Config(ConfigError::IncompatibleRateCombination { 
            sample_rate, bitrate, reason 
        })) => {
            println!("✓ 正确捕获另一个不兼容组合错误:");
            println!("  采样率: {} Hz", sample_rate);
            println!("  比特率: {} kbps", bitrate);
            println!("  原因: {}", reason);
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }
}

fn test_channel_configuration_errors() {
    // Mono mode with 2 channels
    let config = Mp3EncoderConfig::new()
        .channels(2)
        .stereo_mode(StereoMode::Mono);
    
    match Mp3Encoder::new(config) {
        Err(EncoderError::Config(ConfigError::InvalidStereoMode { mode, channels })) => {
            println!("✓ 正确捕获声道配置错误:");
            println!("  声道数: {}", channels);
            println!("  立体声模式: {}", mode);
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }
}

fn test_input_data_errors() {
    let config = Mp3EncoderConfig::new();
    let mut encoder = Mp3Encoder::new(config).unwrap();
    
    // Empty input
    let empty_data: Vec<i16> = Vec::new();
    match encoder.encode_interleaved(&empty_data) {
        Err(EncoderError::InputData(InputDataError::EmptyInput)) => {
            println!("✓ 正确捕获空输入错误");
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }

    // Channel count mismatch
    let left_channel = vec![100i16; 1000];
    let right_channel = vec![200i16; 500]; // Different length
    
    match encoder.encode_separate_channels(&left_channel, Some(&right_channel)) {
        Err(EncoderError::InputData(InputDataError::InvalidChannelCount { expected, actual })) => {
            println!("✓ 正确捕获声道数据长度不匹配错误:");
            println!("  期望长度: {}", expected);
            println!("  实际长度: {}", actual);
        },
        other => {
            println!("✗ 意外的结果: {:?}", other);
        }
    }
}