# Capturer Server

基于 Rust 的视频抓拍服务，可以从视频流中抓取图像，支持 RTSP 和其他视频格式。

## 功能特性

- 从视频流中抓取图像
- 支持多种视频格式（RTSP、MP4 等）
- RESTful API，便于集成
- 基于 FFmpeg 的视频处理
- FLV 流媒体支持
- 可配置的服务器设置
- 对象存储服务（OSS）集成

## 环境要求

- Rust 1.84+（2024 版）
- 系统已安装 FFmpeg
- OpenSSL 用于安全连接

## 安装

### 克隆仓库

```bash
git clone https://github.com/rusthing/capturer.git
cd capturer
```

### 构建项目

```bash
cargo build --release
```

### 使用 Docker

项目包含用于容器化部署的 Dockerfile：

```bash
# 构建 Docker 镜像
docker build -t rusthing/capturer .

# 运行容器
docker run -d -p 8080:8080 rusthing/capturer
```

## 配置

应用程序使用名为 `capturer-svr.toml` 的配置文件。您可以使用 `--config-file` 命令行选项指定自定义配置文件路径。

示例配置：

```toml
[web_server]
port = 8080
host = "0.0.0.0"

[api_client]
# OSS 集成的 API 客户端配置
```

## 使用方法

### 命令行选项

```bash
./capturer-svr --help
```

可用选项：
- `-c, --config-file`: 配置文件路径
- `-p, --port`: Web 服务器端口号
- `-V, --version`: 显示版本信息

### 运行服务器

```bash
# 使用默认设置
./capturer-svr

# 使用自定义端口
./capturer-svr --port 8080

# 使用自定义配置文件
./capturer-svr --config-file /path/to/config.toml
```

## API 文档

服务提供 Swagger UI 文档，在运行时可通过 `/swagger-ui/` 端点访问。

## 架构

项目组织为以下模块：

- `api_doc`: API 文档配置
- `config`: 应用配置管理
- `ctrl`: 控制器，用于处理 HTTP 请求
- `dto`: 数据传输对象
- `ffmpeg`: 使用 FFmpeg 进行视频处理
- `stream`: 流管理（FLV 流）
- `svc`: 业务逻辑服务
- `vo`: 值对象

## FFmpeg 集成

应用程序利用 FFmpeg 进行视频处理和图像提取。包括：

- 视频流抓取
- 从视频帧中提取图像
- 格式转换功能
- 并发操作的会话管理

## 测试

运行项目测试：

```bash
cargo test
```

某些测试可能需要位于 `tests/static/` 目录中的特定测试文件。

## 贡献

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 进行修改
4. 运行测试 (`cargo test`)
5. 提交更改 (`git commit -m 'Add amazing feature'`)
6. 推送到分支 (`git push origin feature/amazing-feature`)
7. 开启 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

- 基于 [Actix Web](https://actix.rs/) 构建 - 一款强大、实用且极快的 Rust Web 框架
- 使用 [FFmpeg](https://ffmpeg.org/) 进行视频处理
- 文档由 [Utoipa](https://github.com/juhaku/utoipa) 提供支持，用于 OpenAPI 生成