# AGENTS.md

This document defines how AI coding agents should work with this repository.

## Project Overview

Shine-RS is a pure Rust MP3 encoder, a faithful port of the Shine C fixed-point MP3 encoder. Output bitstream must match the C reference exactly.

## Build & Test Commands

```bash
# Build (all features)
cargo build --manifest-path crate/Cargo.toml --all-features

# Run all tests
cargo test --manifest-path crate/Cargo.toml --all-features

# Run specific test group
cargo test --manifest-path crate/Cargo.toml --all-features quantization

# Release build
cargo build --release --manifest-path crate/Cargo.toml

# CLI tool
cargo run --release --bin shine-rs -- input.wav output.mp3

# Enable SIMD (optional, default off)
cargo run --release --bin shine-rs -- input.wav output.mp3 --simd

# Benchmarks
cargo bench --manifest-path crate/Cargo.toml
```

## Coding Rules

### Algorithm Fidelity (CRITICAL)

All core algorithms must match the Shine C reference implementation exactly. Before modifying any code in `crate/src/bitstream.rs`, `crate/src/quantization.rs`, `crate/src/mdct.rs`, `crate/src/subband.rs`, or `crate/src/huffman.rs`:

1. Compare against `ref/shine/src/lib/` C code
2. Run `cargo test encoder_validation_cicd` to verify SHA256 output match
3. Do NOT simplify, skip steps, or "optimize" in ways that change output

### Fixed-Point Arithmetic

- Use saturating arithmetic for all intermediate calculations
- `i32::MIN.abs()` overflows — always use `saturating_abs()`
- Multiplication rounding: `mulr` uses `+0x80000000 >> 32`, `mulsr` uses `+0x40000000 >> 31`
- Test edge cases: all-zeros, i32::MIN values, max amplitude

### Unsafe Code

Unsafe is permitted in hot loops for pointer arithmetic (MDCT, quantization). Every unsafe block must have a comment explaining the safety invariant.

### Feature Gating

- `diagnostics`: enables debug logging, frame limits, verbose output
- Core encoding logic must not depend on diagnostics features
- SIMD is opt-in via `use_simd` config flag (default: false)

### Testing

1. **Unit tests**: next to code in `#[cfg(test)]` modules
2. **Integration tests**: `crate/tests/` directory
3. **Reference validation**: must pass `encoder_validation_cicd` test
4. **No test shortcuts**: all tests must pass before merging

### Code Quality

- Zero compiler warnings (`cargo build` must be clean)
- Pass `cargo clippy` with no warnings
- Use `cargo fmt` for formatting
- Document all public APIs with `///` comments
- Non-blocking nits: use `// TODO(perf):` or `// TODO(simd):` for future improvements

## Branch & PR Conventions

- Branch naming: `feat/...`, `fix/...`, `perf/...`, `refactor/...`
- Never push directly to `main`
- PR titles follow Conventional Commits: `feat:`, `fix:`, `docs:`, `ci:`, `perf:`, `refactor:`
- PR description must include:
  - Summary of changes
  - Test results (pass/fail count)
  - Benchmark results if performance-related
  - Breaking changes if any

## File Layout

- `crate/` — published library (core logic)
- `src/` — CLI binary (WAV reader, argument parsing)
- `tests/` — integration tests and benchmarks
- `ref/shine/` — C reference (DO NOT MODIFY)
- `docs/` — documentation

## Git Workflow

```bash
# Start feature
git checkout -b feat/my-feature main

# Commit often
git add -A && git commit -m "feat: add new capability"

# Push and create PR
git push -u origin feat/my-feature
```

## Current Project State

- 160+ tests passing
- Output matches Shine C (SHA256 verified)
- Average 114x real-time encoding (within 14% of C)
- SIMD infrastructure in place (opt-in, not connected to main path)
- Benchmark suite with 6 criterion benchmarks

## Known Pitfalls

1. **i32::MIN overflow**: `x.abs()` when x == i32::MIN → use `x.saturating_abs()`
2. **Reservoir state**: `resv_drain` must reset to 0 every frame
3. **Bitstream padding**: fill bits must be written when reservoir reaches capacity
4. **Time precision**: datetime comparisons in tests must truncate to minute precision

## Performance Guidelines

- Profile before optimizing (MDCT and subband filter are the hot paths)
- Benchmark against criterion suite: `cargo bench`
- SIMD is opt-in via config flag; scalar path is the default
- Don't sacrifice correctness for performance
