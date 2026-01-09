use crate::ffmpeg::ffmpeg_cmd::FfmpegCmd;
use crate::ffmpeg::ffmpeg_error::FfmpegError;
use crate::ffmpeg::ffmpeg_session::FfmpegSession;
use crate::config::capturer_config::{CmdConfig, SessionConfig};
use crate::config::app_config::APP_CONFIG;
use bytes::Bytes;
use chrono::Utc;
use log::{debug, error, info, trace, warn};
use rustc_hash::FxHashMap;
use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, oneshot};
use tokio::time::interval;

/// 全局静态的流管理器实例
pub static STREAM_MANAGER: LazyLock<StreamManager> = LazyLock::new(|| StreamManager::new());

/// 流管理器，负责管理ffmpeg流会话
///
/// 该管理器维护一个会话映射表，用于存储和管理所有活动的流会话。
/// 它还负责定期清理过期的会话，并处理会话生命周期相关事件。
pub struct StreamManager {
    /// 命令读取缓冲区大小
    cmd_read_buffer_size: usize,
    /// 命令广播通道容量
    cmd_channel_capacity: usize,
    /// 会话存储映射表，使用URL作为键，FfmpegSession作为值
    sessions: Arc<RwLock<FxHashMap<String, FfmpegSession>>>,
}

impl StreamManager {
    /// 创建一个新的流管理器实例
    ///
    /// 该函数会从配置中读取相关设置，并启动后台任务来定期清理过期会话。
    pub fn new() -> Self {
        let CmdConfig {
            read_buffer_size: cmd_read_buffer_size,
            channel_capacity: cmd_channel_capacity,
            ..
        } = APP_CONFIG.get().expect("无法获取设置").capturer.cmd;
        let SessionConfig {
            timeout_check_interval: Some(session_timeout_check_interval),
            timeout_period: Some(session_timeout_period),
            ..
        } = APP_CONFIG.get().expect("无法获取设置").capturer.session
        else {
            unreachable!("会话超时检查间隔和超时时间必须配置");
        };

        // 创建会话容器
        let sessions: Arc<RwLock<FxHashMap<String, FfmpegSession>>> =
            Arc::new(RwLock::new(FxHashMap::default()));

        debug!("<定时清除过期会话>任务正在创建....");
        let sessions_clone = Arc::clone(&sessions);
        tokio::spawn(async move {
            let mut interval = interval(session_timeout_check_interval);
            info!(
                "<定时清除过期会话>任务创建完成. 定时检查间隔: {session_timeout_check_interval:?}"
            );
            loop {
                interval.tick().await;
                Self::cleanup_expired_sessions(sessions_clone.clone(), session_timeout_period)
                    .await;
            }
        });

        Self {
            cmd_read_buffer_size,
            cmd_channel_capacity,
            sessions,
        }
    }

