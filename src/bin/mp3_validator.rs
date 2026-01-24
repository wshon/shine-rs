//! MP3 Format Validator
//!
//! A tool to validate MP3 file format step by step, stopping at the first error encountered.
//! This validator checks the MP3 file structure according to the MPEG-1 Audio Layer III specification.

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// MP3 validation errors
#[derive(Debug)]
#[allow(dead_code)]
enum ValidationError {
    IoError(std::io::Error),
    InvalidFileSize(usize),
    InvalidSyncWord { position: usize, found: u16 },
    InvalidMpegVersion { position: usize, bits: u8 },
    InvalidLayer { position: usize, bits: u8 },
    InvalidBitrate { position: usize, index: u8 },
    InvalidSampleRate { position: usize, index: u8 },
    InvalidChannelMode { position: usize, mode: u8 },
    InvalidFrameSize { position: usize, calculated: usize, expected: usize },
    UnexpectedEndOfFile { position: usize },
    InvalidSideInfoLength { position: usize, expected: usize, available: usize },
    InvalidMainDataLength { position: usize, expected: usize, available: usize },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::IoError(e) => write!(f, "IO Error: {}", e),
            ValidationError::InvalidFileSize(size) => write!(f, "Invalid file size: {} bytes (too small for MP3)", size),
            ValidationError::InvalidSyncWord { position, found } => {
                write!(f, "Invalid sync word at position {}: found 0x{:04X}, expected 0xFFE0-0xFFFF", position, found)
            },
            ValidationError::InvalidMpegVersion { position, bits } => {
                write!(f, "Invalid MPEG version at position {}: bits {:02b}, expected 11 (MPEG-1)", position, bits)
            },
            ValidationError::InvalidLayer { position, bits } => {
                write!(f, "Invalid layer at position {}: bits {:02b}, expected 01 (Layer III)", position, bits)
            },
            ValidationError::InvalidBitrate { position, index } => {
                write!(f, "Invalid bitrate index at position {}: {}, expected 1-14", position, index)
            },
            ValidationError::InvalidSampleRate { position, index } => {
                write!(f, "Invalid sample rate index at position {}: {}, expected 0-2", position, index)
            },
            ValidationError::InvalidChannelMode { position, mode } => {
                write!(f, "Invalid channel mode at position {}: {}, expected 0-3", position, mode)
            },
            ValidationError::InvalidFrameSize { position, calculated, expected } => {
                write!(f, "Invalid frame size at position {}: calculated {} bytes, expected {} bytes", position, calculated, expected)
            },
            ValidationError::UnexpectedEndOfFile { position } => {
                write!(f, "Unexpected end of file at position {}", position)
            },
            ValidationError::InvalidSideInfoLength { position, expected, available } => {
                write!(f, "Invalid side info length at position {}: expected {} bytes, only {} available", position, expected, available)
            },
            ValidationError::InvalidMainDataLength { position, expected, available } => {
                write!(f, "Invalid main data length at position {}: expected {} bytes, only {} available", position, expected, available)
            },
        }
    }
}

/// MP3 Frame Header structure
#[derive(Debug)]
#[allow(dead_code)]
struct FrameHeader {
    sync_word: u16,
    mpeg_version: u8,
    layer: u8,
    protection_bit: bool,
    bitrate_index: u8,
    sample_rate_index: u8,
    padding_bit: bool,
    private_bit: bool,
    channel_mode: u8,
    mode_extension: u8,
    copyright: bool,
    original: bool,
    emphasis: u8,
}

/// MP3 Validator
struct Mp3Validator {
    file: File,
    position: usize,
    frame_count: usize,
    verbose: bool,
}

impl Mp3Validator {
    /// Create a new MP3 validator
    fn new(file_path: &Path, verbose: bool) -> Result<Self, ValidationError> {
        let file = File::open(file_path).map_err(ValidationError::IoError)?;
        Ok(Self {
            file,
            position: 0,
            frame_count: 0,
            verbose,
        })
    }

