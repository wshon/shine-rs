//! 高级MP3编码器接口
//!
//! 这个模块提供了一个简单易用的高级接口，封装了底层的shine编码器实现。
//! 它提供了Rust风格的API，同时保留了对底层低级接口的完全访问。

use crate::encoder::{
    ShineConfig, ShineWave, ShineMpeg, shine_initialise, shine_encode_buffer_interleaved,
    shine_flush, shine_set_config_mpeg_defaults, NONE
};
use crate::error::{EncoderError, ConfigError, InputDataError};
use crate::types::ShineGlobalConfig;
use std::collections::VecDeque;

/// 支持的采样率 (Hz)
pub const SUPPORTED_SAMPLE_RATES: &[u32] = &[
    8000, 11025, 12000,    // MPEG 2.5
    16000, 22050, 24000,   // MPEG 2
    32000, 44100, 48000,   // MPEG 1
];

/// 支持的比特率 (kbps)
pub const SUPPORTED_BITRATES: &[u32] = &[
    8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 192, 224, 256, 320
];

/// 立体声模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoMode {
    /// 立体声
    Stereo = 0,
    /// 联合立体声
    JointStereo = 1,
    /// 双声道
    DualChannel = 2,
    /// 单声道
    Mono = 3,
}

/// MP3编码器配置
#[derive(Debug, Clone)]
pub struct Mp3EncoderConfig {
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 比特率 (kbps)
    pub bitrate: u32,
    /// 声道数 (1 = 单声道, 2 = 立体声)
    pub channels: u8,
    /// 立体声模式
    pub stereo_mode: StereoMode,
    /// 版权标志
    pub copyright: bool,
    /// 原创标志
    pub original: bool,
}

impl Default for Mp3EncoderConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bitrate: 128,
            channels: 2,
            stereo_mode: StereoMode::Stereo,
            copyright: false,
            original: true,
        }
    }
}

impl Mp3EncoderConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置采样率
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// 设置比特率
    pub fn bitrate(mut self, bitrate: u32) -> Self {
        self.bitrate = bitrate;
        self
    }

    /// 设置声道数
    pub fn channels(mut self, channels: u8) -> Self {
        self.channels = channels;
        self
    }

    /// 设置立体声模式
    pub fn stereo_mode(mut self, mode: StereoMode) -> Self {
        self.stereo_mode = mode;
        self
    }

    /// 设置版权标志
    pub fn copyright(mut self, copyright: bool) -> Self {
        self.copyright = copyright;
        self
    }

    /// 设置原创标志
    pub fn original(mut self, original: bool) -> Self {
        self.original = original;
        self
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 检查采样率
        if !SUPPORTED_SAMPLE_RATES.contains(&self.sample_rate) {
            return Err(ConfigError::UnsupportedSampleRate(self.sample_rate));
        }

        // 检查比特率
        if !SUPPORTED_BITRATES.contains(&self.bitrate) {
            return Err(ConfigError::UnsupportedBitrate(self.bitrate));
        }

        // 检查声道数
        if self.channels == 0 || self.channels > 2 {
            return Err(ConfigError::InvalidChannels);
        }

        // 检查立体声模式与声道数的兼容性
        match (self.channels, self.stereo_mode) {
            (1, StereoMode::Mono) => {},
            (2, StereoMode::Stereo | StereoMode::JointStereo | StereoMode::DualChannel) => {},
            (channels, mode) => {
                return Err(ConfigError::InvalidStereoMode {
                    mode: format!("{:?}", mode),
                    channels,
                });
            }
        }

        // 使用shine的验证逻辑检查采样率和比特率组合
        let shine_result = crate::encoder::shine_check_config(
            self.sample_rate as i32, 
            self.bitrate as i32
        );
        
        if shine_result < 0 {
            // 确定MPEG版本以提供更详细的错误信息
            let mpeg_version = if self.sample_rate <= 12000 {
                "MPEG-2.5"
            } else if self.sample_rate <= 24000 {
                "MPEG-2"
            } else {
                "MPEG-1"
            };

            let reason = match mpeg_version {
                "MPEG-2.5" => format!("MPEG-2.5 ({}Hz) only supports bitrates up to 64 kbps", self.sample_rate),
                "MPEG-2" => format!("MPEG-2 ({}Hz) only supports bitrates up to 160 kbps", self.sample_rate),
                "MPEG-1" => format!("MPEG-1 ({}Hz) only supports bitrates from 32 to 320 kbps", self.sample_rate),
                _ => "Invalid combination".to_string(),
            };

            return Err(ConfigError::IncompatibleRateCombination {
                sample_rate: self.sample_rate,
                bitrate: self.bitrate,
                reason,
            });
        }

        Ok(())
    }
}

