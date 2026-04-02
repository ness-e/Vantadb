# WAL Strategy — Write-Ahead Log

> **Fase**: 01_Architecture | **Decision**: Bincode format, no HNSW persistence

## 1. Overview

The WAL ensures **durability** for all mutations. On crash, the WAL is replayed to reconstruct the in-memory MemTable. The HNSW index is NOT persisted — it's rebuilt from node vector data during recovery.

## 2. Record Format

```
Per-record layout (append-only file):
┌────────────┬───────────────────┬──────────┐
│ len: u32   │ payload: [u8;len] │ crc: u32 │
│ (4 bytes)  │ (bincode encoded) │ (4 bytes)│
└────────────┴───────────────────┴──────────┘

WalRecord enum:
  Insert(UnifiedNode)          — full node snapshot
  Update { id: u64, node }     — replace node
  Delete { id: u64 }           — tombstone
  Checkpoint { node_count }    — recovery hint
```

## 3. Write Path

```
insert(node) called:
  1. bincode::serialize(WalRecord::Insert(node))   → payload
  2. compute CRC32(payload)
  3. WAL BufWriter: write len + payload + crc       → 64KB buffer
  4. Batch fsync (every N records or on explicit sync)
  5. MemTable HashMap.insert(id, node)
  6. Return Ok(id) to client
```

**Invariant**: Step 3 (WAL write) MUST complete before step 5 (MemTable insert). On crash between 3 and 5, WAL replay catches up.

## 4. Recovery Path

```
Engine::with_wal(path) called:
  1. Open WAL file
  2. Read records sequentially:
     - Insert → HashMap.insert(id, node)
     - Update → HashMap.insert(id, new_node)  
     - Delete → HashMap.remove(id)
     - Checkpoint → skip (informational)
  3. Track max_id → set next_id = max_id + 1
  4. Open WAL for new appends
  5. (Fase 3) Rebuild HNSW index from all vector nodes
```

**Recovery time estimate**:
| Nodes | WAL Size | Replay Time | HNSW Rebuild |
|-------|----------|-------------|--------------|
| 10K | ~65 MB | ~200ms | ~500ms |
| 100K | ~650 MB | ~2s | ~3-5s |
| 1M | ~6.5 GB | ~15s | ~30-60s |

## 5. Fsync Strategy

| Mode | Behavior | Durability | Performance |
|------|----------|------------|-------------|
| **Batch** (default) | fsync every 1000 records or 100ms | Lose last batch on crash | ~50K writes/sec |
| **Immediate** | fsync every record | Full durability | ~5K writes/sec |
| **None** | OS-buffered, no explicit fsync | May lose data on crash | ~200K writes/sec |

Fase 1 default: **Batch mode** (pragmatic balance).

## 6. WAL Rotation (Fase 2)

```
When WAL exceeds 256MB:
  1. Write Checkpoint record
  2. Close current WAL file
  3. Open new WAL file (wal.000002.bin)
  4. Background: compact old WAL after RocksDB flush confirms
```

## 7. CRC32 Implementation

Using a simple, non-cryptographic CRC32 (polynomial 0xEDB88320) implemented inline — no external dependency. Purpose: detect bit-rot and truncated writes, NOT adversarial corruption.

## 8. Why NOT Arrow IPC for WAL

| Factor | Bincode | Arrow IPC |
|--------|---------|-----------|
| Serialize speed | ~2 GB/s | ~800 MB/s |
| Complexity | 1 line | Schema setup required |
| Size | Compact | +10-15% overhead |
| Zero-copy replay | No (deserialize) | Yes |
| Schema evolution | No | Yes |

**Decision**: Bincode for Fase 1 (simplicity wins). Migrate to Arrow IPC in Fase 2 when RocksDB integration enables zero-copy read path end-to-end.

## 9. No HNSW Persistence — Rationale

Persisting the HNSW graph structure requires:
1. Serializing neighbor lists (M*2 links per node)
2. Maintaining consistency between WAL and index file
3. Handling partial writes to index file

Cold-start rebuild cost: **~3-5s for 100k vectors** (acceptable for local dev workload). The simplicity gain (no index corruption bugs, no dual-write path) outweighs the startup penalty.

**Revisit in Fase 3** when CP-Index is implemented — may persist the combined HNSW+bitset structure.
