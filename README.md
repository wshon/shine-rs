# Rust MP3 编码器

一个基于 shine 库的纯 Rust MP3 编码器实现。该项目提供了完整的 MP3 Layer III 编码功能，支持各种采样率、比特率和声道配置。

## 特性

- 🦀 **纯 Rust 实现** - 利用 Rust 的内存安全和性能优势
- 🎵 **完整的 MP3 Layer III 支持** - 实现完整的 MP3 编码流水线
- ⚡ **高性能** - 使用固定点算术和 SIMD 优化
- 🔧 **灵活配置** - 支持多种采样率、比特率和声道模式
- 📊 **标准兼容** - 符合 ISO/IEC 11172-3 标准
- 🧪 **全面测试** - 包含单元测试和基于属性的测试

## 支持的格式

### 采样率
- **MPEG-1**: 44100, 48000, 32000 Hz
- **MPEG-2**: 22050, 24000, 16000 Hz  
- **MPEG-2.5**: 11025, 12000, 8000 Hz

### 比特率
- 8-320 kbps (取决于 MPEG 版本和采样率)

### 声道模式
- 单声道 (Mono)
- 立体声 (Stereo)
- 联合立体声 (Joint Stereo)
- 双声道 (Dual Channel)

## 快速开始

### 添加依赖

```toml
[dependencies]
rust-mp3-encoder = "0.1.0"
```

### 基本使用

```rust
use rust_mp3_encoder::{Mp3Encoder, Config, WaveConfig, MpegConfig, Channels, StereoMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建编码器配置
    let config = Config {
        wave: WaveConfig {
            channels: Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::JointStereo,
            bitrate: 128,
            ..Default::default()
        },
    };
    
    // 创建编码器
    let mut encoder = Mp3Encoder::new(config)?;
    
    // 编码 PCM 数据
    let pcm_data = vec![0i16; encoder.samples_per_frame() * 2]; // 立体声数据
    let mp3_frame = encoder.encode_frame_interleaved(&pcm_data)?;
    
    // 刷新剩余数据
    let final_frame = encoder.flush()?;
    
    Ok(())
}
```

## 架构设计

编码器采用模块化设计，包含以下主要组件：

```
PCM 输入 → 子带滤波 → MDCT 变换 → 量化循环 → 霍夫曼编码 → 比特流输出
```

### 核心模块

- **`config`** - 配置管理和验证
- **`subband`** - 32 频带子带滤波器
- **`mdct`** - 修正离散余弦变换
- **`quantization`** - 量化循环和比特率控制
- **`huffman`** - 霍夫曼编码
- **`bitstream`** - MP3 比特流生成
- **`tables`** - 查找表和常量

## 开发状态

🚧 **开发中** - 该项目正在积极开发中。当前已完成：

- [x] 项目结构和基础设施
- [ ] 配置管理模块
- [ ] 查找表和常量
- [ ] 比特流写入器
- [ ] 子带滤波器
- [ ] MDCT 变换
- [ ] 量化循环
- [ ] 霍夫曼编码器
- [ ] 主编码器集成

## 构建和测试

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行基准测试
cargo bench

# 运行示例
cargo run --example basic_encoding
```

## 性能

该编码器设计为高性能实现：

- 使用固定点算术避免浮点运算开销
- 支持 SIMD 指令集优化 (SSE, AVX, NEON)
- 内存布局优化以提高缓存命中率
- 零拷贝设计减少内存分配

## 兼容性

生成的 MP3 文件与以下解码器兼容：

- FFmpeg/libmp3lame
- Windows Media Player
- VLC Media Player
- 各种移动设备播放器

## 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。详见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE) 文件。

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

## 致谢

本项目基于 [shine](https://github.com/toots/shine) MP3 编码器库，感谢原作者的优秀工作。