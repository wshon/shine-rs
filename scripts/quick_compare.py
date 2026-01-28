#!/usr/bin/env python3
"""
快速MP3编码器对比工具

简化版本，用于快速对比Rust和Shine编码器的输出结果。

用法:
    python scripts/quick_compare.py input.wav
"""

import os
import sys
import subprocess
import time
from pathlib import Path

def main():
    if len(sys.argv) != 2:
        print("用法: python scripts/quick_compare.py input.wav")
        sys.exit(1)
    
    input_file = sys.argv[1]
    
    if not os.path.exists(input_file):
        print(f"错误: 输入文件 '{input_file}' 不存在")
        sys.exit(1)
    
    # 查找编码器
    rust_exe = None
    shine_exe = None
    
    # 查找Rust编码器
    for path in ["target/release/shine-rs-cli.exe", "target/debug/shine-rs-cli.exe"]:
        if os.path.exists(path):
            rust_exe = path
            break
    
    # 查找Shine编码器
    for path in ["ref/shine/shineenc.exe", "ref/shine/build/shineenc.exe"]:
        if os.path.exists(path):
            shine_exe = path
            break
    
    if not rust_exe:
        print("错误: 找不到Rust编码器，请运行: cargo build --release")
        sys.exit(1)
    
    if not shine_exe:
        print("错误: 找不到Shine编码器，请构建Shine")
        sys.exit(1)
    
    # 生成输出文件名
    input_path = Path(input_file)
    base_name = input_path.stem
    output_dir = input_path.parent
    
    rust_output = output_dir / f"{base_name}_rust.mp3"
    shine_output = output_dir / f"{base_name}_shine.mp3"
    
    print(f"输入文件: {input_file}")
    print(f"Rust输出: {rust_output}")
    print(f"Shine输出: {shine_output}")
    print()
    
    # 运行Rust编码器
    print("运行Rust编码器...")
    start_time = time.time()
    rust_result = subprocess.run([rust_exe, input_file, str(rust_output)], 
                                capture_output=True, text=True)
    rust_time = time.time() - start_time
    
    if rust_result.returncode == 0:
        print(f"✅ Rust编码成功 ({rust_time:.2f}秒)")
    else:
        print(f"❌ Rust编码失败: {rust_result.stderr}")
    
    # 运行Shine编码器
    print("运行Shine编码器...")
    start_time = time.time()
    shine_result = subprocess.run([shine_exe, input_file, str(shine_output)], 
                                 capture_output=True, text=True)
    shine_time = time.time() - start_time
    
    if shine_result.returncode == 0:
        print(f"✅ Shine编码成功 ({shine_time:.2f}秒)")
    else:
        print(f"❌ Shine编码失败: {shine_result.stderr}")
    
    # 对比文件大小
    if os.path.exists(rust_output) and os.path.exists(shine_output):
        rust_size = os.path.getsize(rust_output)
        shine_size = os.path.getsize(shine_output)
        
        print(f"\n文件大小对比:")
        print(f"Rust:  {rust_size:,} 字节")
        print(f"Shine: {shine_size:,} 字节")
        
        if rust_size == shine_size:
            print("✅ 文件大小完全相同")
        else:
            diff = abs(rust_size - shine_size)
            diff_percent = (diff / shine_size) * 100
            print(f"差异: {diff:,} 字节 ({diff_percent:.2f}%)")
    
    print(f"\n性能对比:")
    if rust_result.returncode == 0 and shine_result.returncode == 0:
        if rust_time < shine_time:
            speedup = shine_time / rust_time
            print(f"Rust比Shine快 {speedup:.1f}x")
        else:
            slowdown = rust_time / shine_time
            print(f"Rust比Shine慢 {slowdown:.1f}x")

if __name__ == "__main__":
    main()