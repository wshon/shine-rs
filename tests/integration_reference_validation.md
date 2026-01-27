# integration_reference_validation.rs 测试文档

## 测试概述

这个测试套件使用预生成的参考文件验证Rust MP3编码器与Shine参考实现的一致性。它提供了最大的可靠性和可重现性，通过SHA256哈希值验证确保输出的完全一致性。

## 测试目标

- **参考文件验证**: 使用预保存的Shine输出作为黄金标准
- **哈希值验证**: 通过SHA256确保二进制完全一致
- **多配置覆盖**: 测试不同帧数、音频格式和编码场景
- **可重现性**: 消除对外部工具的依赖，确保测试结果稳定

## 核心数据结构

### ReferenceConfig
```rust
struct ReferenceConfig {
    description: String,     // 配置描述
    file_path: String,      // 参考文件路径
    size_bytes: u64,        // 预期文件大小
    sha256: String,         // 预期SHA256哈希
    input_file: String,     // 输入音频文件
    frame_limit: Option<u32>, // 帧数限制
}
```

## 测试函数详解

### `test_sample_file_configurations()`
**目的**: 测试sample-3s.wav文件的各种配置

**运行方式**:
```bash
cargo test test_sample_file_configurations --test integration_reference_validation -- --nocapture
```

**测试配置**:
- 1frame, 2frames, 3frames, 6frames
- 10frames, 15frames, 20frames
- 所有配置使用44.1kHz立体声

**验证内容**:
- 文件大小匹配
- SHA256哈希匹配
- 编码成功完成

**预期结果**: 100%通过率（所有配置应产生相同输出）

### `test_large_file_configurations()`
**目的**: 测试大文件的编码配置

**运行方式**:
```bash
cargo test test_large_file_configurations --test integration_reference_validation -- --nocapture
```

**测试配置**:
- large_3frames, large_6frames
- 使用Free_Test_Data_500KB_WAV.wav

**验证内容**:
- 大文件处理能力
- 内存使用合理性
- 输出一致性

### `test_voice_file_configurations()`
**目的**: 测试语音文件配置（已知问题测试）

**运行方式**:
```bash
cargo test test_voice_file_configurations --test integration_reference_validation -- --nocapture
```

**测试配置**:
- voice_3frames, voice_6frames
- 使用voice-recorder-testing-1-2-3-sound-file.wav（单声道48kHz）

**特殊处理**:
- 不会因失败而中断测试
- 记录失败但继续执行
- 已知单声道48kHz处理差异

### `test_all_passing_configurations()`
**目的**: 运行所有预期通过的配置

**运行方式**:
```bash
cargo test test_all_passing_configurations --test integration_reference_validation -- --nocapture
```

**验证内容**:
- 综合测试所有配置
- 生成详细的结果报告
- 按文件类型分析结果
- 计算总体成功率

**输出格式**:
```
📊 Summary: X passed, Y failed (Z% success rate)
📈 Results by file type:
   Sample file:  A/B identical (C%)
   Voice file:   D/E identical (F%)
   Large file:   G/H identical (I%)
```

### `test_frame_limit_functionality()`
**目的**: 验证帧数限制功能

**运行方式**:
```bash
cargo test test_frame_limit_functionality --test integration_reference_validation -- --nocapture
```

**测试场景**:
- 不同帧数限制（1, 2, 3, 6帧）
- 验证输出文件大小
- 确认环境变量生效

**预期文件大小**:
- 1帧: 416字节
- 2帧: 836字节
- 3帧: 1252字节
- 6帧: 2508字节

### `test_reference_file_integrity()`
**目的**: 验证参考文件的完整性

**运行方式**:
```bash
cargo test test_reference_file_integrity --test integration_reference_validation -- --nocapture
```

**验证内容**:
- 所有参考文件存在
- 文件大小正确
- SHA256哈希正确
- 清单文件格式有效

**故障处理**: 如果参考文件损坏，提示重新生成

### `test_encoding_performance()`
**目的**: 性能基准测试（默认忽略）

**运行方式**:
```bash
cargo test test_encoding_performance --test integration_reference_validation -- --nocapture --ignored
```

**测试内容**:
- 不同帧数的编码时间
- 计算每秒帧数（FPS）
- 性能回归检测

## 辅助函数

### `load_reference_manifest()`
**功能**: 从JSON清单文件加载参考配置

**依赖文件**: `tests/audio/reference_manifest.json`

**清单格式**:
```json
{
  "reference_files": {
    "config_name": {
      "description": "配置描述",
      "file_path": "参考文件路径",
      "size_bytes": 文件大小,
      "sha256": "SHA256哈希值"
    }
  }
}
```

### `calculate_sha256()`
**功能**: 计算文件的SHA256哈希值

### `run_rust_encoder()`
**功能**: 运行Rust编码器生成输出

**参数**:
- 输入文件路径
- 输出文件路径
- 帧数限制（可选）

### `validate_reference_config()`
**功能**: 验证单个参考配置

**验证步骤**:
1. 检查参考文件存在
2. 运行Rust编码器
3. 比较文件大小
4. 比较SHA256哈希
5. 清理临时文件

