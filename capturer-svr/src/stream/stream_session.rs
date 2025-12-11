use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct StreamSession {
    pub last_access_datetime: Arc<RwLock<DateTime<Utc>>>,
    pub sender: Arc<RwLock<Sender<Bytes>>>,
}
