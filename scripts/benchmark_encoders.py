#!/usr/bin/env python3
"""
Performance benchmark script for Shine-RS vs Shine C
Uses existing test audio files to compare encoding performance

使用编码器内置的高精度计时：
- 读取编码器命令行输出中的实时倍率
- 避免外部计时的进程启动和I/O开销
- 获得更准确的编码算法性能数据
"""

import os
import sys
import subprocess
import re
from pathlib import Path

def parse_realtime_ratio(output_text):
    """
    Parse realtime ratio from encoder output
    
    支持两种格式：
    - 新格式：(123.4x realtime) - 高精度计时
    - 旧格式：(infx realtime) - 当编码时间 < 1秒时
    """
    # Look for pattern like "(123.4x realtime)"
    match = re.search(r'\(([0-9]+\.?[0-9]*)x realtime\)', output_text)
    if match:
        return float(match.group(1))
    
    # Look for "infx realtime" (infinite speed - encoding time < 1 second)
    if 'infx realtime' in output_text:
        return float('inf')
    
    return None

def benchmark_shine_c(audio_file, bitrate, output_file):
    """Benchmark Shine C encoder - 读取命令行输出的实际倍率"""
    shine_exe = Path("ref/shine/shineenc.exe")
    if not shine_exe.exists():
        return None
    
    try:
        # 不使用 -q 参数，这样可以读取输出信息
        # 使用绝对路径避免路径问题
        abs_audio_file = os.path.abspath(audio_file)
        abs_output_file = os.path.abspath(output_file)
        cmd = [str(shine_exe), "-b", str(bitrate), abs_audio_file, abs_output_file]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        
        if result.returncode == 0:
            # 从Shine输出解析实际倍率
            shine_reported_ratio = parse_realtime_ratio(result.stdout)
            
            # Clean up output file
            if os.path.exists(abs_output_file):
                os.remove(abs_output_file)
            
            return shine_reported_ratio
        else:
            print(f"❌ Shine C failed (exit code {result.returncode})")
            if result.stderr:
                print(f"   Error: {result.stderr.strip()}")
    except subprocess.TimeoutExpired:
        print(f"⚠️  Shine C timeout for {bitrate}kbps")
    except Exception as e:
        print(f"❌ Shine C error: {e}")
    
    return None

def benchmark_rust(audio_file, bitrate, output_file):
    """Benchmark Rust encoder - 读取命令行输出的实际倍率"""
    rust_exe = Path("target/release/shine-rs-cli.exe")
    if not rust_exe.exists():
        print(f"❌ Rust binary not found: {rust_exe}")
        print("   Please run: cargo build --release")
        return None
    
    try:
        # 不使用 -q 参数，这样可以读取输出信息
        cmd = [str(rust_exe), "-b", str(bitrate), audio_file, output_file]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        
        if result.returncode == 0:
            # 从Rust输出解析实际倍率
            rust_reported_ratio = parse_realtime_ratio(result.stdout)
            
            # Clean up output file
            if os.path.exists(output_file):
                os.remove(output_file)
            
            return rust_reported_ratio
        else:
            print(f"❌ Rust encoder failed (exit code {result.returncode})")
            if result.stderr:
                print(f"   Error: {result.stderr.strip()}")
    except subprocess.TimeoutExpired:
        print(f"⚠️  Rust timeout for {bitrate}kbps")
    except Exception as e:
        print(f"❌ Rust error: {e}")
    
    return None