/// 高级MP3编码器
pub struct Mp3Encoder {
    /// 底层shine配置
    config: Box<ShineGlobalConfig>,
    /// 编码器配置
    encoder_config: Mp3EncoderConfig,
    /// 每次编码需要的样本数
    samples_per_frame: usize,
    /// 输入缓冲区
    input_buffer: VecDeque<i16>,
    /// 是否已完成编码
    finished: bool,
}

impl Mp3Encoder {
    /// 创建新的MP3编码器
    pub fn new(config: Mp3EncoderConfig) -> Result<Self, EncoderError> {
        // 验证配置
        config.validate()?;

        // 转换为shine配置
        let shine_config = Self::create_shine_config(&config)?;

        // 初始化shine编码器
        let global_config = shine_initialise(&shine_config)
            .map_err(|e| EncoderError::Encoding(e))?;

        // 计算每帧需要的样本数（交错格式的总样本数）
        let samples_per_channel = crate::encoder::shine_samples_per_pass(&global_config) as usize;
        let samples_per_frame = samples_per_channel * config.channels as usize;

        Ok(Self {
            config: global_config,
            encoder_config: config,
            samples_per_frame,
            input_buffer: VecDeque::new(),
            finished: false,
        })
    }

    /// 获取编码器配置
    pub fn config(&self) -> &Mp3EncoderConfig {
        &self.encoder_config
    }

    /// 获取每帧需要的样本数
    pub fn samples_per_frame(&self) -> usize {
        self.samples_per_frame
    }

    /// 获取底层shine配置（用于高级用户直接访问）
    pub fn shine_config(&mut self) -> &mut ShineGlobalConfig {
        &mut self.config
    }

    /// 编码PCM音频数据（交错格式）
    /// 
    /// # 参数
    /// - `pcm_data`: 交错格式的PCM数据 (左右声道交替)
    /// 
    /// # 返回值
    /// 返回编码后的MP3数据块的向量
    pub fn encode_interleaved(&mut self, pcm_data: &[i16]) -> Result<Vec<Vec<u8>>, EncoderError> {
        if self.finished {
            return Err(EncoderError::InternalState("Encoder has been finished".to_string()));
        }

        // 验证输入数据
        if pcm_data.is_empty() {
            return Err(EncoderError::InputData(InputDataError::EmptyInput));
        }

        // 将数据添加到缓冲区
        self.input_buffer.extend(pcm_data);

        let mut output_frames = Vec::new();

        // 处理完整的帧
        while self.input_buffer.len() >= self.samples_per_frame {
            let frame_data: Vec<i16> = self.input_buffer.drain(..self.samples_per_frame).collect();
            
            // 调用底层编码函数
            let (mp3_data, written) = shine_encode_buffer_interleaved(
                &mut self.config,
                frame_data.as_ptr()
            ).map_err(|e| EncoderError::Encoding(e))?;

            if written > 0 {
                output_frames.push(mp3_data[..written].to_vec());
            }
        }

        Ok(output_frames)
    }

    /// 编码PCM音频数据（分离声道格式）
    /// 
    /// # 参数
    /// - `left_channel`: 左声道数据
    /// - `right_channel`: 右声道数据（单声道时可为None）
    /// 
    /// # 返回值
    /// 返回编码后的MP3数据块的向量
    pub fn encode_separate_channels(
        &mut self,
        left_channel: &[i16],
        right_channel: Option<&[i16]>
    ) -> Result<Vec<Vec<u8>>, EncoderError> {
        if self.finished {
            return Err(EncoderError::InternalState("Encoder has been finished".to_string()));
        }

        // 验证输入数据
        if left_channel.is_empty() {
            return Err(EncoderError::InputData(InputDataError::EmptyInput));
        }

        // 验证声道数据一致性
        match (self.encoder_config.channels, right_channel) {
            (1, None) => {
                // 单声道，只使用左声道
                self.encode_interleaved(left_channel)
            },
            (2, Some(right)) => {
                if left_channel.len() != right.len() {
                    return Err(EncoderError::InputData(InputDataError::InvalidChannelCount {
                        expected: left_channel.len(),
                        actual: right.len(),
                    }));
                }
                
                // 交错合并左右声道
                let mut interleaved = Vec::with_capacity(left_channel.len() * 2);
                for (l, r) in left_channel.iter().zip(right.iter()) {
                    interleaved.push(*l);
                    interleaved.push(*r);
                }
                
                self.encode_interleaved(&interleaved)
            },
            (1, Some(_)) => {
                Err(EncoderError::InputData(InputDataError::InvalidChannelCount {
                    expected: 1,
                    actual: 2,
                }))
            },
            (2, None) => {
                Err(EncoderError::InputData(InputDataError::InvalidChannelCount {
                    expected: 2,
                    actual: 1,
                }))
            },
            _ => unreachable!(),
        }
    }

