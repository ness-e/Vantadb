---
title: Avance — Archivo de Exclusiones Verificadas (SKIP)
kind: review
status: active
description: Los siguientes identificadores constan igualmente cerrados o absorbidos en HEAD y ya habían sido retirados de Backlog.md antes del saneamiento del 2026-11-22
tags: [review, audit, governance]
---

# Avance — Archivo de Exclusiones Verificadas (`SKIP` / Obsoletas / Absorbidas)

> Registro con evidencia individual de los ítems marcados como `SKIP`, obsoletos o absorbidos que fueron verificados contra `HEAD` y retirados de [`docs/dev/Backlog.md`](../../Backlog.md). Mantiene trazabilidad completa sin contaminar las tablas activas.

## 1. Exclusiones retiradas en el saneamiento 2026-11-22

| ID | Destino / Resolución | Evidencia verificable en `HEAD` |
|---|---|---|
| `FIND-242` | ✅ Ya cerrada | Task file archivado en `docs/dev/tasks/closed/FIND-242.md` y documentada en `activo/bindings.md:1168`. |
| `FIND-248` | ✅ Absorbida | Cubierta por `CRASH-01` (replay exhaustivo de WAL rotados) en `activo/core-engine.md:1532`. |
| `FIND-289` | ✅ Absorbida | Cubierta y acotada en `TEC-07` (`activo/vanta-memory.md:184,318`). |
| `MGR-16` | ✅ Absorbida | Cubierta por `TEC-16` (RRF en planner + MMR formal en `src/sdk/search/mmr.rs`) en `activo/core-engine.md:1580`. |
| `FIND-388` | ✅ Ya resuelta | `Formula/vantadb.rb:12` actualizado a `version "0.9.0"` (`FIND-342`). |
| `FIND-389` | ✅ Obsoleta | `CITATION.cff` no existe en el repositorio (`Path("CITATION.cff").exists() == False`). |
| `FIND-401` | ✅ Ya resuelta | `node scripts/docs/check-docs.mjs` reporta `orphans=0/327` y `wikilinks=0/33`. |
| `FIND-406` | ✅ Ya resuelta | `.opencode/AGENTS.md` documenta con precisión el modelo real de OIDC/Secrets por registry. |
| `FIND-489` | ✅ Ya implementada | Job `test-embedding-features` activo en `.github/workflows/ci-rust.yml:433` para `remote-inference` y `embed-local`. |
| `FIND-221` | ✅ Absorbida | Atomicidad batch-wide por transacción WAL entregada por `FIND-640` (`f282257a9`, `ADR-0061`). |
| `FIND-157` | ✅ Absorbida | Coerción `Int`/`Float` sin pérdida de precisión entregada por `FIND-556` (`7bdcebaa1`). |
| `FIND-523..526`, `FIND-528` | ✅ Absorbidas | Endurecimiento de parser/lexer IQL entregado por `FIND-522` (`35ced06fe`, `6ca7358da`). |
| `FIND-530` | ✅ Ya resuelta | `docs/api/HTTP_API.md` corregido respecto a variantes `UPDATE NODE` / `RELATE`. |
| `FIND-532` | ✅ Ya resuelta | Forma de `record.metadata` alineada entre Node y WASM (`vantadb-ts/tests/integration.test.ts:227`). |
| `FIND-533` | ✅ Ya resuelta | `NativeVantaDB.searchMulti` soporta namespace por defecto/objeto en `vantadb-ts/src/native.ts:1031`. |
| `FIND-538` | ✅ Absorbida | Ejemplos TypeScript reparados en `DEM-04` (`632f49187`, `activo/operaciones.md:699`). |
| `FIND-546` | ✅ Ya resuelta | Alias `VantaDB` preservado en regeneración napi (`21fb5b567`). |
| `FIND-547` | ✅ Ya resuelta | Cero referencias residuales a `0.8.0` en `vantadb-node/index.cjs`. |
| `FIND-557`, `FIND-558` | ✅ Absorbidas | Evaluación vacía de subqueries escalares y proyección por orden de columna entregadas por `FIND-556` (`7bdcebaa1`, `539fc7887`). |
| `FIND-572` | ✅ Absorbida | Underflow en `free_allocation` resuelto junto con `FIND-555` (`activo/core-engine.md:1098`). |
| `FIND-605` | ✅ Absorbida | Lock atómico en `skills.rs` entregado por `NOT-03` (`activo/vanta-memory.md:394`). |
| `FIND-608` | ✅ Absorbida | Sub-ítems `ME-01/ME-02/SE-01/SH-05/GR-05` cerrados por `94855d647`, `975d3b648` y `MEM0-02` (`activo/vanta-memory.md:364`). |
| `FIND-609` | ✅ Absorbida | Persistencia de `LabelIntern` en disco entregada por `NOT-01` (`activo/vanta-memory.md:382`). |
| `NOT-05` | ✅ Implementada | `TrustGuard` activo en `vanta-memory/src/core/dream/mod.rs:167-175,841,856` (>=2 episodios o proveniencia verificada pre-L3). |
| `MEM0-04` | ✅ Implementada | Extractor ontológico bitemporal `BitemporalTriplet` implementado en `vanta-memory/src/core/ontology/mod.rs`. |

## 2. Ítems `SKIP` verificados ya retirados en barridos previos

Los siguientes identificadores constan igualmente cerrados o absorbidos en `HEAD` y ya habían sido retirados de `Backlog.md` antes del saneamiento del 2026-11-22:
- `EGO-14`, `EGO-16` (`4eb5d5d99`, `b7550140e`, `8e08939ff` — registrados en `activo/core-engine.md:1591-1601`)
- `FIND-267` (absorbido por `FIND-257`, `13338001b`)
- `FIND-518`, `FIND-519` (cubiertos en cierre de gramática IQL)
- `FIND-552`, `FIND-553`, `FIND-554`, `FIND-555`, `FIND-559`, `FIND-560` (cubiertos por `FIND-556` / `FIND-563` / governor de `activo/core-engine.md`)
