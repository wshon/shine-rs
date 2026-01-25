# MP3编码器测试数据框架

这个框架提供了一套完整的测试数据收集和验证系统，用于确保MP3编码器实现的正确性和一致性。

## 概述

测试数据框架包含以下组件：

1. **数据收集工具** (`collect_test_data`) - 收集编码过程中的关键参数
2. **验证工具** (`validate_test_data`) - 验证编码实现是否符合预期
3. **JSON格式** - 存储测试用例数据的标准格式
4. **可扩展架构** - 支持不同音频文件和编码参数的测试用例
5. **帧数限制** - 支持限制编码帧数进行快速测试和调试

## 工作流程

### 1. 收集测试数据

使用 `collect_test_data` 工具从音频文件生成测试数据：

```bash
# 基本用法
cargo run --bin collect_test_data -- <input.wav> <output.json> [bitrate] [--max-frames N]

# 示例
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/test_data.json 128
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/sample_3s_test_data.json 192

# 限制帧数进行快速测试
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/quick_test.json 128 --max-frames 3
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/medium_test.json 128 --max-frames 10
```

**参数说明：**
- `input.wav` - 输入WAV文件路径
- `output.json` - 输出JSON测试数据文件路径  
- `bitrate` - MP3比特率（可选，默认128 kbps）
- `--max-frames N` - 限制编码帧数（可选，默认6帧，仅debug模式生效）

### 2. 验证测试数据

使用 `validate_test_data` 工具验证编码实现：

```bash
# 验证测试用例
cargo run --bin validate_test_data -- <test_data.json> [--max-frames N]

# 示例
cargo run --bin validate_test_data -- testing/fixtures/data/test_data.json
cargo run --bin validate_test_data -- testing/fixtures/data/sample_3s_test_data.json

# 限制帧数验证
cargo run --bin validate_test_data -- testing/fixtures/data/quick_test.json --max-frames 3
```

### 3. 环境变量控制

也可以通过环境变量控制帧数限制：

```bash
# Windows PowerShell
$env:RUST_MP3_MAX_FRAMES=5
cargo run --bin collect_test_data -- input.wav output.json
Remove-Item Env:RUST_MP3_MAX_FRAMES

# Linux/macOS
export RUST_MP3_MAX_FRAMES=5
cargo run --bin collect_test_data -- input.wav output.json
unset RUST_MP3_MAX_FRAMES
```

## JSON数据格式

测试数据以JSON格式存储，包含以下结构：

### 元数据 (metadata)
```json
{
  "name": "test_case_test_input_44100hz_2ch_128kbps",
  "input_file": "testing/fixtures/audio/sample-3s.wav",
  "expected_output_size": 1252,
  "expected_hash": "861b8689d7eee5d408feec61cfa6ce6932168e35e6f86fa92bc5f3c77eb37c32",
  "created_at": "2026-01-25T03:20:58.540642300+00:00",
  "description": "Test case for sample-3s.wav at 128 kbps"
}
```

### 编码配置 (config)
```json
{
  "sample_rate": 44100,
  "channels": 2,
  "bitrate": 128,
  "stereo_mode": 0,
  "mpeg_version": 3
}
```

### 帧数据 (frames)
每帧包含以下关键参数：

#### MDCT系数 (mdct_coefficients)
```json
{
  "coefficients": [-14265082, -34259084, -52092837],
  "l3_sb_sample": [182957202]
}
```

#### 量化参数 (quantization)
```json
{
  "xrmax": 250370108,
  "max_bits": 764,
  "part2_3_length": 688,
  "quantizer_step_size": -50,
  "global_gain": 160
}
```

#### 比特流参数 (bitstream)
```json
{
  "padding": 1,
  "bits_per_frame": 3344,
  "written": 416,
  "slot_lag": -0.9183673469387941
}
```

## 验证项目

验证工具会检查以下项目：

### 1. 结构验证
- ✓ MDCT系数结构完整性
- ✓ 量化数据存在性
- ✓ 比特流数据存在性

### 2. 输出验证
- ✓ 输出文件大小匹配
- ✓ SHA256哈希值匹配

### 3. 参数一致性
- ✓ 编码配置匹配
- ✓ WAV文件参数匹配

## 使用场景

### 1. 快速调试测试
使用帧数限制进行快速调试和验证：

