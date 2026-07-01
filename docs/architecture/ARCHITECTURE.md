---
title: VantaDB Internal Architecture
type: architecture
status: active
tags: [vantadb, architecture]
last_reviewed: 2026-07-01
---

# VantaDB Internal Architecture

This document reflects the current repo truth for `v0.1.x`. It describes the embedded core, the durability path, the current retrieval model, and the limits that still matter for product claims.

---

## Design Principles

### 1. Embedded-First

VantaDB is an **embedded library**, not a service. The core (`vantadb-core`) has zero network dependencies. The HTTP server lives in `vanta-cli server` (in-process, behind `server` feature flag).

```
┌─────────────────────────────────────────┐
│         Application (Python/Rust)        │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │     vantadb-core (linked library)   │ │
│  │  ┌──────┐  ┌──────┐  ┌──────────┐ │ │
│  │  │ [[wal|WAL]]  │  │ [[hnsw|HNSW]] │  │ Storage  │ │ │
│  │  └──────┘  └──────┘  └──────────┘ │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### 2. Canonical Data + Derived Indexes

All data has a single source of truth (canonical storage). Indexes are **ephemeral materializations** — if corrupted, they can be rebuilt from canonical data.

```
Source of Truth (Canonical):
├── Documents (text + metadata)
├── Vectors (embeddings)
└── Graph (edges)

Derived Indexes (Rebuildable):
├── [[hnsw|HNSW]] (vector ANN search)
├── [[bm25|BM25]] (lexical search)
└── Payload indexes (structured filters)
```

### 3. Zero-Cost Abstractions

Rust enables high-level abstractions with zero runtime overhead:
- **Traits** for static polymorphism
- **Zero-copy** where possible ([[mmap]])
- **[[simd|SIMD]]** for vector operations

### 4. Durability Before Performance

Write path order — NEVER acknowledge before fsync:

1. Append mutation to [[wal|WAL]]
2. fsync() the [[wal|WAL]] ← **DURABILITY GUARANTEED**
3. Apply to storage backend ([[fjall|Fjall]]/[[rocksdb|RocksDB]])
4. Update derived indexes ([[hnsw|HNSW]], [[bm25|BM25]])
5. ACK to client

---

## WAL Binary Layout

The Write-Ahead Log guarantees durability before any mutation is applied to storage.

### Record Structure

```
┌─────────────────────────────────────┐
│         WAL Record                   │
├─────────────────────────────────────┤
│ Header (8 bytes)                    │
│ ├── Length: u32                     │
│ ├── Type: u8 (Insert/Delete/Update) │
│ └── Flags: u8                       │
├─────────────────────────────────────┤
│ Payload (variable)                  │
│ ├── Key: [u8]                       │
│ ├── Vector: [f32]                   │
│ ├── Text: [u8]                      │
│ └── Metadata: [u8]                  │
├─────────────────────────────────────┤
│ Checksum: u32 ([[crc32c|CRC32C]])              │
└─────────────────────────────────────┘
```

### Write Flow

```
1. Serialize mutation
2. Compute CRC32C of payload
3. Append to wal.log
4. fsync() ← DURABILITY GUARANTEED
5. Apply to Fjall/RocksDB
6. Update indexes (HNSW, BM25)
7. ACK to client
```

### WAL Compaction

Automatic when accumulated size exceeds 256 MB (`compact_wal()`):
- Rotates obsolete WAL segments (post-checkpoint)
- Exposed via `vanta-cli wal compact`
- Zero interruption to read/write operations

---

## Storage Backend: [[fjall|Fjall]] vs [[rocksdb|RocksDB]]

| Feature | [[fjall|Fjall]] (Default) | [[rocksdb|RocksDB]] (Fallback) |
|---------|-----------------|-------------------|
| Language | 100% Rust | C++ (C bindings) |
| Build Time | ~30s | ~5-10min |
| Dependencies | Zero | CMake, Clang, libstdc++ |
| Memory Safety | Safe Rust | `unsafe` in bindings |
| MVCC | ✅ Native | ✅ Supported |
| LSM-Tree | ✅ | ✅ |

The `StorageBackend` trait abstracts the KV layer:

```rust
pub trait StorageBackend: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
}
```

---

## Data Flow Diagrams

### Document Insert Path

```
Client: db.put("doc1", vector, text, metadata)
    │
    ▼
┌────────────────────────┐
│ 1. Validate inputs     │
│    - key not empty     │
│    - valid vector dim  │
│    - valid metadata    │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 2. Serialize mutation  │
│    Mutation::Insert {  │
│      key, vector,      │
│      text, metadata    │
│    }                   │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 3. Append to WAL       │
│    - Compute CRC32C    │
│    - Write record      │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 4. fsync() of WAL      │ ← DURABILITY
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 5. Apply to [[fjall|Fjall]]      │
│    - Insert document   │
│    - Insert vector     │
│    - Insert metadata   │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 6. Update indexes      │
│    - [[hnsw|HNSW]]: add vector  │
│    - [[bm25|BM25]]: index text  │
└──────────┬─────────────┘
           │
           ▼
      ACK to client
