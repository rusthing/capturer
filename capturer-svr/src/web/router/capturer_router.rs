use robotech::macros::router;

#[router(routes[
    ("/capturer/capture_to_jpeg", post(capture_to_jpeg)),   // 抓拍图片
    ("/capturer/stream.live.flv", get(stream)),             // 直播视频流
])]
struct CapturerRouter;
