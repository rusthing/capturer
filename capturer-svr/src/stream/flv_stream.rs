use bytes::Bytes;
use futures::Stream;
use robotech::svc::svc_error::SvcError;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

pub struct FlvStream {
    data_receiver: Receiver<Bytes>,
    header: Arc<OnceLock<Bytes>>,
    is_init: bool,
}

impl FlvStream {
    pub fn new(data_receiver: Receiver<Bytes>, header: Arc<OnceLock<Bytes>>) -> Self {
        Self {
            data_receiver,
            header,
            is_init: true,
        }
    }
}

impl Stream for FlvStream {
    type Item = Result<Bytes, SvcError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let is_init = self.is_init;
        let header = self.header.clone();
        let this = self.get_mut();
        if is_init {
            this.is_init = false;
            if let Some(header) = header.get() {
                return Poll::Ready(Some(Ok(header.clone())));
            }
        }

        match this.data_receiver.try_recv() {
            Ok(bytes) => {
                if is_init {
                    let truncated_bytes = bytes.slice(0..13.min(bytes.len()));
                    let _ = header.set(truncated_bytes);
                }
                Poll::Ready(Some(Ok(bytes)))
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(_) => Poll::Ready(None),
        }
    }
}
