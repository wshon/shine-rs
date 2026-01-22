# 需求文档

## 介绍

本项目旨在用 Rust 重写 shine MP3 编码器，实现一个纯粹的 MP3 编码库。该库将保持与原始 shine 库相似的架构和接口设计，以便后续同步新特性。项目专注于核心编码功能，不包含命令行工具。

## 术语表

- **MP3_Encoder**: 主要的 MP3 编码器结构体
- **Subband_Filter**: 子带滤波器模块
- **MDCT_Transform**: 修正离散余弦变换模块
- **Quantization_Loop**: 量化循环模块
- **Bitstream_Writer**: 比特流写入器
- **Huffman_Encoder**: 霍夫曼编码器
- **Config**: 编码配置结构体
- **PCM_Data**: 脉冲编码调制音频数据
- **MP3_Frame**: MP3 帧数据

## 需求

### 需求 1: 核心编码接口

**用户故事:** 作为开发者，我希望有一个简单的 MP3 编码接口，以便将 PCM 音频数据编码为 MP3 格式。

#### 验收标准

1. WHEN 提供有效的编码配置 THEN MP3_Encoder SHALL 成功初始化
2. WHEN 输入 PCM 音频数据 THEN MP3_Encoder SHALL 返回编码后的 MP3 数据
3. WHEN 编码完成 THEN MP3_Encoder SHALL 提供刷新方法输出剩余数据
4. THE MP3_Encoder SHALL 支持单声道和立体声编码
5. THE MP3_Encoder SHALL 支持标准的采样率 (44100, 48000, 32000, 22050, 24000, 16000, 11025, 12000, 8000 Hz)
6. THE MP3_Encoder SHALL 支持标准的比特率 (32-320 kbps)

### 需求 2: 子带滤波处理

**用户故事:** 作为编码器，我需要将 PCM 数据通过子带滤波器处理，以便进行频域分析。

#### 验收标准

1. WHEN 接收 PCM 数据 THEN Subband_Filter SHALL 将其分解为 32 个子带
2. THE Subband_Filter SHALL 使用多相滤波器实现
3. WHEN 处理立体声数据 THEN Subband_Filter SHALL 独立处理左右声道
4. THE Subband_Filter SHALL 保持与 shine 库相同的滤波器系数

### 需求 3: MDCT 变换处理

**用户故事:** 作为编码器，我需要对子带数据进行 MDCT 变换，以便获得频域系数。

#### 验收标准

1. WHEN 接收子带数据 THEN MDCT_Transform SHALL 执行修正离散余弦变换
2. THE MDCT_Transform SHALL 产生 576 个频域系数 (每个粒度)
3. WHEN 处理立体声数据 THEN MDCT_Transform SHALL 独立处理左右声道
4. THE MDCT_Transform SHALL 使用预计算的余弦表以提高性能

### 需求 4: 量化和循环优化

**用户故事:** 作为编码器，我需要对 MDCT 系数进行量化，以便控制比特率和音质。

#### 验收标准

1. WHEN 接收 MDCT 系数 THEN Quantization_Loop SHALL 执行非线性量化
2. THE Quantization_Loop SHALL 使用迭代算法找到最优量化步长
3. WHEN 量化后比特数超过目标 THEN Quantization_Loop SHALL 增加量化步长
4. WHEN 量化后比特数低于目标 THEN Quantization_Loop SHALL 减少量化步长
5. THE Quantization_Loop SHALL 支持比特储备池机制

### 需求 5: 霍夫曼编码

**用户故事:** 作为编码器，我需要对量化后的系数进行霍夫曼编码，以便实现无损压缩。

#### 验收标准

1. WHEN 接收量化系数 THEN Huffman_Encoder SHALL 使用标准 MP3 霍夫曼表编码
2. THE Huffman_Encoder SHALL 支持所有标准的霍夫曼码表 (0-31)
3. WHEN 编码大值 THEN Huffman_Encoder SHALL 使用转义序列
4. THE Huffman_Encoder SHALL 优化码表选择以最小化比特数

### 需求 6: 比特流生成

**用户故事:** 作为编码器，我需要将编码数据组装成标准的 MP3 比特流格式。

#### 验收标准

1. WHEN 生成 MP3 帧 THEN Bitstream_Writer SHALL 包含正确的帧头
2. THE Bitstream_Writer SHALL 支持 MPEG-1, MPEG-2, MPEG-2.5 格式
3. WHEN 写入侧信息 THEN Bitstream_Writer SHALL 遵循 MP3 标准格式
4. THE Bitstream_Writer SHALL 支持 CRC 校验 (可选)
5. WHEN 帧数据不足 THEN Bitstream_Writer SHALL 添加填充位

### 需求 7: 配置管理

**用户故事:** 作为开发者，我希望能够配置编码参数，以便控制输出质量和格式。

#### 验收标准

1. THE Config SHALL 支持设置采样率、比特率、声道数
2. THE Config SHALL 支持设置立体声模式 (立体声、联合立体声、双声道、单声道)
3. THE Config SHALL 支持设置强调模式和版权标志
4. WHEN 配置无效 THEN Config SHALL 返回错误信息
5. THE Config SHALL 提供默认值设置

### 需求 8: 内存管理和性能

**用户故事:** 作为开发者，我希望编码器具有高性能和安全的内存管理。

#### 验收标准

1. THE MP3_Encoder SHALL 使用 Rust 的所有权系统确保内存安全
2. THE MP3_Encoder SHALL 避免不必要的内存分配
3. WHEN 处理大量数据 THEN MP3_Encoder SHALL 保持稳定的内存使用
4. THE MP3_Encoder SHALL 使用固定点算术以提高性能
5. THE MP3_Encoder SHALL 支持 SIMD 优化 (可选)

### 需求 9: 错误处理

**用户故事:** 作为开发者，我希望编码器能够优雅地处理错误情况。

#### 验收标准

1. WHEN 输入无效配置 THEN MP3_Encoder SHALL 返回配置错误
2. WHEN 输入数据格式错误 THEN MP3_Encoder SHALL 返回数据错误
3. WHEN 内存分配失败 THEN MP3_Encoder SHALL 返回内存错误
4. THE MP3_Encoder SHALL 提供详细的错误信息
5. WHEN 发生错误 THEN MP3_Encoder SHALL 保持内部状态一致性

### 需求 10: 兼容性和测试

**用户故事:** 作为开发者，我希望确保编码器输出与标准 MP3 解码器兼容。

#### 验收标准

1. THE MP3_Encoder SHALL 生成符合 ISO/IEC 11172-3 标准的 MP3 文件
2. WHEN 编码测试音频 THEN 输出 SHALL 能被标准 MP3 解码器正确解码
3. THE MP3_Encoder SHALL 通过与 shine 库的对比测试
4. THE MP3_Encoder SHALL 支持回归测试以确保质量
5. WHEN 解析器解析输出文件 THEN 解析器 SHALL 正确识别所有元数据