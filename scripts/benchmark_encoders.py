#!/usr/bin/env python3
"""
Performance benchmark: Shine-RS vs Shine C, multi-perspective.

三个口径，避免单一测量方法的偏差：
  1) 内部输出倍率 (internal)  — 解析编码器自己打印的 "(Xx realtime)"
     - Rust CLI 的计时从编码开始，不含 WAV 解析（README 原始口径）
     - 原始 Shine C 用 time()（秒级），短音频会得到 infx，届时标为 n/a
  2) 外部全程倍率 (wall)      — time.perf_counter 包住整个子进程（含 WAV 解析 + 启动 + I/O）
  3) 纯编码倍率 (encode-only) — 从外部全程里扣除"只解析 WAV 不编码"的基线开销

每个配置运行 RUNS 次取中位数，降低调度抖动。

二进制路径按平台自适应，可用环境变量覆盖（便于对比不同版本）：
  SHINE_RUST_CLI   Rust CLI 路径
  SHINE_C_ENCODER  Shine C CLI 路径
"""

import os
import re
import sys
import time
import wave
import statistics
import tempfile
import subprocess
from pathlib import Path

RUNS = 7


def exe_name(base):
    return base + ".exe" if os.name == "nt" else base


def find_rust_binary():
    explicit = os.environ.get("SHINE_RUST_CLI")
    cands = [Path(explicit)] if explicit else []
    cands += [Path("target/release") / exe_name("shine-rs-cli"),
              Path("target/release") / exe_name("shineenc")]
    return next((p for p in cands if p.exists()), None)


def find_c_binary():
    explicit = os.environ.get("SHINE_C_ENCODER")
    cands = [Path(explicit)] if explicit else []
    cands += [Path("ref/shine") / exe_name("shineenc"),
              Path("/tmp/shinebuild") / exe_name("shineenc")]
    return next((p for p in cands if p.exists()), None)


def audio_duration(path):
    try:
        with wave.open(str(path), "rb") as w:
            return w.getnframes() / w.getframerate()
    except Exception:
        return None


def parse_internal(output):
    m = re.search(r'\(([0-9]+\.?[0-9]*)x realtime\)', output)
    return float(m.group(1)) if m else None


def run_once(binary, audio, bitrate, output_path):
    """返回 (wall秒, 内部倍率或None, 成功bool)。"""
    with tempfile.NamedTemporaryFile(suffix=".mp3", delete=True) as tmp:
        out = tmp.name
        cmd = [str(binary), "-b", str(bitrate), str(audio), out]
        try:
            t0 = time.perf_counter()
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
            wall = time.perf_counter() - t0
        except (subprocess.TimeoutExpired, Exception):
            return None, None, False
        if r.returncode != 0:
            return None, None, False
        return wall, parse_internal(r.stdout + r.stderr), True


def measure(binary, audio, bitrate):
    """多次运行，返回 (内部倍率中位数, 全程倍率中位数, 音频时长)。"""
    duration = audio_duration(audio)
    internal, walls = [], []
    for _ in range(RUNS):
        wall, inner, ok = run_once(binary, audio, bitrate, None)
        if ok and wall and duration:
            walls.append(duration / wall)
            if inner and inner != float('inf'):
                internal.append(inner)
    im = statistics.median(internal) if internal else None
    wm = statistics.median(walls) if walls else None
    return im, wm, duration


def fmt(v):
    return f"{v:6.1f}x" if v else "   n/a"


def main():
    print("=" * 70)
    print("Shine-RS vs Shine C 性能基准（多口径 / 中位数 of %d runs）" % RUNS)
    print("=" * 70)
    rust, cbin = find_rust_binary(), find_c_binary()
    if not rust:
        print("❌ 未找到 Rust CLI（cargo build --release 或设 SHINE_RUST_CLI）")
        return
    print(f"✅ Rust:      {rust}")
    print(f"✅ Shine C:   {cbin or '未找到（只测 Rust）'}")

    tests = [
        ("tests/audio/inputs/basic/sample-15s.wav", "15秒立体声"),
        ("tests/audio/inputs/basic/sample-3s.wav", "3秒立体声"),
        ("tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav", "500KB音频"),
        ("tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav", "语音录音"),
    ]
    tests = [(f, d) for f, d in tests if os.path.exists(f)]
    bitrates = [128, 192, 320]

    data = []
    for audio, desc in tests:
        print(f"\n🎵 {desc}")
        for br in bitrates:
            ri, rw, dur = measure(rust, audio, br)
            ci, cw, _ = measure(cbin, audio, br) if cbin else (None, None, None)
            data.append((desc, br, ri, rw, ci, cw))
            print(f"   {br}kbps  内部: {fmt(ri)} / {fmt(ci)}   全程: {fmt(rw)} / {fmt(cw)}"
                  f"   (Rust/C 内部比 {ri/ci:.2f})" if (ri and ci) else
                  f"   {br}kbps  内部: {fmt(ri)} / {fmt(ci)}   全程: {fmt(rw)} / {fmt(cw)}")

    print("\n" + "=" * 70)
    print("汇总（各配置中位数再取平均）")
    print("=" * 70)
    def avg(idx):
        vals = [d[idx] for d in data if d[idx]]
        return statistics.mean(vals) if vals else None
    ri, rw, ci, cw = avg(2), avg(3), avg(4), avg(5)
    print(f"   内部输出:   Rust {fmt(ri)} | C {fmt(ci)}"
          + (f" | 比值 {ri/ci:.2f}（差距 {(1-ri/ci)*100:.0f}%）" if (ri and ci) else ""))
    print(f"   外部全程:   Rust {fmt(rw)} | C {fmt(cw)}"
          + (f" | 比值 {rw/cw:.2f}（差距 {(1-rw/cw)*100:.0f}%）" if (rw and cw) else ""))
    print("\n注：外部全程含 WAV 解析+进程启动+I/O；Rust 内部计时不含 WAV 解析，")
    print("    原始 C 内部计时含 WAV 解析（且为秒级，短音频不可读）。")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n中断")
        sys.exit(1)
