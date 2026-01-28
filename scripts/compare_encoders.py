#!/usr/bin/env python3
"""
MP3编码器对比工具

同时使用Rust版本和Shine版本编码器转换WAV文件为MP3，
生成带有_rust和_shine后缀的输出文件，便于对比分析。

用法:
    python scripts/compare_encoders.py input.wav [options]
    
示例:
    python scripts/compare_encoders.py sample.wav
    python scripts/compare_encoders.py sample.wav -b 192 -v
    python scripts/compare_encoders.py sample.wav -b 128 -j -q
"""

import os
import sys
import subprocess
import argparse
import time
from pathlib import Path

def find_executables():
    """查找Rust和Shine编码器的可执行文件路径"""
    
    # Rust编码器路径
    rust_exe = None
    possible_rust_paths = [
        "target/release/shine-rs-cli.exe",
        "target/debug/shine-rs-cli.exe", 
        "shine-rs-cli.exe",
        "target/release/shine-rs-cli",
        "target/debug/shine-rs-cli",
        "shine-rs-cli"
    ]
    
    for path in possible_rust_paths:
        if os.path.exists(path):
            rust_exe = path
            break
    
    # Shine编码器路径
    shine_exe = None
    possible_shine_paths = [
        "ref/shine/shineenc.exe",
        "ref/shine/build/shineenc.exe",
        "ref/shine/shineenc",
        "ref/shine/build/shineenc"
    ]
    
    for path in possible_shine_paths:
        if os.path.exists(path):
            shine_exe = path
            break
    
    return rust_exe, shine_exe

def build_command_args(base_args, encoder_type):
    """构建编码器命令行参数"""
    cmd_args = []
    
    # 处理选项参数
    i = 0
    while i < len(base_args):
        arg = base_args[i]
        if arg.startswith('-'):
            cmd_args.append(arg)
            # 检查是否需要参数值的选项
            if arg in ['-b'] and i + 1 < len(base_args):
                i += 1
                cmd_args.append(base_args[i])
        i += 1
    
    return cmd_args

def run_encoder(exe_path, input_file, output_file, options, encoder_name):
    """运行编码器并返回结果"""
    
    if not exe_path:
        print(f"错误: 找不到{encoder_name}编码器可执行文件")
        return False, None, None
    
    # 构建完整命令
    cmd = [exe_path] + options + [input_file, output_file]
    
    print(f"\n=== 运行 {encoder_name} 编码器 ===")
    print(f"命令: {' '.join(cmd)}")
    
    try:
        start_time = time.time()
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300  # 5分钟超时
        )
        end_time = time.time()
        
        duration = end_time - start_time
        
        if result.returncode == 0:
            print(f"✅ {encoder_name} 编码成功 (耗时: {duration:.2f}秒)")
            if result.stdout.strip():
                print("输出:")
                print(result.stdout)
        else:
            print(f"❌ {encoder_name} 编码失败 (返回码: {result.returncode})")
            if result.stderr.strip():
                print("错误信息:")
                print(result.stderr)
        
        return result.returncode == 0, result.stdout, result.stderr
        
    except subprocess.TimeoutExpired:
        print(f"❌ {encoder_name} 编码超时")
        return False, None, "编码超时"
    except Exception as e:
        print(f"❌ {encoder_name} 编码异常: {e}")
        return False, None, str(e)

def get_file_info(file_path):
    """获取文件信息"""
    if not os.path.exists(file_path):
        return None
    
    stat = os.stat(file_path)
    return {
        'size': stat.st_size,
        'size_mb': stat.st_size / (1024 * 1024),
        'exists': True
    }

def compare_results(rust_output, shine_output, input_file):
    """对比编码结果"""
    print(f"\n=== 编码结果对比 ===")
    
    # 获取文件信息
    input_info = get_file_info(input_file)
    rust_info = get_file_info(rust_output)
    shine_info = get_file_info(shine_output)
    
    if input_info:
        print(f"输入文件: {input_file}")
        print(f"  大小: {input_info['size']:,} 字节 ({input_info['size_mb']:.2f} MB)")
    
    print(f"\nRust版本输出: {rust_output}")
    if rust_info:
        print(f"  大小: {rust_info['size']:,} 字节 ({rust_info['size_mb']:.2f} MB)")
        if input_info:
            compression_ratio = input_info['size'] / rust_info['size']
            print(f"  压缩比: {compression_ratio:.1f}:1")
    else:
        print("  ❌ 文件不存在")
    
    print(f"\nShine版本输出: {shine_output}")
    if shine_info:
        print(f"  大小: {shine_info['size']:,} 字节 ({shine_info['size_mb']:.2f} MB)")
        if input_info:
            compression_ratio = input_info['size'] / shine_info['size']
            print(f"  压缩比: {compression_ratio:.1f}:1")
    else:
        print("  ❌ 文件不存在")
    
    # 大小对比
    if rust_info and shine_info:
        size_diff = abs(rust_info['size'] - shine_info['size'])
        size_diff_percent = (size_diff / shine_info['size']) * 100
        print(f"\n文件大小差异: {size_diff:,} 字节 ({size_diff_percent:.2f}%)")
        
        if size_diff == 0:
            print("✅ 文件大小完全相同")
        elif size_diff_percent < 1.0:
            print("✅ 文件大小非常接近")
        elif size_diff_percent < 5.0:
            print("⚠️  文件大小略有差异")
        else:
            print("❌ 文件大小差异较大")

