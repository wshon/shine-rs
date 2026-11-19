[中文文档](README_zh_CN.md)

# Shine-RS

A pure Rust MP3 encoder implementation based on the Shine library. This project strictly follows the Shine C reference implementation, providing complete MP3 Layer III encoding with support for various sample rates, bitrates, and channel configurations.

**Repository**: https://github.com/wshon/shine-rs

[![License: LGPL-2.0](https://img.shields.io/badge/License-LGPL%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

## Features

- 🦀 **Pure Rust** — memory-safe and performant
- 🎯 **Shine-compatible** — algorithms match Shine C exactly for identical output
- 🎵 **Full MP3 Layer III** — complete encoding pipeline
- ⚡ **Great performance** — outperforms Shine C by 1.14–1.35x across all tested configurations
- 🔧 **Flexible** — multiple sample rates, bitrates, and channel modes
- 📊 **Standard-compliant** — ISO/IEC 11172-3
- 🧪 **Well-tested** — unit tests, integration tests, and reference validation
- 🛠️ **CLI tool** — WAV to MP3 conversion and test-data collection utilities
- 📋 **Debug support** — optional logging and frame limit feature

## Supported Formats

### Sample Rates
- **MPEG-1**: 32000, 44100, 48000 Hz
- **MPEG-2**: 16000, 22050, 24000 Hz
- **MPEG-2.5**: 8000, 11025, 12000 Hz

### Bitrates
- **MPEG-1**: 32-320 kbps
- **MPEG-2**: 8-160 kbps
- **MPEG-2.5**: 8-64 kbps

### Channel Modes
- Mono
- Stereo
- Joint Stereo
- Dual Channel

## Quick Start

### CLI Tool

```bash
# Basic: WAV to MP3
cargo run tests/audio/inputs/basic/sample-3s.wav output.mp3

# Specify bitrate and stereo mode
cargo run input.wav output.mp3 128 stereo

# Debug mode: limit frame count
cargo run input.wav output.mp3 --max-frames 10

# Verbose output
cargo run input.wav output.mp3 --verbose
```

### As a Library

```toml
[dependencies]
shine-rs = { git = "https://github.com/wshon/shine-rs" }
```

```rust
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create encoder configuration
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .bitrate(128)
        .channels(2)
        .stereo_mode(StereoMode::Stereo);

    // Create encoder
    let mut encoder = Mp3Encoder::new(config)?;

    // Encode PCM data
    let pcm_data = vec![0i16; encoder.samples_per_frame()];
    let mp3_frames = encoder.encode_interleaved(&pcm_data)?;

    // Finish encoding
    let final_data = encoder.finish()?;

    println!("Done! Generated {} bytes of MP3 data", final_data.len());
    Ok(())
}
```

> 💡 See [High-Level API Guide](docs/HIGH_LEVEL_API.md) for advanced usage.

## Project Structure

```
shine-rs/
├── 📁 crate/                    # Published library (shine-rs)
│   ├── src/                     # Core MP3 encoder
│   │   ├── bitstream.rs         # Bitstream handling
│   │   ├── encoder.rs           # Low-level API
│   │   ├── mp3_encoder.rs       # High-level API
│   │   ├── huffman.rs           # Huffman coding
│   │   ├── mdct.rs              # MDCT transform
│   │   ├── quantization.rs      # Quantization loop
│   │   ├── subband.rs           # Subband analysis
│   │   ├── tables.rs            # Lookup tables
│   │   ├── reservoir.rs         # Bit reservoir
│   │   ├── error.rs             # Error handling
│   │   ├── types.rs             # Type definitions
│   │   └── lib.rs               # Library entry
│   ├── tests/                   # Library unit tests
│   └── Cargo.toml               # Library config
├── 📁 src/                      # CLI tool
│   ├── main.rs                  # WAV to MP3 CLI
│   ├── util.rs                  # Utilities (WAV reading, PCM processing)
│   └── lib.rs                   # CLI library entry
├── 📁 tests/                    # Integration tests & benchmarks
│   ├── encoder_benchmark.rs     # Performance benchmark
│   ├── encoder_validation_*.rs  # Encoder validation tests
│   └── audio/                   # Test audio files
├── 📁 ref/shine/                # Shine C reference
├── 📁 docs/                     # Documentation
└── 📁 scripts/                  # Helper scripts
```

### Encoding Pipeline

```
PCM input → Subband Filter → MDCT → Quantization Loop → Huffman → Bitstream output
```

Each step strictly follows the Shine C implementation to ensure correctness and output consistency.

## Development Status

✅ Complete MP3 encoding implementation:

- [x] Project structure and infrastructure
- [x] Configuration management
- [x] Lookup tables and constants
- [x] Bitstream writer
- [x] Subband filter (32-band analysis)
- [x] MDCT (Modified Discrete Cosine Transform)
- [x] Quantization loop (bitrate control)
- [x] Huffman encoder
- [x] Main encoder integration
- [x] Output verified against Shine (SHA256 match)
- [x] **Performance** — outperforms Shine C by 1.14–1.35x across all tested configurations
- [x] **Code quality** — zero compiler warnings, all clippy checks pass
- [x] **Benchmark suite** — complete Shine-RS vs Shine performance comparison

### Quality Assurance

- **Algorithm verification**: All core algorithms are compared line-by-line against Shine C
- **Output consistency**: Generated MP3 files are identical to Shine output (SHA256 match)
- **Comprehensive testing**: Unit tests, integration tests, property tests, regression tests, benchmarks
- **Standard compliance**: ISO/IEC 11172-3
- **Code quality**: Zero clippy warnings
- **Performance validation**: Verified with real audio files to ensure optimizations don't affect correctness

## Build & Test

```bash
# Build
cargo build --release

# Run all tests
cargo test

# Run performance benchmark (pure-encode, needs shine-ref source)
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh

# Run integration tests
cargo test --test integration_full_pipeline_validation

# Run validation tests
cargo test encoder_validation_cicd

# CLI tool
cargo run --release tests/audio/inputs/basic/sample-3s.wav output.mp3
```

### Performance Benchmark

```bash
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh
```

Results are shown under [Performance and Compatibility](#performance-and-compatibility).

### Debug & Development

```bash
# Enable debug logging
RUST_LOG=debug cargo run --release input.wav output.mp3

# Limit encoding frames (debugging)
cargo run --release input.wav output.mp3 --max-frames 5

# Run performance benchmark (pure-encode, needs shine-ref source)
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh

# Verbose mode
cargo run --release input.wav output.mp3 --verbose
```

## Performance and Compatibility

### 🚀 Performance

This project achieves excellent encoding performance while keeping the algorithm exactly identical to Shine C.

#### Benchmark

Compares the **pure encoding speed** of Shine-RS and Shine C: both decode audio into memory first, then time only the encoding core (`shine_initialise` + encode loop + `flush`), excluding file reads and writes.

Reproduce (needs shine-ref source):

```bash
SHINE_REF=ref/shine bash scripts/bench/build_and_run.sh
```

Linux pure-encode speedup (2026-11-19, median of 5 runs per config; excludes file I/O):

| Audio | Bitrate | Shine-RS | Shine C |
|---|---|---|---|
| 15s stereo | 128k | 149.0x | 119.2x |
| 15s stereo | 192k | 133.2x | 116.4x |
| 15s stereo | 320k | 140.1x | 113.6x |
| 3s stereo | 128k | 146.6x | 110.4x |
| 3s stereo | 192k | 136.7x | 108.0x |
| 3s stereo | 320k | 132.5x | 105.3x |
| 500KB audio | 128k | 144.6x | 121.3x |
| 500KB audio | 192k | 136.8x | 115.2x |
| 500KB audio | 320k | 129.5x | 105.5x |
| Voice recording | 128k | 287.5x | 218.4x |
| Voice recording | 192k | 257.3x | 208.6x |
| Voice recording | 320k | 259.2x | 192.0x |

Shine-RS is faster in every configuration, by 1.14–1.35x. The gap comes from replacing a saturating add with wrapping arithmetic in the MDCT hot loop, and from reusing the precomputed absolute values (`xrabs`) in the quantizer instead of recomputing `labs` on every call.

#### Technical Advantages

- **Algorithm fidelity**: byte-identical output to Shine C across all reference configurations
- **No overflow branch in the MDCT hot loop**: wrapping add matches C's `+=`, verified under debug overflow checks
- **No per-call abs in the quantizer hot loop**: reuses the `xrabs` values precomputed once per granule (identical values, including the saturating `i32::MIN` case)
- **LLVM LTO**: full link-time optimization, `codegen-units=1`, native CPU tuning

### Algorithm Characteristics

- **Optimized algorithms**: ported from Shine's fixed-point C implementation
- **Encoding speed**: faster than Shine C (1.14–1.35x, see benchmark above)
- **Output**: byte-identical to Shine C, verified by SHA256 against reference files

### Compatibility

Generated MP3 files are compatible with:
- FFmpeg/libmp3lame
- Windows Media Player
- VLC Media Player
- All standard MP3 players

### Quality Guarantees

- **Bit-exact**: Produces identical MP3 bitstreams to Shine
- **Standard compliance**: ISO/IEC 11172-3
- **Regression testing**: Prevents algorithm changes from introducing issues
- **Continuous verification**: Every change validated against Shine output

## Documentation

- [Project Structure](docs/PROJECT_STRUCTURE.md) — detailed project organization
- [Test Data Framework](docs/TEST_DATA_FRAMEWORK.md) — test data collection and validation
- [Frame Limit Feature](docs/FRAME_LIMIT_FEATURE.md) — frame limit for debugging and testing
- [Frame Limit Quick Reference](docs/FRAME_LIMIT_QUICK_REFERENCE.md) — quick usage of frame limit
- [High-Level API Guide](docs/HIGH_LEVEL_API.md) — advanced API usage
- [Logging System](docs/LOGGING_SYSTEM.md) — debug logging configuration
- [Audio File Standardization](docs/AUDIO_FILES_STANDARDIZATION.md) — test audio organization
- [Verification Record](docs/VERIFICATION_RECORD.md) — verification against Shine
- [Performance Benchmark](tests/encoder_benchmark.rs) — Shine-RS vs Shine comparison

## License

LGPL-2.0. See [LICENSE](LICENSE).

## Contributing

Contributions are welcome. Before submitting code, please ensure:

1. Follow the project coding standards (strictly match the Shine implementation)
2. All tests pass, including Shine reference validation
3. Code passes `cargo clippy` with no warnings
4. Add tests for new features

## Acknowledgments

Based on [Shine](https://github.com/toots/shine) MP3 encoder. Thanks to Gabriel Bouvigne (original core), Pete Everett (fixed-point port), Patrick Roberts (multi-platform library), and the Savonet team for long-term maintenance.

Shine-RS strictly follows Shine's core algorithm, continuing its fixed-point encoding advantages while ensuring MP3 quality and ISO/IEC 11172-3 compliance, leveraging Rust's language features for significant performance improvements.
