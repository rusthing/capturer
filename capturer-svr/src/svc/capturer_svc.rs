use crate::config::app_config::APP_CONFIG;
use crate::config::capturer_config::OssConfig;
use crate::dto::capturer_dto::{CapturerCaptureToJpegDto, CapturerGetStreamDto};
use crate::ffmpeg::ffmpeg_cmd::FfmpegCmd;
use crate::stream::flv_stream::FlvStream;
use crate::stream::stream_manager::STREAM_MANAGER;
use futures::Stream;
use log::debug;
use oss_api_client::api_client::OSS_FILE_API_CLIENT;
use robotech::ro::Ro;
use robotech::ro::RoResult;
use robotech::svc::svc_error::SvcError;
use std::sync::Arc;
use wheel_rs::runtime::Error::RuntimeXError;

pub struct CapturerSvc;

impl CapturerSvc {
    pub async fn capture_to_jpeg(
        dto: CapturerCaptureToJpegDto,
    ) -> Result<Ro<serde_json::Value>, SvcError> {
        let OssConfig {
            jpeg_quality,
            bucket,
        } = APP_CONFIG.get().unwrap().capturer.oss.clone();
        let jpeg_bytes = FfmpegCmd::capture_to_jpeg(dto.stream_url.unwrap().as_str(), jpeg_quality)
            .await
            .map_err(|e| RuntimeXError("抓拍异常".to_string(), Box::new(e)))?;

        debug!("获取oss_file_api实例...");
        let oss_file_api_client = OSS_FILE_API_CLIENT.get().unwrap();
        let oss_file_api_ro = oss_file_api_client
            .upload_file_content(
                dto.bucket.unwrap_or(bucket).as_str(),
                &format!("{}.jpg", chrono::Utc::now().timestamp_millis()),
                jpeg_bytes,
                dto.current_user_id,
            )
            .await?;

        Ok(if let RoResult::Success = oss_file_api_ro.result {
            oss_file_api_ro.msg("抓拍成功".to_string())
        } else {
            let msg = oss_file_api_ro.msg.clone();
            oss_file_api_ro.msg(format!("抓拍失败: {}", msg))
        })
    }

    pub async fn stream(
        dto: CapturerGetStreamDto,
    ) -> Result<impl Stream<Item = Result<bytes::Bytes, SvcError>>, SvcError> {
        debug!("获取stream_manager实例...");
        let (data_receiver, header, cache_header_sender) = STREAM_MANAGER
            .get_cmd_receiver(dto.stream_url.unwrap().as_str())
            .await
            .map_err(|e| RuntimeXError("获取流异常".to_string(), Box::new(e)))?;
        debug!("获取flv_stream实例...");
        let flv_stream = FlvStream::new(data_receiver, Arc::clone(&header), cache_header_sender);
        debug!("返回flv_stream...");
        Ok(flv_stream.into_stream())
    }
}
