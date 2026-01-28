# 测试架构文档

本项目采用分层测试架构，将测试分为5个清晰的类别，避免重复并提供全面的验证覆盖。

## 测试分类

### 1. 基本功能测试 (`encoder_basic_functionality.rs`)
**目的**: 验证编码器的核心功能，不依赖复杂的API假设
**特点**:
- 使用CLI接口进行测试
- 验证基本的编码能力
- 测试错误处理
- 默认使用 `tests/audio/sample-3s.wav`

**测试内容**:
- 基本编码功能
- 不同输入格式处理
- 错误条件处理

### 2. 实时对比测试 (`encoder_comparison_live.rs`)
**目的**: 与Shine参考实现进行实时对比
**要求**: 需要Shine二进制文件存在
**特点**:
- 实时运行Rust和Shine编码器
- 比较输出文件的大小和哈希值
- 验证完全一致性

**测试文件**:
- 默认: `tests/audio/sample-3s.wav` (立体声 44.1kHz)
- 语音: `tests/audio/voice-recorder-testing-1-2-3-sound-file.wav` (单声道 48kHz)
- 大文件: `tests/audio/Free_Test_Data_500KB_WAV.wav` (立体声 44.1kHz, 较大文件)

**测试配置**:
- 不同比特率 (128, 192, 256 kbps)
- 默认设置对比

### 3. CI/CD验证测试 (`encoder_validation_cicd.rs`)
**目的**: 使用预生成的参考文件进行验证，适合CI/CD环境
**特点**:
- 不需要Shine二进制文件
- 使用预计算的参考数据
- 快速验证算法一致性

**数据来源**: `tests/integration_reference_validation.data/`
**验证内容**:
- 单声道/立体声配置
- 不同比特率 (128, 192, 256 kbps)
- 文件完整性检查

### 4. 低级API测试 (`encoder_low_level_api.rs`)
**目的**: 验证Shine兼容的低级API函数
**特点**:
- 直接镜像C实现的函数
- 测试配置初始化
- 验证编码器生命周期

**测试内容**:
- `ShineConfig` 初始化和验证
- `shine_initialise`, `shine_encode_buffer_interleaved`, `shine_flush`, `shine_close`
- 不同配置组合
- 错误条件处理
- 属性测试 (使用proptest)

### 5. 高级API测试 (`encoder_high_level_api.rs`)
**目的**: 验证便捷的Rust风格高级API
**特点**:
- 更友好的Rust接口
- 配置验证
- 流式编码支持

**测试内容**:
- `Mp3EncoderConfig` 配置验证
- `Mp3Encoder` 创建和使用
- PCM数据编码 (`encode_interleaved`)
- 编码完成 (`finish`)
- 便利函数 (`encode_pcm_to_mp3`)
- 不同立体声模式
- 错误条件处理
- 完整工作流程测试

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试类别
```bash
# 基本功能测试
cargo test --test encoder_basic_functionality

# 实时对比测试 (需要Shine二进制)
cargo test --test encoder_comparison_live

# CI/CD验证测试
cargo test --test encoder_validation_cicd

# 低级API测试
cargo test --test encoder_low_level_api

# 高级API测试
cargo test --test encoder_high_level_api
```

### 带诊断功能的测试
```bash
cargo test --features diagnostics
```

## 测试状态

### ✅ 通过的测试
- `encoder_basic_functionality` - 基本编码功能正常
- `encoder_low_level_api` - 低级API函数工作正常
- 部分实时对比测试 (默认设置, 128kbps)

### ⚠️ 部分通过的测试
- `encoder_comparison_live` - 某些配置通过，但192kbps和部分文件哈希不匹配
- `encoder_high_level_api` - 需要修复编译错误

### ❌ 失败的测试
- `encoder_validation_cicd` - 哈希值不匹配，表明算法实现还有差异

## 已知问题

1. **比特率参数传递**: 192kbps测试显示比特率参数可能没有正确传递给CLI
2. **算法一致性**: 某些文件的输出与Shine不完全一致，需要进一步调试
3. **高级API编译**: 需要修复类型匹配问题

## 下一步工作

1. 修复高级API测试的编译错误
2. 调试比特率参数传递问题
3. 分析算法差异，确保与Shine完全一致
4. 完善CI/CD验证数据
5. 添加更多边界条件测试

## 测试数据

### 音频文件
- `tests/audio/sample-3s.wav` - 标准测试文件 (立体声 44.1kHz)
- `tests/audio/voice-recorder-testing-1-2-3-sound-file.wav` - 语音测试 (单声道 48kHz)
- `tests/audio/Free_Test_Data_500KB_WAV.wav` - 大文件测试 (立体声 44.1kHz)

### 参考数据
- `tests/integration_reference_validation.data/` - 预生成的Shine参考输出
- `tests/integration_reference_validation.data/reference_manifest.json` - 参考数据清单

## 贡献指南

添加新测试时，请遵循以下原则：
1. 选择合适的测试类别
2. 避免重复现有测试功能
3. 使用描述性的测试名称
4. 包含适当的错误处理验证
5. 遵循项目的编码规范