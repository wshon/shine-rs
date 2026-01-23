# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common commands

All commands are intended to be run from the repository root.

### Build, check, and lint

- Build debug: `cargo build`
- Build release: `cargo build --release`
- Fast typecheck only: `cargo check`
- Lint with Clippy (must be clean): `cargo clippy --all-targets --all-features`

### Tests and property tests

From `README.md` and `tests/README.md`/`.kiro/steering/testing-guidelines.md`:

- Run the full test suite (unit + integration + property tests):
  - `cargo test`
- Quieter test output (useful for large property tests):
  - `cargo test --quiet`
- Run a specific integration test module (mapped to files in `tests/`):
  - Comprehensive validation: `cargo test validation_comprehensive`
  - Debug/diagnostic suite: `cargo test debug_comprehensive`
  - Shine reference comparison: `cargo test shine_reference`
  - Big values validation: `cargo test big_values_validation`
- Run individual long-running / external-tool tests (marked `#[ignore]`):
  - FFmpeg-based validation: `cargo test test_ffmpeg_validation -- --ignored`
  - Shine encoder comparison: `cargo test test_shine_comparison -- --ignored`
  - Real audio file tests: `cargo test test_real_audio_file -- --ignored`
- Control property-test behavior (Proptest):
  - Standard configuration: `PROPTEST_VERBOSE=0 cargo test`
  - More exhaustive runs: `PROPTEST_CASES=1000 cargo test`

For ad‑hoc single-test runs, follow Rust’s standard pattern, e.g.:

- By test function name: `cargo test test_mp3_encoder_creation`
- By module path (see `#[cfg(test)] mod tests { ... }` blocks): `cargo test tests::quantization`

### Benchmarks

- Run Criterion benchmarks (HTML reports enabled via `Cargo.toml`):
  - `cargo bench`

### Examples and binaries

From `README.md` and `Cargo.toml`:

- Run the basic encoding example:
  - `cargo run --example basic_encoding`
- Run helper binaries defined in `Cargo.toml`:
  - WAV → MP3 CLI: `cargo run --bin wav2mp3 -- --help`
  - MP3 validator: `cargo run --bin mp3_validator -- --help`
  - MP3 hex dump: `cargo run --bin mp3_hexdump -- --help`

### External tools used by tests

Some tests and workflows assume these are available (see `tests/README.md`):

- FFmpeg / FFprobe: for validating and inspecting generated MP3s
- `shine` C encoder: used as the reference implementation in comparison tests

Example FFmpeg installation commands from the docs (for human operators):

- Windows (Chocolatey): `choco install ffmpeg`
- macOS (Homebrew): `brew install ffmpeg`
- Debian/Ubuntu: `sudo apt install ffmpeg`

## High-level architecture

This crate is a Rust reimplementation of the C-based `shine` MP3 Layer III encoder. The design intentionally mirrors shine’s architecture, data structures, and control flow, while exposing a clean Rust API.

### Public API surface (`src/lib.rs`)

- The crate root exposes the high-level types:
  - `Mp3Encoder` – primary encoder object orchestrating the complete pipeline.
  - `Config`, `WaveConfig`, `MpegConfig`, `Channels`, `StereoMode`, `Emphasis` – configuration types for audio format and MPEG parameters.
  - Error types: `EncoderError`, `ConfigError`, `InputDataError`, `EncodingError` and a crate-wide `Result<T>` alias.
- `lib.rs` re-exports internal modules:
  - `config`, `encoder`, `bitstream`, `subband`, `mdct`, `quantization`, `reservoir`, `huffman`, `tables`, `error`, `shine_config`.

### Encoding pipeline

The end-to-end MP3 pipeline follows the classic Layer III flow, as documented in `README.md` and `.kiro/specs/rust-mp3-encoder/design.md`:

1. **PCM input** – user provides PCM frames (non-interleaved or interleaved) to `Mp3Encoder`.
2. **Subband filter (`subband`)** – 32-band polyphase analysis filter splits PCM into subband samples.
3. **MDCT transform (`mdct`)** – converts subband samples into 576-frequency-coefficient blocks per granule, including aliasing reduction and windowing.
4. **Quantization loop (`quantization` + `reservoir`)** – implements shine’s inner/outer quantization loops, bit allocation, and use of the bit reservoir:
   - Chooses a quantizer step size via `bin_search_step_size`.
   - Runs an inner loop (`shine_inner_loop`) to satisfy bit constraints.
   - Maintains `GranuleInfo` metadata (e.g., `big_values`, `count1`, `table_select`, `region0_count`, `region1_count`, `quantizer_step_size`, `global_gain`).
