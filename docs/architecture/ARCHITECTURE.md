# VantaDB Internal Architecture

This document reflects the current repo truth for `v0.1.x`. It describes the embedded core, the durability path, the current retrieval model, and the limits that still matter for product claims.

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
- namespace-scoped retrieval with equality filters over scalar metadata through derived payload indexes

What is **not** implemented as a shipped claim today:

- phrase queries, snippets, positions, stemming, stopwords, or Unicode folding
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
- [Persistent Text Index Design](TEXT_INDEX_DESIGN.md) defines the BM25 text index and Hybrid Retrieval v1 RRF behavior for memory search.
