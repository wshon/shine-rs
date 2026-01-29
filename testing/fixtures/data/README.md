# 测试数据文件

这个目录包含用于深度验证编码过程的JSON测试数据文件。

## 文件说明

- `free_test_data_128k_3f.json` - Free Test Data音频文件的3帧编码数据
- `sample-3s_128k_3f.json` - 3秒样本音频的3帧编码数据  
- `sample-3s_192k_3f.json` - 3秒样本音频的192kbps编码数据
- `voice_recorder_128k_3f.json` - 语音录音测试文件的编码数据

## 数据格式

每个JSON文件包含：
- **元数据**: 输入文件、预期输出大小、哈希值等
- **配置**: 采样率、声道数、比特率等编码参数
- **帧数据**: 每帧的详细编码过程数据
  - MDCT系数（混叠减少前后）
  - 量化参数（xrmax, global_gain, part2_3_length等）
  - 比特流参数（padding, bits_per_frame, slot_lag等）

## 使用说明

这些数据文件主要用于：
1. 算法调试和验证
2. 编码过程的逐步验证
3. 与Shine参考实现的对比

注意：这些是历史数据文件，当前推荐使用 `tests/audio/inputs/reference_manifest.json` 进行CI/CD验证。