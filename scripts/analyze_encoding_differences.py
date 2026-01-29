#!/usr/bin/env python3
"""
Encoding Differences Analysis Script

This script analyzes the differences between Rust and Shine encoders
for different audio file types, focusing on mono vs stereo handling.
"""

import os
import sys
import subprocess
import hashlib
from pathlib import Path

def calculate_sha256(file_path):
    """Calculate SHA256 hash of a file."""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            sha256_hash.update(chunk)
    return sha256_hash.hexdigest()

def analyze_mp3_header(file_path):
    """Analyze MP3 file header information."""
    try:
        with open(file_path, "rb") as f:
            # Read first frame header
            data = f.read(4)
            if len(data) < 4:
                return {"error": "File too short"}
            
            # Parse MP3 header
            header = int.from_bytes(data, 'big')
            
            # Extract fields
            sync = (header >> 21) & 0x7FF
            version = (header >> 19) & 0x3
            layer = (header >> 17) & 0x3
            protection = (header >> 16) & 0x1
            bitrate_idx = (header >> 12) & 0xF
            samplerate_idx = (header >> 10) & 0x3
            padding = (header >> 9) & 0x1
            private = (header >> 8) & 0x1
            mode = (header >> 6) & 0x3
            mode_ext = (header >> 4) & 0x3
            copyright = (header >> 3) & 0x1
            original = (header >> 2) & 0x1
            emphasis = header & 0x3
            
            # Decode values
            mode_names = ["Stereo", "Joint Stereo", "Dual Channel", "Mono"]
            version_names = ["MPEG-2.5", "Reserved", "MPEG-2", "MPEG-1"]
            layer_names = ["Reserved", "Layer III", "Layer II", "Layer I"]
            
            bitrates = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0]
            samplerates_v1 = [44100, 48000, 32000, 0]
            samplerates_v2 = [22050, 24000, 16000, 0]
            
            return {
                "sync": f"0x{sync:03X}",
                "version": version_names[version] if version < len(version_names) else f"Unknown({version})",
                "layer": layer_names[layer] if layer < len(layer_names) else f"Unknown({layer})",
                "protection": "CRC" if protection == 0 else "No CRC",
                "bitrate": f"{bitrates[bitrate_idx]} kbps" if bitrate_idx < len(bitrates) else f"Unknown({bitrate_idx})",
                "samplerate": f"{samplerates_v1[samplerate_idx]} Hz" if version == 3 and samplerate_idx < len(samplerates_v1) else f"Unknown({samplerate_idx})",
                "padding": "Yes" if padding else "No",
                "mode": mode_names[mode] if mode < len(mode_names) else f"Unknown({mode})",
                "mode_ext": mode_ext,
                "copyright": "Yes" if copyright else "No",
                "original": "Yes" if original else "No",
                "emphasis": emphasis
            }
    except Exception as e:
        return {"error": str(e)}

def run_encoder_with_analysis(encoder_type, input_file, output_file, frame_limit=3):
    """Run encoder and analyze the output."""
    workspace_root = Path(".").resolve()
    audio_dir = workspace_root / "tests" / "audio"
    input_path = audio_dir / input_file
    
    if encoder_type == "rust":
        cmd = ["cargo", "run", "--", str(input_path), output_file]
        env = os.environ.copy()
        if frame_limit:
            env["RUST_MP3_MAX_FRAMES"] = str(frame_limit)
        cwd = workspace_root
    else:  # shine
        shine_binary = workspace_root / "ref" / "shine" / "shineenc.exe"
        cmd = [str(shine_binary), str(input_path), output_file]
        env = os.environ.copy()
        if frame_limit:
            env["SHINE_MAX_FRAMES"] = str(frame_limit)
        cwd = shine_binary.parent
    
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=30,
            env=env,
            encoding='utf-8',
            errors='ignore'  # Ignore encoding errors
        )
        
        if encoder_type == "shine":
            output_path = cwd / output_file
        else:
            output_path = Path(output_file)
        
        if result.returncode == 0 and output_path.exists():
            file_size = output_path.stat().st_size
            file_hash = calculate_sha256(output_path)
            header_info = analyze_mp3_header(output_path)
            
            return {
                "success": True,
                "size": file_size,
                "hash": file_hash,
                "header": header_info,
                "stdout": result.stdout[:500],  # Truncate for readability
                "stderr": result.stderr[:500]
            }
        else:
            return {
                "success": False,
                "error": f"Exit code {result.returncode}",
                "stdout": result.stdout[:500],
                "stderr": result.stderr[:500]
            }
    except Exception as e:
        return {"success": False, "error": str(e)}

