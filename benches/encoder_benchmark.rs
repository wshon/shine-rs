//! Benchmark tests for the MP3 encoder
//!
//! These benchmarks measure the performance of various encoder components
//! and the overall encoding pipeline.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_mp3_encoder::{Mp3Encoder, Config};
use rust_mp3_encoder::mdct::MdctTransform;

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

fn benchmark_mdct_transform(c: &mut Criterion) {
    let mdct = MdctTransform::new();
    let input = [[1000i32; 32]; 36]; // Realistic test data
    
    c.bench_function("mdct_transform", |b| {
        b.iter(|| {
            let mut output = [0i32; 576];
            mdct.transform(black_box(&input), black_box(&mut output)).unwrap();
            black_box(output);
        })
    });
}

fn benchmark_mdct_transform_fast(c: &mut Criterion) {
    let mdct = MdctTransform::new();
    let input = [[1000i32; 32]; 36]; // Realistic test data
    
    c.bench_function("mdct_transform_fast", |b| {
        b.iter(|| {
            let mut output = [0i32; 576];
            mdct.transform_fast(black_box(&input), black_box(&mut output)).unwrap();
            black_box(output);
        })
    });
}

fn benchmark_aliasing_reduction(c: &mut Criterion) {
    let mdct = MdctTransform::new();
    let coeffs = [1000i32; 576]; // Realistic test data
    
    c.bench_function("aliasing_reduction", |b| {
        b.iter(|| {
            let mut test_coeffs = coeffs;
            mdct.apply_aliasing_reduction(black_box(&mut test_coeffs)).unwrap();
            black_box(test_coeffs);
        })
    });
}

fn benchmark_mdct_batch_transform(c: &mut Criterion) {
    let mdct = MdctTransform::new();
    let inputs = vec![[[1000i32; 32]; 36]; 10]; // 10 frames
    
    c.bench_function("mdct_batch_transform", |b| {
        b.iter(|| {
            let mut outputs = vec![[0i32; 576]; 10];
            mdct.transform_batch(black_box(&inputs), black_box(&mut outputs)).unwrap();
            black_box(outputs);
        })
    });
}

criterion_group!(
    benches, 
    benchmark_encoder_creation, 
    benchmark_config_validation,
    benchmark_mdct_transform,
    benchmark_mdct_transform_fast,
    benchmark_aliasing_reduction,
    benchmark_mdct_batch_transform
);
criterion_main!(benches);