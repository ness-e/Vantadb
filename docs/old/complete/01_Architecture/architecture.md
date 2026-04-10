# ConnectomeDB — System Architecture

> **Fase**: 01_Architecture | **Status**: In Progress | **Target**: Semana 1-2

## 1. Overview

ConnectomeDB is a **single-node, multimodel database engine** that natively unifies three data models in a single storage layer:

| Model | Purpose | Index | Query Operator |
|-------|---------|-------|----------------|
| **Vectorial** | Embeddings (FP32/INT8) | HNSW → CP-Index | `~` (similitud) |
| **Grafo** | Directed labeled edges | Adjacency lists | `SIGUE` (trayectoria) |
| **Relacional** | Typed key-value fields | Bitset + scan | `#` (filtro) |

### Design Principles
1. **Storage-over-Compute**: Persistence and latency before intelligence
2. **Mechanical Sympathy**: Structs aligned to L1 cache (64B lines)
3. **Zero-Copy**: Minimize allocations between storage and query layers
4. **Local-First**: Optimized for 16GB laptop, NOT cloud clusters

## 2. System Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      CLIENT LAYER                           │
│   PG Wire │ REST API │ gRPC Streaming │ Rust Native SDK     │
├─────────────────────────────────────────────────────────────┤
│                      QUERY LAYER                            │
│   Lexer/Parser (Nom) → AST → LogicalPlan → PhysicalPlan    │
│   CBO: Heuristic (F2) → Statistical SCE (F3)               │
│   TEMPERATURE 0.0-1.0 (exhaustive ↔ aggressive pruning)    │
├─────────────────────────────────────────────────────────────┤
│                      EXECUTION LAYER                        │
│   Operators: Scan │ BitsetFilter │ SIGUE │ ~ │ Rank        │
│   Resource Governor │ Circuit Breaker │ OOM Guard           │
├─────────────────────────────────────────────────────────────┤
│                      STORAGE LAYER                          │
│   ┌────────────────┐  ┌───────────┐  ┌──────────────────┐  │
│   │ InMemoryEngine │  │ CP-Index  │  │  WAL (bincode)   │  │
│   │ (HashMap<u64,  │  │ (HNSW +   │  │  append-only     │  │
│   │  UnifiedNode>) │  │  bitset)  │  │  CRC32 per rec   │  │
│   └───────┬────────┘  └─────┬─────┘  └────────┬─────────┘  │
│           │                 │                  │            │
│   ┌───────┴─────────────────┴──────────────────┴─────────┐  │
│   │                 RocksDB (Fase 2)                      │  │
│   │            LSM-tree │ Bloom │ SST compaction          │  │
│   └───────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      HARDWARE LAYER                         │
│   L1 (64B) │ L2 (256KB) │ RAM 16GB │ NVMe SSD │ Disk      │
└─────────────────────────────────────────────────────────────┘
```

## 3. Write Path

```
Client INSERT
    │
    ▼
┌──────────────┐
│  Validate    │──▶ Schema check, dimension match
│  & Normalize │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  WAL Append  │──▶ bincode serialize → len(4B) + payload + CRC(4B)
│  (fsync per  │    BufWriter 64KB → batch fsync
│   batch)     │
└──────┬───────┘
       │
       ├──────────────────┬──────────────┐
       ▼                  ▼              ▼
┌────────────┐   ┌──────────────┐  ┌──────────┐
│  MemTable  │   │ HNSW Insert  │  │  Bitset  │
│ HashMap    │   │ (deferred,   │  │  Update  │
│ .insert()  │   │  Fase 3)     │  │  u128    │
└────────────┘   └──────────────┘  └──────────┘
       │
       ▼ (background, Fase 2)
┌────────────┐
│  RocksDB   │──▶ SST flush + compaction
│  Flush     │
└────────────┘
```

**Invariant**: WAL append completes BEFORE MemTable insertion. On crash, WAL replay reconstructs full state.

## 4. Read Path (Hybrid Query)

```
FROM Usuario#usr45 SIGUE 1..3 "amigo" Persona p
WHERE p.pais="VZLA" AND p.bio ~ "rust expert", min=0.88

Execution order (CBO decides):
  1. Bitset pre-filter  ──▶ O(1) per node (AND on u128)
  2. Graph traversal    ──▶ BFS with label filter
  3. Vector similarity  ──▶ Brute-force F1 / HNSW F3
  4. Relational filter  ──▶ Field predicate evaluation
  5. Rank + truncate    ──▶ Top-K by score DESC
```

## 5. Concurrency Model (Fase 1)

```
┌─────────────────────────────────────┐
│         parking_lot::RwLock         │
│                                     │
│  Read:  Multiple concurrent scans   │
│  Write: Exclusive (WAL + memtable)  │
│                                     │
│  WAL:   Mutex<Option<WalWriter>>    │
│         (serialized writes)         │
└─────────────────────────────────────┘
```

**Fase 3** migrates to Tokio async with sharded locks for higher QPS.

## 6. Memory Budget (16GB Target)

| Component | Budget | Notes |
|-----------|--------|-------|
| OS + Runtime | 2 GB | Rust binary + OS |
| UnifiedNode headers | 2 GB | ~35M nodes @ 56B header |
| Vector data (FP32) | 8 GB | ~1.3M vectors @ 1536d × 4B |
| Edges + Relational | 3 GB | Variable |
| WAL buffer | 64 KB | BufWriter |
| Index overhead | 1 GB | HNSW graph (Fase 3) |
| **Total** | **~16 GB** | |

**Key constraint**: 1M nodes + 100k vectors (1536d) = ~600MB vectors + ~56MB headers + edges ≈ **~1GB total**. Well within 16GB.

## 7. Hardware Matrix

| Platform | RAM | Storage | CPU | GPU | Status |
|----------|-----|---------|-----|-----|--------|
| Laptop dev | 16GB DDR4/5 | NVMe SSD | i7/Ryzen7 | — | **Primary target** |
| Edge | 8GB | microSD/eMMC | ARM Cortex-A76 | — | Fase 3 |
| Server | 32-128GB DDR5 | NVMe RAID | Xeon/EPYC | RTX 3060+ | Fase 3 |

## 8. Error Philosophy

Every query response includes metadata:
```rust
QueryResult {
    nodes: Vec<UnifiedNode>,
    is_partial: bool,        // true if resource limits hit
    exhaustivity: f32,       // 0.0-1.0 search completeness
    source_type: SourceType, // which index/scan was used
}
```

## 9. Key Invariants

1. **WAL-before-MemTable**: No write visible without WAL record
2. **Bitset-first filtering**: Cheapest predicate always evaluated first
3. **No HNSW persistence**: Rebuilt from node data on cold start (~3s/100k vectors)
4. **u128 bitset**: 128 boolean filter dimensions, single-instruction AND
5. **Clone on read**: Nodes cloned from RwLock read guard (Fase 2: Arc<Node> or zero-copy)
