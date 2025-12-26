use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CapturerSettings {
    #[serde(default = "CmdSettings::default")]
    pub cmd: CmdSettings,
    #[serde(default = "SessionSettings::default")]
    pub session: SessionSettings,
    #[serde(default = "OssSettings::default")]
    pub oss: OssSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CmdSettings {
    /// 命令读取缓冲区大小
    #[serde(default = "read_buffer_size_default")]
    pub read_buffer_size: usize,
    /// 命令通道容量(最大消息数量，默认500，如果是25帧/秒，那么最多可以缓存20秒的视频数据)
    #[serde(default = "channel_capacity_default")]
    pub channel_capacity: usize,
    /// 命令接收者数量检查间隔(单位为秒，默认30)
    #[serde(default = "receiver_count_check_interval_seconds_default")]
    pub receiver_count_check_interval_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SessionSettings {
    /// 会话超时检查间隔(单位为秒，默认60)
    #[serde(default = "timeout_check_interval_seconds_default")]
    pub timeout_check_interval_seconds: u64,
    /// 会话超时时间(单位为秒，默认30*60)
    #[serde(default = "timeout_seconds_default")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OssSettings {
    /// 存储桶
    #[serde(default = "bucket_default")]
    pub bucket: String,
    /// 转jpeg质量参数(1-31，数值越小质量越高，默认1)
    #[serde(default = "jpeg_quality_default")]
    pub jpeg_quality: u8,
}

impl Default for CapturerSettings {
    fn default() -> Self {
        CapturerSettings {
            cmd: CmdSettings::default(),
            session: SessionSettings::default(),
            oss: OssSettings::default(),
        }
    }
}

impl Default for CmdSettings {
    fn default() -> Self {
        CmdSettings {
            read_buffer_size: read_buffer_size_default(),
            channel_capacity: channel_capacity_default(),
            receiver_count_check_interval_seconds: receiver_count_check_interval_seconds_default(),
        }
    }
}

fn receiver_count_check_interval_seconds_default() -> u64 {
    5
}

impl Default for SessionSettings {
    fn default() -> Self {
        SessionSettings {
            timeout_check_interval_seconds: timeout_check_interval_seconds_default(),
            timeout_seconds: timeout_seconds_default(),
        }
    }
}

impl Default for OssSettings {
    fn default() -> Self {
        OssSettings {
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
fn timeout_check_interval_seconds_default() -> u64 {
    60
}

fn timeout_seconds_default() -> u64 {
    30 * 60
}

fn channel_capacity_default() -> usize {
    500
}

fn read_buffer_size_default() -> usize {
    65536
}
