# Capturer Server

A Rust-based video capturing service that captures images from video streams, supporting RTSP and other video formats.

## Features

- Capture images from video streams
- Support for various video formats (RTSP, MP4, etc.)
- RESTful API for easy integration
- FFmpeg-based video processing
- FLV streaming support
- Configurable server settings
- Object Storage Service (OSS) integration

## Prerequisites

- Rust 1.84+ (edition 2024)
- FFmpeg installed on the system
- OpenSSL for secure connections

## Installation

### Clone the repository

```bash
git clone https://github.com/rusthing/capturer.git
cd capturer
```

### Build the project

```bash
cargo build --release
```

### Using Docker

The project includes a Dockerfile for containerized deployment:

```bash
# Build the Docker image
docker build -t rusthing/capturer .

# Run the container
docker run -d -p 8080:8080 rusthing/capturer
```

## Configuration

The application uses a configuration file named `capturer-svr.toml`. You can specify a custom configuration file path using the `--config-file` command-line option.

Example configuration:

```toml
[web_server]
port = 8080
host = "0.0.0.0"

[api_client]
# API client configuration for OSS integration
```

## Usage

### Command Line Options

```bash
./capturer-svr --help
```

Available options:
- `-c, --config-file`: Path to the configuration file
- `-p, --port`: Web server port number
- `-V, --version`: Show version information

### Running the Server

```bash
# With default settings
./capturer-svr

# With custom port
./capturer-svr --port 8080

# With custom configuration file
./capturer-svr --config-file /path/to/config.toml
```

## API Documentation

The service provides Swagger UI documentation available at `/swagger-ui/` endpoint when running.

## Architecture

The project is organized in the following modules:

- `api_doc`: API documentation configuration
- `config`: Application configuration management
- `ctrl`: Controllers for handling HTTP requests
- `dto`: Data transfer objects
- `ffmpeg`: Video processing with FFmpeg
- `stream`: Stream management (FLV streaming)
- `svc`: Business logic services
- `vo`: Value objects

## FFmpeg Integration

The application leverages FFmpeg for video processing and image extraction. It includes:

- Video stream capture
- Image extraction from video frames
- Format conversion capabilities
- Session management for concurrent operations

## Testing

Run the project tests:

```bash
cargo test
```

Some tests may require specific test files located in the `tests/static/` directory.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run the tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Actix Web](https://actix.rs/) - A powerful, pragmatic, and extremely fast web framework for Rust
- Uses [FFmpeg](https://ffmpeg.org/) for video processing
- Documentation powered by [Utoipa](https://github.com/juhaku/utoipa) for OpenAPI generation