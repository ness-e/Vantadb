---
title: Web → VantaDB Concept Mapping
type: reference
status: stable
tags: [vantadb, mapping, web-developer, onboarding, architecture]
description: "Translation layer for web developers learning VantaDB. Maps familiar web/DB concepts to their VantaDB equivalents with file pointers."
---

# Web → VantaDB Concept Mapping

> **Purpose:** A translation dictionary for web developers (REST APIs, SQL, async, relational DBs) entering the VantaDB codebase. When your web instinct says X, look here to find the VantaDB equivalent.

## Core Operations

| Web / REST Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| `POST /data` (create) | `StorageEngine::insert()` / `VantaEmbedded::put()` | `src/storage/engine/ops.rs:138` |
| `GET /data/:id` (read) | `StorageEngine::get()` / `VantaEmbedded::get()` | `src/storage/engine/ops.rs:497` |
| `PUT /data/:id` (update) | `StorageEngine::insert()` (upsert by id) | `src/storage/engine/ops.rs:138` |
| `DELETE /data/:id` | `StorageEngine::delete()` + tombstone | `src/storage/engine/ops.rs:701` |
| `GET /data?batch` | `StorageEngine::get_many()` / `scan()` | `src/storage/engine/ops.rs:576` |
| `PATCH /data/:id` (partial) | `StorageEngine::insert()` with merge semantics | `src/storage/engine/ops.rs:138` |

## Transaction & Storage Layer

| Web / DB Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| `BEGIN / COMMIT` transaction | `WalRecord::Begin(u64)` / `WalRecord::Commit(u64)` | `src/wal.rs:30` |
| WAL (PostgreSQL) | `ShardedWal` + `WalWriter` with CRC32C checksums | `src/wal_sharded.rs`, `src/wal.rs` |
| `fsync` / durability | `WalWriter::sync()` → `File::sync_data()` | `src/wal.rs:381-386` |
| Connection pool deadlock | `parking_lot::RwLock` contention in `StorageEngine` (8+ locks) | `src/storage/engine/mod.rs:166-204` |
| Connection timeout | `FairMutex` with `try_lock_for(insert_lock_timeout_ms)` | `src/storage/engine/mod.rs:161` |
| Table / Collection | `BackendPartition` (8 typed partitions) | `src/backend.rs:30` |
| Row / Document | `UnifiedNode` (u128 id + fields + vector + edges) | `src/node.rs:705` |
| Index maintenance | `CPIndex` HNSW graph rebuild + auto-tune `ef_search` | `src/index/refresh.rs`, `src/index/auto_tune.rs` |

## Search & Indexing

| Web / DB Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| Elasticsearch full-text | BM25 inline with optional tantivy tokenizer | `src/text_index.rs`, `src/sdk/search.rs:136` |
| `SELECT ... WHERE field = x` | `ScalarIndex::lookup(field, value)` — O(1) hash index | `src/scalar_index.rs` |
| Full-text search ranking | BM25 scorer: `idf * ((tf * (k1 + 1)) / (tf + k1 * (1 - b + b * dl/avgdl)))` | `src/sdk/search.rs:227-231` |
| Hybrid search (vector + text) | `classify()` → `Hybrid` → BM25 + HNSW → `fuse_rrf()` | `src/planner.rs:68-77` |
| RRF fusion (ensemble) | `fuse_rrf()` with `RRF_K = 60.0`, candidate budget `[32, 256]` | `src/planner.rs:110-128` |

## Caching & Memory

| Web / DB Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| Redis / Memcached | `thread_local! { RefCell<LruCache::new(64)> }` (Python bindings) | `vantadb-python/src/lib.rs:34` |
| ORM lazy loading | BM25 `doc_stats_cache` — lazy `HashMap` under RwLock | `src/sdk/search.rs` |
| In-memory cache layer | `RwLock<HashMap<u128, UnifiedNode>>` volatile cache (hot nodes) | `src/storage/engine/mod.rs:166` |
| Eviction policy (LRU) | Eviction by `hits`/`last_accessed` score + `eviction_ratio` weights | `src/config.rs:193` (VantaConfig) |
| Rate limiting | `MemoryGovernor` with watermarks + backpressure | `src/memory_governor.rs`, `src/governor.rs` |

