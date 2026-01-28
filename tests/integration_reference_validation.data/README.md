# 参考验证测试数据

这个目录包含 `integration_reference_validation.rs` 测试所需的所有数据文件。

## 文件说明

### reference_manifest.json
参考文件配置清单，包含：
- 每个参考文件的描述
- 文件路径（相对于项目根目录）
- 预期文件大小（字节）
- SHA256哈希值用于完整性验证

### 参考MP3文件
以 `shine_reference_` 开头的MP3文件是使用Shine编码器生成的参考输出：

#### 标准测试文件（基于sample-3s.wav）
- `shine_reference_1frame.mp3` - 单帧参考（416字节）
- `shine_reference_2frames.mp3` - 2帧参考（836字节）
- `shine_reference_3frames.mp3` - 3帧参考（1252字节）
- `shine_reference_6frames.mp3` - 6帧参考（2508字节）
- `shine_reference_10frames.mp3` - 10帧参考（4180字节）
- `shine_reference_15frames.mp3` - 15帧参考（6268字节）
- `shine_reference_20frames.mp3` - 20帧参考（8360字节）

#### 语音文件测试（基于voice-recorder-testing-1-2-3-sound-file.wav）
- `shine_reference_voice_3frames.mp3` - 语音3帧参考（1152字节）
- `shine_reference_voice_6frames.mp3` - 语音6帧参考（2304字节）

#### 大文件测试（基于Free_Test_Data_500KB_WAV.wav）
- `shine_reference_large_3frames.mp3` - 大文件3帧参考（1252字节）
- `shine_reference_large_6frames.mp3` - 大文件6帧参考（2508字节）

## 生成方式

这些参考文件通过以下脚本生成：
```bash
python scripts/generate_reference_files.py
```

该脚本会：
1. 使用Shine编码器对各种输入文件进行编码
2. 生成不同帧数限制的参考输出
3. 计算每个文件的SHA256哈希值
4. 更新 `reference_manifest.json` 清单文件

## 使用方式

参考验证测试会：
1. 读取 `reference_manifest.json` 获取预期结果
2. 使用Rust编码器对相同输入进行编码
3. 对比输出文件的大小和SHA256哈希值
4. 验证Rust实现与Shine参考实现的一致性

## 文件完整性

所有参考文件都包含SHA256哈希值用于完整性验证。如果文件被意外修改或损坏，测试会检测到并报告错误。

## 重新生成

如果需要重新生成参考文件（例如算法更新后），运行：
```bash
# 确保Shine编码器已构建
cd ref/shine && ./build.ps1 && cd ../..

# 重新生成所有参考文件
python scripts/generate_reference_files.py

# 验证新生成的文件
cargo test integration_reference_validation
```

## 注意事项

- 这些文件是二进制MP3文件，不应手动编辑
- 文件大小和哈希值必须与清单文件中的值完全匹配
- 参考文件基于特定版本的Shine编码器生成，算法变更可能需要重新生成
- 语音文件测试可能因为单声道48kHz处理差异而失败（这是已知问题）