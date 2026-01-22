# 测试指导原则

## Proptest 配置规范

### 禁用详细参数输出

在使用 proptest 进行属性测试时，为了避免测试失败时显示过长的参数值，推荐使用自定义 panic 钩子作为兜底方案：

```rust
use std::sync::Once;

static INIT: Once = Once::new();

/// 设置自定义 panic 钩子，只输出通用错误信息
fn setup_panic_hook() {
    INIT.call_once(|| {
        std::panic::set_hook(Box::new(|_| {
            eprintln!("Test failed: Property test assertion failed");
        }));
    });
}

proptest! {
    #[test]
    fn your_property_test(input in your_strategy()) {
        setup_panic_hook();
        // 测试逻辑
    }
}
```

### 配置说明

- **自定义 panic 钩子**：最有效的方法，完全覆盖所有失败输出，只显示通用错误信息
- **兜底方案**：无论 proptest 如何配置，都能确保不显示详细的失败参数
- **简洁输出**：失败时只显示 "Test failed: Property test assertion failed"
- 注意：这会让调试变得困难，但可以保持测试输出完全简洁

### 传统配置方法（可选）

如果不使用自定义 panic 钩子，也可以尝试配置 proptest：

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // 测试用例数量
        verbose: 0,  // 禁用详细参数输出
        max_shrink_iters: 0,  // 禁用收缩，避免显示 minimal failing input
        failure_persistence: None,  // 禁用失败持久化
        ..ProptestConfig::default()
    })]
    
    #[test]
    fn your_property_test(input in your_strategy()) {
        // 测试逻辑
    }
}
```

### 错误消息规范

使用简洁明确的错误消息：

```rust
// ✅ 好的做法
prop_assert!(result.is_ok(), "Encoding failed");
prop_assert!(value <= 576, "Value too large");

// ❌ 避免的做法
prop_assert!(result.is_ok(), "编码应该成功，但是失败了，可能的原因包括...");
```

### 命令行选项

如果需要进一步控制输出，可以使用：

```bash
# 静默模式
cargo test --quiet

# 环境变量控制
set PROPTEST_VERBOSE=0
cargo test
```

## 单元测试规范

- 单元测试使用简洁的断言消息
- 避免在测试名称中包含过多细节
- 使用描述性但简洁的测试函数名

## 测试组织

- 属性测试放在单独的模块中
- 单元测试放在 `unit_tests` 模块中
- 使用清晰的模块结构分离不同类型的测试