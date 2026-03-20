FROM rust:latest

WORKDIR /app

# Установка тулчейна для кросс-компиляции под Windows
RUN apt-get update && apt-get install -y gcc-mingw-w64-x86-64 && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-pc-windows-gnu

COPY Cargo.toml ./
COPY src/ ./src/

# Сборка под Linux
RUN cargo build --release

# Сборка под Windows
RUN cargo build --release --target x86_64-pc-windows-gnu

CMD ["sh", "-c", "\
  mkdir -p /dist/linux /dist/windows && \
  cp /app/target/release/rscan /dist/linux/ && \
  cp /app/target/x86_64-pc-windows-gnu/release/rscan.exe /dist/windows/ && \
  echo 'Build complete:' && \
  echo '  Linux:   /dist/linux/rscan' && \
  echo '  Windows: /dist/windows/rscan.exe'"]
