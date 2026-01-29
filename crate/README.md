# shine-rs 库

这是 shine-rs MP3 编码器的核心库实现。该库提供了完整的 MP3 Layer III 编码功能，严格遵循 Shine C 语言参考实现。

## 库架构

### 核心模块

- **`encoder`** - 主编码器模块，提供底层 Shine 兼容接口
- **`mp3_encoder`** - 高级编码器接口，提供更友好的 Rust API
- **`config`** - 编码配置管理
- **`error`** - 错误类型定义

### 算法模块

- **`subband`** - 32频带子带分析滤波器
- **`mdct`** - 修正离散余弦变换 (Modified Discrete Cosine Transform)
- **`quantization`** - 量化循环和比特率控制
- **`huffman`** - Huffman 编码器
- **`bitstream`** - MP3 比特流写入器
- **`reservoir`** - 比特池管理

### 数据和查找表

- **`tables`** - 所有 MP3 编码所需的查找表
- **`psychoacoustic`** - 心理声学模型（简化版）

## API 使用

### 高级接口 (推荐)

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

// 编码音频数据 (交错格式)
let pcm_samples = vec![0i16; encoder.samples_per_frame()];
let mp3_data = encoder.encode_interleaved(&pcm_samples)?;

// 完成编码
let final_data = encoder.finish()?;
```

### 底层接口 (Shine 兼容)

```rust
use shine_rs::{
    ShineConfig, ShineWave, ShineMpeg,
    shine_initialise, shine_encode_buffer_interleaved, 
    shine_flush, shine_close
};

// 初始化配置
let mut config = ShineConfig::default();
config.wave.samplerate = 44100;
config.wave.channels = 2;
config.mpeg.bitr = 128;

// 初始化编码器
shine_initialise(&mut config);

// 编码数据
let pcm_data = vec![0i16; config.samples_per_pass()];
let mp3_data = shine_encode_buffer_interleaved(&mut config, &pcm_data);

// 完成编码
let final_data = shine_flush(&mut config);
shine_close(&mut config);
```

## 配置选项

### Mp3EncoderConfig

```rust
pub struct Mp3EncoderConfig {
    sample_rate: u32,      // 采样率 (8000-48000 Hz)
    bitrate: u32,          // 比特率 (8-320 kbps)
    channels: u16,         // 声道数 (1-2)
    stereo_mode: StereoMode, // 立体声模式
    copyright: bool,       // 版权标志
    original: bool,        // 原创标志
    emphasis: Emphasis,    // 预加重
}
```

### 立体声模式

```rust
pub enum StereoMode {
    Stereo,      // 立体声
    JointStereo, // 联合立体声
    DualChannel, // 双声道
    Mono,        // 单声道
}
```

### 支持的采样率和比特率组合

| MPEG版本 | 采样率 (Hz) | 比特率范围 (kbps) |
|----------|-------------|-------------------|
| MPEG-1   | 32000, 44100, 48000 | 32-320 |
| MPEG-2   | 16000, 22050, 24000 | 8-160  |
| MPEG-2.5 | 8000, 11025, 12000  | 8-64   |

## 错误处理

```rust
use shine_rs::EncodingError;

match encoder.encode_interleaved(&pcm_data) {
    Ok(mp3_data) => {
        // 处理编码数据
    }
    Err(EncodingError::InvalidSampleRate(rate)) => {
        eprintln!("不支持的采样率: {}", rate);
    }
    Err(EncodingError::InvalidBitrate(bitrate)) => {
        eprintln!("不支持的比特率: {}", bitrate);
    }
    Err(EncodingError::InvalidChannelCount(channels)) => {
        eprintln!("不支持的声道数: {}", channels);
    }
    Err(e) => {
        eprintln!("编码错误: {}", e);
    }
}
```

## 内存管理

### 缓冲区大小

```rust
// 获取每帧所需的 PCM 样本数
let samples_per_frame = encoder.samples_per_frame(); // 通常是 1152

