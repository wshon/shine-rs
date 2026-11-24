[中文文档](README_zh_CN.md)

# shine-rs Library

A pure Rust MP3 encoder library based on the Shine library. It provides complete MPEG Layer III encoding, strictly following the Shine C reference implementation.

**Repository**: https://github.com/wshon/shine-rs

## Features

- **Pure Rust** — memory-safe implementation of the Shine fixed-point encoder
- **Bit-exact output** — byte-identical MP3 bitstreams to the Shine C reference
- **Full MPEG Layer III** — MPEG-1 / MPEG-2 / MPEG-2.5, all standard sample rates and bitrates
- **Channel modes** — mono, stereo, joint stereo, dual channel
- **Two API levels** — a high-level safe interface and a low-level Shine-compatible interface
- **Optional diagnostics** — frame data collection behind the `diagnostics` feature

## Modules

| Module | Description |
|---|---|
| `encoder` | Low-level Shine-compatible API: `ShineConfig`, `ShineWave`, `ShineMpeg` |
| `mp3_encoder` | High-level API: `Mp3Encoder`, `Mp3EncoderConfig`, `StereoMode` |
| `error` | Error types: `EncoderError`, `ConfigError`, `InputDataError`, `EncodingError` |
| `types` | Shared configuration types (`ShineGlobalConfig`) |
| `bitstream` | MP3 bitstream writing |
| `subband` | 32-band subband analysis filter |
| `mdct` | Modified Discrete Cosine Transform |
| `quantization` | Quantization loop and bitrate control |
| `huffman` | Huffman coding |
| `reservoir` | Bit reservoir management |
| `tables` | Lookup tables for MP3 encoding |
| `diagnostics` | Frame data collection (requires the `diagnostics` feature) |

## High-Level API

```rust
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

// Build a configuration with the builder-style setters
let config = Mp3EncoderConfig::new()
    .sample_rate(44100)
    .bitrate(128)
    .channels(2)
    .stereo_mode(StereoMode::Stereo);

let mut encoder = Mp3Encoder::new(config)?;

// PCM samples must be interleaved (left, right, left, right, ...);
// feed exactly samples_per_frame() samples per encode call.
let pcm = vec![0i16; encoder.samples_per_frame()];
let frames = encoder.encode_interleaved(&pcm)?; // Vec<Vec<u8>>

let tail = encoder.finish()?;
```

`Mp3Encoder` also exposes `config()`, `shine_config()`, `encode_separate_channels()`, `encode_pcm_to_mp3()`, and `is_finished()`.

## Low-Level API (Shine Compatible)

```rust
use shine_rs::{
    ShineConfig, ShineWave,
    shine_initialise, shine_encode_buffer_interleaved, shine_flush,
    shine_close,
};

let mut cfg = ShineConfig::default();
cfg.wave.channels = 2;
cfg.wave.samplerate = 44100;
cfg.mpeg.bitr = 128;

let mut global = shine_initialise(&cfg)?;
let samples = shine_rs::encoder::shine_samples_per_pass(&global) as usize * 2;
let pcm = vec![0i16; samples];

// SAFETY: pcm contains at least samples_per_pass * channels valid samples.
let (chunk, written) = unsafe { shine_encode_buffer_interleaved(&mut global, pcm.as_ptr()) }?;
let (tail, tail_len) = shine_flush(&mut global);
shine_close(global);
```

## Configuration

`Mp3EncoderConfig` fields and builder setters:

| Field | Type | Setter | Notes |
|---|---|---|---|
| `sample_rate` | `u32` | `.sample_rate(44100)` | 8000–48000 Hz |
| `bitrate` | `u32` | `.bitrate(128)` | 8–320 kbps |
| `channels` | `u8` | `.channels(2)` | 1 or 2 |
| `stereo_mode` | `StereoMode` | `.stereo_mode(StereoMode::Stereo)` | `Stereo`, `JointStereo`, `DualChannel`, `Mono` |
| `copyright` | `bool` | `.copyright(false)` | copyright flag |
| `original` | `bool` | `.original(true)` | original flag |

`StereoMode` and the supported sample-rate/bitrate constants (`SUPPORTED_SAMPLE_RATES`, `SUPPORTED_BITRATES`) are re-exported at the crate root.

| MPEG version | Sample rate (Hz) | Bitrate (kbps) |
|---|---|---|
| MPEG-1 | 32000, 44100, 48000 | 32–320 |
| MPEG-2 | 16000, 22050, 24000 | 8–160 |
| MPEG-2.5 | 8000, 11025, 12000 | 8–64 |

## Error Handling

All public fallible APIs return errors from the `error` module, re-exported at the crate root:

```rust
use shine_rs::{ConfigError, EncoderError, InputDataError};

// EncoderError is the top-level error returned by Mp3Encoder methods.
// It wraps ConfigError, InputDataError, and EncodingError.
match Mp3Encoder::new(config) {
    Ok(encoder) => { /* ... */ },
    Err(EncoderError::Config(ConfigError::UnsupportedSampleRate(rate))) => {
        eprintln!("Unsupported sample rate: {rate} Hz");
    },
    Err(EncoderError::InputData(InputDataError::EmptyInput)) => {
        eprintln!("No PCM data provided");
    },
    Err(e) => eprintln!("Encoding failed: {e}"),
}
```

`EncodingError` variants include `QuantizationFailed`, `HuffmanError(String)`, `BitstreamError(String)`, `MdctError(String)`, `SubbandError(String)`, `InvalidInputLength`, `InvalidDataLength`, `InvalidChannelIndex`, `BitReservoirOverflow`, and `ValidationError(String)`.

## Features

| Feature | Description |
|---|---|
| `default` | Core encoding only |
| `diagnostics` | Enables the `diagnostics` module and frame-data collection helpers |

```toml
[dependencies]
shine-rs = { version = "0.1", features = ["diagnostics"] }
```

With `diagnostics` enabled, the crate root re-exports `get_current_frame_number()`, `get_next_frame_number()`, and `reset_frame_counter()`; without it, no-op stubs are provided so the same calls always compile.

## Performance

`shine-rs` matches the Shine C reference bit-for-bit while running faster. The benchmark compares pure encoding speed (audio decoded into memory first; file I/O excluded), measured on Linux, median of 5 runs per configuration, 2026-11-19:

| Audio | Bitrate | shine-rs | Shine C |
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

shine-rs is faster in every configuration, by 1.14–1.35x. The gap comes from wrapping arithmetic in the MDCT hot loop and reusing precomputed `xrabs` values in the quantizer instead of recomputing `labs` on every call.

## Testing

```bash
# Unit tests and reference validation
cargo test

# With diagnostics enabled
cargo test --features diagnostics

# Reference bitstream validation
cargo test encoder_validation_cicd
```

## License

LGPL-2.0. See [LICENSE](https://github.com/wshon/shine-rs/blob/main/LICENSE).
