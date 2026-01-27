# mp3_encoder_tests.rs 测试文档

## 测试概述

这个测试套件包含对高级MP3编码器API的全面测试，涵盖配置验证、编码功能和错误处理。它测试用户友好的高级接口，而不是底层的Shine API。

## 测试目标

- **配置验证**: 确保编码器配置的有效性检查
- **编码功能**: 验证各种编码场景的正确性
- **错误处理**: 测试错误情况的正确处理
- **API易用性**: 确保高级API的用户友好性

## 测试模块结构

### 1. unit_tests 模块
**目的**: 测试单个组件和配置验证逻辑

### 2. integration_tests 模块  
**目的**: 测试完整的编码工作流程

### 3. error_handling_tests 模块
**目的**: 测试各种错误情况的处理

### 4. property_tests 模块
**目的**: 使用属性测试验证编码器行为

## 详细测试函数

### Unit Tests

#### `test_config_validation_valid()`
**目的**: 验证有效配置通过验证

**运行方式**:
```bash
cargo test test_config_validation_valid --test mp3_encoder_tests -- --nocapture
```

**验证内容**:
- 标准配置（44100Hz, 128kbps, 立体声）应该通过验证

#### `test_config_validation_invalid_*()` 系列
**目的**: 验证无效配置被正确拒绝

**测试场景**:
- 不支持的采样率（12345Hz）
- 不支持的比特率（999kbps）
- 无效的声道数（0或3）

#### `test_config_validation_incompatible_combinations()`
**目的**: 验证不兼容的采样率/比特率组合被拒绝

**测试场景**:
- MPEG-2.5 + 高比特率（8000Hz + 128kbps）
- MPEG-2 + 超高比特率（22050Hz + 320kbps）
- MPEG-1 + 超低比特率（44100Hz + 16kbps）

**预期错误**: `ConfigError::IncompatibleRateCombination`

#### `test_config_validation_valid_combinations()`
**目的**: 验证有效的MPEG版本和比特率组合

**有效组合**:
- MPEG-2.5: 8000Hz/32kbps, 11025Hz/64kbps
- MPEG-2: 16000Hz/80kbps, 22050Hz/160kbps
- MPEG-1: 32000Hz/32kbps, 44100Hz/320kbps

#### `test_supported_sample_rates()` 和 `test_supported_bitrates()`
**目的**: 验证所有声明支持的采样率和比特率确实可用

**验证方法**: 使用`encoder::shine_check_config()`检查Shine支持情况

#### `test_samples_per_frame_*()` 系列
**目的**: 验证每帧样本数计算正确

**预期值**:
- MPEG-1立体声: 2304样本（2颗粒 × 576样本 × 2声道）
- MPEG-2立体声: 1152样本（1颗粒 × 576样本 × 2声道）
- MPEG-1单声道: 1152样本（2颗粒 × 576样本 × 1声道）

### Integration Tests

#### `test_simple_encoding_stereo()`
**目的**: 测试基本的立体声编码功能

**运行方式**:
```bash
cargo test test_simple_encoding_stereo --test mp3_encoder_tests -- --nocapture
```

**测试流程**:
1. 创建44100Hz/128kbps/立体声配置
2. 生成正弦波测试数据（4608样本，2帧）
3. 执行编码并验证输出

#### `test_simple_encoding_mono()`
**目的**: 测试基本的单声道编码功能

**配置**: 22050Hz/64kbps/单声道
**测试数据**: 2304样本（2帧单声道数据）

#### `test_batch_encoding()`
**目的**: 测试批量编码功能（一次性编码完整音频）

**测试场景**: 1秒22050Hz单声道音频的完整编码

#### `test_separate_channels_*()` 系列
**目的**: 测试分离声道编码功能

**立体声测试**:
- 左声道: 440Hz正弦波
- 右声道: 880Hz正弦波

**单声道测试**:
- 单声道: 440Hz正弦波

#### `test_streaming_encoding()`
**目的**: 测试流式编码功能

**测试流程**:
1. 将音频分成5个块
2. 逐块编码
3. 收集所有输出
4. 验证总输出合理

### Error Handling Tests

#### `test_empty_input_error()`
**目的**: 验证空输入的错误处理

**预期错误**: `EncoderError::InputData(InputDataError::EmptyInput)`

#### `test_channel_count_mismatch_error()`
**目的**: 验证声道数不匹配的错误处理

**测试场景**: 立体声配置但提供不同长度的左右声道数据

#### `test_mono_with_two_channels_error()`
**目的**: 验证单声道配置但提供双声道数据的错误处理

#### `test_stereo_with_one_channel_error()`
**目的**: 验证立体声配置但只提供单声道数据的错误处理

#### `test_finished_encoder_error()`
**目的**: 验证已完成的编码器不能继续编码

**测试流程**:
1. 调用`finish()`完成编码
2. 尝试继续编码
3. 验证返回`EncoderError::InternalState`

#### `test_double_finish()`
**目的**: 验证多次调用`finish()`的行为

**预期行为**: 第二次调用应返回空数据而不是错误

### Property Tests

#### `test_config_validation_properties()`
**目的**: 使用属性测试验证配置验证逻辑

**测试范围**:
- 所有支持的采样率
- 所有支持的比特率
- 1-2声道

