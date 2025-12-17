use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::sync::{Arc, OnceLock, RwLock};
use tokio::process::Child;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct StreamSession {
    /// 子进程
    pub child: Arc<Child>,
    /// 数据发送者
    pub data_sender: Arc<Sender<Bytes>>,
    /// 视频格式头缓存
    pub header: Arc<OnceLock<Bytes>>,
    /// 最后访问时间
    pub last_access_datetime: Arc<RwLock<Option<DateTime<Utc>>>>,
}
