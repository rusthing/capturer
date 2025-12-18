use bytes::Bytes;
use futures::Stream;
use log::debug;
use robotech::svc::svc_error::SvcError;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, oneshot};

pub struct FlvStream {
    data_receiver: Receiver<Bytes>,
    header: Arc<OnceLock<Bytes>>,
    cache_header_sender: Option<oneshot::Sender<Bytes>>,
    is_init: bool,
}

impl FlvStream {
    pub fn new(
        data_receiver: Receiver<Bytes>,
        header: Arc<OnceLock<Bytes>>,
        cache_header_sender: Option<oneshot::Sender<Bytes>>,
    ) -> Self {
        Self {
            data_receiver,
            header,
            cache_header_sender,
            is_init: true,
        }
    }
}

impl Stream for FlvStream {
    type Item = Result<Bytes, SvcError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let is_init = self.is_init;
        let header = Arc::clone(&self.header);
        let this = self.get_mut();
        if is_init {
            if let Some(header) = header.get() {
                debug!("要拉取的流已经存在，直接将缓存头部写入输出流: {:?}", header);
                this.is_init = false;
                return Poll::Ready(Some(Ok(header.clone())));
            }
        }

        match this.data_receiver.try_recv() {
            Ok(bytes) => {
                if is_init {
                    this.is_init = false;
                    if let Some(cache_header_sender) = this.cache_header_sender.take() {
                        let _ = cache_header_sender.send(bytes.clone());
                    }
                }
                Poll::Ready(Some(Ok(bytes)))
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                // 注册唤醒器，等待数据到达时被系统自动唤醒
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(_) => Poll::Ready(None),
        }
    }
}
