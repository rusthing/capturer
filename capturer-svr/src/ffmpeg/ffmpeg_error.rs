use std::string::FromUtf8Error;
use wheel_rs::cmd::cmd_error::CmdError;

#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    #[error("ffprobe执行命令失败: {0}")]
    FfprobeCmdError(#[from] CmdError),
    #[error("ffprobe运行时信息按utf8编码解析失败: {0}")]
    FfprobeParseUtf8Fail(FromUtf8Error),
    #[error("ffprobe运行时信息按json格式解析失败: {0}")]
    FfprobeParseJsonFail(serde_json::Error),
    #[error("解析ffprobe运行时信息失败: {0}")]
    FfprobeParseFail(String),
}