def run_benchmark():
    """Run the complete benchmark"""
    print("🚀 Shine-RS vs Shine C Performance Benchmark")
    print("=" * 60)
    print("📋 使用编译后的二进制文件进行性能测试")
    print("   - Rust: target/release/shine-rs-cli.exe")
    print("   - Shine C: ref/shine/shineenc.exe")
    print("   - 如果缺少二进制文件，请先运行编译命令")
    print()
    print("📋 编码器内置计时分析:")
    print("   - 读取编码器命令行输出中的实时倍率")
    print("   - 避免外部计时的进程启动和I/O开销")
    print("   - 获得更准确的编码算法性能数据")
    print()
    
    # Check if required binaries exist
    rust_exe = Path("target/release/shine-rs-cli.exe")
    shine_exe = Path("ref/shine/shineenc.exe")
    
    print("🔍 检查必要的二进制文件...")
    if not rust_exe.exists():
        print(f"❌ Rust 二进制文件不存在: {rust_exe}")
        print("   请运行: cargo build --release")
        return
    else:
        print(f"✅ Rust 二进制文件: {rust_exe}")
    
    if not shine_exe.exists():
        print(f"⚠️  Shine C 二进制文件不存在: {shine_exe}")
        print("   请运行: cd ref/shine && .\\build.ps1")
        print("   将只测试 Rust 编码器性能")
    else:
        print(f"✅ Shine C 二进制文件: {shine_exe}")
    
    # Test configurations - 使用现有的测试音频文件
    test_files = [
        ("tests/audio/inputs/basic/sample-15s.wav", "15秒测试音频"),
        ("tests/audio/inputs/basic/sample-3s.wav", "3秒测试音频"),
        ("tests/audio/inputs/basic/Free_Test_Data_500KB_WAV.wav", "500KB测试音频"),
        ("tests/audio/inputs/basic/voice-recorder-testing-1-2-3-sound-file.wav", "语音测试音频"),
    ]
    
    bitrates = [128, 192, 320]
    
    # Check test files exist
    print("\n📁 检查测试音频文件...")
    available_files = []
    for filename, description in test_files:
        if os.path.exists(filename):
            print(f"✅ {description}: {filename}")
            available_files.append((filename, description))
        else:
            print(f"❌ 文件不存在: {filename}")
    
    if not available_files:
        print("❌ 没有找到可用的测试音频文件")
        return
    
    print(f"\n🧪 Running benchmark tests...")
    
    results = []
    
    for filename, description in available_files:
        print(f"\n🎵 测试: {description}")
        print(f"   文件: {filename}")
        
        for bitrate in bitrates:
            print(f"   📊 {bitrate}kbps: ", end="", flush=True)
            
            # Test Rust encoder
            rust_output = f"temp_rust_{bitrate}.mp3"
            rust_ratio = benchmark_rust(filename, bitrate, rust_output)
            
            # Test Shine C encoder
            shine_output = f"temp_shine_{bitrate}.mp3"
            shine_ratio = benchmark_shine_c(filename, bitrate, shine_output)
            
            if rust_ratio is not None and shine_ratio is not None:
                if shine_ratio == float('inf'):
                    print(f"Rust: {rust_ratio:.1f}x | Shine: inf | 🚀 Rust measurable")
                else:
                    speedup = rust_ratio / shine_ratio
                    print(f"Rust: {rust_ratio:.1f}x | Shine: {shine_ratio:.1f}x | 🚀{speedup:.1f}x faster")
                
                results.append({
                    'file': filename,
                    'description': description,
                    'bitrate': bitrate,
                    'rust_ratio': rust_ratio,
                    'shine_ratio': shine_ratio
                })
            elif rust_ratio is not None:
                print(f"Rust: {rust_ratio:.1f}x | Shine: failed")
            elif shine_ratio is not None:
                print(f"Rust: failed | Shine: {shine_ratio:.1f}x")
            else:
                print("Both failed")
    
    # Print summary
    print("\n" + "=" * 60)
    print("📈 Performance Summary")
    print("=" * 60)
    
    if results:
        for bitrate in bitrates:
            bitrate_results = [r for r in results if r['bitrate'] == bitrate]
            if bitrate_results:
                rust_avg = sum(r['rust_ratio'] for r in bitrate_results if r['rust_ratio'] != float('inf')) / len(bitrate_results)
                shine_finite = [r['shine_ratio'] for r in bitrate_results if r['shine_ratio'] != float('inf')]
                
                if shine_finite:
                    shine_avg = sum(shine_finite) / len(shine_finite)
                    speedup = rust_avg / shine_avg
                    print(f"🎯 {bitrate}kbps: Rust {rust_avg:.1f}x | Shine {shine_avg:.1f}x | 🚀{speedup:.1f}x faster")
                else:
                    print(f"🎯 {bitrate}kbps: Rust {rust_avg:.1f}x | Shine: unmeasurable (too fast)")
        
        # Overall average
        all_rust = [r['rust_ratio'] for r in results if r['rust_ratio'] != float('inf')]
        all_shine = [r['shine_ratio'] for r in results if r['shine_ratio'] != float('inf')]
        
        if all_rust and all_shine:
            overall_rust = sum(all_rust) / len(all_rust)
            overall_shine = sum(all_shine) / len(all_shine)
            overall_speedup = overall_rust / overall_shine
            
            print(f"\n🏆 Overall: Rust {overall_rust:.1f}x | Shine {overall_shine:.1f}x | 🚀{overall_speedup:.1f}x faster")
        elif all_rust:
            overall_rust = sum(all_rust) / len(all_rust)
            print(f"\n🏆 Overall: Rust {overall_rust:.1f}x | Shine: mostly unmeasurable (too fast)")
    
    # Clean up temporary MP3 files
    print(f"\n🧹 清理临时文件...")
    temp_files = [f for f in os.listdir('.') if f.startswith('temp_') and f.endswith('.mp3')]
    for temp_file in temp_files:
        try:
            os.remove(temp_file)
            print(f"   删除: {temp_file}")
        except OSError:
            pass
    
    print("✅ 性能测试完成!")

if __name__ == "__main__":
    try:
        run_benchmark()
    except KeyboardInterrupt:
        print("\n⚠️  Benchmark interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Benchmark failed: {e}")
        sys.exit(1)