---
title: Changelog
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-07-20
aliases: []
---

# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.4.0] — 2026-07-20 — Inception Release

### Added

- Initial public release of VantaDB.
- Embedded persistent vector/graph database engine.
- HNSW vector index with configurable distance metrics.
- WAL (Write-Ahead Log) with automatic crash recovery.
- Arrow IPC integration for zero-copy data interchange.
- CLI tool (`vanta`) for database operations.
- HTTP server with rate limiting and TLS support.
- Python SDK (`vantadb_py`) with PyO3 bindings.
- WASM build for browser-based querying.
- TypeScript SDK (`vantadb-ts`).
- AI framework adapters: LangChain, LlamaIndex, Haystack, CrewAI, DSPy, Litellm, OpenAI, Ollama, Mem0, Letta.
- MCP server (`vantadb-mcp`) for Model Context Protocol.
- Prometheus metrics and OpenTelemetry tracing.
- Encryption at rest (AES-GCM).
- PITR (Point-in-Time Recovery).
- WAL shipping for replication.
- Hot-reload of configuration.
- Failpoints for fault injection testing.
- Custom allocator support (mimalloc, jemalloc).

### Fixed

- CI/CD pipelines: FUZZ, release binaries, adapters, SBOM, wheels, code coverage — all green.
- Serialization bounds check overflow in `src/index/serialize.rs`.
- `vantadb-mcp` excluded from binary releases (library-only crate).
- Conditional `Attach to Release` step in adapter release workflow.
- Code coverage runner RAM increase (6GB → 8GB) to prevent LLD SIGBUS.

### Changed

- Workspace version reset to v0.4.0 — clean semantic versioning start.
- All previous tags (v0.1.0 through v0.3.0-stable, wasm-*, ts-*, adapters-*) removed.
- Root crate version inherits from `[workspace.package]`.

### Removed

- All pre-release tags and orphan GitHub Releases from v0.1.x era.
