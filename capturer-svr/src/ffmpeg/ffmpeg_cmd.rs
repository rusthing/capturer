use crate::ffmpeg::ffmpeg_error::FfmpegError;
use crate::ffmpeg::ffmpeg_vo::{CodecType, FfprobeCmdInfo, StreamMetadata};
use clap::ValueHint::Unknown;
use log::debug;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};
use wheel_rs::cmd::cmd_utils::exec;

pub struct FfmpegCmd {}

impl FfmpegCmd {
    /// 探测流信息（编码格式、分辨率等）
    pub async fn probe_stream_info(rtsp_url: &str) -> Result<StreamMetadata, FfmpegError> {
        debug!("probe_stream_info: {}", rtsp_url);
        let stdout = exec(
            "ffprobe",
            &[
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=codec_name,width,height,r_frame_rate,bit_rate",
                "-of",
                "json",
                rtsp_url,
            ],
        )?;

        let stdout = String::from_utf8(stdout).map_err(|e| FfmpegError::FfprobeParseUtf8Fail(e))?;
        debug!("运行ffprobe命令成功: {}", stdout);
        let stdout: FfprobeCmdInfo =
            serde_json::from_str(&stdout).map_err(|e| FfmpegError::FfprobeParseJsonFail(e))?;
        let streams = stdout
            .streams
            .ok_or_else(|| FfmpegError::FfprobeParseFail("No streams found".to_string()))?;
        let stream = &streams[0];
        let codec_name = stream
            .codec_name
            .as_ref()
            .ok_or_else(|| FfmpegError::FfprobeParseFail("No codec_name found".to_string()))?
            .as_str();
        let codec = match codec_name {
            "h264" => CodecType::H264,
            "hevc" => CodecType::H265,
            codec_name_str => CodecType::Unknown(codec_name_str.to_string()),
        };

        let width = stream.width.unwrap_or_default();
        let height = stream.height.unwrap_or_default();

        let fps: u8 = if let Some(fps) = &stream.r_frame_rate {
            if let Some(pos) = fps.find('/') {
                let num: u8 = fps[..pos].parse().unwrap_or(0);
                let den: u8 = fps[pos + 1..].parse().unwrap_or(1);
                num / den
            } else {
                0
            }
        } else {
            0
        };

        Ok(StreamMetadata {
            codec,
            width,
            height,
            fps,
        })
    }

    /// 拉流（智能转码：H.265 转 H.264，H.264 直通）
    pub async fn pull_stream(
        rtsp_url: &str,
        frame_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
        metadata: Arc<RwLock<Option<StreamMetadata>>>,
    ) -> Result<Child, Box<dyn std::error::Error>> {
        // 先探测流信息
        let stream_metadata = Self::probe_stream_info(rtsp_url).await?;

        // 构建基础参数
        let mut ffmpeg_args = vec![
            "-rtsp_transport".to_string(),
            "tcp".to_string(),
            "-i".to_string(),
            rtsp_url.to_string(),
            "pipe:1".to_string(),
        ];

        // 根据编码类型添加特定参数
        match stream_metadata.codec {
            // H.264 直通，不转码（性能最优）
            CodecType::H264 => {
                ffmpeg_args.extend_from_slice(&[
                    "-c:v".to_string(), // 视频编解码器设置参数
                    "copy".to_string(), // 直通，不转码
                ]);
            }
            // H.265 转 H.264 或未知编码使用保险转码
            CodecType::H265 | CodecType::Unknown(_) => {
                ffmpeg_args.extend_from_slice(&[
                    "-c:v".to_string(),      // 视频编解码器设置参数
                    "libx264".to_string(),   // 使用H.264编码
                    "-preset".to_string(),   // 编码预设参数
                    "ultrafast".to_string(), // 超快速编码（低延迟，较低压缩率）
                ]);
            }
        }

        let mut child = Command::new("ffmpeg")
            .args(&ffmpeg_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        // 异步读取输出
        if let Some(mut stdout) = child.stdout.take() {
            let frame_tx_clone = frame_tx.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 65536];
                loop {
                    match std::io::Read::read(&mut stdout, &mut buffer) {
                        Ok(0) => {
                            log::warn!("ffmpeg stdout closed");
                            break;
                        }
                        Ok(n) => {
                            let _ = frame_tx_clone.send(buffer[..n].to_vec()).await;
                        }
                        Err(e) => {
                            log::error!("Read error: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        Ok(child)
    }

    /// 抓拍单帧为 JPEG
    pub async fn capture_to_jpeg(rtsp_url: &str) -> Result<Vec<u8>, FfmpegError> {
        debug!("capture_frame_as_jpeg: {}", rtsp_url);
        Ok(exec(
            "ffmpeg",
            &[
                "-rtsp_transport",
                "tcp",
                "-i",
                rtsp_url,
                "-vframes",
                "1",
                "-f",
                "image2",
                "-c:v",
                "mjpeg",
                "-q:v",
                "2", // 质量 1-31，越小越好
                "pipe:1",
            ],
        )?)
    }
}
