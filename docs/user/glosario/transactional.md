---
title: Transactional
kind: glossary
status: stable
description: "A transactional system guarantees that operations on data comply with the ACID properties (Atomicity, Consistency, Isolation, Durability), ensuring that mutations are reliable even in the face of system failures, crashes or concurrency"
aliases: [Transaccional, ACID, Transactional]
tags: [concept, acid, durabilidad, consistencia]
links: "[[README.md]]"
---

# Transactional

## Definition

A **transactional** system guarantees that operations on data comply with the **ACID** properties (Atomicity, Consistency, Isolation, Durability), ensuring that mutations are reliable even in the face of system failures, crashes or concurrency.

## ACID Properties

| Propiedad | Definición | Implementación en VantaDB |
|-----------|-----------|--------------------------|
| **Atomicidad** | Todo o nada: una transacción se completa entera o no se aplica | [wal](./wal.md) — replay trunca en el primer registro inválido; batch-append con `batch_insert_with_opts` bajo un único guard |
| **Consistencia** | El sistema pasa de un estado válido a otro estado válido | Validación de constraints + índices derivados |
| **Aislamiento** | Escrituras serializadas; lecturas concurrentes coherentes por orden WAL→storage | `insert_lock` global (`RwLock`) + orden del WAL — **no** MVCC |
| **Durabilidad** | Una transacción confirmada sobrevive a crashes | [wal](./wal.md) con [fsync](./fsync.md) + [crc32c](./crc32c.md) |

## Why it Matters in VantaDB

VantaDB manages **persistent memory for AI agents**. If an agent stores important context (conversations, decisions, acquired knowledge), it **cannot be lost** due to a crash or power outage.

### The VantaDB Transactional Contract

```
Agente de IA                VantaDB                    Disco
    │                          │                          │
    │──── put(document) ──────▶│                          │
    │                          │── Write to WAL ─────────▶│
    │                          │── fsync() ──────────────▶│ [DURABLE]
    │                          │◀─ ACK ──────────────────│
    │◀──── Success ────────────│                          │
    │                          │                          │
    │                          │  [CRASH / Power Loss]    │
    │                          │                          │
    │                          │── Replay WAL ───────────▶│ [RECOVERED]
    │                          │                          │
```

### Fundamental Rule

> **No mutation is confirmed to the client until the [wal](./wal.md) is synchronized to physical disk using [fsync](./fsync.md).**

Esto diferencia a VantaDB de sistemas que:
- Escriben en memoria y hacen flush periódico (riesgo de pérdida)
- Usan WAL pero sin fsync síncrono (riesgo en cortes de energía)
- No tienen WAL (sin garantías de durabilidad)

## Multi-model Transactionality

VantaDB is **transactional across multiple representations**:

```
Transacción Atómica
├── Documento canónico (fuente de verdad)
├── Embedding vectorial (vectors)
├── Relaciones de graph (aristas)
├── Metadatos tipados (payload)
└── Índices derivados (hnsw, bm25)
```
*Components linked in transaction:* [vectors](./vectors.md), [graph](./graph.md), [hnsw](./hnsw.md), [bm25](./bm25.md)


If you update a document:
- ✅ The document is updated
- ✅ Your embedding regenerates
- ✅ Your graph relationships stay consistent
- ✅ Indexes are reindexed
- ✅ Everything in a single atomic transaction

**O todo sucede, o nada sucede.** No hay estados intermedios visibles.

## Configurable Durability Levels

VantaDB implementa tres modos de sincronización configurables (`SyncMode`,
`src/config.rs:88`), evaluados en `WalWriter::maybe_sync` (`src/wal.rs:377`):

| Modo | Descripción | Latencia | Riesgo |
|------|-------------|----------|--------|
| `SyncMode::Always` | sync en cada write | Alta (~5-10ms) | Cero pérdida de datos |
| `SyncMode::Periodic` *(default)* | sync cada `flush_threshold` registros; threshold default = 1 → **equivale a sync por write** | Baja | ≤ `flush_threshold - 1` registros |
| `SyncMode::Never` | Sin sync automático (OS decide) | Mínima | Pérdida potencial de los últimos writes |

> ✅ **AUD-01 cerrado:** el sync-before-ACK está implementado y verificado
> (`maybe_sync` → `WalWriter::sync()` = `flush()` + `sync_data()`), y los modos
> son configurables vía `with_sync_mode()` / `with_flush_threshold()` o
> `VANTADB_FLUSH_THRESHOLD`. Ver [fsync](./fsync.md).

**Matiz importante sobre aislamiento:** el mecanismo real no es MVCC. VantaDB
serializa las escrituras con un `insert_lock` global (`RwLock`) de proceso; la
coherencia entre réplicas concurrentes de lectura y escritura la garantiza el
orden WAL→storage, no el aislamiento snapshot. Los "transaction buffers" del
commit path existen (`src/ingestion.rs`, `commit_transaction`) pero no ofrecen
aislamiento multi-versión.

## Comparison with Alternatives

| Sistema | Transaccional | Atomicidad Multi-Modelo | Durabilidad Real |
|---------|--------------|------------------------|------------------|
| **VantaDB** | ⚠️ Parcial — A y D sólidas (WAL + CRC32C + sync configurable); I por serialización (`insert_lock`), no MVCC | ✅ Doc + Vector + Grafo | ✅ WAL + CRC32C + sync por write (default) |
| **Pinecone** | Parcial | ❌ Solo vectores | ✅ Cloud-managed |
| **ChromaDB** | ⚠️ Básico | ⚠️ Doc + Vector | ⚠️ Dependiente de backend |
| **Qdrant** | ✅ ACID | ⚠️ Doc + Vector + Payload | ✅ WAL |
| **FAISS** | ❌ No transaccional | ❌ Solo índices | ❌ Sin persistencia propia |

## Anti-Pattern: "Transactional Only in Documentation"

Many systems claim to be transactional but:
- They do not fsync before the ACK
- They lose data in crashes
- Rebuild indexes from inconsistent state

VantaDB must **demonstrate** transactionality by:
1. Crash-injection tests ([chaos-testing](./chaos-testing.md))
2. Checksum verification [crc32c](./crc32c.md) in replay
3. Post-recovery consistency validation

## See Also

- [wal](./wal.md) — Mechanism that enables durability
- [fsync](./fsync.md) — Physical persistence guarantee
- [crc32c](./crc32c.md) — Record Integrity
- [file-locking](./file-locking.md) — Exclusión entre procesos (`.vanta.lock`)

---

*Being transactional is not an optional feature, it is the fundamental contract of a database that manages persistent memory.*

