# 高级API使用指南

这个文档介绍如何使用Rust MP3编码器的底层接口。项目提供了直接基于 Shine C 实现的 Rust 接口，确保算法的完全一致性。

## 快速开始

### 基本用法

```rust
use shine_rs::{ShineConfig, WaveConfig, MpegConfig, shine_initialise, shine_encode_buffer_interleaved};

// 创建配置
let config = ShineConfig {
    wave: WaveConfig {
        channels: 2,
        samplerate: 44100,
    },
    mpeg: MpegConfig {
        mode: 0,  // 立体声
        bitr: 128,
        emph: 0,
        copyright: 0,
        original: 1,
    },
};

// 初始化编码器
let mut global_config = shine_initialise(&config)?;

// 编码PCM数据
let pcm_data: Vec<i16> = vec![/* 你的PCM数据 */];
let (mp3_data, written) = shine_encode_buffer_interleaved(&mut global_config, pcm_data.as_ptr())?;
```

### 使用命令行工具

```bash
# 基本转换
cargo run --bin wav2mp3 input.wav output.mp3

# 指定比特率和模式
cargo run --bin wav2mp3 input.wav output.mp3 128 stereo

# 调试模式（限制帧数）
cargo run --bin wav2mp3 input.wav output.mp3 --max-frames 10
```

## 配置选项

### Mp3EncoderConfig

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `sample_rate` | `u32` | 44100 | 采样率 (Hz) |
| `bitrate` | `u32` | 128 | 比特率 (kbps) |
| `channels` | `u8` | 2 | 声道数 (1或2) |
| `stereo_mode` | `StereoMode` | `Stereo` | 立体声模式 |
| `copyright` | `bool` | false | 版权标志 |
| `original` | `bool` | true | 原创标志 |

### 支持的参数

#### 采样率 (Hz)
- 8000, 11025, 12000 (MPEG 2.5)
- 16000, 22050, 24000 (MPEG 2)
- 32000, 44100, 48000 (MPEG 1)

#### 比特率 (kbps)
- 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320

#### 立体声模式
- `StereoMode::Stereo` - 标准立体声
- `StereoMode::JointStereo` - 联合立体声（更高效）
- `StereoMode::DualChannel` - 双声道
- `StereoMode::Mono` - 单声道

## 使用模式

### 1. 流式编码（推荐用于大文件）

```rust
let mut encoder = Mp3Encoder::new(config)?;
let mut output_file = File::create("output.mp3")?;

// 分块处理音频数据
while let Some(chunk) = get_next_audio_chunk() {
    let mp3_frames = encoder.encode_interleaved(&chunk)?;
    for frame in mp3_frames {
        output_file.write_all(&frame)?;
    }
}

// 完成编码
let final_data = encoder.finish()?;
output_file.write_all(&final_data)?;
```

### 2. 分离声道编码

```rust
let left_channel: Vec<i16> = vec![/* 左声道数据 */];
let right_channel: Vec<i16> = vec![/* 右声道数据 */];

let mp3_frames = encoder.encode_separate_channels(
    &left_channel, 
    Some(&right_channel)
)?;
```

### 3. 单声道编码

```rust
let config = Mp3EncoderConfig::new()
    .channels(1)
    .stereo_mode(StereoMode::Mono);

let mut encoder = Mp3Encoder::new(config)?;
let mono_data: Vec<i16> = vec![/* 单声道数据 */];
let mp3_frames = encoder.encode_separate_channels(&mono_data, None)?;
```

## 高级功能

### 访问底层shine配置

```rust
let mut encoder = Mp3Encoder::new(config)?;

// 获取底层shine配置进行高级操作
let shine_config = encoder.shine_config();
// 现在可以直接调用底层shine函数
```

### 监控编码状态

```rust
// 检查缓冲区中剩余的样本数
let buffered = encoder.buffered_samples();

// 检查编码器是否已完成
let finished = encoder.is_finished();

// 获取每帧需要的样本数
let samples_per_frame = encoder.samples_per_frame();
```

## 错误处理

高级接口使用结构化的错误类型：

```rust
use rust_mp3_encoder::{EncoderError, ConfigError, InputDataError};

match encoder.encode_interleaved(&pcm_data) {
    Ok(frames) => { /* 处理成功 */ },
    Err(EncoderError::Config(ConfigError::UnsupportedSampleRate(rate))) => {
        eprintln!("不支持的采样率: {}", rate);
    },
    Err(EncoderError::InputData(InputDataError::EmptyInput)) => {
        eprintln!("输入数据为空");
    },
    Err(e) => {
        eprintln!("编码错误: {}", e);
    }
}
```

## 性能建议

1. **流式处理**: 对于大文件，使用流式编码而不是一次性加载所有数据
2. **缓冲区大小**: 使用 `encoder.samples_per_frame()` 的倍数作为输入块大小
3. **内存管理**: 及时处理编码输出，避免累积大量MP3数据在内存中
4. **配置验证**: 在创建编码器前调用 `config.validate()` 进行早期错误检测

## 与底层接口的关系

高级接口是底层shine接口的封装：

```rust
// 高级接口
let mut encoder = Mp3Encoder::new(config)?;
let frames = encoder.encode_interleaved(&pcm_data)?;

// 等价的底层接口调用
let shine_config = create_shine_config(&config);
let mut global_config = shine_initialise(&shine_config)?;
let (data, written) = shine_encode_buffer_interleaved(&mut global_config, pcm_data.as_ptr())?;
```

高级接口提供了：
- 自动缓冲区管理
- 类型安全的配置
- 结构化错误处理
- 内存安全保证
- Rust风格的API

同时保留了对底层接口的完全访问，让高级用户可以在需要时进行精细控制。

## 示例代码

完整的示例代码请参考 `examples/simple_encoding.rs` 文件。