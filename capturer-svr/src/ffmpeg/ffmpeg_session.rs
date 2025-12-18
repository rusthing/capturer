use bytes::Bytes;
use chrono::{DateTime, Utc};
use log::debug;
use std::sync::{Arc, OnceLock, RwLock};
use tokio::sync::broadcast::Sender;
use wheel_rs::cmd::std::kill_process_by_id;

/// ffmpeg会话结构体
///
/// 代表一个正在进行的ffmpeg流处理会话，包含相关信息和资源
#[derive(Clone)]
pub struct FfmpegSession {
    /// 子进程ID
    pub child_id: u32,
    /// 数据发送者
    pub data_sender: Arc<Sender<Bytes>>,
    /// 视频格式头缓存
    pub header: Arc<OnceLock<Bytes>>,
    /// 最后访问时间
    ///
    /// None表示当前会话处于活跃状态
    /// Some(DateTime)表示会话最后一次被访问的时间，用于判断是否超时
    pub last_access_datetime: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl Drop for FfmpegSession {
    /// 当会话被销毁时，自动终止关联的ffmpeg子进程
    fn drop(&mut self) {
        debug!("会话关闭...");
        let _ = kill_process_by_id(self.child_id);
    }
}
