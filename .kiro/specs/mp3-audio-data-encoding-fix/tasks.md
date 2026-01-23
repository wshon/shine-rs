# 实现计划: MP3音频数据编码修复

## 概述

本实现计划专注于修复MP3编码器的音频数据编码问题（主数据区域全零），通过与shine源码的严格对比和修复，确保生成有效的MP3文件。任务设计简洁明了，便于问题排查。

## 任务

- [x] 1. 文件结构和模块组织一致性检查
  - [x] 1.1 核心算法模块对应关系验证
    - 检查量化模块：`src/quantization.rs` ↔ `ref/shine/src/lib/l3loop.c`
    - 检查霍夫曼编码：`src/huffman.rs` ↔ `ref/shine/src/lib/huffman.c`
    - 检查比特流处理：`src/bitstream.rs` ↔ `ref/shine/src/lib/bitstream.c` + `l3bitstream.c`
    - 检查MDCT变换：`src/mdct.rs` ↔ `ref/shine/src/lib/l3mdct.c`
    - 验证每个模块的主要函数是否与shine对应文件中的函数匹配
    - _需求: 2.4, 2.5_

  - [x] 1.2 数据处理模块对应关系验证
    - 检查子带分析：`src/subband.rs` ↔ `ref/shine/src/lib/l3subband.c`
    - 检查查找表：`src/tables.rs` ↔ `ref/shine/src/lib/tables.c`
    - 检查比特储备池：`src/reservoir.rs` ↔ `ref/shine/src/lib/reservoir.c`
    - 验证数据结构和常量定义是否完整对应
    - _需求: 2.4, 2.5_

  - [x] 1.3 控制和配置模块对应关系验证
    - 检查主编码流程：`src/encoder.rs` ↔ `ref/shine/src/lib/layer3.c`
    - 检查数据结构定义：`src/shine_config.rs` ↔ `ref/shine/src/lib/types.h`
    - 检查高级配置封装：`src/config.rs` ↔ shine配置相关逻辑
    - 检查错误处理：`src/error.rs` ↔ shine错误处理逻辑
    - _需求: 2.4, 2.5_

  - [x] 1.4 函数分布一致性分析
    - 分析每个Rust模块中的函数是否都有对应的shine函数
    - 检查是否有shine函数在Rust中缺失或放错模块
    - 识别跨模块依赖是否合理，避免循环依赖
    - _需求: 2.4, 2.5_

  - [x] 1.5 模块完整性和重构需求评估
    - 验证是否有遗漏的shine功能模块
    - 识别需要重构或重新组织的模块
    - 评估模块边界是否清晰，职责是否单一
    - 制定模块重构计划（如果需要）
    - _需求: 2.4, 2.5_

- [x] 2. 核心数据结构一致性检查
  - 对比shine/src/lib/types.h中的所有关键结构体定义
  - 验证gr_info、shine_side_info_t、shine_global_config等结构体字段完全一致
  - 检查字段顺序、类型、大小是否与shine匹配
  - 验证结构体内存布局和对齐方式
  - 确保枚举类型和常量定义与shine一致
  - _需求: 2.4, 2.5_

- [-] 3. 关键函数实现一致性验证
  - 基于任务1和任务2建立的架构和数据结构基础，逐模块验证函数实现
  - [x] 3.1 量化模块函数验证 (src/quantization.rs ↔ l3loop.c)
    - 对比quantize、bin_search_StepSize、shine_inner_loop、shine_outer_loop函数
    - 验证量化步长计算、比特分配算法与shine完全一致
    - 确保GrInfo结构体的使用方式与shine一致
    - _需求: 2.1, 2.2, 2.3_

  - [x] 3.2 霍夫曼编码模块函数验证 (src/huffman.rs ↔ huffman.c)
    - 对比calc_runlen、subdivide、bigv_tab_select、bigv_bitcount函数
    - 验证码表选择和区域划分算法与shine一致
    - 确保霍夫曼表的使用方式正确
    - _需求: 3.1, 3.2, 3.3_

  - [ ] 3.3 比特流模块函数验证 (src/bitstream.rs ↔ bitstream.c + l3bitstream.c)
    - 对比shine_putbits、shine_write_main_data、shine_format_bitstream函数
    - 验证比特操作、帧格式和主数据写入逻辑与shine一致
    - 确保比特对齐和缓存处理正确
    - _需求: 4.1, 4.2, 4.3_

  - [ ] 3.4 其他核心模块函数验证
    - MDCT模块 (src/mdct.rs ↔ l3mdct.c): 变换算法和窗口函数
    - 子带模块 (src/subband.rs ↔ l3subband.c): 滤波器实现
    - 储备池模块 (src/reservoir.rs ↔ reservoir.c): 比特分配策略
    - 主编码流程 (src/encoder.rs ↔ layer3.c): 整体控制逻辑
    - _需求: 5.1, 5.2, 5.3, 5.4_

