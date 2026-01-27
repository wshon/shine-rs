# 调试工作流程指南

## 核心调试原则

### MDCT系数验证流程
- **问题识别**: 当MDCT测试失败时，首先确定是混叠减少前还是混叠减少后的系数问题
- **数据收集**: 使用Shine参考实现的JSON调试模式收集准确的参考数据
- **逐步验证**: 分别验证混叠减少前后的系数，确保算法在每个步骤都与Shine一致

### 测试数据生成和更新

#### Shine调试输出配置
```bash
# 启用JSON调试模式和帧数限制
$env:SHINE_JSON_DEBUG="1"
$env:SHINE_MAX_FRAMES="3"

# 运行Shine编码器生成调试数据
.\shineenc.exe -b 128 "path\to\audio.wav" output.mp3
```

#### 重新构建Shine编码器
```bash
# Windows环境下重新构建Shine
cd ref/shine
.\build.ps1
```

#### 生成完整测试数据集
```bash
# 运行Python脚本生成所有测试数据
python scripts\generate_reference_data.py
```

### 测试执行命令

#### 运行特定集成测试
```bash
# 运行完整编码管道测试（需要diagnostics特性）
cargo test test_complete_encoding_pipeline --features diagnostics --verbose

# 运行所有集成管道验证测试
cargo test integration_pipeline_validation --features diagnostics

# 运行测试并显示详细输出
cargo test --features diagnostics -- --nocapture test_complete_encoding_pipeline
```

#### 诊断和静态检查
```bash
# 检查编译警告和错误
cargo check

# 运行Clippy静态分析
cargo clippy

# 使用getDiagnostics工具检查特定文件
# (在IDE中使用getDiagnostics工具)
```

## 调试数据结构更新流程

### 1. 识别数据结构变更需求
- 当发现测试数据缺少关键信息时（如混叠减少后的系数）
- 需要在Rust和Shine两端同步更新数据收集逻辑

### 2. 更新Shine调试输出
在`ref/shine/src/lib/l3mdct.c`中添加新的JSON调试输出：
```c
// 在混叠减少后添加调试输出
if (json_debug) {
    printf("{\"type\":\"mdct_coeff_after_aliasing\",\"frame\":%d,\"band\":%d,\"k\":%d,\"value\":%d}\n", 
           frame_count, 0, 17, mdct_enc[0][17]);
    // ... 其他系数
}
```

### 3. 更新Python解析脚本
在`scripts/generate_reference_data.py`中更新数据结构：
```python
# 更新MdctData结构以包含混叠前后的系数
'mdct_coefficients': {
    'coefficients_before_aliasing': [0, 0, 0],
    'coefficients_after_aliasing': [0, 0, 0],
    'l3_sb_sample': [0]
}
```

### 4. 更新Rust数据结构
在`crate/src/diagnostics_data.rs`中同步更新：
```rust
pub struct MdctData {
    pub coefficients_before_aliasing: Vec<i32>,
    pub coefficients_after_aliasing: Vec<i32>,
    pub l3_sb_sample: Vec<i32>,
}
```

## 常见问题排查

### 量化参数数据收集问题
- **症状**: TestDataCollector中的global_gain值与量化循环中记录的值不一致
- **原因**: 数据被多个channel/granule处理覆盖，或在错误的时机收集
- **解决**: 
  1. 只在ch=0, gr=0时收集数据
  2. 在shine_resv_frame_end之后收集最终值
  3. 保存易被覆盖的变量（如xrmax）

### Part2_3_length不匹配问题
- **症状**: 实际值比参考值小，通常差异为100-200
- **原因**: 缺少reservoir的stuffing bits调整
- **解决**: 在shine_resv_frame_end调用后收集part2_3_length的最终值

### 全局增益计算错误
- **症状**: global_gain值不正确，通常是quantizer_step_size计算问题
- **原因**: 
  1. 使用了错误的quantizer_step_size值
  2. 在错误的时机计算global_gain
- **解决**: 严格按照Shine的顺序：reservoir调整 → 设置global_gain

### xrmax值被覆盖问题
- **症状**: 记录的xrmax值与预期不符，通常是其他channel/granule的值
- **原因**: config.l3loop.xrmax在处理不同channel/granule时被覆盖
- **解决**: 在处理第一个channel/granule时保存xrmax值

### 哈希值不匹配问题
- **症状**: 测试显示哈希值不匹配，但值看起来相同（大小写差异）
- **原因**: 通常是字符串格式化的大小写问题
- **解决**: 检查哈希值比较逻辑，确保大小写一致性

