---
inclusion: always
---

# Testing Guidelines

## Core Testing Principles

### Test Organization
- **Property tests**: Use `proptest` for algorithm validation and edge case discovery
- **Unit tests**: Focus on individual function behavior and error conditions  
- **Integration tests**: Validate complete encoding workflows against reference outputs
- **Module structure**: Separate test types into distinct modules (`#[cfg(test)]` blocks)

### Error Message Standards
- Use concise, descriptive assertion messages in English
- Avoid verbose explanations in test failure output
- Format: `prop_assert!(condition, "Brief description")`

## Proptest Configuration

### Standard Configuration
Always use this configuration for property tests to minimize verbose output:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        verbose: 0,
        max_shrink_iters: 0,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]
    
    #[test]
    fn test_function(input in strategy()) {
        // Test logic
    }
}
```

### Clean Error Output (Optional)
For tests with large data structures, use this panic hook to truncate verbose output:

```rust
use std::sync::Once;
static INIT: Once = Once::new();

fn setup_clean_errors() {
    INIT.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            if let Some(s) = info.payload().downcast_ref::<String>() {
                let msg = if s.len() > 200 { &s[..197] } else { s };
                eprintln!("Test failed: {}", msg.trim());
            }
        }));
    });
}
```

## Test Naming and Structure

### Function Names
- Use descriptive but concise names: `test_quantization_bounds`, `test_invalid_bitrate`
- Avoid overly detailed names that duplicate test logic
- Pattern: `test_<module>_<behavior>` or `test_<condition>`

### Assertion Messages
```rust
// ✅ Good
prop_assert!(result.is_ok(), "Encoding failed");
prop_assert!(value <= MAX_VALUE, "Value exceeds limit");

// ❌ Avoid
prop_assert!(result.is_ok(), "编码应该成功但失败了");
prop_assert!(value <= MAX_VALUE, "The value is too large and exceeds the maximum allowed limit");
```

### Module Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    mod unit_tests {
        // Individual function tests
    }
    
    mod property_tests {
        // Proptest-based validation
    }
    
    mod integration_tests {
        // End-to-end workflow tests
    }
}
```

## Algorithm Testing Requirements

### Reference Validation
- Compare outputs against `ref/shine` implementation when possible
- Use identical input parameters and validate numerical precision
- Test edge cases: boundary values, invalid inputs, buffer limits

### Data Generation Strategies
- Use appropriate proptest strategies for audio data ranges
- Limit array sizes to reasonable bounds (avoid memory issues)
- Generate realistic audio parameters (sample rates, bit rates, etc.)

### Performance Considerations
- Keep test execution time reasonable (< 1 second per property test)
- Use smaller data sets for property tests, larger for integration tests
- Consider using `#[ignore]` for expensive tests that run separately

## Command Line Usage

```bash
# Run all tests
cargo test

# Run with minimal output
cargo test --quiet

# Run specific test module
cargo test tests::quantization

# Set environment for proptest
PROPTEST_VERBOSE=0 cargo test
```
