# 项目结构说明

这个文档描述了MP3编码器项目的文件夹组织结构和各部分的作用。

## 根目录结构

```
shine-rs/
├── .claude/                    # Claude AI配置文件
├── .git/                       # Git版本控制
├── .kiro/                      # Kiro IDE配置
├── benches/                    # 性能基准测试
├── debug_outputs/              # 调试输出文件
├── docs/                       # 项目文档
├── examples/                   # 示例代码
├── ref/                        # 参考实现（Shine C代码）
├── scripts/                    # 脚本工具
├── src/                        # Rust源代码
├── target/                     # Cargo构建输出
├── testing/                    # 测试相关文件
│   ├── fixtures/               # 测试固件和数据
│   │   ├── audio/              # 测试音频文件
│   │   ├── data/               # 测试数据JSON文件
│   │   ├── output/             # 测试输出文件
│   │   └── tools/              # 测试工具
│   ├── integration/            # 集成测试
│   └── regression/             # 回归测试数据
├── .gitignore                  # Git忽略文件配置
├── Cargo.lock                  # 依赖锁定文件
├── Cargo.toml                  # 项目配置文件
└── README.md                   # 项目说明
```

## 详细目录说明

### 核心代码目录

#### `src/` - Rust源代码
```
src/
├── bin/                        # 可执行程序
│   ├── collect_test_data.rs    # 测试数据收集工具
│   ├── validate_test_data.rs   # 测试数据验证工具
│   └── wav2mp3.rs             # WAV到MP3转换工具
├── bitstream.rs               # 比特流处理模块
├── encoder.rs                 # 主编码器模块
├── error.rs                   # 错误类型定义
├── huffman.rs                 # Huffman编码模块
├── lib.rs                     # 库入口文件
├── mdct.rs                    # MDCT变换模块
├── pcm_utils.rs              # PCM音频处理工具
├── quantization.rs           # 量化模块
├── reservoir.rs              # 比特池模块
├── subband.rs                # 子带分析模块
├── tables.rs                 # 查找表模块
├── test_data.rs              # 测试数据结构定义
└── types.rs                  # 类型定义
```

#### `ref/` - 参考实现
```
ref/
└── shine/                     # Shine C语言参考实现
    ├── src/lib/               # Shine核心算法源码
    └── ...                    # 其他Shine文件
```

### 测试相关目录

#### `testing/` - 测试相关文件
统一的测试目录，包含所有测试相关的子目录：

```
testing/
├── proptest-regressions/       # Proptest回归测试数据
├── test_data/                  # 测试数据JSON文件
├── test_inputs/                # 测试输入音频文件
└── tests/                      # 集成测试
    ├── bin/                    # 测试可执行程序
    ├── output/                 # 测试输出文件
    ├── integration_full_pipeline_validation.rs  # 完整流水线验证测试
    ├── integration_scfsi_consistency.rs         # SCFSI一致性测试
    └── README.md               # 测试说明文档
```

#### `testing/test_data/` - 测试数据
存储JSON格式的测试用例数据，用于验证编码器实现的正确性：
```
testing/test_data/
├── test_data.json             # 基础测试用例
├── sample_3s_test_data.json   # 长音频测试用例
└── ...                        # 其他测试用例
```

#### `testing/test_inputs/` - 测试输入文件
存储用于测试的WAV音频文件：
```
testing/test_inputs/
├── test_input.wav             # 基础测试音频
├── sample-3s.wav             # 3秒测试音频
└── ...                        # 其他测试音频
```

#### `testing/proptest-regressions/` - 属性测试回归数据
存储proptest发现的失败用例，用于回归测试：
```
testing/proptest-regressions/
├── bitstream.txt              # 比特流模块回归数据
└── subband.txt               # 子带模块回归数据
```

### 工具和脚本目录

#### `scripts/` - 脚本工具
```
scripts/
├── create_test_wav.py         # Python脚本：生成测试WAV文件
├── run_test_suite.ps1         # PowerShell脚本：运行完整测试套件
└── test_bitstream_endian.c    # C程序：测试比特流字节序
```

#### `benches/` - 性能基准测试
```
benches/
└── encoder_benchmark.rs       # 编码器性能基准测试
```

### 输出和临时目录

#### `debug_outputs/` - 调试输出
存储调试过程中生成的临时文件和输出：
```
debug_outputs/
├── rust_debug.log            # Rust编码器调试日志
├── shine_debug.log           # Shine编码器调试日志
├── *.mp3                     # 调试生成的MP3文件
└── ...                       # 其他调试文件
```

#### `target/` - Cargo构建输出
Cargo自动生成的构建输出目录，包含编译后的二进制文件和中间文件。

### 文档目录

#### `docs/` - 项目文档
```
docs/
├── debug_analysis.md         # 调试分析文档
├── PROJECT_STRUCTURE.md      # 项目结构说明（本文档）
├── TEST_DATA_FRAMEWORK.md    # 测试数据框架文档
└── VERIFICATION_RECORD.md    # 验证记录文档
```

### 配置文件

#### 项目配置
- `Cargo.toml` - Rust项目配置，定义依赖、构建设置等
- `Cargo.lock` - 依赖版本锁定文件，确保构建一致性
- `.gitignore` - Git忽略文件配置

#### IDE和工具配置
- `.claude/` - Claude AI助手配置
- `.kiro/` - Kiro IDE配置
- `examples/` - 示例代码目录（当前为空）

## 文件组织原则

### 1. 按功能分类
- **核心算法**: `src/` 目录下的各个模块
- **测试相关**: `testing/` 目录下的所有测试文件
- **工具脚本**: `scripts/` 目录
- **文档**: `docs/` 目录

### 2. 输入输出分离
- **输入文件**: `testing/test_inputs/` 目录
- **输出文件**: `debug_outputs/` 目录
- **测试数据**: `testing/test_data/` 目录

### 3. 临时文件隔离
- **调试输出**: `debug_outputs/` 目录
- **构建输出**: `target/` 目录（Git忽略）
- **回归数据**: `proptest-regressions/` 目录

## 使用指南

### 开发工作流
1. **源码修改**: 在 `src/` 目录下修改Rust代码
2. **测试验证**: 使用 `test_data/` 中的测试用例验证
3. **调试分析**: 查看 `debug_outputs/` 中的调试输出
4. **文档更新**: 在 `docs/` 目录下更新相关文档

### 测试工作流
1. **添加测试音频**: 放入 `testing/test_inputs/` 目录
2. **收集测试数据**: 使用 `scripts/run_test_suite.ps1` 或手动运行工具
3. **验证实现**: 运行集成测试和单元测试
4. **查看结果**: 检查 `debug_outputs/` 中的输出文件

### 脚本使用
- **完整测试**: `scripts/run_test_suite.ps1`
- **生成测试音频**: `scripts/create_test_wav.py`
- **字节序测试**: `scripts/test_bitstream_endian.c`

## 维护建议

1. **定期清理**: 清理 `debug_outputs/` 和 `target/` 目录中的临时文件
2. **版本控制**: 将重要的测试数据和文档纳入版本控制
3. **文档同步**: 代码修改后及时更新相关文档
4. **测试覆盖**: 为新功能添加相应的测试用例和文档

这个结构设计确保了项目的可维护性、可测试性和可扩展性，同时保持了清晰的组织层次。