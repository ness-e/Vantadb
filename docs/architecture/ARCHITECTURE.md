# VantaDB Internal Architecture

This document reflects the current repo truth for `v0.1.x`. It describes the embedded core, the durability path, the current retrieval model, and the limits that still matter for product claims.

## 1. Product Boundary

VantaDB is currently an **embedded persistent memory engine** with:

- a local Rust core
- a stable embedded SDK boundary in `src/sdk.rs`
- an optional server wrapper around the same core

The current release should not be read as a universal multimodel platform, an enterprise control plane, or a full text+vector hybrid engine. Graph edges and structured metadata are part of the internal record model, but the primary product boundary today is persistent memory plus vector retrieval.

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

The current ANN path is:

- HNSW
- cosine similarity
- namespace-scoped vector retrieval
- equality filters over scalar metadata through derived payload indexes

What is **not** implemented as a shipped claim today:

- BM25
- RRF
- lexical-first hybrid ranking
- adaptive planner selection between text/vector paths

Any mention of “hybrid search” in the current repo should therefore be read as **vector retrieval plus structured filters**, not as BM25 + vector fusion.

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
- memory `export_namespace/export_all/import_file`
- vector search
- query
- add edge
- flush/close
- capabilities

Distribution hardening such as PyPI, wheels, signing, and installers is intentionally deferred until this boundary and the observability contract are stable.

## 6. Memory and Telemetry

`memory_limit_bytes` currently acts as a runtime budget hint for backend and mmap choices. It should **not** be read as a proven hard RSS ceiling.

Process-level telemetry is now treated as:

- process-scoped
- explicit about source and units
- separate from logical HNSW memory estimates
- separate from mmap residency and OS page cache

See [Memory Telemetry Contract](../operations/MEMORY_TELEMETRY.md) for the current metric schema and validation harness.

## 7. Mutation, Recovery, and Text Index Roadmap

- [Mutation and Recovery Protocol](MUTATION_RECOVERY_PROTOCOL.md) defines the canonical mutation order and rebuild behavior for ANN and derived indexes.
- [Minimal Text Index Design](TEXT_INDEX_DESIGN.md) defines the tokenizer/key scaffold required before BM25/RRF. It is not connected to public search yet.
