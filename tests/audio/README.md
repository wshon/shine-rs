# 测试音频文件说明

本目录包含用于MP3编码器测试的音频文件。

## 文件来源

- **Free_Test_Data_500KB_WAV.wav** - 来自 https://freetestdata.com/audio-files/wav/
- **sample-3s.wav** - 来自 https://samplelib.com/zh/sample-wav.html
- **voice-recorder-testing-1-2-3-sound-file.wav** - 来自 https://voicerecorder.org/blog/free-download-testing-1-2-3-sound-file/

## 主要测试文件

### sample-3s.wav ⭐ 
- **主要测试文件** - 3秒的WAV格式音频文件
- 用于大部分编码测试和验证
- 提供一致的测试基准
- 优先在所有测试中使用此文件

### Free_Test_Data_500KB_WAV.wav
- **大文件测试** - 约500KB的WAV格式音频文件
- 用于测试较大音频文件的编码性能
- 验证编码器处理长音频的能力

### voice-recorder-testing-1-2-3-sound-file.wav
- **语音测试** - 包含语音内容的WAV文件
- 用于测试编码器对语音信号的处理
- 验证不同音频内容类型的编码效果

## 使用指南

### 基础测试（推荐）
优先使用 `sample-3s.wav` 进行日常测试：

```bash
# 基础编码测试
cargo run --bin wav2mp3 testing/fixtures/audio/sample-3s.wav testing/fixtures/output/sample-3s-output.mp3

# 收集测试数据
cargo run --bin collect_test_data -- testing/fixtures/audio/sample-3s.wav testing/fixtures/data/test_data.json 128

# 验证测试数据
cargo run --bin validate_test_data -- testing/fixtures/data/test_data.json
```

### 扩展测试
使用其他文件进行特定测试：

```bash
# 大文件编码测试
cargo run --bin wav2mp3 testing/fixtures/audio/Free_Test_Data_500KB_WAV.wav testing/fixtures/output/large_output.mp3

# 语音文件编码测试
cargo run --bin wav2mp3 testing/fixtures/audio/voice-recorder-testing-1-2-3-sound-file.wav testing/fixtures/output/voice_output.mp3
```

## 测试流程

1. **编码测试**：使用 `sample-3s.wav` 作为主要输入进行编码测试
2. **数据验证**：通过测试数据框架验证编码正确性
3. **性能测试**：使用不同大小和类型的音频文件测试性能
4. **格式验证**：使用MP3验证工具检查输出格式

## 文件特性

| 文件名 | 大小 | 时长 | 主要用途 |
|--------|------|------|----------|
| sample-3s.wav | 小 | 3秒 | 主要测试文件 ⭐ |
| Free_Test_Data_500KB_WAV.wav | 大 | 较长 | 性能测试 |
| voice-recorder-testing-1-2-3-sound-file.wav | 中 | 中等 | 语音测试 |

## 测试验证目标

这些文件主要用于：
- **编码功能验证** - 确保编码器能正确处理WAV输入
- **输出一致性** - 验证与shine参考实现的一致性
- **性能测试** - 测试不同大小文件的编码性能
- **回归测试** - 防止代码修改影响编码质量
- **集成测试** - 验证完整的编码流程

## 注意事项

- **优先使用** `sample-3s.wav` 进行开发和调试
- 所有测试文件都应该是有效的WAV格式
- 大文件测试应该在性能测试阶段进行
- 确保测试文件的音频参数（采样率、声道数等）符合编码器要求
- 新增测试文件时，应该同时更新相关的测试用例和文档