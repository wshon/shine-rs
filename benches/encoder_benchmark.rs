//! Benchmark tests for the MP3 encoder
//!
//! These benchmarks measure the performance of various encoder components
//! and the overall encoding pipeline.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::config::{WaveConfig, MpegConfig, StereoMode, Emphasis};
use rust_mp3_encoder::shine_config::ShineGlobalConfig;
use rust_mp3_encoder::mdct::shine_mdct_sub;

fn benchmark_encoder_creation(c: &mut Criterion) {
    c.bench_function("encoder_creation", |b| {
        b.iter(|| {
            let config = black_box(Config::new());
            let _encoder = Mp3Encoder::new(config).unwrap();
        })
    });
}

fn benchmark_config_validation(c: &mut Criterion) {
    let config = Config::new();
    
    c.bench_function("config_validation", |b| {
        b.iter(|| {
            black_box(config.validate()).unwrap();
        })
    });
}

fn create_test_config() -> ShineGlobalConfig {
    let config = Config {
        wave: WaveConfig {
            channels: rust_mp3_encoder::config::Channels::Stereo,
            sample_rate: 44100,
        },
        mpeg: MpegConfig {
            mode: StereoMode::Stereo,
            bitrate: 128,
            emphasis: Emphasis::None,
            copyright: false,
            original: true,
        },
    };
    
    let mut shine_config = ShineGlobalConfig::new(config).unwrap();
    shine_config.initialize().unwrap();
    shine_config
}

fn benchmark_mdct_transform(c: &mut Criterion) {
    let mut config = create_test_config();
    
    // Fill with realistic test data
    for ch in 0..2 {
        for gr in 0..2 {
            for t in 0..18 {
                for sb in 0..32 {
                    config.l3_sb_sample[ch][gr][t][sb] = 1000;
                    config.l3_sb_sample[ch][gr + 1][t][sb] = 1000;
                }
            }
        }
    }
    
    c.bench_function("mdct_transform", |b| {
        b.iter(|| {
            let mut test_config = config.clone();
            shine_mdct_sub(black_box(&mut test_config), black_box(1));
            black_box(test_config.mdct_freq);
        })
    });
}

criterion_group!(
    benches, 
    benchmark_encoder_creation, 
    benchmark_config_validation,
    benchmark_mdct_transform
);
criterion_main!(benches);