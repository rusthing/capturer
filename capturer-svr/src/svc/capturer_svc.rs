use crate::dto::capturer_dto::CapturerCaptureRtspToJpgDto;
use crate::vo::capturer_vo::CapturerCaptureVo;
use image::{ImageBuffer, Rgb};
use robotech::ro::Ro;
use robotech::svc::svc_error::SvcError;
use std::io::Cursor;

pub struct CapturerSvc;

impl CapturerSvc {
    pub fn capture_rtsp_to_jpg(
        dto: CapturerCaptureRtspToJpgDto,
    ) -> Result<Ro<CapturerCaptureVo>, SvcError> {
        ffmpeg_next::init().map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

        let mut input_context = ffmpeg_next::format::input(&dto.url.unwrap())
            .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;
        let input = input_context
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .expect("No video stream found");
        let video_stream_index = input.index();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .expect("Failed to create decoder context");
        let mut decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

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

        for (_, packet) in input_context.packets() {
            // let packet = packet?;
            if packet.stream() != video_stream_index {
                continue;
            }

            decoder
                .send_packet(&packet)
                .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;
            let mut frame = ffmpeg_next::frame::Video::empty();
            if decoder.receive_frame(&mut frame).is_ok() {
                // 转换为 RGB24
                let mut rgb_frame = ffmpeg_next::frame::Video::empty();
                scaler
                    .run(&frame, &mut rgb_frame)
                    .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

                // 提取数据
                let width = rgb_frame.width();
                let height = rgb_frame.height();
                let data = rgb_frame.data(0);
                let stride = rgb_frame.stride(0);

                let mut img_buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);
                for y in 0..height {
                    for x in 0..width {
                        let base = (y as usize * stride) + (x as usize * 3);
                        let pixel = Rgb([data[base], data[base + 1], data[base + 2]]);
                        img_buffer.put_pixel(x, y, pixel);
                    }
                }

                let mut jpeg_bytes = Vec::new();
                let mut cursor = Cursor::new(&mut jpeg_bytes);
                image::codecs::jpeg::JpegEncoder::new(&mut cursor)
                    .encode(
                        img_buffer.as_raw(),
                        width,
                        height,
                        image::ColorType::Rgb8.into(),
                    )
                    .map_err(|e| SvcError::RuntimeXError(Box::new(e)))?;

                return Ok(Ro::success("抓拍成功".to_string())
                    .extra(Some(CapturerCaptureVo { data: jpeg_bytes })));
            }
        }

        Err(SvcError::RuntimeError(
            "Failed to capture a frame".to_string(),
        ))
    }
}