### 编译特性问题
- **症状**: `diagnostics_data`模块找不到
- **原因**: 缺少`diagnostics`特性
- **解决**: 在测试命令中添加`--features diagnostics`

### 测试数据版本不匹配
- **症状**: 测试期望的数据结构与实际不符
- **原因**: 测试数据文件使用旧格式
- **解决**: 重新运行`python scripts\generate_reference_data.py`

## 量化参数验证调试流程

### 问题识别和诊断
当量化参数测试失败时，按以下步骤进行调试：

#### 1. 运行集成测试获取详细错误信息
```bash
# 运行量化验证测试，显示详细输出
cargo test test_encoding_validation_all_files --features diagnostics -- --nocapture

# 或运行特定的量化一致性测试
cargo test test_quantization_encoding_consistency --features diagnostics -- --nocapture
```

#### 2. 分析错误类型
常见的量化参数不匹配错误：
- `Global gain mismatch`: 全局增益计算错误
- `Part2_3_length mismatch`: 部分2+3长度不匹配（通常是reservoir调整问题）
- `Max bits mismatch`: 最大比特数计算错误
- `xrmax mismatch`: 最大频谱值不匹配（通常是数据收集时机问题）

#### 3. 启用调试输出分析数据流
```bash
# 设置调试环境变量
$env:RUST_MP3_DEBUG_FRAMES="6"

# 运行测试查看详细的调试输出
cargo test test_encoding_validation_all_files --features diagnostics -- --nocapture
```

### 数据收集时机问题调试

#### 问题症状
- TestDataCollector中的值与量化循环中记录的值不一致
- 不同channel/granule的数据相互覆盖
- reservoir调整前后的值不匹配

#### 调试方法
1. **检查数据收集的channel和granule限制**：
```rust
// 确保只在第一个channel和granule收集数据
if ch == 0 && gr == 0 {
    crate::diagnostics_data::record_quant_data(/* ... */);
}
```

2. **验证reservoir调整时机**：
```rust
// 在shine_resv_frame_end之后收集最终数据
crate::reservoir::shine_resv_frame_end(config);
// 然后记录最终的part2_3_length值
```

3. **保存易被覆盖的值**：
```rust
// 在处理第一个channel/granule时保存xrmax
#[cfg(feature = "diagnostics")]
let mut saved_xrmax = 0i32;
if ch == 0 && gr == 0 {
    saved_xrmax = config.l3loop.xrmax;
}
```

### 全局增益计算问题调试

#### Shine参考公式验证
```c
// Shine中的全局增益计算（ref/shine/src/lib/l3loop.c:200）
cod_info->global_gain = cod_info->quantizerStepSize + 210;
```

#### Rust实现验证
```rust
// 确保计算顺序与Shine一致
cod_info.global_gain = (quantizer_step_size + 210) as u32;
```

#### 调试命令
```bash
# 查看全局增益计算的详细过程
cargo test test_encoding_validation_all_files --features diagnostics -- --nocapture | grep "global_gain"
```

### Part2_3_length调试流程

#### 问题根源
Part2_3_length在两个地方被修改：
1. 量化循环：`part2_3_length = part2_length + bits`
2. Reservoir调整：`part2_3_length += stuffingBits`

#### 调试步骤
1. **检查Shine的调用顺序**：
```bash
# 查看Shine的part2_3_length输出时机
cd ref/shine
$env:SHINE_JSON_DEBUG="1"
$env:SHINE_MAX_FRAMES="3"
.\shineenc.exe -b 128 "audio.wav" output.mp3
```

2. **验证Rust的调用顺序**：
```rust
// 确保在所有reservoir调整完成后记录数据
crate::reservoir::shine_resv_frame_end(config);
// 然后记录最终值
let cod_info = &config.side_info.gr[0].ch[0].tt;
record_quant_data(/* 使用cod_info.part2_3_length */);
```

3. **对比中间值和最终值**：
```bash
# 查看part2_3_length的变化过程
cargo test --features diagnostics -- --nocapture | grep "part2_3_length"
```

### 测试数据一致性验证

#### 验证命令序列
```bash
# 1. 重新生成Shine参考数据
cd ref/shine
.\build.ps1
cd ../..
python scripts\generate_reference_data.py

# 2. 运行完整的验证测试
cargo test integration_pipeline_validation --features diagnostics

# 3. 检查特定参数的一致性
cargo test test_quantization_encoding_consistency --features diagnostics -- --nocapture
```