def main():
    parser = argparse.ArgumentParser(
        description="同时使用Rust和Shine编码器转换WAV文件为MP3",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  python scripts/compare_encoders.py sample.wav
  python scripts/compare_encoders.py sample.wav -b 192 -v
  python scripts/compare_encoders.py sample.wav -b 128 -j -q
  python scripts/compare_encoders.py sample.wav -m -b 96
        """
    )
    
    parser.add_argument('input_file', help='输入WAV文件路径')
    parser.add_argument('-b', '--bitrate', type=int, help='比特率 [8-320]，默认128kbit')
    parser.add_argument('-m', '--mono', action='store_true', help='强制单声道模式')
    parser.add_argument('-c', '--copyright', action='store_true', help='设置版权标志')
    parser.add_argument('-j', '--joint-stereo', action='store_true', help='联合立体声编码')
    parser.add_argument('-d', '--dual-channel', action='store_true', help='双声道编码')
    parser.add_argument('-q', '--quiet', action='store_true', help='安静模式')
    parser.add_argument('-v', '--verbose', action='store_true', help='详细模式')
    parser.add_argument('--rust-only', action='store_true', help='仅运行Rust编码器')
    parser.add_argument('--shine-only', action='store_true', help='仅运行Shine编码器')
    parser.add_argument('--output-dir', help='输出目录，默认与输入文件同目录')
    
    args = parser.parse_args()
    
    # 检查输入文件
    if not os.path.exists(args.input_file):
        print(f"错误: 输入文件 '{args.input_file}' 不存在")
        sys.exit(1)
    
    # 查找编码器
    rust_exe, shine_exe = find_executables()
    
    if not args.shine_only and not rust_exe:
        print("错误: 找不到Rust编码器，请先编译项目:")
        print("  cargo build --release")
        sys.exit(1)
    
    if not args.rust_only and not shine_exe:
        print("错误: 找不到Shine编码器，请先构建Shine:")
        print("  cd ref/shine && ./build.ps1")
        sys.exit(1)
    
    # 生成输出文件名
    input_path = Path(args.input_file)
    output_dir = Path(args.output_dir) if args.output_dir else input_path.parent
    base_name = input_path.stem
    
    rust_output = output_dir / f"{base_name}_rust.mp3"
    shine_output = output_dir / f"{base_name}_shine.mp3"
    
    # 构建编码器选项
    options = []
    if args.bitrate:
        options.extend(['-b', str(args.bitrate)])
    if args.mono:
        options.append('-m')
    if args.copyright:
        options.append('-c')
    if args.joint_stereo:
        options.append('-j')
    if args.dual_channel:
        options.append('-d')
    if args.quiet:
        options.append('-q')
    if args.verbose:
        options.append('-v')
    
    print(f"输入文件: {args.input_file}")
    print(f"输出目录: {output_dir}")
    if options:
        print(f"编码选项: {' '.join(options)}")
    
    # 运行编码器
    rust_success = True
    shine_success = True
    
    if not args.shine_only:
        rust_success, rust_stdout, rust_stderr = run_encoder(
            rust_exe, args.input_file, str(rust_output), options, "Rust"
        )
    
    if not args.rust_only:
        shine_success, shine_stdout, shine_stderr = run_encoder(
            shine_exe, args.input_file, str(shine_output), options, "Shine"
        )
    
    # 对比结果
    if not args.rust_only and not args.shine_only:
        compare_results(str(rust_output), str(shine_output), args.input_file)
    
    # 总结
    print(f"\n=== 编码完成 ===")
    if not args.shine_only:
        if rust_success:
            print(f"✅ Rust版本: {rust_output}")
        else:
            print(f"❌ Rust版本编码失败")
    
    if not args.rust_only:
        if shine_success:
            print(f"✅ Shine版本: {shine_output}")
        else:
            print(f"❌ Shine版本编码失败")
    
    # 返回适当的退出码
    if (not args.shine_only and not rust_success) or (not args.rust_only and not shine_success):
        sys.exit(1)

if __name__ == "__main__":
    main()