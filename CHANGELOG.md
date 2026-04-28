# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.1.0] - Initial MVP Release

### Included

- Embedded multimodel engine unifying vector, graph, and relational metadata operations in Rust.
- Fjall as the default backend, with RocksDB retained as an explicit fallback path.
- WAL-backed crash recovery for the storage layer.
- HNSW index reconstruction from VantaFile when the index artifact is missing or damaged.
- HNSW recall certification coverage for MVP-quality validation.
- Local server default bind hardened to `127.0.0.1:8080`.
- SHA256 checksums published alongside release binaries.
- Split CI strategy: fast deterministic gate plus heavy certification.

### Release Notes

- Python SDK remains source-install only for now; PyPI publication is not part of `v0.1.0`.
- Docker and Ollama remain deferred or experimental and are not official release channels for this MVP.

## [v0.1.0-rc2] - DX Hardening

### Added

- Checksums SHA256 in release assets.
- Basic smoke test (`dev-tools/smoke_test.sh`).
- Native PowerShell smoke test for Windows (`dev-tools/smoke_test.ps1`).
- Formalized security policy (`SECURITY.md`) and contributing guidelines (`CONTRIBUTING.md`).

### Changed

- Default secure bind `127.0.0.1` instead of `0.0.0.0`.
- Reduced log noise of the MaintenanceWorker for idle cycles.
- Updated Windows RC documentation for standalone binaries.
- Fixed storage engine initialization to ensure base directory creation.

## [v0.1.0-rc1] - MVP Release Candidate

### Added

- Fjall integration as the default StorageBackend, with RocksDB as an explicit fallback, establishing robust BackendCapabilities.
- WAL replay and crash recovery mechanisms specifically for Fjall.
- Index reconstruction capability from VantaFile upon engine restart.
- CI Workflow Split: `rust_ci.yml` for fast local/deterministic gates and `heavy_certification.yml` for stress tests.
- Standardized HNSW index with deterministic configurable limits (`m`, `ef_construction`, `m_max0`), beam search graph exploration, and priority queue heuristic neighbor selection.
- `tests/hnsw_recall.rs` to validate algorithm mathematical precision and verify index capabilities.
- Python SDK rebranded to `vantadb-python` exposing PyO3 capabilities.

### Changed

- Fixed HNSW recall logic and regressions, ensuring ≥0.95 recall accuracy via certification tests.
- Restructured `structured_api_v2` to rely strictly on local core engine logic, omitting external networking.
- Paused Ollama/LLM integration out of the fast MVP gate to stabilize local CI operations.
- Docker artifacts (`Dockerfile`, `docker-compose.yml`) moved to `examples/docker/` and marked as experimental; they are no longer part of the official MVP release candidate or automatic GHCR publish.
- Complete semantic overhaul of the codebase purging old biological terminology (removing "neurons", "synapses"). Replaced strictly with mathematical equivalents (`UnifiedNode`, `Edge`).
- Updated project naming globally from *ConnectomeDB / NexusDB* to **VantaDB**.
- Simplified the internal index module replacing placeholder graph mappings with formalized spatial structures.

## [Legacy Build] - Connectome Prototype

### Removed

- Unstable proto-CGR algorithms.
- Hard-coded vector similarity loops lacking validation suites.