```bash
# 快速测试前3帧
cargo run --bin collect_test_data -- audio.wav quick_debug.json 128 --max-frames 3
cargo run --bin validate_test_data -- quick_debug.json --max-frames 3

# 中等规模测试（10帧）
cargo run --bin collect_test_data -- audio.wav medium_test.json 128 --max-frames 10
cargo run --bin validate_test_data -- medium_test.json --max-frames 10
```

### 2. 回归测试
确保代码修改不会破坏现有功能：

```bash
# 收集基准数据（完整文件）
cargo run --bin collect_test_data -- baseline.wav baseline_test.json 128

# 修改代码后验证
cargo run --bin validate_test_data -- baseline_test.json

# 快速回归测试（仅前几帧）
cargo run --bin collect_test_data -- baseline.wav quick_baseline.json 128 --max-frames 6
cargo run --bin validate_test_data -- quick_baseline.json --max-frames 6
```

### 3. 多参数测试
测试不同编码参数的组合：

```bash
# 不同比特率（限制帧数加速测试）
cargo run --bin collect_test_data -- audio.wav test_128k.json 128 --max-frames 5
cargo run --bin collect_test_data -- audio.wav test_192k.json 192 --max-frames 5
cargo run --bin collect_test_data -- audio.wav test_256k.json 256 --max-frames 5

# 验证所有配置
cargo run --bin validate_test_data -- test_128k.json --max-frames 5
cargo run --bin validate_test_data -- test_192k.json --max-frames 5
cargo run --bin validate_test_data -- test_256k.json --max-frames 5
```

### 4. 与Shine对比验证
确保与参考实现的一致性：

```bash
# 收集Rust实现的前6帧数据
cargo run --bin collect_test_data -- test.wav rust_6frames.json 128 --max-frames 6

# 使用相同帧数验证一致性
cargo run --bin validate_test_data -- rust_6frames.json --max-frames 6

# 对比不同帧数的结果
cargo run --bin collect_test_data -- test.wav rust_3frames.json 128 --max-frames 3
cargo run --bin collect_test_data -- test.wav rust_10frames.json 128 --max-frames 10
```

### 5. 跨平台验证
确保不同平台生成相同结果：

```bash
# 在平台A收集数据（限制帧数）
cargo run --bin collect_test_data -- test.wav platform_a_test.json 128 --max-frames 6

# 在平台B验证
cargo run --bin validate_test_data -- platform_a_test.json --max-frames 6
```

## 扩展测试用例

### 添加新的音频文件
1. 将WAV文件放入 `testing/fixtures/audio/` 目录
2. 使用 `collect_test_data` 生成测试数据到 `testing/fixtures/data/` 目录
3. 手动更新JSON中的 `expected_output_size` 和 `expected_hash`
4. 使用 `validate_test_data` 验证

### 添加新的编码参数
支持的参数组合：
- **采样率**: 44100, 48000, 32000, 22050, 24000, 16000, 11025, 12000, 8000 Hz
- **比特率**: 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320 kbps
- **声道**: 单声道 (mono), 立体声 (stereo)

### 自定义验证逻辑
可以扩展 `validate_test_data.rs` 来添加更多验证项目：

```rust
// 添加自定义验证
if expected_frame.custom_parameter > threshold {
    validation_results.pass();
    println!("  ✓ Custom validation passed");
} else {
    validation_results.fail("Custom validation failed".to_string());
}
```

## 帧数限制功能

### 概述
帧数限制功能允许在测试和调试时只处理音频文件的前N帧，大大加速测试过程。

### 特性
- **调试模式专用**: 只在debug构建中生效，release模式会忽略限制
- **多种控制方式**: 支持命令行参数和环境变量
- **快速测试**: 适用于算法验证、回归测试和调试

### 控制方式

#### 1. 命令行参数（推荐）
```bash
# collect_test_data工具
cargo run --bin collect_test_data -- input.wav output.json 128 --max-frames 3

# validate_test_data工具  
cargo run --bin validate_test_data -- test.json --max-frames 3

# wav2mp3工具
cargo run --bin wav2mp3 -- input.wav output.mp3 128 stereo --max-frames 5
```

#### 2. 环境变量
```bash
# Windows PowerShell
$env:RUST_MP3_MAX_FRAMES=6
cargo run --bin collect_test_data -- input.wav output.json

# Linux/macOS
export RUST_MP3_MAX_FRAMES=6
cargo run --bin collect_test_data -- input.wav output.json
```

