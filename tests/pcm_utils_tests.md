# pcm_utils_tests.rs 测试文档

## 测试概述

这个测试套件专门测试PCM（脉冲编码调制）数据处理工具函数，主要包括去交错（deinterleaving）和格式转换工具。这些工具函数是音频处理管道中的重要组成部分。

## 测试目标

- **去交错功能验证**: 确保立体声PCM数据正确分离为左右声道
- **数据格式处理**: 验证不同PCM数据格式的正确处理
- **边界条件测试**: 确保在各种边界情况下函数行为正确
- **性能验证**: 验证大数据量处理的正确性和效率

## 测试函数详解

### `test_deinterleave_non_interleaved_stereo()`
**目的**: 测试非交错立体声数据的去交错处理

**运行方式**:
```bash
cargo test test_deinterleave_non_interleaved_stereo --test pcm_utils_tests -- --nocapture
```

**测试数据**:
```rust
// 输入: [1, 2, 3, 4, 5, 6] 
// 格式: L: [1,2,3], R: [4,5,6] (非交错格式)
// 预期输出: 左声道 [1,2,3], 右声道 [4,5,6]
```

**验证内容**:
- 左声道数据正确提取
- 右声道数据正确提取
- 数据顺序保持正确

### `test_deinterleave_interleaved_stereo()`
**目的**: 测试交错立体声数据的去交错处理

**运行方式**:
```bash
cargo test test_deinterleave_interleaved_stereo --test pcm_utils_tests -- --nocapture
```

**测试数据**:
```rust
// 输入: [1, 4, 2, 5, 3, 6]
// 格式: L1,R1,L2,R2,L3,R3 (交错格式)
// 预期输出: 左声道 [1,2,3], 右声道 [4,5,6]
```

**验证内容**:
- 交错数据正确分离
- 左右声道数据完整性
- 样本顺序正确性

### `test_deinterleave_mono()`
**目的**: 测试单声道数据处理

**运行方式**:
```bash
cargo test test_deinterleave_mono --test pcm_utils_tests -- --nocapture
```

**测试数据**:
```rust
// 输入: [1, 2, 3, 4]
// 格式: 单声道
// 预期输出: 单声道 [1,2,3,4]
```

**验证内容**:
- 单声道数据完整复制
- 无数据丢失或重复

### `test_deinterleave_partial_data()`
**目的**: 测试部分数据的处理能力

**运行方式**:
```bash
cargo test test_deinterleave_partial_data --test pcm_utils_tests -- --nocapture
```

**测试场景**:
- 输入数据少于预期长度
- 不完整的立体声对

**验证内容**:
- 函数不会崩溃
- 正确处理可用数据
- 优雅处理数据不足情况

### `test_deinterleave_empty_data()`
**目的**: 测试空数据输入的处理

**运行方式**:
```bash
cargo test test_deinterleave_empty_data --test pcm_utils_tests -- --nocapture
```

**测试场景**:
- 空的PCM数据向量
- 零长度输入

**验证内容**:
- 函数不会崩溃
- 输出缓冲区保持空状态
- 正确处理边界条件

### `test_deinterleave_large_data()`
**目的**: 测试大数据量的处理性能和正确性

**运行方式**:
```bash
cargo test test_deinterleave_large_data --test pcm_utils_tests -- --nocapture
```

**测试数据**:
- 2048个立体声样本对（4096个总样本）
- 左声道: 0, 1, 2, ...
- 右声道: 10000, 10001, 10002, ...

**验证内容**:
- 大数据量处理正确性
- 性能不会显著下降
- 内存使用合理
- 数据完整性保持

### `test_deinterleave_boundary_values()`
**目的**: 测试边界值的处理

**运行方式**:
```bash
cargo test test_deinterleave_boundary_values --test pcm_utils_tests -- --nocapture
```

**测试数据**:
```rust
// 输入: [i16::MIN, i16::MAX, 0, -1]
// 包含16位整数的最小值、最大值、零和-1
```

**验证内容**:
- 正确处理16位整数边界值
- 无溢出或下溢
- 数据类型转换正确

### `test_deinterleave_single_sample()`
**目的**: 测试单个样本的处理

**运行方式**:
```bash
cargo test test_deinterleave_single_sample --test pcm_utils_tests -- --nocapture
```

**测试场景**:
- 每个声道只有一个样本
- 最小有效数据量

**验证内容**:
- 单样本正确处理
- 无额外数据生成
- 基本功能完整性

### `test_deinterleave_buffer_reuse()`
**目的**: 测试缓冲区重用的正确性

**运行方式**:
```bash
cargo test test_deinterleave_buffer_reuse --test pcm_utils_tests -- --nocapture
```

**测试流程**:
1. 第一次去交错操作
2. 清空缓冲区
3. 第二次去交错操作
4. 验证结果独立性

**验证内容**:
- 缓冲区正确清理
- 无残留数据影响
- 重用功能正常

