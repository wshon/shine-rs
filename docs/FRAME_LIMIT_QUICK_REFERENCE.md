# 帧数限制功能快速参考

## 基本用法

### 命令行参数（推荐）
```bash
# wav2mp3工具
cargo run --bin wav2mp3 input.wav output.mp3 --max-frames N

# collect_test_data工具
cargo run --bin collect_test_data input.wav output.json [bitrate] --max-frames N

# validate_test_data工具
cargo run --bin validate_test_data test.json --max-frames N
```

### 环境变量
```bash
# Windows PowerShell
$env:RUST_MP3_MAX_FRAMES=N

# Linux/macOS
export RUST_MP3_MAX_FRAMES=N
```

## 优先级规则
1. 命令行参数 `--max-frames N` （最高）
2. 环境变量 `RUST_MP3_MAX_FRAMES`
3. 工具默认值（collect_test_data默认6帧）

## 模式对比

| 功能 | Debug模式 | Release模式 |
|------|-----------|-------------|
| 帧数限制 | ✅ 生效 | ❌ 忽略 |
| 调试输出 | ✅ 详细 | ❌ 无 |
| 性能 | 较慢 | 最快 |

## 常用帧数建议

| 帧数 | 用途 | 时间 |
|------|------|------|
| 1 | 快速调试 | 秒级 |
| 3 | 基础验证 | 秒级 |
| 6 | 标准测试 | 秒级 |
| 10 | 中等测试 | 秒级 |
| 无限制 | 完整验证 | 分钟级 |

## 快速示例

```bash
# 快速调试（1帧）
cargo run --bin wav2mp3 test.wav debug.mp3 --max-frames 1

# 标准测试（6帧）
cargo run --bin collect_test_data test.wav test.json 128 --max-frames 6

# 环境变量批量设置
$env:RUST_MP3_MAX_FRAMES=3
cargo run --bin collect_test_data test1.wav test1.json
cargo run --bin collect_test_data test2.wav test2.json
Remove-Item Env:RUST_MP3_MAX_FRAMES

# 完整编码（生产环境）
cargo run --release --bin wav2mp3 input.wav output.mp3
```

## 故障排除

### 帧数限制不生效
- ✅ 使用debug模式：`cargo run`
- ❌ 避免release模式：`cargo run --release`

### 参数格式
- ✅ 正确：`--max-frames 5`
- ❌ 错误：`--max-frames=5`

### 环境变量检查
```bash
# 检查当前值
echo $env:RUST_MP3_MAX_FRAMES  # Windows
echo $RUST_MP3_MAX_FRAMES      # Linux/macOS

# 清除变量
Remove-Item Env:RUST_MP3_MAX_FRAMES  # Windows
unset RUST_MP3_MAX_FRAMES            # Linux/macOS
```

## 调试输出示例

```
Frame limit set to: 3 frames
[RUST F1] MDCT[0][0][0][17] = 808302
[RUST F1] xrmax=174601576, max_bits=764
[RUST F1] pad=1, bits=3344, written=416, slot_lag=-0.918367
[RUST F2] ...
[RUST F3] ...
[RUST] Stopping after 3 frames for comparison
```