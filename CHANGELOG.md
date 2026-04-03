# CHANGELOG - v0.4.0 "Cognitive Sovereignty"

## [v0.4.0] - current
### ✨ Features
- **Cognitive Logic:** Hybrid execution layer supporting LISP S-Expressions (`src/eval`).
- **Sovereignty Module:** `DevilsAdvocate` write auditing to prevent contradictory or low-trust mutations.
- **Shadow Kernel:** Auditable memory via soft-deletion (Tombstones) and async Garbage Collection.
- **Cognitive Fuel:** Sandbox-protected execution of dynamic rules with resource limits.

## [v0.3.0]
### 🚀 Features
- **Neon Synapse:** SIMD-accelerated vector similarity using the `wide` crate for sub-millisecond KNN.
- **CP-Index:** Co-located Pre-filter HNSW implementation utilizing `u128` bitsets for combined semantic-relational pruning.
- **Node Topology:** Enhanced edge management for complex graph traversal.

## [v0.2.0]
### 🚀 Features
- **Obsidian Core:** Integrated RocksDB as the primary persistence engine.
- **Zero-Copy Serialization:** Buffer pinning and zero-alloc path for node retrieval via `bincode`.
- **Atomic WAL:** Write-Ahead Logging for crash-consistent state recovery.

## [v0.1.0] - Foundation
### Features
- **Phase 1 (Architecture):** `UnifiedNode` struct containing vectors, edges, and relational data. Custom `RwLock` in-memory engine.
- **Phase 2 (Query Engine):** EBNF `nom` parser resolving hybrid syntax (`FROM`, `SIGUE`, `~`, `RANK BY`).
- **Phase 3 (Integrations):** Added Resource Governor (OOM guard & Temperature execution). Scaffolded API handlers for Ollama.
