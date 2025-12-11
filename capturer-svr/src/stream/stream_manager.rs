use crate::ffmpeg::ffmpeg_cmd::FfmpegCmd;
use crate::ffmpeg::ffmpeg_error::FfmpegError;
use crate::stream::stream_session::StreamSession;
use bytes::Bytes;
use chrono::Utc;
use rustc_hash::FxHashMap;
use std::sync::{Arc, LazyLock, RwLock};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

static STREAM_SESSIONS: LazyLock<Arc<RwLock<FxHashMap<String, StreamSession>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(FxHashMap::default())));

pub struct StreamManager {}

impl StreamManager {
    pub async fn get_receiver(url: &str) -> Result<Receiver<Bytes>, FfmpegError> {
        Ok(
            if let Some(stream_session) = STREAM_SESSIONS.read().unwrap().get(url) {
                *stream_session.last_access_datetime.write().unwrap() = Utc::now();
                stream_session.sender.read().unwrap().subscribe()
            } else {
                let (sender, receiver) = broadcast::channel(100);
                let mut write_guard = STREAM_SESSIONS.write().unwrap();
                let stream_session = StreamSession {
                    last_access_datetime: Arc::new(RwLock::new(Utc::now())),
                    sender: Arc::new(RwLock::new(sender.clone())),
                };
                write_guard.insert(url.to_string(), stream_session);

                FfmpegCmd::pull_and_transcode_stream(url, sender).await?;

                receiver
            },
        )
    }
}
