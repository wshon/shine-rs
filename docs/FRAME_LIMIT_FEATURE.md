# 帧数限制功能

## 概述

帧数限制功能允许用户在调试和测试时限制MP3编码器处理的帧数，这对于快速测试、调试和验证非常有用。

## 功能特性

- **调试模式专用**: 帧数限制只在debug模式下生效，release模式下会编码完整文件
- **多种控制方式**: 支持命令行参数和环境变量两种控制方式
- **所有工具支持**: wav2mp3、collect_test_data、validate_test_data都支持帧数限制

## 使用方法

### 1. 命令行参数方式

#### wav2mp3工具
```bash
# 限制编码3帧
cargo run --bin wav2mp3 input.wav output.mp3 --max-frames 3

# 限制编码10帧
cargo run --bin wav2mp3 input.wav output.mp3 128 stereo --max-frames 10
```

#### collect_test_data工具
```bash
# 收集3帧的测试数据
cargo run --bin collect_test_data input.wav test_data.json --max-frames 3

# 收集10帧的测试数据，使用192kbps
cargo run --bin collect_test_data input.wav test_data.json 192 --max-frames 10
```

#### validate_test_data工具
```bash
# 验证时限制3帧
cargo run --bin validate_test_data test_data.json --max-frames 3
```

### 2. 环境变量方式

#### Windows PowerShell
```powershell
# 设置环境变量
$env:RUST_MP3_MAX_FRAMES=5

# 运行工具（会自动使用环境变量设置的帧数）
cargo run --bin wav2mp3 input.wav output.mp3

# 清除环境变量
Remove-Item Env:RUST_MP3_MAX_FRAMES
```

#### Linux/macOS
```bash
# 设置环境变量
export RUST_MP3_MAX_FRAMES=5

# 运行工具
cargo run --bin wav2mp3 input.wav output.mp3

# 清除环境变量
unset RUST_MP3_MAX_FRAMES
```

## 优先级规则

参数优先级从高到低：
1. 命令行参数 `--max-frames N`
2. 环境变量 `RUST_MP3_MAX_FRAMES`
3. 默认值（collect_test_data默认6帧，其他工具无限制）

## 调试输出

在debug模式下，当设置了帧数限制时，会显示：
- 帧数限制设置信息：`Frame limit set to: N frames`
- 每帧的详细编码参数
- 停止信息：`[RUST] Stopping after N frames for comparison`

## Release模式行为

在release模式下：
- 不会显示调试输出
- 不会应用帧数限制（会编码完整文件）
- 环境变量和命令行参数会被忽略

## 使用场景

### 1. 快速调试
```bash
# 只编码前3帧进行快速测试
cargo run --bin wav2mp3 test.wav debug.mp3 --max-frames 3
```

### 2. 与Shine对比验证
```bash
# 编码相同帧数进行对比
cargo run --bin wav2mp3 input.wav rust_output.mp3 --max-frames 6
# 然后与shine生成的前6帧进行对比
```

### 3. 测试数据收集
```bash
# 收集特定帧数的测试数据
cargo run --bin collect_test_data sample.wav test_case.json --max-frames 10
```

### 4. 回归测试
```bash
# 验证特定帧数的编码一致性
cargo run --bin validate_test_data test_case.json --max-frames 10
```

## 注意事项

1. **仅调试模式**: 帧数限制只在debug编译下生效
2. **完整性**: 限制帧数的文件可能不是有效的完整MP3文件
3. **测试用途**: 主要用于调试、测试和验证，不适用于生产环境
4. **一致性**: 确保在对比测试时使用相同的帧数限制

## 实现细节

- 帧数限制通过环境变量`RUST_MP3_MAX_FRAMES`在编码器内部实现
- 当达到限制时，编码器返回`StopAfterFrames`错误
- 工具捕获此错误并正常完成文件写入
- 使用条件编译`#[cfg(debug_assertions)]`确保只在debug模式生效