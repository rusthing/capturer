use bytes::Bytes;
use robotech::cfg::CfgError;
use std::io::Error;
use std::string::FromUtf8Error;
use tokio::sync::broadcast::error::SendError;
use wheel_rs::cmd::cmd_error::CmdError;

#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    #[error("执行ffprobe命令失败: {0}")]
    FfprobeCmd(#[from] CmdError),
    #[error("执行ffprobe后按utf8编码解析信息失败: {0}")]
    FfprobeParseUtf8(FromUtf8Error),
    #[error("执行ffprobe后按json格式解析信息失败: {0}")]
    FfprobeParseJson(serde_json::Error),
    #[error("执行ffprobe后解析信息失败: {0}")]
    FfprobeParse(String),
    #[error("获取ffmpeg配置失败: {0}")]
    FfmpegConfig(#[from] CfgError),
    #[error("执行ffmpeg后获取stdout失败: {0}")]
    FfmpegTakeStdout(String),
    #[error("关闭ffmpeg失败: {0}")]
    FfmpegKill(Error),
    #[error("ffmpeg发送数据失败: {0}")]
    FfmpegSend(SendError<Bytes>),
    #[error("读取ffmpeg会话失败: {0}")]
    FfmpegSessionRead(String),
}
