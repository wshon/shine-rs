# AGENTS.md

This document defines how AI coding agents should work with this repository.

## Project Overview

Shine-RS is a pure Rust MP3 encoder, a faithful port of the Shine C fixed-point MP3 encoder. Output bitstream must match the C reference exactly, byte for byte.

## Build & Test Commands

```bash
# Build (all features)
cargo build --all-features

# Run all tests
cargo test --all-features

# Run a specific test group
cargo test --all-features quantization

# Release build
cargo build --release

# CLI tool (WAV to MP3)
cargo run --release --bin shine-rs-cli -- input.wav output.mp3

# Performance benchmark (needs the Shine C reference submodule: git submodule update --init ref/shine)
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh
```

## Coding Rules

### Algorithm Fidelity (CRITICAL)

All core algorithms must match the Shine C reference implementation exactly. Before modifying any code in `crate/src/bitstream.rs`, `crate/src/quantization.rs`, `crate/src/mdct.rs`, `crate/src/subband.rs`, or `crate/src/huffman.rs`:

1. Compare against the C code under `ref/shine/src/lib/`
2. Run `cargo test encoder_validation_cicd` to verify SHA256 output matches
3. Run `cargo test -p shine-rs --test encoder_boundaries` under debug (`overflow-checks = true`) — boundary inputs like `i32::MIN` and full-scale audio must not panic
4. Do NOT simplify, skip steps, or "optimize" in ways that change output

Use the exact truncated constants from the C source (`PI64 = 0.049087385212`, `LN2 = 0.69314718`, etc.). Full-precision constants such as `std::f64::consts::PI / 64.0` shift values across integer boundaries after rounding and change the bitstream. The constants test (`crate/tests/types_tests.rs`) asserts these truncated literals; keep it in sync.

### Fixed-Point Arithmetic

- MDCT hot loops use `wrapping_add` to match C's `+=` — one instruction, no saturating branch
- `labs` keeps `saturating_abs`: `i32::MIN` would otherwise produce a negative value and index out of bounds in `quantize_with_l3loop`
- The quantizer hot loop reads precomputed `xrabs[i]` (== `labs(xr[i])`, computed once per granule in `shine_iteration_loop`) instead of recomputing `labs` per call. Invariant: `xr` (`mdct_freq`) is never modified after precompute — preserve this when touching the iteration loop, and keep the precondition documented on `quantize_with_l3loop`
- Edge cases are verified by debug builds (`overflow-checks = true`) plus `encoder_boundaries` tests
- Multiplication rounding: `mulr` uses `+0x80000000 >> 32`, `mulsr` uses `+0x40000000 >> 31`
- Test edge cases: all-zeros, i32::MIN values, max amplitude

### Unsafe Code

Unsafe is permitted in hot loops for pointer arithmetic (MDCT, quantization). Every unsafe block must have a comment explaining the safety invariant. Do not add new unsafe to production code without strong justification — prefer safe slice/iterator patterns.

### Feature Gating

- `diagnostics`: enables debug logging, frame limits, verbose output
- Core encoding logic must not depend on diagnostics features

### Testing

1. **Unit tests**: next to code in `#[cfg(test)]` modules
2. **Integration tests**: `crate/tests/` and `tests/` directories
3. **Reference validation**: must pass `encoder_validation_cicd`
4. **Boundary tests**: must pass `encoder_boundaries` in debug (overflow checks on)
5. **No test shortcuts**: all tests must pass before merging

### Code Quality

- Zero compiler warnings (`cargo build` must be clean)
- Pass `cargo clippy --all-targets --all-features -- -D warnings`
- Pass `cargo fmt --all -- --check` (run `cargo fmt --all` before committing; the editor here may skip formatting as "untrusted workspace")
- Document all public APIs with `///` comments

## Branch & PR Conventions

- Branch naming: `feat/...`, `fix/...`, `docs/...`, `perf/...`, `refactor/...`, `ci/...`
- Never push directly to `main` — always work through a PR
- PR titles follow Conventional Commits: `feat:`, `fix:`, `docs:`, `ci:`, `perf:`, `refactor:`
- Write PR descriptions in English, concise, focused on results
- PR description must include a summary, test results, and benchmark results if performance-related

## File Layout

- `crate/` — published library (core logic)
- `src/` — CLI binary (WAV reader, argument parsing)
- `tests/` — integration tests and benchmarks
- `ref/shine/` — Shine C reference submodule (DO NOT MODIFY; `git submodule update --init ref/shine`)
- `scripts/` — helper scripts and benchmarks
- `docs/` — documentation

## Documentation

READMEs are bilingual: English is the primary file (`README.md`) and Chinese is a separate file (`README_zh_CN.md`) linked from the top of each. Keep translation content in sync when editing.

## Git Workflow

```bash
# Start a feature branch
git checkout -b feat/my-feature main

# Commit often
git add -A && git commit -m "feat: add new capability"

# Push and create a PR
git push -u origin feat/my-feature
```

## Known Pitfalls

1. **i32::MIN overflow**: MDCT uses `wrapping_add`; `labs` uses `saturating_abs` (a negative abs would index out of bounds). Both verified by `encoder_boundaries` under debug `overflow-checks = true`
2. **Reservoir state**: `resv_drain` must reset to 0 every frame
3. **Bitstream padding**: fill bits must be written when reservoir reaches capacity
4. **Truncated constants**: use C's literal values, not full-precision `std::f64::consts`
5. **Time precision**: datetime comparisons in tests must truncate to minute precision
6. **MSRV**: 1.83 minimum — `const fn` float arithmetic in `tables.rs` (`shine_ew`) is unstable before that; CI `minimum-rust-version` job enforces it
7. **Hot-loop CSE is manual**: the compiler cannot reuse `labs(xr[i])` across `quantize` calls because writes through `ix` may alias the `xr` raw pointer — reuse the precomputed `xrabs` array instead of recomputing

## Performance

Pure-encode speed exceeds Shine C by 1.14–1.35x across all tested configurations (2026-11-19 benchmark, median of 5 runs per config, see README). The gap comes from replacing saturating add with `wrapping_add` in the MDCT hot loop and reusing precomputed `xrabs` values in the quantizer; `labs` keeps `saturating_abs` for safety. Profile before optimizing — MDCT and subband filter are the hot paths. Never sacrifice byte-identical output or boundary robustness for performance: verify with `encoder_validation_cicd`, `encoder_boundaries` (debug), and golden SHA256 tests. Use `scripts/bench/build_and_run.sh` for reproducible benchmarks.
