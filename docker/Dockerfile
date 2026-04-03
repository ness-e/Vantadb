# Dockerfile
# ConnectomeDB: Neural-Inspired Multimodel Database for Local AI
FROM rust:1.76-slim-bullseye AS builder

# Install build dependencies for RocksDB (requires libclang + cmake)
RUN apt-get update && apt-get install -y \
    libclang-dev \
    clang \
    cmake \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/connectomedb

# Cache dependencies layer: copy manifests first, build dummy, then copy real source
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin connectomedb 2>/dev/null || true && \
    rm -f src/main.rs

COPY src ./src
COPY benches ./benches
COPY tests ./tests

RUN touch src/main.rs && cargo build --release --bin connectomedb

# --- Runtime Stage ---
FROM debian:bullseye-slim

# Install runtime dependencies for RocksDB (libsnappy, liblz4)
RUN apt-get update && apt-get install -y \
    libsnappy1v5 \
    liblz4-1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/connectomedb/target/release/connectomedb /usr/local/bin/connectomedb

ENV RUST_LOG=info
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["connectomedb"]
