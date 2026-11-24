[English Documentation](https://github.com/wshon/shine-rs/blob/main/crate/README.md)

# Shine-RS

一个基于 Shine 库的纯 Rust MP3 编码器实现。该项目严格遵循 Shine C 语言参考实现，提供完整的 MP3 Layer III 编码功能，支持各种采样率、比特率和声道配置。

**项目地址**: https://github.com/wshon/shine-rs

[![License: LGPL-2.0](https://img.shields.io/badge/License-LGPL%202.0-blue.svg)](https://github.com/wshon/shine-rs/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

## 特性

- 🦀 **纯 Rust 实现** - 利用 Rust 的内存安全和性能优势
- 🎯 **严格遵循 Shine** - 算法与 Shine C 实现完全一致，确保输出质量
- 🎵 **完整的 MP3 Layer III 支持** - 实现完整的 MP3 编码流水线
- ⚡ **卓越性能** - 所有测试配置均快于 Shine C 1.14–1.35 倍
- 🔧 **灵活配置** - 支持多种采样率、比特率和声道模式
- 📊 **标准兼容** - 符合 ISO/IEC 11172-3 标准
- 🧪 **全面测试** - 包含单元测试、集成测试和与 Shine 的对比验证
- 🛠️ **实用工具** - 提供 WAV 转 MP3 命令行工具和测试数据收集工具
- 📋 **调试支持** - 可选的调试日志和帧数限制功能

## 支持的格式

### 采样率
- **MPEG-1**: 32000, 44100, 48000 Hz
- **MPEG-2**: 16000, 22050, 24000 Hz  
- **MPEG-2.5**: 8000, 11025, 12000 Hz

### 比特率
- **MPEG-1**: 32-320 kbps
- **MPEG-2**: 8-160 kbps
- **MPEG-2.5**: 8-64 kbps

### 声道模式
- 单声道 (Mono)
- 立体声 (Stereo)
- 联合立体声 (Joint Stereo)
- 双声道 (Dual Channel)

## 快速开始

### 使用命令行工具

```bash
# 基本用法：WAV 转 MP3
cargo run tests/audio/inputs/basic/sample-3s.wav output.mp3

# 指定比特率和立体声模式
cargo run input.wav output.mp3 128 stereo

# 调试模式：限制编码帧数
cargo run input.wav output.mp3 --max-frames 10

# 详细输出模式
cargo run input.wav output.mp3 --verbose
```

### 作为库使用

```toml
[dependencies]
shine-rs = { git = "https://github.com/wshon/shine-rs" }
```

```rust
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建编码器配置
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .bitrate(128)
        .channels(2)
        .stereo_mode(StereoMode::Stereo);
    
    // 创建编码器
    let mut encoder = Mp3Encoder::new(config)?;
    
    // 编码 PCM 数据
    let pcm_data = vec![0i16; encoder.samples_per_frame()];
    let mp3_frames = encoder.encode_interleaved(&pcm_data)?;
    
    // 完成编码
    let final_data = encoder.finish()?;
    
    println!("编码完成，生成 {} 字节 MP3 数据", final_data.len());
    Ok(())
}
```

> 💡 **提示**: 项目还提供了底层接口，直接对应 Shine C 实现。详见 [高级 API 使用指南](https://github.com/wshon/shine-rs/blob/main/docs/HIGH_LEVEL_API.md)。

## 项目结构

```
shine-rs/
├── 📁 crate/                    # 发布的库代码 (shine-rs)
│   ├── src/                     # 核心 MP3 编码器实现
│   │   ├── bitstream.rs         # 比特流处理
│   │   ├── encoder.rs           # 主编码器（底层接口）
│   │   ├── mp3_encoder.rs       # 高级编码器接口
│   │   ├── huffman.rs           # Huffman 编码
│   │   ├── mdct.rs              # MDCT 变换
│   │   ├── quantization.rs      # 量化算法
│   │   ├── subband.rs           # 子带分析
│   │   ├── tables.rs            # 查找表
│   │   ├── reservoir.rs         # 比特池管理
│   │   ├── error.rs             # 错误处理
│   │   ├── types.rs             # 类型定义
│   │   └── lib.rs               # 库入口
│   ├── tests/                   # 库的单元测试
│   └── Cargo.toml               # 库配置
├── 📁 src/                      # CLI 工具代码
│   ├── main.rs                  # WAV 转 MP3 命令行工具
│   ├── util.rs                  # 工具函数（WAV 读取、PCM 处理等）
│   └── lib.rs                   # CLI 工具库入口
├── 📁 tests/                    # 集成测试和基准测试
│   ├── encoder_benchmark.rs     # 性能基准测试
│   ├── encoder_validation_*.rs  # 编码器验证测试
│   └── audio/                   # 测试音频文件
├── 📁 ref/shine/                # Shine C 参考实现
├── 📁 docs/                     # 项目文档
└── 📁 scripts/                  # 辅助脚本
```

### 核心算法流程

```
PCM 输入 → 子带滤波 → MDCT 变换 → 量化循环 → Huffman 编码 → 比特流输出
```

每个步骤都严格按照 Shine C 实现，确保算法的正确性和输出的一致性。

## 开发状态

✅ **已完成** - 该项目已实现完整的 MP3 编码功能：

- [x] 项目结构和基础设施
- [x] 配置管理模块
- [x] 查找表和常量
- [x] 比特流写入器
- [x] 子带滤波器（32 频带分析）
- [x] MDCT 变换（修正离散余弦变换）
- [x] 量化循环（比特率控制）
- [x] Huffman 编码器
- [x] 主编码器集成
- [x] 与 Shine 输出完全一致验证
- [x] **性能优化** - 所有测试配置均快于 Shine C 1.14–1.35 倍
- [x] **代码质量** - 零编译警告，通过所有静态检查
- [x] **性能基准测试** - 完整的 Shine-RS vs Shine 性能对比系统

### 质量保证

- **算法验证**: 所有核心算法都与 Shine C 实现逐行对比验证
- **输出一致性**: 生成的 MP3 文件与 Shine 输出完全相同（SHA256 哈希匹配）
- **全面测试**: 包含单元测试、集成测试、属性测试、回归测试和性能基准测试
- **标准符合**: 严格遵循 ISO/IEC 11172-3 MP3 标准
- **代码质量**: 零编译警告，通过 clippy 静态检查
- **性能验证**: 通过真实音频文件验证，确保优化不影响算法正确性

## 构建和测试

```bash
# 构建项目
cargo build --release

# 运行所有测试
cargo test

# 运行性能基准测试（纯编码口径，需 shine-ref 源码）
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh

# 运行集成测试
cargo test --test integration_full_pipeline_validation

# 使用新的参考验证系统
cargo test encoder_validation_cicd

# 运行命令行工具
cargo run --release tests/audio/inputs/basic/sample-3s.wav output.mp3
```

### 性能基准测试

```bash
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh
```

对比结果见下方「性能和兼容性」。

### 调试和开发

```bash
# 启用调试日志
RUST_LOG=debug cargo run --release input.wav output.mp3

# 限制编码帧数（调试用）
cargo run --release input.wav output.mp3 --max-frames 5

# 运行性能基准测试（纯编码口径，需 shine-ref 源码）
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh

# 详细输出模式
cargo run --release input.wav output.mp3 --verbose
```

## 性能和兼容性

### 🚀 性能优势

本项目在保持算法完全一致的前提下，实现了卓越的编码性能：

#### 性能对比

对比 Shine-RS 与原始 Shine C 的**纯编码速度**：两端均先把音频解码到内存，再只测量编码核心（`shine_initialise` + 编码循环 + `flush`），不含文件读取与写入。

复现方式（需 shine-ref 源码）：

```bash
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh
```

Linux 纯编码倍率（2026-11-19，每组 5 次取中位；不含文件 I/O）：

| 测试音频 | 码率 | Shine-RS | Shine C |
|---|---|---|---|
| 15s 立体声 | 128k | 149.0x | 119.2x |
| 15s 立体声 | 192k | 133.2x | 116.4x |
| 15s 立体声 | 320k | 140.1x | 113.6x |
| 3s 立体声 | 128k | 146.6x | 110.4x |
| 3s 立体声 | 192k | 136.7x | 108.0x |
| 3s 立体声 | 320k | 132.5x | 105.3x |
| 500KB 音频 | 128k | 144.6x | 121.3x |
| 500KB 音频 | 192k | 136.8x | 115.2x |
| 500KB 音频 | 320k | 129.5x | 105.5x |
| 语音录音 | 128k | 287.5x | 218.4x |
| 语音录音 | 192k | 257.3x | 208.6x |
| 语音录音 | 320k | 259.2x | 192.0x |

所有配置下 Shine-RS 均更快，幅度 1.14–1.35 倍。差距来自 MDCT 热循环中用回绕运算替代饱和加法，以及量化器直接复用预计算的绝对值数组（`xrabs`），不再每次调用都重算 `labs`。


#### 技术优势

- **算法保真**: 所有参考配置下与 Shine C 输出逐字节一致
- **MDCT 热循环无溢出分支**: 回绕加法与 C 的 `+=` 一致，经 debug 溢出检查验证
- **量化热循环无逐次 abs**: 直接复用每 granule 预计算一次的 `xrabs` 绝对值数组（值完全一致，含 `i32::MIN` 的饱和情形）
- **LLVM LTO**: 全链接时优化，`codegen-units=1`，native CPU 调优

### 算法特征

- **算法优化**: 基于 Shine 的定点 C 实现移植
- **编码速度**: 快于 Shine C（1.14–1.35 倍，见上表）
- **输出**: 与 Shine C 逐字节一致，经 SHA256 对照参考文件验证

### 兼容性

生成的 MP3 文件与以下解码器完全兼容：

- FFmpeg/libmp3lame
- Windows Media Player
- VLC Media Player
- 各种移动设备播放器
- 所有符合 MP3 标准的播放器

### 质量保证

- **位级精确**: 与 Shine 生成完全相同的 MP3 比特流
- **标准符合**: 严格遵循 ISO/IEC 11172-3 标准
- **回归测试**: 防止算法修改引入的问题
- **持续验证**: 每次修改都与 Shine 输出对比验证

## 文档

- [项目结构说明](https://github.com/wshon/shine-rs/blob/main/docs/PROJECT_STRUCTURE.md) - 详细的项目组织结构
- [测试数据框架](https://github.com/wshon/shine-rs/blob/main/docs/TEST_DATA_FRAMEWORK.md) - 测试数据收集和验证系统
- [帧数限制功能](https://github.com/wshon/shine-rs/blob/main/docs/FRAME_LIMIT_FEATURE.md) - 调试和测试的帧数限制功能
- [帧数限制快速参考](https://github.com/wshon/shine-rs/blob/main/docs/FRAME_LIMIT_QUICK_REFERENCE.md) - 帧数限制功能的快速使用指南
- [高级 API 使用指南](https://github.com/wshon/shine-rs/blob/main/docs/HIGH_LEVEL_API.md) - 高级接口的使用方法
- [日志系统使用指南](https://github.com/wshon/shine-rs/blob/main/docs/LOGGING_SYSTEM.md) - 调试日志的配置和使用
- [音频文件标准化](https://github.com/wshon/shine-rs/blob/main/docs/AUDIO_FILES_STANDARDIZATION.md) - 测试音频文件的组织规范
- [验证记录](https://github.com/wshon/shine-rs/blob/main/docs/VERIFICATION_RECORD.md) - 与 Shine 实现的验证记录
- [性能基准测试](https://github.com/wshon/shine-rs/blob/main/tests/encoder_benchmark.rs) - Shine-RS vs Shine 性能对比

## 许可证

本项目采用 GNU Library General Public License v2.0 (LGPL-2.0) 发布。详见 [LICENSE](https://github.com/wshon/shine-rs/blob/main/LICENSE) 文件。

## 贡献

欢迎贡献！在提交代码前，请确保：

1. 遵循项目的编码规范（严格按照 Shine 实现）
2. 所有测试通过，包括与 Shine 的对比验证
3. 代码通过 `cargo clippy` 检查，无警告
4. 为新功能添加相应的测试用例

## 致谢

本项目基于 [Shine](https://github.com/toots/shine) MP3 编码器库开发，谨向所有为该项目付出努力的开发者致以诚挚感谢。感谢 Gabriel Bouvigne 创作原始核心代码，Pete Everett 完成定点数运算移植以适配无 FPU 设备，Patrick Roberts 实现多平台适配与库化重构，同时感谢 Savonet 团队对该项目的长期维护与迭代，为开源社区提供了高质量的定点数 MP3 编码方案。

Shine-RS 严格遵循 Shine 的核心算法实现，延续其定点数编码优势，确保 MP3 编码的质量稳定性与标准符合性（兼容 ISO/IEC 11172-3 标准），同时通过 Rust 语言特性实现了显著的性能提升。

---

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
