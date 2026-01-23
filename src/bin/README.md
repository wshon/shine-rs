cargo run --bin wav2mp3 tests/input/sample-12s.wav tests/output/sample-12s-output.mp3
cargo run --bin mp3_validator tests/output/sample-12s-output.mp3
ffmpeg -i tests/output/sample-12s-output.mp3 -y tests/output/decoded-sample.wav
ffprobe -v quiet -print_format json -show_format tests/output/decoded-sample.wav
对比 sample-12s.wav 和 decoded-sample.wav 的一致性