    /// Validate the entire MP3 file
    fn validate(&mut self) -> Result<(), ValidationError> {
        println!("🔍 开始验证 MP3 文件格式...");
        
        // Step 1: Check file size
        print!("📏 检查文件大小... ");
        self.check_file_size()?;
        println!("✅");
        
        // Step 2: Skip ID3v2 tag if present
        print!("🏷️  检查 ID3 标签... ");
        self.skip_id3v2_tag()?;
        println!("✅");
        
        // Step 3: Validate frames
        print!("🎵 验证 MP3 帧... ");
        while !self.is_end_of_file()? {
            self.validate_frame()?;
            if !self.verbose && self.frame_count % 10 == 0 {
                print!("{} ", self.frame_count);
            }
        }
        println!("✅");
        
        println!("✅ MP3 文件验证成功！共验证了 {} 个帧", self.frame_count);
        Ok(())
    }

    /// Check if file size is reasonable for MP3
    fn check_file_size(&mut self) -> Result<(), ValidationError> {
        let metadata = self.file.metadata().map_err(ValidationError::IoError)?;
        let file_size = metadata.len() as usize;
        
        if self.verbose {
            println!("文件大小: {} 字节", file_size);
        }
        
        if file_size < 4 {
            return Err(ValidationError::InvalidFileSize(file_size));
        }
        
        Ok(())
    }

    /// Skip ID3v2 tag if present
    fn skip_id3v2_tag(&mut self) -> Result<(), ValidationError> {
        let mut buffer = [0u8; 10];
        self.file.read_exact(&mut buffer).map_err(ValidationError::IoError)?;
        
        if &buffer[0..3] == b"ID3" {
            // ID3v2 tag present, calculate size and skip
            let size = ((buffer[6] as u32 & 0x7F) << 21) |
                      ((buffer[7] as u32 & 0x7F) << 14) |
                      ((buffer[8] as u32 & 0x7F) << 7) |
                      (buffer[9] as u32 & 0x7F);
            
            if self.verbose {
                println!("发现 ID3v2 标签，大小: {} 字节", size);
            }
            
            self.file.seek(SeekFrom::Start(10 + size as u64)).map_err(ValidationError::IoError)?;
            self.position = (10 + size) as usize;
        } else {
            // No ID3v2 tag, reset to beginning
            self.file.seek(SeekFrom::Start(0)).map_err(ValidationError::IoError)?;
            self.position = 0;
        }
        
        Ok(())
    }

    /// Check if we've reached the end of file
    fn is_end_of_file(&mut self) -> Result<bool, ValidationError> {
        let current_pos = self.file.stream_position().map_err(ValidationError::IoError)?;
        let file_size = self.file.metadata().map_err(ValidationError::IoError)?.len();
        Ok(current_pos >= file_size)
    }

    /// Validate a single MP3 frame
    fn validate_frame(&mut self) -> Result<(), ValidationError> {
        self.frame_count += 1;
        
        if self.verbose {
            println!("\n🎵 验证第 {} 个帧 (位置: {} / 0x{:X})", self.frame_count, self.position, self.position);
        }
        
        // Step 1: Parse frame header
        let header = self.parse_frame_header()?;
        
        // Step 2: Validate header fields
        self.validate_header_fields(&header)?;
        
        // Step 3: Calculate frame size
        let frame_size = self.calculate_frame_size(&header)?;
        
        // Step 4: Validate side information
        self.validate_side_info(&header, frame_size)?;
        
        // Step 5: Skip to next frame
        self.skip_to_next_frame(frame_size)?;
        
        if self.verbose {
            println!("✅ 第 {} 个帧验证通过", self.frame_count);
        }
        Ok(())
    }

    /// Parse MP3 frame header (4 bytes)
    fn parse_frame_header(&mut self) -> Result<FrameHeader, ValidationError> {
        let mut header_bytes = [0u8; 4];
        self.file.read_exact(&mut header_bytes).map_err(ValidationError::IoError)?;
        
        let header_u32 = u32::from_be_bytes(header_bytes);
        
        let header = FrameHeader {
            sync_word: ((header_u32 >> 20) & 0xFFF) as u16,
            mpeg_version: ((header_u32 >> 19) & 0x1) as u8,
            layer: ((header_u32 >> 17) & 0x3) as u8,
            protection_bit: ((header_u32 >> 16) & 0x1) != 0,
            bitrate_index: ((header_u32 >> 12) & 0xF) as u8,
            sample_rate_index: ((header_u32 >> 10) & 0x3) as u8,
            padding_bit: ((header_u32 >> 9) & 0x1) != 0,
            private_bit: ((header_u32 >> 8) & 0x1) != 0,
            channel_mode: ((header_u32 >> 6) & 0x3) as u8,
            mode_extension: ((header_u32 >> 4) & 0x3) as u8,
            copyright: ((header_u32 >> 3) & 0x1) != 0,
            original: ((header_u32 >> 2) & 0x1) != 0,
            emphasis: (header_u32 & 0x3) as u8,
        };
        
        if self.verbose {
            println!("📋 帧头解析: sync=0x{:03X}, version={}, layer={}, bitrate_idx={}, sample_rate_idx={}, mode={}", 
                    header.sync_word, header.mpeg_version, header.layer, 
                    header.bitrate_index, header.sample_rate_index, header.channel_mode);
        }
        
        Ok(header)
    }

