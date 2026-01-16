# Capturer Server 核心模块

这是 Capturer Server 应用的核心模块，负责使用 FFmpeg 进行视频抓拍功能。

## 概述

`capturer-svr` crate 提供了一个视频抓拍服务，可以从视频流中提取图像。它使用 Rust 构建，并使用 Actix Web 作为 HTTP 接口，FFmpeg 进行视频处理。

## 功能特性

- 从各种来源抓取视频流（RTSP、本地文件等）
- 从视频帧中提取图像
- 用于抓拍操作的 RESTful API 端点
- 与 FFmpeg 集成进行视频处理
- 用于并发操作的流管理
- 与 OSS（对象存储服务）集成以存储抓拍的图像
- FLV 流支持
- 通过 Utoipa 提供 API 文档

## 架构

该模块组织为以下组件：

### API 层
- `api_doc`: 使用 Utoipa 的 API 文档配置
- `ctrl`: 处理 HTTP 请求和路由的控制器
- `dto`: 请求/响应数据的数据传输对象
- `vo`: 表示业务实体的值对象

### 服务层
- `svc`: 业务逻辑服务
- `stream`: 流管理和 FLV 流实现

### 视频处理
- `ffmpeg`: 用于视频处理的核心 FFmpeg 集成
  - `ffmpeg_cmd`: 为 FFmpeg 构建命令
  - `ffmpeg_eo`: FFmpeg 执行对象
  - `ffmpeg_error`: FFmpeg 操作的错误处理
  - `ffmpeg_session`: 视频处理的会话管理

### 配置
- `config`: 应用配置管理
  - `app_config`: 通用应用配置
  - `capturer_config`: 视频抓拍特定配置

## 依赖项

主要依赖包括：
- [Actix Web](https://actix.rs/): HTTP 服务器的 Web 框架
- [FFmpeg](https://ffmpeg.org/): 视频处理引擎
- [Tokio](https://tokio.rs/): 异步运行时
- [Utoipa](https://github.com/juhaku/utoipa): OpenAPI 文档
- [Serde](https://serde.rs/): 序列化/反序列化
- [Clap](https://github.com/clap-rs/clap): 命令行参数解析

## 构建

构建项目：

```bash
# 在 capturer-svr 目录中
cargo build

# 或从工作区根目录
cd ..
cargo build -p capturer-svr
```

## 运行

可以通过以下方式启动服务器：

```bash
# 从 capturer-svr 目录
cargo run

# 或从工作区根目录
cd ..
cargo run -p capturer-svr

# 使用自定义配置
cargo run -- --config-file ./capturer-svr.toml

# 使用自定义端口
cargo run -- --port 8080
```

## 配置

服务器使用 TOML 配置文件。默认情况下，它查找 `capturer-svr.toml`，但可以通过 `--config-file` 命令行选项接受自定义路径。

## API 端点

该服务为视频抓拍操作提供 RESTful API。服务器运行时可在 `/swagger-ui/` 获取文档。

## 测试

运行测试：

```bash
cargo test
```

`tests/` 目录中有集成测试，用于验证 FFmpeg 功能。

## Docker 镜像

### 地址

[Docker Hub](https://hub.docker.com/nnzbz/capturer)

### 制作并发布镜像

```bash
docker buildx build --platform linux/arm64,linux/amd64 -t nnzbz/capturer:1.0.1 . --push
```

## 贡献

1. 确保已安装 Rust 和 FFmpeg
2. 进行更改
3. 运行测试: `cargo test`
4. 格式化代码: `cargo fmt`
5. 检查问题: `cargo clippy`

## 许可证

此项目采用 MIT 许可证。