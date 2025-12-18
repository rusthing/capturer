use crate::ffmpeg::ffmpeg_eo::{AudioCodecType, FfprobeCmdInfo, StreamMetadata, VideoCodecType};
use crate::ffmpeg::ffmpeg_error::FfmpegError;
use bytes::Bytes;
use log::{debug, info};
use tokio::process::Child;
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;
use wheel_rs::cmd;

/// ffmpeg命令执行模块
///
/// 该模块提供了基于ffmpeg工具的视频流处理功能，包括：
/// - RTSP流媒体信息探测
/// - 视频流转码和拉流
/// - 视频帧抓拍为JPEG图片
pub struct FfmpegCmd {}

impl FfmpegCmd {
    /// # 探测流信息（编码格式、分辨率等）
    ///
    /// 使用ffprobe工具探测流的基本信息，包括：
    /// - 视频编码格式（H.264/H.265等）
    /// - 分辨率（宽高）
    /// - 帧率
    ///
    /// ## 参数
    /// * `stream_url` - 探测流的地址
    ///
    /// ## 返回值
    /// 返回包含流媒体元数据的Result
    pub async fn probe_stream_info(stream_url: &str) -> Result<StreamMetadata, FfmpegError> {
        info!("probe_stream_info {stream_url}....");
        let stdout = cmd::std::execute(
            "ffprobe",
            &[
                "-v",            // 设置ffprobe的日志级别参数
                "error",         // 日志级别为error，只显示错误信息
                "-show_streams", // 显示输入文件的所有流信息
                "-show_entries", // 指定要显示的流条目参数
                "stream=codec_type,codec_name,width,height,r_frame_rate,sample_rate", // 指定要显示的具体字段：编码类型、编码名称、宽度、高度、帧率、采样率
                "-of",      // 设置输出格式参数
                "json",     // 输出格式为JSON，便于程序解析
                stream_url, // 输入的RTSP流地址
            ],
        )?;

        let stdout = String::from_utf8(stdout).map_err(|e| FfmpegError::FfprobeParseUtf8Fail(e))?;
        debug!("运行ffprobe命令成功: {}", stdout);
        let stdout: FfprobeCmdInfo =
            serde_json::from_str(&stdout).map_err(|e| FfmpegError::FfprobeParseJsonFail(e))?;
        let streams = stdout.streams;
        let mut stream_metadata = StreamMetadata::default();
        // 遍历所有流，分别处理视频和音频流
        for stream in &streams {
            if stream.codec_type == "video" {
                let codec_name = stream.codec_name.clone().ok_or_else(|| {
                    FfmpegError::FfprobeParseFail("缺少codec_name字段".to_string())
                })?;
                stream_metadata.video_codec = Some(match codec_name.as_str() {
                    "h264" => VideoCodecType::H264,
                    "hevc" => VideoCodecType::H265,
                    codec_name_str => VideoCodecType::Other(codec_name_str.to_string()),
                });

                stream_metadata.width = stream.width;
                stream_metadata.height = stream.height;

                stream_metadata.fps = if let Some(r_frame_rate) = &stream.r_frame_rate {
                    if let Some(pos) = r_frame_rate.find('/') {
                        let numerator_str = &r_frame_rate[..pos];
                        let denominator_str = &r_frame_rate[pos + 1..];

                        match (numerator_str.parse::<u8>(), denominator_str.parse::<u8>()) {
                            (Ok(num), Ok(den)) if den != 0 => Some(num / den),
                            _ => {
                                debug!("无效的帧率格式: {}", r_frame_rate);
                                None
                            }
                        }
                    } else {
                        debug!("帧率格式不包含'/': {}", r_frame_rate);
                        None
                    }
                } else {
                    None
                }
            } else if stream.codec_type == "audio" {
                stream_metadata.audio_codec =
                    Some(if let Some(codec_name) = stream.codec_name.clone() {
                        match codec_name.as_str() {
                            "aac" => AudioCodecType::AAC,
                            "mp2" => AudioCodecType::MP2,
                            "mp3" => AudioCodecType::MP3,
                            "pcm_mulaw" => AudioCodecType::G711mulaw,
                            "pcm_alaw" => AudioCodecType::G711alaw,
                            "adpcm_g726le" => AudioCodecType::G726,
                            codec_name_str => AudioCodecType::Other(codec_name_str.to_string()),
                        }
                    } else {
                        AudioCodecType::Unknown
                    });
                stream_metadata.sample_rate = stream
                    .sample_rate
                    .as_ref()
                    .and_then(|s| s.parse::<u32>().ok());
            }
        }

        Ok(stream_metadata)
    }

    /// # 抓拍单帧为 JPEG
    ///
    /// 从RTSP流中抓取单帧画面并编码为JPEG格式图片。
    ///
    /// ## 参数
    /// * `stream_url` - 抓拍流的地址
    ///
    /// ## 返回值
    /// 返回包含JPEG图片数据的字节数组
    pub async fn capture_to_jpeg(
        stream_url: &str,
        jpeg_quality: u8,
    ) -> Result<Vec<u8>, FfmpegError> {
        info!("capture_to_jpeg {stream_url}....");
        let jpeg_quality = &jpeg_quality.to_string();
        Ok(cmd::std::execute(
            "ffmpeg",
            &[
                "-rtsp_transport", // 设置RTSP传输方式参数
                "tcp",             // 使用TCP协议传输（更稳定）
                "-i",              // 指定输入源参数
                stream_url,        // 输入的RTSP流地址
                "-vframes",        // 设置要输出的视频帧数参数
                "1",               // 只抓取一帧画面
                "-f",              // 指定输出格式参数
                "image2pipe",      // 图像格式（JPEG、PNG等通用图像格式容器）
                "-c:v",            // 设置视频编解码器参数
                "mjpeg",           // 使用MJPEG编码
                "-q:v",            // 设置视频质量参数
                jpeg_quality,      // JPEG质量等级，1-31，数值越小质量越高
                "pipe:1",          // 输出到标准输出管道
            ],
        )?)
    }

