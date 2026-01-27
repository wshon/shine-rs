# integration_scfsi_consistency.rs 测试文档

## 测试概述

这个测试套件专门验证SCFSI（Scale Factor Selection Information，标量因子选择信息）的一致性，确保Rust MP3编码器在SCFSI计算和编码方面与Shine参考实现产生完全相同的输出。

## 测试目标

- **SCFSI一致性验证**: 确保SCFSI计算与Shine完全一致
- **二进制输出匹配**: 验证生成的MP3文件与Shine参考文件完全相同
- **算法正确性**: 验证SCFSI决策逻辑和条件计算
- **版本兼容性**: 确保SCFSI仅在MPEG-I版本中计算

## 测试常量

```rust
const EXPECTED_SHINE_HASH: &str = "4385b617a86cb3891ce3c99dabe6b47c2ac9182b32c46cbc5ad167fb28b959c4";
const EXPECTED_FILE_SIZE: u64 = 2508;
const TEST_FRAME_COUNT: &str = "6";
```

## 测试函数详解

### 1. `test_scfsi_consistency_with_shine()`
**目的**: 验证Rust编码器与Shine参考实现的完全一致性

**运行方式**:
```bash
cargo test test_scfsi_consistency_with_shine -- --nocapture
```

**验证内容**:
- 使用预保存的参考文件进行比较
- 验证参考文件完整性（SHA256哈希和文件大小）
- 运行Rust编码器生成6帧输出
- 比较文件大小和SHA256哈希值
- 执行字节级详细比较（如果哈希不匹配）

**依赖文件**:
- `tests/audio/sample-3s.wav` - 输入音频文件
- `tests/audio/shine_reference_6frames.mp3` - Shine参考输出

**环境变量**: `RUST_MP3_MAX_FRAMES=6`

### 2. `test_scfsi_version_check()`
**目的**: 验证SCFSI仅在MPEG-I版本中计算

**运行方式**:
```bash
cargo test test_scfsi_version_check -- --nocapture
```

**验证内容**:
- MPEG-I（版本3）应该计算SCFSI
- MPEG-II（版本2）不应该计算SCFSI
- 验证版本检查逻辑正确性

### 3. `test_frame_limit_configuration()`
**目的**: 测试帧数限制环境变量的配置

**运行方式**:
```bash
cargo test test_frame_limit_configuration -- --nocapture
```

**验证内容**:
- 测试不同帧数限制（3、6、10帧）
- 验证输出文件大小符合预期
- 确认环境变量正确生效

**预期文件大小**:
- 3帧: 1252字节
- 6帧: 2508字节
- 10帧: 4180字节

### 4. `test_scfsi_band_calculation()`
**目的**: 验证SCFSI频带计算逻辑

**运行方式**:
```bash
cargo test test_scfsi_band_calculation -- --nocapture
```

**验证内容**:
- SCFSI频带边界常量验证
- SCFSI决策阈值验证
- SCFSI决策逻辑测试

**关键常量**:
```rust
const SCFSI_BAND_LONG: [i32; 5] = [0, 6, 11, 16, 21];
const EN_SCFSI_BAND_KRIT: i32 = 10;
const XM_SCFSI_BAND_KRIT: i32 = 10;
```

**决策逻辑**:
```rust
// 如果 sum0 < EN_SCFSI_BAND_KRIT && sum1 < XM_SCFSI_BAND_KRIT，则 SCFSI = 1
// 否则 SCFSI = 0
```

### 5. `test_scfsi_condition_calculation()`
**目的**: 测试SCFSI条件计算逻辑

**运行方式**:
```bash
cargo test test_scfsi_condition_calculation -- --nocapture
```

**验证内容**:
- 条件计算必须等于6才执行SCFSI计算
- 验证各个条件的贡献值
- 模拟Shine的条件计算逻辑

**条件计算逻辑**:
```rust
let mut condition = 0;
// 对于每个颗粒：
//   如果 xrmaxl[gr] != 0: condition += 1
//   condition += 1 (总是)
// 如果 |en_tot[0] - en_tot[1]| < EN_TOT_KRIT: condition += 1
// 如果 tp < EN_DIF_KRIT: condition += 1
// 当 condition == 6 时，执行SCFSI计算
```

### 6. `test_known_scfsi_values()`
**目的**: 记录已知的SCFSI值作为参考

**运行方式**:
```bash
cargo test test_known_scfsi_values -- --nocapture
```

**验证内容**:
- 记录调试会话中验证的SCFSI值
- 确保SCFSI值在有效范围内（0或1）
- 作为预期行为的文档

**已知SCFSI值**:
```rust
// Frame 1: ch0=[0,1,0,1], ch1=[0,1,0,1]
// Frame 2: ch0=[1,1,1,1], ch1=[1,1,1,1]  
// Frame 3: ch0=[0,1,1,1], ch1=[0,1,1,1]
```

