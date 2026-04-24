# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - MVP Release Candidate
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
- Complete semantic overhaul of the codebase purging old biological terminology (removing "neurons", "synapses"). Replaced strictly with mathematical equivalents (`UnifiedNode`, `Edge`).
- Updated project naming globally from *ConnectomeDB / NexusDB* to **VantaDB**.
- Simplified the internal index module replacing placeholder graph mappings with formalized spatial structures.

## [Legacy Build] - Connectome Prototype
### Removed
- Unstable proto-CGR algorithms.
- Hard-coded vector similarity loops lacking validation suites.
