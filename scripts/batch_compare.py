#!/usr/bin/env python3
"""
批量MP3编码器对比工具

对多个WAV文件同时进行Rust和Shine编码器对比测试。

用法:
    python scripts/batch_compare.py [directory] [options]
    
示例:
    python scripts/batch_compare.py testing/
    python scripts/batch_compare.py . -b 192
    python scripts/batch_compare.py testing/ --pattern "*.wav" -v
"""

import os
import sys
import subprocess
import time
import glob
import argparse
from pathlib import Path
import json

def find_executables():
    """查找编码器可执行文件"""
    rust_exe = None
    shine_exe = None
    
    # 查找Rust编码器
    for path in ["target/release/shine-rs-cli.exe", "target/debug/shine-rs-cli.exe", 
                 "target/release/shine-rs-cli", "target/debug/shine-rs-cli"]:
        if os.path.exists(path):
            rust_exe = path
            break
    
    # 查找Shine编码器
    for path in ["ref/shine/shineenc.exe", "ref/shine/build/shineenc.exe",
                 "ref/shine/shineenc", "ref/shine/build/shineenc"]:
        if os.path.exists(path):
            shine_exe = path
            break
    
    return rust_exe, shine_exe

def run_encoder(exe_path, input_file, output_file, options):
    """运行编码器"""
    cmd = [exe_path] + options + [input_file, output_file]
    
    try:
        start_time = time.time()
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        end_time = time.time()
        
        return {
            'success': result.returncode == 0,
            'time': end_time - start_time,
            'stdout': result.stdout,
            'stderr': result.stderr,
            'returncode': result.returncode
        }
    except subprocess.TimeoutExpired:
        return {
            'success': False,
            'time': 300,
            'stdout': '',
            'stderr': '编码超时',
            'returncode': -1
        }
    except Exception as e:
        return {
            'success': False,
            'time': 0,
            'stdout': '',
            'stderr': str(e),
            'returncode': -1
        }

def get_file_info(file_path):
    """获取文件信息"""
    if not os.path.exists(file_path):
        return None
    
    stat = os.stat(file_path)
    return {
        'size': stat.st_size,
        'size_mb': stat.st_size / (1024 * 1024)
    }

def process_file(input_file, rust_exe, shine_exe, options, output_dir, verbose=False):
    """处理单个文件"""
    input_path = Path(input_file)
    base_name = input_path.stem
    
    rust_output = output_dir / f"{base_name}_rust.mp3"
    shine_output = output_dir / f"{base_name}_shine.mp3"
    
    if verbose:
        print(f"\n处理文件: {input_file}")
        print(f"  Rust输出: {rust_output}")
        print(f"  Shine输出: {shine_output}")
    
    # 运行编码器
    rust_result = run_encoder(rust_exe, input_file, str(rust_output), options)
    shine_result = run_encoder(shine_exe, input_file, str(shine_output), options)
    
    # 获取文件信息
    input_info = get_file_info(input_file)
    rust_info = get_file_info(rust_output) if rust_result['success'] else None
    shine_info = get_file_info(shine_output) if shine_result['success'] else None
    
    result = {
        'input_file': input_file,
        'input_size': input_info['size'] if input_info else 0,
        'rust': {
            'success': rust_result['success'],
            'time': rust_result['time'],
            'output_file': str(rust_output),
            'output_size': rust_info['size'] if rust_info else 0,
            'error': rust_result['stderr'] if not rust_result['success'] else None
        },
        'shine': {
            'success': shine_result['success'],
            'time': shine_result['time'],
            'output_file': str(shine_output),
            'output_size': shine_info['size'] if shine_info else 0,
            'error': shine_result['stderr'] if not shine_result['success'] else None
        }
    }
    
    # 计算差异
    if rust_info and shine_info:
        size_diff = abs(rust_info['size'] - shine_info['size'])
        result['size_diff'] = size_diff
        result['size_diff_percent'] = (size_diff / shine_info['size']) * 100
    
    if verbose:
        if rust_result['success']:
            print(f"  ✅ Rust: {rust_result['time']:.2f}s, {rust_info['size']:,} bytes")
        else:
            print(f"  ❌ Rust: {rust_result['stderr']}")
        
        if shine_result['success']:
            print(f"  ✅ Shine: {shine_result['time']:.2f}s, {shine_info['size']:,} bytes")
        else:
            print(f"  ❌ Shine: {shine_result['stderr']}")
        
        if 'size_diff' in result:
            print(f"  📊 大小差异: {result['size_diff']:,} bytes ({result['size_diff_percent']:.2f}%)")
    
    return result