#### 数据收集验证
```bash
# 验证TestDataCollector是否正确收集数据
cargo test --features diagnostics -- --nocapture | grep "Recording:"
cargo test --features diagnostics -- --nocapture | grep "Using TestDataCollector"
```

## 验证检查清单

### MDCT算法验证
- [x] 混叠减少前的系数与Shine完全一致
- [x] 混叫减少后的系数与Shine完全一致  
- [x] l3_sb_sample值与Shine完全一致
- [x] MDCT输入数据与Shine完全一致

### 量化算法验证
- [x] 全局增益计算与Shine完全一致
- [x] Part2_3_length包含reservoir调整
- [x] xrmax值正确保存和传递
- [x] quantizer_step_size计算正确
- [x] 数据收集时机正确（在reservoir调整后）

### 测试数据完整性
- [x] 所有测试文件包含最新的数据结构
- [x] JSON格式正确且可解析
- [x] 参考哈希值准确
- [x] 帧数据完整（通常3-6帧）
- [x] TestDataCollector正确收集和存储数据

### 代码质量检查
- [ ] 无编译警告
- [ ] 通过Clippy检查
- [x] 所有核心算法测试通过
- [x] 调试输出清晰可读

## 性能优化注意事项

### 测试执行效率
- 使用`--features diagnostics`仅在需要时启用诊断功能
- 限制测试帧数以减少执行时间
- 使用`--verbose`仅在调试时启用详细输出

### 数据收集优化
- 仅收集前6帧的数据以平衡准确性和性能
- 使用条件编译确保诊断代码不影响生产性能
- 及时清理临时调试文件

## 文档更新要求

### 代码注释
- 在关键算法点添加与Shine对应关系的注释
- 记录调试输出的用途和格式
- 说明测试数据的生成方法

### 测试文档
- 更新测试数据格式说明
- 记录新增的验证点
- 维护调试命令参考

## 成功案例记录

### 量化参数验证问题解决案例 (2026-01-27)

#### 问题描述
量化参数验证测试失败，出现以下错误：
- `Global gain mismatch: actual=210, reference=173`
- `Part2_3_length mismatch: actual=763, reference=915`
- `xrmax mismatch: actual=543987899, reference=174601576`

#### 调试过程
1. **识别数据收集时机问题**：
   - 发现TestDataCollector中的值与量化循环中的调试输出不一致
   - 通过调试输出确认量化循环计算正确，但数据收集有问题

2. **分析Shine的调用顺序**：
   - 查看`ref/shine/src/lib/l3loop.c`确认正确的调用顺序
   - 发现Shine在`shine_ResvFrameEnd`后输出最终的part2_3_length值

3. **修复数据收集逻辑**：
   - 将数据收集移到`shine_resv_frame_end`之后
   - 只在ch=0, gr=0时收集数据，避免覆盖
   - 保存xrmax值避免被后续处理覆盖

#### 关键修复代码
```rust
// 在量化循环结束后，shine_resv_frame_end之后收集数据
crate::reservoir::shine_resv_frame_end(config);

#[cfg(feature = "diagnostics")]
{
    if frame_num <= debug_frames {
        let cod_info = &config.side_info.gr[0].ch[0].tt;
        let max_bits = crate::reservoir::shine_max_reservoir_bits(&config.pe[0][0], &config);
        
        crate::diagnostics_data::record_quant_data(
            saved_xrmax,  // 使用保存的xrmax值
            max_bits,
            cod_info.part2_3_length,  // 最终的part2_3_length值
            cod_info.quantizer_step_size,
            cod_info.global_gain
        );
    }
}
```

#### 验证结果
修复后所有量化参数完全匹配：
- ✅ Frame 1: xrmax=174601576, part2_3_length=915, global_gain=170
- ✅ Frame 2: xrmax=761934185, part2_3_length=824, global_gain=175  
- ✅ Frame 3: xrmax=398722265, part2_3_length=936, global_gain=173

#### 经验总结
1. **严格遵循Shine的调用顺序**：数据收集必须在所有算法步骤完成后进行
2. **避免数据覆盖**：在多channel/granule处理中要小心数据被覆盖
3. **验证中间步骤**：通过调试输出确认每个步骤的正确性
4. **参考Shine源码**：遇到问题时优先查看Shine的实现逻辑

#### 相关命令
```bash
# 运行验证测试
cargo test test_encoding_validation_all_files --features diagnostics -- --nocapture

# 查看特定参数的调试输出
cargo test --features diagnostics -- --nocapture | grep "Recording:"
cargo test --features diagnostics -- --nocapture | grep "global_gain"
```