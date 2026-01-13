# Capturer Server Core Module

This is the core module of the Capturer Server application, responsible for video capturing functionality using FFmpeg.

## Overview

The `capturer-svr` crate provides a video capturing service that extracts images from video streams. It's built with Rust and uses Actix Web for the HTTP interface and FFmpeg for video processing.

## Features

- Video stream capturing from various sources (RTSP, local files, etc.)
- Image extraction from video frames
- RESTful API endpoints for capturing operations
- FFmpeg integration for video processing
- Stream management for concurrent operations
- Integration with OSS (Object Storage Service) for storing captured images
- FLV streaming support
- API documentation via Utoipa

## Architecture

The module is organized in the following components:

### API Layer
- `api_doc`: API documentation configuration using Utoipa
- `ctrl`: Controllers handling HTTP requests and routing
- `dto`: Data Transfer Objects for request/response data
- `vo`: Value Objects for representing business entities

### Service Layer
- `svc`: Business logic services
- `stream`: Stream management and FLV streaming implementation

### Video Processing
- `ffmpeg`: Core FFmpeg integration for video processing
  - `ffmpeg_cmd`: Command construction for FFmpeg
  - `ffmpeg_eo`: FFmpeg execution objects
  - `ffmpeg_error`: Error handling for FFmpeg operations
  - `ffmpeg_session`: Session management for video processing

### Configuration
- `config`: Application configuration management
  - `app_config`: General application configuration
  - `capturer_config`: Video capturing specific configuration

## Dependencies

Key dependencies include:
- [Actix Web](https://actix.rs/): Web framework for HTTP server
- [FFmpeg](https://ffmpeg.org/): Video processing engine
- [Tokio](https://tokio.rs/): Asynchronous runtime
- [Utoipa](https://github.com/juhaku/utoipa): OpenAPI documentation
- [Serde](https://serde.rs/): Serialization/deserialization
- [Clap](https://github.com/clap-rs/clap): Command-line argument parsing

## Building

To build the project:

```bash
# In the capturer-svr directory
cargo build

# Or from the workspace root
cd ..
cargo build -p capturer-svr
```

## Running

The server can be started with:

```bash
# From the capturer-svr directory
cargo run

# Or from the workspace root
cd ..
cargo run -p capturer-svr

# With custom configuration
cargo run -- --config-file ./capturer-svr.toml

# With custom port
cargo run -- --port 8080
```

## Configuration

The server uses a TOML configuration file. By default, it looks for `capturer-svr.toml` but can accept a custom path via the `--config-file` command-line option.

## API Endpoints

The service provides RESTful APIs for video capturing operations. Documentation is available at `/swagger-ui/` when the server is running.

## Testing

Run the tests with:

```bash
cargo test
```

There are integration tests in the `tests/` directory that verify FFmpeg functionality.

## Contributing

1. Ensure you have Rust and FFmpeg installed
2. Make your changes
3. Run tests: `cargo test`
4. Format code: `cargo fmt`
5. Check for issues: `cargo clippy`

## License

This project is licensed under the MIT License.