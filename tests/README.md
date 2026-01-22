# MP3 编码器测试套件

本目录包含了MP3编码器的完整测试套件，按功能模块组织。

## 测试文件结构

### 核心验证测试
- **`validation_comprehensive.rs`** - 综合验证测试
  - 基础编码功能测试
  - 不同配置参数测试
  - FFmpeg兼容性验证
  - 真实音频文件处理
  - 信号生成器和验证工具

### 调试和诊断工具
- **`debug_comprehensive.rs`** - 综合调试测试
  - 编码管道隔离测试
  - 不同信号模式测试
  - 属性测试（proptest）
  - 性能和边界条件测试

- **`debug_tools.rs`** - 调试工具模块
  - 帧分析器
  - 管道分析器
  - 信号生成器
  - MP3结构分析工具

### 参考实现比较
- **`shine_reference_tests.rs`** - Shine参考实现比较
  - 与shine编码器的输出对比
  - 算法一致性验证
  - 数值精度测试
  - WAV文件处理工具

### 专项验证测试
- **`big_values_validation_test.rs`** - big_values字段验证
  - MP3规范合规性检查
  - 边界值测试
  - 不同信号模式验证

## 测试分类

### 单元测试 (Unit Tests)
- 测试单个函数和模块的行为
- 快速执行，覆盖基本功能
- 位于各个测试文件的 `tests` 模块中

### 集成测试 (Integration Tests)
- 测试完整的编码流程
- 验证模块间的协作
- 包含端到端的编码验证

### 属性测试 (Property Tests)
- 使用 `proptest` 进行随机输入测试
- 发现边界条件和异常情况
- 验证算法的鲁棒性

### 参考验证测试 (Reference Tests)
- 与shine参考实现对比
- 确保算法一致性
- 数值精度验证

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定模块测试
```bash
# 验证测试
cargo test validation_comprehensive

# 调试测试
cargo test debug_comprehensive

# Shine比较测试
cargo test shine_reference

# Big values验证
cargo test big_values_validation
```

### 运行需要外部工具的测试
```bash
# 需要FFmpeg的测试
cargo test test_ffmpeg_validation -- --ignored

# 需要shine编码器的测试
cargo test test_shine_comparison -- --ignored

# 需要真实音频文件的测试
cargo test test_real_audio_file -- --ignored
```

### 运行属性测试
```bash
# 设置proptest环境变量
PROPTEST_VERBOSE=0 cargo test

# 运行更多测试用例
PROPTEST_CASES=1000 cargo test
```

## 测试数据

### 输入文件 (`tests/input/`)
- 测试用的WAV音频文件
- 参考数据文件

### 输出文件 (`tests/output/`)
- 测试生成的MP3文件
- 调试输出和分析结果
- 自动清理旧文件

## 测试指南

### 添加新测试
1. 确定测试类型（单元/集成/属性/参考）
2. 选择合适的测试文件
3. 遵循命名约定：`test_<module>_<behavior>`
4. 使用英文错误消息
5. 保持测试简洁明确

### 调试测试失败
1. 使用 `debug_comprehensive.rs` 中的工具
2. 检查生成的MP3文件
3. 与shine参考实现对比
4. 分析编码管道各阶段

### 性能测试
1. 使用 `cargo bench` 运行基准测试
2. 属性测试控制在1秒内完成
3. 大数据集测试使用 `#[ignore]` 标记

## 依赖工具

### 可选外部工具
- **FFmpeg** - MP3文件验证和分析
- **FFprobe** - 获取MP3文件信息
- **shine** - 参考实现比较

### 安装说明
```bash
# Windows (使用 chocolatey)
choco install ffmpeg

# macOS (使用 homebrew)
brew install ffmpeg

# Ubuntu/Debian
sudo apt install ffmpeg

# shine编码器需要单独编译安装
```

## 注意事项

1. **零警告政策** - 所有测试必须无编译警告
2. **英文消息** - 错误消息和断言使用英文
3. **简洁明确** - 避免冗长的测试名称和消息
4. **模块化** - 相关功能组织在同一文件中
5. **文档化** - 复杂测试需要注释说明