[中文文档](README_zh_CN.md)

# shine-rs Library

This is the core library implementation of the shine-rs MP3 encoder. The library provides complete MP3 Layer III encoding functionality, strictly following the Shine C reference implementation.

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
