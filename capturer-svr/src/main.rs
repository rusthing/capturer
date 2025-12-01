use capturer_svr::settings::settings::{init_settings, SETTINGS};
use capturer_svr::web_service_config::web_service_config;
use clap::Parser;
use log::info;
use oss_api::api::oss_api_utils::init_oss_api;
use robotech::env::init_env;
use robotech::log::log::init_log;
use robotech::web_server::start_web_server;

/// 视频抓拍工具
///
/// SUMMARY: 这是一个用于视频抓拍的工具，可以将视频进行抓拍并返回图片
///
#[derive(Parser, Debug)]
// 命令行参数使用定义
// version: 命令行添加 -V/--version参数可以查看版本信息
// about: --help命令第一行显示文档注释的内容
// long_about = None: 只显示文档注释的第一行(包括about的和arg的)
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version,
    about,
    help_template = "{name} v{version} - {about}\n\nAUTHOR: {author}\n\nUSAGE: {usage}\n\nOPTIONS:\n{options}"
)]
struct Args {
    /// 配置文件的路径
    #[arg(short, long)]
    config_file: Option<String>,

    /// Web服务器的端口号
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    info!("程序正在启动……");

    info!("初始化环境变量...");
    init_env();

    info!("初始化日志系统...");
    init_log().unwrap();

    info!("解析命令行参数...");
    let args = Args::parse();

    info!("初始化设置选项...");
    init_settings(args.config_file, args.port);

    info!("初始化API...");
    let api_settings = SETTINGS.get().unwrap().api.clone();
    init_oss_api(api_settings);

    // 启动Web服务
    let web_server_settings = SETTINGS.get().unwrap().web_server.clone();
    start_web_server(web_server_settings, web_service_config).await;

    info!("退出程序");
}
