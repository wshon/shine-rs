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

## 验证检查清单

### MDCT算法验证
- [ ] 混叠减少前的系数与Shine完全一致
- [ ] 混叫减少后的系数与Shine完全一致  
- [ ] l3_sb_sample值与Shine完全一致
- [ ] MDCT输入数据与Shine完全一致

### 测试数据完整性
- [ ] 所有测试文件包含最新的数据结构
- [ ] JSON格式正确且可解析
- [ ] 参考哈希值准确
- [ ] 帧数据完整（通常3-6帧）

### 代码质量检查
- [ ] 无编译警告
- [ ] 通过Clippy检查
- [ ] 所有测试通过
- [ ] 调试输出清晰可读

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