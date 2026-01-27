# integration_pipeline_validation.rs 测试文档

## 测试概述

这个测试套件执行数据驱动的MP3编码器集成测试，通过将编码器输出与参考数据进行比较来验证实际编码功能。它集成了validate_test_data工具的功能，进行包括输出哈希验证在内的全面端到端验证。

## 测试目标

- **完整编码管道验证**: 验证从WAV输入到MP3输出的完整流程
- **算法一致性验证**: 确保MDCT、量化、比特流等各个算法模块与Shine参考实现一致
- **数据驱动测试**: 自动发现并测试所有JSON测试数据文件
- **性能监控**: 确保编码验证在合理时间内完成

## 测试文件结构

### 依赖的测试数据
- **数据目录**: `tests/integration_pipeline_validation.data/`
- **数据格式**: JSON文件，包含帧级别的参考数据
- **音频文件**: `tests/audio/` 目录下的WAV文件

### 环境变量
- `RUST_MP3_DEBUG_FRAMES`: 设置调试帧数限制，自动根据测试数据调整

## 测试函数详解

### 1. `test_complete_encoding_pipeline()`
**目的**: 测试完整的编码管道，包括输出哈希验证

**运行方式**:
```bash
cargo test test_complete_encoding_pipeline --features diagnostics -- --nocapture
```

**验证内容**:
- WAV文件读取和配置匹配
- 完整的MP3编码流程
- 输出文件大小验证
- SHA256哈希值验证

**注意事项**:
- 需要`diagnostics`特性
- 当前为非致命错误模式（记录错误但不中断测试）
- 实现稳定后可启用严格模式

### 2. `test_encoding_validation_all_files()`
**目的**: 对所有发现的测试数据文件执行编码验证

**运行方式**:
```bash
cargo test test_encoding_validation_all_files --features diagnostics -- --nocapture
```

**验证内容**:
- 自动发现所有JSON测试文件
- 逐帧验证MDCT系数、量化参数、比特流数据
- 与参考数据的精确比较

**失败处理**: 任何文件验证失败都会导致测试中断

### 3. `test_mdct_encoding_consistency()`
**目的**: 专门测试MDCT算法的一致性

**运行方式**:
```bash
cargo test test_mdct_encoding_consistency --features diagnostics -- --nocapture
```

**验证内容**:
- 混叠减少前的MDCT系数
- 混叠减少后的MDCT系数
- l3_sb_sample数据
- 允许±1的整数差异容忍度

### 4. `test_quantization_encoding_consistency()`
**目的**: 测试量化算法的一致性

**运行方式**:
```bash
cargo test test_quantization_encoding_consistency --features diagnostics -- --nocapture
```

**验证内容**:
- global_gain（全局增益）
- part2_3_length（部分2+3长度）
- max_bits（最大比特数）
- xrmax（最大频谱值，允许±1差异）

### 5. `test_bitstream_encoding_consistency()`
**目的**: 测试比特流输出的一致性

**运行方式**:
```bash
cargo test test_bitstream_encoding_consistency --features diagnostics -- --nocapture
```

**验证内容**:
- written（写入字节数）
- bits_per_frame（每帧比特数）
- slot_lag（时隙延迟，允许1e-6容忍度）
- padding（填充标志）

### 6. `test_encoding_config_validation()`
**目的**: 验证编码配置的有效性

**运行方式**:
```bash
cargo test test_encoding_config_validation --features diagnostics -- --nocapture
```

**验证内容**:
- 比特率有效性（32-320 kbps范围内的标准值）
- 采样率有效性（32000, 44100, 48000 Hz）
- 声道数有效性（1或2）
- MPEG版本（必须为3，即MPEG-I）

### 7. `test_test_data_coverage()`
**目的**: 确保测试数据覆盖不同的编码场景

**运行方式**:
```bash
cargo test test_test_data_coverage --features diagnostics -- --nocapture
```

**验证内容**:
- 多种比特率覆盖
- 不同音频文件覆盖
- 单声道/立体声配置覆盖

### 8. `test_encoding_validation_performance()`
**目的**: 监控编码验证的性能

