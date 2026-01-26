//! 简单的MP3编码示例
//!
//! 这个示例展示了如何使用高级接口进行MP3编码

use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode, encode_pcm_to_mp3};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 示例1: 使用高级接口进行流式编码
    println!("示例1: 流式编码");
    stream_encoding_example()?;

    // 示例2: 一次性编码整个文件
    println!("示例2: 一次性编码");
    batch_encoding_example()?;

    // 示例3: 使用不同的配置
    println!("示例3: 自定义配置");
    custom_config_example()?;

    println!("所有示例完成！");
    Ok(())
}

/// 流式编码示例 - 适合处理大文件或实时音频
fn stream_encoding_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建编码器配置
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .bitrate(128)
        .channels(2)
        .stereo_mode(StereoMode::Stereo);

    // 创建编码器
    let mut encoder = Mp3Encoder::new(config)?;

    // 创建输出文件
    let mut output_file = File::create("output_stream.mp3")?;

    // 模拟音频数据流（这里生成简单的正弦波）
    let sample_rate = 44100;
    let duration_seconds = 2;
    let total_samples = sample_rate * duration_seconds * 2; // 立体声

    let mut samples_processed = 0;
    let chunk_size = encoder.samples_per_frame() * 4; // 处理4帧的数据

    while samples_processed < total_samples {
        // 生成一块音频数据
        let mut chunk = Vec::new();
        for i in 0..chunk_size.min(total_samples - samples_processed) {
            let sample_index = samples_processed + i;
            let time = sample_index as f32 / (sample_rate * 2) as f32;
            
            // 生成440Hz的正弦波
            let sample = (time * 440.0 * 2.0 * std::f32::consts::PI).sin();
            let sample_i16 = (sample * 32767.0) as i16;
            
            chunk.push(sample_i16);
        }

        // 编码这块数据
        let mp3_frames = encoder.encode_interleaved(&chunk)?;
        
        // 写入输出文件
        for frame in mp3_frames {
            output_file.write_all(&frame)?;
        }

        samples_processed += chunk.len();
        println!("已处理 {} / {} 样本", samples_processed, total_samples);
    }

    // 完成编码
    let final_data = encoder.finish()?;
    if !final_data.is_empty() {
        output_file.write_all(&final_data)?;
    }

    println!("流式编码完成，输出文件: output_stream.mp3");
    Ok(())
}

/// 一次性编码示例 - 适合小文件
fn batch_encoding_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = Mp3EncoderConfig::new()
        .sample_rate(22050)  // 较低的采样率
        .bitrate(96)         // 较低的比特率
        .channels(1)         // 单声道
        .stereo_mode(StereoMode::Mono);

    // 生成测试音频数据（1秒的440Hz正弦波）
    let sample_rate = 22050;
    let duration_seconds = 1;
    let total_samples = sample_rate * duration_seconds;

    let mut pcm_data = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        let time = i as f32 / sample_rate as f32;
        let sample = (time * 440.0 * 2.0 * std::f32::consts::PI).sin();
        let sample_i16 = (sample * 32767.0) as i16;
        pcm_data.push(sample_i16);
    }

    // 一次性编码
    let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;

    // 写入文件
    let mut output_file = File::create("output_batch.mp3")?;
    output_file.write_all(&mp3_data)?;

    println!("一次性编码完成，输出文件: output_batch.mp3");
    println!("输入: {} 样本, 输出: {} 字节", pcm_data.len(), mp3_data.len());
    Ok(())
}

/// 自定义配置示例
fn custom_config_example() -> Result<(), Box<dyn std::error::Error>> {
    // 高质量立体声配置
    let high_quality_config = Mp3EncoderConfig::new()
        .sample_rate(48000)
        .bitrate(320)
        .channels(2)
        .stereo_mode(StereoMode::JointStereo)
        .copyright(true)
        .original(true);

    println!("高质量配置: {:?}", high_quality_config);

    // 低质量单声道配置（适合语音）
    let voice_config = Mp3EncoderConfig::new()
        .sample_rate(16000)
        .bitrate(32)
        .channels(1)
        .stereo_mode(StereoMode::Mono);

    println!("语音配置: {:?}", voice_config);

    // 验证配置
    high_quality_config.validate()?;
    voice_config.validate()?;

    println!("所有配置验证通过");

    // 创建编码器并显示信息
    let encoder = Mp3Encoder::new(high_quality_config)?;
    println!("每帧样本数: {}", encoder.samples_per_frame());
    println!("编码器配置: {:?}", encoder.config());

    Ok(())
}