## Concurrency & Consistency

| Web / DB Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| Pessimistic lock | `parking_lot::RwLock::write()` — blocks readers | `src/storage/engine/mod.rs:166` |
| Optimistic lock | `ArcSwap<CPIndex>::load()` — RCUA-style, lock-free reads | `src/storage/engine/mod.rs:158` |
| MVCC / version conflict | `ConflictResolver` with version vectors + three-way merge | `src/governance/conflict.rs:131` |
| Eventual consistency | `ConsistencyBuffer<T>` with TTL + backpressure + batch flush | `src/governance/consistency.rs:46` |
| Atomic counter | `AtomicU64` — next_id, next_txn_id, last_query_timestamp | `src/storage/engine/mod.rs:168-170` |
| Concurrent hash map | `DashMap<u128, HnswNode>` (HNSW) / `DashSet<(u128, u128)>` (edges) | `src/index/graph.rs:277`, `src/edge_index.rs:7` |

## Graph & Relations

| Web / DB Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| Neo4j graph traversal | `GraphTraverser::bfs_traverse()` / `dfs_traverse()` | `src/graph.rs:26-86` |
| Adjacency list | `EdgeIndex` — `DashSet<(u128, u128)>` directed edges | `src/edge_index.rs:7` |
| Edge with properties | `Edge { target: u128, label: String, weight: f32 }` on `UnifiedNode` | `src/node.rs:376-384` |
| Topological sort | `GraphTraverser::topological_sort()` with cycle detection | `src/graph.rs:94-107` |
| `MATCH (a)-[r]->(b)` (Cypher) | Edge index insert + GraphTraverser BFS | `src/edge_index.rs:7`, `src/graph.rs:26` |

## Bindings & FFI (for web devs learning Rust FFI)

| Web / Language Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| Python C extension | PyO3 bindings with `py.detach()` (GIL released) | `vantadb-python/src/lib.rs:717` |
| WASM / Browser | `wasm-bindgen` + `serde-wasm-bindgen` (full serialize) | `vantadb-wasm/src/lib.rs:242` |
| NumPy array (zero-copy) | `VantaVector.__array_interface__` (⚠️ raw ptr, potential UB) | `vantadb-python/src/lib.rs:1754` |
| JS async/await | `VantaDB` WASM struct + OPFS `createWritable` | `vantadb-wasm/src/lib.rs:242` |
| JSON serialization | `postcard` (binary compact) for WAL/index; `serde_json` for exports | `src/wal.rs`, `src/index/serialize.rs` |

## CI/CD Analogy (for web devs)

| Web / DevOps Concept | VantaDB Equivalent | Architecture |
|---|---|---|
| GitHub Actions fast gate | `ci-rust-10.yml` — build + lint + fast tests (< 5 min) | `.github/workflows/ci-rust-10.yml` |
| E2E / stress tests | `heavy-certification-50.yml` — weekly, up to 2 hr | `.github/workflows/heavy-certification-50.yml` |
| npm publish | `release-npm-61.yml` → `wasm-pack` + `npm publish` | `.github/workflows/release-npm-61.yml` |
| PyPI publish | `release-wheels-60.yml` → `maturin` + `twine` | `.github/workflows/release-wheels-60.yml` |

## Quick Decision Tree

```
¿Tienes un bug? 
  ├─ ¿Falla en CI?          → Revisa P0-1 (cancel-in-progress), P0-2 (continue-on-error)
  ├─ ¿Test flaky?           → Crea GitHub Issue con tag "flaky", no lo silencies
  └─ ¿Comportamiento raro?  → Carga skill systematic-debugging

¿Necesitas leer/escribir datos?
  ├─ Un solo nodo           → StorageEngine::insert() / get() / delete()
  ├─ Muchos nodos           → put_batch() / get_many() / scan()
  └─ Con búsqueda vectorial → VantaEmbedded::search() → planner.classify()

¿Problema de concurrencia?
  ├─ Deadlock sospechado    → Revisa parking_lot::RwLock en StorageEngine (8+ instancias)
  ├─ Race condition         → ¿DashMap / DashSet? Si no, revisa AtomicU64 / ArcSwap
  └─ Perdida de datos       → WAL: revisa ShardedWal::append() + WalWriter::sync()
```
