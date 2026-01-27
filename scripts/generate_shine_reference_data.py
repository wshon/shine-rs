#!/usr/bin/env python3
"""
Generate reference test data using Shine MP3 encoder

This script uses the Shine reference implementation to generate accurate
test data for integration tests. Based on debug_analysis.md findings,
the Rust implementation now matches Shine exactly, so we use Shine as
the authoritative source for test data.
"""

import os
import sys
import json
import subprocess
import hashlib
import struct
import wave
from datetime import datetime
from pathlib import Path

# Test configurations to generate
TEST_CONFIGS = [
    {
        "name": "sample-3s_128k_3f",
        "audio_file": "tests/audio/sample-3s.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "3-second sample at 128kbps, 3 frames"
    },
    {
        "name": "sample-3s_192k_3f", 
        "audio_file": "tests/audio/sample-3s.wav",
        "bitrate": 192,
        "frames": 3,
        "description": "3-second sample at 192kbps, 3 frames"
    },
    {
        "name": "voice_recorder_128k_3f",
        "audio_file": "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "Voice recording at 128kbps, 3 frames"
    },
    {
        "name": "free_test_data_128k_3f",
        "audio_file": "tests/audio/Free_Test_Data_500KB_WAV.wav", 
        "bitrate": 128,
        "frames": 3,
        "description": "Free test data at 128kbps, 3 frames"
    }
]

def read_wav_metadata(wav_path):
    """Read WAV file metadata"""
    try:
        with wave.open(wav_path, 'rb') as wav_file:
            channels = wav_file.getnchannels()
            sample_rate = wav_file.getframerate()
            frames = wav_file.getnframes()
            sample_width = wav_file.getsampwidth()
            
            return {
                "channels": channels,
                "sample_rate": sample_rate,
                "frames": frames,
                "sample_width": sample_width
            }
    except Exception as e:
        print(f"Error reading WAV file {wav_path}: {e}")
        return None

def calculate_sha256(file_path):
    """Calculate SHA256 hash of file"""
    sha256_hash = hashlib.sha256()
    try:
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest().upper()
    except Exception as e:
        print(f"Error calculating hash for {file_path}: {e}")
        return ""

def run_shine_encoder(audio_file, output_file, bitrate):
    """Run Shine encoder with specified parameters"""
    shine_exe = "ref/shine/shineenc.exe"
    
    if not os.path.exists(shine_exe):
        print(f"Error: Shine encoder not found at {shine_exe}")
        return False
    
    if not os.path.exists(audio_file):
        print(f"Error: Audio file not found at {audio_file}")
        return False
    
    # Run Shine encoder
    cmd = [shine_exe, "-b", str(bitrate), audio_file, output_file]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=".")
        if result.returncode == 0:
            print(f"✓ Shine encoding successful: {output_file}")
            return True
        else:
            print(f"✗ Shine encoding failed:")
            print(f"  Command: {' '.join(cmd)}")
            print(f"  Return code: {result.returncode}")
            print(f"  Stdout: {result.stdout}")
            print(f"  Stderr: {result.stderr}")
            return False
    except Exception as e:
        print(f"Error running Shine encoder: {e}")
        return False

def create_modified_shine_with_debug():
    """Create a modified version of Shine that outputs debug data"""
    
    # This would require modifying the Shine source code to output
    # intermediate values like MDCT coefficients, quantization parameters, etc.
    # For now, we'll generate basic test data structure
    
    print("Note: Full debug data extraction requires modified Shine source")
    print("Generating basic test data structure with placeholders")
    
    return True