**验证逻辑**: 只有Shine支持的组合应该通过验证

#### `test_encoder_creation_properties()`
**目的**: 验证编码器创建的属性

**验证**: 有效配置应该成功创建编码器

#### `test_small_data_encoding()`
**目的**: 测试小数据块的编码行为

**测试范围**:
- 采样率: 22050Hz, 44100Hz
- 数据大小: 100-1000样本

**验证**: 编码过程不应崩溃

## 运行测试

### 运行所有测试
```bash
cargo test --test mp3_encoder_tests -- --nocapture
```

### 运行特定模块
```bash
# 单元测试
cargo test unit_tests --test mp3_encoder_tests -- --nocapture

# 集成测试
cargo test integration_tests --test mp3_encoder_tests -- --nocapture

# 错误处理测试
cargo test error_handling_tests --test mp3_encoder_tests -- --nocapture

# 属性测试
cargo test property_tests --test mp3_encoder_tests -- --nocapture
```

### 运行特定测试
```bash
cargo test test_simple_encoding_stereo --test mp3_encoder_tests -- --nocapture
```

## 故障排除

### 常见问题

#### 1. 配置验证失败
**症状**: `ConfigError::IncompatibleRateCombination`
**原因**: 采样率和比特率组合不被Shine支持
**解决**: 检查MPEG版本限制：
- MPEG-2.5: 最高64kbps
- MPEG-2: 最高160kbps
- MPEG-1: 32-320kbps

#### 2. 编码器创建失败
**症状**: `EncoderError::Config`
**原因**: 配置无效或不支持
**解决**: 
1. 验证采样率在支持列表中
2. 验证比特率在支持列表中
3. 检查声道数（1或2）

#### 3. 输入数据错误
**症状**: `InputDataError::EmptyInput` 或 `InputDataError::InvalidChannelCount`
**原因**: 输入数据格式不正确
**解决**:
1. 确保输入数据非空
2. 立体声需要偶数个样本（交错格式）
3. 分离声道模式下左右声道长度必须相同

#### 4. 编码器状态错误
**症状**: `EncoderError::InternalState`
**原因**: 在编码器完成后尝试继续编码
**解决**: 创建新的编码器实例

### 调试技巧

#### 1. 启用详细输出
```bash
cargo test --test mp3_encoder_tests -- --nocapture
```

#### 2. 检查Shine配置支持
```rust
use shine_rs::encoder;
let supported = encoder::shine_check_config(sample_rate, bitrate);
println!("Config support: {}", supported); // >= 0 表示支持
```

#### 3. 验证输入数据格式
```rust
// 立体声交错格式: [L0, R0, L1, R1, ...]
assert_eq!(stereo_data.len() % 2, 0);

// 分离声道格式
assert_eq!(left_channel.len(), right_channel.len());
```

## 测试数据生成

### 正弦波生成
```rust
fn generate_sine_wave(frequency: f32, sample_rate: u32, duration_samples: usize) -> Vec<i16> {
    (0..duration_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (frequency * 2.0 * std::f32::consts::PI * t).sin() * 16384.0) as i16
        })
        .collect()
}
```

### 立体声交错数据
```rust
fn generate_stereo_interleaved(left_freq: f32, right_freq: f32, sample_rate: u32, samples_per_channel: usize) -> Vec<i16> {
    let mut data = Vec::new();
    for i in 0..samples_per_channel {
        let t = i as f32 / sample_rate as f32;
        let left = (left_freq * 2.0 * std::f32::consts::PI * t).sin() * 16384.0) as i16;
        let right = (right_freq * 2.0 * std::f32::consts::PI * t).sin() * 16384.0) as i16;
        data.push(left);
        data.push(right);
    }
    data
}
```

## 性能考虑

### 测试执行时间
- 单元测试: < 1秒
- 集成测试: < 5秒
- 属性测试: < 10秒（50个案例）

### 内存使用
- 测试数据大小控制在合理范围内
- 避免生成过大的音频数据
- 及时释放编码器资源

## 已知问题

### 1. Shine配置检查依赖
- **状态**: 依赖`encoder::shine_check_config()`
- **影响**: 测试结果依赖Shine实现
- **改进**: 考虑独立的配置验证逻辑

### 2. 属性测试覆盖范围
- **状态**: 当前覆盖基本场景
- **改进**: 增加边界条件和异常情况测试

### 3. 错误消息国际化
- **状态**: 错误消息为英文
- **改进**: 考虑支持多语言错误消息

## 维护指南

### 添加新测试
1. 确定测试类别（单元/集成/错误处理/属性）
2. 选择合适的测试模块
3. 遵循现有的命名约定
4. 添加适当的文档注释

### 更新配置支持
1. 更新`SUPPORTED_SAMPLE_RATES`和`SUPPORTED_BITRATES`常量
2. 更新相关的验证测试
3. 确保属性测试覆盖新配置

### 性能优化
1. 监控测试执行时间
2. 优化测试数据生成
3. 考虑并行化独立测试

## 成功标准

- **所有单元测试通过**: 配置验证逻辑正确
- **集成测试成功**: 编码功能正常工作
- **错误处理完善**: 所有错误情况得到正确处理
- **属性测试稳定**: 随机输入不会导致崩溃
- **API易用性**: 高级API提供良好的用户体验