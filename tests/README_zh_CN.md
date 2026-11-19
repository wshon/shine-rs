[English Documentation](README.md)

# 测试架构文档

## 测试套件组织

测试套件已重组为5个清晰的类别，消除了重复功能：

### 1. 基础功能测试 (`encoder_basic_functionality.rs`)
**状态**: ✅ 全部通过
**目的**: 验证编码器的基本功能和错误处理
**测试内容**:
- 基本编码功能
- 错误处理机制
- 不同输入格式支持

### 2. 实时比较测试 (`encoder_comparison_live.rs`)
**状态**: ✅ 通过（需外部 Shine 二进制，故默认忽略）
**目的**: 与Shine C 参考实现进行实时逐字节对比
**测试内容**:
- 默认文件比较
- 不同比特率测试
- 语音文件比较
- 大文件比较

**说明**: Rust 输出与 Shine C **逐字节一致**。这些测试默认被忽略，因为需要外部 `ref/shine/shineenc` 二进制。需要手动执行：
```bash
# 运行所有实时比较测试
cargo test --test encoder_comparison_live -- --ignored

# 运行特定测试
cargo test test_default_file_comparison -- --ignored
```

### 3. CI/CD验证测试 (`encoder_validation_cicd.rs`)
**状态**: ✅ 全部通过
**目的**: 使用预生成参考数据进行验证（不依赖Shine二进制）
**测试内容**:
- 标准配置验证
- 参考文件完整性检查

**注意**: 参考数据由 Rust 编码器生成，作为回归基线使用。Rust 输出与 Shine C 逐字节一致，这些基线等同于 C 的参考输出。

### 4. 低级API测试 (`encoder_low_level_api.rs`)
**状态**: ✅ 全部通过
**目的**: 验证Shine兼容的低级API
**测试内容**:
- Shine配置创建和验证
- 低级编码函数
- 内存管理
- 错误处理

### 5. 高级API测试 (`encoder_high_level_api.rs`)
**状态**: ✅ 全部通过
**目的**: 验证Rust风格的高级API
**测试内容**:
- 配置验证
- 编码器创建
- PCM编码
- 不同配置测试
- 立体声模式
- 错误条件处理
- 完整编码工作流

## 测试命令

### 运行所有测试
```bash
cargo test
```

### 运行特定测试类别
```bash
# 基础功能
cargo test --test encoder_basic_functionality

# 实时比较（默认忽略，需要手动执行）
cargo test --test encoder_comparison_live -- --ignored

# CI/CD验证
cargo test --test encoder_validation_cicd

# 低级API
cargo test --test encoder_low_level_api

# 高级API
cargo test --test encoder_high_level_api
```

## 数值一致性

Rust 编码器与 Shine C 逐字节一致，覆盖 6 种采样率/码率配置（22.05/24/32/44.1/48 kHz，64–192 kbps）。实现中沿用 Shine C 的常量与定点运算约定，以保持码流一致。

## CLI 参数顺序

编码命令使用 `-b <bitrate> <input> <output>` 的参数顺序。

## 测试覆盖率

- ✅ 基础编码功能
- ✅ 错误处理
- ✅ 不同配置支持
- ✅ 高级和低级API
- ✅ 内存管理
- ✅ 与Shine C 的逐字节一致性

## 下一步工作

1. **完善测试覆盖**: 添加更多边界条件测试
2. **性能测试**: 添加性能基准测试
3. **文档完善**: 更新API文档和使用示例