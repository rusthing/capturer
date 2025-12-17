use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CapturerSettings {
    #[serde(default = "OssSettings::default")]
    pub oss: OssSettings,
    #[serde(default = "StreamSettings::default")]
    pub stream: StreamSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StreamSettings {
    /// 读取缓冲区大小
    #[serde()]
    pub read_buffer_size: Option<usize>,
    /// 进程检查间隔(单位为秒，默认60)
    #[serde(default = "check_interval_seconds_default")]
    pub check_interval_seconds: u64,
    /// 超时时间(单位为秒，默认30*60)
    #[serde(default = "timeout_seconds_default")]
    pub timeout_seconds: u64,
    /// 通道容量(默认100)
    #[serde(default = "channel_capacity_default")]
    pub channel_capacity: usize,
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
            oss: OssSettings::default(),
            stream: StreamSettings::default(),
        }
    }
}

impl Default for StreamSettings {
    fn default() -> Self {
        StreamSettings {
            read_buffer_size: None,
            check_interval_seconds: check_interval_seconds_default(),
            timeout_seconds: timeout_seconds_default(),
            channel_capacity: channel_capacity_default(),
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

fn check_interval_seconds_default() -> u64 {
    60
}

fn timeout_seconds_default() -> u64 {
    30 * 60
}

fn channel_capacity_default() -> usize {
    100
}
