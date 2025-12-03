#[cfg(test)]
mod tests {
    use log::info;
    use robotech::env::init_env;
    use robotech::log::log::init_log;
    use capturer_svr::ffmpeg::ffmpeg_cmd::FfmpegCmd;
    use tokio;

    #[tokio::test]
    async fn test_probe_stream_info_with_valid_data() {
        init_env();
        init_log().unwrap();

        let result = FfmpegCmd::probe_stream_info(
            "rtsp://admin:gssxt456@192.168.1.100:554/h264/ch1/main/av_stream",
        )
        .await;

        match result {
            Ok(metadata) => {
                info!("探测到流的信息: {:?}", metadata);
            }
            Err(e) => {
                info!("Error: {:?}", e);
            }
        }
    }
}
