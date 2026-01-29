# 测试数据框架文档

## 概述

本项目实现了一个完整的测试数据框架，用于验证MP3编码器的正确性。框架包含数据收集、验证和自动化测试三个主要组件。

## 框架结构

### 1. 测试数据收集 (`collect_test_data`)

**功能**: 从实际编码过程中收集关键参数数据
**位置**: `src/bin/collect_test_data.rs`
**输出**: JSON格式的测试数据文件

**使用方法**:
```bash
cargo run --bin collect_test_data -- input.wav output.json [--max-frames N] [--bitrate RATE]
```

**收集的数据**:
- 编码配置参数（比特率、采样率、声道数等）
- 每帧的MDCT系数
- 量化参数（global_gain、big_values、part2_3_length等）
- SCFSI值
- 比特流参数（写入字节数、slot lag等）

### 2. 参考验证系统

**功能**: 使用预生成的参考文件验证编码器输出
**位置**: `tests/encoder_validation_cicd.rs`
**数据**: `tests/audio/inputs/reference_manifest.json`

**使用方法**:
```bash
cargo test encoder_validation_cicd
```

**验证内容**:
- 最终MP3文件完整性
- 文件大小和SHA256哈希匹配
- SCFSI计算正确性
- 比特流参数准确性

### 3. 自动化测试脚本

**功能**: 批量生成和验证测试数据
**位置**: `scripts/run_test_suite.ps1`

**使用方法**:
```powershell
.\scripts\run_test_suite.ps1
```

## 测试数据文件

### 当前测试数据

所有测试数据文件位于 `testing/fixtures/data/` 目录：

1. **sample-3s_128k_3f.json** - 主要测试文件
   - 音频: sample-3s.wav (3秒立体声)
   - 配置: 128kbps, 44100Hz, 立体声
   - 帧数: 6帧

2. **sample-3s_192k_6f.json** - 高比特率测试
   - 音频: sample-3s.wav
   - 配置: 192kbps, 44100Hz, 立体声
   - 帧数: 6帧

3. **free_test_data_128k_6f.json** - 长音频测试
   - 音频: Free_Test_Data_500KB_WAV.wav
   - 配置: 128kbps, 44100Hz, 立体声
   - 帧数: 6帧

4. **voice_recorder_128k_6f.json** - 单声道测试
   - 音频: voice-recorder-testing-1-2-3-sound-file.wav
   - 配置: 128kbps, 44100Hz, 单声道
   - 帧数: 6帧

### 数据文件格式

```json
{
  "audio_file": "testing/fixtures/audio/sample-3s.wav",
  "encoding_config": {
    "bitrate": 128,
    "sample_rate": 44100,
    "channels": 2,
    "mpeg_version": 3,
    "layer": 1
  },
  "frames": [
    {
      "frame_number": 1,
      "mdct_data": [[[...]], [[...]]],  // [channel][subband][sample]
      "quantization_data": [[[...]], [[...]]], // [channel][granule][params]
      "scfsi_data": [[...], [...]],    // [channel][band]
      "bitstream_data": {
        "written_bytes": 416,
        "bits_per_frame": 3344,
        "padding": 1,
        "slot_lag_before": -0.959184,
        "slot_lag_after": -0.918367
      }
    }
  ]
}
```

## 测试组织结构

### 单元测试 (`src/tests/`)

每个模块都有对应的单元测试文件：

- **`bitstream_tests.rs`** - 比特流操作测试
- **`mdct_tests.rs`** - MDCT变换测试
- **`quantization_tests.rs`** - 量化算法测试
- **`scfsi_tests.rs`** - SCFSI计算测试
- **`subband_tests.rs`** - 子带滤波器测试

每个单元测试模块包含：
- 基础功能测试
- 边界条件测试
- 属性测试（使用proptest）
- 真实数据验证测试

### 集成测试 (`testing/integration/`)

- **`integration_data_driven_validation.rs`** - 数据驱动的集成测试
  - 自动发现所有JSON测试数据文件
  - 验证数据文件格式和内容
  - 跨文件一致性检查
  - 覆盖率分析

