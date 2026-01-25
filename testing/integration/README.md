# MP3编码器测试套件

本目录包含了基于真实数据的完整MP3编码流水线验证测试。

## 测试文件概述

### `integration_full_pipeline_validation.rs`
基于 `testing/fixtures/audio/sample-3s.wav` 前三帧真实编码数据的完整流水线验证测试。

#### 测试数据来源
所有测试数据都来自实际的编码会话，通过运行以下命令获取：
```bash
cargo run --bin wav2mp3 -- testing/fixtures/audio/sample-3s.wav test_sample_3s_real_data.mp3
```

#### 测试覆盖范围

**1. 子带滤波器验证** (`test_subband_filter_frame_1_validation`)
- 验证子带分析滤波器的输出
- 测试第一帧的l3_sb_sample数组前8个频带的值
- 确保子带样本在合理范围内

**2. MDCT输入数据验证** (`test_mdct_input_data_validation`)
- 验证MDCT输入数据的正确性
- 测试帧间数据传递机制（granule overlap）
- Frame 1: 第一个granule应为零（无前序数据）
- Frame 2/3: 应使用前一帧保存的数据作为输入

**3. MDCT系数验证** (`test_mdct_coefficients_all_frames_validation`)
- 验证所有三帧的MDCT系数计算
- 测试K17、K16、K15系数的具体数值
- 确保系数在合理范围内且帧间有变化

**4. 量化参数验证** (`test_quantization_parameters_all_frames_validation`)
- 验证xrmax、global_gain、big_values等量化参数
- 测试所有参数都在MP3标准范围内
- 验证参数在不同帧间的合理变化

**5. SCFSI计算验证** (`test_scfsi_calculation_all_frames_validation`)
- 验证Scale Factor Selection Information的计算
- 测试所有三帧的SCFSI值：
  - Frame 1: [0,1,0,1] - 交替模式
  - Frame 2: [1,1,1,1] - 全部重用前一帧
  - Frame 3: [0,1,1,1] - 第一频带重新计算

**6. 比特流参数验证** (`test_bitstream_frame_parameters_validation`)
- 验证帧大小：Frame 1=416字节，Frame 2=420字节，Frame 3=416字节
- 总计1252字节（前三帧）
- 验证padding决策和bits_per_frame一致性

**7. Slot Lag机制验证** (`test_slot_lag_mechanism_validation`)
- 验证CBR编码中的slot lag计算
- 测试帧间slot lag的连续性
- 验证每帧的slot lag增量（~0.040816）

**8. Part2_3_length和Count1验证** (`test_part2_3_length_validation`)
- 验证Huffman编码数据长度
- 测试count1值（四元组计数）
- 确保所有值在MP3标准范围内

**9. 通道一致性验证** (`test_channel_consistency_all_frames`)
- 验证立体声编码中左右声道参数的一致性
- 测试所有三帧的通道间参数匹配

**10. MP3格式合规性验证** (`test_mp3_format_compliance`)
- 验证MPEG版本、层、采样率等格式参数
- 确保符合MP3标准规范

**11. 颗粒参数关系验证** (`test_granule_parameter_relationships`)
- 验证同一帧内不同granule间的参数关系
- 测试复杂度递增趋势

**12. 数学属性验证** (`test_encoding_pipeline_mathematical_properties`)
- 验证编码流水线中的数学关系
- 测试xrmax与global_gain的相关性
- 验证big_values和count1的系数总数限制

## 真实数据常量

### Frame 1 数据
```rust
// 子带滤波器输出
L3_SB_SAMPLE_CH0_GR1_FIRST_8: [1490, 647, 269, 691, 702, -204, -837, -291]

// MDCT系数
MDCT_COEFF_BAND_0_K17: 808302
MDCT_COEFF_BAND_0_K16: 3145162  
MDCT_COEFF_BAND_0_K15: 6527797

// 量化参数
XRMAX_CH0_GR0: 174601576
GLOBAL_GAIN_CH0_GR0: 170
BIG_VALUES_CH0_GR0: 94

// SCFSI值
SCFSI_CH0: [0, 1, 0, 1]
```

### Frame 2 数据
```rust
// MDCT系数
MDCT_COEFF_BAND_0_K17: -17369047
MDCT_COEFF_BAND_0_K16: 13912238
MDCT_COEFF_BAND_0_K15: 31910201

// 量化参数
XRMAX_CH0_GR0: 761934185
GLOBAL_GAIN_CH0_GR0: 175

// SCFSI值
SCFSI_CH0: [1, 1, 1, 1]
```

### Frame 3 数据
```rust
// MDCT系数
MDCT_COEFF_BAND_0_K17: -20877153
MDCT_COEFF_BAND_0_K16: -19736998
MDCT_COEFF_BAND_0_K15: -24380058

// 量化参数
XRMAX_CH0_GR0: 398722265
GLOBAL_GAIN_CH0_GR0: 173

// SCFSI值
SCFSI_CH0: [0, 1, 1, 1]
```

## 运行测试

```bash
# 运行完整的流水线验证测试
cargo test --test integration_full_pipeline_validation

# 运行特定测试
cargo test test_mdct_coefficients_all_frames_validation

# 运行所有测试（包括单元测试）
cargo test
```

## 测试意义

这些测试确保了：

1. **算法正确性**: 每个编码步骤都产生预期的输出
2. **数据一致性**: 帧间数据传递正确，无数据损坏
3. **标准合规性**: 所有参数都符合MP3标准限制
4. **回归防护**: 防止未来修改破坏现有功能
5. **调试支持**: 提供详细的中间数据用于问题定位

通过使用真实的编码数据，这些测试提供了比合成数据更可靠的验证，确保编码器在实际使用场景中的正确性。