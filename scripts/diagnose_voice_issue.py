#!/usr/bin/env python3
"""
Voice File Issue Diagnosis Script

This script helps diagnose why the voice recording produces different outputs
between Rust and Shine encoders.
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

def run_encoder(encoder_type, input_file, output_file, frame_limit=None):
    """Run encoder and return success status and file info."""
    workspace_root = Path(".").resolve()
    audio_dir = workspace_root / "tests" / "audio"
    input_path = audio_dir / input_file
    
    if encoder_type == "rust":
        cmd = ["cargo", "run", "--", str(input_path), output_file]
        env = os.environ.copy()
        if frame_limit:
            env["RUST_MP3_MAX_FRAMES"] = str(frame_limit)
    else:  # shine
        shine_binary = workspace_root / "ref" / "shine" / "shineenc.exe"
        cmd = [str(shine_binary), str(input_path), output_file]
        env = os.environ.copy()
        if frame_limit:
            env["SHINE_MAX_FRAMES"] = str(frame_limit)
    
    try:
        result = subprocess.run(
            cmd,
            cwd=workspace_root if encoder_type == "rust" else shine_binary.parent,
            capture_output=True,
            text=True,
            timeout=30,
            env=env
        )
        
        if result.returncode == 0 and Path(output_file).exists():
            file_size = Path(output_file).stat().st_size
            file_hash = calculate_sha256(Path(output_file))
            return True, {
                "size": file_size,
                "hash": file_hash,
                "stdout": result.stdout,
                "stderr": result.stderr
            }
        else:
            return False, {
                "error": f"Exit code {result.returncode}",
                "stdout": result.stdout,
                "stderr": result.stderr
            }
    except Exception as e:
        return False, {"error": str(e)}

def analyze_wav_file(wav_file):
    """Analyze WAV file properties."""
    audio_dir = Path("tests/audio")
    wav_path = audio_dir / wav_file
    
    if not wav_path.exists():
        return {"error": f"File not found: {wav_path}"}
    
    file_size = wav_path.stat().st_size
    
    # Try to read WAV header
    try:
        with open(wav_path, "rb") as f:
            # Read WAV header
            riff = f.read(4)  # Should be b'RIFF'
            file_size_header = int.from_bytes(f.read(4), 'little')
            wave = f.read(4)  # Should be b'WAVE'
            
            # Find fmt chunk
            while True:
                chunk_id = f.read(4)
                if not chunk_id:
                    break
                chunk_size = int.from_bytes(f.read(4), 'little')
                
                if chunk_id == b'fmt ':
                    # Read format data
                    audio_format = int.from_bytes(f.read(2), 'little')
                    num_channels = int.from_bytes(f.read(2), 'little')
                    sample_rate = int.from_bytes(f.read(4), 'little')
                    byte_rate = int.from_bytes(f.read(4), 'little')
                    block_align = int.from_bytes(f.read(2), 'little')
                    bits_per_sample = int.from_bytes(f.read(2), 'little')
                    
                    return {
                        "file_size": file_size,
                        "audio_format": audio_format,
                        "channels": num_channels,
                        "sample_rate": sample_rate,
                        "byte_rate": byte_rate,
                        "block_align": block_align,
                        "bits_per_sample": bits_per_sample
                    }
                else:
                    # Skip this chunk
                    f.seek(chunk_size, 1)
                    
    except Exception as e:
        return {"error": f"Failed to parse WAV: {e}", "file_size": file_size}

def main():
    print("🔍 Diagnosing voice file encoding differences...")
    
    # Analyze the voice WAV file
    print("\n📊 Analyzing voice WAV file...")
    voice_info = analyze_wav_file("voice-recorder-testing-1-2-3-sound-file.wav")
    print(f"Voice file info: {voice_info}")
    
    # Compare with sample file
    print("\n📊 Analyzing sample WAV file...")
    sample_info = analyze_wav_file("sample-3s.wav")
    print(f"Sample file info: {sample_info}")
    
    # Test 3-frame encoding with both files
    test_configs = [
        ("voice-recorder-testing-1-2-3-sound-file.wav", 3),
        ("sample-3s.wav", 3)
    ]
    
    for input_file, frames in test_configs:
        print(f"\n🎵 Testing {input_file} with {frames} frames...")
        
        # Run Rust encoder
        rust_output = f"rust_{input_file.replace('.wav', '')}_{frames}frames.mp3"
        rust_success, rust_info = run_encoder("rust", input_file, rust_output, frames)
        
        # Run Shine encoder
        shine_output = f"shine_{input_file.replace('.wav', '')}_{frames}frames.mp3"
        shine_success, shine_info = run_encoder("shine", input_file, shine_output, frames)
        
        print(f"Rust result: {'✅' if rust_success else '❌'}")
        if rust_success:
            print(f"  Size: {rust_info['size']} bytes")
            print(f"  Hash: {rust_info['hash'][:16]}...")
        else:
            print(f"  Error: {rust_info['error']}")
        
        print(f"Shine result: {'✅' if shine_success else '❌'}")
        if shine_success:
            print(f"  Size: {shine_info['size']} bytes")
            print(f"  Hash: {shine_info['hash'][:16]}...")
        else:
            print(f"  Error: {shine_info['error']}")
        
        if rust_success and shine_success:
            if rust_info['hash'] == shine_info['hash']:
                print("  🎉 Hashes match!")
            else:
                print("  ⚠️  Hashes differ!")
                print(f"    Rust:  {rust_info['hash']}")
                print(f"    Shine: {shine_info['hash']}")
        
        # Clean up
        for output in [rust_output, shine_output]:
            if Path(output).exists():
                Path(output).unlink()

if __name__ == "__main__":
    main()