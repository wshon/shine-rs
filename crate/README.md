[中文文档](https://github.com/wshon/shine-rs/blob/main/crate/README_zh_CN.md)

# Shine-RS

A pure Rust MP3 encoder implementation based on the Shine library. This project strictly follows the Shine C reference implementation, providing complete MP3 Layer III encoding with support for various sample rates, bitrates, and channel configurations.

**Repository**: https://github.com/wshon/shine-rs

[![License: LGPL-2.0](https://img.shields.io/badge/License-LGPL%202.0-blue.svg)](https://github.com/wshon/shine-rs/blob/main/LICENSE)
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

> 💡 See [High-Level API Guide](https://github.com/wshon/shine-rs/blob/main/docs/HIGH_LEVEL_API.md) for advanced usage.

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

- [Project Structure](https://github.com/wshon/shine-rs/blob/main/docs/PROJECT_STRUCTURE.md) — detailed project organization
- [Test Data Framework](https://github.com/wshon/shine-rs/blob/main/docs/TEST_DATA_FRAMEWORK.md) — test data collection and validation
- [Frame Limit Feature](https://github.com/wshon/shine-rs/blob/main/docs/FRAME_LIMIT_FEATURE.md) — frame limit for debugging and testing
- [Frame Limit Quick Reference](https://github.com/wshon/shine-rs/blob/main/docs/FRAME_LIMIT_QUICK_REFERENCE.md) — quick usage of frame limit
- [High-Level API Guide](https://github.com/wshon/shine-rs/blob/main/docs/HIGH_LEVEL_API.md) — advanced API usage
- [Logging System](https://github.com/wshon/shine-rs/blob/main/docs/LOGGING_SYSTEM.md) — debug logging configuration
- [Audio File Standardization](https://github.com/wshon/shine-rs/blob/main/docs/AUDIO_FILES_STANDARDIZATION.md) — test audio organization
- [Verification Record](https://github.com/wshon/shine-rs/blob/main/docs/VERIFICATION_RECORD.md) — verification against Shine
- [Performance Benchmark](https://github.com/wshon/shine-rs/blob/main/tests/encoder_benchmark.rs) — Shine-RS vs Shine comparison

## License

LGPL-2.0. See [LICENSE](https://github.com/wshon/shine-rs/blob/main/LICENSE).

## Contributing

Contributions are welcome. Before submitting code, please ensure:

1. Follow the project coding standards (strictly match the Shine implementation)
2. All tests pass, including Shine reference validation
3. Code passes `cargo clippy` with no warnings
4. Add tests for new features

## Acknowledgments

Based on [Shine](https://github.com/toots/shine) MP3 encoder. Thanks to Gabriel Bouvigne (original core), Pete Everett (fixed-point port), Patrick Roberts (multi-platform library), and the Savonet team for long-term maintenance.

Shine-RS strictly follows Shine's core algorithm, continuing its fixed-point encoding advantages while ensuring MP3 quality and ISO/IEC 11172-3 compliance, leveraging Rust's language features for significant performance improvements.

---

## Library Architecture

### Core Modules

- **`encoder`** - Main encoder module, provides low-level Shine-compatible interface
- **`mp3_encoder`** - High-level encoder interface, provides friendlier Rust API
- **`config`** - Encoding configuration management
- **`error`** - Error type definitions

### Algorithm Modules

- **`subband`** - 32-band subband analysis filter
- **`mdct`** - Modified Discrete Cosine Transform
- **`quantization`** - Quantization loop and bitrate control
- **`huffman`** - Huffman encoder
- **`bitstream`** - MP3 bitstream writer
- **`reservoir`** - Bit reservoir management

### Data and Lookup Tables

- **`tables`** - All lookup tables required for MP3 encoding
- **`psychoacoustic`** - Psychoacoustic model (simplified)

## API Usage

### High-level Interface (Recommended)

```rust
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

// Create config
let config = Mp3EncoderConfig::new()
    .sample_rate(44100)
    .bitrate(128)
    .channels(2)
    .stereo_mode(StereoMode::Stereo);

// Create encoder
let mut encoder = Mp3Encoder::new(config)?;

// Encode audio data (interleaved format)
let pcm_samples = vec![0i16; encoder.samples_per_frame()];
let mp3_data = encoder.encode_interleaved(&pcm_samples)?;

// Finish encoding
let final_data = encoder.finish()?;
```

### Low-level Interface (Shine Compatible)

```rust
use shine_rs::{
    ShineConfig, ShineWave, ShineMpeg,
    shine_initialise, shine_encode_buffer_interleaved,
    shine_flush, shine_close
};

// Initialize config
let mut config = ShineConfig::default();
config.wave.samplerate = 44100;
config.wave.channels = 2;
config.mpeg.bitr = 128;

// Initialize encoder
shine_initialise(&mut config);

// Encode data
let pcm_data = vec![0i16; config.samples_per_pass()];
let mp3_data = shine_encode_buffer_interleaved(&mut config, &pcm_data);

// Finish encoding
let final_data = shine_flush(&mut config);
shine_close(&mut config);
```

## Configuration Options

### Mp3EncoderConfig

```rust
pub struct Mp3EncoderConfig {
    sample_rate: u32,      // Sample rate (8000-48000 Hz)
    bitrate: u32,          // Bitrate (8-320 kbps)
    channels: u16,         // Number of channels (1-2)
    stereo_mode: StereoMode, // Stereo mode
    copyright: bool,       // Copyright flag
    original: bool,        // Original flag
    emphasis: Emphasis,    // Emphasis
}
```

### Stereo Modes

```rust
pub enum StereoMode {
    Stereo,      // Stereo
    JointStereo, // Joint Stereo
    DualChannel, // Dual Channel
    Mono,        // Mono
}
```

### Supported Sample Rate and Bitrate Combinations

| MPEG Version | Sample Rate (Hz) | Bitrate Range (kbps) |
|--------------|------------------|---------------------|
| MPEG-1       | 32000, 44100, 48000 | 32-320 |
| MPEG-2       | 16000, 22050, 24000 | 8-160  |
| MPEG-2.5     | 8000, 11025, 12000  | 8-64   |

## Error Handling

```rust
use shine_rs::EncodingError;

match encoder.encode_interleaved(&pcm_data) {
    Ok(mp3_data) => {
        // Process encoded data
    }
    Err(EncodingError::InvalidSampleRate(rate)) => {
        eprintln!("Unsupported sample rate: {}", rate);
    }
    Err(EncodingError::InvalidBitrate(bitrate)) => {
        eprintln!("Unsupported bitrate: {}", bitrate);
    }
    Err(EncodingError::InvalidChannelCount(channels)) => {
        eprintln!("Unsupported channel count: {}", channels);
    }
    Err(e) => {
        eprintln!("Encoding error: {}", e);
    }
}
```

## Memory Management

### Buffer Sizes

```rust
// Get PCM samples required per frame
let samples_per_frame = encoder.samples_per_frame(); // Usually 1152

// Get maximum MP3 frame size
let max_mp3_frame_size = encoder.max_mp3_frame_size(); // Depends on bitrate

// Pre-allocate buffers
let mut pcm_buffer = vec![0i16; samples_per_frame];
let mut mp3_buffer = Vec::with_capacity(max_mp3_frame_size);
```

### Batch Processing

```rust
// Process large amounts of audio data
let chunk_size = encoder.samples_per_frame();
for chunk in pcm_data.chunks(chunk_size) {
    if chunk.len() == chunk_size {
        let mp3_frame = encoder.encode_interleaved(chunk)?;
        output.extend_from_slice(&mp3_frame);
    } else {
        // Handle last incomplete chunk
        let mut padded_chunk = vec![0i16; chunk_size];
        padded_chunk[..chunk.len()].copy_from_slice(chunk);
        let mp3_frame = encoder.encode_interleaved(&padded_chunk)?;
        output.extend_from_slice(&mp3_frame);
    }
}
```

## Debug Features

### Enabling Diagnostics Feature

```toml
[dependencies]
shine-rs = { version = "0.1", features = ["diagnostics"] }
```

```rust
#[cfg(feature = "diagnostics")]
{
    // Access internal diagnostic data
    let diagnostics = encoder.get_diagnostics();
    println!("MDCT coefficients: {:?}", diagnostics.mdct_coefficients);
    println!("Quantization params: {:?}", diagnostics.quantization_params);
}
```

### Logging Output

```rust
use log::{info, debug};

// Enable logging
env_logger::init();

// Detailed logging during encoding
debug!("Starting encode frame {}", frame_number);
info!("Encoding complete, output {} bytes", mp3_data.len());
```

## Performance Optimization

### Pre-allocated Buffers

```rust
// Avoid repeated allocations
let mut encoder = Mp3Encoder::new(config)?;
let mut pcm_buffer = vec![0i16; encoder.samples_per_frame()];
let mut mp3_output = Vec::new();

loop {
    // Reuse buffers
    if let Some(samples) = read_audio_samples(&mut pcm_buffer) {
        let mp3_frame = encoder.encode_interleaved(&pcm_buffer[..samples])?;
        mp3_output.extend_from_slice(&mp3_frame);
    } else {
        break;
    }
}
```

### Batch Processing

```rust
// Process multiple frames to reduce function call overhead
const BATCH_SIZE: usize = 10;
let frame_size = encoder.samples_per_frame();
let batch_size = frame_size * BATCH_SIZE;

for batch in pcm_data.chunks(batch_size) {
    for frame in batch.chunks(frame_size) {
        let mp3_frame = encoder.encode_interleaved(frame)?;
        output.extend_from_slice(&mp3_frame);
    }
}
```

## Thread Safety

This library is not thread-safe. For multi-threaded use, create separate encoder instances per thread:

```rust
use std::thread;
use std::sync::mpsc;

// Create separate encoder for each thread
let handles: Vec<_> = (0..num_threads).map(|_| {
    let config = config.clone();
    thread::spawn(move || {
        let mut encoder = Mp3Encoder::new(config).unwrap();
        // Process audio data...
    })
}).collect();
```

## Correspondence with Shine C Implementation

| Rust Function | Shine C Function | Description |
|---------------|------------------|-------------|
| `Mp3Encoder::new()` | `shine_initialise()` | Initialize encoder |
| `encode_interleaved()` | `shine_encode_buffer_interleaved()` | Encode interleaved audio data |
| `finish()` | `shine_flush()` + `shine_close()` | Finish encoding and cleanup |
| `samples_per_frame()` | `shine_samples_per_pass()` | Samples per frame |

## Testing and Verification

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_*

# Enable diagnostics feature tests
cargo test --features diagnostics

# Performance benchmarks
cargo bench
```

## Build Features

- `default` - Standard functionality
- `diagnostics` - Enable internal diagnostic data access
- `logging` - Enable detailed logging output

```toml
[dependencies]
shine-rs = { version = "0.1", features = ["diagnostics", "logging"] }
```