### 优先级规则
1. 命令行参数 `--max-frames N`（最高优先级）
2. 环境变量 `RUST_MP3_MAX_FRAMES`
3. 工具默认值（collect_test_data默认6帧，其他工具无限制）

### 使用建议

#### 快速调试（1-3帧）
```bash
# 验证基本算法正确性
cargo run --bin collect_test_data -- test.wav debug.json 128 --max-frames 1
cargo run --bin validate_test_data -- debug.json --max-frames 1
```

#### 中等测试（5-10帧）
```bash
# 验证算法稳定性
cargo run --bin collect_test_data -- test.wav stable.json 128 --max-frames 6
cargo run --bin validate_test_data -- stable.json --max-frames 6
```

#### 完整测试（无限制）
```bash
# 生产环境验证
cargo run --bin collect_test_data -- test.wav full.json 128
cargo run --bin validate_test_data -- full.json
```

### 调试输出
在debug模式下会显示：
- 帧数限制设置：`Frame limit set to: N frames`
- 每帧详细参数：`[RUST F1] MDCT[...], xrmax=..., pad=...`
- 停止信息：`[RUST] Stopping after N frames for comparison`

## 调试模式

测试数据收集和帧数限制只在debug模式下工作：

```bash
# Debug模式 - 收集数据并显示详细输出
cargo run --bin collect_test_data -- input.wav output.json

# Debug模式 - 限制帧数进行快速测试
cargo run --bin collect_test_data -- input.wav output.json 128 --max-frames 3

# Release模式 - 正常编码，无数据收集，忽略帧数限制
cargo run --release --bin wav2mp3 -- input.wav output.mp3
```

### Debug vs Release行为对比

| 功能 | Debug模式 | Release模式 |
|------|-----------|-------------|
| 帧数限制 | ✅ 生效 | ❌ 忽略 |
| 调试输出 | ✅ 详细输出 | ❌ 无输出 |
| 测试数据收集 | ✅ 收集 | ❌ 不收集 |
| 编码性能 | 较慢（调试信息） | 最快（优化） |
| 用途 | 开发、测试、调试 | 生产环境 |

## 性能考虑

- 数据收集只影响debug构建，不影响release性能
- JSON文件大小与帧数成正比（每帧约100-200字节）
- 验证过程需要重新编码，时间与音频长度成正比

## 故障排除

### 常见错误

1. **输入文件不存在**
   ```
   Error: Input file 'test.wav' does not exist
   ```
   解决：检查文件路径是否正确

2. **JSON格式错误**
   ```
   Error: JSON parsing failed
   ```
   解决：检查JSON文件格式是否正确

3. **哈希不匹配**
   ```
   Output hash mismatch: Expected: xxx, Actual: yyy
   ```
   解决：检查编码实现是否有变化，或更新预期哈希值

4. **帧数限制在Release模式不生效**
   ```
   # Release模式会忽略帧数限制
   cargo run --release --bin wav2mp3 -- input.wav output.mp3 --max-frames 3
   ```
   解决：使用debug模式进行测试，release模式用于生产环境

5. **环境变量冲突**
   ```
   Frame limit set to: 10 frames  # 但命令行指定了 --max-frames 5
   ```
   解决：命令行参数优先级更高，或清除环境变量

### 帧数限制相关问题

1. **帧数限制不生效**
   - 确认使用debug模式：`cargo run`（不是`cargo run --release`）
   - 检查参数格式：`--max-frames 5`（不是`--max-frames=5`）
   - 验证环境变量：`echo $RUST_MP3_MAX_FRAMES`

2. **测试数据不匹配**
   - 确保收集和验证使用相同的帧数限制
   - 检查JSON文件中的帧数是否与预期一致
   - 验证音频文件是否相同

3. **性能问题**
   - 使用较少帧数进行快速测试（1-3帧）
   - 避免在CI中使用完整文件测试
   - 考虑使用环境变量批量设置帧数限制

### 调试技巧

1. **启用详细输出**：使用debug构建查看详细的编码参数
2. **分步验证**：先验证结构，再验证数值
3. **对比工具**：使用JSON diff工具比较不同版本的测试数据
4. **帧数对比**：使用不同帧数限制验证算法一致性
5. **环境隔离**：清除环境变量避免意外影响

