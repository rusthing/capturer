use crate::ffmpeg::ffmpeg_error::FfmpegError;
use crate::ffmpeg::ffmpeg_vo::{CodecType, FfprobeCmdInfo, StreamMetadata};
use log::debug;
use std::process::Command;

pub struct FfmpegCmd {}

impl FfmpegCmd {
    /// 探测流信息（编码格式、分辨率等）
    pub async fn probe_stream_info(rtsp_url: &str) -> Result<StreamMetadata, FfmpegError> {
        debug!("probe_stream_info: {}", rtsp_url);
        debug!("运行ffprobe命令....");
        let output = Command::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=codec_name,width,height,r_frame_rate,bit_rate",
                "-of",
                "json",
                rtsp_url,
            ])
            .output()
            .map_err(|e| FfmpegError::FfprobeExecuteFail(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(FfmpegError::FfprobeRunFail(stderr));
        }
        let stdout =
            String::from_utf8(output.stdout).map_err(|e| FfmpegError::FfprobeParseUtf8Fail(e))?;
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
}