5. **Huffman coding (`huffman`)** – encodes quantized spectral coefficients into variable-length codes using the standard MP3 Huffman tables defined in `tables`.
6. **Bitstream formatting (`bitstream`)** – writes MPEG frame headers, side information, main data, and padding into a `BitstreamWriter`, then flushes to a contiguous frame buffer.

`src/encoder.rs` implements this orchestration in methods like:

- `Mp3Encoder::encode_frame` / `encode_frame_interleaved` – one-shot encoding for full PCM frames.
- `encode_samples` – incremental encoding with internal buffering until a full frame is ready.
- `flush` – finalizes any buffered PCM (with zero-padding if needed) into a complete MP3 frame.
- `encode_frame_pipeline` – Rust translation of shine’s `shine_encode_buffer_internal`, including padding decisions, mean bits computation, MDCT, iteration loop, and bitstream formatting.

Internally, `Mp3Encoder` holds a `ShineGlobalConfig` (`src/shine_config.rs`) that mirrors shine’s global state structure; most low-level state (MDCT buffers, side info, bitstream state) hangs off this configuration.

### Module-level responsibilities

You can think of the crate’s modules in terms of shine’s corresponding C files:

- `config` – high-level encoder configuration and validation:
  - Maps `WaveConfig`/`MpegConfig` into MPEG version, samples per frame, allowed bitrates/sample rates, etc.
  - Provides helpers like `Config::samples_per_frame()` and `Config::mpeg_version()` used across the pipeline.
- `shine_config` – Rust equivalents of shine’s global structs:
  - Encapsulates the full encoder state (buffers, MDCT frequency arrays, side info structures, etc.).
  - Provides initialization (`ShineGlobalConfig::new` / `initialize`) that closely tracks shine’s startup path.
- `subband` – polyphase filterbank over 32 subbands, maintaining filter history and per-channel state.
- `mdct` – MDCT core with cosine tables and window coefficients; transforms `subband` output into MDCT coefficients and performs alias reduction.
- `quantization` – quantization loops and `GranuleInfo`/side-info details; contains logic for `big_values`, `count1`, region splitting, and step-size selection.
- `reservoir` – bit reservoir management (tracking frame/granule bit budgets and adjusting the reservoir at frame boundaries).
- `huffman` – Huffman table definitions and routines to encode `big_values` and `count1` regions according to MPEG’s tables.
- `bitstream` – low-level bit writer, MPEG header + side-info formatting, and frame finalization (including CRC when enabled).
- `error` – error type hierarchy used across the crate (configuration errors, input data issues, encoding failures, memory/state errors).

Binaries in `src/bin/` (`wav2mp3.rs`, `mp3_validator.rs`, `mp3_hexdump.rs`) are thin CLI frontends around the library, used for end-to-end experimentation and validation rather than as the primary public API surface.

### Testing and verification structure

There are two main testing layers:

1. **In-crate tests (`src/encoder.rs` and other modules):**
   - Standard unit tests validate configuration handling, frame encoding, incremental encoding, flushing, and reset behavior.
   - Extensive Proptest suites exercise `Mp3Encoder` across many valid configurations (sample rates, bitrates, channel modes, emphasis) and verify properties such as:
     - Successful initialization for any compatible configuration.
     - Correct `samples_per_frame` calculation for MPEG-1 vs MPEG-2/2.5.
     - Frame structure correctness (sync word, header fields like MPEG version and Layer bits).
     - Flush semantics (all buffered data emitted exactly once, subsequent flushes empty).

2. **External tests in `tests/` (see `tests/README.md`):**
   - `validation_comprehensive.rs` – end-to-end coverage across configurations, FFmpeg compatibility checks, and real audio processing.
   - `debug_comprehensive.rs` + `debug_tools.rs` – debugging harnesses (frame/pipeline analyzers, signal generators) and property tests targeting boundary conditions and performance.
   - `shine_reference_tests.rs` – direct comparisons against the shine encoder outputs (bitstream equality / numerical equivalence).
   - `big_values_validation_test.rs` / `big_values_comprehensive_test.rs` – enforcement of MP3 standard constraints (e.g., `big_values <= 288`) and stress tests on Huffman-related regions.
   - Additional focused tests like `mdct_algorithm_test.rs`, `quantization_algorithm_test.rs`, `frame_format_test.rs` validate specific algorithmic and bitstream details.

