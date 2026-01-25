#!/usr/bin/env python3
"""
Create a simple test WAV file for debugging MP3 encoder
"""

import struct
import math

def create_test_wav(filename, duration=0.1, sample_rate=44100, channels=2):
    """Create a simple test WAV file with sine wave"""
    
    # Calculate number of samples
    num_samples = int(duration * sample_rate)
    
    # Generate sine wave data
    samples = []
    for i in range(num_samples):
        # Generate a 440Hz sine wave
        t = i / sample_rate
        sample_value = int(16000 * math.sin(2 * math.pi * 440 * t))
        
        if channels == 1:
            samples.append(sample_value)
        else:
            # Stereo: same signal on both channels
            samples.append(sample_value)  # Left channel
            samples.append(sample_value)  # Right channel
    
    # WAV file header
    with open(filename, 'wb') as f:
        # RIFF header
        f.write(b'RIFF')
        
        # File size (will be updated later)
        file_size_pos = f.tell()
        f.write(struct.pack('<I', 0))  # Placeholder
        
        f.write(b'WAVE')
        
        # fmt chunk
        f.write(b'fmt ')
        f.write(struct.pack('<I', 16))  # fmt chunk size
        f.write(struct.pack('<H', 1))   # PCM format
        f.write(struct.pack('<H', channels))  # Number of channels
        f.write(struct.pack('<I', sample_rate))  # Sample rate
        f.write(struct.pack('<I', sample_rate * channels * 2))  # Byte rate
        f.write(struct.pack('<H', channels * 2))  # Block align
        f.write(struct.pack('<H', 16))  # Bits per sample
        
        # data chunk
        f.write(b'data')
        data_size = len(samples) * 2  # 16-bit samples
        f.write(struct.pack('<I', data_size))
        
        # Write sample data
        for sample in samples:
            f.write(struct.pack('<h', sample))
        
        # Update file size in header
        file_size = f.tell() - 8
        f.seek(file_size_pos)
        f.write(struct.pack('<I', file_size))

if __name__ == '__main__':
    # Create a simple test WAV file
    create_test_wav('test_input.wav', duration=0.1, sample_rate=44100, channels=2)
    print("Created test_input.wav (0.1 seconds, 44.1kHz, stereo)")
    
    # Also create a mono version
    create_test_wav('test_input_mono.wav', duration=0.1, sample_rate=44100, channels=1)
    print("Created test_input_mono.wav (0.1 seconds, 44.1kHz, mono)")