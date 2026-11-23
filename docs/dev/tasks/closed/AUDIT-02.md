---
title: "AUDIT-02: Corrección InFilter bitset, caché BM25 y desacople ShreddedRowStore"
kind: task
status: completed
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-engine"
category: "storage-correctness"
tags: [storage, infilter, bm25, shred, bitset]
description: "Corrección del bug node.bitset en InFilter, caché de estadísticas BM25 y desacople de ShreddedRowStore en memory.put."
---

# AUDIT-02: Corrección InFilter bitset, caché BM25 y desacople ShreddedRowStore

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-31-audit-remediation-master.md`
- **Fuente:** Hallazgos FIND-503, FIND-604, FIND-607
- **Esfuerzo:** 🟡 2d
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust (core storage & search)
- **Turns estimados:** 15-20
- **Creado:** 2026-10-31T06:00:00Z
- **last-synced:** 2026-10-31T06:00:00Z
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3

## Objetivo

Resolver tres fallos críticos de correctness y rendimiento en el núcleo de almacenamiento y consulta:
1. Eliminar el fallo en `InFilter` (`src/storage/query_planner.rs` / `src/sdk/search/lexical.rs`) donde la evaluación sobre `node.bitset` resultaba en un conjunto vacío cuando intervenían campos no escalares o ausentes en el store fragmentado, provocando que búsquedas vectoriales filtradas retornaran 0 resultados erróneamente.
2. Introducir una caché atómica con invalidación basada en versiones para las estadísticas documentales BM25 (longitud promedio de documento `avg_dl`, frecuencias totales), eliminando el recálculo O(N) por consulta.
3. Desacoplar la escritura síncrona en `ShreddedRowStore` durante `memory.put` en `src/sdk/api/memory.rs`, trasladándola a un procesamiento asíncrono/background pos-WAL para no penalizar la latencia del camino de inserción.

## Archivos en scope

- `src/storage/query_planner.rs`: planificación y filtrado `InFilter`.
- `src/sdk/search/lexical.rs`: evaluación de filtros y fallback cuando campos no son escalares.
- `src/index/text_index.rs` / `src/index/bm25.rs`: cálculo y cacheo de estadísticas documentales.
- `src/shred/mod.rs`: serialización de filas fragmentadas y manejo de tipos compuestos.
- `src/sdk/api/memory.rs`: pipeline de inserción de memorias y acople con shredding.

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | Búsqueda vectorial filtrada (`search` con filtros), `memory.put`, consultas BM25 |
| Callees | `ShreddedRowStore`, `BM25Index`, `StorageEngine` |
| Implicaciones | Desaparece la discrepancia de resultados vacíos en filtros `IN`; latencia de `memory.put` desciende significativamente; throughput de búsqueda BM25 aumenta. |

## Criterios de aceptación

1. Un filtro `IN` o de igualdad que involucra tipos no escalares o campos opcionales recurre correctamente al fallback de metadatos completos sin descartar nodos válidos.
2. Las estadísticas BM25 se sirven desde caché en memoria y solo se recalculan tras confirmarse mutaciones de inserción o borrado en la colección.
3. `memory.put` completa la escritura durable en WAL y estado primario sin bloquearse esperando la generación de índices fragmentados secundarios.
4. Cobertura de pruebas unitarias y de integración validando paridad de resultados entre fast-path y fallback.

## Contrato

```bash
cargo test -p vantadb test_infilter_non_scalar_fallback -- --nocapture
cargo test -p vantadb test_bm25_stats_cache_invalidation -- --nocapture
cargo test -p vantadb test_memory_put_shred_decoupling -- --nocapture
cargo clippy -p vantadb --all-targets -- -D warnings
```

## Steps

### Step 1: Corregir lógica de bitset y fallback en `InFilter`
- Modificar `src/sdk/search/lexical.rs` y `src/storage/query_planner.rs` para que la falta de un campo en `ShreddedRowStore` no marque exclusión tajante sino desvío al validador de nodo completo.
- Asegurar que `node.bitset` mantenga cardinalidad correcta.

### Step 2: Implementar caché de estadísticas BM25
- Crear estructura `Bm25CollectionStats` encapsulada en `ArcSwap` o `RwLock` con un `version_epoch`.
- Actualizar `version_epoch` en cada commit WAL y regenerar estadísticas de forma perezosa o amortizada.

### Step 3: Desacoplar `ShreddedRowStore` en `memory.put`
- Separar la llamada síncrona `shred_row` del flujo crítico de `memory.put`.
- Encolar actualización para ejecución concurrente tras la confirmación del WAL.

## Invariantes de dominio

- La consistencia de lectura debe mantenerse: si un nodo es retornado, debe cumplir estrictamente con los filtros de la consulta.
- La durabilidad ante fallos garantizada por el WAL no puede degradarse.