    /// # 拉流转码（智能转码：H.265 转 H.264，H.264 直通）
    ///
    /// 从流拉取视频数据，并根据编码格式进行智能转码处理：
    /// - H.264编码：直接透传，不进行转码以提高性能
    /// - H.265编码：转码为H.264以保证兼容性
    /// - 其他编码：转码为H.264以保证兼容性
    ///
    /// ## 参数
    /// * `stream_url` - 拉流的地址
    /// * `frame_tx` - 用于发送视频帧数据的通道
    /// * `metadata` - 用于存储流媒体元数据的共享引用
    ///
    /// ## 返回值
    /// 返回ffmpeg子进程的句柄
    pub async fn pull_and_transcode_stream(
        stream_url: &str,
        data_sender: Sender<Bytes>,
        process_exit_sender: oneshot::Sender<()>,
        read_buffer_size: Option<usize>,
    ) -> Result<Child, FfmpegError> {
        info!("pull_and_transcode_stream {stream_url}....");
        // 先探测流信息
        let stream_metadata = Self::probe_stream_info(stream_url).await?;

        // 构建基础参数
        let mut ffmpeg_args = vec![
            "-rtsp_transport", // 设置RTSP传输方式参数
            "tcp",             // 强制 TCP，防止丢包花屏
            "-i",              // 输入源参数
            stream_url,        // 输入的RTSP流地址
            "-f",              // 输出格式参数
            "flv",             // 输出格式必须为 flv
            "-flvflags",       // FLV 容器格式
            "no_duration_filesize", // 指示 ffmpeg 在输出 FLV 文件时不计算和写入文件的总时长(duration)和大小(filesize)到 FLV 的头部信息中
                                    // "-g",                   // 关键帧间隔参数
                                    // "25",                   // 关键帧间隔为 25 帧（每 25 帧插入一个关键帧）
        ];

        // 根据编码类型添加特定参数
        match stream_metadata
            .video_codec
            .ok_or_else(|| FfmpegError::FfprobeParseFail("未发现视频编解码器".to_string()))?
        {
            // H.264 直通，不转码（性能最优）
            VideoCodecType::H264 => {
                ffmpeg_args.extend_from_slice(&[
                    "-c:v", // 视频编解码器设置参数
                    "copy", // 直通，不转码
                ]);
            }
            // H.265或未知编码使用H.264转码
            VideoCodecType::H265 | VideoCodecType::Other(_) => {
                ffmpeg_args.extend_from_slice(&[
                    "-c:v",        // 视频编解码器设置参数
                    "libx264",     // 使用H.264编码(flv 需要)
                    "-preset",     // 编码预设参数
                    "superfast",   // 超快速编码（低延迟，较低压缩率，比ultrafast稍慢但CPU占用更低）
                    "-tune",       // 编码调优参数
                    "zerolatency", // 零延迟调优
                    "-crf",        // 码率控制参数
                    "32",          // 码率控制等级，范围0-51，数值越小质量越高
                    "-profile:v",  // 编码档次
                    "baseline",    // baseline档次，编码复杂度最低
                    "-threads",    // 线程数
                    "1",           // 限制线程数以减少CPU占用
                ]);
            }
        }

        match stream_metadata.audio_codec {
            Some(AudioCodecType::AAC) => {
                ffmpeg_args.extend_from_slice(&[
                    "-c:a", // 音频编解码器设置参数
                    "copy", // 直通，不转码
                ]);
            }
            Some(AudioCodecType::MP3) => {
                if let Some(sample_rate) = stream_metadata.sample_rate
                    && ([44100, 22050, 11025].contains(&sample_rate))
                {
                    ffmpeg_args.extend_from_slice(&[
                        "-c:a", // 音频编解码器设置参数
                        "copy", // 直通，不转码
                    ]);
                } else {
                    ffmpeg_args.extend_from_slice(&[
                        "-c:a", // 音频编解码器设置参数
                        "aac",  // 音频转为 aac (flv 需要)
                    ]);
                }
            }
            Some(AudioCodecType::Unknown | AudioCodecType::NotSupported(_)) | None => {
                ffmpeg_args.extend_from_slice(&[
                    "-an", // 禁用音频流，不处理也不输出任何音频
                ]);
            }
            Some(_) => {
                ffmpeg_args.extend_from_slice(&[
                    "-c:a", // 音频编解码器设置参数
                    "aac",  // 音频转为 aac (flv 需要)
                ]);
            }
        }

        ffmpeg_args.extend_from_slice(&[
            "pipe:1", // 输出到标准输出管道
        ]);

        // 执行ffmpeg命令
        Ok(cmd::spawn::execute(
            "ffmpeg",
            &ffmpeg_args,
            data_sender,
            process_exit_sender,
            read_buffer_size,
        )?)
    }
}
