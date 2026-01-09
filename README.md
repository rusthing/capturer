# Capturer Service

[中文版本](README_zh.md)

Capturer Service is a video capturing tool developed in Rust that can capture frames from video streams and return them as images. It supports RTSP
stream capture and conversion to JPEG format.

## Features

- Capture single frames from RTSP video streams
- Convert captured frames to JPEG format
- Stream FLV video content
- RESTful API for easy integration
- Docker support for easy deployment
- Configuration via TOML files

## Prerequisites

- Rust toolchain
- FFmpeg installed on the system
- Access to RTSP video streams (for testing)

## Building

### Using Cargo

```bash
cd capturer-svr
cargo build --release
```

### Using Docker

```bash
docker build -t capturer-svr .
```

## Configuration

The service can be configured using a TOML configuration file. By default, it looks for `capturer-svr.toml` in the working directory.

Example configuration:

```toml
[web-server]
port = 9850

[capturer]
stream.session-check-interval-seconds = 5
stream.session-timeout-seconds = 5
stream.channel_capacity = 5
```

## Running

### Direct execution

```bash
./target/release/capturer-svr [OPTIONS]
```

Options:

- `-c, --config-file <CONFIG_FILE>`: Path to the configuration file
- `-p, --port <PORT>`: Web server port number

### Using Docker

```bash
docker run -p 9850:9850 capturer-svr
```

## API Endpoints

### Capture to JPEG

```
POST /capturer/capture_to_jpeg
```

Request Body:

```json
{
  "streamUrl": "rtsp://example.com/stream"
}
```

### Stream FLV

```
GET /capturer/stream.live.flv?streamUrl=rtsp://example.com/stream
```

## Docker Image

### address

[Docker Hub](https://hub.docker.com/nnzbz/capturer)

### build and publish image

```bash
docker buildx build --platform linux/arm64,linux/amd64 -t nnzbz/capturer:1.0.0 . --push
```

## Project Structure

```
capturer/
├── capturer-svr/           # Main service implementation
│   ├── src/                # Source code
│   │   ├── ctrl/           # Controllers for API endpoints
│   │   ├── dto/            # Data transfer objects
│   │   ├── ffmpeg/         # FFmpeg integration
│   │   ├── config/         # Configuration handling
│   │   ├── stream/         # Streaming functionality
│   │   ├── svc/            # Business logic services
│   │   ├── utils/          # Utility functions
│   │   └── vo/             # Value objects
│   ├── tests/              # Integration tests
│   └── capturer-svr.toml   # Default configuration
└── Dockerfile              # Docker build configuration
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.