    /// Validate frame header fields
    fn validate_header_fields(&self, header: &FrameHeader) -> Result<(), ValidationError> {
        // Check sync word (must be 0xFFE or higher)
        if header.sync_word < 0xFFE {
            return Err(ValidationError::InvalidSyncWord {
                position: self.position,
                found: header.sync_word,
            });
        }

        // Check MPEG version (1 = MPEG-1)
        if header.mpeg_version != 1 {
            return Err(ValidationError::InvalidMpegVersion {
                position: self.position,
                bits: header.mpeg_version,
            });
        }

        // Check layer (1 = Layer III)
        if header.layer != 1 {
            return Err(ValidationError::InvalidLayer {
                position: self.position,
                bits: header.layer,
            });
        }

        // Check bitrate index (1-14 are valid, 0 and 15 are invalid)
        if header.bitrate_index == 0 || header.bitrate_index == 15 {
            return Err(ValidationError::InvalidBitrate {
                position: self.position,
                index: header.bitrate_index,
            });
        }

        // Check sample rate index (0-2 are valid for MPEG-1)
        if header.sample_rate_index > 2 {
            return Err(ValidationError::InvalidSampleRate {
                position: self.position,
                index: header.sample_rate_index,
            });
        }

        // Check channel mode (0-3 are valid)
        if header.channel_mode > 3 {
            return Err(ValidationError::InvalidChannelMode {
                position: self.position,
                mode: header.channel_mode,
            });
        }

        if self.verbose {
            println!("✅ 帧头字段验证通过");
        }
        Ok(())
    }

    /// Calculate frame size based on header
    fn calculate_frame_size(&self, header: &FrameHeader) -> Result<usize, ValidationError> {
        // MPEG-1 Layer III bitrates (kbps)
        const BITRATES: [u32; 16] = [
            0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0
        ];

        // MPEG-1 sample rates (Hz)
        const SAMPLE_RATES: [u32; 4] = [44100, 48000, 32000, 0];

        let bitrate = BITRATES[header.bitrate_index as usize] * 1000; // Convert to bps
        let sample_rate = SAMPLE_RATES[header.sample_rate_index as usize];
        
        // Frame size calculation for MPEG-1 Layer III (using floating point for precision)
        // frame_size = floor(144 * bitrate / sample_rate) + padding
        // This matches shine's calculation: avg_slots_per_frame with bits_per_slot = 8
        let frame_size_float = 144.0 * (bitrate as f64) / (sample_rate as f64);
        let frame_size = frame_size_float as usize + if header.padding_bit { 1 } else { 0 };
        
        if self.verbose {
            println!("📐 计算帧大小: {} 字节 (bitrate={}kbps, sample_rate={}Hz, padding={}, 精确值={:.2})", 
                    frame_size, bitrate/1000, sample_rate, header.padding_bit, frame_size_float);
        }
        
        Ok(frame_size)
    }

    /// Validate side information
    fn validate_side_info(&mut self, header: &FrameHeader, frame_size: usize) -> Result<(), ValidationError> {
        // Calculate side info size based on channel mode
        let side_info_size = match header.channel_mode {
            3 => 17, // Mono: 17 bytes
            _ => 32, // Stereo/Joint Stereo/Dual Channel: 32 bytes
        };

        if self.verbose {
            println!("📊 验证侧信息: {} 字节", side_info_size);
        }

        // Check if we have enough bytes for side info
        let remaining_frame_size = frame_size - 4; // Subtract header size
        if remaining_frame_size < side_info_size {
            return Err(ValidationError::InvalidSideInfoLength {
                position: self.position + 4,
                expected: side_info_size,
                available: remaining_frame_size,
            });
        }

        // Read and validate side info
        let mut side_info_bytes = vec![0u8; side_info_size];
        self.file.read_exact(&mut side_info_bytes).map_err(ValidationError::IoError)?;

        // Basic side info validation
        self.validate_side_info_content(&side_info_bytes, header)?;

        if self.verbose {
            println!("✅ 侧信息验证通过");
        }
        Ok(())
    }

