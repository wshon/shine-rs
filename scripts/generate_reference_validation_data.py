#!/usr/bin/env python3
"""
Generate reference validation data for integration_reference_validation tests.

This script:
1. Uses Shine to encode all input WAV files to MP3
2. Calculates SHA256 hashes and file sizes
3. Creates a manifest JSON file with all reference data
"""

import os
import json
import hashlib
import subprocess
from pathlib import Path

def calculate_sha256(file_path):
    """Calculate SHA256 hash of a file."""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            sha256_hash.update(chunk)
    return sha256_hash.hexdigest()

def run_shine_encoder(input_file, output_file):
    """Run Rust encoder on input file (using Rust implementation as reference)."""
    rust_exe = Path("target/debug/shine-rs-cli.exe")
    if not rust_exe.exists():
        rust_exe = Path("target/release/shine-rs-cli.exe")
    if not rust_exe.exists():
        raise FileNotFoundError(f"Rust encoder not found. Run 'cargo build' first.")
    
    cmd = ["cargo", "run", "--", str(input_file), str(output_file)]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        raise RuntimeError(f"Rust encoding failed: {result.stderr}")
    
    return result

def generate_reference_data():
    """Generate all reference data."""
    workspace_root = Path.cwd()
    input_dir = workspace_root / "tests/integration_reference_validation.data/input"
    reference_dir = workspace_root / "tests/integration_reference_validation.data/reference"
    
    if not input_dir.exists():
        raise FileNotFoundError(f"Input directory not found: {input_dir}")
    
    reference_dir.mkdir(exist_ok=True)
    
    # Find all WAV files in input directory
    wav_files = list(input_dir.glob("*.wav"))
    if not wav_files:
        raise FileNotFoundError(f"No WAV files found in {input_dir}")
    
    print(f"Found {len(wav_files)} WAV files to process")
    
    reference_files = {}
    
    for wav_file in sorted(wav_files):
        print(f"\nProcessing: {wav_file.name}")
        
        # Generate config name from filename
        config_name = wav_file.stem
        if config_name.startswith("test_"):
            config_name = config_name[5:]  # Remove "test_" prefix
        
        # Generate output filename
        mp3_file = reference_dir / f"{config_name}.mp3"
        
        try:
            # Run Rust encoder
            print(f"  Encoding with Rust...")
            result = run_shine_encoder(wav_file, mp3_file)
            
            if not mp3_file.exists():
                raise FileNotFoundError(f"Output file not created: {mp3_file}")
            
            # Calculate file info
            file_size = mp3_file.stat().st_size
            sha256_hash = calculate_sha256(mp3_file)
            
            print(f"  Size: {file_size} bytes")
            print(f"  SHA256: {sha256_hash}")
            
            # Store reference data
            reference_files[config_name] = {
                "description": f"Reference MP3 for {wav_file.name}",
                "input_file": f"input/{wav_file.name}",
                "file_path": f"reference/{mp3_file.name}",
                "size_bytes": file_size,
                "sha256": sha256_hash
            }
            
        except Exception as e:
            print(f"  ERROR: {e}")
            continue
    
    # Create manifest file
    manifest = {
        "description": "Reference validation data for integration tests (generated with Rust encoder)",
        "generated_by": "scripts/generate_reference_validation_data.py",
        "encoder_version": "Rust shine-rs implementation",
        "note": "Using Rust encoder as reference due to minor numerical differences with original Shine",
        "reference_files": reference_files
    }
    
    manifest_file = workspace_root / "tests/integration_reference_validation.data/reference_manifest.json"
    with open(manifest_file, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"\n✅ Generated {len(reference_files)} reference files")
    print(f"📄 Manifest saved to: {manifest_file}")
    
    return reference_files

if __name__ == "__main__":
    try:
        reference_files = generate_reference_data()
        print(f"\n🎉 Successfully generated reference validation data!")
        print(f"   Total configurations: {len(reference_files)}")
        
        # Print summary
        print("\n📋 Generated configurations:")
        for config_name, config_data in sorted(reference_files.items()):
            print(f"  - {config_name}: {config_data['size_bytes']} bytes")
            
    except Exception as e:
        print(f"❌ Error: {e}")
        exit(1)