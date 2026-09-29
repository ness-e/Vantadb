---
title: Glosario VantaDB
kind: index
status: stable
description: "Complete index of technical concepts: WAL, HNSW, BM25, FFI, mmap, GIL, RRF, Fjall, RocksDB and more"
aliases: [Glossary, Concepts, Technical Reference, Dictionary]
tags: [vantadb, glossary, reference, concepts, technical]
type: glossary
last_reviewed: "2026-09-15"
links: master-index.md
---

# Glossary of Technical Concepts — VantaDB

> Complete reference of all technical, architectural, and product concepts mentioned across the VantaDB documentation.

---

## Product & Architecture Concepts

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [embedded](./embedded.md) | Database operating in-process within the application | Core product identity |
| [local-first](./local-first.md) | Design philosophy prioritizing local operations over network | Fundamental architectural principle |
| [transactional](./transactional.md) | ACID guarantee over data mutations | Core durability contract |
| [zero-config](./zero-config.md) | Usage experience without manual setup or config | Competitive advantage over alternatives |
| [rag](./rag.md) | Retrieval-Augmented Generation | Primary use case |
| [vectors](./vectors.md) | High-dimensional numerical representations | Central data type |
| [graph](./graph.md) | Node-edge structure with properties | Complementary data model |

---

## Persistence Mechanisms

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [persistence](./persistence.md) | Ability to retain data beyond process lifecycle | General durability concept |
| [wal](./wal.md) | Write-Ahead Log — mutation journaling | Durability guarantee prior to ACK |
| [fjall](./fjall.md) | 100% Rust LSM-tree engine | Default canonical backend |
| [rocksdb](./rocksdb.md) | LSM-tree engine by Meta (C++) | Alternative backend / benchmarking |
| [mmap](./mmap.md) | Memory-Mapped I/O | Zero-copy vector reading |
| [fsync](./fsync.md) | Physical disk synchronization | Real persistence guarantee |
| [crc32c](./crc32c.md) | Hardware-accelerated checksum | WAL record integrity |
| [lsm-tree](./lsm-tree.md) | Log-Structured Merge-Tree | Underlying storage engine pattern |
| [mvcc](./mvcc.md) | Multi-Version Concurrency Control | Transactional isolation |
| [crdt](./crdt.md) | Conflict-free Replicated Data Types | Distributed convergence for multi-node scaling |
| [bincode](./bincode.md) | Legacy binary format, no longer a dependency | Superseded by postcard for WAL and index state |
| [serde](./serde.md) | Rust serialization/deserialization framework | JSON for HTTP API, postcard for disk storage |

---

## Indexes & Search

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [vector-search](./vector-search.md) | Semantic similarity search using vectors | Primary retrieval method |
| [lexical-search](./lexical-search.md) | Exact keyword matching search | Complement to vector-search |
| [hybrid-search](./hybrid-search.md) | Unified vector + lexical retrieval | Key differentiator |
| [hnsw](./hnsw.md) | Hierarchical Navigable Small World | Main vector index for ANN |
| [bm25](./bm25.md) | Best Matching 25 — lexical scoring | Full-text index |
| [rrf](./rrf.md) | Reciprocal Rank Fusion | Fusion of hybrid rankings |
| [vector-similarity](./vector-similarity.md) | Distance metrics between vectors (cosine, L2, dot) | Distance computation |
| [ann](./ann.md) | Approximate Nearest Neighbor | Algorithm class for vector search |
| [payload-indexes](./payload-indexes.md) | Metadata-field filtering | High-performance query filtering |

---

## Concurrency & Safety

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [python-sdk](./python-sdk.md) | Python bindings generated via PyO3 | Primary end-user interface |
| [gil](./gil.md) | Global Interpreter Lock (Python) | CPU bottleneck bypassed by PyO3 |
| [ffi](./ffi.md) | Foreign Function Interface | Python-Rust boundary |
| [pyo3](./pyo3.md) | Rust/Python binding framework | SDK foundation |
| [file-locking](./file-locking.md) | Process-level advisory file locks | Multi-process corruption prevention |
| [rwlock](./rwlock.md) | Read-write lock pattern | Core engine concurrency |

---

## Operations & CI/CD

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [ci-cd](./ci-cd.md) | Continuous Integration / Deployment | Release automation |
| [benchmarks](./benchmarks.md) | Standardized performance testing | Validating performance claims |
| [chaos-testing](./chaos-testing.md) | Controlled failure injection | WAL durability validation |
| [failpoints](./failpoints.md) | Error injection points | Recovery path testing |
| [oidc](./oidc.md) | OpenID Connect | Secure publishing to PyPI |
| [sigstore](./sigstore.md) | Artifact signing | Verifiable build provenance |
| [slsa](./slsa.md) | Supply-chain Levels for Software Artifacts | Build security framework |
| [opentelemetry](./opentelemetry.md) | OpenTelemetry tracing and metrics | System observability |

---

## Use Cases & Protocols

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [rag](./rag.md) | Retrieval-Augmented Generation | Primary use case |
| [graphrag](./graphrag.md) | RAG with graph traversal | Reducing context tokens by 40-60% |
| [ai-agents](./ai-agents.md) | Autonomous agent systems with memory | Core target user profile |
| [mcp](./mcp.md) | Model Context Protocol | Integration with IDEs and agents |
| [wasm](./wasm.md) | Binary instruction format for stack-based VM | Browser and edge runtime via Rust compilation target |

