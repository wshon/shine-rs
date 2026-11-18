//! Pure-encode benchmark harness.
//! Reads WAV into memory OUTSIDE the timed region, then times only
//! shine_initialise + encode loop + flush. Output is discarded (no file I/O).
//! Usage: cargo run --release --example bench_encode -- <wav> <bitrate>

use shine_rs::{
    shine_encode_buffer_interleaved, shine_flush, shine_initialise,
    shine_set_config_mpeg_defaults, ShineConfig, ShineMpeg, ShineWave,
};
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let wav = &args[1];
    let bitrate: i32 = args[2].parse().unwrap();

    // --- WAV parse (OUTSIDE timed region) ---
    let mut reader = hound::WavReader::open(wav).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as i32;
    let sample_rate = spec.sample_rate as i32;
    let pcm: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    let duration = pcm.len() as f64 / (sample_rate as f64 * channels as f64);
    drop(reader);

    // --- pure encode (TIMED) ---
    let mut config = ShineConfig {
        wave: ShineWave { channels, samplerate: sample_rate },
        mpeg: ShineMpeg { mode: if channels > 1 { 0 } else { 3 }, bitr: bitrate, emph: 0, copyright: 0, original: 1 },
    };
    shine_set_config_mpeg_defaults(&mut config.mpeg);
    config.mpeg.bitr = bitrate;

    let spp = if sample_rate >= 32000 { 1152 } else { 576 };
    let frame_samples = spp * channels as usize;

    let start = Instant::now();
    let mut enc = shine_initialise(&config).unwrap();
    let mut pos = 0usize;
    let mut total = 0usize;
    while pos < pcm.len() {
        let n = std::cmp::min(frame_samples, pcm.len() - pos);
        let mut buf = vec![0i16; frame_samples];
        buf[..n].copy_from_slice(&pcm[pos..pos + n]);
        let (_, written) = unsafe { shine_encode_buffer_interleaved(&mut enc, buf.as_ptr()) }.unwrap();
        total += written;
        pos += n;
    }
    let (_, w) = shine_flush(&mut enc);
    total += w;
    let elapsed = start.elapsed().as_secs_f64();

    println!("{:.1}x realtime", duration / elapsed);
    let _ = total;
}
