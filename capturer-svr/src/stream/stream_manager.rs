use crate::ffmpeg::ffmpeg_cmd::FfmpegCmd;
use crate::ffmpeg::ffmpeg_error::FfmpegError;
use crate::settings::capturer_settings::StreamSettings;
use crate::settings::settings::SETTINGS;
use crate::stream::stream_session::StreamSession;
use bytes::Bytes;
use chrono::{Duration, Utc};
use log::debug;
use rustc_hash::FxHashMap;
use std::sync::{mpsc, Arc, LazyLock, OnceLock, RwLock};
use std::{ptr, thread};
use tokio::process::Child;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
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
        // 开启线程做循环，遍历sessions，删除超时的会话
        let sessions_clone = sessions.clone();
        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;
                Self::cleanup_expired_sessions(sessions_clone.clone(), timeout_seconds).await;
            }
        });
        Self { sessions }
    }

    pub async fn get_session(&self, url: &str) -> Option<StreamSession> {
        let sessions = &self.sessions;
        if let Some(stream_session) = sessions.read().unwrap().get(url) {
            *stream_session.last_access_datetime.write().unwrap() = Utc::now();
            Some(stream_session.clone())
        } else {
            None
        }
    }

    pub async fn get_data_receiver(
        &self,
        url: &str,
    ) -> Result<(Receiver<Bytes>, Arc<OnceLock<Bytes>>), FfmpegError> {
        let sessions = &self.sessions;
        Ok(
            if let Some(stream_session) = sessions.read().unwrap().get(url) {
                *stream_session.last_access_datetime.write().unwrap() = Utc::now();
                (
                    stream_session.sender.read().unwrap().subscribe(),
                    stream_session.header.clone(),
                )
            } else {
                {
                    let read_buffer_size = SETTINGS.get().unwrap().capturer.stream.read_buffer_size;
                    let (data_sender, data_receiver) = broadcast::channel(100);
                    let (process_end_sender, process_end_receiver) = mpsc::channel();
                    let last_access_datetime = Arc::new(RwLock::new(Utc::now()));
                    let sender = Arc::new(RwLock::new(data_sender.clone()));
                    let header = Arc::new(OnceLock::new());
                    let child = Arc::new(
                        FfmpegCmd::pull_and_transcode_stream(
                            url,
                            data_sender,
                            process_end_sender,
                            read_buffer_size,
                        )
                        .await?,
                    );
                    let stream_session = StreamSession {
                        last_access_datetime,
                        sender,
                        child: child.clone(),
                        header: header.clone(),
                    };
                    {
                        let mut write_guard = sessions.write().unwrap();
                        write_guard.insert(url.to_string(), stream_session.clone());
                    }
                    let child_clone = child.clone();
                    let sessions_clone = sessions.clone();
                    thread::spawn(async move || {
                        let _ = process_end_receiver.recv();
                        Self::remove_session(sessions_clone, Arc::try_unwrap(child_clone).unwrap())
                            .await;
                    });
                    (data_receiver, header)
                }
            },
        )
    }

    /// 进程退出后删除会话
    async fn remove_session(sessions: Arc<RwLock<FxHashMap<String, StreamSession>>>, child: Child) {
        let mut remove_key = None;
        {
            let read_guard = sessions.read().unwrap();
            for (key, session) in read_guard.iter() {
                if ptr::eq(&Arc::try_unwrap(session.child.clone()).unwrap(), &child) {
                    remove_key = Some(key.clone());
                    break;
                }
            }
        }
        let remove_key = remove_key.unwrap();
        debug!("remove session: {}", remove_key);
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
        let cut_off_datetime = Utc::now() - Duration::minutes(timeout_seconds as i64);
        let mut expired_keys = Vec::new();

        // 收集过期的会话键
        {
            let read_guard = sessions.read().unwrap();
            for (key, session) in read_guard.iter() {
                let last_access_datetime = session.last_access_datetime.read().unwrap();
                if *last_access_datetime < cut_off_datetime {
                    expired_keys.push((key.clone(), session.clone()));
                }
            }
        }

        // 删除过期的会话
        if !expired_keys.is_empty() {
            tokio::spawn(async move {
                for (key, session) in expired_keys {
                    {
                        let mut write_guard = sessions.write().unwrap();
                        write_guard.remove(&key);
                    }
                    // 在独立任务中杀掉子进程，避免阻塞清理过程
                    // let _ = session.child.kill().await;
                    let child = Arc::try_unwrap(session.child).unwrap();
                    let _ = kill_process(child).await;
                }
            });
        }
    }
}
