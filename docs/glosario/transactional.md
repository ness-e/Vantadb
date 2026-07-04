---
title: "Transactional"
type: glossary-entry
status: stable
tags: [concept, acid, durabilidad, consistencia]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Transaccional, ACID, Transactional]
description: "Propiedad de un sistema que garantiza que las operaciones sobre datos cumplan las propiedades ACID (Atomicidad, Consistencia, Aislamiento, Durabilidad)"
---
# Transactional

##Definition

A **transactional** system guarantees that operations on data comply with the **ACID** properties (Atomicity, Consistency, Isolation, Durability), ensuring that mutations are reliable even in the face of system failures, crashes or concurrency.

## ACID Properties

| Propiedad | Definición | Implementación en VantaDB |
|-----------|-----------|--------------------------|
| **Atomicidad** | Todo o nada: una transacción se completa entera o no se aplica | [[wal]] + rollback automático |
| **Consistencia** | El sistema pasa de un estado válido a otro estado válido | Validación de constraints + índices derivados |
| **Aislamiento** | Transacciones concurrentes no interfieren entre sí | [[mvcc]] en [[fjall]] + [[rwlock]] |
| **Durabilidad** | Una transacción confirmada sobrevive a crashes | [[wal]] con [[fsync]] + [[crc32c]] |

##Why it Matters in VantaDB

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

> **No mutation is confirmed to the client until the [[wal]] is synchronized to physical disk using [[fsync]].**

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
*Components linked in transaction:* [[vectors]], [[graph]], [[hnsw]], [[bm25]]


If you update a document:
- ✅ The document is updated
- ✅ Your embedding regenerates
- ✅ Your graph relationships stay consistent
- ✅ Indexes are reindexed
- ✅ Everything in a single atomic transaction

**O todo sucede, o nada sucede.** No hay estados intermedios visibles.

## Configurable Durability Levels

VantaDB supports (or should support) multiple synchronization modes:

| Modo | Descripción | Latencia | Riesgo |
|------|-------------|----------|--------|
| **SyncAlways** | fsync en cada write | Alta (~5-10ms) | Cero pérdida de datos |
| **SyncPeriodic** | fsync cada N ms | Baja (<1ms) | Pérdida de últimos N ms |
| **SyncNever** | Sin fsync (OS decide) | Mínima | Pérdida potencial alta |

> ⚠️ **Audit Finding (AUD-01):** The snapshot does not demonstrate that VantaDB implements configurable synchronous fsync. This is **blocking** for durability claims.

## Comparison with Alternatives

| Sistema | Transaccional | Atomicidad Multi-Modelo | Durabilidad Real |
|---------|--------------|------------------------|------------------|
| **VantaDB** | ✅ ACID completo | ✅ Doc + Vector + Grafo | ✅ WAL + fsync |
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
1. Crash-injection tests ([[chaos-testing]])
2. Checksum verification [[crc32c]] in replay
3. Post-recovery consistency validation

## See Also

- [[wal]] — Mechanism that enables durability
- [[fsync]] — Physical persistence guarantee
- [[crc32c]] — Record Integrity
- [[mvcc]] — Isolation of concurrent transactions
- [[fjall]] — Backend with native transactional support

---

*Being transactional is not an optional feature, it is the fundamental contract of a database that manages persistent memory.*

