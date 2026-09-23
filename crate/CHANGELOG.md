# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-09-23

### Fixed

- Clamp region addresses (address1, address2) to `big_values` region boundary, preventing Huffman bit count from exceeding `part2_3_length` and causing bitstream overrun on the last granule.
- Reset region addresses (address1, address2, address3) when `big_values == 0`, preventing stale addresses from a prior quantization pass from corrupting the bitstream.

Both bugs exist in the original C reference implementation (`toots/shine` `l3loop.c:subdivide`).

## [0.1.4] - 2026-09-14

### Changed

- CI/PR title check downgraded from blocking to warning.

## [0.1.3] - 2026-09-13

### Changed

- Reorganized project structure and consolidated testing infrastructure.

## [0.1.2] - 2026-09-09

### Changed

- Updated shine reference implementation.

## [0.1.1] - 2026-09-08

### Changed

- Minor version bump.

## [0.1.0] - 2026-09-08

### Added

- Initial release: pure Rust MP3 encoder based on the Shine library, providing complete MPEG Layer III encoding functionality.

[0.1.5]: https://github.com/wshon/shine-rs/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/wshon/shine-rs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/wshon/shine-rs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wshon/shine-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wshon/shine-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wshon/shine-rs/releases/tag/v0.1.0