### 调试命令示例

```bash
# 检查环境变量
echo $RUST_MP3_MAX_FRAMES  # Linux/macOS
echo $env:RUST_MP3_MAX_FRAMES  # Windows PowerShell

# 清除环境变量
unset RUST_MP3_MAX_FRAMES  # Linux/macOS
Remove-Item Env:RUST_MP3_MAX_FRAMES  # Windows PowerShell

# 验证帧数限制
cargo run --bin wav2mp3 -- test.wav output.mp3 --max-frames 1  # 应该很快完成
cargo run --release --bin wav2mp3 -- test.wav output.mp3 --max-frames 1  # 应该编码完整文件

# 对比不同帧数的输出
cargo run --bin collect_test_data -- test.wav test_1f.json 128 --max-frames 1
cargo run --bin collect_test_data -- test.wav test_3f.json 128 --max-frames 3
# 比较两个JSON文件的差异
```

## 最佳实践

1. **版本控制**：将测试数据JSON文件纳入版本控制
2. **命名规范**：使用描述性的文件名（如 `sample_44k_stereo_128k_3frames.json`）
3. **定期更新**：在重大算法改进后更新基准测试数据
4. **文档记录**：为每个测试用例添加清晰的描述
5. **自动化**：集成到CI/CD流程中进行自动验证
6. **帧数策略**：根据测试目的选择合适的帧数限制

### 帧数限制使用策略

#### 开发阶段
```bash
# 快速验证算法修改（1-3帧）
cargo run --bin collect_test_data -- test.wav dev_quick.json 128 --max-frames 1
cargo run --bin validate_test_data -- dev_quick.json --max-frames 1

# 中等验证（5-10帧）
cargo run --bin collect_test_data -- test.wav dev_medium.json 128 --max-frames 6
cargo run --bin validate_test_data -- dev_medium.json --max-frames 6
```

#### 测试阶段
```bash
# 回归测试（完整文件）
cargo run --bin collect_test_data -- test.wav regression.json 128
cargo run --bin validate_test_data -- regression.json

# 快速回归（前10帧）
cargo run --bin collect_test_data -- test.wav quick_regression.json 128 --max-frames 10
cargo run --bin validate_test_data -- quick_regression.json --max-frames 10
```

#### CI/CD集成
```bash
# 快速CI检查（前5帧，节省时间）
cargo run --bin validate_test_data -- ci_baseline.json --max-frames 5

# 完整验证（夜间构建）
cargo run --bin validate_test_data -- full_baseline.json
```

### 测试用例组织建议

#### 按帧数分类
- `*_1frame.json` - 单帧快速测试
- `*_3frames.json` - 基础算法验证
- `*_6frames.json` - 标准测试（默认）
- `*_full.json` - 完整文件测试

#### 按用途分类
- `debug_*.json` - 调试用测试用例
- `regression_*.json` - 回归测试基准
- `benchmark_*.json` - 性能基准测试
- `edge_case_*.json` - 边界条件测试

## 示例测试用例

项目包含以下预定义测试用例：

### 基础测试用例
1. **testing/fixtures/data/test_data.json** - 基础测试用例（44.1kHz, 立体声, 128kbps, 6帧）
2. **testing/fixtures/data/sample_3s_test_data.json** - 长音频测试用例（3秒音频, 完整）

### 帧数限制测试用例
3. **testing/fixtures/data/quick_test_3frames.json** - 快速测试（3帧）
4. **testing/fixtures/data/debug_test_1frame.json** - 调试测试（1帧）
5. **testing/fixtures/data/medium_test_10frames.json** - 中等测试（10帧）

### 多参数测试用例
6. **testing/fixtures/data/test_192k_6frames.json** - 192kbps测试（6帧）
7. **testing/fixtures/data/test_256k_6frames.json** - 256kbps测试（6帧）

这些测试用例覆盖了常见的编码场景和不同的帧数限制策略，可以作为回归测试的基础。

### 创建新测试用例的建议

```bash
# 创建快速调试用例
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/debug_1frame.json 128 --max-frames 1

# 创建标准回归用例
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/regression_6frames.json 128 --max-frames 6

# 创建完整验证用例
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/full_validation.json 128

# 创建不同比特率用例
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/test_192k_5frames.json 192 --max-frames 5
```