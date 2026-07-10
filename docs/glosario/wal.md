---
title: "wal"
type: glossary-entry
status: stable
tags: [persistence, wal, durability, recovery]
last_refined: 2026-07
links: "[[README.md]]"
aliases: [Write-Ahead Log, Journal, Transaction Log]
description: "Journaling mechanism where mutations are first written to a sequential log before being applied to the main storage, guaranteeing ACID durability"
---
#WAL—Write-Ahead Log

## Definition

The **Write-Ahead Log (WAL)** is a sequential, append-only record of all data mutations, where **each change is written to the log BEFORE being applied to the main database**. This guarantees durability and allows recovery after crashes.

## Fundamental Principle

> **WAL Golden Rule:** No mutation is committed to the client until its record is physically written to disk (fsync).

```
Orden correcto:
1. Append al WAL
2. fsync() del WAL
3. Aplicar cambio al storage
4. ACK al cliente

INCORRECT order (data loss):
1. Apply change to storage
2. ACK to the client
3. Append to WAL (asynchronous)
```

## WAL Record Structure

```
┌─────────────────────────────────────┐
│ Header (8 bytes)                    │
│ ├── Length: u32                     │
│ ├── Type: u8 (Insert/Delete/Update) │
│ └── Flags: u8                       │
├─────────────────────────────────────┤
│ Payload (variable)                  │
│ ├── Key: [u8]                       │
│ ├── Vector: [f32] (si aplica)       │
│ ├── Text: [u8]                      │
│ └── Metadata: [u8]                  │
├─────────────────────────────────────┤
│ Checksum: u32 (CRC32C)              │
└─────────────────────────────────────┘
```

## Implementation in VantaDB

VantaDB uses a **sharded WAL** (`ShardedWal`) that distributes records in round-robin fashion across N shard files to reduce contention and improve write throughput.

```
Archivos WAL típicos (N=4):
  vanta.shard0.wal
  vanta.shard1.wal
  vanta.shard2.wal
  vanta.shard3.wal
```

Each shard is a sequential append-only file with the same record format (header + payload + CRC32C). Shard count is configured via `wal_shards` (default: `4`, env: `VANTADB_WAL_SHARDS`).

### Writing Flow

```
Cliente: put("doc1", vector, text, metadata)
    │
    ▼
┌──────────────────────────────┐
│ Serializar mutación          │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ Calcular CRC32C              │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ ShardedWal.append(record)    │
│ shard_idx = counter % N      │
│ counter += 1                 │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ Append a vanta.shard{idx}.wal│
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ fsync() ← DURABLE            │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ Aplicar a Fjall/HNSW         │
└──────────┬───────────────────┘
           │
           ▼
      ACK al cliente
```

### Recovery Flow — Sort-Based Multi-Shard Replay

Dado que los registros se distribuyen round-robin entre N shards, la recuperación no puede simplemente iterar un solo archivo. El algoritmo correcto es:

1. Leer **todos** los registros de **todos** los shards
2. Calcular la posición global de cada registro: `global_seq = shard_idx + N * local_pos`
3. Saltar registros con `global_seq ≤ checkpoint_seq`
4. Ordenar registros restantes por `global_seq`
5. Replay en orden secuencial global

```
Arranque tras crash
    │
    ▼
┌──────────────────────────────────┐
│ Para cada shard 0..N-1:          │
│  - Abrir vanta.shard{idx}.wal    │
│  - Leer todos los registros      │
│  - Calcular global_seq por record│
└──────────┬───────────────────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Leer checkpoint_seq del backend  │
│  - full_rounds = seq / N         │
│  - remainder   = seq % N         │
│  - Saltar records ≤ checkpoint   │
│     por shard                    │
└──────────┬───────────────────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Ordenar por global_seq           │
│ (sort-based, O(M log M))         │
└──────────┬───────────────────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Replay en orden:                 │
│  - Insert → VantaFile + HNSW    │
│  - Update → VantaFile + HNSW    │
│  - Delete → tombstone VantaFile │
└──────────┬───────────────────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Reconstruir índices derivados    │
│ (BM25, payload indexes) desde    │
│ estado canónico                  │
└──────────┬───────────────────────┘
           │
           ▼
   Base de datos lista
```

Esto garantiza que el orden de escritura original se preserva exactamente, incluso cuando los shards tienen cantidades desiguales de registros. Probado en `test_wal_replay_mixed_mutations`.

## Checkpointing

The WAL grows indefinitely without management. **Checkpointing** is the process of:

1. **Flush** of all pending data to main storage
2. **Mark** the checkpoint sequence number (how many global records are persisted)
3. **Truncate/rotate** the WAL shard files

With shards, the checkpoint is a single global `checkpoint_seq` (the total number of records written so far). Each shard calculates how many records to skip using:

```
full_rounds = checkpoint_seq / N   (N = número de shards)
remainder   = checkpoint_seq % N

Shard 0: skip hasta local_pos = full_rounds + (0 < remainder ? 1 : 0)
Shard 1: skip hasta local_pos = full_rounds + (1 < remainder ? 1 : 0)
...
Shard k: skip hasta local_pos = full_rounds + (k < remainder ? 1 : 0)
```

Compaction (`compact_wal()`) performs a full flush, saves `checkpoint_seq`, and rotates the shard files.

## Durability Modes

| Mode | fsync | Latency | Loss Risk |
|------|-------|---------|-----------|
| **Always** | Every write | ~5-10ms | Zero |
| **Periodic** | Every N ms | <1ms | Last N ms |
| **Never** | OS decides | ~0.1ms | High |

## WAL Guarantees in VantaDB

### What WAL Guarantees

✅ **Durability:** Once confirmed, the data survives crashes
✅ **Atomicity:** Complete transactions or none
✅ **Deterministic recovery:** Replay produces the same state

### What WAL Does NOT Guarantee

❌ **Consistency between indexes:** That depends on the coherence protocol
❌ **Concurrency isolation:** That depends on locks/MVCC
❌ **Performance:** Synchronous fsync adds latency

## Known Issues

## Comparison with Other Systems

| System | WAL | fsync Default | Checksum | Recovery |
|--------|-----|---------------|----------|----------|
| **VantaDB** | ✅ Sharded (N-way) | ✅ Configurable (Always/Periodic/Never) | ✅ CRC32C | ✅ Sort-based multi-shard replay |
| **SQLite** | ✅ | Always | ✅ CRC32 | ✅ Automatic |
| **PostgreSQL** | ✅ | Always | ✅ CRC32 | ✅ Automatic |
| **RocksDB** | ✅ | Configurable | ✅ CRC32 | ✅ Automatic |
| **FAISS** | ❌ | N/A | N/A | ❌ No persistence |

## See Also

- [[fsync]] — Physical persistence guarantee
- [[crc32c]] — Record integrity
- [[fjall]] — Backend with its own WAL
- [[transactional]] — Property enabled by WAL
- [[chaos-testing]] — Validating durability

### Related Implementation Documentation
- [[../operations/DURABILITY_GUARANTEES|Durability Guarantees]]

---

*The WAL is the durability contract of a database. Without it, there are no guarantees.*

