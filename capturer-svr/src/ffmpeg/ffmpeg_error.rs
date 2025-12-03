use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    #[error("ffprobe执行失败: {0}")]
    FfprobeExecuteFail(std::io::Error),
    #[error("ffprobe运行失败: {0}")]
    FfprobeRunFail(String),
    #[error("ffprobe运行时信息按utf8编码解析失败: {0}")]
    FfprobeParseUtf8Fail(FromUtf8Error),
    #[error("ffprobe运行时信息按json格式解析失败: {0}")]
    FfprobeParseJsonFail(serde_json::Error),
    #[error("解析ffprobe运行时信息失败: {0}")]
    FfprobeParseFail(String),
}
