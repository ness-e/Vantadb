---
title: Bitacora — Development Log
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-07-01
---

# bitacora — Development Log

## Julio 2026

### Semana 1 (2026-07-01) — Documentation Audit, Rust Examples & Certification

Today's work focused on auditing and improving project documentation:

- **AUD-06 — Documentation audit:** Reviewed all docs for consistency, fixed mixed-language content, and expanded coverage.
- **AUD-07 — FAQ created:** `docs/FAQ.md` added with common questions on general usage, configuration, and troubleshooting.
- **AUD-08 — Rust examples shipped:** Created 4 compilable examples in `examples/rust/`:
  - `basic.rs` — CRUD operations with `VantaEmbedded`
  - `hybrid.rs` — Combined text + vector search
  - `graphrag.rs` — Knowledge graph traversal with BFS
  - `concurrent.rs` — Multi-threaded concurrent access
- **AUD-09 — Memory Telemetry doc fixed:** `docs/operations/MEMORY_TELEMETRY.md` title and content unified to English.
- **AUD-10 — Master Index verified:** All internal links confirmed valid.
- **AUD-11 — Performance optimizations:** Review and tune HNSW insert lock timeout defaults, reduce allocation hot paths in planner.
- **AUD-12 — Dead code removal:** Stripped unused imports, deprecated `VantaOpenOptions`, removed commented-out hardware profile references.
- **AUD-13 — Config hardening:** Validated all `VantaConfig` defaults, added env-var overrides for lock timeouts.
- **AUD-14 — Documentation improvements:** Unified terminology across `CONFIGURATION.md`, `DURABILITY_GUARANTEES.md`, and SDK references.
- **AUD-15 — Test expansion:** Extended WAL resilience tests, added edge cases for zero-vector and TTL-expired searches.
- **AUD-16 — Doc consolidation:** Archived redundant Spanish MPTS sections to `docs/archive/`, removed stale wikilinks.
- **AUD-17 — CI hardening:** Enabled failpoint-based chaos tests in CI, added `cargo audit` step.
- **AUD-18 — Changelog update:** Backfilled missing entries in `CHANGELOG.md` for v0.1.3–v0.1.5.

---

### 2. Structural Risks and Technical Debt Detected

Under strict scalability and performance scrutiny, there are friction points that require attention before consolidating version `v0.2.0`:

- **Ingestion Bottleneck (Single-Writer):** Although the Python SDK exposes a parallelized `put_batch` via Rayon, the underlying HNSW graph construction still exhibits fine-grained locking (despite the transition from `RwLock` to `DashMap` and `ArcSwap`). Under high-density continuous insertion workloads, this becomes the main limiting factor compared to server-based vector databases.

- **Primitive Default BM25 Tokenization:** The base text indexer (`lowercase-ascii-alnum`) is insufficient for multilingual production searches. Although the `advanced-tokenizer` feature based on Tantivy (v4 schema) exists, keeping the tokenizer simple by default postpones stemming, stopwords, and Unicode folding issues for end-users.

- **Process Concurrency Management:** The engine relies on an exclusive lock file (`.vanta.lock`). Although resilience tests (`file_locking_stress.rs`) exist for stale locks, in environments where concurrent agents attempt to instantiate the engine from multiple independent Python processes, hard contention failures (`DatabaseBusy`) will occur.

- **Lack of Point-in-Time Recovery (PITR) Support:** Fjall does not natively support checkpoints like RocksDB. The current backup policy relies on logical backups (JSONL) or cold copies (Cold Copy), adding latency and operational complexity if the volume of the agent's memory data grows significantly.
  
### Strategic Direction (Towards v0.2.0)

The project is at an optimal stage for transitioning core development to distribution stabilization. Execution priorities should be aligned as follows:

1. **Promotion of Search Quality v2:** Transition the `advanced-tokenizer` (Tantivy) as the default option in release builds to ensure semantic parity in lexical retrieval. Expose snippet and highlighting capabilities in the public Python API.

2. **Pilot Program in Real Environments:** Freeze the addition of new architectural features (experimental LISP/IQL) and focus telemetry on the real behavior of heap memory drift under AI agents in Phase 3.4.

3. **CI/CD Pipeline Hardening:** Complete SLSA Level 2 certification via GitHub Attestations and execute the transition of the TestPyPI flow to the PyPI production registry to enable zero-friction adoption in the local-first community.