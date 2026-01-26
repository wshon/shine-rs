# Scripts Directory

This directory contains Python scripts for managing reference files, validation, and performance testing for the MP3 encoder project.

## 🛠️ Available Scripts

### 1. Reference File Generation (`generate_reference_files.py`)

Generates reference MP3 files using the Shine encoder for testing purposes.

**Usage:**
```bash
# Generate all reference files
python scripts/generate_reference_files.py

# Generate specific configurations
python scripts/generate_reference_files.py --configs 3frames 6frames

# Don't update test constants automatically
python scripts/generate_reference_files.py --no-update-tests

# Specify workspace directory
python scripts/generate_reference_files.py --workspace /path/to/project
```

**Features:**
- ✅ 11 different configurations (1-20 frames)
- ✅ Multiple audio formats support
- ✅ Automatic validation and hash calculation
- ✅ Test constant updates
- ✅ Cross-platform compatibility

### 2. Reference File Validation (`validate_reference_files.py`)

Validates that the Rust encoder produces identical output to Shine reference files.

**Usage:**
```bash
# Validate all reference files
python scripts/validate_reference_files.py

# Validate specific configurations
python scripts/validate_reference_files.py --configs 3frames 6frames voice_3frames

# Specify workspace directory
python scripts/validate_reference_files.py --workspace /path/to/project
```

**Features:**
- ✅ Comprehensive validation across all configurations
- ✅ SHA256 hash verification
- ✅ File size checking
- ✅ Detailed error reporting
- ✅ Success rate statistics

### 3. Performance Benchmark (`benchmark_encoders.py`)

Benchmarks the performance of Rust and Shine encoders.

**Usage:**
```bash
# Benchmark all configurations
python scripts/benchmark_encoders.py

# Benchmark specific configurations with multiple iterations
python scripts/benchmark_encoders.py --configs 3frames 6frames --iterations 5

# Save detailed report to JSON
python scripts/benchmark_encoders.py --output benchmark_report.json

# Specify workspace directory
python scripts/benchmark_encoders.py --workspace /path/to/project
```

**Features:**
- ✅ Performance comparison between Rust and Shine
- ✅ Multiple iteration support for accuracy
- ✅ Frames per second calculation
- ✅ Statistical analysis
- ✅ JSON report generation

### 4. Voice Issue Diagnosis (`diagnose_voice_issue.py`)

Diagnoses encoding differences for voice/mono audio files.

**Usage:**
```bash
# Diagnose voice file encoding issues
python scripts/diagnose_voice_issue.py
```

**Features:**
- ✅ Audio format analysis
- ✅ Encoder output comparison
- ✅ MP3 header parsing
- ✅ Detailed debugging information

### 5. Encoding Differences Analysis (`analyze_encoding_differences.py`)

Analyzes differences between encoders for various audio formats.

**Usage:**
```bash
# Analyze encoding differences
python scripts/analyze_encoding_differences.py
```

**Features:**
- ✅ Multi-format audio analysis
- ✅ Header comparison
- ✅ Detailed difference reporting
- ✅ Cross-platform compatibility

## 📊 Current Test Status

### ✅ Passing Configurations (9/11 - 82% success rate)

| Configuration | Frames | Input File | Size | Status |
|---------------|--------|------------|------|--------|
| 1frame | 1 | sample-3s.wav | 416 bytes | ✅ |
| 2frames | 2 | sample-3s.wav | 836 bytes | ✅ |
| 3frames | 3 | sample-3s.wav | 1252 bytes | ✅ |
| 6frames | 6 | sample-3s.wav | 2508 bytes | ✅ |
| 10frames | 10 | sample-3s.wav | 4180 bytes | ✅ |
| 15frames | 15 | sample-3s.wav | 6268 bytes | ✅ |
| 20frames | 20 | sample-3s.wav | 8360 bytes | ✅ |
| large_3frames | 3 | Free_Test_Data_500KB_WAV.wav | 1252 bytes | ✅ |
| large_6frames | 6 | Free_Test_Data_500KB_WAV.wav | 2508 bytes | ✅ |

### ⚠️ Known Issues (2/11)

| Configuration | Issue | Cause |
|---------------|-------|-------|
| voice_3frames | Hash mismatch | Mono 48kHz processing differences |
| voice_6frames | Hash mismatch | Mono 48kHz processing differences |

## 🔧 Environment Variables

Both Rust and Shine encoders support frame limiting through environment variables:

**Rust Encoder:**
```bash
RUST_MP3_MAX_FRAMES=6 cargo run -- input.wav output.mp3
```

**Shine Encoder:**
```bash
SHINE_MAX_FRAMES=6 ./ref/shine/shineenc input.wav output.mp3
```

## 📁 Generated Files

The scripts generate and manage the following files:

```
tests/audio/
├── reference_manifest.json          # Reference file metadata
├── shine_reference_*.mp3           # Generated reference files
└── README.md                       # Audio files documentation

tests/docs/
└── environment_variable_integration.md  # Environment variable docs
```

## 🚀 Quick Start

1. **Generate reference files:**
   ```bash
   python scripts/generate_reference_files.py
   ```

2. **Validate Rust implementation:**
   ```bash
   python scripts/validate_reference_files.py
   ```

3. **Run performance benchmark:**
   ```bash
   python scripts/benchmark_encoders.py --configs 3frames 6frames
   ```

## 🎯 Integration with Rust Tests

These Python scripts complement the Rust integration tests:

```bash
# Run Rust integration tests
cargo test --test integration_reference_validation

# Run Python validation
python scripts/validate_reference_files.py
```

Both should produce consistent results, with the Python scripts providing more detailed diagnostics.

## 📈 Success Metrics

- **File Size Match**: Rust output matches Shine output exactly
- **SHA256 Hash Match**: Byte-level identical files
- **Performance Comparison**: Objective speed measurements
- **Cross-Platform Consistency**: Same results on different systems

## 🛡️ Error Handling

All scripts include comprehensive error handling:

- **Missing files**: Clear error messages with suggested fixes
- **Encoding failures**: Detailed stdout/stderr capture
- **Hash mismatches**: Precise difference reporting
- **Timeout handling**: Graceful handling of long-running processes

## 📚 Related Documentation

- [Testing Guide](../docs/TESTING_GUIDE.md)
- [Reference Data Status](../docs/REFERENCE_DATA_STATUS.md)
- [Completion Summary](../REFERENCE_DATA_COMPLETION_SUMMARY.md)
- [Environment Variable Integration](../tests/docs/environment_variable_integration.md)

This script collection provides enterprise-grade testing infrastructure for the MP3 encoder project, ensuring high quality and reliability.