```

### Hybrid Search Path

```
Client: db.search(vector, text, top_k=10)
    │
    ├─▶ HNSW Index
    │   └─▶ Candidate List 1: [doc5, doc12, doc23, ...]
    │
    ├─▶ BM25 Index
    │   └─▶ Candidate List 2: [doc3, doc7, doc12, doc45, ...]
    │
    └─▶ RRF Fusion
        └─▶ Unified Ranking: [doc12, doc7, doc45, ...]
            │
            ▼
        Return top-K to client
```

---

## System Architecture (Layered)

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 5: SDK / API                                          │
│  ├── Python SDK (PyO3 bindings)                             │
│  ├── Rust SDK (native VantaEmbedded API)                    │
│  ├── TypeScript SDK (WASM)                                  │
│  └── MCP Server (agent protocol)                            │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: Query Engine                                       │
│  ├── Query Planner (AST + optimization)                     │
│  ├── Hybrid Search (HNSW + BM25 + RRF)                     │
│  └── Graph Traversal (multi-hop)                            │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Indexes                                            │
│  ├── HNSW Index (vector ANN)                                │
│  ├── BM25 Index (lexical)                                   │
│  └── Payload Indexes (filters)                              │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Storage Engine                                     │
│  ├── WAL (Write-Ahead Log)                                  │
│  ├── Fjall Backend (default, 100% Rust LSM-tree)            │
│  └── RocksDB Backend (fallback via C bindings)              │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Persistence                                        │
│  ├── mmap (memory-mapped I/O)                               │
│  ├── fsync (durability)                                     │
│  └── CRC32C (integrity)                                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Component Map

| Component | Source File | Responsibility |
|-----------|------------|----------------|
| **VantaEmbedded** | `src/sdk.rs` | Public API boundary |
| **StorageEngine** | `src/storage.rs` | Storage orchestration |
| **WalWriter** | `src/wal.rs` | Write-ahead log |
| **HnswIndex** | `src/index.rs` | Vector ANN index |
| **Bm25Index** | `src/text_index.rs` | Lexical search index |
| **FjallBackend** | `src/backends/fjall_backend.rs` | LSM-tree backend |
| **UnifiedNode** | `src/node.rs` | Unified data model |

---

## HNSW Index

**Purpose:** Approximate nearest neighbor (ANN) search in logarithmic time.

```
Layer 2 (sparsest):
    [A] ────────── [D]

Layer 1 (intermediate):
    [A] ─── [B] ─── [D]
     │       │       │
    [E] ─── [C] ─── [F]

Layer 0 (densest, all vectors):
    [A]─[B]─[C]─[D]─[E]─[F]─[G]─[H]─[I]─[J]
```

**Parameters:**
- **M:** Max connections per node (default: 16)
- **ef_construction:** Candidates during construction (default: 200)
- **ef_search:** Candidates during search (default: 100)

**Persistence:** Full graph memory-mapped (mmap) → instant load.

## BM25 Index

**Purpose:** Keyword-based lexical search via inverted index.

```
Inverted Index:
"database" → [doc1, doc3, doc7, ...]
"vector"  → [doc1, doc8, doc20, ...]
"search"  → [doc3, doc7, doc12, ...]
```

---

## Unified Data Model

The `UnifiedNode` in `src/node.rs` is the canonical internal representation:

```rust
pub struct UnifiedNode {
    pub id: u64,
    pub bitset: u128,
    pub semantic_cluster: u32,
    pub flags: NodeFlags,
    pub vector: VectorRepresentations,
    pub epoch: u32,
    pub edges: Vec<Edge>,
    pub relational: RelFields,
    pub tier: NodeTier,
    pub hits: u32,
    pub last_accessed: u64,
    pub confidence_score: f32,
    pub importance: f32,
    pub ext_metadata: HashMap<String, Vec<u8>>,
}
```

The product-level memory model (`VantaMemoryInput`, `VantaMemoryRecord`, etc.) provides a simpler interface over this internal representation.

---

## WASM Support

The core compiles for `wasm32-wasip1` via conditional compilation:

| Dependency | Native | WASM | Strategy |
|-----------|--------|------|----------|
| **sysinfo** | Real | Stub | Optional feature |
| **memmap2** | mmap | Vec-backed shim | Optional, shim in `src/wasm/mmap.rs` |
| **fs2** | File locking | Ok stub | Optional, empty stub |
| **prometheus** | Real metrics | cfg-gated statics | `#[cfg(feature = "prometheus")]` |
| **rayon** | Thread pool | Sequential fallback | Optional, `iter().map().collect()` |