    /// Validate side information content
    fn validate_side_info_content(&self, side_info: &[u8], header: &FrameHeader) -> Result<(), ValidationError> {
        // Parse main_data_begin (9 bits)
        let main_data_begin = ((side_info[0] as u16) << 1) | ((side_info[1] as u16) >> 7);
        
        if self.verbose {
            println!("🔍 主数据开始位置: {}", main_data_begin);
        }

        // For mono
        if header.channel_mode == 3 {
            // Parse granule info for mono
            self.validate_granule_info_mono(&side_info[2..])?;
        } else {
            // Parse granule info for stereo
            self.validate_granule_info_stereo(&side_info[4..])?;
        }

        Ok(())
    }

    /// Validate granule info for mono
    fn validate_granule_info_mono(&self, granule_data: &[u8]) -> Result<(), ValidationError> {
        // Each granule info is 59 bits for mono, we have 2 granules
        // This is a simplified validation - in a full implementation,
        // we would parse all fields and validate their ranges
        
        if granule_data.len() < 15 { // Minimum bytes needed for 2 granules in mono
            return Err(ValidationError::InvalidSideInfoLength {
                position: self.position,
                expected: 15,
                available: granule_data.len(),
            });
        }

        if self.verbose {
            println!("✅ 单声道颗粒信息验证通过");
        }
        Ok(())
    }

    /// Validate granule info for stereo
    fn validate_granule_info_stereo(&self, granule_data: &[u8]) -> Result<(), ValidationError> {
        // Each granule info is 59 bits per channel, we have 2 granules and 2 channels
        // This is a simplified validation
        
        if granule_data.len() < 28 { // Minimum bytes needed for 2 granules in stereo
            return Err(ValidationError::InvalidSideInfoLength {
                position: self.position,
                expected: 28,
                available: granule_data.len(),
            });
        }

        if self.verbose {
            println!("✅ 立体声颗粒信息验证通过");
        }
        Ok(())
    }

    /// Skip to next frame
    fn skip_to_next_frame(&mut self, frame_size: usize) -> Result<(), ValidationError> {
        // We've already read the header (4 bytes) and side info
        // Calculate remaining bytes to skip
        let current_pos = self.file.stream_position().map_err(ValidationError::IoError)? as usize;
        let frame_start = self.position;
        let bytes_read = current_pos - frame_start;
        let bytes_to_skip = frame_size.saturating_sub(bytes_read);

        if bytes_to_skip > 0 {
            self.file.seek(SeekFrom::Current(bytes_to_skip as i64)).map_err(ValidationError::IoError)?;
        }

        self.position = frame_start + frame_size;
        
        if self.verbose {
            println!("⏭️  跳转到下一帧 (位置: {} / 0x{:X})", self.position, self.position);
        }
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args.len() > 3 {
        eprintln!("用法: {} <mp3文件路径> [--verbose]", args[0]);
        eprintln!("示例: {} tests/output/encoded_output.mp3", args[0]);
        eprintln!("      {} tests/output/encoded_output.mp3 --verbose", args[0]);
        std::process::exit(1);
    }

    let file_path = Path::new(&args[1]);
    let verbose = args.len() == 3 && args[2] == "--verbose";
    
    if !file_path.exists() {
        eprintln!("❌ 错误: 文件不存在: {}", file_path.display());
        std::process::exit(1);
    }

    println!("🎵 MP3 格式验证工具");
    println!("📁 验证文件: {}", file_path.display());
    if verbose {
        println!("{}", "=".repeat(50));
    }

    match Mp3Validator::new(file_path, verbose) {
        Ok(mut validator) => {
            if let Err(error) = validator.validate() {
                println!("\n❌ 验证失败:");
                println!("   {}", error);
                std::process::exit(1);
            }
        }
        Err(error) => {
            println!("❌ 无法打开文件: {}", error);
            std::process::exit(1);
        }
    }

    println!("\n🎉 所有检查都通过了！MP3 文件格式正确。");
}