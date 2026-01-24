# Claude Working Guidelines for shine-rs Project

This document consolidates all project-specific guidelines for working with the shine-rs MP3 encoder implementation.

---

## 1. Language & Communication Preferences

**Conversation Language**: Use **Chinese** for all user communication and explanations.

**Code Language**: Keep all code (including comments, variable names, function names, error messages, and logs) in **English**.

**Documentation**: Use Chinese for README and documentation, but explain technical terms in Chinese when needed.

---

## 2. Project Architecture & Core Principles

**Project Type**: MP3 encoder implementation in Rust, strictly following the `ref/shine/` C reference implementation.

### Critical Rule
**Always reference shine source code first** when implementing or debugging algorithms. Never modify Rust implementation without verifying against shine.

### Key Principles
- **Strict correspondence**: Every algorithm must match shine's C implementation line-by-line
- **No "optimizations"**: Do not simplify or change shine's original logic
- **Type mapping**:
  - C `int` → Rust `i32`
  - C `unsigned int` → Rust `u32`
  - C `float` → Rust `f32`
  - C `double` → Rust `f64`
- **Constant consistency**: All constants, lookup tables, and formulas must match shine exactly
- **Function signature consistency**: Function names, parameters, order, and return values must align with shine

### Module Structure
- **Core modules**: `bitstream`, `encoder`, `huffman`, `mdct`, `quantization`, `reservoir`, `subband`, `tables`
- **Config module**: `config`
- **Error module**: `error`
- **Target**: Keep files under 500 lines, single responsibility, no circular dependencies

---

## 3. Development Workflow

### Code Modification Process
1. View shine source code in `ref/shine/src/lib/`
2. Understand shine's algorithm logic, data flow, and function calls
3. Check current code state with diagnostics
4. Implement Rust version **strictly following shine**
5. Re-check diagnostics after changes
6. Validate against shine output with identical inputs
7. Run relevant tests
8. Ensure zero compile warnings/errors

### Debugging Protocol
- **Never** adjust algorithms based on intuition
- **Never** delete or weaken tests to make them pass
- **Always** compare Rust implementation line-by-line with shine C code
- **Always** verify boundary conditions match shine's handling
- **Always** record shine correspondence (file, function, line numbers)

---

## 4. Code Quality Requirements

### Static Analysis
- Zero compile warnings (use `getDiagnostics` tool)
- Fix all `cargo clippy` suggestions
- Verify with `cargo check`

### Rust Style
- Variables/functions: `snake_case`
- Structs/enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Error Handling
- Use `Result<T, E>` for error propagation
- Custom errors in `src/error.rs`
- Avoid `unwrap()`/`expect()` except in tests
- Keep error messages concise and clear

### Documentation
- All public APIs need `///` doc comments
- Complex algorithms need explanations of MP3 encoding role
- Reference shine source files/functions in comments
- Add inline comments for key algorithm steps

---

## 5. Testing Guidelines

### Test Organization
- **Property tests**: Use `proptest` for algorithm validation and edge cases
- **Unit tests**: Individual function behavior and error conditions
- **Integration tests**: Complete encoding workflows against reference outputs
- Separate test types into distinct `#[cfg(test)]` modules

### Proptest Configuration (Always Use This)
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

### Test Naming
- Pattern: `test_<module>_<behavior>` or `test_<condition>`
- Examples: `test_quantization_bounds`, `test_invalid_bitrate`
- Avoid overly detailed names that duplicate logic

### Assertion Messages (English Only)
```rust
// ✅ Good
prop_assert!(result.is_ok(), "Encoding failed");
prop_assert!(value <= MAX_VALUE, "Value exceeds limit");

// ❌ Avoid
prop_assert!(result.is_ok(), "编码应该成功但失败了");
prop_assert!(value <= MAX_VALUE, "The value is too large and exceeds the maximum allowed limit");
```

### Module Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    mod unit_tests { /* Individual function tests */ }
    mod property_tests { /* Proptest-based validation */ }
    mod integration_tests { /* End-to-end workflow tests */ }
}
```

### Algorithm Testing
- Compare outputs against `ref/shine` implementation
- Use identical input parameters and validate numerical precision
- Test edge cases: boundary values, invalid inputs, buffer limits
- Use appropriate proptest strategies for audio data ranges
- Limit array sizes to reasonable bounds
- Keep test execution time reasonable (< 1 second per property test)

### Command Line Usage
```bash
cargo test                    # Run all tests
cargo test --quiet            # Minimal output
cargo test tests::quantization # Specific module
PROPTEST_VERBOSE=0 cargo test # Proptest environment
```

---

## 6. Shine Correspondence Maintenance

### File Mapping
- Encoder main → shine main encoding flow
- Quantization → shine quantization/calculation
- MDCT → shine MDCT transform
- Subband → shine subband analysis
- Bitstream → shine bitstream
- Huffman → shine Huffman coding
- Tables → shine lookup tables
- Reservoir → shine reservoir

### Critical Function Correspondence
- Encoding main flow functions
- MDCT transform functions
- Quantization loop functions
- Bitstream processing functions
- Huffman encoding functions

### Data Structure Mapping
- Global config → shine global config
- Granule info → shine granule info
- Side info → shine side info
- Psy model → shine psychoacoustic model

### Validation Requirements
- Each function must pass unit tests matching shine output
- Key algorithms must pass numerical precision tests
- Complete encoding must generate identical MP3 files as shine

---

## 7. Performance Considerations

- MP3 encoding is compute-intensive; prioritize correctness first
- Optimize performance hotspots only after correctness is verified
- Use `cargo bench` for performance benchmarking
- Avoid unnecessary memory allocation and copying
- Use `#[ignore]` for expensive tests that run separately

---

## Quick Reference Commands

### Diagnostics & Checks
```bash
cargo check                    # Compile check
cargo clippy                   # Linting
cargo test                     # Run all tests
cargo test --quiet             # Minimal output
cargo bench                    # Performance benchmarks
```

### Test Execution
```bash
cargo test tests::module_name  # Specific test module
PROPTEST_VERBOSE=0 cargo test  # Clean proptest output
cargo test -- --nocapture      # Show print output
```

### Reference Validation
```bash
# Compare with shine reference
./ref/shine/input.wav output.mp3
cargo run -- --input input.wav --output output.mp3
```

---

## Key Reminders

1. **Always check shine source first** - `ref/shine/src/lib/`
2. **Never modify algorithms without shine reference**
3. **Keep code in English, conversation in Chinese**
4. **Zero warnings policy** - Fix all diagnostics
5. **Test failure means implementation issue** - Fix implementation, not tests
6. **Maintain shine correspondence** - Document file/function/line mappings
7. **Use proptest config** - Always with `verbose: 0` and `max_shrink_iters: 0`
8. **English assertion messages** - Keep them concise
9. **Algorithm correctness first** - Performance optimization later
10. **Reference validation** - Compare outputs with shine for all key functions