Browser target (`wasm32-unknown-unknown`) uses `web_time::SystemTime` to avoid panics from `std::time::SystemTime::now()` (unavailable in browsers).

## 1. Product Boundary

VantaDB is currently an **embedded persistent memory engine** with:

- a local Rust core
- a stable embedded SDK boundary in `src/sdk.rs`
- an optional server wrapper around the same core

The current release should not be read as a universal multimodel platform, an enterprise control plane, or a competitive full-text search platform. Graph edges and structured metadata are part of the internal record model, but the primary product boundary today is embedded persistent memory with vector, BM25 text-only, and Hybrid Retrieval v1.

## 2. Record Model

The internal core data model is `UnifiedNode` in `src/node.rs`. Each node can hold:

- a unique `u64` identifier
- a vector payload
- typed relational fields
- local graph edges
- access and confidence metadata
- flags and tiering state

This model allows the engine to keep vector, metadata, and edge information in a single logical record. It does **not** imply that every feature is equally productized in the current SDK surface.

The product-level memory model is separate and lives in `src/sdk.rs`:

- `VantaMemoryInput`
- `VantaMemoryRecord`
- `VantaMemoryMetadata`
- `VantaMemoryListOptions`
- `VantaMemorySearchRequest`

Memory identity is deterministic over `namespace + "\0" + key`. `UnifiedNode` remains internal storage representation, not the public product API.

## 3. Storage and Durability

Durability is built around three layers:

1. **`StorageBackend` trait**: abstracts the KV layer.
2. **Fjall / RocksDB backends**: Fjall is the default; RocksDB remains an explicit fallback path.
3. **WAL + VantaFile + HNSW artifact**: node mutations are journaled, canonical vector payloads are stored in VantaFile, and the ANN index can be rebuilt if the derived artifact is missing.

Current repo guarantees that matter:

- WAL replay exists for crash recovery
- HNSW reconstruction from VantaFile exists
- manual ANN rebuild is available through the SDK and CLI
- namespace/payload indexes are derived state and can be rebuilt from canonical records
- JSONL export/import is available for namespace-scoped or full memory movement
- the server path is not the source of truth; it wraps the embedded core

## 4. Retrieval Model Today

The current memory retrieval paths are:

- vector-only retrieval using HNSW/cosine over canonical memory records
- BM25 text-only retrieval over the persistent text index
- Hybrid Retrieval v1 using a minimal planner and RRF over independently ranked vector and BM25 candidates
- basic quoted phrase matching over persisted token positions
- namespace-scoped retrieval with equality filters over scalar metadata through derived payload indexes

What is **not** implemented as a shipped claim today:

- rich snippets/highlighting, public ranking explanations, stemming, stopwords, or Unicode folding
- learned/adaptive ranking or ranking explanations
- competitive hybrid-search parity claims
- server-first search platform behavior

Any mention of hybrid search in the current repo should therefore be read as **Hybrid Retrieval v1**: BM25 plus vector rankings fused with RRF under a simple deterministic planner, not as a broad search platform claim.

## 5. Embedded SDK Boundary

The stable embedded boundary now lives in `src/sdk.rs`. It exists to keep external consumers away from:

- `StorageEngine`
- `Executor`
- direct HNSW lock access
- internal hardware and storage plumbing

The Python binding routes through this boundary and currently exposes:

- open
- legacy node insert/get/delete
- memory `put/get/delete/list/search`
- manual `rebuild_index`
- text-index `audit_text_index` and `repair_text_index`
- memory `export_namespace/export_all/import_file`
- operational metrics
- vector search
- query
- add edge
- flush/close
- capabilities

Distribution hardening now has wheel CI, version-coherence checks, a manual
TestPyPI gate, tag-gated production publishing, and Sigstore signing. Actual
PyPI publication remains a release-manager action outside normal development
tasks.

## 6. Memory and Telemetry

`memory_limit_bytes` currently acts as a runtime budget hint for backend and mmap choices. It should **not** be read as a proven hard RSS ceiling.

Process-level telemetry is now treated as:

- process-scoped
- explicit about source and units
- explicit about logical HNSW estimates through `hnsw_logical_bytes`
- explicit about mmap residency when available through `mmap_resident_bytes`
- separate from OS page cache and backend allocator internals

See [Memory Telemetry Contract](../operations/MEMORY_TELEMETRY.md) for the current metric schema and validation harness.

## 7. Mutation, Recovery, and Text Index Roadmap

- [Mutation and Recovery Protocol](MUTATION_RECOVERY_PROTOCOL.md) defines the canonical mutation order and rebuild behavior for ANN and derived indexes.
- [Persistent Text Index Design](TEXT_INDEX_DESIGN.md) defines the BM25 text index, phrase-position support, and Hybrid Retrieval v1 RRF behavior for memory search.