## 属性测试

### `test_scfsi_decision_properties()`
**目的**: 使用属性测试验证SCFSI决策逻辑

**测试范围**: sum0, sum1 ∈ [0, 200)

**验证属性**:
- SCFSI值必须是二进制（0或1）
- 当两个和都低于阈值时，SCFSI应为1
- 当任一和达到或超过阈值时，SCFSI应为0

### `test_condition_calculation_properties()`
**目的**: 验证条件计算的属性

**测试范围**: 
- xrmaxl0, xrmaxl1 ∈ [0, 1000000)
- en_tot_diff ∈ [0, 50)
- tp ∈ [0, 200)

**验证属性**:
- 条件值应在[2, 6]范围内
- 当所有条件满足时，条件值应为6

## 运行所有测试

```bash
# 运行所有SCFSI一致性测试
cargo test --test integration_scfsi_consistency -- --nocapture

# 运行特定测试
cargo test test_scfsi_consistency_with_shine --test integration_scfsi_consistency -- --nocapture

# 运行属性测试
cargo test property_tests --test integration_scfsi_consistency -- --nocapture
```

## 故障排除

### 常见问题

#### 1. 参考文件哈希不匹配
**症状**: "Reference file hash mismatch - file may be corrupted"
**原因**: 参考文件损坏或版本不匹配
**解决**: 重新生成参考文件
```bash
cd ref/shine
$env:SHINE_MAX_FRAMES="6"
.\shineenc.exe -b 128 "../../tests/audio/sample-3s.wav" "../../tests/audio/shine_reference_6frames.mp3"
```

#### 2. 输出哈希不匹配
**症状**: "SHA256 hash mismatch"
**原因**: SCFSI计算实现与Shine不一致
**解决**:
1. 查看`ref/shine/src/lib/l3loop.c`中的`calc_scfsi`函数
2. 对比Rust实现中的SCFSI计算逻辑
3. 确保条件计算和决策逻辑完全一致

#### 3. 文件大小不匹配
**症状**: 输出文件大小与预期不符
**原因**: 帧数限制未正确应用或编码参数不匹配
**解决**:
1. 确认`RUST_MP3_MAX_FRAMES`环境变量设置
2. 验证编码配置（比特率、采样率等）
3. 检查帧处理逻辑

#### 4. 测试文件缺失
**症状**: "Test input file not found" 或 "Shine reference file not found"
**解决**: 确保以下文件存在：
- `tests/audio/sample-3s.wav`
- `tests/audio/shine_reference_6frames.mp3`

## SCFSI算法详解

### SCFSI的作用
SCFSI用于在连续帧之间共享标量因子，以提高编码效率。当相邻帧的频谱特性相似时，可以重用前一帧的标量因子。

### 计算步骤
1. **版本检查**: 仅在MPEG-I中计算SCFSI
2. **条件评估**: 计算6个条件，全部满足时才进行SCFSI计算
3. **频带分析**: 对每个SCFSI频带计算能量和差异
4. **决策**: 基于阈值比较决定是否使用SCFSI

### 关键阈值
- `EN_SCFSI_BAND_KRIT = 10`: SCFSI频带能量阈值
- `XM_SCFSI_BAND_KRIT = 10`: SCFSI频带最大值阈值
- `EN_TOT_KRIT = 10`: 总能量差异阈值
- `EN_DIF_KRIT = 100`: 能量差异阈值

## 已知问题

### 1. 字节级比较性能
- **状态**: 仅在哈希不匹配时执行
- **影响**: 可能增加调试时间
- **改进**: 考虑添加更详细的差异报告

### 2. 属性测试覆盖范围
- **状态**: 当前覆盖基本场景
- **改进**: 可以增加边界条件和极值测试

### 3. 参考文件依赖
- **状态**: 依赖预生成的参考文件
- **风险**: 文件损坏或丢失
- **改进**: 考虑自动生成和验证机制

## 维护指南

### 更新参考文件
1. 确保Shine编码器是最新版本
2. 使用相同的输入文件和参数重新生成
3. 更新测试中的哈希值和文件大小常量

### 添加新测试场景
1. 添加不同的音频文件测试
2. 测试不同的编码参数组合
3. 验证边界条件和异常情况

### 性能优化
1. 监控测试执行时间
2. 优化字节级比较逻辑
3. 考虑并行化属性测试

## 成功标准

- **完全二进制匹配**: Rust输出与Shine参考文件完全相同
- **SCFSI值正确**: 所有帧的SCFSI值与预期一致
- **条件计算正确**: 条件评估逻辑与Shine完全一致
- **版本兼容性**: 正确处理不同MPEG版本的SCFSI计算