## 运行测试

### 运行所有PCM工具测试
```bash
cargo test --test pcm_utils_tests -- --nocapture
```

### 运行特定测试
```bash
cargo test test_deinterleave_interleaved_stereo --test pcm_utils_tests -- --nocapture
```

### 运行性能相关测试
```bash
cargo test test_deinterleave_large_data --test pcm_utils_tests -- --nocapture
```

## 故障排除

### 常见问题

#### 1. 数据长度不匹配
**症状**: 输出缓冲区长度与预期不符
**原因**: 
- 输入数据长度计算错误
- 声道数参数不正确
- 样本数参数错误

**解决**:
```rust
// 确保参数正确
let samples_per_channel = total_samples / channels;
deinterleave_pcm_interleaved(&pcm_data, channels, samples_per_channel, &mut buffers);
```

#### 2. 交错格式理解错误
**症状**: 左右声道数据混乱
**原因**: 混淆了交错和非交错格式

**格式说明**:
```rust
// 交错格式 (Interleaved): [L0, R0, L1, R1, L2, R2, ...]
// 非交错格式 (Non-interleaved): [L0, L1, L2, ...], [R0, R1, R2, ...]
```

#### 3. 缓冲区未正确初始化
**症状**: 输出缓冲区包含意外数据
**解决**:
```rust
// 确保缓冲区正确初始化
let mut buffers = vec![Vec::new(); channels];
// 或者清空现有缓冲区
for buffer in &mut buffers {
    buffer.clear();
}
```

#### 4. 边界值处理问题
**症状**: 在极值输入时出现错误
**解决**: 确保函数正确处理`i16::MIN`和`i16::MAX`

### 调试技巧

#### 1. 打印中间结果
```rust
println!("Input data: {:?}", &pcm_data[..std::cmp::min(10, pcm_data.len())]);
println!("Left channel: {:?}", &buffers[0][..std::cmp::min(5, buffers[0].len())]);
println!("Right channel: {:?}", &buffers[1][..std::cmp::min(5, buffers[1].len())]);
```

#### 2. 验证数据完整性
```rust
// 验证总样本数
let total_output_samples: usize = buffers.iter().map(|b| b.len()).sum();
assert_eq!(total_output_samples, expected_total_samples);
```

#### 3. 检查内存使用
```rust
// 监控大数据测试的内存使用
let initial_capacity = buffers[0].capacity();
// ... 执行测试
let final_capacity = buffers[0].capacity();
println!("Buffer capacity change: {} -> {}", initial_capacity, final_capacity);
```

## 性能考虑

### 基准测试
- **小数据** (< 100样本): < 1ms
- **中等数据** (1000-10000样本): < 10ms  
- **大数据** (> 10000样本): < 100ms

### 内存效率
- 避免不必要的数据复制
- 重用缓冲区以减少分配
- 预分配已知大小的缓冲区

### 优化建议
```rust
// 预分配缓冲区容量
let mut buffers = vec![Vec::with_capacity(samples_per_channel); channels];

// 批量处理而非逐样本处理
// 使用迭代器和切片操作提高效率
```

## 测试数据模式

### 简单测试数据
```rust
// 小规模，易于验证的数据
let pcm_data = vec![1, 2, 3, 4, 5, 6];
```

### 模式化测试数据
```rust
// 有规律的数据，便于验证算法正确性
let mut pcm_data = Vec::new();
for i in 0..samples {
    pcm_data.push(i as i16);           // 左声道
    pcm_data.push((i + offset) as i16); // 右声道
}
```

### 边界测试数据
```rust
// 包含极值的数据
let pcm_data = vec![i16::MIN, i16::MAX, 0, -1, 1];
```

## 已知问题

### 1. 大数据性能
- **状态**: 当前实现对大数据处理效率可接受
- **改进**: 可考虑SIMD优化或并行处理

### 2. 错误处理
- **状态**: 当前主要依赖断言进行验证
- **改进**: 可增加更详细的错误报告

### 3. 内存分配
- **状态**: 每次调用可能重新分配缓冲区
- **改进**: 考虑缓冲区池或预分配策略

## 维护指南

### 添加新测试
1. 确定测试场景（正常/边界/错误）
2. 设计合适的测试数据
3. 编写清晰的验证逻辑
4. 添加适当的文档注释

### 性能测试
1. 定期运行大数据测试
2. 监控执行时间变化
3. 分析内存使用模式
4. 识别性能瓶颈

### 代码覆盖率
1. 确保所有分支都有测试覆盖
2. 测试各种输入组合
3. 验证错误处理路径

## 成功标准

- **功能正确性**: 所有去交错操作产生正确结果
- **边界安全性**: 极值和边界条件得到正确处理
- **性能达标**: 大数据处理在可接受时间内完成
- **内存效率**: 无内存泄漏，合理的内存使用
- **代码健壮性**: 各种输入情况下不会崩溃