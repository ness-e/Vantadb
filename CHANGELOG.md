# CHANGELOG - v0.4.0 "Cognitive Sovereignty"

## [v0.4.0] - current
### ✨ Features
- **NeuLISP (Cognitive Inference):** Evolution of LISP S-Expressions to support native probabilistic inference returning `(Value, TrustScore)`. Includes `OP_VEC_SIM` for SIMD-accelerated execution of the Similitude Operator (`~`) and `OP_TRUST_CHECK`.
- **Biological Governance (Amygdala Budget):** SleepWorker now enforces a strict 5% budget on RAM-resident nodes with high `semantic_valence` (>0.8) to protect them from Bayesian Forgetfulness and consolidation, emulating human core memory.
- **Multi-Lobe Memory Architecture (RocksDB):** Data partitioning now extends explicitly to `default`, `shadow_kernel`, `tombstones`, and `deep_memory` Column Families.
- **Cognitive Sovereignty:** `DevilsAdvocate` write auditing to prevent contradictory or low-trust mutations.
- **Cognitive Fuel:** Sandbox-protected execution of dynamic rules with resource limits.
- **Neural Summarization (Fase 26):** SleepWorker Stage 3 clusters "Onírico" nodes by thread and invokes Ollama (`summarize_context`) to compress them into a single Neurona de Resumen in `deep_memory`, preserving semantic lineage via `ancestors` field for future Archeological Rehydration.
- **HNSW Consolidation Fix:** `StorageEngine::consolidate_node()` now atomically persists nodes AND updates the in-memory HNSW index, preventing index-disk divergence during circadian maintenance.
- **Deep Memory Writer:** New `StorageEngine::insert_to_cf()` method for direct writes to named Column Families (e.g. `deep_memory`), enabling Lobe-specific persistence bypassing the default CF.

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
