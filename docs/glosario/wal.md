---
title: "wal"
type: glossary-entry
status: stable
tags: [persistence, wal, durabilidad, recovery]
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

## Estructura de un Registro WAL

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

Cada shard es un archivo append-only secuencial con el mismo formato de registro (header + payload + CRC32C). El número de shards se configura via `wal_shards` (default: `4`, env: `VANTADB_WAL_SHARDS`).

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

El WAL crece indefinidamente si no se gestiona. **Checkpointing** es el proceso de:

1. **Flush** of all pending data to main storage
2. **Mark** the checkpoint sequence number (cuántos records globales están persistidos)
3. **Truncate/rotate** the WAL shard files

Con shards, el checkpoint se representa como un solo `checkpoint_seq` global (el número total de records escritos hasta el momento). Cada shard calcula cuántos registros saltar usando:

```
full_rounds = checkpoint_seq / N   (N = número de shards)
remainder   = checkpoint_seq % N

Shard 0: skip hasta local_pos = full_rounds + (0 < remainder ? 1 : 0)
Shard 1: skip hasta local_pos = full_rounds + (1 < remainder ? 1 : 0)
...
Shard k: skip hasta local_pos = full_rounds + (k < remainder ? 1 : 0)
```

La compactación (`compact_wal()`) hace flush completo, guarda `checkpoint_seq`, y rota los archivos shard.

## Durability Modes

| Modo | fsync | Latencia | Riesgo de Pérdida |
|------|-------|----------|-------------------|
| **Always** | Cada write | ~5-10ms | Cero |
| **Periodic** | Cada N ms | <1ms | Últimos N ms |
| **Never** | OS decide | ~0.1ms | Alta |

## Garantías del WAL en VantaDB

### What WAL Guarantees

✅ **Durability:** Once confirmed, the data survives crashes
✅ **Atomicity:** Complete transactions or none
✅ **Deterministic recovery:** Replay produces the same state

### Lo que WAL NO Garantiza

❌ **Consistency between indexes:** That depends on the coherence protocol
❌ **Concurrency isolation:** That depends on locks/MVCC
❌ **Performance:** Synchronous fsync adds latency

## Known Issues

## Comparison with Other Systems

| Sistema | WAL | fsync Default | Checksum | Recovery |
|---------|-----|---------------|----------|----------|
| **VantaDB** | ✅ Sharded (N-way) | ✅ Configurable (Always/Periodic/Never) | ✅ CRC32C | ✅ Sort-based multi-shard replay |
| **SQLite** | ✅ | Siempre | ✅ CRC32 | ✅ Automático |
| **PostgreSQL** | ✅ | Siempre | ✅ CRC32 | ✅ Automático |
| **RocksDB** | ✅ | Configurable | ✅ CRC32 | ✅ Automático |
| **FAISS** | ❌ | N/A | N/A | ❌ Sin persistencia |

## See Also

- [[fsync]] — Garantía de persistencia física
- [[crc32c]] — Integridad de registros
- [[fjall]] — Backend con WAL propio
- [[transactional]] — Propiedad que el WAL habilita
- [[chaos-testing]] — Cómo validar durabilidad

### Related Implementation Documentation
- [[../operations/DURABILITY_GUARANTEES|Durability Guarantees]]

---

*The WAL is the durability contract of a database. Without it, there are no guarantees.*

