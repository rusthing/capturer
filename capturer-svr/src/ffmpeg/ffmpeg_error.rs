use std::io::Error;
use std::string::FromUtf8Error;
use wheel_rs::cmd::cmd_error::CmdError;

#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    #[error("执行ffprobe命令失败: {0}")]
    FfprobeCmdError(#[from] CmdError),
    #[error("执行ffprobe后按utf8编码解析信息失败: {0}")]
    FfprobeParseUtf8Fail(FromUtf8Error),
    #[error("执行ffprobe后按json格式解析信息失败: {0}")]
    FfprobeParseJsonFail(serde_json::Error),
    #[error("执行ffprobe后解析信息失败: {0}")]
    FfprobeParseFail(String),
    #[error("执行ffmpeg后获取stdout失败: {0}")]
    FfmpegTakeStdoutError(String),
    #[error("关闭ffmpeg失败: {0}")]
    FfmpegKillError(Error),
}