def generate_test_data_structure(config, wav_metadata, mp3_file):
    """Generate test data structure"""
    
    # Calculate file size and hash
    file_size = os.path.getsize(mp3_file) if os.path.exists(mp3_file) else 0
    file_hash = calculate_sha256(mp3_file) if os.path.exists(mp3_file) else ""
    
    # Determine stereo mode based on channels
    stereo_mode = 3 if wav_metadata["channels"] == 1 else 0  # 3=mono, 0=stereo
    
    test_data = {
        "metadata": {
            "name": f"test_case_{config['name']}_{wav_metadata['sample_rate']}hz_{wav_metadata['channels']}ch_{config['bitrate']}kbps",
            "input_file": config["audio_file"],
            "expected_output_size": file_size,
            "expected_hash": file_hash,
            "created_at": datetime.utcnow().isoformat() + "Z",
            "description": config["description"],
            "generated_by": "Shine reference implementation"
        },
        "config": {
            "sample_rate": wav_metadata["sample_rate"],
            "channels": wav_metadata["channels"],
            "bitrate": config["bitrate"],
            "stereo_mode": stereo_mode,
            "mpeg_version": 3  # MPEG-I
        },
        "frames": []
    }
    
    # Generate placeholder frame data
    # In a full implementation, this would come from modified Shine output
    for frame_num in range(1, config["frames"] + 1):
        frame_data = {
            "frame_number": frame_num,
            "mdct_coefficients": {
                "coefficients": [0, 0, 0],  # Placeholder - would come from Shine debug output
                "l3_sb_sample": [0]  # Placeholder
            },
            "quantization": {
                "xrmax": 0,  # Placeholder
                "max_bits": 0,
                "part2_3_length": 0,
                "quantizer_step_size": 0,
                "global_gain": 0
            },
            "bitstream": {
                "padding": 0,
                "bits_per_frame": 0,
                "written": 0,
                "slot_lag": 0.0
            }
        }
        test_data["frames"].append(frame_data)
    
    return test_data

def main():
    """Main function to generate reference test data"""
    
    print("Generating Shine reference test data...")
    print("=" * 50)
    
    # Create output directory
    output_dir = Path("tests/pipeline_data")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Create temporary directory for MP3 files
    temp_dir = Path("temp_shine_output")
    temp_dir.mkdir(exist_ok=True)
    
    success_count = 0
    
    for config in TEST_CONFIGS:
        print(f"\nProcessing: {config['name']}")
        print(f"Audio file: {config['audio_file']}")
        print(f"Bitrate: {config['bitrate']}kbps")
        
        # Check if audio file exists
        if not os.path.exists(config["audio_file"]):
            print(f"⚠ Skipping {config['name']} - audio file not found")
            continue
        
        # Read WAV metadata
        wav_metadata = read_wav_metadata(config["audio_file"])
        if not wav_metadata:
            print(f"⚠ Skipping {config['name']} - could not read WAV metadata")
            continue
        
        print(f"WAV info: {wav_metadata['channels']}ch, {wav_metadata['sample_rate']}Hz")
        
        # Generate MP3 with Shine
        mp3_file = temp_dir / f"{config['name']}.mp3"
        if run_shine_encoder(config["audio_file"], str(mp3_file), config["bitrate"]):
            
            # Generate test data structure
            test_data = generate_test_data_structure(config, wav_metadata, str(mp3_file))
            
            # Save test data
            json_file = output_dir / f"{config['name']}.json"
            with open(json_file, 'w', encoding='utf-8') as f:
                json.dump(test_data, f, indent=2, ensure_ascii=False)
            
            print(f"✓ Generated test data: {json_file}")
            print(f"  Output size: {test_data['metadata']['expected_output_size']} bytes")
            print(f"  SHA256: {test_data['metadata']['expected_hash'][:16]}...")
            
            success_count += 1
        else:
            print(f"✗ Failed to generate MP3 for {config['name']}")
    
    # Cleanup
    if temp_dir.exists():
        import shutil
        shutil.rmtree(temp_dir)
    
    print(f"\n" + "=" * 50)
    print(f"Generated {success_count}/{len(TEST_CONFIGS)} test data files")
    
    if success_count > 0:
        print("\nNext steps:")
        print("1. To get full debug data, modify Shine source to output intermediate values")
        print("2. Run the integration tests: cargo test test_complete_encoding_pipeline")
        print("3. Verify hash matches between Rust and Shine implementations")
    
    return success_count > 0

if __name__ == "__main__":
    if main():
        sys.exit(0)
    else:
        sys.exit(1)