- [ ] 4. 编码流水线一致性验证
  - [ ] 4.1 添加数据流验证和异常检测
    - 建立各阶段数据的合理性检查标准：
      * PCM输入：非零样本比例 > 1%，动态范围合理
      * 子带输出：能量分布符合音频特征，非零系数 > 10%
      * MDCT系数：频域能量分布合理，低频系数通常较大
      * 量化系数：保留足够非零系数（通常 > 5%），big_values < 288
      * 霍夫曼编码：生成的比特数与非零系数数量相关
      * 比特流：主数据区域非零字节 > 50%
    - 实现端到端测试：使用已知音频文件，验证最终MP3可被ffmpeg解码
    - 添加与现有测试用例的对比验证（使用tests/input/中的音频文件）
    - _需求: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [ ] 4.2 关键算法逻辑验证
    - 量化循环验证：
      * 对比shine/src/lib/l3loop.c中的quantize函数实现
      * 验证量化步长计算公式与shine一致
      * 确保big_values和count1区域边界计算正确
    - 霍夫曼编码验证：
      * 对比shine/src/lib/huffman.c中的编码逻辑
      * 验证码表选择算法与shine一致
      * 确保区域划分逻辑正确
    - 比特流写入验证：
      * 对比shine/src/lib/l3bitstream.c中的写入逻辑
      * 验证主数据写入和比特对齐处理
    - 使用单元测试验证关键函数的输出合理性
    - _需求: 5.1, 5.2, 5.3, 5.4_

- [ ] 5. 问题修复和验证
  - 根据数据流验证和源码对比结果，修复发现的不一致问题
  - 验证修复效果的具体标准：
    * 生成的MP3文件主数据区域非零字节比例 > 50%
    * 使用ffmpeg验证MP3文件格式正确性：`ffmpeg -v error -i output.mp3 -f null -`
    * 使用ffprobe检查音频流信息：`ffprobe -v quiet -show_streams output.mp3`
    * 对比修复前后的编码统计数据（非零系数数量、比特使用等）
  - 运行现有测试套件确保无回归：`cargo test`
  - _需求: 4.4, 8.1, 8.2_

- [ ] 6. 最终验证和质量保证
  - 运行完整测试套件：`cargo test --all-targets --all-features`
  - 使用多种音频文件进行端到端测试（tests/input/目录中的文件）
  - 验证音频质量标准：
    * 生成的MP3文件大小合理（与配置的比特率匹配）
    * 音频可被多种解码器正确播放（ffmpeg, VLC等）
    * 频谱分析显示音频内容保持完整
  - 音质损失对比验证：
    * 将原始WAV编码为MP3，再解码回WAV格式
    * 使用ffmpeg计算信噪比：`ffmpeg -i original.wav -i decoded.wav -filter_complex "[0:0][1:0]amerge=inputs=2[a]" -map "[a]" -f null -`
    * 对比频谱图，确保主要频率成分保持
    * 验证动态范围和音量水平的保持程度
  - 性能回归测试：确保修复不影响编码速度
  - 与shine库输出进行端到端对比（文件大小、格式兼容性、音质）
  - _需求: 8.4, 9.1_

## 注意事项

- **架构优先**: 首先确保文件组织和模块结构与shine对应，建立清晰的架构视图
- **数据结构其次**: 在架构清晰后，深入检查核心数据结构的一致性
- **源码对比**: 重点通过阅读shine源码验证算法逻辑一致性，而不是运行时对比
- **内部监控**: 通过统计和异常检测来发现数据流问题
- **逐步验证**: 先检查静态结构，再检查动态流程，最后验证端到端输出
- **最小修改**: 只修复不一致的地方，避免不必要的改动
- **调试输出**: 添加临时调试输出便于问题定位，修复完成后可移除