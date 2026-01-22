//! Benchmark tests for the MP3 encoder
//!
//! These benchmarks measure the performance of various encoder components
//! and the overall encoding pipeline.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_mp3_encoder::{Mp3Encoder, Config};

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

criterion_group!(benches, benchmark_encoder_creation, benchmark_config_validation);
criterion_main!(benches);