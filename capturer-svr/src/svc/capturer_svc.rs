use crate::dto::capturer_dto::CapturerCaptureRtspToJpgDto;
use crate::settings::settings::SETTINGS;
use image::{ImageBuffer, Rgb};
use log::debug;
use oss_api::api::oss_api_utils::OSS_FILE_API;
use robotech::ro::ro_result::RoResult;
use robotech::ro::Ro;
use robotech::svc::svc_error::SvcError;
use std::io::Cursor;

pub struct CapturerSvc;

impl CapturerSvc {
    pub async fn capture_rtsp_to_jpg(
        dto: CapturerCaptureRtspToJpgDto,
    ) -> Result<Ro<serde_json::Value>, SvcError> {
        let mut input = ffmpeg_next::format::input(&dto.rtsp_url.unwrap())
            .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;
        let input_stream = input
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .expect("No video stream found");
        let video_stream_index = input_stream.index();

        let decoder_context =
            ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())
                .expect("Failed to create decoder context");
        let mut decoder = decoder_context
            .decoder()
            .video()
            .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

        // 创建缩放器上下文，使用明确的颜色空间参数
        let mut scaler = ffmpeg_next::software::scaling::context::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg_next::format::Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            ffmpeg_next::software::scaling::flag::Flags::BILINEAR,
        )
        .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

        for (stream, packet) in input.packets() {
            if stream.index() != video_stream_index || !packet.is_key() {
                continue;
            }

            decoder
                .send_packet(&packet)
                .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;
            let mut frame = ffmpeg_next::frame::Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                // 转换为 RGB24
                let mut rgb_frame = ffmpeg_next::frame::Video::empty();
                scaler
                    .run(&frame, &mut rgb_frame)
                    .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

                // 提取数据 (RGB24格式)
                let buffer = rgb_frame.data(0);
                let width = rgb_frame.width();
                let height = rgb_frame.height();
                // let stride = rgb_frame.stride(0);

                let img_buffer: ImageBuffer<Rgb<u8>, _> =
                    ImageBuffer::from_raw(width, height, buffer.to_owned())
                        .ok_or("Failed to create image buffer")
                        .map_err(|e| SvcError::RuntimeError(e.to_string()))?;

                let mut jpeg_bytes = Vec::new();
                image::codecs::jpeg::JpegEncoder::new(Cursor::new(&mut jpeg_bytes))
                    .encode_image(&img_buffer)
                    .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

                debug!("获取oss_file_api实例...");
                let oss_file_api = OSS_FILE_API.get().unwrap();

                let oss_file_api_ro = oss_file_api
                    .upload_file_content(
                        dto.bucket
                            .unwrap_or(SETTINGS.get().unwrap().capturer.bucket.clone())
                            .as_str(),
                        &format!("{}.jpg", chrono::Utc::now().timestamp_millis()),
                        jpeg_bytes,
                    )
                    .await?;

                return Ok(match oss_file_api_ro.result {
                    RoResult::Success => oss_file_api_ro.msg("抓拍成功".to_string()),
                    _ => {
                        let msg = oss_file_api_ro.msg.clone();
                        oss_file_api_ro.msg(format!("抓拍失败: {}", msg))
                    }
                });
            }
        }

        Err(SvcError::RuntimeError(
            "Failed to capture a frame".to_string(),
        ))
    }
}
