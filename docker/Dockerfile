# Dockerfile
# ConnectomeDB: Neural-Inspired Multimodel Database for Local AI
FROM rust:1.76-slim-bullseye AS builder

WORKDIR /usr/src/connectomedb
COPY Cargo.toml Cargo.lock ./
# Note: In a real project, we would build dependencies here to cache them using dummy main.rs
COPY src ./src
COPY benches ./benches
COPY tests ./tests

RUN cargo build --release

FROM debian:bullseye-slim
# Install curl for healthchecks
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/connectomedb/target/release/connectome-server /usr/local/bin/connectome-server

ENV RUST_LOG=info
EXPOSE 8080

CMD ["connectome-server"]
