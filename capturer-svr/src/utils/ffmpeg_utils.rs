use ffmpeg_next::Error;
use log::info;

pub fn init_ffmpeg() -> Result<(), Error> {
    info!("初始化ffmpeg...");
    ffmpeg_next::init()
}
