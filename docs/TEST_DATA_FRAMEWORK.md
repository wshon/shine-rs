# MP3编码器测试数据框架

这个框架提供了一套完整的测试数据收集和验证系统，用于确保MP3编码器实现的正确性和一致性。

## 概述

测试数据框架包含以下组件：

1. **数据收集工具** (`collect_test_data`) - 收集编码过程中的关键参数
2. **验证工具** (`validate_test_data`) - 验证编码实现是否符合预期
3. **JSON格式** - 存储测试用例数据的标准格式
4. **可扩展架构** - 支持不同音频文件和编码参数的测试用例

## 工作流程

### 1. 收集测试数据

使用 `collect_test_data` 工具从音频文件生成测试数据：

```bash
# 基本用法
cargo run --bin collect_test_data -- <input.wav> <output.json> [bitrate]

# 示例
cargo run --bin collect_test_data -- testing/fixtures/audio/test_input.wav testing/fixtures/data/test_data.json 128
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/sample_3s_test_data.json 192
```

**参数说明：**
- `input.wav` - 输入WAV文件路径
- `output.json` - 输出JSON测试数据文件路径  
- `bitrate` - MP3比特率（可选，默认128 kbps）

### 2. 验证测试数据

使用 `validate_test_data` 工具验证编码实现：

```bash
# 验证测试用例
cargo run --bin validate_test_data -- <test_data.json>

# 示例
cargo run --bin validate_test_data -- testing/fixtures/data/test_data.json
cargo run --bin validate_test_data -- testing/fixtures/data/sample_3s_test_data.json
```

## JSON数据格式

测试数据以JSON格式存储，包含以下结构：

### 元数据 (metadata)
```json
{
  "name": "test_case_test_input_44100hz_2ch_128kbps",
  "input_file": "testing/fixtures/audio/test_input.wav",
  "expected_output_size": 1252,
  "expected_hash": "861b8689d7eee5d408feec61cfa6ce6932168e35e6f86fa92bc5f3c77eb37c32",
  "created_at": "2026-01-25T03:20:58.540642300+00:00",
  "description": "Test case for test_input.wav at 128 kbps"
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

### 1. 回归测试
确保代码修改不会破坏现有功能：

```bash
# 收集基准数据
cargo run --bin collect_test_data -- baseline.wav baseline_test.json 128

# 修改代码后验证
cargo run --bin validate_test_data -- baseline_test.json
```

### 2. 多参数测试
测试不同编码参数的组合：

```bash
# 不同比特率
cargo run --bin collect_test_data -- audio.wav test_128k.json 128
cargo run --bin collect_test_data -- audio.wav test_192k.json 192
cargo run --bin collect_test_data -- audio.wav test_256k.json 256

# 验证所有配置
cargo run --bin validate_test_data -- test_128k.json
cargo run --bin validate_test_data -- test_192k.json
cargo run --bin validate_test_data -- test_256k.json
```

### 3. 跨平台验证
确保不同平台生成相同结果：

```bash
# 在平台A收集数据
cargo run --bin collect_test_data -- test.wav platform_a_test.json

# 在平台B验证
cargo run --bin validate_test_data -- platform_a_test.json
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

## 调试模式

测试数据收集只在debug模式下工作：

```bash
# Debug模式 - 收集数据并显示详细输出
cargo run --bin collect_test_data -- input.wav output.json

# Release模式 - 正常编码，无数据收集
cargo run --release --bin wav2mp3 -- input.wav output.mp3
```

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

### 调试技巧

1. **启用详细输出**：使用debug构建查看详细的编码参数
2. **分步验证**：先验证结构，再验证数值
3. **对比工具**：使用JSON diff工具比较不同版本的测试数据

## 最佳实践

1. **版本控制**：将测试数据JSON文件纳入版本控制
2. **命名规范**：使用描述性的文件名（如 `sample_44k_stereo_128k.json`）
3. **定期更新**：在重大算法改进后更新基准测试数据
4. **文档记录**：为每个测试用例添加清晰的描述
5. **自动化**：集成到CI/CD流程中进行自动验证

## 示例测试用例

项目包含以下预定义测试用例：

1. **testing/fixtures/data/test_data.json** - 基础测试用例（44.1kHz, 立体声, 128kbps）
2. **testing/fixtures/data/sample_3s_test_data.json** - 长音频测试用例（3秒音频）

这些测试用例覆盖了常见的编码场景，可以作为回归测试的基础。