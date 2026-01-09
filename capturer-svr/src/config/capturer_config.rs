use serde::{Deserialize, Serialize};
use std::time::Duration;
use wheel_rs::serde::duration_option_serde;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CapturerConfig {
    #[serde(default = "CmdConfig::default")]
    pub cmd: CmdConfig,
    #[serde(default = "SessionConfig::default")]
    pub session: SessionConfig,
    #[serde(default = "OssConfig::default")]
    pub oss: OssConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CmdConfig {
    /// 命令读取缓冲区大小
    #[serde(default = "read_buffer_size_default")]
    pub read_buffer_size: usize,
    /// 命令通道容量(最大消息数量，默认500，如果是25帧/秒，那么最多可以缓存20秒的视频数据)
    #[serde(default = "channel_capacity_default")]
    pub channel_capacity: usize,
    /// 命令接收者数量检查间隔(单位为秒，默认30)
    #[serde(
        with = "duration_option_serde",
        default = "receiver_count_check_interval_default"
    )]
    pub receiver_count_check_interval: Option<Duration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SessionConfig {
    /// 会话超时检查间隔(单位为秒，默认60)
    #[serde(
        with = "duration_option_serde",
        default = "timeout_check_interval_default"
    )]
    pub timeout_check_interval: Option<Duration>,
    /// 会话超时时间(单位为秒，默认30*60)
    #[serde(with = "duration_option_serde", default = "timeout_period_default")]
    pub timeout_period: Option<Duration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OssConfig {
    /// 存储桶
    #[serde(default = "bucket_default")]
    pub bucket: String,
    /// 转jpeg质量参数(1-31，数值越小质量越高，默认1)
    #[serde(default = "jpeg_quality_default")]
    pub jpeg_quality: u8,
}

impl Default for CapturerConfig {
    fn default() -> Self {
        CapturerConfig {
            cmd: CmdConfig::default(),
            session: SessionConfig::default(),
            oss: OssConfig::default(),
        }
    }
}

impl Default for CmdConfig {
    fn default() -> Self {
        CmdConfig {
            read_buffer_size: read_buffer_size_default(),
            channel_capacity: channel_capacity_default(),
            receiver_count_check_interval: receiver_count_check_interval_default(),
        }
    }
}

fn receiver_count_check_interval_default() -> Option<Duration> {
    Some(Duration::from_secs(5))
}

impl Default for SessionConfig {
    fn default() -> Self {
        SessionConfig {
            timeout_check_interval: timeout_check_interval_default(),
            timeout_period: timeout_period_default(),
        }
    }
}

impl Default for OssConfig {
    fn default() -> Self {
        OssConfig {
            bucket: bucket_default(),
            jpeg_quality: jpeg_quality_default(),
        }
    }
}

fn bucket_default() -> String {
    "capturer".to_string()
}

fn jpeg_quality_default() -> u8 {
    1
}
fn timeout_check_interval_default() -> Option<Duration> {
    Some(Duration::from_secs(60))
}

fn timeout_period_default() -> Option<Duration> {
    Some(Duration::from_secs(30 * 60))
}

fn channel_capacity_default() -> usize {
    500
}

fn read_buffer_size_default() -> usize {
    65536
}
