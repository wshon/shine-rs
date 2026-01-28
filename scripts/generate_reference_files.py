#!/usr/bin/env python3
"""
Reference File Generator for MP3 Encoder Tests

This script generates reference MP3 files using the Shine encoder and validates
them for use in automated testing. It ensures reproducible test data across
different environments.

Usage:
    python scripts/generate_reference_files.py [options]

Requirements:
    - Shine encoder binary at ref/shine/shineenc.exe
    - Input WAV files in tests/audio/
    - Python 3.6+
"""

import os
import sys
import subprocess
import hashlib
import json
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import shutil

class ReferenceFileGenerator:
    """Generates and validates reference files for MP3 encoder testing."""
    
    def __init__(self, workspace_root: str = "."):
        self.workspace_root = Path(workspace_root).resolve()
        self.shine_binary = self.workspace_root / "ref" / "shine" / "shineenc"
        self.audio_dir = self.workspace_root / "tests" / "audio"
        self.reference_configs = {
            # Basic frame count tests
            "3frames": {
                "description": "3-frame reference for quick testing",
                "frame_limit": 3,
                "expected_size": 1252,
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_3frames.mp3"
            },
            "6frames": {
                "description": "6-frame reference for SCFSI consistency testing",
                "frame_limit": 6,
                "expected_size": 2508,
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_6frames.mp3"
            },
            "10frames": {
                "description": "10-frame reference for extended testing",
                "frame_limit": 10,
                "expected_size": 4180,
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_10frames.mp3"
            },
            "15frames": {
                "description": "15-frame reference for medium-length testing",
                "frame_limit": 15,
                "expected_size": 6268,  # Updated from actual generation
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_15frames.mp3"
            },
            "20frames": {
                "description": "20-frame reference for longer testing",
                "frame_limit": 20,
                "expected_size": 8360,  # Updated from actual generation
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_20frames.mp3"
            },
            
            # Different input file tests (if available)
            "voice_3frames": {
                "description": "3-frame reference using voice recording",
                "frame_limit": 3,
                "expected_size": 1152,  # Updated from actual generation
                "input_file": "voice-recorder-testing-1-2-3-sound-file.wav",
                "output_file": "shine_reference_voice_3frames.mp3"
            },
            "voice_6frames": {
                "description": "6-frame reference using voice recording",
                "frame_limit": 6,
                "expected_size": 2304,  # Updated from actual generation
                "input_file": "voice-recorder-testing-1-2-3-sound-file.wav",
                "output_file": "shine_reference_voice_6frames.mp3"
            },
            
            # Large file tests (if available)
            "large_3frames": {
                "description": "3-frame reference using large test file",
                "frame_limit": 3,
                "expected_size": 1252,  # Will be updated after first generation
                "input_file": "Free_Test_Data_500KB_WAV.wav",
                "output_file": "shine_reference_large_3frames.mp3"
            },
            "large_6frames": {
                "description": "6-frame reference using large test file",
                "frame_limit": 6,
                "expected_size": 2508,  # Will be updated after first generation
                "input_file": "Free_Test_Data_500KB_WAV.wav",
                "output_file": "shine_reference_large_6frames.mp3"
            },
            
            # Edge case tests
            "1frame": {
                "description": "Single frame reference for minimal testing",
                "frame_limit": 1,
                "expected_size": 416,  # Updated from actual generation
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_1frame.mp3"
            },
            "2frames": {
                "description": "2-frame reference for minimal pair testing",
                "frame_limit": 2,
                "expected_size": 836,  # Updated from actual generation
                "input_file": "sample-3s.wav",
                "output_file": "shine_reference_2frames.mp3"
            }
        }
    
    def check_prerequisites(self) -> bool:
        """Check if all required files and tools are available."""
        print("🔍 Checking prerequisites...")
        
        # Check Shine binary (try different extensions)
        shine_candidates = [
            self.shine_binary,
            self.shine_binary.with_suffix('.exe'),
            self.shine_binary.parent / 'shineenc.exe'
        ]
        
        shine_found = None
        for candidate in shine_candidates:
            if candidate.exists():
                shine_found = candidate
                break
        
        if not shine_found:
            print(f"❌ Shine encoder not found. Tried:")
            for candidate in shine_candidates:
                print(f"   - {candidate}")
            print("   Please ensure Shine is built and available.")
            return False
        
        self.shine_binary = shine_found
        print(f"✅ Shine encoder found: {self.shine_binary}")
        
        # Check audio directory
        if not self.audio_dir.exists():
            print(f"❌ Audio directory not found: {self.audio_dir}")
            return False
        print(f"✅ Audio directory found: {self.audio_dir}")
        
        # Check input files
        missing_files = []
        for config in self.reference_configs.values():
            input_path = self.audio_dir / config["input_file"]
            if not input_path.exists():
                missing_files.append(str(input_path))
            else:
                print(f"✅ Input file found: {input_path}")
        
        if missing_files:
            print("❌ Missing input files:")
            for file in missing_files:
                print(f"   - {file}")
            return False
        
        return True
    
    def calculate_sha256(self, file_path: Path) -> str:
        """Calculate SHA256 hash of a file."""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest()
    
    def run_shine_encoder(self, input_file: Path, output_file: Path, 
                         frame_limit: Optional[int] = None) -> Tuple[bool, str]:
        """Run Shine encoder with specified parameters."""
        cmd = [str(self.shine_binary), str(input_file), str(output_file)]
        
        print(f"🎵 Running Shine encoder...")
        print(f"   Command: {' '.join(cmd)}")
        print(f"   Frame limit: {frame_limit if frame_limit else 'unlimited'}")
        
        # Set up environment variables
        env = os.environ.copy()
        if frame_limit is not None:
            env["SHINE_MAX_FRAMES"] = str(frame_limit)
        
        try:
            # Change to Shine directory to ensure proper execution
            result = subprocess.run(
                cmd,
                cwd=self.shine_binary.parent,
                capture_output=True,
                text=True,
                timeout=30,
                env=env
            )
            
            if result.returncode == 0:
                print("✅ Shine encoder completed successfully")
                return True, result.stdout
            else:
                print(f"❌ Shine encoder failed with code {result.returncode}")
                print(f"   stdout: {result.stdout}")
                print(f"   stderr: {result.stderr}")
                return False, result.stderr
                
        except subprocess.TimeoutExpired:
            print("❌ Shine encoder timed out")
            return False, "Timeout expired"
        except Exception as e:
            print(f"❌ Error running Shine encoder: {e}")
            return False, str(e)
    
    def validate_output(self, output_file: Path, expected_size: int, 
                       config_name: str = None) -> Dict:
        """Validate the generated output file."""
        if not output_file.exists():
            return {
                "valid": False,
                "error": f"Output file not found: {output_file}"
            }
        
        # Check file size
        actual_size = output_file.stat().st_size
        
        # For new configurations, we might not know the exact size
        # If expected_size is an estimate (ends with common frame sizes), 
        # we'll accept the actual size and suggest updating the config
        size_mismatch = actual_size != expected_size
        if size_mismatch and config_name:
            print(f"   📏 Size difference detected for {config_name}:")
            print(f"      Expected: {expected_size} bytes")
            print(f"      Actual:   {actual_size} bytes")
            print(f"      Consider updating the expected_size in the configuration")
        
        # Calculate hash
        file_hash = self.calculate_sha256(output_file)
        
        return {
            "valid": True,  # Always valid if file exists
            "size": actual_size,
            "expected_size": expected_size,
            "size_matches": not size_mismatch,
            "sha256": file_hash,
            "path": str(output_file)
        }
    
    def generate_reference_file(self, config_name: str) -> Dict:
        """Generate a single reference file."""
        config = self.reference_configs[config_name]
        print(f"\n📁 Generating reference file: {config_name}")
        print(f"   Description: {config['description']}")
        
        input_path = self.audio_dir / config["input_file"]
        output_path = self.audio_dir / config["output_file"]
        
        # Remove existing output file
        if output_path.exists():
            output_path.unlink()
            print(f"   Removed existing file: {output_path}")
        
        # Run Shine encoder
        success, message = self.run_shine_encoder(
            input_path, 
            output_path,
            config.get("frame_limit")
        )
        
        if not success:
            return {
                "config": config_name,
                "success": False,
                "error": message
            }
        
        # Validate output
        validation = self.validate_output(output_path, config["expected_size"], config_name)
        
        if validation["valid"]:
            print(f"✅ Reference file generated successfully")
            print(f"   File: {output_path}")
            print(f"   Size: {validation['size']} bytes")
            if not validation.get("size_matches", True):
                print(f"   ⚠️  Size differs from expected ({validation['expected_size']} bytes)")
            print(f"   SHA256: {validation['sha256']}")
            
            return {
                "config": config_name,
                "success": True,
                "file_path": str(output_path),
                "size": validation["size"],
                "expected_size": validation["expected_size"],
                "size_matches": validation.get("size_matches", True),
                "sha256": validation["sha256"],
                "description": config["description"]
            }
        else:
            print(f"❌ Validation failed: {validation['error']}")
            return {
                "config": config_name,
                "success": False,
                "error": validation["error"]
            }
    
    def update_test_constants(self, results: List[Dict]) -> bool:
        """Update test constants with new hash values."""
        print("\n🔧 Updating test constants...")
        
        # Find the 6-frame result for SCFSI tests
        scfsi_result = None
        for result in results:
            if result["config"] == "6frames" and result["success"]:
                scfsi_result = result
                break
        
        if not scfsi_result:
            print("❌ No successful 6-frame result found for SCFSI tests")
            return False
        
        # Update SCFSI test file
        scfsi_test_file = self.workspace_root / "tests" / "integration_scfsi_consistency.rs"
        if not scfsi_test_file.exists():
            print(f"❌ SCFSI test file not found: {scfsi_test_file}")
            return False
        
        try:
            # Read current content
            content = scfsi_test_file.read_text(encoding='utf-8')
            
            # Update hash constant
            old_hash_line = None
            new_hash_line = f'const EXPECTED_SHINE_HASH: &str = "{scfsi_result["sha256"]}";'
            
            lines = content.split('\n')
            for i, line in enumerate(lines):
                if line.strip().startswith('const EXPECTED_SHINE_HASH:'):
                    old_hash_line = line.strip()
                    lines[i] = new_hash_line
                    break
            
            if old_hash_line:
                # Write updated content
                scfsi_test_file.write_text('\n'.join(lines), encoding='utf-8')
                print(f"✅ Updated SCFSI test constants")
                print(f"   Old: {old_hash_line}")
                print(f"   New: {new_hash_line}")
                return True
            else:
                print("❌ Could not find EXPECTED_SHINE_HASH constant in test file")
                return False
                
        except Exception as e:
            print(f"❌ Error updating test file: {e}")
            return False
    
    def generate_manifest(self, results: List[Dict]) -> bool:
        """Generate a manifest file with all reference file information."""
        manifest_path = self.audio_dir / "reference_manifest.json"
        
        # Get current timestamp in a cross-platform way
        from datetime import datetime
        current_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        manifest = {
            "generated_at": current_time,
            "generator_version": "1.0.0",
            "shine_binary": str(self.shine_binary),
            "reference_files": {}
        }
        
        for result in results:
            if result["success"]:
                manifest["reference_files"][result["config"]] = {
                    "description": result["description"],
                    "file_path": result["file_path"],
                    "size_bytes": result["size"],
                    "sha256": result["sha256"]
                }
        
        try:
            with open(manifest_path, 'w', encoding='utf-8') as f:
                json.dump(manifest, f, indent=2, ensure_ascii=False)
            print(f"✅ Generated manifest: {manifest_path}")
            return True
        except Exception as e:
            print(f"❌ Error generating manifest: {e}")
            return False
    
    def run(self, configs: Optional[List[str]] = None, 
            update_tests: bool = True) -> bool:
        """Run the reference file generation process."""
        print("🚀 Starting reference file generation...")
        print(f"   Workspace: {self.workspace_root}")
        
        # Check prerequisites
        if not self.check_prerequisites():
            return False
        
        # Determine which configs to generate
        if configs is None:
            configs = list(self.reference_configs.keys())
        
        # Validate config names
        invalid_configs = [c for c in configs if c not in self.reference_configs]
        if invalid_configs:
            print(f"❌ Invalid config names: {invalid_configs}")
            print(f"   Available configs: {list(self.reference_configs.keys())}")
            return False
        
        # Generate reference files
        results = []
        for config_name in configs:
            result = self.generate_reference_file(config_name)
            results.append(result)
        
        # Summary
        successful = [r for r in results if r["success"]]
        failed = [r for r in results if not r["success"]]
        
        print(f"\n📊 Generation Summary:")
        print(f"   ✅ Successful: {len(successful)}")
        print(f"   ❌ Failed: {len(failed)}")
        
        if failed:
            print("\n❌ Failed generations:")
            for result in failed:
                print(f"   - {result['config']}: {result['error']}")
        
        if successful:
            print("\n✅ Successful generations:")
            for result in successful:
                print(f"   - {result['config']}: {result['file_path']}")
                print(f"     Size: {result['size']} bytes, SHA256: {result['sha256'][:16]}...")
        
        # Update test constants if requested
        if update_tests and successful:
            self.update_test_constants(results)
        
        # Generate manifest
        if successful:
            self.generate_manifest(results)
        
        return len(failed) == 0

def main():
    parser = argparse.ArgumentParser(
        description="Generate reference files for MP3 encoder testing"
    )
    parser.add_argument(
        "--configs", 
        nargs="+", 
        help="Specific configs to generate (default: all)"
    )
    parser.add_argument(
        "--no-update-tests", 
        action="store_true",
        help="Don't update test constants automatically"
    )
    parser.add_argument(
        "--workspace", 
        default=".",
        help="Workspace root directory (default: current directory)"
    )
    
    args = parser.parse_args()
    
    generator = ReferenceFileGenerator(args.workspace)
    success = generator.run(
        configs=args.configs,
        update_tests=not args.no_update_tests
    )
    
    if success:
        print("\n🎉 Reference file generation completed successfully!")
        print("\nNext steps:")
        print("1. Run tests to verify the new reference files")
        print("2. Commit the updated reference files and test constants")
        print("3. Update documentation if needed")
        sys.exit(0)
    else:
        print("\n💥 Reference file generation failed!")
        print("Please check the errors above and try again.")
        sys.exit(1)

if __name__ == "__main__":
    main()