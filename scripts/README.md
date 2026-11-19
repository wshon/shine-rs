[中文文档](README_zh_CN.md)

# MP3 Encoder Reference Data Generation Script

## Overview

`generate_reference_data.py` is a complete reference data generation script for generating real Shine reference data for MP3 encoder integration testing.

## Features

The script automatically performs the following steps:

1. **Run Shine encoder** - Run Shine encoder with specified audio files and parameters
2. **Capture debug output** - Extract MDCT coefficients, quantization parameters, and bitstream data
3. **Parse debug data** - Extract key algorithm parameters from Shine's debug output
4. **Generate JSON test data** - Create JSON test files containing real reference values
5. **Calculate file hash** - Generate SHA256 hash of MP3 files for verification

## Usage

```bash
# Generate all reference data
python scripts/generate_reference_data.py
```

## Generated Data

The script generates the following files in the `testing/fixtures/data/` directory:

- `sample-3s_128k_3f_real.json` - 3-second sample, 128kbps, 3 frames
- `voice_recorder_128k_3f_real.json` - Voice recording, 128kbps, 3 frames  
- `free_test_data_128k_3f_real.json` - Free test data, 128kbps, 3 frames
- `sample-3s_192k_3f_real.json` - 3-second sample, 192kbps, 3 frames

## Data Structure

Each JSON file contains:

### Metadata
- `name`: Test case name
- `input_file`: Input audio file path
- `expected_output_size`: Expected MP3 file size
- `expected_hash`: Expected SHA256 hash value
- `created_at`: Creation time
- `description`: Description
- `generated_by`: Generator tool information

### Configuration
- `sample_rate`: Sample rate
- `channels`: Number of channels
- `bitrate`: Bitrate
- `stereo_mode`: Stereo mode (0=stereo, 3=mono)
- `mpeg_version`: MPEG version (3=MPEG-I)

### Frame Data
Each frame contains:

#### MDCT Coefficients
- `coefficients`: MDCT coefficients [k17, k16, k15]
- `l3_sb_sample`: Subband sample data

#### Quantization Parameters
- `xrmax`: Maximum spectral value
- `max_bits`: Maximum bits
- `part2_3_length`: Part2/3 length
- `quantizer_step_size`: Quantizer step size
- `global_gain`: Global gain

#### Bitstream Parameters
- `padding`: Padding bits
- `bits_per_frame`: Bits per frame
- `written`: Bytes written
- `slot_lag`: Slot lag

## Requirements

- Python 3.6+
- Shine encoder (`ref/shine/shineenc.exe`)
- Test audio files in `tests/audio/` directory

## Configuration

To add new test configurations, modify the `TEST_CONFIGS` list in the script:

```python
TEST_CONFIGS = [
    {
        "name": "my_test_128k_3f_real",
        "audio_file": "tests/audio/my_audio.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "My custom test case"
    }
]
```

## Verification

Generated reference data can be used for integration tests:

```bash
# Run integration tests to verify Rust implementation matches Shine
cargo test test_complete_encoding_pipeline
```

## Notes

1. **Shine debug output**: The script depends on Shine encoder's debug output; ensure you use a version with debug info
2. **Path handling**: The script automatically handles relative and absolute paths
3. **Frame limit**: Use environment variable `SHINE_MAX_FRAMES` to limit encoding frames
4. **Data precision**: All values are identical to Shine's output to ensure test accuracy

## Troubleshooting

### Common Issues

1. **"Shine encoder not found"**
   - Ensure `ref/shine/shineenc.exe` exists
   - Check if Shine is compiled correctly

2. **"Audio file not found"**
   - Ensure audio files exist at the specified path
   - Check if file paths are correct

3. **"No debug data extracted"**
   - Ensure Shine version includes debug output
   - Check if environment variable `SHINE_MAX_FRAMES` is set

### Debugging Tips

- View Shine's stdout and stderr
- Check if generated MP3 files exist
- Verify WAV file format is correct (16-bit PCM)
