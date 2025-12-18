use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FfprobeCmdStreamsInfo {
    pub codec_type: String,
    pub codec_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub r_frame_rate: Option<String>,
    pub sample_rate: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FfprobeCmdInfo {
    pub programs: Vec<String>,
    pub streams: Vec<FfprobeCmdStreamsInfo>,
}

/// 视频编解码器类型枚举
#[derive(Debug, PartialEq, Clone)]
pub enum VideoCodecType {
    /// H.264 编解码器
    H264,
    /// H.265 (HEVC) 编解码器
    H265,
    /// 其它编解码器，携带原始编解码器名称
    Other(String),
}

/// 音频编解码器类型枚举
#[derive(Debug, PartialEq, Clone)]
pub enum AudioCodecType {
    /// 未知音频编解码器
    Unknown,
    /// AAC 编解码器
    AAC,
    /// MP2 编解码器
    MP2,
    /// MP3 编解码器
    MP3,
    /// PCM Mu-law 编解码器
    G711mulaw,
    /// PCM A-law 编解码器
    G711alaw,
    /// ADPCM G.726-LE 编解码器
    G726,
    /// 不支持的音频编解码器，携带原始编解码器名称
    NotSupported(String),
    /// 其它编解码器，携带原始编解码器名称
    Other(String),
}

/// 流媒体元数据结构
#[derive(Debug, Default, Clone)]
pub struct StreamMetadata {
    /// 视频编解码器类型
    pub video_codec: Option<VideoCodecType>,
    /// 音频编解码器类型
    pub audio_codec: Option<AudioCodecType>,
    /// 视频宽度
    pub width: Option<u32>,
    /// 视频高度
    pub height: Option<u32>,
    /// 帧率
    pub fps: Option<u8>,
    /// 音频采样率
    pub sample_rate: Option<u32>,
}
