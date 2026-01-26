# Examples - 示例代码

本目录包含了 shine-rs MP3 编码器的使用示例，展示了如何使用高级 API 进行 MP3 编码。

## 快速开始

```bash
# 运行主要示例（包含 WAV 文件测试）
cargo run --example simple_encoding

# 运行错误处理示例
cargo run --example error_handling

# 查看生成的文件
ls -la output_*.mp3
```

## 示例列表

### simple_encoding.rs - 简单编码示例

这是一个综合性的示例，展示了 `Mp3Encoder` 高级 API 的各种用法：

#### 功能特性

1. **真实 WAV 文件编码** - 读取并编码 `testing/fixtures/audio/sample-3s.wav` 文件
2. **合成音频编码** - 生成 440Hz 正弦波并编码为 MP3
3. **流式编码演示** - 展示如何分块处理大量音频数据
4. **多种配置测试** - 演示不同采样率、比特率和立体声模式
5. **错误处理演示** - 展示配置验证和错误处理机制

#### 运行示例

```bash
# 运行简单编码示例
cargo run --example simple_encoding
```

#### 输出文件

运行示例后会在项目根目录生成以下 MP3 文件：

| 文件名 | 描述 | 配置 | 大小 (约) |
|--------|------|------|-----------|
| `output_sample.mp3` | 从 sample-3s.wav 编码的 MP3 文件 | 44.1kHz, 128kbps, 立体声 | ~51KB |
| `output_simple.mp3` | 使用便利函数编码的合成音频 | 44.1kHz, 128kbps, 立体声 | ~16KB |
| `output_streaming.mp3` | 使用流式编码器编码的合成音频 | 44.1kHz, 128kbps, 立体声 | ~16KB |
| `output_mono.mp3` | 单声道低比特率版本 | 22kHz, 64kbps, 单声道 | ~16KB |
| `output_hq.mp3` | 高质量立体声版本 | 48kHz, 320kbps, 联合立体声 | ~40KB |

**文件特性说明:**
- 所有生成的 MP3 文件都符合标准 MPEG Layer III 格式
- 可以使用任何标准 MP3 播放器播放
- 文件头包含正确的 MPEG 帧同步信息
- 比特率和采样率信息正确编码在帧头中

### error_handling.rs - 错误处理示例

展示了各种错误情况的处理方式，包括：

- 无效的配置参数
- 不兼容的采样率和比特率组合
- 输入数据验证错误
- 编码过程中的错误处理

#### 运行示例

```bash
# 运行错误处理示例
cargo run --example error_handling
```

## API 使用指南

### 基本用法

```rust
use shine_rs::mp3_encoder::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

// 创建编码器配置
let config = Mp3EncoderConfig::new()
    .sample_rate(44100)
    .bitrate(128)
    .channels(2)
    .stereo_mode(StereoMode::Stereo);

// 创建编码器
let mut encoder = Mp3Encoder::new(config)?;

// 编码 PCM 数据
let frames = encoder.encode_interleaved(&pcm_data)?;

// 完成编码
let final_data = encoder.finish()?;
```

### 便利函数

```rust
use shine_rs::mp3_encoder::{encode_pcm_to_mp3, Mp3EncoderConfig};

// 一次性编码整个 PCM 数据
let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;
```

## 支持的配置

### 采样率 (Hz)
- MPEG-1: 32000, 44100, 48000
- MPEG-2: 16000, 22050, 24000  
- MPEG-2.5: 8000, 11025, 12000

### 比特率 (kbps)
- 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 192, 224, 256, 320

### 立体声模式
- `StereoMode::Stereo` - 标准立体声
- `StereoMode::JointStereo` - 联合立体声 (推荐)
- `StereoMode::DualChannel` - 双声道
- `StereoMode::Mono` - 单声道

## 文件格式说明

### 输入格式
- **PCM 数据**: 16-bit 有符号整数 (`i16`)
- **交错格式**: 左右声道样本交替排列 (L, R, L, R, ...)
- **分离格式**: 左右声道分别提供

### 输出格式
- **MP3 格式**: 符合 MPEG-1/2/2.5 Layer III 标准
- **比特流**: 标准 MP3 文件格式，可被任何 MP3 播放器播放

## 性能特性

### 压缩比
- 典型压缩比: 10:1 到 12:1 (128kbps)
- 高质量: 4:1 到 6:1 (320kbps)
- 低比特率: 15:1 到 20:1 (64kbps)

### 编码速度
- 实时编码: 支持实时音频流编码
- 批处理: 高效处理大文件
- 内存使用: 低内存占用，适合嵌入式应用

## 验证生成的文件

### 使用 FFmpeg 验证

```bash
# 查看 MP3 文件信息
ffprobe output_sample.mp3

# 播放文件
ffplay output_sample.mp3

# 转换为 WAV 进行比较
ffmpeg -i output_sample.mp3 output_sample_decoded.wav
```

### 使用十六进制查看器检查文件头

```bash
# Windows (PowerShell)
Format-Hex output_sample.mp3 | Select-Object -First 5

# Linux/macOS
hexdump -C output_sample.mp3 | head -5
```

**标准 MP3 文件头格式:**
- 前 4 字节应该是 `FF FB` 或类似的 MPEG 同步字
- 包含版本、层、比特率、采样率等信息

### 文件大小验证

生成的文件大小应该符合以下公式：
```
文件大小 (字节) ≈ (比特率 × 持续时间) / 8
```

例如：3.2 秒的音频，128kbps 比特率：
```
预期大小 = (128 × 1000 × 3.2) / 8 = 51,200 字节 ≈ 51KB
```

## 故障排除

### 常见错误

1. **不支持的采样率**
   ```
   Configuration error: Unsupported sample rate: 96000 Hz
   ```
   解决方案: 使用支持的采样率 (见上方列表)

2. **不兼容的比特率组合**
   ```
   Configuration error: Incompatible sample rate and bitrate combination
   ```
   解决方案: 检查 MPEG 版本限制，低采样率不支持高比特率

3. **无效的声道配置**
   ```
   Configuration error: Invalid stereo mode for channel count
   ```
   解决方案: 单声道使用 `StereoMode::Mono`，立体声使用其他模式

### 调试技巧

1. **启用详细日志**
   ```bash
   RUST_LOG=debug cargo run --example simple_encoding
   ```

2. **检查输出文件**
   ```bash
   # 检查 MP3 文件头
   hexdump -C output_sample.mp3 | head -5
   
   # 播放测试
   ffplay output_sample.mp3
   ```

3. **验证配置**
   ```rust
   config.validate()?; // 在创建编码器前验证配置
   ```

## 相关文档

- [高级 API 文档](../docs/HIGH_LEVEL_API.md)
- [项目结构说明](../docs/PROJECT_STRUCTURE.md)
- [测试数据框架](../docs/TEST_DATA_FRAMEWORK.md)