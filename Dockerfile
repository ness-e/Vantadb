# syntax=docker/dockerfile:1
# VantaDB Server — multi-stage build with dependency caching & minimal runtime
# https://vantadb.dev
ARG RUST_VERSION=1.94
ARG BINARY=vantadb-server
ARG APP_VERSION=0.3.0

# ───────────────────────────────────────────────────────
# Stage 1 — Build the Rust binary
# ───────────────────────────────────────────────────────
FROM rust:${RUST_VERSION}-slim-bookworm AS builder
ARG BINARY

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# ── Dependency layer ──
# Copy only manifests so Docker caches dependency compilation separately
COPY Cargo.toml Cargo.lock ./
COPY vantadb-server/Cargo.toml vantadb-server/
COPY vantadb-mcp/Cargo.toml vantadb-mcp/
COPY vantadb-python/Cargo.toml vantadb-python/
COPY vantadb-wasm/Cargo.toml vantadb-wasm/
COPY vantadb-mem0/Cargo.toml vantadb-mem0/
COPY vantadb-letta/Cargo.toml vantadb-letta/
COPY vantadb-crewai/Cargo.toml vantadb-crewai/
COPY vantadb-dspy/Cargo.toml vantadb-dspy/
COPY vantadb-haystack/Cargo.toml vantadb-haystack/
COPY vantadb-litellm/Cargo.toml vantadb-litellm/
COPY vantadb-openai/Cargo.toml vantadb-openai/
COPY vantadb-ollama/Cargo.toml vantadb-ollama/

# Skeleton sources to resolve & compile all dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    mkdir -p vantadb-server/src && echo "fn main() {}" > vantadb-server/src/main.rs && \
    mkdir -p vantadb-mcp/src && echo "" > vantadb-mcp/src/lib.rs && \
    for d in vantadb-python vantadb-wasm vantadb-mem0 vantadb-letta vantadb-crewai vantadb-dspy vantadb-haystack vantadb-litellm vantadb-openai vantadb-ollama; do \
      mkdir -p "$d/src" && echo "" > "$d/src/lib.rs"; \
    done

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --package ${BINARY}

# Remove skeleton before copying real sources
RUN rm -rf src/ vantadb-*/src/

# ── Real build ──
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --config 'profile.release.strip="symbols"' --package ${BINARY}

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

COPY --from=builder /build/target/release/${BINARY} /usr/local/bin/vantadb-server

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
