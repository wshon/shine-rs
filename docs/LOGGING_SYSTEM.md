# 日志系统使用指南

## 概述

项目已经从直接的 `println!` 调试输出改为使用结构化的日志系统。这提供了更好的日志级别控制和格式化。

## 日志级别

### INFO 级别
- 用于重要的用户信息，如文件读取、编码进度、最终结果
- 默认显示级别
- 示例：文件信息、编码参数、完成状态

### DEBUG 级别  
- 用于详细的调试信息，包括算法内部状态
- 需要设置 `RUST_LOG=debug` 才能看到
- 示例：MDCT 系数、量化参数、比特流详情

### ERROR 级别
- 用于错误信息和失败情况
- 总是显示

## 启用核心库 Debug 日志的方法

### 方法1: 使用 diagnostics feature (推荐)

在 `tools/wav2mp3/Cargo.toml` 中启用 feature：
```toml
[dependencies]
shine-rs = { path = "../..", features = ["diagnostics"] }
```

### 方法2: 强制依赖库 debug 模式

在 `tools/wav2mp3/Cargo.toml` 中添加：
```toml
[profile.dev.package.shine-rs]
debug-assertions = true
opt-level = 0
```

### 方法3: 使用 debug 构建

确保使用 debug 构建而不是 release 构建：
```bash
cargo build  # debug 构建
# 而不是 cargo build --release
```

## 使用方法

### 基本使用（INFO 级别）
```bash
# 使用默认的 INFO 级别
tools/target/debug/wav2mp3.exe input.wav output.mp3
```

### 启用 DEBUG 日志
```bash
# Windows PowerShell
$env:RUST_LOG="debug"; tools/target/debug/wav2mp3.exe input.wav output.mp3

# Windows CMD
set RUST_LOG=debug && tools/target/debug/wav2mp3.exe input.wav output.mp3

# Linux/macOS
RUST_LOG=debug tools/target/debug/wav2mp3 input.wav output.mp3
```

### 控制 debug 日志帧数
```bash
# 显示前 10 帧的 debug 信息
$env:RUST_MP3_DEBUG_FRAMES="10"; $env:RUST_LOG="debug"; tools/target/debug/wav2mp3.exe input.wav output.mp3

# 只显示第 1 帧的 debug 信息
$env:RUST_MP3_DEBUG_FRAMES="1"; $env:RUST_LOG="debug"; tools/target/debug/wav2mp3.exe input.wav output.mp3
```

### 详细模式
```bash
# 使用 --verbose 标志显示帧级别的详细信息
tools/target/debug/wav2mp3.exe input.wav output.mp3 --verbose
```

## 日志输出示例

### INFO 级别输出
```
[INFO ] Reading WAV file: testing/fixtures/audio/sample-3s.wav
[INFO ] WAV info: 44100 Hz, 2 channels, 281856 samples
[INFO ] Encoding with: 128 kbps, mode 0
[INFO ] Encoding 122 frames of 1152 samples each
[INFO ] Total MP3 data: 50988 bytes
[INFO ] Writing MP3 file: testing/fixtures/output/sample-3s-output.mp3
✅ Conversion completed successfully!
   Input size:  563712 bytes
   Output size: 50988 bytes
   Compression: 11.1:1
   Duration:    3.20 seconds
```

### DEBUG 级别输出（核心库）

**注意**: 核心库的 debug 日志目前使用 `#[cfg(any(debug_assertions, feature = "diagnostics"))]` 条件编译，这意味着：
- 可以通过启用 `diagnostics` feature 在任何构建模式下使用
- 也可以在 debug 构建中自动启用
- 需要设置 `RUST_LOG=debug` 环境变量才能看到
- 在 release 构建中，如果没有启用 `diagnostics` feature，这些日志代码会被完全移除以保证性能

当前的 debug 日志包括：
```
[DEBUG] [Frame 1] MDCT[0][0][0][17] = 808302
[DEBUG] [Frame 1] MDCT[0][0][0][16] = 3145162
[DEBUG] [Frame 1] Saved l3_sb_sample[0][0][0][0] = -35329013
[DEBUG] [Frame 1] xrmax=174601576, max_bits=764
[DEBUG] [Frame 1] part2_3_length=763, quantizer_step_size=-40, global_gain=170
[DEBUG] [Frame 1] pad=1, bits=3344, written=416, slot_lag=-0.918367
```

**如果看不到核心库的 debug 日志**，请确保：
1. 启用了 `diagnostics` feature 或使用 debug 构建
2. 设置了 `RUST_LOG=debug` 环境变量
3. 这些日志只在算法的关键点输出（可通过 `RUST_MP3_DEBUG_FRAMES` 控制帧数）

## 配置选项

### 环境变量
- `RUST_LOG`: 控制日志级别 (`error`, `warn`, `info`, `debug`, `trace`)
- `RUST_MP3_MAX_FRAMES`: 限制编码的帧数（调试用）
- `RUST_MP3_DEBUG_FRAMES`: 控制 debug 日志输出的帧数（默认：6）

### 命令行选项
- `--verbose`: 启用详细的帧级别输出
- `--max-frames N`: 限制编码帧数

## 开发者注意事项

### 在代码中使用日志
```rust
use log::{info, debug, error};

// 信息级别 - 用户需要看到的重要信息
info!("Processing file: {}", filename);

// 调试级别 - 开发和调试信息
#[cfg(debug_assertions)]
debug!("Internal state: value={}", value);

// 错误级别 - 错误和失败
error!("Failed to process: {}", error);
```

### 日志格式
- 时间戳已禁用以保持输出简洁
- 模块路径已禁用
- 目标已禁用
- 格式：`[LEVEL] message`

## 与 shine 参考实现的对比

调试日志的格式设计为便于与 shine C 实现的输出进行对比：

```
# Rust 输出
[DEBUG] [Frame 1] MDCT[0][0][0][17] = 808302

# Shine 输出（如果添加相应的 printf）
[SHINE DEBUG Frame 1] MDCT[0][0][0][17] = 808302
```

这样可以方便地验证算法实现的正确性。