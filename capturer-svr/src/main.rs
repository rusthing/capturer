use capturer_svr::config::app_config::AppConfig;
use capturer_svr::config::capturer_config::{init_capturer_config, update_capturer_config};
use capturer_svr::stream::stream_manager::{init_stream_manager, update_stream_manager};
use clap::Parser;
use oss_api_client::api_client::{init_oss_api_client, update_oss_api_client};
use robotech::app::{wait_app_exit, AppWatcher};
use robotech::env::init_env;
use robotech::log::LogWatcher;
use robotech::macros::log_call;
use robotech::signal::SignalManager;
use robotech::web::{start_web_server, stop_web_service};
use std::sync::Arc;
use tracing::info;

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

    /// 监听信号，支持指令如下:
    /// * `start` - 默认值，先发送`SIGCONT`信号(kill -0)，检查程序是否已运行(如果程序已运行，会报错)，然后启动程序
    /// * `restart` - 先发送`SIGTERM`信号(kill -15)，如果旧程序已运行，收到信号后会停止运行，然后启动新程序
    /// * `stop`/`s` - 发送`SIGTERM`信号(kill -15)，用于终止程序，优雅退出
    /// * `kill`/`k` - 发送`SIGKILL`信号(kill -9)，用于强制终止程序
    #[arg(
        short,
        long,
        default_value = "start",
        long_help = r#"监听信号，支持指令如下:
    start - 默认值，先发送 SIGCONT 信号(kill -0)，检查程序是否已运行(如果程序已运行，会报错)，然后启动程序
    restart - 先发送 SIGTERM 信号(kill -15)，如果旧程序已运行，收到信号后会停止运行，然后启动新程序
    stop/s - 发送 SIGTERM 信号(kill -15)，用于终止程序，优雅退出
    kill/k - 发送 SIGKILL 信号(kill -9)，用于强制终止程序"#
    )]
    signal: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let Args {
        signal,
        config_file,
        port,
    } = Args::parse();

    // 初始化环境变量;
    init_env()?;
    // 初始化日志系统
    let log_watcher = LogWatcher::new().await?;

    // 初始化信号(_signal_manager变量将在程序优雅退出时释放，释放时删除pid文件)
    let (mut signal_manager, old_pid) = SignalManager::new(signal)?;

    let app_watcher: AppWatcher<AppConfig> = AppWatcher::new(
        config_file,
        log_watcher.config_changed_tx.clone(),
        move |app_config: Arc<AppConfig>| async move {
            // 更新capturer的配置
            update_capturer_config(app_config.capturer.clone())?;
            // 更新oss的API客户端的配置
            update_oss_api_client(app_config.api_client.clone())?;
            // 更新流管理器
            update_stream_manager(app_config.capturer.clone())?;

            // 应用配置
            apply_app_config(app_config, port, None)
                .await
                .expect("配置无法应用");
            info!("重新加载配置成功");
            Ok(())
        },
    )
    .await?;

    // 初始化capturer的配置
    init_capturer_config(app_watcher.app_config.capturer.clone())?;
    // 初始化oss的API客户端的配置
    init_oss_api_client(app_watcher.app_config.api_client.clone())?;
    // 初始化流管理器
    init_stream_manager(app_watcher.app_config.capturer.clone())?;

    // 应用配置
    apply_app_config(app_watcher.app_config.clone(), port, old_pid).await?;

    // 监听系统信号与等待退出
    let signal_receiver = signal_manager.watch_signal()?;
    Ok(wait_app_exit(signal_receiver, || async move {
        stop_web_service().await.expect("无法停止旧的Web服务");
        Ok(())
    })
    .await?)
}

///
/// # 应用配置
///
/// ## Arguments
/// * `port` - 一个可选的u16值，指定Web服务器监听的端口。如果未指定，则使用配置文件中的设置或默认值。
/// * `old_pid` - 一个可选的i32值，代表旧进程ID，用于在重启时清理资源等操作。
///
/// ## Functionality
/// 1. 加载并构建应用配置信息。
/// 2. 将配置信息保存到全局上下文中以供其他部分访问。
/// 3. 根据配置中的数据库设置执行数据库迁移以确保数据库结构是最新的。
/// 4. 初始化ID生成器，可能用于生成全局唯一ID。
/// 5. 建立与数据库的连接。
/// 6. 使用提供的或默认的端口号启动Web服务器，并处理任何给定的旧进程ID。
///
/// ## Errors
/// 如果在升级数据库版本时遇到问题，将打印错误信息并终止程序执行。
///
/// ## Examples
/// ```ignore
/// // 使用默认配置和端口初始化配置
/// init_config(None, None, None).await;
///
/// // 指定配置文件路径、自定义端口和旧进程ID来初始化配置
/// init_config(Some(String::from("path/to/app.toml")), Some(8080), Some(1234)).await;
/// ```
///
#[log_call]
pub async fn apply_app_config(
    app_config: Arc<AppConfig>,
    port: Option<u16>,
    old_pid: Option<u32>,
) -> anyhow::Result<()> {
    info!("应用App配置...");
    let AppConfig {
        web_server: web_server_config,
        ..
    } = AppConfig::clone(&app_config);

    // 启动Web服务器
    start_web_server(web_server_config, port, old_pid).await?;

    Ok(())
}
