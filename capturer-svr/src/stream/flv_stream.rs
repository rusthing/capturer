use async_stream::stream;
use bytes::Bytes;
use futures::Stream;
use log::{debug, warn};
use robotech::svc::svc_error::SvcError;
use std::sync::{Arc, OnceLock};
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

    /// 将 FlvStream 转换为异步流
    pub fn into_stream(self) -> impl Stream<Item = Result<Bytes, SvcError>> {
        stream! {
            let mut this = self;

            // 处理初始头部数据
            if this.is_init {
                if let Some(header) = this.header.get() {
                    debug!("要拉取的流已经存在，直接将缓存头部写入输出流: {:?}", header);
                    this.is_init = false;
                    yield Ok(header.clone());
                }
            }

            // 持续接收数据
            loop {
                match this.data_receiver.recv().await {
                    Ok(bytes) => {
                        if this.is_init {
                            this.is_init = false;
                            if let Some(cache_header_sender) = this.cache_header_sender.take() {
                                let _ = cache_header_sender.send(bytes.clone());
                            }
                        }
                        yield Ok(bytes);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("发送端关闭，结束流");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        warn!("消息滞后，跳过并继续接收");
                        continue;
                    }
                }
            }
        }
    }
}
