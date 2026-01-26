# 参考文件生成脚本

这个目录包含用于生成和维护MP3编码器测试参考文件的脚本。

## 脚本说明

### generate_reference_files.py

自动化生成参考MP3文件的Python脚本，用于确保测试的可靠性和可复制性。

#### 功能特性

- **跨平台兼容**: 自动检测Shine编码器二进制文件（支持Linux/macOS/Windows）
- **多配置支持**: 支持生成不同帧数限制的参考文件
- **自动验证**: 验证生成文件的大小和完整性
- **测试常量更新**: 自动更新测试代码中的哈希值常量
- **清单生成**: 生成包含所有参考文件信息的JSON清单

#### 使用方法

```bash
# 生成所有参考文件
python scripts/generate_reference_files.py

# 只生成6帧参考文件（用于SCFSI测试）
python scripts/generate_reference_files.py --configs 6frames

# 生成3帧参考文件（用于快速测试）
python scripts/generate_reference_files.py --configs 3frames

# 生成多个配置
python scripts/generate_reference_files.py --configs 6frames 3frames

# 不自动更新测试常量
python scripts/generate_reference_files.py --no-update-tests

# 指定工作目录
python scripts/generate_reference_files.py --workspace /path/to/shine-rs
```

#### 配置说明

脚本支持以下预定义配置：

| 配置名 | 描述 | 帧数 | 预期大小 | 用途 |
|--------|------|------|----------|------|
| 6frames | 6帧参考文件 | 6 | 2508字节 | SCFSI一致性测试 |
| 3frames | 3帧参考文件 | 3 | 1252字节 | 快速测试 |

#### 输出文件

脚本会生成以下文件：

- `tests/audio/shine_reference_6frames.mp3` - 6帧参考文件
- `tests/audio/shine_reference_3frames.mp3` - 3帧参考文件（如果生成）
- `tests/audio/reference_manifest.json` - 参考文件清单

#### 前置条件

1. **Shine编码器**: 确保Shine编码器已构建并可用
   - Linux/macOS: `ref/shine/shineenc`
   - Windows: `ref/shine/shineenc.exe`

2. **输入文件**: 确保测试音频文件存在
   - `tests/audio/sample-3s.wav`

3. **Python环境**: Python 3.6+

#### 工作流程

1. **检查前置条件**: 验证Shine编码器和输入文件
2. **生成参考文件**: 使用Shine编码器生成MP3文件
3. **验证输出**: 检查文件大小和计算SHA256哈希
4. **更新测试常量**: 自动更新测试代码中的哈希值
5. **生成清单**: 创建包含所有文件信息的JSON清单

#### 错误处理

脚本包含完整的错误处理：

- **缺少Shine编码器**: 提供清晰的错误信息和解决建议
- **输入文件不存在**: 列出所有缺少的文件
- **编码失败**: 显示Shine编码器的错误输出
- **验证失败**: 报告文件大小或哈希不匹配

#### 示例输出

```
🚀 Starting reference file generation...
   Workspace: /path/to/shine-rs
🔍 Checking prerequisites...
✅ Shine encoder found: /path/to/shine-rs/ref/shine/shineenc
✅ Audio directory found: /path/to/shine-rs/tests/audio
✅ Input file found: /path/to/shine-rs/tests/audio/sample-3s.wav

📁 Generating reference file: 6frames
   Description: 6-frame reference for SCFSI consistency testing
🎵 Running Shine encoder...
   Command: /path/to/shine-rs/ref/shine/shineenc /path/to/shine-rs/tests/audio/sample-3s.wav /path/to/shine-rs/tests/audio/shine_reference_6frames.mp3
   Frame limit: 6
✅ Shine encoder completed successfully
✅ Reference file generated successfully
   File: /path/to/shine-rs/tests/audio/shine_reference_6frames.mp3
   Size: 2508 bytes
   SHA256: 4385b617a86cb3891ce3c99dabe6b47c2ac9182b32c46cbc5ad167fb28b959c4

📊 Generation Summary:
   ✅ Successful: 1
   ❌ Failed: 0

🔧 Updating test constants...
✅ Updated SCFSI test constants
✅ Generated manifest: /path/to/shine-rs/tests/audio/reference_manifest.json

🎉 Reference file generation completed successfully!
```

## 维护指南

### 添加新配置

要添加新的参考文件配置，编辑`generate_reference_files.py`中的`reference_configs`字典：

```python
self.reference_configs = {
    "new_config": {
        "description": "新配置的描述",
        "frame_limit": 10,  # 帧数限制
        "expected_size": 4180,  # 预期文件大小（字节）
        "input_file": "sample-3s.wav",  # 输入文件名
        "output_file": "shine_reference_10frames.mp3"  # 输出文件名
    }
}
```

### 更新Shine编码器

如果Shine编码器有更新，重新生成参考文件：

```bash
# 重新构建Shine
cd ref/shine
make clean && make

# 重新生成所有参考文件
python scripts/generate_reference_files.py
```

### 验证参考文件

生成参考文件后，运行测试验证：

```bash
# 运行SCFSI一致性测试
cargo test test_scfsi_consistency_with_shine --features diagnostics

# 运行所有SCFSI测试
cargo test --test integration_scfsi_consistency --features diagnostics
```

## 故障排除

### 常见问题

1. **Shine编码器未找到**
   - 确保Shine已正确构建
   - 检查二进制文件权限（Linux/macOS需要执行权限）

2. **输入文件缺失**
   - 确保`tests/audio/sample-3s.wav`存在
   - 检查文件路径和权限

3. **文件大小不匹配**
   - 可能是Shine版本差异导致
   - 检查Shine的调试输出和帧数限制

4. **哈希值不匹配**
   - 重新生成参考文件
   - 检查Shine编码器是否有修改

### 调试模式

要获得更详细的调试信息，可以修改脚本中的日志级别或添加额外的调试输出。

## 集成到CI/CD

可以将参考文件生成集成到持续集成流程中：

```yaml
# GitHub Actions示例
- name: Generate reference files
  run: python scripts/generate_reference_files.py --no-update-tests

- name: Verify reference files
  run: cargo test --test integration_scfsi_consistency --features diagnostics
```

这确保了参考文件始终与当前的Shine实现保持同步。