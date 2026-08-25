# syntax=docker/dockerfile:1
# VantaDB Server — multi-stage build with dependency caching & minimal runtime
# https://vantadb.dev
ARG RUST_VERSION=1.95.0
ARG BINARY=vantadb-server
ARG APP_VERSION=0.5.0

# ───────────────────────────────────────────────────────
# Stage 1 — Build the Rust binary
# ───────────────────────────────────────────────────────
FROM rust:${RUST_VERSION}-slim-bookworm AS builder
ARG BINARY

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Pre-install cargo-watch for dev hot-reload (avoids re-install on every compose up)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-watch --locked

WORKDIR /build

# ── Dependency layer ──
# Copy only manifests so Docker caches dependency compilation separately
COPY Cargo.toml Cargo.lock ./
COPY vantadb-server/Cargo.toml vantadb-server/
COPY vantadb-mcp/Cargo.toml vantadb-mcp/
COPY vanta-memory/Cargo.toml vanta-memory/
COPY vanta-proxy/Cargo.toml vanta-proxy/
COPY vantadb-python/Cargo.toml vantadb-python/
COPY vantadb-wasm/Cargo.toml vantadb-wasm/

# Skeleton sources so cargo can resolve & compile all dependencies.
# Only vantadb-server, vantadb-mcp and the root crate are needed to build the
# server binary — the integration crates (mem0, letta, crewai, dspy, haystack,
# litellm, openai, ollama) live in integrations/ as Python packages and are
# not workspace members (see [workspace.members] in Cargo.toml).
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    mkdir -p vantadb-server/src && echo "fn main() {}" > vantadb-server/src/main.rs && \
    mkdir -p vantadb-mcp/src && echo "" > vantadb-mcp/src/lib.rs &&
    mkdir -p vanta-memory/src && echo "" > vanta-memory/src/lib.rs &&
    mkdir -p vanta-proxy/src && echo "" > vanta-proxy/src/lib.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --profile ci --package ${BINARY}

# Remove skeletons before copying real sources. Root src/main.rs is a skeleton
# only (the repo root has no main.rs) — scoped to the paths that exist.
RUN rm -rf src/ vantadb-server/src vantadb-mcp/src vanta-memory/src vanta-proxy/src

# ── Real build ──
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --profile ci --package ${BINARY}

# ───────────────────────────────────────────────────────
# Stage 2 — Minimal runtime image
# ───────────────────────────────────────────────────────
FROM debian:bookworm-slim
ARG BINARY
ARG APP_VERSION

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root user + data directory
RUN groupadd --gid 1001 vantadb && \
    useradd --uid 1001 --gid vantadb --create-home vantadb && \
    mkdir -p /var/lib/vantadb && \
    chown -R vantadb:vantadb /var/lib/vantadb

COPY --from=builder /build/target/ci/${BINARY} /usr/local/bin/vantadb-server

# OCI metadata labels
LABEL maintainer="VantaDB Team <dev@vantadb.dev>" \
      org.opencontainers.image.title="VantaDB Server" \
      org.opencontainers.image.description="Embedded persistent memory and vector retrieval engine for local-first AI applications" \
      org.opencontainers.image.version="${APP_VERSION}" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/ness-e/Vantadb" \
      org.opencontainers.image.url="https://vantadb.dev" \
      org.opencontainers.image.documentation="https://docs.rs/vantadb"

# Drop privileges
USER vantadb
WORKDIR /var/lib/vantadb

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --retries=3 --start-period=10s \
  CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["vantadb-server"]
