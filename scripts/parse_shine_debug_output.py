#!/usr/bin/env python3
"""
Parse Shine debug output to generate real test data

This script parses the debug output from the modified Shine encoder
to extract real MDCT coefficients, quantization parameters, and bitstream data.
"""

import os
import sys
import json
import re
import hashlib
from datetime import datetime
from pathlib import Path

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
        import wave
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

def parse_shine_debug_output(debug_file):
    """Parse Shine debug output to extract frame data"""
    
    if not os.path.exists(debug_file):
        print(f"Debug file not found: {debug_file}")
        return []
    
    frames = {}
    current_frame = None
    
    with open(debug_file, 'r', encoding='utf-8') as f:
        for line in f:
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
                                'coefficients': [],
                                'l3_sb_sample': []
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
                    # Extract MDCT coefficients k17
                    coeff_match = re.search(r'k 17: (-?\d+)', line)
                    if coeff_match:
                        k17 = int(coeff_match.group(1))
                        if len(current_frame['mdct_coefficients']['coefficients']) == 0:
                            current_frame['mdct_coefficients']['coefficients'] = [k17, 0, 0]
                        else:
                            current_frame['mdct_coefficients']['coefficients'][0] = k17
                
                elif "MDCT coeff band 0 k 16:" in line:
                    # Extract MDCT coefficients k16
                    coeff_match = re.search(r'k 16: (-?\d+)', line)
                    if coeff_match:
                        k16 = int(coeff_match.group(1))
                        if len(current_frame['mdct_coefficients']['coefficients']) == 0:
                            current_frame['mdct_coefficients']['coefficients'] = [0, k16, 0]
                        else:
                            current_frame['mdct_coefficients']['coefficients'][1] = k16
                
                elif "MDCT coeff band 0 k 15:" in line:
                    # Extract MDCT coefficients k15
                    coeff_match = re.search(r'k 15: (-?\d+)', line)
                    if coeff_match:
                        k15 = int(coeff_match.group(1))
                        if len(current_frame['mdct_coefficients']['coefficients']) == 0:
                            current_frame['mdct_coefficients']['coefficients'] = [0, 0, k15]
                        else:
                            current_frame['mdct_coefficients']['coefficients'][2] = k15
                
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

def main():
    """Main function to parse Shine debug data and generate test files"""
    
    print("Parsing Shine Debug Output")
    print("=" * 30)
    
    # Test cases to process
    test_cases = [
        {
            "debug_file": "ref/shine/shine_debug_sample-3s.txt",
            "audio_file": "tests/audio/sample-3s.wav",
            "mp3_file": "ref/shine/shine_sample-3s_3frames_debug.mp3",
            "bitrate": 128,
            "name": "sample-3s_128k_3f_real"
        },
        {
            "debug_file": "ref/shine/shine_debug_voice_recorder.txt",
            "audio_file": "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav",
            "mp3_file": "ref/shine/shine_voice_recorder_3frames_debug.mp3",
            "bitrate": 128,
            "name": "voice_recorder_128k_3f_real"
        }
    ]
    
    output_dir = Path("tests/pipeline_data")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    for test_case in test_cases:
        print(f"\\nProcessing: {test_case['name']}")
        
        # Check if files exist
        if not os.path.exists(test_case["debug_file"]):
            print(f"Debug file not found: {test_case['debug_file']}")
            continue
        
        if not os.path.exists(test_case["audio_file"]):
            print(f"Audio file not found: {test_case['audio_file']}")
            continue
        
        # Read WAV metadata
        wav_metadata = read_wav_metadata(test_case["audio_file"])
        if not wav_metadata:
            print(f"Could not read WAV metadata for {test_case['audio_file']}")
            continue
        
        # Parse debug data
        frames = parse_shine_debug_output(test_case["debug_file"])
        
        if not frames:
            print(f"No debug data found in {test_case['debug_file']}")
            continue
        
        # Calculate MP3 file hash and size
        mp3_size = 0
        mp3_hash = ""
        if os.path.exists(test_case["mp3_file"]):
            mp3_size = os.path.getsize(test_case["mp3_file"])
            mp3_hash = calculate_sha256(test_case["mp3_file"])
        
        # Determine stereo mode
        stereo_mode = 3 if wav_metadata["channels"] == 1 else 0  # 3=mono, 0=stereo
        
        # Create test data structure with real data
        test_data = {
            "metadata": {
                "name": f"test_case_{test_case['name']}_{wav_metadata['sample_rate']}hz_{wav_metadata['channels']}ch_{test_case['bitrate']}kbps",
                "input_file": test_case["audio_file"],
                "expected_output_size": mp3_size,
                "expected_hash": mp3_hash,
                "created_at": datetime.utcnow().isoformat() + "Z",
                "description": f"Real Shine debug data for {test_case['name']}",
                "generated_by": "Shine reference implementation with debug output"
            },
            "config": {
                "sample_rate": wav_metadata["sample_rate"],
                "channels": wav_metadata["channels"],
                "bitrate": test_case["bitrate"],
                "stereo_mode": stereo_mode,
                "mpeg_version": 3
            },
            "frames": frames[:3]  # Limit to first 3 frames
        }
        
        # Save test data
        json_file = output_dir / f"{test_case['name']}.json"
        with open(json_file, 'w', encoding='utf-8') as f:
            json.dump(test_data, f, indent=2, ensure_ascii=False)
        
        print(f"✓ Generated real test data: {json_file}")
        print(f"  Frames extracted: {len(frames)}")
        print(f"  MP3 size: {mp3_size} bytes")
        print(f"  MP3 hash: {mp3_hash[:16]}...")
        
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
    
    print(f"\\n" + "=" * 30)
    print("Real debug data generation complete!")
    
    return True

if __name__ == "__main__":
    if main():
        sys.exit(0)
    else:
        sys.exit(1)