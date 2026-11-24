[English Documentation](README.md)

# shine-rs 库

基于 Shine 库的纯 Rust MP3 编码器库，提供完整的 MPEG Layer III 编码功能，严格遵循 Shine C 语言参考实现。

**项目地址**: https://github.com/wshon/shine-rs

## 特性

- **纯 Rust** — 以内存安全的方式实现 Shine 定点编码器
- **比特级一致** — 输出与 Shine C 参考实现逐字节相同的 MP3 码流
- **完整 MPEG Layer III** — 支持 MPEG-1 / MPEG-2 / MPEG-2.5 及所有标准采样率与比特率
- **声道模式** — 单声道、立体声、联合立体声、双声道
- **两级 API** — 高级安全接口与底层 Shine 兼容接口
- **可选诊断** — 通过 `diagnostics` 特性启用帧数据采集

## 模块

| 模块 | 说明 |
|---|---|
| `encoder` | 底层 Shine 兼容 API：`ShineConfig`、`ShineWave`、`ShineMpeg` |
| `mp3_encoder` | 高级 API：`Mp3Encoder`、`Mp3EncoderConfig`、`StereoMode` |
| `error` | 错误类型：`EncoderError`、`ConfigError`、`InputDataError`、`EncodingError` |
| `types` | 共享配置类型（`ShineGlobalConfig`） |
| `bitstream` | MP3 比特流写入 |
| `subband` | 32 频带子带分析滤波器 |
| `mdct` | 修正离散余弦变换 |
| `quantization` | 量化循环与比特率控制 |
| `huffman` | Huffman 编码 |
| `reservoir` | 比特池管理 |
| `tables` | MP3 编码查找表 |
| `diagnostics` | 帧数据采集（需要 `diagnostics` 特性） |

## 高级 API

```rust
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};

// 通过 builder 风格的方法设置配置
let config = Mp3EncoderConfig::new()
    .sample_rate(44100)
    .bitrate(128)
    .channels(2)
    .stereo_mode(StereoMode::Stereo);

let mut encoder = Mp3Encoder::new(config)?;

// PCM 数据必须是交错的（左、右、左、右...），
// 每次编码传入恰好 samples_per_frame() 个样本。
let pcm = vec![0i16; encoder.samples_per_frame()];
let frames = encoder.encode_interleaved(&pcm)?; // Vec<Vec<u8>>

let tail = encoder.finish()?;
```

`Mp3Encoder` 还提供 `config()`、`shine_config()`、`encode_separate_channels()`、`encode_pcm_to_mp3()` 和 `is_finished()`。

## 底层 API（Shine 兼容）

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

// SAFETY: pcm 至少包含 samples_per_pass * channels 个有效样本。
let (chunk, written) = unsafe { shine_encode_buffer_interleaved(&mut global, pcm.as_ptr()) }?;
let (tail, tail_len) = shine_flush(&mut global);
shine_close(global);
```

## 配置

`Mp3EncoderConfig` 字段及其 setter：

| 字段 | 类型 | Setter | 说明 |
|---|---|---|---|
| `sample_rate` | `u32` | `.sample_rate(44100)` | 8000–48000 Hz |
| `bitrate` | `u32` | `.bitrate(128)` | 8–320 kbps |
| `channels` | `u8` | `.channels(2)` | 1 或 2 |
| `stereo_mode` | `StereoMode` | `.stereo_mode(StereoMode::Stereo)` | `Stereo`、`JointStereo`、`DualChannel`、`Mono` |
| `copyright` | `bool` | `.copyright(false)` | 版权标志 |
| `original` | `bool` | `.original(true)` | 原创标志 |

`StereoMode` 以及支持的采样率/比特率常量（`SUPPORTED_SAMPLE_RATES`、`SUPPORTED_BITRATES`）均在 crate 根重新导出。

| MPEG 版本 | 采样率 (Hz) | 比特率范围 (kbps) |
|---|---|---|
| MPEG-1 | 32000, 44100, 48000 | 32–320 |
| MPEG-2 | 16000, 22050, 24000 | 8–160 |
| MPEG-2.5 | 8000, 11025, 12000 | 8–64 |

## 错误处理

所有公开的可失败 API 都返回 `error` 模块中的错误类型，并在 crate 根重新导出：

```rust
use shine_rs::{ConfigError, EncoderError, InputDataError};

// EncoderError 是 Mp3Encoder 方法返回的顶层错误，
// 它包装了 ConfigError、InputDataError 和 EncodingError。
match Mp3Encoder::new(config) {
    Ok(encoder) => { /* ... */ },
    Err(EncoderError::Config(ConfigError::UnsupportedSampleRate(rate))) => {
        eprintln!("不支持的采样率: {rate} Hz");
    },
    Err(EncoderError::InputData(InputDataError::EmptyInput)) => {
        eprintln!("未提供 PCM 数据");
    },
    Err(e) => eprintln!("编码失败: {e}"),
}
```

`EncodingError` 的变体包括 `QuantizationFailed`、`HuffmanError(String)`、`BitstreamError(String)`、`MdctError(String)`、`SubbandError(String)`、`InvalidInputLength`、`InvalidDataLength`、`InvalidChannelIndex`、`BitReservoirOverflow` 和 `ValidationError(String)`。

## 特性

| 特性 | 说明 |
|---|---|
| `default` | 仅核心编码功能 |
| `diagnostics` | 启用 `diagnostics` 模块及帧数据采集辅助函数 |

```toml
[dependencies]
shine-rs = { version = "0.1", features = ["diagnostics"] }
```

启用 `diagnostics` 时，crate 根重新导出 `get_current_frame_number()`、`get_next_frame_number()` 和 `reset_frame_counter()`；未启用时提供空实现的桩函数，因此相同的调用始终可以编译。

## 性能

`shine-rs` 与 Shine C 参考实现逐字节一致，同时速度更快。基准测试比较纯编码速度（音频先解码到内存，不含文件 I/O），测试环境为 Linux，每组配置 5 次取中位，日期 2026-11-19：

| 音频 | 比特率 | shine-rs | Shine C |
|---|---|---|---|
| 15 秒立体声 | 128k | 149.0x | 119.2x |
| 15 秒立体声 | 192k | 133.2x | 116.4x |
| 15 秒立体声 | 320k | 140.1x | 113.6x |
| 3 秒立体声 | 128k | 146.6x | 110.4x |
| 3 秒立体声 | 192k | 136.7x | 108.0x |
| 3 秒立体声 | 320k | 132.5x | 105.3x |
| 500KB 音频 | 128k | 144.6x | 121.3x |
| 500KB 音频 | 192k | 136.8x | 115.2x |
| 500KB 音频 | 320k | 129.5x | 105.5x |
| 录音 | 128k | 287.5x | 218.4x |
| 录音 | 192k | 257.3x | 208.6x |
| 录音 | 320k | 259.2x | 192.0x |

所有配置下 shine-rs 都更快，幅度为 1.14–1.35 倍。差距来自 MDCT 热循环中用回绕运算替代饱和加法，以及量化器直接复用预计算的 `xrabs` 值，不再每次调用都重算 `labs`。

## 测试

```bash
# 单元测试与参考验证
cargo test

# 启用诊断特性
cargo test --features diagnostics

# 参考码流验证
cargo test encoder_validation_cicd
```

## 许可证

LGPL-2.0。详见 [LICENSE](https://github.com/wshon/shine-rs/blob/main/LICENSE)。