## 属性测试

### `test_frame_limit_bounds()`
**目的**: 测试帧数限制的边界条件

**测试范围**: 1-100帧
**验证**: 编码器不会崩溃或异常

### `test_hash_consistency()`
**目的**: 验证哈希计算的一致性

**验证**: 相同内容总是产生相同哈希

## 运行测试

### 运行所有参考验证测试
```bash
cargo test --test integration_reference_validation -- --nocapture
```

### 运行特定测试类别
```bash
# 样本文件测试
cargo test test_sample_file_configurations --test integration_reference_validation -- --nocapture

# 大文件测试
cargo test test_large_file_configurations --test integration_reference_validation -- --nocapture

# 语音文件测试（预期部分失败）
cargo test test_voice_file_configurations --test integration_reference_validation -- --nocapture

# 综合测试
cargo test test_all_passing_configurations --test integration_reference_validation -- --nocapture
```

### 运行性能测试
```bash
cargo test test_encoding_performance --test integration_reference_validation -- --nocapture --ignored
```

## 故障排除

### 常见问题

#### 1. 参考清单文件缺失
**症状**: "Reference manifest not found"
**解决**:
```bash
python scripts/generate_reference_files.py
```

#### 2. 参考文件损坏
**症状**: "Reference file hash mismatch - file may be corrupted"
**解决**: 重新生成参考文件
```bash
cd ref/shine
.\build.ps1
cd ../..
python scripts/generate_reference_files.py
```

#### 3. 输入音频文件缺失
**症状**: "Input file not found"
**解决**: 确保以下文件存在：
- `tests/audio/sample-3s.wav`
- `tests/audio/voice-recorder-testing-1-2-3-sound-file.wav`
- `tests/audio/Free_Test_Data_500KB_WAV.wav`

#### 4. 哈希值不匹配
**症状**: "Hash mismatch: Rust: xxx, Expected: yyy"
**原因**: Rust实现与Shine输出不一致
**调试步骤**:
1. 检查编码参数是否正确
2. 验证算法实现与Shine一致
3. 查看具体的差异点

#### 5. 文件大小不匹配
**症状**: "Size mismatch: Rust=X bytes, Expected=Y bytes"
**原因**: 
- 帧数限制未正确应用
- 编码参数不匹配
- 算法实现差异

### 调试技巧

#### 1. 启用详细输出
```bash
cargo test --test integration_reference_validation -- --nocapture
```

#### 2. 检查参考文件完整性
```bash
cargo test test_reference_file_integrity --test integration_reference_validation -- --nocapture
```

#### 3. 验证帧数限制功能
```bash
cargo test test_frame_limit_functionality --test integration_reference_validation -- --nocapture
```

#### 4. 单独测试特定配置
```bash
# 修改测试代码，只测试特定配置
let test_configs = ["3frames"]; // 只测试3帧配置
```

## 参考文件管理

### 生成新参考文件
```bash
# 1. 确保Shine编码器可用
cd ref/shine
.\build.ps1

# 2. 生成参考文件和清单
cd ../..
python scripts/generate_reference_files.py
```

### 验证参考文件
```bash
# 检查参考文件完整性
cargo test test_reference_file_integrity --test integration_reference_validation -- --nocapture
```

### 更新参考文件
当需要更新参考文件时：
1. 备份现有参考文件
2. 重新生成参考文件
3. 运行完整性检查
4. 更新测试中的预期值

## 性能基准

### 编码性能
- **小文件** (3帧): < 0.1秒
- **中等文件** (6帧): < 0.2秒
- **大文件** (20帧): < 1秒

### 测试执行时间
- **单个配置**: < 2秒
- **完整测试套件**: < 30秒
- **性能测试**: < 10秒

## 已知问题

### 1. 语音文件差异
- **状态**: voice文件测试预期失败
- **原因**: 单声道48kHz处理算法差异
- **影响**: 不影响核心功能，已在测试中标记

### 2. 参考文件依赖
- **状态**: 依赖预生成的参考文件
- **风险**: 文件损坏或丢失会导致测试失败
- **缓解**: 提供重新生成脚本和完整性检查

### 3. 平台差异
- **状态**: 可能存在平台相关的编码差异
- **缓解**: 使用固定的参考文件和严格的哈希验证

## 维护指南

### 添加新配置
1. 在`generate_reference_files.py`中添加新配置
2. 重新生成参考文件和清单
3. 更新测试代码以包含新配置
4. 验证新配置的测试结果

### 更新音频文件
1. 添加新的音频文件到`tests/audio/`
2. 更新`get_input_file_from_config()`函数
3. 生成对应的参考文件
4. 添加相应的测试配置

### 性能优化
1. 监控测试执行时间
2. 优化文件I/O操作
3. 考虑并行化独立测试
4. 减少不必要的文件操作

## 成功标准

- **参考文件完整性**: 所有参考文件哈希验证通过
- **样本文件测试**: 100%通过率
- **大文件测试**: 100%通过率
- **语音文件测试**: 记录差异但不中断测试
- **性能达标**: 编码时间在可接受范围内
- **可重现性**: 多次运行产生相同结果