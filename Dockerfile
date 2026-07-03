# syntax=docker/dockerfile:1
# Multi-stage build — compiled Rust binary, minimal runtime image
FROM rust:1.86-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN cargo build --release --package vantadb-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/vantadb-server /usr/local/bin/vantadb-server

EXPOSE 8080
ENTRYPOINT ["vantadb-server"]
