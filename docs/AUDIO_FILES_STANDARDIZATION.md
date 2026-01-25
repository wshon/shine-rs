# 测试音频文件标准化

本文档记录了测试音频文件的标准化过程和使用规范。

## 标准化目标

1. **简化测试文件** - 只保留必要的测试音频文件
2. **统一测试标准** - 优先使用单一主要测试文件
3. **清晰的用途分工** - 不同文件有明确的测试目的
4. **减少维护成本** - 避免过多冗余的测试文件

## 最终保留的文件

### 主要测试文件
- **`sample-3s.wav`** ⭐ - 主要测试文件，用于大部分测试场景
- **`Free_Test_Data_500KB_WAV.wav`** - 大文件性能测试
- **`voice-recorder-testing-1-2-3-sound-file.wav`** - 语音内容测试

### 已删除的文件
- `sample-12s.wav` - 与sample-3s.wav功能重复
- `test_input.wav` - 替换为sample-3s.wav
- `test_input_mono.wav` - 不再需要单独的单声道测试
- `demo.mp3` - 不再需要MP3格式的测试文件
- `sample-12s.mp3` - 不再需要
- `sample-3s.mp3` - 不再需要
- `*-shine.mp3` - 参考输出文件，不再需要预存
- `test.mp3` - 不再需要

## 使用规范

### 优先级顺序
1. **首选**: `sample-3s.wav` - 用于所有基础测试
2. **性能测试**: `Free_Test_Data_500KB_WAV.wav` - 用于大文件测试
3. **特殊测试**: `voice-recorder-testing-1-2-3-sound-file.wav` - 用于语音测试

### 测试场景分配

#### 日常开发测试
```bash
# 基础编码测试
cargo run --bin wav2mp3 testing/fixtures/audio/sample-3s.wav output.mp3

# 数据收集
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav test_data.json 128

# 集成测试
cargo test --test integration_scfsi_consistency
```

#### 性能测试
```bash
# 大文件编码测试
cargo run --bin wav2mp3 testing/fixtures/audio/Free_Test_Data_500KB_WAV.wav large_output.mp3
```

#### 特殊内容测试
```bash
# 语音内容编码测试
cargo run --bin wav2mp3 testing/fixtures/audio/voice-recorder-testing-1-2-3-sound-file.wav voice_output.mp3
```

## 更新记录

### 代码文件更新
以下文件中的音频文件引用已更新为使用 `sample-3s.wav`：

1. **测试代码**:
   - `testing/integration/integration_scfsi_consistency.rs`
   - `testing/integration/integration_full_pipeline_validation.rs`

2. **工具代码**:
   - `src/bin/collect_test_data.rs`
   - `src/bin/README.md`

3. **文档**:
   - `docs/TEST_DATA_FRAMEWORK.md`
   - `testing/fixtures/audio/README.md`

### 路径更新
所有引用都已更新为新的路径格式：
- 旧格式: `test_input.wav`, `sample-12s.wav`
- 新格式: `testing/fixtures/audio/sample-3s.wav`

## 文件特性对比

| 文件名 | 大小 | 时长 | 采样率 | 声道 | 主要用途 |
|--------|------|------|--------|------|----------|
| sample-3s.wav | 小 | 3秒 | 44.1kHz | 立体声 | 主要测试 ⭐ |
| Free_Test_Data_500KB_WAV.wav | 大 | 较长 | 未知 | 未知 | 性能测试 |
| voice-recorder-testing-1-2-3-sound-file.wav | 中 | 中等 | 未知 | 未知 | 语音测试 |

## 优势总结

### 1. 简化维护
- **减少文件数量**: 从10+个文件减少到3个核心文件
- **统一标准**: 所有基础测试使用同一个文件
- **清晰分工**: 每个文件有明确的测试目的

### 2. 提高效率
- **快速测试**: sample-3s.wav文件小，测试速度快
- **一致性**: 所有开发者使用相同的测试文件
- **易于调试**: 问题复现更容易

### 3. 降低复杂度
- **减少选择困难**: 明确的优先级顺序
- **简化配置**: 更少的路径配置需要维护
- **统一文档**: 所有示例使用相同的文件

## 迁移指南

### 对于开发者
1. **更新本地脚本**: 将所有测试脚本中的文件名更新为 `sample-3s.wav`
2. **清理旧文件**: 删除本地的旧测试文件
3. **使用新路径**: 采用 `testing/fixtures/audio/` 路径格式

### 对于CI/CD
1. **更新测试脚本**: 确保自动化测试使用新的文件路径
2. **验证文件存在**: 确保所需的3个文件在测试环境中可用
3. **更新文档**: 同步更新相关的测试文档

## 后续维护

### 添加新测试文件的原则
1. **必要性评估**: 确认新文件确实有独特的测试价值
2. **避免重复**: 不添加与现有文件功能重复的文件
3. **文档更新**: 新文件必须有清晰的用途说明
4. **测试集成**: 新文件应该集成到自动化测试中

### 定期审查
- **季度审查**: 每季度审查测试文件的使用情况
- **清理无用文件**: 删除不再使用的测试文件
- **优化测试效率**: 根据使用情况调整文件优先级

## 结论

通过标准化测试音频文件，我们实现了：
- **简化的测试流程** - 明确的文件使用优先级
- **提高的开发效率** - 统一的测试标准
- **降低的维护成本** - 更少的文件需要管理
- **更好的一致性** - 所有测试使用相同的基准

这个标准化过程为项目的长期维护和发展奠定了良好的基础。