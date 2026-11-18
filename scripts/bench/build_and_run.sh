#!/usr/bin/env bash
# Build and run the pure-encode benchmarks (Rust vs Shine C).
#
# Both harnesses read the WAV into memory BEFORE the timed region, then time
# only shine_initialise + encode loop + flush. Output is discarded, so no
# file I/O and no WAV parsing is included in the measured time.
#
# Requires the Shine C reference source checkout (configure-free):
#   export SHINE_REF=/path/to/shine-ref
# Defaults to ../shine-ref.
set -euo pipefail
cd "$(dirname "$0")/../.."   # repo root

SHINE_REF="${SHINE_REF:-../shine-ref}"
OUT=/tmp/shine-bench
mkdir -p "$OUT"

echo "=== Building Rust harness ==="
cargo build --release -p shine-rs --example bench_encode

echo "=== Building C harness (from $SHINE_REF) ==="
gcc -O3 -o "$OUT/bench_c" \
  scripts/bench/bench_encode_c.c \
  "$SHINE_REF/src/bin/wave.c" \
  "$SHINE_REF/src/lib/l3bitstream.c" \
  "$SHINE_REF/src/lib/tables.c" \
  "$SHINE_REF/src/lib/huffman.c" \
  "$SHINE_REF/src/lib/bitstream.c" \
  "$SHINE_REF/src/lib/l3loop.c" \
  "$SHINE_REF/src/lib/reservoir.c" \
  "$SHINE_REF/src/lib/layer3.c" \
  "$SHINE_REF/src/lib/l3mdct.c" \
  "$SHINE_REF/src/lib/l3subband.c" \
  -I "$SHINE_REF/src/lib" -I "$SHINE_REF/src/bin" -lm

echo "=== Running (5 runs each, median) ==="
for wav in \
  tests/audio/inputs/basic/sample-15s.wav \
  tests/audio/inputs/basic/sample-3s.wav \
  tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav \
  tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav; do
  for br in 128 192 320; do
    r=$(for i in 1 2 3 4 5; do target/release/examples/bench_encode "$wav" "$br"; done | grep -oE '[0-9.]+' | sort -n | sed -n 3p)
    c=$(for i in 1 2 3 4 5; do "$OUT/bench_c" "$wav" "$br"; done | grep -oE '[0-9.]+' | sort -n | sed -n 3p)
    ratio=$(awk "BEGIN{printf \"%.2f\", $r/$c}")
    printf "%-48s %3dkbps  Rust %6sx  C %6sx  ratio %s\n" \
      "$(basename "$wav")" "$br" "$r" "$c" "$ratio"
  done
done
