#!/usr/bin/env python3
"""
Extract debug data from modified Shine encoder

This script requires a modified version of Shine that outputs debug information
including MDCT coefficients, quantization parameters, and bitstream data.

Based on debug_analysis.md findings, we need to capture:
- MDCT coefficients and l3_sb_sample data
- Quantization parameters (xrmax, global_gain, part2_3_length, etc.)
- Bitstream parameters (padding, bits_per_frame, written, slot_lag)
"""

import os
import sys
import json
import subprocess
import hashlib
import re
from datetime import datetime
from pathlib import Path

def create_modified_shine_source():
    """
    Create instructions for modifying Shine source to output debug data
    """
    
    instructions = """
To extract real debug data from Shine, you need to modify the Shine source code:

1. In ref/shine/src/lib/layer3.c, add debug output after key calculations:

   // After MDCT calculation (around line 200)
   printf("[SHINE DEBUG Frame %d] MDCT coeff band 0 k 17: %d, k 16: %d, k 15: %d\\n", 
          frame_count, mdct_freq[0][17], mdct_freq[0][16], mdct_freq[0][15]);
   
   // After l3_sb_sample calculation
   printf("[SHINE DEBUG Frame %d] l3_sb_sample[0][1][0]: first value: %d\\n", 
          frame_count, l3_sb_sample[0][1][0]);

2. In ref/shine/src/lib/l3loop.c, add quantization debug output:

   // After xrmax calculation
   printf("[SHINE DEBUG Frame %d] xrmax: %d\\n", frame_count, xrmax);
   
   // After global_gain calculation
   printf("[SHINE DEBUG Frame %d] global_gain: %d\\n", frame_count, global_gain);
   
   // After part2_3_length calculation
   printf("[SHINE DEBUG Frame %d] part2_3_length: %d\\n", frame_count, part2_3_length);

3. In ref/shine/src/lib/bitstream.c, add bitstream debug output:

   // After padding calculation
   printf("[SHINE DEBUG Frame %d] padding: %d\\n", frame_count, padding);
   
   // After bits_per_frame calculation
   printf("[SHINE DEBUG Frame %d] bits_per_frame: %d\\n", frame_count, bits_per_frame);
   
   // After written bytes calculation
   printf("[SHINE DEBUG Frame %d] written: %d\\n", frame_count, written);
   
   // After slot_lag calculation
   printf("[SHINE DEBUG Frame %d] slot_lag: %f\\n", frame_count, slot_lag);

4. Recompile Shine:
   cd ref/shine
   make clean
   make

5. Run with debug output:
   ./shineenc.exe input.wav output.mp3 2>&1 | tee debug_output.txt

6. Parse the debug output with this script.
"""
    
    print(instructions)
    return False

