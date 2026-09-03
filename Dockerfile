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

# Pre-install cargo-watch for dev hot-reload (docker-compose.dev.yml builds
# `target: builder` and runs `cargo watch`) — cached as its own layer.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-watch --locked

WORKDIR /build

# The full (dockerignore-filtered) source is copied once; BuildKit cache mounts
# below provide dependency compilation caching across builds. Note (SRV-07): the
# previous "skeleton sources" pre-build layer was removed — cargo validates the
# root Cargo.toml's explicit [[bin]]/[[test]] target paths at manifest-load time
# (a skeleton cannot skip them), and a `--mount=type=cache` target dir is never
# committed to the image, so `COPY --from=builder /build/target/...` could never
# resolve. Keep `tests/` present in the build context (.dockerignore).
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --profile ci --package ${BINARY} && \
    cp "target/ci/${BINARY}" "/usr/local/bin/${BINARY}-artifact"

# ───────────────────────────────────────────────────────
# Stage 2 — Minimal runtime image
# ───────────────────────────────────────────────────────
FROM debian:bookworm-slim
ARG BINARY
ARG APP_VERSION
# Default runtime user uid; override at build time with
#   --build-arg VANTA_RUNAS_UID=<uid>
# At runtime no rebuild is needed: `docker run --user 10001:10001 ...` works
# because the data dir is world-writable (see below).
ARG VANTA_RUNAS_UID=1001

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Unprivileged run (qdrant pattern):
#  * default: non-root user `vantadb` (uid ${VANTA_RUNAS_UID}, gid 1001)
#  * arbitrary --user: /var/lib/vantadb (data dir + WORKDIR) is mode 0777 so any
#    uid can write it; named volumes inherit these perms at creation. Host
#    bind-mounts need the host dir writable by the running uid (chown or chmod
#    777). See docs/operations/DEPLOYMENT_GUIDE.md §3 "Run unprivileged".
RUN groupadd --gid 1001 vantadb && \
    useradd --uid "${VANTA_RUNAS_UID}" --gid vantadb --create-home vantadb && \
    mkdir -p /var/lib/vantadb && \
    chmod 777 /var/lib/vantadb

COPY --from=builder /usr/local/bin/${BINARY}-artifact /usr/local/bin/vantadb-server

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
