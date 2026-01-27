#!/usr/bin/env python3
"""
Complete reference data generator for MP3 encoder testing

This script automatically:
1. Runs Shine encoder with debug output for specified audio files
2. Parses the debug output to extract MDCT coefficients, quantization parameters, and bitstream data
3. Generates JSON test data files with real Shine reference values
4. Calculates MP3 file hashes for validation

Usage:
    python scripts/generate_reference_data.py
"""

import os
import sys
import json
import re
import hashlib
import subprocess
import wave
from datetime import datetime
from pathlib import Path

# Test configurations to generate
TEST_CONFIGS = [
    {
        "name": "sample-3s_128k_3f_real",
        "audio_file": "tests/audio/sample-3s.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "3-second sample at 128kbps, 3 frames - real Shine data"
    },
    {
        "name": "voice_recorder_128k_3f_real",
        "audio_file": "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "Voice recording at 128kbps, 3 frames - real Shine data"
    },
    {
        "name": "free_test_data_128k_3f_real",
        "audio_file": "tests/audio/Free_Test_Data_500KB_WAV.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "Free test data at 128kbps, 3 frames - real Shine data"
    },
    {
        "name": "sample-3s_192k_3f_real",
        "audio_file": "tests/audio/sample-3s.wav",
        "bitrate": 192,
        "frames": 3,
        "description": "3-second sample at 192kbps, 3 frames - real Shine data"
    }
]

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

def run_shine_with_debug(audio_file, output_file, bitrate, max_frames):
    """Run Shine encoder with debug output"""
    shine_exe = "ref/shine/shineenc.exe"
    
    if not os.path.exists(shine_exe):
        print(f"Error: Shine encoder not found at {shine_exe}")
        return None, None
    
    if not os.path.exists(audio_file):
        print(f"Error: Audio file not found at {audio_file}")
        return None, None
    
    # Convert to absolute paths
    audio_file_abs = os.path.abspath(audio_file)
    
    # Set environment variable for frame limit
    env = os.environ.copy()
    env["SHINE_MAX_FRAMES"] = str(max_frames)
    
    # Run Shine encoder with debug output
    cmd = [shine_exe, "-b", str(bitrate), audio_file_abs, output_file]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, 
                              cwd="ref/shine", env=env, encoding='utf-8', errors='replace')
        
        if result.returncode == 0:
            print(f"✓ Shine encoding successful: {output_file}")
            # Combine stdout and stderr for debug output
            debug_output = result.stdout + result.stderr
            return debug_output, f"ref/shine/{output_file}"
        else:
            print(f"✗ Shine encoding failed:")
            print(f"  Command: {' '.join(cmd)}")
            print(f"  Return code: {result.returncode}")
            print(f"  Stdout: {result.stdout}")
            print(f"  Stderr: {result.stderr}")
            return None, None
    except Exception as e:
        print(f"Error running Shine encoder: {e}")
        return None, None

def parse_shine_debug_output(debug_output):
    """Parse Shine debug output to extract frame data"""
    
    frames = {}
    current_frame = None
    
    for line in debug_output.split('\n'):
        line = line.strip()
        
        # Parse frame-specific debug output
        if "[SHINE DEBUG Frame" in line:
            # Extract frame number
            frame_match = re.search(r'Frame (\d+)', line)
            if frame_match:
                frame_num = int(frame_match.group(1))
                
                # Initialize frame if new
                if frame_num not in frames:
                    frames[frame_num] = {
                        'frame_number': frame_num,
                        'mdct_coefficients': {
                            'coefficients': [0, 0, 0],
                            'l3_sb_sample': [0]
                        },
                        'quantization': {
                            'xrmax': 0,
                            'max_bits': 0,
                            'part2_3_length': 0,
                            'quantizer_step_size': 0,
                            'global_gain': 0
                        },
                        'bitstream': {
                            'padding': 0,
                            'bits_per_frame': 0,
                            'written': 0,
                            'slot_lag': 0.0
                        }
                    }
                
                current_frame = frames[frame_num]
            
            # Parse specific debug values
            if "MDCT coeff band 0 k 17:" in line:
                coeff_match = re.search(r'k 17: (-?\d+)', line)
                if coeff_match:
                    current_frame['mdct_coefficients']['coefficients'][0] = int(coeff_match.group(1))
            
            elif "MDCT coeff band 0 k 16:" in line:
                coeff_match = re.search(r'k 16: (-?\d+)', line)
                if coeff_match:
                    current_frame['mdct_coefficients']['coefficients'][1] = int(coeff_match.group(1))
            
            elif "MDCT coeff band 0 k 15:" in line:
                coeff_match = re.search(r'k 15: (-?\d+)', line)
                if coeff_match:
                    current_frame['mdct_coefficients']['coefficients'][2] = int(coeff_match.group(1))
            
            elif "l3_sb_sample[0][1][0]: first 8 bands:" in line:
                # Extract l3_sb_sample - get first value from the array
                sample_match = re.search(r'first 8 bands: \[(-?\d+)', line)
                if sample_match:
                    current_frame['mdct_coefficients']['l3_sb_sample'] = [
                        int(sample_match.group(1))
                    ]
            
            elif "ch=0, gr=0: xrmax=" in line:
                # Extract xrmax for channel 0, granule 0 (first granule)
                xrmax_match = re.search(r'xrmax=(-?\d+)', line)
                if xrmax_match:
                    current_frame['quantization']['xrmax'] = int(xrmax_match.group(1))
            
            elif "ch=0, gr=0: max_bits=" in line:
                # Extract max_bits
                bits_match = re.search(r'max_bits=(-?\d+)', line)
                if bits_match:
                    current_frame['quantization']['max_bits'] = int(bits_match.group(1))
            
            elif "ch=0, gr=0: part2_3_length=" in line:
                # Extract part2_3_length
                length_match = re.search(r'part2_3_length=(-?\d+)', line)
                if length_match:
                    current_frame['quantization']['part2_3_length'] = int(length_match.group(1))
            
            elif "ch=0, gr=0: quantizerStepSize=" in line:
                # Extract quantizer step size and global gain
                step_match = re.search(r'quantizerStepSize=(-?\d+), global_gain=(-?\d+)', line)
                if step_match:
                    current_frame['quantization']['quantizer_step_size'] = int(step_match.group(1))
                    current_frame['quantization']['global_gain'] = int(step_match.group(2))
            
            elif "padding=" in line and "bits_per_frame=" in line and "slot_lag=" in line:
                # Extract bitstream parameters
                padding_match = re.search(r'padding=(-?\d+)', line)
                bits_match = re.search(r'bits_per_frame=(-?\d+)', line)
                lag_match = re.search(r'slot_lag=(-?\d+\.?\d*)', line)
                
                if padding_match:
                    current_frame['bitstream']['padding'] = int(padding_match.group(1))
                if bits_match:
                    current_frame['bitstream']['bits_per_frame'] = int(bits_match.group(1))
                if lag_match:
                    current_frame['bitstream']['slot_lag'] = float(lag_match.group(1))
            
            elif "written=" in line and "bytes" in line:
                # Extract written bytes
                written_match = re.search(r'written=(-?\d+)', line)
                if written_match:
                    current_frame['bitstream']['written'] = int(written_match.group(1))
    
    # Convert to sorted list
    frame_list = []
    for frame_num in sorted(frames.keys()):
        frame_list.append(frames[frame_num])
    
    return frame_list

