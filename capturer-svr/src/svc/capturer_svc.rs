use crate::dto::capturer_dto::CapturerCaptureToJpegDto;
use crate::ffmpeg::ffmpeg_cmd::FfmpegCmd;
use crate::settings::settings::SETTINGS;
use log::debug;
use oss_api::api::oss_api_utils::OSS_FILE_API;
use robotech::ro::Ro;
use robotech::ro::RoResult;
use robotech::svc::svc_error::SvcError;
use wheel_rs::runtime::Error::RuntimeXError;

pub struct CapturerSvc;

impl CapturerSvc {
    pub async fn capture_to_jpeg(
        dto: CapturerCaptureToJpegDto,
    ) -> Result<Ro<serde_json::Value>, SvcError> {
        let jpeg_bytes = FfmpegCmd::capture_to_jpeg(dto.stream_url.unwrap().as_str())
            .await
            .map_err(|e| RuntimeXError("抓拍异常".to_string(), Box::new(e)))?;

        debug!("获取oss_file_api实例...");
        let oss_file_api = OSS_FILE_API.get().unwrap();
        let oss_file_api_ro = oss_file_api
            .upload_file_content(
                dto.bucket
                    .unwrap_or(SETTINGS.get().unwrap().capturer.bucket.clone())
                    .as_str(),
                &format!("{}.jpg", chrono::Utc::now().timestamp_millis()),
                jpeg_bytes,
                dto.current_user_id,
            )
            .await?;

        Ok(match oss_file_api_ro.result {
            RoResult::Success => oss_file_api_ro.msg("抓拍成功".to_string()),
            _ => {
                let msg = oss_file_api_ro.msg.clone();
                oss_file_api_ro.msg(format!("抓拍失败: {}", msg))
            }
        })
    }
}
