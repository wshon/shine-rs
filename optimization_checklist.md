# 代码优化安全检查清单

## 优化前必须执行

### 1. 基础测试验证
```bash
# 确保所有测试通过
cargo test
cargo test --features diagnostics

# 检查编译警告
cargo clippy
cargo check
```

### 2. 创建性能基线
```bash
# 记录当前性能
cargo bench > baseline_performance.txt

# 或使用简单的时间测量
time cargo run --release -- input.wav output.mp3
```

### 3. 创建输出基线
```bash
# 生成参考输出
cargo run --release -- tests/audio/sample-3s.wav baseline_output.mp3
sha256sum baseline_output.mp3 > baseline_hash.txt
```

## 优化过程中

### 每次修改后执行
```bash
# 1. 立即检查编译
cargo check

# 2. 运行相关测试
cargo test [module_name]

# 3. 检查诊断特性
cargo test --features diagnostics
```

### 关键算法修改后
```bash
# 验证输出一致性
cargo run --release -- tests/audio/sample-3s.wav test_output.mp3
sha256sum test_output.mp3
# 对比 baseline_hash.txt

# 运行完整测试套件
cargo test
```

## 优化完成后

### 完整验证
```bash
# 1. 所有测试通过
cargo test --all-features

# 2. 性能未退化
cargo bench
# 对比 baseline_performance.txt

# 3. 输出完全一致
# 对比所有测试文件的哈希值

# 4. 无编译警告
cargo clippy -- -D warnings
```

### 回归测试
```bash
# 运行扩展测试（如果可用）
python scripts/validate_reference_files.py

# 测试不同配置
cargo test test_different_bitrates
cargo test test_different_configurations
```

## 风险评估

### 低风险优化 ✅
- 变量重命名
- 代码格式化  
- 注释改进
- 非算法性能优化
- 错误处理改进

### 中风险优化 ⚠️
- 函数重构（保持逻辑不变）
- 数据结构优化
- 内存分配优化
- 循环优化

### 高风险优化 🚨
- MDCT算法修改
- 量化参数计算
- 比特流编码逻辑
- 子带滤波器
- 查找表修改

## 紧急回滚

如果发现问题：
```bash
# 1. 立即停止优化
git stash

# 2. 验证原始版本
cargo test

# 3. 逐步恢复修改
git stash pop
# 或
git reset --hard HEAD~1
```