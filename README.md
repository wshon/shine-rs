# Shine-RS

一个基于 Shine 库的纯 Rust MP3 编码器实现。该项目严格遵循 Shine C 语言参考实现，提供完整的 MP3 Layer III 编码功能，支持各种采样率、比特率和声道配置。

**项目地址**: https://github.com/wshon/shine-rs

[![License: LGPL-2.0](https://img.shields.io/badge/License-LGPL%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

## 特性

- 🦀 **纯 Rust 实现** - 利用 Rust 的内存安全和性能优势
- 🎯 **严格遵循 Shine** - 算法与 Shine C 实现完全一致，确保输出质量
- 🎵 **完整的 MP3 Layer III 支持** - 实现完整的 MP3 编码流水线
- ⚡ **卓越性能** - 与原始 Shine C 性能相当，多数配置达其 80% 以上，高码率基本持平
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
- [x] **性能优化** - 与 Shine C 性能相当，多数配置在其 80% 以上
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
SHINE_REF=../shine-ref bash scripts/bench/build_and_run.sh

# 运行集成测试
cargo test --test integration_full_pipeline_validation

# 使用新的参考验证系统
cargo test encoder_validation_cicd

# 运行命令行工具
cargo run --release tests/audio/inputs/basic/sample-3s.wav output.mp3
```

### 性能基准测试

项目提供对称的纯编码基准（Rust `crate/examples/bench_encode.rs` vs C `scripts/bench/bench_encode_c.c`）：

```bash
# 一键对比（自动编译两个 harness）
SHINE_REF=../shine-ref bash scripts/bench/build_and_run.sh

# 测试特点：
# - WAV 解码到内存后才开始计时，不计入文件 I/O 与解析
# - 只测 shine_initialise + 编码循环 + flush，输出丢弃
# - 每个配置跑 5 次取中位数，两端口径完全一致
```

结果示例（Linux，纯编码倍率）：
```
sample-15s.wav    128kbps  Rust  90.9x  C 112.8x  ratio 0.81
sample-15s.wav    320kbps  Rust  79.1x  C  78.5x  ratio 1.01
voice-recorder    128kbps  Rust 169.2x  C 205.3x  ratio 0.82
```

### 调试和开发

```bash
# 启用调试日志
RUST_LOG=debug cargo run --release input.wav output.mp3

# 限制编码帧数（调试用）
cargo run --release input.wav output.mp3 --max-frames 5

# 运行性能基准测试（纯编码口径，需 shine-ref 源码）
SHINE_REF=../shine-ref bash scripts/bench/build_and_run.sh

# 详细输出模式
cargo run --release input.wav output.mp3 --verbose
```

## 性能和兼容性

### 🚀 性能优势

本项目在保持算法完全一致的前提下，实现了卓越的编码性能：

#### 测量方法

两端使用**对称的纯编码 harness**：先把 WAV 解码到内存（计时区外），然后用高精度时钟只测 `shine_initialise` + 编码循环 + `shine_flush`，输出直接丢弃。这样 WAV 解析、文件写入、进程启动都不计入。每个配置跑 5 次取中位数。

- Rust harness: `crate/examples/bench_encode.rs`
- C harness: `scripts/bench/bench_encode_c.c`（需 shine-ref 源码，见 `scripts/bench/build_and_run.sh`）
- 一键复现: `SHINE_REF=/path/to/shine-ref bash scripts/bench/build_and_run.sh`

> 旧版 `scripts/benchmark_encoders.py` 读取编码器内部输出，但 Shine C 原始 `main.c` 用秒级 `time()`，短音频只能得到 `infx`，无法可靠对比；本页数据统一采用上述 harness。

#### Linux 实测（2026-11-18，x86-64，rustc 1.98 / gcc 12）

纯编码倍率（中位数）：

| 测试音频 | 码率 | Shine-RS | Shine C | 比值 |
|---|---|---|---|---|
| 15s 立体声 | 128k | 90.9x | 112.8x | 0.81 |
| 15s 立体声 | 192k | 85.9x | 91.7x | 0.94 |
| 15s 立体声 | 320k | 79.1x | 78.5x | 1.01 |
| 3s 立体声 | 128k | 82.4x | 89.8x | 0.92 |
| 3s 立体声 | 192k | 72.2x | 89.0x | 0.81 |
| 3s 立体声 | 320k | 79.1x | 87.4x | 0.91 |
| 500KB 音频 | 128k | 79.7x | 99.6x | 0.80 |
| 500KB 音频 | 192k | 83.6x | 100.0x | 0.84 |
| 500KB 音频 | 320k | 78.6x | 93.0x | 0.85 |
| 语音录音 | 128k | 169.2x | 205.3x | 0.82 |
| 语音录音 | 192k | 181.3x | 192.9x | 0.94 |
| 语音录音 | 320k | 158.5x | 164.0x | 0.97 |

- 全部配置均达 **72x+** 实时编码；Shine-RS 与 Shine C 的比值大致在 **0.80–1.01**，高码率下基本持平。
- 改动前后对比（同一 harness、同一机器）：PI64/LN2 常量替换与 clippy 迭代器改写**未造成性能退化**，差异落在测量噪声内。

#### Windows 数据（历史记录）

早期在 Windows 上用编码器内部输出测得：Shine-RS 平均 114.1x、Shine C 平均 130.4x（差距约 14%），320kbps 时差距最小（120.0x vs 123.5x）。该组数据来自另一台机器与旧测量口径，与上面的 Linux 实测不可直接比较，仅作历史参考。

#### 性能分析

1. **算法实现质量高**: Rust 实现与手工优化的 C 代码性能相当，多数配置在 20% 以内
2. **编译器优化有效**: 高码率场景两者基本持平（部分配置 Rust 反超）
3. **内存安全代价小**: 获得内存安全的同时，性能损失在可接受范围内
4. **表现稳定**: 不同音频类型和码率下都保持超实时编码


#### 技术优势

- **内存安全**: Rust 的零成本抽象和内存安全保证
- **现代编译优化**: LLVM 后端的先进优化技术
- **算法保真**: 在保持与 Shine 完全一致的前提下优化实现
- **零拷贝**: 利用 Rust 所有权系统减少不必要的内存分配

> 💡 **性能测试**: 运行 `SHINE_REF=../shine-ref bash scripts/bench/build_and_run.sh` 复现上面的对比数据

### 算法特征

### 算法特征

- **算法优化**: 基于 Shine 的高效 C 实现移植，使用 Rust 技术进一步优化
- **内存安全**: Rust 的零成本抽象和内存安全保证
- **编码速度**: 与 Shine C 实现性能相当，仅14%差距的卓越表现
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
- [性能基准测试](tests/encoder_benchmark.rs) - Shine-RS vs Shine 性能对比

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

Shine-RS 严格遵循 Shine 的核心算法实现，延续其定点数编码优势，确保 MP3 编码的质量稳定性与标准符合性（兼容 ISO/IEC 11172-3 标准），同时通过 Rust 语言特性实现了显著的性能提升。
