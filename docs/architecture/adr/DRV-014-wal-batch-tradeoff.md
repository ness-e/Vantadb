---
title: "DRV-014: WAL batch-append tradeoff"
type: adr
status: accepted
date: 2026-07-31
tags: [vantadb, wal, performance, adr]
---

# ADR: WAL batch-append tradeoff (DRV-014 reverted)

## Status

Accepted (2026-07-31).

## Context

`DRV-014` ("batch WAL writes") fue completada en `3bdfc93e` eliminando el clon de `WalRecords` en el hot path. Posteriormente, `cae92db3` "perf(engine): Phase 1 optimizations — WAL batch" **revirtió deliberadamente** ese fix:

El clon de WalRecords (`Vec<Vec<WalRecord>>` + `record.clone()`) se reintrodujo para agrupar por shard y usar `WalWriter::batch_append()`: **1 lock + 1 write_all + 1 maybe_sync por shard**, logrando 3-5× de speedup en WAL writes.

## Decision

El código actual NO refleja el fix original de DRV-014. El clon es un **tradeoff de performance intencional** (agrupar por shard para batch-append), no deuda pendiente. La tarea DRV-014 se mantiene cerrada ✅; el tradeoff posterior es de mayor prioridad que el fix revertido.

## Consequences

- **Positivo:** WAL writes 3-5× más rápidos; menos syncs de disco.
- **Negativo:** costo de clonar records en memoria por batch.
- **Gate futuro:** si el clon aparece en profiling como cuello de botella, re-evaluar la eliminación del clon manteniendo el batch-append por shard.
