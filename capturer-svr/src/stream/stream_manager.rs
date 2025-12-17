use crate::ffmpeg::ffmpeg_cmd::FfmpegCmd;
use crate::ffmpeg::ffmpeg_error::FfmpegError;
use crate::settings::capturer_settings::StreamSettings;
use crate::settings::settings::SETTINGS;
use crate::stream::stream_session::StreamSession;
use bytes::Bytes;
use chrono::{Duration, Utc};
use log::{debug, info};
use rustc_hash::FxHashMap;
use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, oneshot};
use tokio::time::{interval, Duration as TokioDuration};
use wheel_rs::cmd::spawn::kill_process;

pub static STREAM_MANAGER: LazyLock<StreamManager> = LazyLock::new(|| StreamManager::new());

pub struct StreamManager {
    sessions: Arc<RwLock<FxHashMap<String, StreamSession>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        let StreamSettings {
            check_interval_seconds,
            timeout_seconds,
            ..
        } = SETTINGS.get().unwrap().capturer.stream;
        let sessions: Arc<RwLock<FxHashMap<String, StreamSession>>> =
            Arc::new(RwLock::new(FxHashMap::default()));
        // 定时检查器
        let mut interval_timer = interval(TokioDuration::from_secs(check_interval_seconds));
        info!("<定时清除过期会话>任务正在创建....");
        let sessions_clone = sessions.clone();
        tokio::spawn(async move {
            info!("<定时清除过期会话>任务创建完成. 定时检查间隔: {check_interval_seconds}秒");
            loop {
                interval_timer.tick().await;
                Self::cleanup_expired_sessions(sessions_clone.clone(), timeout_seconds).await;
            }
        });
        Self { sessions }
    }

    pub async fn get_data_receiver(
        &self,
        url: &str,
        channel_capacity: usize,
    ) -> Result<
        (
            Receiver<Bytes>,
            Arc<OnceLock<Bytes>>,
            Option<oneshot::Sender<Bytes>>,
        ),
        FfmpegError,
    > {
        let sessions = &self.sessions;
        Ok(
            if let Some(stream_session) = sessions.read().unwrap().get(url) {
                *stream_session.last_access_datetime.write().unwrap() = None;
                (
                    stream_session.data_sender.subscribe(),
                    Arc::clone(&stream_session.header),
                    None,
                )
            } else {
                {
                    let read_buffer_size = SETTINGS.get().unwrap().capturer.stream.read_buffer_size;
                    let (data_sender, data_receiver) = broadcast::channel(channel_capacity);
                    let (process_exit_sender, process_exit_receiver) = oneshot::channel();
                    let (cache_header_sender, cache_header_receiver) = oneshot::channel();
                    let last_access_datetime = Arc::new(RwLock::new(None));

                    info!("开始拉流与解码....");
                    let child = Arc::new(
                        FfmpegCmd::pull_and_transcode_stream(
                            url,
                            data_sender.clone(),
                            process_exit_sender,
                            read_buffer_size,
                        )
                        .await?,
                    );
                    let child_id = child.id().unwrap();
                    info!("ffmpeg child pid: {child_id}");

                    info!("<子进程{child_id}>会话正在创建....");
                    let data_sender = Arc::new(data_sender);
                    let header = Arc::new(OnceLock::new());
                    let session = StreamSession {
                        child: child.clone(),
                        last_access_datetime,
                        data_sender,
                        header: Arc::clone(&header),
                    };
                    {
                        let mut write_guard = sessions.write().unwrap();
                        write_guard.insert(url.to_string(), session.clone());
                    }
                    info!("<子进程{child_id}>会话创建完成.");

                    info!("<监听子进程{child_id}缓存头部>任务正在创建....");
                    let header_clone = Arc::clone(&header);
                    tokio::spawn(async move {
                        info!("<监听子进程{child_id}缓存头部>任务创建完成.");
                        let bytes = cache_header_receiver.await.unwrap();
                        debug!("子进程{child_id}缓存头部: {:?}", bytes);
                        let _ = header_clone.set(bytes);
                    });

                    info!("<监听子进程{child_id}退出>任务正在创建....");
                    let sessions_clone = sessions.clone();
                    tokio::spawn(async move {
                        info!("<监听子进程{child_id}退出>任务创建完成.");
                        let _ = process_exit_receiver.await;
                        debug!("检测到子进程{child_id}已经退出");
                        Self::remove_session_after_process_exit(sessions_clone, child_id).await;
                    });

                    (data_receiver, header, Some(cache_header_sender))
                }
            },
        )
    }

    /// 进程退出后删除会话
    async fn remove_session_after_process_exit(
        sessions: Arc<RwLock<FxHashMap<String, StreamSession>>>,
        child_id: u32,
    ) {
        debug!("子进程{child_id}退出后删除会话");
        let mut remove_key = None;
        {
            let read_guard = sessions.read().unwrap();
            for (key, session) in read_guard.iter() {
                if session.child.id().unwrap() == child_id {
                    remove_key = Some(key.clone());
                    break;
                }
            }
        }
        let remove_key = remove_key.unwrap();
        debug!("删除会话: {}", remove_key);
        {
            let mut write_guard = sessions.write().unwrap();
            write_guard.remove(&remove_key);
        }
    }

    /// 清理超过一定时间未访问的会话
    async fn cleanup_expired_sessions(
        sessions: Arc<RwLock<FxHashMap<String, StreamSession>>>,
        timeout_seconds: u64,
    ) {
        debug!("开始清理过期会话....");
        let cut_off_datetime = Utc::now() - Duration::minutes(timeout_seconds as i64);
        let mut expired_keys = Vec::new();

        // 收集过期的会话键
        {
            let sessions_read_lock = sessions.read().unwrap();
            for (key, session) in sessions_read_lock.iter() {
                let last_access_datetime_lock = session.last_access_datetime.read().unwrap();
                if let Some(last_access_datetime) = *last_access_datetime_lock {
                    if last_access_datetime < cut_off_datetime {
                        expired_keys.push((key.clone(), Arc::clone(&session.child)));
                    }
                }
            }
        }

        if !expired_keys.is_empty() {
            info!("<清理过期会话>任务正在创建....");
            tokio::spawn(async move {
                info!("<清理过期会话>任务创建完成.");
                for (key, child) in expired_keys {
                    debug!("开始删除会话{key}....");
                    {
                        let mut write_guard = sessions.write().unwrap();
                        write_guard.remove(&key);
                    }
                    debug!("会话{key}删除完成.");
                    let child = Arc::try_unwrap(child).unwrap();
                    let child_id = child.id().unwrap();
                    debug!("开始查杀子进程{child_id}....");
                    // 在独立任务中杀掉子进程，避免阻塞清理过程
                    let _ = kill_process(child).await;
                    debug!("子进程{child_id}查杀完成.");
                }
            });
        }
    }
}