// 获取最大 MP3 帧大小
let max_mp3_frame_size = encoder.max_mp3_frame_size(); // 取决于比特率

// 预分配缓冲区
let mut pcm_buffer = vec![0i16; samples_per_frame];
let mut mp3_buffer = Vec::with_capacity(max_mp3_frame_size);
```

### 批量处理

```rust
// 处理大量音频数据
let chunk_size = encoder.samples_per_frame();
for chunk in pcm_data.chunks(chunk_size) {
    if chunk.len() == chunk_size {
        let mp3_frame = encoder.encode_interleaved(chunk)?;
        output.extend_from_slice(&mp3_frame);
    } else {
        // 处理最后一个不完整的块
        let mut padded_chunk = vec![0i16; chunk_size];
        padded_chunk[..chunk.len()].copy_from_slice(chunk);
        let mp3_frame = encoder.encode_interleaved(&padded_chunk)?;
        output.extend_from_slice(&mp3_frame);
    }
}
```

## 调试功能

### 启用诊断特性

```toml
[dependencies]
shine-rs = { version = "0.1", features = ["diagnostics"] }
```

```rust
#[cfg(feature = "diagnostics")]
{
    // 访问内部诊断数据
    let diagnostics = encoder.get_diagnostics();
    println!("MDCT 系数: {:?}", diagnostics.mdct_coefficients);
    println!("量化参数: {:?}", diagnostics.quantization_params);
}
```

### 日志输出

```rust
use log::{info, debug};

// 启用日志
env_logger::init();

// 编码时会输出详细日志
debug!("开始编码帧 {}", frame_number);
info!("编码完成，输出 {} 字节", mp3_data.len());
```

## 性能优化

### 预分配缓冲区

```rust
// 避免重复分配
let mut encoder = Mp3Encoder::new(config)?;
let mut pcm_buffer = vec![0i16; encoder.samples_per_frame()];
let mut mp3_output = Vec::new();

loop {
    // 重用缓冲区
    if let Some(samples) = read_audio_samples(&mut pcm_buffer) {
        let mp3_frame = encoder.encode_interleaved(&pcm_buffer[..samples])?;
        mp3_output.extend_from_slice(&mp3_frame);
    } else {
        break;
    }
}
```

### 批量处理

```rust
// 处理多个帧以减少函数调用开销
const BATCH_SIZE: usize = 10;
let frame_size = encoder.samples_per_frame();
let batch_size = frame_size * BATCH_SIZE;

for batch in pcm_data.chunks(batch_size) {
    for frame in batch.chunks(frame_size) {
        let mp3_frame = encoder.encode_interleaved(frame)?;
        output.extend_from_slice(&mp3_frame);
    }
}
```

## 线程安全

该库不是线程安全的。如需在多线程环境中使用，请为每个线程创建独立的编码器实例：

```rust
use std::thread;
use std::sync::mpsc;

// 为每个线程创建独立的编码器
let handles: Vec<_> = (0..num_threads).map(|_| {
    let config = config.clone();
    thread::spawn(move || {
        let mut encoder = Mp3Encoder::new(config).unwrap();
        // 处理音频数据...
    })
}).collect();
```

## 与 Shine C 实现的对应关系

| Rust 函数 | Shine C 函数 | 说明 |
|-----------|--------------|------|
| `Mp3Encoder::new()` | `shine_initialise()` | 初始化编码器 |
| `encode_interleaved()` | `shine_encode_buffer_interleaved()` | 编码交错音频数据 |
| `finish()` | `shine_flush()` + `shine_close()` | 完成编码并清理 |
| `samples_per_frame()` | `shine_samples_per_pass()` | 每帧样本数 |

## 测试和验证

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration_*

# 启用诊断特性测试
cargo test --features diagnostics

# 性能基准测试
cargo bench
```

## 构建特性

- `default` - 标准功能
- `diagnostics` - 启用内部诊断数据访问
- `logging` - 启用详细日志输出

```toml
[dependencies]
shine-rs = { version = "0.1", features = ["diagnostics", "logging"] }
```