def print_summary(results):
    """打印汇总统计"""
    total_files = len(results)
    rust_success = sum(1 for r in results if r['rust']['success'])
    shine_success = sum(1 for r in results if r['shine']['success'])
    
    print(f"\n=== 批量编码汇总 ===")
    print(f"总文件数: {total_files}")
    print(f"Rust成功: {rust_success}/{total_files} ({rust_success/total_files*100:.1f}%)")
    print(f"Shine成功: {shine_success}/{total_files} ({shine_success/total_files*100:.1f}%)")
    
    # 成功的文件统计
    successful_results = [r for r in results if r['rust']['success'] and r['shine']['success']]
    
    if successful_results:
        print(f"\n=== 性能对比 (成功编码的{len(successful_results)}个文件) ===")
        
        total_rust_time = sum(r['rust']['time'] for r in successful_results)
        total_shine_time = sum(r['shine']['time'] for r in successful_results)
        
        print(f"总编码时间:")
        print(f"  Rust:  {total_rust_time:.2f}秒")
        print(f"  Shine: {total_shine_time:.2f}秒")
        
        if total_rust_time > 0 and total_shine_time > 0:
            if total_rust_time < total_shine_time:
                speedup = total_shine_time / total_rust_time
                print(f"  Rust比Shine快 {speedup:.1f}x")
            else:
                slowdown = total_rust_time / total_shine_time
                print(f"  Rust比Shine慢 {slowdown:.1f}x")
        
        # 文件大小统计
        size_diffs = [r['size_diff_percent'] for r in successful_results if 'size_diff_percent' in r]
        if size_diffs:
            avg_diff = sum(size_diffs) / len(size_diffs)
            max_diff = max(size_diffs)
            min_diff = min(size_diffs)
            
            print(f"\n文件大小差异统计:")
            print(f"  平均差异: {avg_diff:.2f}%")
            print(f"  最大差异: {max_diff:.2f}%")
            print(f"  最小差异: {min_diff:.2f}%")
            
            identical_count = sum(1 for d in size_diffs if d == 0)
            print(f"  完全相同: {identical_count}/{len(size_diffs)} ({identical_count/len(size_diffs)*100:.1f}%)")

def main():
    parser = argparse.ArgumentParser(description="批量MP3编码器对比工具")
    parser.add_argument('directory', nargs='?', default='.', help='搜索目录，默认当前目录')
    parser.add_argument('--pattern', default='*.wav', help='文件匹配模式，默认*.wav')
    parser.add_argument('-b', '--bitrate', type=int, help='比特率')
    parser.add_argument('-m', '--mono', action='store_true', help='强制单声道')
    parser.add_argument('-j', '--joint-stereo', action='store_true', help='联合立体声')
    parser.add_argument('-q', '--quiet', action='store_true', help='安静模式')
    parser.add_argument('-v', '--verbose', action='store_true', help='详细模式')
    parser.add_argument('--output-dir', help='输出目录，默认与输入文件同目录')
    parser.add_argument('--save-report', help='保存详细报告到JSON文件')
    
    args = parser.parse_args()
    
    # 查找编码器
    rust_exe, shine_exe = find_executables()
    
    if not rust_exe:
        print("错误: 找不到Rust编码器，请运行: cargo build --release")
        sys.exit(1)
    
    if not shine_exe:
        print("错误: 找不到Shine编码器")
        sys.exit(1)
    
    # 查找WAV文件
    search_pattern = os.path.join(args.directory, args.pattern)
    wav_files = glob.glob(search_pattern, recursive=True)
    
    if not wav_files:
        print(f"在 '{args.directory}' 中找不到匹配 '{args.pattern}' 的文件")
        sys.exit(1)
    
    print(f"找到 {len(wav_files)} 个WAV文件")
    
    # 构建编码选项
    options = []
    if args.bitrate:
        options.extend(['-b', str(args.bitrate)])
    if args.mono:
        options.append('-m')
    if args.joint_stereo:
        options.append('-j')
    if args.quiet:
        options.append('-q')
    
    if options:
        print(f"编码选项: {' '.join(options)}")
    
    # 处理文件
    results = []
    output_dir = Path(args.output_dir) if args.output_dir else None
    
    for i, wav_file in enumerate(wav_files, 1):
        if not args.verbose:
            print(f"处理 {i}/{len(wav_files)}: {os.path.basename(wav_file)}")
        
        file_output_dir = output_dir if output_dir else Path(wav_file).parent
        result = process_file(wav_file, rust_exe, shine_exe, options, file_output_dir, args.verbose)
        results.append(result)
    
    # 打印汇总
    print_summary(results)
    
    # 保存报告
    if args.save_report:
        with open(args.save_report, 'w', encoding='utf-8') as f:
            json.dump(results, f, indent=2, ensure_ascii=False)
        print(f"\n详细报告已保存到: {args.save_report}")

if __name__ == "__main__":
    main()