def generate_test_data_structure(config, wav_metadata, mp3_file, frames):
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
            "created_at": datetime.now().isoformat() + "Z",
            "description": config["description"],
            "generated_by": "Shine reference implementation with debug output"
        },
        "config": {
            "sample_rate": wav_metadata["sample_rate"],
            "channels": wav_metadata["channels"],
            "bitrate": config["bitrate"],
            "stereo_mode": stereo_mode,
            "mpeg_version": 3  # MPEG-I
        },
        "frames": frames[:config["frames"]]  # Limit to specified frame count
    }
    
    return test_data

def main():
    """Main function to generate all reference test data"""
    
    print("MP3 Encoder Reference Data Generator")
    print("=" * 50)
    print("Generating reference data using Shine encoder with debug output")
    print()
    
    # Create output directory
    output_dir = Path("tests/pipeline_data")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    success_count = 0
    
    for config in TEST_CONFIGS:
        print(f"Processing: {config['name']}")
        print(f"Audio file: {config['audio_file']}")
        print(f"Bitrate: {config['bitrate']}kbps, Frames: {config['frames']}")
        
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
        
        # Generate MP3 with Shine and capture debug output
        mp3_filename = f"{config['name']}.mp3"
        debug_output, mp3_file = run_shine_with_debug(
            config["audio_file"], mp3_filename, config["bitrate"], config["frames"]
        )
        
        if debug_output is None or mp3_file is None:
            print(f"✗ Failed to generate MP3 for {config['name']}")
            continue
        
        # Parse debug data
        frames = parse_shine_debug_output(debug_output)
        
        if not frames:
            print(f"✗ No debug data extracted for {config['name']}")
            continue
        
        # Generate test data structure
        test_data = generate_test_data_structure(config, wav_metadata, mp3_file, frames)
        
        # Save test data
        json_file = output_dir / f"{config['name']}.json"
        with open(json_file, 'w', encoding='utf-8') as f:
            json.dump(test_data, f, indent=2, ensure_ascii=False)
        
        print(f"✓ Generated test data: {json_file}")
        print(f"  Output size: {test_data['metadata']['expected_output_size']} bytes")
        print(f"  SHA256: {test_data['metadata']['expected_hash'][:16]}...")
        print(f"  Frames extracted: {len(frames)}")
        
        # Print sample data for verification
        if frames:
            frame1 = frames[0]
            print(f"  Frame 1 sample data:")
            print(f"    MDCT coefficients: {frame1['mdct_coefficients']['coefficients']}")
            print(f"    l3_sb_sample: {frame1['mdct_coefficients']['l3_sb_sample']}")
            print(f"    xrmax: {frame1['quantization']['xrmax']}")
            print(f"    global_gain: {frame1['quantization']['global_gain']}")
            print(f"    padding: {frame1['bitstream']['padding']}")
            print(f"    written: {frame1['bitstream']['written']}")
        
        success_count += 1
        print()
    
    print("=" * 50)
    print(f"Generated {success_count}/{len(TEST_CONFIGS)} reference data files")
    
    if success_count > 0:
        print("\\nNext steps:")
        print("1. Run integration tests: cargo test test_complete_encoding_pipeline")
        print("2. Verify Rust implementation matches Shine reference data")
        print("3. All tests should pass with identical hashes")
    
    return success_count > 0

if __name__ == "__main__":
    if main():
        sys.exit(0)
    else:
        sys.exit(1)