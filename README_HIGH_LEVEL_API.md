# 高级MP3编码器接口

这个项目为Rust MP3编码器提供了一个简单易用的高级接口，同时保留了对底层shine实现的完全访问。

## 特性

- **简单易用**: Rust风格的API，支持链式配置
- **类型安全**: 编译时配置验证和错误处理
- **流式编码**: 支持大文件的分块处理
- **多种输入格式**: 支持交错和分离声道格式
- **完整的错误处理**: 结构化的错误类型
- **底层访问**: 保留对shine低级接口的完全访问

## 快速开始

### 基本用法

```rust
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

// 创建配置
let config = Mp3EncoderConfig::new()
    .sample_rate(44100)
    .bitrate(128)
    .channels(2)
    .stereo_mode(StereoMode::Stereo);

// 创建编码器
let mut encoder = Mp3Encoder::new(config)?;

// 编码PCM数据
let pcm_data: Vec<i16> = vec![/* 你的PCM数据 */];
let mp3_frames = encoder.encode_interleaved(&pcm_data)?;

// 完成编码
let final_data = encoder.finish()?;
```

### 一次性编码

```rust
use shine_rs::{encode_pcm_to_mp3, Mp3EncoderConfig};

let config = Mp3EncoderConfig::new();
let pcm_data: Vec<i16> = vec![/* 你的PCM数据 */];
let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;
```

## 支持的配置

### 采样率 (Hz)
- **MPEG-1**: 32000, 44100, 48000
- **MPEG-2**: 16000, 22050, 24000  
- **MPEG-2.5**: 8000, 11025, 12000

### 比特率 (kbps)
- **MPEG-1**: 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320
- **MPEG-2**: 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160
- **MPEG-2.5**: 8, 16, 24, 32, 40, 48, 56, 64

### 立体声模式
- `StereoMode::Stereo` - 标准立体声
- `StereoMode::JointStereo` - 联合立体声
- `StereoMode::DualChannel` - 双声道
- `StereoMode::Mono` - 单声道

## 使用示例

### 流式编码（推荐用于大文件）

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

### 分离声道编码

```rust
let left_channel: Vec<i16> = vec![/* 左声道数据 */];
let right_channel: Vec<i16> = vec![/* 右声道数据 */];

let mp3_frames = encoder.encode_separate_channels(
    &left_channel, 
    Some(&right_channel)
)?;
```

### 单声道编码

```rust
let config = Mp3EncoderConfig::new()
    .channels(1)
    .stereo_mode(StereoMode::Mono);

let mut encoder = Mp3Encoder::new(config)?;
let mono_data: Vec<i16> = vec![/* 单声道数据 */];
let mp3_frames = encoder.encode_separate_channels(&mono_data, None)?;
```

## 错误处理

高级接口提供了详细的错误信息，帮助用户快速定位问题：

```rust
use shine_rs::{EncoderError, ConfigError, InputDataError};

match encoder.encode_interleaved(&pcm_data) {
    Ok(frames) => { /* 处理成功 */ },
    Err(EncoderError::Config(ConfigError::UnsupportedSampleRate(rate))) => {
        eprintln!("不支持的采样率: {} Hz", rate);
    },
    Err(EncoderError::Config(ConfigError::UnsupportedBitrate(bitrate))) => {
        eprintln!("不支持的比特率: {} kbps", bitrate);
    },
    Err(EncoderError::Config(ConfigError::IncompatibleRateCombination { 
        sample_rate, bitrate, reason 
    })) => {
        eprintln!("不兼容的采样率和比特率组合: {} Hz @ {} kbps - {}", 
                 sample_rate, bitrate, reason);
    },
    Err(EncoderError::InputData(InputDataError::EmptyInput)) => {
        eprintln!("输入数据为空");
    },
    Err(e) => {
        eprintln!("编码错误: {}", e);
    }
}
```

### 配置验证错误

接口会自动验证配置参数的有效性，并提供具体的错误信息：

- **不支持的采样率**: 当采样率不在支持列表中时
- **不支持的比特率**: 当比特率不在支持列表中时  
- **不兼容的组合**: 当采样率和比特率组合不被MPEG标准支持时，会提供详细说明：
  - MPEG-2.5 (8-12 kHz): 仅支持 8-64 kbps
  - MPEG-2 (16-24 kHz): 仅支持 8-160 kbps
  - MPEG-1 (32-48 kHz): 仅支持 32-320 kbps
- **无效声道配置**: 当声道数与立体声模式不匹配时

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

## 性能建议

1. **流式处理**: 对于大文件，使用流式编码而不是一次性加载所有数据
2. **缓冲区大小**: 使用 `encoder.samples_per_frame()` 的倍数作为输入块大小
3. **内存管理**: 及时处理编码输出，避免累积大量MP3数据在内存中
4. **配置验证**: 在创建编码器前调用 `config.validate()` 进行早期错误检测

## 与底层接口的关系

高级接口是底层shine接口的封装，提供了：

- **自动缓冲区管理**: 处理不完整的帧数据
- **类型安全的配置**: 编译时验证配置参数
- **结构化错误处理**: 清晰的错误类型和消息
- **内存安全保证**: 避免手动内存管理
- **Rust风格的API**: 符合Rust惯例的接口设计

同时保留了对底层接口的完全访问，让高级用户可以在需要时进行精细控制。

## 测试

项目包含全面的测试套件：

```bash
# 运行高级接口测试
cargo test --test mp3_encoder_tests

# 运行所有测试
cargo test
```

测试覆盖了：
- 配置验证
- 编码功能
- 错误处理
- 属性测试（使用proptest）
- 集成测试

## 示例代码

完整的示例代码请参考：
- `examples/simple_encoding.rs` - 基本使用示例
- `tests/mp3_encoder_tests.rs` - 完整的测试用例
- `docs/HIGH_LEVEL_API.md` - 详细的API文档