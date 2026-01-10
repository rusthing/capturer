#[cfg(test)]
#[ctor::ctor]
fn init_tests() {
    robotech::env::init_env();
    robotech::log::init_log();
}

#[cfg(test)]
mod tests {
    use capturer_svr::ffmpeg::ffmpeg_cmd::FfmpegCmd;
    use log::info;
    use std::fs::File;
    use std::io::Write;
    use tokio;

    // const RTSP_URL: &str = "rtsp://admin:gssxt456@192.168.1.100:554/h264/ch1/main/av_stream";
    const RTSP_URL: &str = "rtsp://admin:lh123456789@221.7.253.103:40554";

    #[tokio::test]
    async fn test_probe_stream_info_with_valid_data() {
        let result = FfmpegCmd::probe_stream_info(RTSP_URL).await;

        match result {
            Ok(metadata) => {
                info!("探测到流的信息: {:?}", metadata);
            }
            Err(e) => {
                info!("Error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_capture_to_jpeg() {
        let result = FfmpegCmd::capture_to_jpeg(RTSP_URL, 2).await;

        match result {
            Ok(data) => {
                info!("抓拍图片成功，大小: {} 字节", data.len());
                // 验证返回的数据是否为有效的JPEG图像
                assert!(!data.is_empty());
                // JPEG文件通常以0xFFD8开头，以0xFFD9结尾
                assert_eq!(data[0], 0xFF);
                assert_eq!(data[1], 0xD8);
                assert_eq!(data[data.len() - 2], 0xFF);
                assert_eq!(data[data.len() - 1], 0xD9);

                // 将图片数据保存为文件
                let filename = "test_capture.jpg";
                let mut file = File::create(filename).expect("无法创建文件");
                file.write_all(&data).expect("无法写入文件");
                info!("图片已保存为: {}", filename);
            }
            Err(e) => {
                info!("抓拍失败: {:?}", e);
            }
        }
    }
}
