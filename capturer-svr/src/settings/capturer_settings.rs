use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CapturerSettings {
    /// 存储桶
    #[serde(default = "bucket_default")]
    pub bucket: String,
}

impl Default for CapturerSettings {
    fn default() -> Self {
        CapturerSettings {
            bucket: bucket_default(),
        }
    }
}

fn bucket_default() -> String {
    "capturer".to_string()
}
