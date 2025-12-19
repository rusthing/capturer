# 视频抓拍服务

[English Version](README.md)

视频抓拍服务是一个使用 Rust 开发的视频抓拍工具，可以从视频流中抓拍帧并以图片形式返回。支持 RTSP 流抓拍和转换为 JPEG 格式。

## 功能特性

- 从 RTSP 视频流中抓拍单帧
- 将抓拍的帧转换为 JPEG 格式
- 支持 FLV 视频流传输
- 提供 RESTful API 便于集成
- 支持 Docker 部署
- 支持通过 TOML 文件配置

## 环境要求

- Rust 工具链
- 系统中安装 FFmpeg
- 可访问的 RTSP 视频流（用于测试）

## 构建

### 使用 Cargo 构建

```bash
cd capturer-svr
cargo build --release
```

### 使用 Docker 构建

```bash
docker build -t capturer-svr .
```

## 配置

服务可以通过 TOML 配置文件进行配置。默认情况下，会在工作目录中查找 `capturer-svr.toml` 文件。

配置示例：
```toml
[web-server]
port = 9850

[capturer]
stream.session-check-interval-seconds = 5
stream.session-timeout-seconds = 5
stream.channel_capacity = 5
```

## 运行

### 直接运行

```bash
./target/release/capturer-svr [OPTIONS]
```

选项：
- `-c, --config-file <CONFIG_FILE>`: 配置文件路径
- `-p, --port <PORT>`: Web 服务端口号

### 使用 Docker 运行

```bash
docker run -p 9850:9850 capturer-svr
```

## API 接口

### 抓拍为 JPEG

```
POST /capturer/capture_to_jpeg
```

请求体：
```json
{
  "streamUrl": "rtsp://example.com/stream"
}
```

### FLV 流传输

```
GET /capturer/stream.live.flv?streamUrl=rtsp://example.com/stream
```

## 项目结构

```
capturer/
├── capturer-svr/           # 主服务实现
│   ├── src/                # 源代码
│   │   ├── ctrl/           # API 控制器
│   │   ├── dto/            # 数据传输对象
│   │   ├── ffmpeg/         # FFmpeg 集成
│   │   ├── settings/       # 配置处理
│   │   ├── stream/         # 流媒体功能
│   │   ├── svc/            # 业务逻辑服务
│   │   ├── utils/          # 工具函数
│   │   └── vo/             # 值对象
│   ├── tests/              # 集成测试
│   └── capturer-svr.toml   # 默认配置
└── Dockerfile              # Docker 构建配置
```

## 许可证

该项目采用 MIT 许可证 - 详见 LICENSE 文件了解更多详情。