# Task API-STD-04 — INDIVIDUAL (3/11) TypeScript SDK

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual TS: funcionamiento + uso + código + veredicto por fallo.

## 2. Funcionamiento

Vista WASM (`serde_wasm_bindgen`, `inner`) + backend nativo alternativo (`native.ts` subset sync). 43 métodos + sub-clientes `memory/graph/system` (delegación congelada, `vantadb.ts:264-344`). Dominios memory (16) / graph (11) / system (16).

## 3. Uso (ejemplo mínimo)

```ts
import { VantaDB } from "vantadb-ts";
const db = await VantaDB.open("/tmp/vanta.db");
db.put({ namespace: "docs", key: "a", payload: "hello" });
const hits = db.search({ namespace: "docs", vector: [...] });
```

## 4. Código (re-verificado 2026-09-24, grep+lectura directa)

- **T1 CONFIRMADO — `score→distance` sin invertir (3 sitios):** `distance: h.score as number` en `:609` (search), `:663` (searchMulti), `:742` (similarToKey) vs doc lower-is-better `types.ts:143-147`. Mismo número, semántica contraria.
- **T2 CONFIRMADO — traversals truncan IDs:** `roots: number[]` en `:90-107` (interfaz) y `:1328` (impl) vs `removeEdge(source: number|bigint…)` con guards >2^53 (`:1294-1308`). El equipo sabe del límite pero BFS/DFS/topo/isDag/filtered/degree no lo aplican — IDs >2^53 mueren en traversals.
- **T3 CONFIRMADO — `importRecords` bucle JS:** `get+put` por registro (`:907-923`), `skipped: 0` fijo (`:924`), motivo documentado FIND-79 (`:880-886`: wasm `import_records` rechaza `MemoryInput` y shape `get()`). Pierde `created_at/version/history`; conteo `inserted/updated` local con race.
- **T4 sin `bulk_import` CONFIRMADO:** cero matches `bulk_import` en `vantadb.ts` (40 matches del grep sin esa línea).
- **T5 `searchMulti/similarToKey/supersede/removeEdge/count` presentes** (`:60-62,88,638,733` + sub-clientes `:264-309`) — doc `BINDINGS_NAMESPACES:134` al día en esto.

## 5. Veredicto + implicaciones

4/5 fallos confirmados (T5 = doc correcta, no fallo). Propuestas 15: (a) `distance`→`score` o invertir valor + test (breaking `feat!:`); (b) `roots: number|bigint[]` + guards como `removeEdge`; (c) arreglar wasm `import_records` (aceptar `MemoryInput` + u64-string) y delegar, o declarar bucle JS como diseño con sus límites; (d) exponer `bulk_import`. Blast radius: `vantadb.ts` + `native.ts` + `types.ts` + tests `__tests__/`.

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: TS ficha completa, 4/5 + 1 doc-ok

## Context Save Point

API-STD-04 DONE. Next: API-STD-05 (Node). Deuda: ninguna nueva. WIP: ninguno.
