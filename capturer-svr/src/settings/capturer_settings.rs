use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CapturerSettings {
    /// 存储桶
    #[serde(default = "bucket_default")]
    pub bucket: String,
    #[serde()]
    pub jpeg_quality: Option<u8>,
}

impl Default for CapturerSettings {
    fn default() -> Self {
        CapturerSettings {
            bucket: bucket_default(),
            jpeg_quality: None,
        }
    }
}

fn bucket_default() -> String {
    "capturer".to_string()
}
