# VantaDB Server — Multi-stage build for minimal production image
# Syntax: docker build -t vantadb-server:latest -f vantadb-server/Dockerfile .
#         docker build -t vantadb-server:unprivileged -f vantadb-server/Dockerfile --target unprivileged .

# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.94-slim-bookworm AS builder

WORKDIR /build

# Install build deps (mold linker for speed, pkg-config for openssl, etc.)
RUN apt-get update && apt-get install -y --no-install-recommends \
    mold \
    pkg-config \
    libssl-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first (cargo cache)
COPY Cargo.toml Cargo.lock ./
COPY vantadb/Cargo.toml vantadb/
COPY vantadb-server/Cargo.toml vantadb-server/
COPY vantadb-mcp/Cargo.toml vantadb-mcp/

# Pre-fetch dependencies (cache layer)
RUN mkdir -p vantadb/src vantadb-server/src vantadb-mcp/src \
    && echo 'fn main() {}' > vantadb/src/lib.rs \
    && echo 'fn main() {}' > vantadb-server/src/main.rs \
    && echo 'fn main() {}' > vantadb-mcp/src/lib.rs \
    && cargo fetch --locked --workspace --manifest-path Cargo.toml 2>/dev/null || true

# Copy full source
COPY . .

# Build release binary with mold (fast linking)
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"
RUN cargo build --release --package vantadb-server --locked

# ── Runtime base (Debian slim, non-root) ─────────────────────────────────────
FROM debian:bookworm-slim AS runtime-base

# Install runtime deps only (ca-certificates for TLS, libssl for rustls)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --no-create-home --uid 1000 --shell /bin/false vantadb

WORKDIR /var/lib/vantadb
ENV VANTADB_STORAGE_PATH=/var/lib/vantadb/data

# Copy binary
COPY --from=builder /build/target/release/vanta-cli /usr/local/bin/vanta-cli

# Data volume
VOLUME ["/var/lib/vantadb/data"]

# Healthcheck (uses /health endpoint)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/health || exit 1

EXPOSE 8080

# Default user (overridden in unprivileged stage)
USER vantadb

ENTRYPOINT ["vanta-cli", "server", "--http", "--host", "0.0.0.0", "--port", "8080", "--db", "/var/lib/vantadb/data"]

# ── Unprivileged variant (read-only rootfs, no capabilities) ──────────────────
FROM runtime-base AS unprivileged

# Drop all capabilities, read-only rootfs (handled by docker run --read-only --cap-drop=ALL)
# The binary and config are baked in; only /var/lib/vantadb/data is writable via volume.
# No additional changes needed — the base stage already runs as UID 1000.
# This stage exists as a named target for: docker build --target unprivileged ...

# ── GitHub Release binary variant (download instead of build) ─────────────────
# Usage: docker build --target release-binary --build-arg VERSION=v0.5.0 ...
FROM debian:bookworm-slim AS release-binary

ARG VERSION=latest
ARG TARGETARCH=amd64

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --no-create-home --uid 1000 --shell /bin/false vantadb

WORKDIR /var/lib/vantadb
ENV VANTADB_STORAGE_PATH=/var/lib/vantadb/data

# Download pre-built binary from GitHub Releases
# Expected asset: vantadb-server-${VERSION}-${TARGETARCH}-unknown-linux-gnu.tar.gz
RUN set -eu; \
    if [ "$VERSION" = "latest" ]; then \
        URL="https://github.com/ness-e/Vantadb/releases/latest/download/vantadb-server-${TARGETARCH}-unknown-linux-gnu.tar.gz"; \
    else \
        URL="https://github.com/ness-e/Vantadb/releases/download/${VERSION}/vantadb-server-${VERSION}-${TARGETARCH}-unknown-linux-gnu.tar.gz"; \
    fi; \
    echo "Downloading $URL"; \
    curl -fsSL "$URL" | tar -xz -C /usr/local/bin vantadb-server 2>/dev/null || \
    (echo "Release binary not found; falling back to local build" && exit 1); \
    mv /usr/local/bin/vantadb-server /usr/local/bin/vanta-cli; \
    chmod +x /usr/local/bin/vanta-cli

VOLUME ["/var/lib/vantadb/data"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/health || exit 1
EXPOSE 8080
USER vantadb
ENTRYPOINT ["vanta-cli", "server", "--http", "--host", "0.0.0.0", "--port", "8080", "--db", "/var/lib/vantadb/data"]