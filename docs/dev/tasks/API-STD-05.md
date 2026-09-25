# Task API-STD-05 — INDIVIDUAL (4/11) Node nativo

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual Node NAPI: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Vista NAPI-RS (`.node`, server-side, más rápido que WASM): clase `VantaDb` (`index.d.ts:243`) mínima + extras propios (`versions`, `vacuum`, `searchWithMethod`). Async (Promise).

## 3. Uso (ejemplo mínimo)

```js
const { VantaDb } = require("vantadb-node");
const db = await VantaDb.open("/tmp/vanta.db");
await db.put({ namespace: "docs", key: "a", payload: "hello" });
```

## 4. Código (re-verificado 2026-09-24, lectura directa `.d.ts`)

- **N1 CONFIRMADO — superficie mínima:** `VantaDb` (`:243-447`) sin export/import/bulk/audit/search_vector/hardware/recover (cero matches en el grep de 27). Solo `rebuildIndex/compact/flush/purge` + `versions(:389)/vacuum(:401)/searchWithMethod(:440)`.
- **N2 CONFIRMADO — lectura sin escritura sparse:** `MemoryRecord.sparse_vector` sí (`:67`) pero `MemoryInput` NO (`:44-52`).
- **N3 CONFIRMADO — sin `exclude_superseded`:** ni `SearchRequest` (`:87-95`) ni `MemoryListOptions` (`:74-78`) lo tienen.
- **N4 `method` separado CONFIRMADO:** `searchWithMethod(request, method?)` (`:440`) vs `method` inline Python / ausente TS.
- **N5 BUENA PRÁCTICA YA EXISTENTE:** `node_id: string` decimal (`:63-64` doc MAX_SAFE_INTEGER) — el estándar `u128→string` que hay que copiar a los demás. `distance_metric: 'Cosine'|'Euclidean'` capitalizado (`:93`) = coincide OpenAPI, NO MCP (lowercase) — veredicto para 15.
- **N6 `cursor?: number` numérico** (`:77,83`) vs OpenAPI `string|null` — mismo drift que HTTP (para 16).

## 5. Veredicto + implicaciones

6/6 confirmados (N5/N6 =andidatos a estándar, no fallos puros). Propuestas 15: decidir Node mínimo-declarado vs convergente; añadir escritura sparse + `exclude_superseded`; unificar `method`; tomar `node_id: string` como canónico cross-binding; alinear case `distance_metric` (una sola forma en TODAS).

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: Node ficha completa

## Context Save Point

API-STD-05 DONE. Next: API-STD-06 (WASM). Deuda: ninguna. WIP: ninguno.
