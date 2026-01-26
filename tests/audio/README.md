# Test Audio Files

This directory contains audio files and reference outputs used for testing the MP3 encoder.

## Files

### Input Files
- `sample-3s.wav` - 3-second stereo WAV file used for consistency testing
- `Free_Test_Data_500KB_WAV.wav` - Larger test file for comprehensive testing
- `voice-recorder-testing-1-2-3-sound-file.wav` - Voice recording test file

### Reference Files
- `shine_reference_6frames.mp3` - Reference output from Shine encoder (6 frames only)

## Reference File Generation

The reference file `shine_reference_6frames.mp3` was generated using the following process:

1. **Shine Version**: Liquidsoap version with debug modifications
2. **Command**: `./shineenc.exe sample-3s.wav shine_reference_6frames.mp3`
3. **Frame Limit**: 6 frames (hardcoded in Shine for debugging)
4. **Settings**: Default Shine settings (128kbps, 44.1kHz, stereo)

## Verification Data

### shine_reference_6frames.mp3
- **File Size**: 2508 bytes
- **SHA256**: `4385b617a86cb3891ce3c99dabe6b47c2ac9182b32c46cbc5ad167fb28b959c4`
- **Frame Count**: 6 frames
- **Generated**: 2026-01-26
- **Verified Against**: Shine reference implementation with SCFSI debugging enabled

## Test Reliability

The reference files are used to ensure:
1. **Reproducibility**: Tests don't depend on external tools at runtime
2. **Consistency**: Same reference data across different environments
3. **Stability**: No variation due to Shine version differences
4. **Verification**: Known-good outputs for algorithm validation

## Regenerating Reference Files

If reference files need to be regenerated:

1. Ensure Shine encoder is available in `ref/shine/shineenc.exe`
2. Run: `cd ref/shine && ./shineenc.exe ../../tests/audio/sample-3s.wav new_reference.mp3`
3. Verify the output matches expected characteristics
4. Update the SHA256 hash in the test constants
5. Copy to `tests/audio/` directory

## SCFSI Test Data

The 6-frame reference file contains the following SCFSI patterns (verified through debugging):

- **Frame 1**: SCFSI = [0,1,0,1] for both channels
- **Frame 2**: SCFSI = [1,1,1,1] for both channels  
- **Frame 3**: SCFSI = [0,1,1,1] for both channels
- **Frame 4**: SCFSI = [1,1,1,0] for both channels
- **Frame 5**: SCFSI = [1,1,1,1] for both channels
- **Frame 6**: SCFSI = [1,1,1,1] for both channels

These patterns validate the SCFSI calculation algorithm across different audio characteristics.