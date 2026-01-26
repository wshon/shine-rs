#!/usr/bin/env python3
"""
Encoder Performance Benchmark Script

This script benchmarks the performance of Rust and Shine encoders using
the prepared reference files, providing detailed timing and throughput analysis.
"""

import os
import sys
import subprocess
import time
import json
from pathlib import Path
from typing import Dict, List, Tuple

class EncoderBenchmark:
    """Benchmarks encoder performance across different configurations."""
    
    def __init__(self, workspace_root: str = "."):
        self.workspace_root = Path(workspace_root).resolve()
        self.audio_dir = self.workspace_root / "tests" / "audio"
        self.manifest_file = self.audio_dir / "reference_manifest.json"
        
    def load_manifest(self) -> Dict:
        """Load the reference file manifest."""
        if not self.manifest_file.exists():
            raise FileNotFoundError(f"Manifest file not found: {self.manifest_file}")
        
        with open(self.manifest_file, 'r', encoding='utf-8') as f:
            return json.load(f)
    
    def get_input_file_from_config(self, config_name: str) -> str:
        """Extract input file name from config name."""
        if "voice" in config_name:
            return "voice-recorder-testing-1-2-3-sound-file.wav"
        elif "large" in config_name:
            return "Free_Test_Data_500KB_WAV.wav"
        else:
            return "sample-3s.wav"
    
    def get_frame_limit_from_config(self, config_name: str) -> int:
        """Extract frame limit from config name."""
        if "1frame" in config_name:
            return 1
        elif "2frames" in config_name:
            return 2
        elif "3frames" in config_name:
            return 3
        elif "6frames" in config_name:
            return 6
        elif "10frames" in config_name:
            return 10
        elif "15frames" in config_name:
            return 15
        elif "20frames" in config_name:
            return 20
        return 6  # default
    
    def benchmark_encoder(self, encoder_type: str, input_file: str, 
                         output_file: str, frame_limit: int, 
                         iterations: int = 3) -> Dict:
        """Benchmark a single encoder configuration."""
        input_path = self.audio_dir / input_file
        
        if not input_path.exists():
            return {"error": f"Input file not found: {input_path}"}
        
        times = []
        
        for i in range(iterations):
            if encoder_type == "rust":
                cmd = ["cargo", "run", "--release", "--", str(input_path), output_file]
                env = os.environ.copy()
                env["RUST_MP3_MAX_FRAMES"] = str(frame_limit)
                cwd = self.workspace_root
            else:  # shine
                shine_binary = self.workspace_root / "ref" / "shine" / "shineenc.exe"
                cmd = [str(shine_binary), str(input_path), output_file]
                env = os.environ.copy()
                env["SHINE_MAX_FRAMES"] = str(frame_limit)
                cwd = shine_binary.parent
            
            try:
                start_time = time.time()
                result = subprocess.run(
                    cmd,
                    cwd=cwd,
                    capture_output=True,
                    text=True,
                    timeout=30,
                    env=env,
                    encoding='utf-8',
                    errors='ignore'
                )
                end_time = time.time()
                
                if result.returncode == 0:
                    times.append(end_time - start_time)
                    # Clean up output file
                    output_path = cwd / output_file if encoder_type == "shine" else Path(output_file)
                    if output_path.exists():
                        output_path.unlink()
                else:
                    return {
                        "error": f"Encoding failed: exit code {result.returncode}",
                        "stderr": result.stderr[:200]
                    }
            except subprocess.TimeoutExpired:
                return {"error": "Timeout expired"}
            except Exception as e:
                return {"error": str(e)}
        
        if times:
            avg_time = sum(times) / len(times)
            min_time = min(times)
            max_time = max(times)
            
            # Calculate throughput (frames per second)
            fps = frame_limit / avg_time if avg_time > 0 else 0
            
            return {
                "success": True,
                "iterations": iterations,
                "times": times,
                "avg_time": avg_time,
                "min_time": min_time,
                "max_time": max_time,
                "frames_per_second": fps,
                "frames": frame_limit
            }
        else:
            return {"error": "No successful runs"}
    
    def run_benchmark_suite(self, configs: List[str] = None, 
                           iterations: int = 3) -> Dict:
        """Run comprehensive benchmark suite."""
        print("🚀 开始编码器性能基准测试...")
        print(f"   工作目录: {self.workspace_root}")
        print(f"   迭代次数: {iterations}")
        
        # Load manifest
        try:
            manifest = self.load_manifest()
        except Exception as e:
            return {"error": f"Failed to load manifest: {e}"}
        
        reference_files = manifest.get("reference_files", {})
        
        # Filter configs if specified
        if configs:
            available_configs = set(reference_files.keys())
            requested_configs = set(configs)
            missing_configs = requested_configs - available_configs
            
            if missing_configs:
                return {"error": f"Unknown configs: {missing_configs}"}
            
            reference_files = {k: v for k, v in reference_files.items() if k in configs}
        
        print(f"   测试配置: {len(reference_files)}个")
        
        results = {}
        
        for config_name, reference_info in reference_files.items():
            print(f"\n📊 基准测试: {config_name}")
            print(f"   描述: {reference_info['description']}")
            
            input_file = self.get_input_file_from_config(config_name)
            frame_limit = self.get_frame_limit_from_config(config_name)
            
            # Benchmark Rust encoder
            print("   测试Rust编码器...")
            rust_result = self.benchmark_encoder(
                "rust", input_file, f"bench_rust_{config_name}.mp3", 
                frame_limit, iterations
            )
            
            # Benchmark Shine encoder
            print("   测试Shine编码器...")
            shine_result = self.benchmark_encoder(
                "shine", input_file, f"bench_shine_{config_name}.mp3", 
                frame_limit, iterations
            )
            
            results[config_name] = {
                "config": config_name,
                "input_file": input_file,
                "frame_limit": frame_limit,
                "rust": rust_result,
                "shine": shine_result
            }
            
            # Print results
            if rust_result.get("success"):
                print(f"   Rust:  {rust_result['avg_time']:.3f}s 平均 ({rust_result['frames_per_second']:.1f} fps)")
            else:
                print(f"   Rust:  ❌ {rust_result.get('error', 'Unknown error')}")
            
            if shine_result.get("success"):
                print(f"   Shine: {shine_result['avg_time']:.3f}s 平均 ({shine_result['frames_per_second']:.1f} fps)")
            else:
                print(f"   Shine: ❌ {shine_result.get('error', 'Unknown error')}")
            
            # Calculate speedup if both succeeded
            if (rust_result.get("success") and shine_result.get("success")):
                speedup = shine_result['avg_time'] / rust_result['avg_time']
                if speedup > 1:
                    print(f"   🚀 Rust比Shine快 {speedup:.2f}x")
                elif speedup < 1:
                    print(f"   🐌 Rust比Shine慢 {1/speedup:.2f}x")
                else:
                    print(f"   ⚖️  性能相当")
        
        return {
            "success": True,
            "benchmark_time": time.strftime("%Y-%m-%d %H:%M:%S"),
            "iterations": iterations,
            "results": results
        }
    
    def generate_report(self, benchmark_results: Dict, output_file: str = None):
        """Generate a detailed benchmark report."""
        if not benchmark_results.get("success"):
            print(f"❌ 基准测试失败: {benchmark_results.get('error')}")
            return
        
        results = benchmark_results["results"]
        
        print(f"\n📈 性能基准测试报告")
        print(f"=" * 60)
        print(f"测试时间: {benchmark_results['benchmark_time']}")
        print(f"迭代次数: {benchmark_results['iterations']}")
        print(f"测试配置: {len(results)}个")
        
        # Summary statistics
        rust_times = []
        shine_times = []
        speedups = []
        
        print(f"\n详细结果:")
        print(f"{'配置':<15} {'帧数':<4} {'Rust(s)':<8} {'Shine(s)':<8} {'加速比':<8} {'状态'}")
        print(f"-" * 60)
        
        for config_name, result in results.items():
            rust_result = result["rust"]
            shine_result = result["shine"]
            frame_limit = result["frame_limit"]
            
            rust_time_str = f"{rust_result['avg_time']:.3f}" if rust_result.get("success") else "FAIL"
            shine_time_str = f"{shine_result['avg_time']:.3f}" if shine_result.get("success") else "FAIL"
            
            if rust_result.get("success") and shine_result.get("success"):
                speedup = shine_result['avg_time'] / rust_result['avg_time']
                speedup_str = f"{speedup:.2f}x"
                status = "✅"
                
                rust_times.append(rust_result['avg_time'])
                shine_times.append(shine_result['avg_time'])
                speedups.append(speedup)
            else:
                speedup_str = "N/A"
                status = "❌"
            
            print(f"{config_name:<15} {frame_limit:<4} {rust_time_str:<8} {shine_time_str:<8} {speedup_str:<8} {status}")
        
        # Overall statistics
        if rust_times and shine_times:
            avg_rust = sum(rust_times) / len(rust_times)
            avg_shine = sum(shine_times) / len(shine_times)
            avg_speedup = sum(speedups) / len(speedups)
            
            print(f"\n总体统计:")
            print(f"平均Rust时间:  {avg_rust:.3f}s")
            print(f"平均Shine时间: {avg_shine:.3f}s")
            print(f"平均加速比:    {avg_speedup:.2f}x")
            
            if avg_speedup > 1:
                print(f"🎉 Rust编码器平均比Shine快 {avg_speedup:.2f}倍!")
            elif avg_speedup < 1:
                print(f"⚠️  Rust编码器平均比Shine慢 {1/avg_speedup:.2f}倍")
            else:
                print(f"⚖️  两个编码器性能相当")
        
        # Save detailed report if requested
        if output_file:
            report_path = Path(output_file)
            with open(report_path, 'w', encoding='utf-8') as f:
                json.dump(benchmark_results, f, indent=2, ensure_ascii=False)
            print(f"\n📄 详细报告已保存到: {report_path}")

def main():
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Benchmark Rust and Shine encoder performance"
    )
    parser.add_argument(
        "--configs", 
        nargs="+", 
        help="Specific configs to benchmark (default: all)"
    )
    parser.add_argument(
        "--iterations", 
        type=int, 
        default=3,
        help="Number of iterations per test (default: 3)"
    )
    parser.add_argument(
        "--output", 
        help="Output file for detailed JSON report"
    )
    parser.add_argument(
        "--workspace", 
        default=".",
        help="Workspace root directory (default: current directory)"
    )
    
    args = parser.parse_args()
    
    benchmark = EncoderBenchmark(args.workspace)
    results = benchmark.run_benchmark_suite(
        configs=args.configs,
        iterations=args.iterations
    )
    
    benchmark.generate_report(results, args.output)
    
    if results.get("success"):
        print("\n🎉 基准测试完成!")
        sys.exit(0)
    else:
        print(f"\n💥 基准测试失败: {results.get('error')}")
        sys.exit(1)

if __name__ == "__main__":
    main()