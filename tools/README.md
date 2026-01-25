# MP3 Encoder Tools

This directory contains various command-line tools for the rust-mp3-encoder project.

## Available Tools

### wav2mp3
WAV to MP3 converter using the rust-mp3-encoder library.

```bash
cd wav2mp3
cargo run -- input.wav output.mp3
```

### collect_test_data
Tool for collecting test data during the encoding process.

```bash
cd collect_test_data
cargo run -- input.wav
```

### validate_test_data
Tool for validating collected test data.

```bash
cd validate_test_data
cargo run -- test_data.json
```

### mp3_validator
Tool for validating MP3 file structure and content.

```bash
cd mp3_validator
cargo run -- file.mp3
```

### mp3_hexdump
Tool for displaying MP3 file content in hexadecimal format.

```bash
cd mp3_hexdump
cargo run -- file.mp3
```

## Building All Tools

To build all tools at once, run from this directory:

```bash
cargo build --workspace
```

## Running Tests

To run tests for all tools:

```bash
cargo test --workspace
```