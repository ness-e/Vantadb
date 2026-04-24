# ==========================================
# STAGE 1: BUILD STAGE
# ==========================================
FROM rust:slim-bookworm AS builder

# Instalar dependencias requeridas por rust-rocksdb / pyo3
RUN apt-get update && apt-get install -y \
    clang \
    llvm \
    cmake \
    make \
    g++ \
    libsnappy-dev \
    liblz4-dev \
    libzstd-dev \
    git \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/vantadb
COPY . .

# Compilar release asegurando optimizaciones LTO + O3 (por defecto en release)
RUN cargo build --release --bin vanta-server

# ==========================================
# STAGE 2: RUNTIME STAGE
# ==========================================
FROM debian:bookworm-slim

# Instalar dependencias runtime para RocksDB local
RUN apt-get update && apt-get install -y \
    libsnappy1v5 \
    liblz4-1 \
    libzstd1 \
    ca-certificates \
    gawk \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get clean

WORKDIR /vantadb

# Inyectar binario y entrypoint dinámico
COPY --from=builder /usr/src/vantadb/target/release/vanta-server /usr/local/bin/vanta-server
COPY start.sh /usr/local/bin/start.sh

# Preparar entorno minimalista
RUN chmod +x /usr/local/bin/start.sh \
 && mkdir -p /vantadb/data

# Puerto por defecto (MCP / HTTP)
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/start.sh"]