    /// 完成编码并获取剩余数据
    /// 
    /// # 返回值
    /// 返回最后的MP3数据块
    pub fn finish(&mut self) -> Result<Vec<u8>, EncoderError> {
        if self.finished {
            return Ok(Vec::new());
        }

        self.finished = true;

        // 处理剩余的不完整帧（用零填充）
        let mut final_output = Vec::new();

        if !self.input_buffer.is_empty() {
            // 用零填充到完整帧大小
            while self.input_buffer.len() < self.samples_per_frame {
                self.input_buffer.push_back(0);
            }

            let frame_data: Vec<i16> = self.input_buffer.drain(..).collect();
            
            let (mp3_data, written) = shine_encode_buffer_interleaved(
                &mut self.config,
                frame_data.as_ptr()
            ).map_err(|e| EncoderError::Encoding(e))?;

            if written > 0 {
                final_output.extend_from_slice(&mp3_data[..written]);
            }
        }

        // 刷新编码器缓冲区
        let (flush_data, flush_written) = shine_flush(&mut self.config);
        if flush_written > 0 {
            final_output.extend_from_slice(&flush_data[..flush_written]);
        }

        Ok(final_output)
    }

    /// 获取缓冲区中剩余的样本数
    pub fn buffered_samples(&self) -> usize {
        self.input_buffer.len()
    }

    /// 检查编码器是否已完成
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// 创建shine配置
    fn create_shine_config(config: &Mp3EncoderConfig) -> Result<ShineConfig, ConfigError> {
        let mut mpeg = ShineMpeg {
            mode: config.stereo_mode as i32,
            bitr: config.bitrate as i32,
            emph: NONE,
            copyright: if config.copyright { 1 } else { 0 },
            original: if config.original { 1 } else { 0 },
        };

        // 设置默认值
        shine_set_config_mpeg_defaults(&mut mpeg);
        
        // 应用用户配置
        mpeg.mode = config.stereo_mode as i32;
        mpeg.bitr = config.bitrate as i32;
        mpeg.copyright = if config.copyright { 1 } else { 0 };
        mpeg.original = if config.original { 1 } else { 0 };

        let wave = ShineWave {
            channels: config.channels as i32,
            samplerate: config.sample_rate as i32,
        };

        Ok(ShineConfig { wave, mpeg })
    }
}

impl Drop for Mp3Encoder {
    fn drop(&mut self) {
        // 注意：这里我们不能调用shine_close，因为它需要获取Box的所有权
        // 但是Rust的Drop trait只提供&mut self
        // 幸运的是，Rust的自动内存管理会处理清理工作
    }
}

/// 便利函数：一次性编码整个PCM数据
/// 
/// # 参数
/// - `config`: 编码器配置
/// - `pcm_data`: 交错格式的PCM数据
/// 
/// # 返回值
/// 返回完整的MP3数据
pub fn encode_pcm_to_mp3(
    config: Mp3EncoderConfig,
    pcm_data: &[i16]
) -> Result<Vec<u8>, EncoderError> {
    let mut encoder = Mp3Encoder::new(config)?;
    
    let mut mp3_data = Vec::new();
    
    // 编码所有数据
    let frames = encoder.encode_interleaved(pcm_data)?;
    for frame in frames {
        mp3_data.extend(frame);
    }
    
    // 完成编码
    let final_data = encoder.finish()?;
    mp3_data.extend(final_data);
    
    Ok(mp3_data)
}

  