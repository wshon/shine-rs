#!/usr/bin/env python3
"""
Generate a test WAV file with exactly 3 MP3 frames worth of audio data.
Each MP3 frame contains 1152 samples, so we need 3456 samples total.
"""

import wave
import struct
import math

def generate_test_wav(filename, sample_rate=44100, channels=2, duration_frames=3):
    """
    Generate a test WAV file with specified number of MP3 frames.
    
    Args:
        filename: Output WAV filename
        sample_rate: Sample rate in Hz
        channels: Number of channels (1=mono, 2=stereo)
        duration_frames: Number of MP3 frames to generate
    """
    samples_per_frame = 1152  # MP3 Layer III frame size
    total_samples = samples_per_frame * duration_frames
    
    print(f"Generating {filename}:")
    print(f"  Sample rate: {sample_rate} Hz")
    print(f"  Channels: {channels}")
    print(f"  MP3 frames: {duration_frames}")
    print(f"  Samples per frame: {samples_per_frame}")
    print(f"  Total samples: {total_samples}")
    print(f"  Duration: {total_samples / sample_rate:.3f} seconds")
    
    with wave.open(filename, 'w') as wav_file:
        wav_file.setnchannels(channels)
        wav_file.setsampwidth(2)  # 16-bit
        wav_file.setframerate(sample_rate)
        
        # Generate audio data - audible sine waves with proper amplitude
        audio_data = []
        
        # Use higher amplitude for audible sound (about 50% of max 16-bit range)
        amplitude = 16384  # Was too quiet, now using full range
        
        for i in range(total_samples):
            # Time in seconds
            t = i / sample_rate
            
            if channels == 1:
                # Mono: single sine wave at 440 Hz (A4 note)
                sample = int(amplitude * math.sin(2 * math.pi * 440 * t))
                # Clamp to 16-bit range
                sample = max(-32768, min(32767, sample))
                audio_data.append(struct.pack('<h', sample))
            else:
                # Stereo: left channel 440 Hz (A4), right channel 554 Hz (C#5)
                # Using musical intervals for more pleasant sound
                left_sample = int(amplitude * 0.7 * math.sin(2 * math.pi * 440 * t))
                right_sample = int(amplitude * 0.7 * math.sin(2 * math.pi * 554.37 * t))
                
                # Clamp to 16-bit range
                left_sample = max(-32768, min(32767, left_sample))
                right_sample = max(-32768, min(32767, right_sample))
                
                audio_data.append(struct.pack('<hh', left_sample, right_sample))
        
        # Write all audio data
        wav_file.writeframes(b''.join(audio_data))
    
    print(f"Generated {filename} successfully!")

if __name__ == "__main__":
    # Generate test files for different frame counts
    frame_counts = [1, 2, 3, 6, 10, 15, 20]
    
    print("Generating test WAV files for frame limit replacement...")
    print("=" * 60)
    
    for frames in frame_counts:
        # Generate stereo versions
        generate_test_wav(f"tests/audio/test_{frames}frames_stereo.wav", 
                         channels=2, duration_frames=frames)
        
        # Generate mono versions  
        generate_test_wav(f"tests/audio/test_{frames}frames_mono.wav", 
                         channels=1, duration_frames=frames)
        
        print()  # Add spacing between different frame counts
    
    print("All test WAV files generated successfully!")
    print("These files can be used to replace frame limit functionality in tests.")