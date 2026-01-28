# 批量编码器对比集成测试

## 概述

`integration_batch_encoder_comparison.rs` 是一个综合性的集成测试模块，用于批量对比Rust版本和Shine版本MP3编码器的输出结果。该测试自动处理 `tests/audio/` 目录下的所有WAV文件，验证两个编码器的一致性和性能。

## 测试功能

### 核心功能
- **自动发现**: 自动查找并测试 `tests/audio/` 目录下的所有WAV文件
- **双编码器对比**: 同时运行Rust和Shine编码器，生成对比结果
- **多种配置测试**: 支持不同比特率和立体声模式的测试
- **详细统计分析**: 提供成功率、性能对比、文件大小差异等统计信息

### 测试用例

#### 1. `test_batch_encoding_comparison_default`
- **配置**: 默认设置（128kbps，自动立体声模式）
- **目标**: 验证基本编码功能的一致性
- **断言**:
  - 至少80%的文件成功编码
  - 平均文件大小差异小于5%

#### 2. `test_batch_encoding_comparison_192kbps`
- **配置**: 192kbps比特率
- **目标**: 验证高比特率编码的一致性
- **断言**:
  - 至少70%的文件成功编码
  - 平均文件大小差异小于5%

#### 3. `test_batch_encoding_comparison_joint_stereo`
- **配置**: 128kbps联合立体声模式
- **目标**: 验证联合立体声编码的一致性
- **断言**:
  - 至少70%的文件成功编码
  - 平均文件大小差异小于5%

#### 4. `test_batch_encoding_comparison_comprehensive` (标记为ignore)
- **配置**: 多种配置的综合测试
- **目标**: 全面验证各种编码参数组合
- **断言**: 总体平均成功率大于70%

## 数据结构

### `EncodingResult`
记录单个编码器的执行结果：
```rust
struct EncodingResult {
    success: bool,           // 编码是否成功
    duration_ms: u128,       // 编码耗时（毫秒）
    output_size: u64,        // 输出文件大小
    error_message: Option<String>, // 错误信息
}
```

### `ComparisonResult`
记录单个文件的对比结果：
```rust
struct ComparisonResult {
    input_file: PathBuf,           // 输入文件路径
    input_size: u64,               // 输入文件大小
    rust_result: EncodingResult,   // Rust编码器结果
    shine_result: EncodingResult,  // Shine编码器结果
    size_difference: i64,          // 文件大小差异
    size_difference_percent: f64,  // 文件大小差异百分比
}
```

### `BatchTestStats`
批量测试的统计信息：
```rust
struct BatchTestStats {
    total_files: usize,                    // 总文件数
    rust_success_count: usize,             // Rust成功数
    shine_success_count: usize,            // Shine成功数
    both_success_count: usize,             // 双方都成功数
    total_rust_time_ms: u128,              // Rust总耗时
    total_shine_time_ms: u128,             // Shine总耗时
    average_size_difference_percent: f64,  // 平均大小差异百分比
    max_size_difference_percent: f64,      // 最大大小差异百分比
    identical_files_count: usize,          // 完全相同文件数
}
```

## 运行方式

### 运行所有批量对比测试
```bash
cargo test integration_batch_encoder_comparison
```

### 运行特定测试
```bash
# 默认设置测试
cargo test test_batch_encoding_comparison_default

# 192kbps测试
cargo test test_batch_encoding_comparison_192kbps

# 联合立体声测试
cargo test test_batch_encoding_comparison_joint_stereo
```

### 运行综合测试（需要显式包含ignored测试）
```bash
cargo test test_batch_encoding_comparison_comprehensive -- --ignored
```

### 显示详细输出
```bash
cargo test integration_batch_encoder_comparison -- --nocapture
```

## 前置条件

### 1. 编码器构建
测试需要两个编码器都已构建：

**Rust编码器**:
```bash
cargo build --release
```

**Shine编码器**:
```bash
cd ref/shine
./build.ps1  # Windows
# 或
make         # Linux/macOS
```

### 2. 测试音频文件
确保 `tests/audio/` 目录下有WAV格式的测试文件。

## 输出文件

测试会在 `tests/audio/` 目录下生成对比文件：
- `filename_rust.mp3` - Rust编码器输出
- `filename_shine.mp3` - Shine编码器输出

这些文件在测试开始时会被清理，确保测试结果的准确性。

## 统计分析

### 成功率分析
- **总文件数**: 测试的WAV文件总数
- **编码器成功率**: 各编码器的成功编码比例
- **双方成功率**: 两个编码器都成功的文件比例

### 性能分析
- **编码时间对比**: 两个编码器的总耗时对比
- **速度倍数**: 相对性能差异（如"Rust比Shine快2.1x"）

### 文件大小分析
- **平均差异**: 所有成功文件的平均大小差异百分比
- **最大差异**: 单个文件的最大大小差异
- **完全相同**: 输出文件完全相同的文件数量

## 断言标准

### 成功率标准
- **基本测试**: 至少80%成功率
- **特殊配置**: 至少70%成功率（考虑到某些配置的限制）
- **综合测试**: 平均70%成功率

### 一致性标准
- **文件大小差异**: 平均差异小于5%
- **算法一致性**: 期望大部分文件输出相同或非常接近

## 故障排除

### 常见问题

#### 1. 找不到编码器
```
找不到Rust编码器，请运行: cargo build --release
找不到Shine编码器，请构建Shine
```
**解决**: 确保两个编码器都已正确构建

#### 2. 找不到音频文件
```
在tests/audio目录中找不到WAV文件
```
**解决**: 确保 `tests/audio/` 目录下有WAV格式的测试文件

#### 3. 编码成功率过低
```
Rust编码器成功率过低: 5/10
```
**解决**: 检查编码器实现，可能存在算法问题

#### 4. 文件大小差异过大
```
平均文件大小差异过大: 15.30%
```
**解决**: 检查算法实现的一致性，特别是量化和比特池管理

### 调试建议

1. **使用详细输出**: 添加 `-- --nocapture` 查看详细测试过程
2. **检查单个文件**: 查看具体哪些文件编码失败或差异较大
3. **对比输出文件**: 使用十六进制编辑器对比生成的MP3文件
4. **检查编码器日志**: 查看编码器的错误输出

## 集成到CI/CD

这个测试可以集成到持续集成流程中：

```yaml
# GitHub Actions示例
- name: Run batch encoder comparison tests
  run: |
    cargo build --release
    cd ref/shine && ./build.ps1 && cd ../..
    cargo test integration_batch_encoder_comparison -- --nocapture
```

## 扩展性

### 添加新的测试配置
可以轻松添加新的测试用例：
```rust
#[test]
fn test_batch_encoding_comparison_mono() {
    let stats = run_batch_comparison_test(Some(128), false, true); // 添加mono参数
    // 断言...
}
```

### 添加新的统计指标
可以扩展 `BatchTestStats` 结构体添加更多分析指标：
- 编码质量评估
- 频谱分析对比
- 心理声学模型差异

这个集成测试为验证MP3编码器的正确性和性能提供了全面的自动化测试框架。