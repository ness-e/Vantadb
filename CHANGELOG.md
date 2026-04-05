# CHANGELOG

## [v0.5.0] - 2026-04-05 "Quantum Cognition" (EN PROGRESO)
### 🔄 Version Bump
- Transición de v0.4.0 a v0.5.0 tras completar las 11 fases del Cognitive OS (Fases 20-30).
- Próximas fases: Uncertainty Zones, Synaptic Depression, Contextual Priming, MMap Neural Index.

---

## [v0.4.0] - 2026-04-05 "Cognitive Sovereignty"
### ✨ Features
- **Memory Rehydration Protocol (Fase 30):** Arqueología Semántica — recuperación zero-copy de nodos archivados en `shadow_kernel` via `StorageEngine::rehydrate()`. Flag `NodeFlags::REHYDRATED` para trazabilidad de provenance. `ExecutionResult::StaleContext` no-bloqueante cuando `TrustScore < 0.4`. `SleepWorker` purga nodos archaeológicos en fase REM.
- **NeuLISP VM Bytecode (Fase 29):** Máquina virtual con pila de floats y vectores. Opcodes: `OpPushFloat`, `OpPushVector`, `OpTrustCheck`, `OpVecSim`, `OpRehydrate`. Retorno probabilístico `(Value, TrustScore)`.
- **Inference Optimization (Fase 28):** Bloom Filters nativos RocksDB con L0 Pinning en RAM para `default` y `deep_memory`. Protocolo MCP sobre STDIO (JSON-RPC 2.0) con herramientas `query_lisp`, `search_semantic`, `inject_context`, `read_axioms`.
- **Modo Camaleón (Fase 27):** Auto-detección de hardware (`cpufeatures` + `sysinfo`). Perfiles `Survival/Performance/Enterprise`. Ajuste dinámico de RocksDB cache y `cortex_ram` capacity (25% RAM).
- **Neural Summarization (Fase 26):** SleepWorker Stage 3 agrupa nodos "Oníricos" por thread e invoca Ollama (`summarize_context`) para comprimir en Neurona de Resumen (`deep_memory`). Linaje semántico via campo `ancestors`.
- **Lobe Segmentation (Fase 25):** 4 Column Families: `default`, `shadow_kernel`, `deep_memory`, `tombstones`. Compresión Zstd diferenciada.
- **Memory Hierarchy (Fase 24):** Dualidad `STNeuron` (RAM) / `LTNeuron` (disco). Promoción dinámica al alcanzar `hits >= 50`.
- **Sovereignty Governance (Fase 23):** `DevilsAdvocate` + `TrustArbiter`. Borrados atómicos via `WriteBatch`.
- **NeuLISP Cognition (Fase 22):** Parser S-Expressions, `LispSandbox` con Cognitive Fuel (1000 steps), operador de similitud `~`.
- **SIMD Optimization (Fase 21):** `wide::f32x8` para cosine similarity. Fallback escalar para hardware sin AVX.
- **SleepWorker (Fase 20):** Daemon circadiano con Olvido Bayesiano, migración STN→LTN, Presupuesto de Amígdala (5%).

### 🐛 Fixes
- `StorageEngine::consolidate_node()` — fix del gap HNSW (sincroniza disco + index).
- `rehydrate()` — corregida verificación `is_tombstone()` (shadow_kernel almacena nodos originales sin flag).
- MCP/HTTP handlers — cobertura exhaustiva de `ExecutionResult::StaleContext`.

---

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
- **Phase 1 (Architecture):** `UnifiedNode` struct containing vectors, edges, and relational data.
- **Phase 2 (Query Engine):** EBNF `nom` parser resolving hybrid syntax (`FROM`, `SIGUE`, `~`, `RANK BY`).
- **Phase 3 (Integrations):** Added Resource Governor (OOM guard & Temperature execution).