---

## Performance & Optimization

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [recall](./recall.md) | Quality metric: % of true neighbors retrieved | HNSW index validation |
| [latency](./latency.md) | Response time (p50, p95, p99) | Performance metric |
| [memory-efficiency](./memory-efficiency.md) | RAM footprint per indexed vector | Resource optimization |
| [simd](./simd.md) | Single Instruction, Multiple Data | Distance computation acceleration |
| [zero-copy](./zero-copy.md) | Avoid memory duplication | High-throughput reading |
| [dashmap](./dashmap.md) | Concurrent sharded hash map | Low-contention concurrency |
| [backpressure](./backpressure.md) | Flow control under high load | Out-of-memory (OOM) prevention |

---

## Enterprise Features (Planned)

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [rbac](./rbac.md) | Role-Based Access Control | Granular security |
| [multi-tenancy](./multi-tenancy.md) | Tenant isolation | VantaDB Cloud architecture |

---

---

## Competitors & Ecosystem

| Concept | Short Description | Relevance in VantaDB |
|---------|-------------------|----------------------|
| [qdrant](./qdrant.md) | Rust vector search engine, client-server architecture | Competitor — VantaDB differentiates on embedded/local-first |
| [lancedb](./lancedb.md) | Open-source embedded vector database on Lance columnar format | Competitor — VantaDB differentiates on hybrid search + schema-less |

---

## Quick Navigation

### By Category

**New to VantaDB?** Start with:
1. [embedded](./embedded.md) → Understand the product identity
2. [local-first](./local-first.md) → Grasp the core philosophy
3. [rag](./rag.md) → Learn the primary use case
4. [persistence](./persistence.md) → Understand durability guarantees
5. [vector-search](./vector-search.md) → Explore similarity search
6. [hybrid-search](./hybrid-search.md) → Read about the hybrid search differentiator

**Technical Profile?** Deep dive into:
- [fjall](./fjall.md) vs [rocksdb](./rocksdb.md) → Storage backend selection
- [gil](./gil.md) + [ffi](./ffi.md) + [pyo3](./pyo3.md) + [python-sdk](./python-sdk.md) → Python-Rust concurrency
- [mmap](./mmap.md) + [fsync](./fsync.md) → Persistence and performance
- [mvcc](./mvcc.md) + [lsm-tree](./lsm-tree.md) → Storage internals
- [recall](./recall.md) + [latency](./latency.md) + [memory-efficiency](./memory-efficiency.md) → Performance metrics

**Product Profile?** Focus on:
- [zero-config](./zero-config.md) → Core user benefit
- [transactional](./transactional.md) → Reliability contract
- [vectors](./vectors.md) + [graph](./graph.md) → Multimodal data model
- [hybrid-search](./hybrid-search.md) + [rrf](./rrf.md) → Search quality

---

## Concept Relationships

```mermaid
graph TD
    E[embedded](./embedded.md) --> LF[local-first](./local-first.md)
    LF --> ZC[zero-config](./zero-config.md)
    E --> T[transactional](./transactional.md)
    T --> WAL[wal](./wal.md)
    WAL --> CRC[crc32c](./crc32c.md)
    WAL --> FS[fsync](./fsync.md)
    WAL --> FJ[fjall](./fjall.md)
    FJ --> LSM[lsm-tree](./lsm-tree.md)
    LSM --> MVCC[mvcc](./mvcc.md)
    
    P[persistence](./persistence.md) --> WAL
    P --> FJ
    P --> FS
    
    RAG[rag](./rag.md) --> V[vectors](./vectors.md)
    RAG --> G[graph](./graph.md)
    
    BV[vector-search](./vector-search.md) --> HNSW[hnsw](./hnsw.md)
    BL[lexical-search](./lexical-search.md) --> BM25[bm25](./bm25.md)
    BH[hybrid-search](./hybrid-search.md) --> BV
    BH --> BL
    BV --> RRF[rrf](./rrf.md)
    BL --> RRF
    
    SDK[python-sdk](./python-sdk.md) --> PyO3[pyo3](./pyo3.md)
    PyO3 --> GIL[gil](./gil.md)
    PyO3 --> FFI[ffi](./ffi.md)
    
    mmap[mmap](./mmap.md) --> HNSW
    FileLocking[file-locking](./file-locking.md) --> T
    RwLock[rwlock](./rwlock.md) --> T
    
    R[recall](./recall.md) --> BV
    L[latency](./latency.md) --> BV
    ME[memory-efficiency](./memory-efficiency.md) --> HNSW
    
    style E fill:#f9f,stroke:#333,stroke-width:3px
    style RAG fill:#bbf,stroke:#333,stroke-width:3px
    style WAL fill:#fbb,stroke:#900,stroke-width:2px
    style BH fill:#bfb,stroke:#333,stroke-width:2px
```

---

## Conventions

- All glossary concepts are linked using Obsidian wikilinks format (`[[Concept]]`).
- Term files are lowercase kebab-case files inside the `glosario/` directory.
- Definitions include: definition, key characteristics, why it matters in VantaDB, trade-offs, and related terms.