    /// 获取指定URL的命令接收者
    ///
    /// 如果该URL对应的会话已存在，则返回现有会话的命令接收者；
    /// 否则创建一个新的ffmpeg会话并返回其命令接收者。
    ///
    /// # 参数
    ///
    /// * `url`: 流媒体地址
    ///
    /// # 返回值
    ///
    /// 返回一个包含以下元素的元组：
    /// * `Receiver<Bytes>`: 命令接收者
    /// * `Arc<OnceLock<Bytes>>`: 视频头部信息缓存
    /// * `Option<oneshot::Sender<Bytes>>`: 缓存头部信息发送器（仅新会话有）
    ///
    /// # 错误处理
    ///
    /// 如果无法获取锁或ffmpeg命令执行失败，将返回相应的错误。
    pub async fn get_cmd_receiver(
        &self,
        url: &str,
    ) -> Result<
        (
            Receiver<Bytes>,
            Arc<OnceLock<Bytes>>,
            Option<oneshot::Sender<Bytes>>,
        ),
        FfmpegError,
    > {
        info!("获取命令接收者: {}", url);
        let cmd_receiver_count_check_interval = APP_CONFIG
            .get()
            .expect("无法获取设置")
            .capturer
            .cmd
            .receiver_count_check_interval
            .unwrap();

        let sessions = &self.sessions;
        {
            debug!("获取会话读锁...");
            let sessions_read_lock = sessions.read().map_err(|e| {
                error!("无法获取会话读锁: {}", e);
                FfmpegError::FfmpegSessionReadError("无法获取会话读锁".to_string())
            })?;

            debug!("检查会话是否存在: {}", url);
            if let Some(session) = sessions_read_lock.get(url) {
                {
                    debug!("获取 last_access_datetime 写锁...");
                    let mut last_access_datetime_write_lock =
                        session.last_access_datetime.write().map_err(|e| {
                            error!("无法获取 last_access_datetime 写锁: {}", e);
                            FfmpegError::FfmpegSessionReadError(
                                "无法获取 last_access_datetime 写锁".to_string(),
                            )
                        })?;
                    debug!("更新最后访问时间为None（表示活跃中）: {:?}", Utc::now());
                    *last_access_datetime_write_lock = None;
                }

                debug!("返回命令接收者...");
                return Ok((
                    session.data_sender.subscribe(),
                    Arc::clone(&session.header),
                    None,
                ));
            }
        }

        debug!("创建新会话...");
        let (data_sender, data_receiver) = broadcast::channel(self.cmd_channel_capacity);
        let (process_exit_sender, process_exit_receiver) = oneshot::channel();
        let (cache_header_sender, cache_header_receiver) = oneshot::channel();
        let last_access_datetime = Arc::new(RwLock::new(None));

        // 拉流并解码
        let child = Arc::new(
            FfmpegCmd::pull_and_transcode_stream(
                url,
                data_sender.clone(),
                process_exit_sender,
                self.cmd_read_buffer_size,
            )
            .await?,
        );

        let child_id = child
            .id()
            .ok_or_else(|| FfmpegError::FfmpegSessionReadError("无法获取子进程ID".to_string()))?;
        debug!("ffmpeg child pid: {child_id}");

        info!("<子进程{child_id}>会话正在创建....");
        let data_sender = Arc::new(data_sender);
        let header = Arc::new(OnceLock::new());
        let session = FfmpegSession {
            child_id,
            last_access_datetime,
            data_sender: Arc::clone(&data_sender),
            header: Arc::clone(&header),
        };
        // 插入新会话到会话映射表
        {
            debug!("获取会话写锁...");
            let mut sessions_write_lock = sessions.write().map_err(|e| {
                error!("无法获取会话写锁: {}", e);
                FfmpegError::FfmpegSessionReadError("无法获取会话写锁".to_string())
            })?;
            sessions_write_lock.insert(url.to_string(), session.clone());
        }
        info!("<子进程{child_id}>会话创建完成.");

        // 启动监听接收者数量的任务
        info!("<监听子进程{child_id}接收者数量>任务正在创建....");
        let session_clone = Arc::new(session);
        tokio::spawn(async move {
            let mut interval = interval(cmd_receiver_count_check_interval);
            info!(
                "<监听子进程{child_id}接收者数量>任务创建完成. 定时检查间隔: {cmd_receiver_count_check_interval:?}"
            );
            loop {
                interval.tick().await;

                if data_sender.receiver_count() == 0 {
                    trace!("获取 last_access_datetime 读锁...");
                    let last_access_datetime = if let Ok(last_access_datetime_read_lock) =
                        session_clone.last_access_datetime.read()
                    {
                        *last_access_datetime_read_lock
                    } else {
                        warn!("无法获取 last_access_datetime 读锁");
                        continue;
                    };
                    if last_access_datetime.is_none() {
                        info!("子进程{child_id}接收者数量==0，记录会话过期时间");
                        trace!("获取 last_access_datetime 写锁...");
                        if let Ok(mut last_access_datetime_write_lock) =
                            session_clone.last_access_datetime.write()
                        {
                            let now = Utc::now();
                            debug!("更新最后访问时间为当前时间: {:?}", Utc::now());
                            *last_access_datetime_write_lock = Some(now);
                        } else {
                            warn!("无法获取 last_access_datetime 写锁");
                        }
                    }
                }
            }
        });

        // 启动监听缓存头部的任务
        info!("<监听子进程{child_id}缓存头部>任务正在创建....");
        let header_clone = Arc::clone(&header);
        let cache_header_receiver_clone = cache_header_receiver; // 重命名变量避免混淆
        tokio::spawn(async move {
            info!("<监听子进程{child_id}缓存头部>任务创建完成.");
            if let Ok(bytes) = cache_header_receiver_clone.await {
                debug!("子进程{child_id}缓存头部: {:?}", bytes);
                let _ = header_clone.set(bytes);
            } else {
                warn!("无法接收缓存头部数据");
            }
        });

        // 启动监听子进程退出的任务
        info!("<监听子进程{child_id}退出>任务正在创建....");
        let sessions_clone = Arc::clone(&self.sessions);
        tokio::spawn(async move {
            info!("<监听子进程{child_id}退出>任务创建完成.");
            if let Ok(_) = process_exit_receiver.await {
                debug!("检测到子进程{child_id}已经退出");
                Self::remove_session_after_process_exit(sessions_clone, child_id).await;
            } else {
                warn!("监听子进程退出通道异常关闭");
            }
        });

        Ok((data_receiver, header, Some(cache_header_sender)))
    }

