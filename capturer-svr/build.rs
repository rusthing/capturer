use std::fs;
use std::path::Path;

/// # 该函数将配置文件从源位置复制到目标目录中
///
/// 主要用于构建过程中确保配置文件被正确地包含在输出目录里
///
/// ## Panics
/// - 当环境变量"OUT_DIR"不存在时会panic
/// - 当路径操作失败时会panic
/// - 当文件复制失败时会panic
fn main() {
    // println!("cargo:warning=🔍 正在运行 build.rs");
    // 获取输出目录路径
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // 复制配置文件到输出目录
    copy_config_file(&out_dir, "toml");
    copy_config_file(&out_dir, "yml");
    copy_config_file(&out_dir, "json");
    copy_config_file(&out_dir, "ini");
    copy_config_file(&out_dir, "ron");
}

/// 复制指定扩展名的配置文件到输出目录
///
/// 该函数会查找与当前包同名的配置文件（如 `capturer-svr.toml`），
/// 并将其从项目根目录复制到构建输出目录中。
///
/// # 参数
///
/// - `out_dir`: 构建输出目录路径
/// - `file_ext`: 配置文件的扩展名（如 "toml", "yml" 等）
///
/// # 行为
///
/// 1. 构造配置文件名：`{包名}.{扩展名}`
/// 2. 在项目根目录查找该配置文件
/// 3. 计算目标路径（输出目录向上三级目录）
/// 4. 如果源文件存在，则复制到目标路径
///
/// # Panics
///
/// - 当无法访问环境变量时
/// - 当路径操作失败时
/// - 当文件复制失败时
fn copy_config_file(out_dir: &str, file_ext: &str) {
    // 获取源配置文件路径
    let config_file_name = format!("{}.{}", env!("CARGO_PKG_NAME"), file_ext);
    let project_root = env!("CARGO_MANIFEST_DIR");
    let config_file_path = Path::new(project_root).join(&config_file_name);

    // 构造目标文件路径，通过向上回溯OUT_DIR的父级目录来定位
    let dest_path = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .unwrap()
        .join(&config_file_name);

    // 如果源配置文件存在，则执行复制操作
    if config_file_path.exists() {
        // println!(
        //     "cargo:warning=copy {:?} to {:?}",
        //     config_file_path, dest_path
        // );
        fs::copy(config_file_path, dest_path).expect("Failed to copy config file");
    }
}
