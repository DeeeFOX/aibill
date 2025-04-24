# ---- Builder Stage ----
# Use the official Rust image as a builder
FROM rust:1.86-slim as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo manifest and lock file
COPY coze_token_service/Cargo.toml coze_token_service/Cargo.lock ./

# Install necessary build dependencies (OpenSSL dev libraries and pkg-config)
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build dependencies first to leverage Docker cache
# Create a dummy main.rs to build only dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY coze_token_service/src ./src

# Build the application in release mode
# Ensure the target directory is clean before the final build
RUN rm -f target/release/deps/coze_token_service*
RUN cargo build --release

# ---- Runtime Stage ----
# Use a minimal Debian image for the runtime environment
FROM debian:12-slim

# Install necessary runtime dependencies (ca-certificates for potential HTTPS calls if needed later)
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/local/bin

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/coze_token_service .

# Expose the port the application listens on
EXPOSE 9000

# Set the entrypoint for the container
# The application will read JWT_PRIVATE_KEY from the environment at runtime
CMD ["./coze_token_service"]