    /// 进程退出后删除会话
    ///
    /// 当ffmpeg子进程退出时，该函数会被调用以清理对应的会话记录。
    ///
    /// # 参数
    ///
    /// * `sessions`: 会话存储映射表的引用
    /// * `child_id`: 退出的子进程ID
    async fn remove_session_after_process_exit(
        sessions: Arc<RwLock<FxHashMap<String, FfmpegSession>>>,
        child_id: u32,
    ) {
        info!("子进程{child_id}退出后删除会话");
        let mut remove_key = None;

        debug!("获取 sessions 读锁...");
        if let Ok(sessions_read_lock) = sessions.read() {
            debug!("查找要删除子进程{child_id}的会话");
            for (key, session) in sessions_read_lock.iter() {
                if session.child_id == child_id {
                    remove_key = Some(key.clone());
                    break;
                }
            }
        } else {
            warn!("无法获取 sessions 读锁");
            return;
        }

        // 删除找到的会话
        if let Some(remove_key) = remove_key {
            debug!("删除会话: {}", remove_key);
            debug!("获取 sessions 写锁...");
            if let Ok(mut sessions_write_lock) = sessions.write() {
                sessions_write_lock.remove(&remove_key);
            } else {
                warn!("无法获取 sessions 写锁");
            }
        }
    }

    /// 清理超过一定时间未访问的会话
    ///
    /// 定期执行的清理任务，会遍历所有会话并移除超时的会话。
    ///
    /// # 参数
    ///
    /// * `sessions`: 会话存储映射表的引用
    /// * `timeout_seconds`: 会话超时时间（秒）
    async fn cleanup_expired_sessions(
        sessions: Arc<RwLock<FxHashMap<String, FfmpegSession>>>,
        timeout_period: Duration,
    ) {
        info!("开始执行<清理过期会话>任务....");
        let mut expired_keys = Vec::new();

        debug!("获取 sessions 读锁...");
        if let Ok(sessions_read_lock) = sessions.read() {
            debug!("遍历所有会话, 检查是否过期...");
            for (key, session) in sessions_read_lock.iter() {
                debug!("获取 last_access_datetime 读锁...");
                if let Ok(last_access_datetime_read_lock) = session.last_access_datetime.read() {
                    debug!("检查会话{key}是否过期....");
                    if let Some(last_access_datetime) = *last_access_datetime_read_lock {
                        let cut_off_datetime = last_access_datetime + timeout_period;
                        debug!(
                            "会话{key}最后访问时间: {last_access_datetime}, 过期时间: {cut_off_datetime}"
                        );
                        if Utc::now() > cut_off_datetime {
                            debug!("会话{key}已过期, 需要删除会话");
                            expired_keys.push(key.clone());
                        } else {
                            debug!("会话{key}未过期, 无需删除");
                        }
                    }
                } else {
                    warn!("无法获取 last_access_datetime 读锁");
                }
            }
            debug!("遍历完成, 删除过期会话...");
        } else {
            warn!("无法获取 sessions 读锁");
            return;
        }

        // 删除过期会话
        if !expired_keys.is_empty() {
            info!("<清理过期会话>任务正在创建....");
            let sessions_clone = Arc::clone(&sessions);
            tokio::spawn(async move {
                info!("<清理过期会话>任务创建完成.");
                for key in expired_keys {
                    debug!("开始删除会话{key}....");
                    if let Ok(mut sessions_write_lock) = sessions_clone.write() {
                        sessions_write_lock.remove(&key);
                    } else {
                        warn!("无法获取 sessions 写锁");
                    }
                    debug!("会话{key}删除完成.");
                }
            });
        }
    }
}
