# MP3编码器对比工具

这些Python脚本用于对比Rust版本和Shine版本的MP3编码器，帮助验证编码结果的一致性和性能差异。

## 脚本说明

### 1. compare_encoders.py - 完整对比工具

功能最全面的对比工具，支持所有命令行选项和详细的结果分析。

**用法:**
```bash
python scripts/compare_encoders.py input.wav [options]
```

**选项:**
- `-b, --bitrate`: 比特率 [8-320]，默认128kbit
- `-m, --mono`: 强制单声道模式
- `-c, --copyright`: 设置版权标志
- `-j, --joint-stereo`: 联合立体声编码
- `-d, --dual-channel`: 双声道编码
- `-q, --quiet`: 安静模式
- `-v, --verbose`: 详细模式
- `--rust-only`: 仅运行Rust编码器
- `--shine-only`: 仅运行Shine编码器
- `--output-dir`: 指定输出目录

**示例:**
```bash
# 基本用法
python scripts/compare_encoders.py sample.wav

# 指定比特率和详细模式
python scripts/compare_encoders.py sample.wav -b 192 -v

# 联合立体声模式
python scripts/compare_encoders.py sample.wav -b 128 -j

# 仅测试Rust版本
python scripts/compare_encoders.py sample.wav --rust-only -v
```

### 2. quick_compare.py - 快速对比工具

简化版本，用于快速对比两个编码器的基本结果。

**用法:**
```bash
python scripts/quick_compare.py input.wav
```

**特点:**
- 使用默认设置（128kbps，自动立体声模式）
- 显示编码时间和文件大小对比
- 输出简洁明了

### 3. batch_compare.py - 批量对比工具

用于批量处理多个WAV文件，进行大规模对比测试。

**用法:**
```bash
python scripts/batch_compare.py [directory] [options]
```

**选项:**
- `--pattern`: 文件匹配模式，默认`*.wav`
- `-b, --bitrate`: 比特率
- `-m, --mono`: 强制单声道
- `-j, --joint-stereo`: 联合立体声
- `-q, --quiet`: 安静模式
- `-v, --verbose`: 详细模式
- `--output-dir`: 输出目录
- `--save-report`: 保存详细报告到JSON文件

**示例:**
```bash
# 处理testing目录下的所有WAV文件
python scripts/batch_compare.py testing/

# 指定比特率和详细模式
python scripts/batch_compare.py . -b 192 -v

# 保存详细报告
python scripts/batch_compare.py testing/ --save-report report.json
```

## 输出文件命名

所有脚本都会生成带有后缀的输出文件：
- `filename_rust.mp3` - Rust版本编码器的输出
- `filename_shine.mp3` - Shine版本编码器的输出

## 前置条件

### 1. 构建编码器

**Rust版本:**
```bash
cargo build --release
```

**Shine版本:**
```bash
cd ref/shine
./build.ps1  # Windows
# 或
make         # Linux/macOS
```

### 2. Python依赖

脚本使用Python标准库，无需额外安装依赖。

## 结果分析

### 文件大小对比
- **完全相同**: 文件大小完全一致（理想情况）
- **非常接近**: 差异小于1%（可接受）
- **略有差异**: 差异1-5%（需要检查）
- **差异较大**: 差异大于5%（需要调试）

### 性能对比
脚本会显示两个编码器的执行时间对比，帮助评估性能差异。

### 详细统计
在详细模式下，脚本会显示：
- 输入文件信息
- 输出文件大小和压缩比
- MP3文件头信息
- 实际比特率计算

## 常见问题

### 1. 找不到编码器可执行文件
确保已经正确构建了两个编码器：
```bash
# 检查Rust编码器
ls target/release/shine-rs-cli.exe

# 检查Shine编码器  
ls ref/shine/shineenc.exe
```

### 2. 编码失败
检查输入文件格式是否正确：
- 必须是WAV格式
- 支持16位PCM编码
- 支持常见采样率（44.1kHz, 48kHz等）

### 3. 文件大小差异较大
可能的原因：
- 算法实现差异
- 量化参数不同
- 比特池管理差异
- 需要进一步调试算法实现

## 调试建议

1. **从简单文件开始**: 使用短时间、单声道的测试文件
2. **使用详细模式**: 添加`-v`选项查看详细输出
3. **对比特定参数**: 使用相同的比特率和立体声模式
4. **检查中间结果**: 查看编码过程中的调试信息

## 示例工作流

```bash
# 1. 快速测试单个文件
python scripts/quick_compare.py sample.wav

# 2. 详细对比特定设置
python scripts/compare_encoders.py sample.wav -b 128 -j -v

# 3. 批量测试所有文件
python scripts/batch_compare.py testing/ -v --save-report results.json

# 4. 分析结果
cat results.json | jq '.[] | select(.size_diff_percent > 1)'
```

这些工具可以帮助确保Rust版本的MP3编码器与Shine参考实现保持一致，是验证算法正确性的重要工具。