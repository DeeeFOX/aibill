# ---- Builder Stage ----
FROM rust:1.86-slim-bookworm as builder

# 创建APT源文件并配置国内镜像
RUN mkdir -p /etc/apt/sources.list.d && \
    echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm main contrib non-free" > /etc/apt/sources.list && \
    echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian-security bookworm-security main" >> /etc/apt/sources.list

# 配置Cargo国内镜像
RUN mkdir -p /usr/local/cargo && \
    echo '[source.crates-io]' > /usr/local/cargo/config && \
    echo 'replace-with = "tuna"' >> /usr/local/cargo/config && \
    echo '[source.tuna]' >> /usr/local/cargo/config && \
    echo 'registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"' >> /usr/local/cargo/config

# 安装构建依赖（新增perl和基本构建工具）
RUN dpkg --add-architecture amd64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        perl \
        build-essential \
        musl-dev \
        musl-tools \
        pkg-config \
        libssl-dev:amd64 \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

ARG TARGET=x86_64-unknown-linux-musl
ENV TARGET=${TARGET} \
    CC_x86_64_unknown_linux_musl=musl-gcc \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc \
    PKG_CONFIG_ALLOW_CROSS=1 \
    OPENSSL_STATIC=1

# 添加musl目标平台
RUN rustup target add ${TARGET}

WORKDIR /usr/src/app
COPY coze_token_service/Cargo.toml coze_token_service/Cargo.lock ./

ARG TARGET=x86_64-unknown-linux-musl
ENV TARGET=${TARGET} 

# 预编译依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target ${TARGET} && \
    rm -rf src

# Copy the actual source code
COPY coze_token_service/src ./src

# Build the application in release mode
# Ensure the target directory is clean before the final build
RUN rm -f target/release/deps/coze_token_service*

ARG TARGET=x86_64-unknown-linux-musl
ENV TARGET=${TARGET} 
# 编译应用程序
RUN cargo build --release --target ${TARGET}

# ---- Runtime Stage ----
FROM alpine:3.19
RUN apk add --no-cache ca-certificates tzdata && \
    ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime

ARG TARGET=x86_64-unknown-linux-musl
ENV TARGET=${TARGET} 
WORKDIR /app
COPY --from=builder /usr/src/app/target/${TARGET}/release/coze_token_service .

EXPOSE 9000
CMD ["./coze_token_service"]