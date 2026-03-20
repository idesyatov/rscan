# Stage 1: Build
FROM rust:latest AS builder

WORKDIR /app

# Install Windows cross-compilation toolchain
RUN apt-get update && apt-get install -y gcc-mingw-w64-x86-64 && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-pc-windows-gnu

# Cache dependencies: copy manifests and build with dummy source
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN cargo build --release --target x86_64-pc-windows-gnu 2>/dev/null || true
RUN rm -rf src

# Build real source
COPY src/ ./src/
RUN touch src/main.rs
RUN cargo build --release
RUN cargo build --release --target x86_64-pc-windows-gnu

# Stage 2: Minimal image with just the copy command
FROM debian:bookworm-slim

COPY --from=builder /app/target/release/rscan /build/linux/rscan
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/rscan.exe /build/windows/rscan.exe

CMD ["sh", "-c", "\
  mkdir -p /dist/linux /dist/windows && \
  cp /build/linux/rscan /dist/linux/ && \
  cp /build/windows/rscan.exe /dist/windows/ && \
  echo 'Build complete:' && \
  echo '  Linux:   /dist/linux/rscan' && \
  echo '  Windows: /dist/windows/rscan.exe'"]