- **`integration_full_pipeline_validation.rs`** - 完整流水线集成测试
  - 端到端编码流程测试
  - 组件间协作验证
  - 性能特征测试

- **`integration_scfsi_consistency.rs`** - SCFSI一致性集成测试
  - 与Shine参考实现对比
  - SCFSI算法完整性验证

## 使用指南

### 1. 生成新的测试数据

```bash
# 为新音频文件生成测试数据
cargo run --bin collect_test_data -- testing/fixtures/audio/new_audio.wav testing/fixtures/data/new_audio_128k_6f.json --bitrate 128 --max-frames 6
```

### 2. 验证现有实现

```bash
# 运行参考验证测试
cargo test encoder_validation_cicd

# 运行所有单元测试
cargo test --lib

# 运行所有集成测试
cargo test --test '*'

# 运行数据驱动测试
cargo test --test integration_data_driven_validation
```

### 3. 添加新的测试场景

1. 将新音频文件放入 `testing/fixtures/audio/`
2. 使用 `collect_test_data` 生成测试数据
3. 数据驱动测试会自动发现并验证新文件

### 4. 调试测试失败

```bash
# 运行特定测试并显示详细输出
cargo test test_name -- --nocapture

# 运行属性测试时显示失败案例
PROPTEST_VERBOSE=1 cargo test
```

## 测试覆盖范围

### 音频特征覆盖
- ✅ 立体声音频 (sample-3s.wav, Free_Test_Data_500KB_WAV.wav)
- ✅ 单声道音频 (voice-recorder-testing-1-2-3-sound-file.wav)
- ✅ 不同时长 (3秒短音频, 长音频)
- ✅ 不同音频内容 (音乐, 语音)

### 编码参数覆盖
- ✅ 多种比特率 (128kbps, 192kbps)
- ✅ 标准采样率 (44100Hz)
- ✅ 单声道和立体声
- ✅ MPEG-I Layer III

### 算法组件覆盖
- ✅ 子带分析滤波器
- ✅ MDCT变换
- ✅ 量化循环
- ✅ SCFSI计算
- ✅ Huffman编码
- ✅ 比特流生成

### 验证类型覆盖
- ✅ 数值精度验证
- ✅ 边界条件测试
- ✅ 属性测试
- ✅ 一致性检查
- ✅ 性能测试框架

## 维护指南

### 添加新的测试数据字段

1. 更新 `src/test_data.rs` 中的数据结构
2. 修改 `collect_test_data` 工具收集新字段
3. 更新 `validate_test_data` 工具验证新字段
4. 添加相应的单元测试

### 扩展验证逻辑

1. 在相应的单元测试模块中添加新测试
2. 更新集成测试以包含新的验证场景
3. 确保属性测试覆盖新的边界条件

### 性能优化

1. 使用 `#[ignore]` 标记耗时测试
2. 调整proptest的案例数量
3. 考虑并行测试执行

## 最佳实践

1. **数据驱动**: 优先使用真实音频数据而非合成数据
2. **自动化**: 所有测试应该能够自动运行和验证
3. **覆盖全面**: 确保测试覆盖所有关键算法路径
4. **快速反馈**: 单元测试应该快速执行
5. **可重现**: 测试结果应该在不同环境下一致
6. **文档化**: 每个测试都应该有清晰的目的说明

## 故障排除

### 常见问题

1. **测试数据文件不存在**
   - 确保运行了 `collect_test_data` 生成数据文件
   - 检查文件路径是否正确

2. **数值精度不匹配**
   - 检查是否使用了相同的编译优化级别
   - 验证浮点数计算的一致性

3. **属性测试失败**
   - 检查测试的假设是否正确
   - 调整测试参数范围

4. **集成测试超时**
   - 减少测试数据大小
   - 使用 `#[ignore]` 标记长时间运行的测试

### 调试技巧

1. 使用 `--nocapture` 查看测试输出
2. 使用 `RUST_LOG=debug` 启用调试日志
3. 使用 `PROPTEST_VERBOSE=1` 查看属性测试详情
4. 使用 `cargo test -- --test-threads=1` 串行运行测试