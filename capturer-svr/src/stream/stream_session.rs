use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::sync::{Arc, OnceLock, RwLock};
use tokio::process::Child;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct StreamSession {
    /// 最后访问时间
    pub last_access_datetime: Arc<RwLock<DateTime<Utc>>>,
    /// 发送者
    pub sender: Arc<RwLock<Sender<Bytes>>>,
    /// 子进程
    pub child: Arc<Child>,
    /// 视频格式头缓存
    pub header: Arc<OnceLock<Bytes>>,
}