def main():
    print("🔍 分析编码器差异...")
    
    workspace_root = Path(".").resolve()
    
    # Test configurations
    test_files = [
        ("sample-3s.wav", "立体声 44.1kHz"),
        ("voice-recorder-testing-1-2-3-sound-file.wav", "单声道 48kHz"),
        ("Free_Test_Data_500KB_WAV.wav", "大文件测试")
    ]
    
    for input_file, description in test_files:
        print(f"\n🎵 测试文件: {input_file} ({description})")
        
        # Generate outputs
        rust_output = f"analysis_rust_{input_file.replace('.wav', '.mp3')}"
        shine_output = f"analysis_shine_{input_file.replace('.wav', '.mp3')}"
        
        print("   运行Rust编码器...")
        rust_result = run_encoder_with_analysis("rust", input_file, rust_output)
        
        print("   运行Shine编码器...")
        shine_result = run_encoder_with_analysis("shine", input_file, shine_output)
        
        # Compare results
        print(f"\n   📊 结果对比:")
        print(f"   Rust: {'✅' if rust_result['success'] else '❌'}")
        if rust_result['success']:
            print(f"      大小: {rust_result['size']} 字节")
            print(f"      哈希: {rust_result['hash'][:16]}...")
            if 'error' not in rust_result['header']:
                print(f"      模式: {rust_result['header']['mode']}")
                print(f"      比特率: {rust_result['header']['bitrate']}")
                print(f"      采样率: {rust_result['header']['samplerate']}")
        else:
            print(f"      错误: {rust_result['error']}")
        
        print(f"   Shine: {'✅' if shine_result['success'] else '❌'}")
        if shine_result['success']:
            print(f"      大小: {shine_result['size']} 字节")
            print(f"      哈希: {shine_result['hash'][:16]}...")
            if 'error' not in shine_result['header']:
                print(f"      模式: {shine_result['header']['mode']}")
                print(f"      比特率: {shine_result['header']['bitrate']}")
                print(f"      采样率: {shine_result['header']['samplerate']}")
        else:
            print(f"      错误: {shine_result['error']}")
        
        # Check if outputs match
        if rust_result['success'] and shine_result['success']:
            if rust_result['hash'] == shine_result['hash']:
                print("   🎉 输出完全一致!")
            else:
                print("   ⚠️  输出不一致!")
                print(f"      大小差异: {rust_result['size'] - shine_result['size']} 字节")
                
                # Compare headers
                if ('error' not in rust_result['header'] and 
                    'error' not in shine_result['header']):
                    rust_header = rust_result['header']
                    shine_header = shine_result['header']
                    
                    differences = []
                    for key in rust_header:
                        if key in shine_header and rust_header[key] != shine_header[key]:
                            differences.append(f"{key}: Rust={rust_header[key]}, Shine={shine_header[key]}")
                    
                    if differences:
                        print("      头部差异:")
                        for diff in differences:
                            print(f"        {diff}")
        
        # Clean up
        for output in [rust_output, shine_output]:
            rust_path = Path(output)
            shine_path = workspace_root / "ref" / "shine" / output
            for path in [rust_path, shine_path]:
                if path.exists():
                    path.unlink()
        
        print()

if __name__ == "__main__":
    main()