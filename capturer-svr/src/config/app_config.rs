use crate::config::capturer_config::CapturerConfig;
use robotech::api_client::ApiClientConfig;
use robotech::web::WebServerConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    /// 抓拍机设置置
    #[serde(default = "CapturerConfig::default")]
    pub capturer: CapturerConfig,
    /// Web服务器设置
    #[serde(default = "WebServerConfig::default")]
    pub web: WebServerConfig,
    /// API客户端设置
    #[serde(default = "HashMap::default")]
    pub api_client: HashMap<String, ApiClientConfig>,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        Self {
            capturer: CapturerConfig::default(),
            web: WebServerConfig::default(),
            api_client: HashMap::default(),
        }
    }
}
