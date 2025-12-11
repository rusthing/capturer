use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FfprobeCmdStreamsInfo {
    pub codec_type: String,
    pub codec_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub r_frame_rate: String,
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
    /// MP3 编解码器
    MP3,
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
}
//
// /// 流媒体会话结构
// #[derive(Clone)]
// pub struct StreamSession {
//     /// 流ID
//     pub stream_id: String,
//     /// RTSP地址
//     pub rtsp_url: String,
//     /// 最后访问时间
//     pub last_access: Arc<RwLock<DateTime<Utc>>>,
//     /// 是否激活状态
//     pub is_active: Arc<RwLock<bool>>,
//     /// 帧缓冲区
//     pub frame_buffer: Arc<RwLock<Vec<u8>>>,
//     /// 流媒体元数据
//     pub metadata: Arc<RwLock<Option<StreamMetadata>>>,
//     /// ffmpeg进程
//     pub ffmpeg_process: Arc<RwLock<Option<Child>>>,
//     /// 错误计数
//     pub error_count: Arc<RwLock<u32>>,
//     /// 最后错误时间
//     pub last_error_time: Arc<RwLock<Option<DateTime<Utc>>>>,
//     /// 重启计数
//     pub restart_count: Arc<RwLock<u32>>,
// }
//
// /// 流管理器
// pub struct StreamManager {
//     /// 会话集合
//     sessions: Arc<DashMap<String, StreamSession>>,
//     /// 空闲超时秒数
//     idle_timeout_secs: u64,
//     /// 最大重启尝试次数
//     max_restart_attempts: u32,
//     /// 错误阈值
//     error_threshold: u32,
// }
