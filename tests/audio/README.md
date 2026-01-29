# 测试音频文件组织结构

## 目录结构

```
tests/audio/
├── README.md                    # 本文档
├── inputs/                      # 输入音频文件（源WAV文件）
│   ├── basic/                   # 基础测试文件
│   │   ├── sample-3s.wav        # 3秒立体声样本 (44.1kHz)
│   │   ├── voice-recorder-testing-1-2-3-sound-file.wav  # 语音测试文件 (48kHz mono)
│   │   └── Free_Test_Data_500KB_WAV.wav  # 大文件测试 (44.1kHz stereo)
│   └── frame-specific/          # 特定帧数测试文件
│       ├── test_1frames_mono.wav
│       ├── test_1frames_stereo.wav
│       ├── test_2frames_mono.wav
│       ├── test_2frames_stereo.wav
│       ├── test_3frames_mono.wav
│       ├── test_3frames_stereo.wav
│       ├── test_6frames_mono.wav
│       ├── test_6frames_stereo.wav
│       ├── test_10frames_mono.wav
│       ├── test_10frames_stereo.wav
│       ├── test_15frames_mono.wav
│       ├── test_15frames_stereo.wav
│       ├── test_20frames_mono.wav
│       └── test_20frames_stereo.wav
└── outputs/                     # 输出文件（测试生成的MP3文件）
    ├── comparison/              # 实时比较测试输出
    │   ├── *_rust_*.mp3         # Rust编码器输出
    │   └── *_shine_*.mp3        # Shine编码器输出
    └── temp/                    # 临时测试输出文件
        └── test_*.mp3           # 各种测试生成的临时文件
```

## 文件用途说明

### 基础输入文件
- **sample-3s.wav**: 默认测试文件，3秒立体声，44.1kHz采样率
- **voice-recorder-testing-1-2-3-sound-file.wav**: 语音测试文件，单声道，48kHz采样率
- **Free_Test_Data_500KB_WAV.wav**: 大文件测试，立体声，44.1kHz采样率

### 帧数特定文件
这些文件用于测试特定帧数的编码，每个文件包含精确的帧数：
- **1-3帧**: 用于基础功能测试
- **6帧**: 用于标准测试
- **10-20帧**: 用于扩展测试

### 输出文件
- **comparison/**: 实时比较测试的输出，包含Rust和Shine编码器的对比结果
- **temp/**: 临时测试文件，可以安全删除

## 测试文件使用

### 基础功能测试
```rust
// 使用默认测试文件
let input_file = "tests/audio/inputs/basic/sample-3s.wav";
```

### 实时比较测试
```rust
// 比较不同类型的音频文件
let test_files = [
    "tests/audio/inputs/basic/sample-3s.wav",
    "tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav",
    "tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav",
];
```

### CI/CD验证测试
使用 `tests/integration_reference_validation.data/` 中的预生成参考数据。

## 文件管理

### 清理临时文件
```bash
# 清理临时输出文件
rm -rf tests/audio/outputs/temp/*

# 清理比较输出文件
rm -rf tests/audio/outputs/comparison/*
```

### 重新生成测试文件
```bash
# 重新生成帧数特定的WAV文件
python scripts/generate_test_wav.py

# 重新生成参考验证数据
python scripts/generate_reference_validation_data.py
```

## 注意事项

1. **不要提交输出文件**: `outputs/` 目录中的文件不应该提交到版本控制
2. **保持输入文件稳定**: `inputs/` 目录中的WAV文件是测试的基础，不应随意修改
3. **定期清理**: 定期清理临时输出文件以节省磁盘空间
4. **文件命名规范**: 新增测试文件应遵循现有的命名规范