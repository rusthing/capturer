use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CapturerSettings {
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