**运行方式**:
```bash
cargo test test_encoding_validation_performance --features diagnostics -- --nocapture
```

**验证内容**:
- 单个文件验证时间应小于10秒
- 性能回归检测

## 运行所有测试

```bash
# 运行所有管道验证测试
cargo test --test integration_pipeline_validation --features diagnostics -- --nocapture

# 运行特定测试
cargo test test_complete_encoding_pipeline --test integration_pipeline_validation --features diagnostics -- --nocapture
```

## 故障排除

### 常见问题

#### 1. 测试数据文件未找到
**症状**: "Should find at least one test data file"
**解决**: 
```bash
# 生成测试数据
python scripts/generate_reference_data.py
```

#### 2. 音频文件缺失
**症状**: "audio file not found"
**解决**: 确保`tests/audio/`目录包含所需的WAV文件

#### 3. MDCT系数不匹配
**症状**: "MDCT coefficient mismatch"
**原因**: MDCT算法实现与Shine不一致
**解决**: 
1. 查看`ref/shine/src/lib/l3mdct.c`
2. 对比Rust实现中的MDCT算法
3. 确保混叠减少前后的系数计算完全一致

#### 4. 量化参数不匹配
**症状**: "Global gain mismatch" 或 "Part2_3_length mismatch"
**原因**: 量化循环或reservoir调整实现问题
**解决**:
1. 查看`ref/shine/src/lib/l3loop.c`
2. 确保数据收集在正确时机（reservoir调整后）
3. 验证全局增益计算公式

#### 5. 编译特性缺失
**症状**: "diagnostics_data module not found"
**解决**: 确保使用`--features diagnostics`标志

## 测试数据要求

### JSON文件格式
```json
{
  "metadata": {
    "name": "test_name",
    "input_file": "audio_file.wav",
    "expected_output_size": 1234,
    "expected_hash": "sha256_hash",
    "created_at": "2026-01-27T...",
    "description": "Test description"
  },
  "config": {
    "bitrate": 128,
    "sample_rate": 44100,
    "channels": 2,
    "stereo_mode": 1,
    "mpeg_version": 3
  },
  "frames": [
    {
      "frame_number": 1,
      "mdct_coefficients": { ... },
      "quantization": { ... },
      "bitstream": { ... }
    }
  ]
}
```

### 容忍度设置
- **MDCT系数**: ±1整数差异
- **量化参数**: 精确匹配（除xrmax允许±1）
- **比特流参数**: 精确匹配（除slot_lag允许1e-6）

## 性能基准

- **单文件验证**: < 10秒
- **完整测试套件**: 取决于测试文件数量
- **内存使用**: 应保持在合理范围内

## 已知问题

### 1. 完整管道验证当前为非致命模式
- **状态**: 记录错误但不中断测试
- **原因**: 实现仍在完善中
- **计划**: 实现稳定后启用严格模式

### 2. 单声道48kHz文件可能有差异
- **状态**: 已知问题
- **原因**: 单声道处理算法差异
- **影响**: 不影响核心功能正确性

### 3. 测试数据依赖外部生成
- **状态**: 需要Python脚本生成
- **依赖**: Shine编码器和Python环境
- **改进**: 考虑内置测试数据生成

## 维护指南

### 添加新测试场景
1. 在`tests/integration_pipeline_validation.data/`添加新的JSON文件
2. 确保对应的音频文件存在于`tests/audio/`
3. 运行测试验证新场景

### 更新容忍度
1. 修改验证函数中的容忍度常量
2. 确保修改有充分理由（算法精度要求）
3. 更新文档说明

### 性能优化
1. 监控`test_encoding_validation_performance`结果
2. 识别性能瓶颈
3. 在保证正确性前提下优化

## 成功标准

- **所有测试通过**: 表示实现与Shine高度一致
- **MDCT一致性**: 混叠减少前后系数完全匹配
- **量化一致性**: 全局增益和比特分配完全匹配
- **比特流一致性**: 输出字节和参数完全匹配
- **性能达标**: 验证时间在可接受范围内