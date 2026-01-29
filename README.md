# Rust MP3 编码器

一个基于 Shine 库的纯 Rust MP3 编码器实现。该项目严格遵循 Shine C 语言参考实现，提供完整的 MP3 Layer III 编码功能，支持各种采样率、比特率和声道配置。

**项目地址**: https://github.com/wshon/shine-rs

[![License: LGPL-2.0](https://img.shields.io/badge/License-LGPL%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

## 特性

- 🦀 **纯 Rust 实现** - 利用 Rust 的内存安全和性能优势
- 🎯 **严格遵循 Shine** - 算法与 Shine C 实现完全一致，确保输出质量
- 🎵 **完整的 MP3 Layer III 支持** - 实现完整的 MP3 编码流水线
- ⚡ **高性能** - 优化的算法实现，保持与 Shine 相同的性能特征
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
cargo run testing/fixtures/audio/sample-3s.wav output.mp3

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

> 💡 **提示**: 项目还提供了底层接口，直接对应 Shine C 实现。详见 [高级 API 使用指南](docs/HIGH_LEVEL_API.md)。

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
├── 📁 testing/                  # 测试相关文件
│   ├── fixtures/                # 测试数据和音频文件
│   └── integration/             # 集成测试
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

### 质量保证

- **算法验证**: 所有核心算法都与 Shine C 实现逐行对比验证
- **输出一致性**: 生成的 MP3 文件与 Shine 输出完全相同（SHA256 哈希匹配）
- **全面测试**: 包含单元测试、集成测试、属性测试和回归测试
- **标准符合**: 严格遵循 ISO/IEC 11172-3 MP3 标准

## 构建和测试

```bash
# 构建项目
cargo build

# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration_full_pipeline_validation

# 使用新的参考验证系统
cargo test encoder_validation_cicd

# 运行命令行工具
cargo run testing/fixtures/audio/sample-3s.wav output.mp3
```

### 调试和开发

```bash
# 启用调试日志
RUST_LOG=debug cargo run input.wav output.mp3

# 限制编码帧数（调试用）
cargo run input.wav output.mp3 --max-frames 5
```

## 性能和兼容性

### 性能特征

- **算法优化**: 基于 Shine 的高效 C 实现移植
- **内存安全**: Rust 的零成本抽象和内存安全保证
- **编码速度**: 与 Shine C 实现相当的编码性能
- **资源使用**: 优化的内存布局和缓存友好的数据访问

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

- [项目结构说明](docs/PROJECT_STRUCTURE.md) - 详细的项目组织结构
- [测试数据框架](docs/TEST_DATA_FRAMEWORK.md) - 测试数据收集和验证系统
- [帧数限制功能](docs/FRAME_LIMIT_FEATURE.md) - 调试和测试的帧数限制功能
- [帧数限制快速参考](docs/FRAME_LIMIT_QUICK_REFERENCE.md) - 帧数限制功能的快速使用指南
- [高级 API 使用指南](docs/HIGH_LEVEL_API.md) - 高级接口的使用方法
- [日志系统使用指南](docs/LOGGING_SYSTEM.md) - 调试日志的配置和使用
- [音频文件标准化](docs/AUDIO_FILES_STANDARDIZATION.md) - 测试音频文件的组织规范
- [验证记录](docs/VERIFICATION_RECORD.md) - 与 Shine 实现的验证记录

## 许可证

本项目采用 GNU Library General Public License v2.0 (LGPL-2.0) 发布。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎贡献！在提交代码前，请确保：

1. 遵循项目的编码规范（严格按照 Shine 实现）
2. 所有测试通过，包括与 Shine 的对比验证
3. 代码通过 `cargo clippy` 检查，无警告
4. 为新功能添加相应的测试用例

## 致谢

本项目基于 [Shine](https://github.com/toots/shine) MP3 编码器库开发，谨向所有为该项目付出努力的开发者致以诚挚感谢。感谢 Gabriel Bouvigne 创作原始核心代码，Pete Everett 完成定点数运算移植以适配无 FPU 设备，Patrick Roberts 实现多平台适配与库化重构，同时感谢 Savonet 团队对该项目的长期维护与迭代，为开源社区提供了高质量的定点数 MP3 编码方案。

本项目严格遵循 Shine 的核心算法实现，延续其定点数编码优势，确保 MP3 编码的质量稳定性与标准符合性（兼容 ISO/IEC 11172-3 标准）。