def parse_shine_debug_output(debug_file, audio_file, bitrate):
    """
    Parse Shine debug output to extract frame data
    """
    
    if not os.path.exists(debug_file):
        print(f"Debug file not found: {debug_file}")
        print("You need to run modified Shine first to generate debug output.")
        return None
    
    frames = []
    current_frame = None
    
    with open(debug_file, 'r') as f:
        for line in f:
            line = line.strip()
            
            # Parse frame-specific debug output
            if "[SHINE DEBUG Frame" in line:
                # Extract frame number
                frame_match = re.search(r'Frame (\d+)', line)
                if frame_match:
                    frame_num = int(frame_match.group(1))
                    
                    # Initialize frame if new
                    if current_frame is None or current_frame['frame_number'] != frame_num:
                        if current_frame is not None:
                            frames.append(current_frame)
                        
                        current_frame = {
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
                
                # Parse specific debug values
                if "MDCT coeff" in line:
                    # Extract MDCT coefficients
                    coeff_match = re.search(r'k 17: (-?\d+), k 16: (-?\d+), k 15: (-?\d+)', line)
                    if coeff_match:
                        current_frame['mdct_coefficients']['coefficients'] = [
                            int(coeff_match.group(1)),
                            int(coeff_match.group(2)),
                            int(coeff_match.group(3))
                        ]
                
                elif "l3_sb_sample" in line:
                    # Extract l3_sb_sample
                    sample_match = re.search(r'first value: (-?\d+)', line)
                    if sample_match:
                        current_frame['mdct_coefficients']['l3_sb_sample'] = [
                            int(sample_match.group(1))
                        ]
                
                elif "xrmax:" in line:
                    # Extract xrmax
                    xrmax_match = re.search(r'xrmax: (-?\d+)', line)
                    if xrmax_match:
                        current_frame['quantization']['xrmax'] = int(xrmax_match.group(1))
                
                elif "global_gain:" in line:
                    # Extract global_gain
                    gain_match = re.search(r'global_gain: (-?\d+)', line)
                    if gain_match:
                        current_frame['quantization']['global_gain'] = int(gain_match.group(1))
                
                elif "part2_3_length:" in line:
                    # Extract part2_3_length
                    length_match = re.search(r'part2_3_length: (-?\d+)', line)
                    if length_match:
                        current_frame['quantization']['part2_3_length'] = int(length_match.group(1))
                
                elif "padding:" in line:
                    # Extract padding
                    padding_match = re.search(r'padding: (-?\d+)', line)
                    if padding_match:
                        current_frame['bitstream']['padding'] = int(padding_match.group(1))
                
                elif "bits_per_frame:" in line:
                    # Extract bits_per_frame
                    bits_match = re.search(r'bits_per_frame: (-?\d+)', line)
                    if bits_match:
                        current_frame['bitstream']['bits_per_frame'] = int(bits_match.group(1))
                
                elif "written:" in line:
                    # Extract written
                    written_match = re.search(r'written: (-?\d+)', line)
                    if written_match:
                        current_frame['bitstream']['written'] = int(written_match.group(1))
                
                elif "slot_lag:" in line:
                    # Extract slot_lag
                    lag_match = re.search(r'slot_lag: (-?\d+\.?\d*)', line)
                    if lag_match:
                        current_frame['bitstream']['slot_lag'] = float(lag_match.group(1))
    
    # Add the last frame
    if current_frame is not None:
        frames.append(current_frame)
    
    return frames

def main():
    """
    Main function to extract Shine debug data
    """
    
    print("Shine Debug Data Extractor")
    print("=" * 40)
    
    # Check if we have modified Shine
    shine_exe = "ref/shine/shineenc.exe"
    if not os.path.exists(shine_exe):
        print(f"Shine encoder not found at {shine_exe}")
        return False
    
    # Check for debug output file
    debug_file = "ref/shine/debug_output.txt"
    
    if not os.path.exists(debug_file):
        print("No debug output file found.")
        print("You need to modify Shine source code to output debug information.")
        create_modified_shine_source()
        return False
    
    # Parse debug output for each test case
    test_cases = [
        {
            "audio_file": "tests/audio/sample-3s.wav",
            "bitrate": 128,
            "name": "sample-3s_128k_3f_real"
        },
        {
            "audio_file": "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav", 
            "bitrate": 128,
            "name": "voice_recorder_128k_3f_real"
        }
    ]
    
    output_dir = Path("tests/pipeline_data")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    for test_case in test_cases:
        print(f"\\nProcessing: {test_case['name']}")
        
        if not os.path.exists(test_case["audio_file"]):
            print(f"Audio file not found: {test_case['audio_file']}")
            continue
        
        # Parse debug data
        frames = parse_shine_debug_output(debug_file, test_case["audio_file"], test_case["bitrate"])
        
        if frames and len(frames) > 0:
            # Create test data structure with real data
            test_data = {
                "metadata": {
                    "name": f"test_case_{test_case['name']}",
                    "input_file": test_case["audio_file"],
                    "expected_output_size": 0,  # Will be filled by actual encoding
                    "expected_hash": "",  # Will be filled by actual encoding
                    "created_at": datetime.utcnow().isoformat() + "Z",
                    "description": f"Real Shine debug data for {test_case['name']}",
                    "generated_by": "Shine reference implementation with debug output"
                },
                "config": {
                    "sample_rate": 44100,  # Default, should be read from WAV
                    "channels": 2,  # Default, should be read from WAV
                    "bitrate": test_case["bitrate"],
                    "stereo_mode": 0,  # Will be determined by channels
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
        else:
            print(f"✗ No debug data found for {test_case['name']}")
    
    print("\\n" + "=" * 40)
    print("To get real debug data:")
    print("1. Modify Shine source code as shown above")
    print("2. Recompile Shine")
    print("3. Run: ./shineenc.exe input.wav output.mp3 2>&1 | tee debug_output.txt")
    print("4. Run this script again")
    
    return True

if __name__ == "__main__":
    if main():
        sys.exit(0)
    else:
        sys.exit(1)