Property tests are configured consistently via `.kiro/steering/testing-guidelines.md` (e.g., `cases: 100`, `verbose: 0`, `max_shrink_iters: 0`, no persistence), and there is optional plumbing for shortened panic messages to keep logs readable.

## Project-specific guidelines for agents

The `.kiro/steering` documents act as project-specific “rules” for this codebase. Future agents should respect these constraints:

### 1. Shine parity is the primary invariant

From `.kiro/steering/coding-standards.md`:

- The Rust implementation must **strictly follow** the C `shine` reference located under `ref/shine/`:
  - Algorithms should be line-by-line equivalents (loop structure, conditionals, and evaluation order should match).
  - Function names, signatures (parameters, order, and types), and return semantics must correspond to shine, translated into idiomatic Rust naming.
  - Data structures (global config, granule info, side info, psychoacoustic state) should mirror shine’s structs in field meanings and array sizes.
  - All constants, lookup tables, and macro behavior must match shine exactly (values, lengths, and indexing schemes).
- “Optimizations” that change algorithmic behavior are not allowed unless they are explicitly reconciled with the shine implementation and corresponding tests.

**Agent implication:** When modifying any algorithmic code (MDCT, quantization, Huffman coding, reservoir, bitstream formatting), first locate and study the corresponding function in `ref/shine/src/lib/…`, then ensure the Rust change preserves the same behavior and observable outputs. Use the shine comparison tests in `tests/` as a backstop.

### 2. Maintain MP3 standard constraints explicitly

- Constraints derived from the MP3 spec (e.g., limits on `big_values`, region sizes, scalefactor band ranges) must be enforced in code, not only in tests.
- Tests such as `big_values_validation_test.rs` exist specifically to enforce these constraints; do not weaken or remove them to “fix” failures.

**Agent implication:** If a test around `big_values`, Huffman region boundaries, side-info fields, or bit reservoir usage fails, assume the implementation is wrong, not the test. Investigate the corresponding shine code and spec sections before changing any assertions.

### 3. Zero-warning, diagnostics-first workflow

- The project follows a **zero-warning** policy:
  - Code should be clean under `cargo check`, `cargo clippy`, and normal compilation (no warnings left unaddressed).
- Diagnostic tools (including Warp’s language server diagnostics) should be used early when making changes.

**Agent implication:** For any non-trivial set of edits, plan to:

1. Run `cargo check` and `cargo clippy --all-targets --all-features` after changes.
2. Address all warnings/clippy lints as part of the same change unless explicitly instructed otherwise.

### 4. Testing strategy and Proptest configuration

From `.kiro/steering/testing-guidelines.md` and `tests/README.md`:

- Use Proptest for algorithmic and boundary-condition verification; keep configurations consistent (cases ~100, `verbose = 0`, no shrink persistence) unless you have a strong project-specific reason to adjust them.
- Structure tests into logical submodules (`unit_tests`, `property_tests`, `integration_tests`) under a shared `#[cfg(test)] mod tests` when adding new tests to existing modules.
- Keep assertion messages short and in English; avoid verbose explanations or localized strings.

**Agent implication:** When adding new tests:

- Mirror the existing patterns around Proptest configuration and module organization.
- Prefer property tests that encode the high-level “correctness properties” listed in the design doc (e.g., preservation of invariants across transformations) over ad-hoc examples.

### 5. Respect module boundaries and file roles

- Core algorithmic responsibilities are already partitioned (`subband`, `mdct`, `quantization`, `reservoir`, `huffman`, `bitstream`, `tables`, `config`, `shine_config`, `encoder`).
- Each module is expected to stay relatively focused and manageable in size; cross-module logic should flow through well-defined interfaces rather than ad-hoc shared state.

**Agent implication:** When implementing new behavior or refactoring:

- Place new logic in the module that corresponds to its shine counterpart (or the conceptual MP3 pipeline stage) rather than introducing new cross-cutting helpers that blur responsibilities.
- Avoid introducing circular dependencies between modules; if you need shared types, prefer small shared data-structure modules over reaching “backwards” across the pipeline.

---

If you need additional context beyond this file, start with:

- `README.md` for a user-facing overview and basic usage.
- `.kiro/specs/rust-mp3-encoder/design.md` for detailed architecture and data-flow design.
- `.kiro/steering/coding-standards.md` and `.kiro/steering/testing-guidelines.md` for project-specific rules.
- `tests/README.md` for how the integration and comparison tests are structured.
