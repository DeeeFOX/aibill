# ---- Builder Stage ----
FROM rust:1.86 as builder
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGET

# 创建APT源文件并配置国内镜像
RUN mkdir -p /etc/apt/sources.list.d && \
    echo "deb https://mirrors.aliyun.com/debian bookworm main contrib non-free" > /etc/apt/sources.list && \
    echo "deb https://mirrors.aliyun.com/debian-security bookworm-security main" >> /etc/apt/sources.list && \
    echo "deb https://mirrors.aliyun.com/debian bookworm-updates main" >> /etc/apt/sources.list && \
    find /etc/apt/sources.list.d/ -type f -exec sed -i 's|http://deb.debian.org|https://mirrors.aliyun.com|g' {} \;

# 配置Cargo国内镜像
RUN mkdir -p /usr/local/cargo && \
    echo '[source.crates-io]' > /usr/local/cargo/config && \
    echo 'replace-with = "aliyun"' >> /usr/local/cargo/config && \
    echo '[source.aliyun]' >> /usr/local/cargo/config && \
    echo 'registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"' >> /usr/local/cargo/config

# 安装构建依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        musl-tools \
    && rm -rf /var/lib/apt/lists/*

ENV TARGET=${TARGET} \
    PATH="/usr/${TARGET%%-*}-linux-musl/bin:${PATH}" \
    CC=${TARGET%%-*}-linux-musl-gcc

# 设置 Rustup 国内镜像
ENV RUSTUP_DIST_SERVER=https://mirrors.aliyun.com/rustup \
    RUSTUP_UPDATE_ROOT=https://mirrors.aliyun.com/rustup/rustup

# 添加musl目标平台
RUN rustup target add ${TARGET}

WORKDIR /usr/src/app
COPY coze_token_service/Cargo.toml coze_token_service/Cargo.lock ./

# 预编译依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target ${TARGET} && \
    rm -rf src

# 复制源代码
COPY coze_token_service/src ./src

# 编译应用程序
RUN rm -f target/${TARGET}/release/coze_token_service* && \
    cargo build --release --target ${TARGET}

# ---- Runtime Stage ----
FROM alpine:3.19 as runtime
ARG TARGET
RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /usr/src/app/target/${TARGET}/release/coze_token_service .

EXPOSE 9000
ENV RUST_LOG=debug
CMD ["./coze_token_service"]
