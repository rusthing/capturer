FROM nnzbz/rust-app:1.0.1

# 更新包管理器并安装 ffmpeg
RUN apk update && apk upgrade && \
    apk add --no-cache ffmpeg

# 复制应用程序二进制文件
COPY target/x86_64-unknown-linux-musl/release/capturer-svr /usr/local/myapp/myapp
