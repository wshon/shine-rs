# MP3编码器测试流程

## 基本测试步骤

### 1. WAV转MP3
```bash
cargo run --bin wav2mp3 testing/fixtures/audio/sample-12s.wav testing/fixtures/output/sample-12s-output.mp3
```

### 2. MP3格式验证
```bash
cargo run --bin mp3_validator testing/fixtures/output/sample-12s-output.mp3
```

### 3. MP3解码回WAV
```bash
ffmpeg -i testing/fixtures/output/sample-12s-output.mp3 -y testing/fixtures/output/decoded-sample.wav
```

### 4. 文件信息对比
```bash
ffprobe -v quiet -print_format json -show_format testing/fixtures/output/decoded-sample.wav
ffprobe -v quiet -print_format json -show_format testing/fixtures/audio/sample-12s.wav
```

## 音量分析和问题诊断

### 检查音频音量
```bash
# 检查原始WAV文件音量
ffmpeg -i testing/fixtures/audio/sample-12s.wav -af "volumedetect" -f null -

# 检查生成的MP3文件音量
ffmpeg -i testing/fixtures/output/sample-12s-output.mp3 -af "volumedetect" -f null -
```

### 已知问题：音量过低
**问题描述：** 生成的MP3文件音量极低，播放时听不到声音

**测试结果对比：**
- 原始WAV：平均音量 -25.9dB，最大音量 -12.2dB
- 生成MP3：平均音量 -91.0dB，最大音量 -84.3dB
- 音量差异：约65-70dB的衰减

### 验证音频数据完整性
```bash
# 放大MP3音频以验证数据存在
ffmpeg -i testing/fixtures/output/sample-12s-output.mp3 -af "volume=70dB" testing/fixtures/output/amplified-sample.wav

# 检查放大后的音量
ffmpeg -i testing/fixtures/output/amplified-sample.wav -af "volumedetect" -f null -
```

### 十六进制分析
```bash
# 检查生成的MP3文件内部结构
cargo run --bin mp3_hexdump testing/fixtures/output/sample-12s-output.mp3

# 对比shine参考实现
cargo run --bin mp3_hexdump testing/fixtures/audio/sample-12s-shine.mp3
```

**关键发现：**
- 放大后的音频是杂音，不是原始音频
- 十六进制分析显示：
  - 帧头正确 (`FF FB 92 44`)
  - **主数据区域几乎全是零** (`00 00 00 00...`)
  - Shine参考实现的主数据区域充满了音频数据

**结论：** 问题不是音量衰减，而是音频数据根本没有被正确编码到MP3文件中。主数据区域为空表明问题出现在：
1. 量化过程 - 所有系数可能被错误地量化为零
2. Huffman编码 - 编码过程可能有严重错误
3. 比特流写入 - 主数据可能没有被正确写入帧中
ffmpeg -i testing/fixtures/output/sample-12s-output.mp3 -af "volumedetect" -f null - 2>&1