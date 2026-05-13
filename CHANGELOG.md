# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes yet.

## [v0.1.1] - 2026-05-13

### Added

- Five-minute quickstart covering CLI memory operations, Python source install, vector search, BM25 text search, Hybrid Retrieval v1, JSONL export, and text-index audit.
- Python package README and cleaner `vantadb-py` metadata for wheel and TestPyPI validation.
- v0.1.1 release readiness checklist with local validation, Python wheel workflow, TestPyPI guardrails, and draft release notes.
- Draft public issue backlog for packaging, quickstart validation, Search Quality v2, benchmarks, backup/restore, Python distribution policy, and namespace-scoped memory examples.
- Persistent memory MVP surface with canonical SDK records, first-class namespaces, and `put/get/delete/list/search`.
- Manual ANN rebuild API through Rust SDK, Python SDK, and `vanta-cli rebuild-index`.
- JSONL memory export/import through Rust SDK, Python SDK, and `vanta-cli export/import`.
- Derived namespace and payload indexes for namespace lists and scalar metadata equality filters.
- Backend prefix scans for derived namespace/payload index lookups.
- Operational metrics API for startup, WAL replay, rebuild, export, import, and import errors.
- Derived-index state validation and repair on open.
- Minimal text-index tokenizer/key-shape scaffold before BM25/RRF.
- Persistent inverted text index for memory payload postings, with rebuild, repair-on-open, import rebuild, and operational metrics.
- BM25 text-only memory search over the persistent text index, including TF postings, DF/doc-length/namespace corpus stats, metadata filters, deterministic ordering, and read-only compatibility errors for stale schemas.
- Hybrid Retrieval v1 for memory search, using a minimal planner and RRF fusion over BM25 text rankings and vector rankings.
- Operational metrics for hybrid query latency, fused hybrid candidates, and planner route counts.
- Debug-build hybrid planner/RRF certification helper for route, budget, candidate counts, fused candidates, and top logical identities.
- Text-index schema v3 with persisted token positions in posting values and basic quoted phrase query support over `text_query`.
- Debug-build search explanation helper for snippets from canonical payloads, BM25 term contributions, matched phrases, and RRF ranks.
- Python wheel CI workflow for Linux, macOS, and Windows with generated-wheel smoke install and manual TestPyPI upload gate.
- Embedded-memory hybrid benchmark and certification corpora for text-only, vector-only, phrase, and hybrid retrieval paths.
- Text-index structural audit coverage and operational metrics for lexical queries, candidates scored, and audit failures.
- Public read-only text-index audit through Rust SDK, Python SDK, and `vanta-cli audit-index`.
- Operational roadmap in `docs/operations/ROADMAP.md` covering hardening, backup/restore, Python release engineering, and Search Quality v2.
- Fjall cold-copy restore validation covering canonical records, BM25/phrase text search, and hybrid retrieval.
- Project tracking CSV and text-index phase closeout evidence before BM25/RRF.
- `memory_export_import`, `derived_indexes`, and `memory_brutality` tests, including a 10K-record operational smoke.
- `text_index_recovery` tests for rebuild, repair-on-open, stale posting cleanup, tokenization/key contract, BM25 scoring behavior, phrase positions, hybrid RRF behavior, deterministic hybrid corpus coverage, debug planner/explain reporting, namespace/filter isolation, deterministic ordering, read-only non-repair, and import/export reconstruction.

### Changed

- Bumped Rust crate, Python crate, Python package metadata, and lockfile entries to `0.1.1`.
- Reaffirmed that v0.1.x remains embedded-first/local-first and that experimental surfaces are not part of the production-facing MVP.
- Validated TestPyPI upload and clean TestPyPI install for `vantadb-py==0.1.1`; production PyPI remains deferred.
- Repositioned the repo narrative around embedded persistent memory, cosine HNSW retrieval, and structured fields.
- Documented a process-scoped memory telemetry contract and added a controlled validation harness.
- Stabilized the embedded SDK boundary as the supported path for the Python binding.
- Expanded the embedded CLI from `put/get/list` to include rebuild and JSONL movement flows.
- Enabled public hybrid `text_query + query_vector` through RRF while keeping the existing text-only and vector-only behavior stable.
- Clarified that JSONL export/import is logical data movement, not physical backup.

### Deferred

- GitHub issue creation remains deferred until explicit approval.
- Public ranking explanation APIs, rich snippets/highlighting, and competitive hybrid-search parity claims.
- Stemming, stopwords, Unicode folding, and tokenizer evolution beyond `lowercase-ascii-alnum`.
- PyPI production publication, signing, and external distribution hardening.

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
