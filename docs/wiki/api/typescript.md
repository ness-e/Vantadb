---
title: "API de TypeScript (npm `vantadb`)"
kind: reference
status: active
description: "Wrapper TypeScript en npm: sub-clientes por dominio, clase Client sobre WASM por defecto y ruta nativa opcional vía vantadb-node."
---

# API de TypeScript (npm `vantadb`)

Audiencia: usuarios JS/TS (Node ≥22.19, Bun, Deno, browser). Referencia canónica: [TS_SDK](../../api/TS_SDK.md) y `vantadb-ts/README.md`.

## Paquete

`[HECHO]` `vantadb` 0.9.0 en npm (canal declarado en SURFACE_MANIFEST), ESM, `engines.node >=22.19`, exports `.`, `./types` y `./native` (`vantadb-ts/package.json`). `[HECHO]` Es un wrapper: la clase exportada actual es `Client` ([`vantadb-ts/src/vantadb.ts:174`](../../../vantadb-ts/src/vantadb.ts#L174), envuelve `WasmClient`) y delega en `vantadb-wasm` (ruta por defecto) o `vantadb-node` vía `native.ts` (subset async: `capabilities`, `close`, `delete`, `flush`, `get`, `graphragSearch`, `list`, `listNamespaces`, `put`, `putBatch`, `search`). El inventario de métodos por dominio vive en BINDINGS_NAMESPACES. Tipos/errores/guards en `types.ts`, `errors.ts`, `guards.ts`, `metadata.ts`.

## Sub-clientes por dominio (SDKB-02)

`[HECHO]` Getters lazy que devuelven objetos delegados congelados — `db.memory`, `db.graph`, `db.wiki` (`recoverArchivedNodes`), `db.system`; `conversation`/`skills` se omiten en v1 por no existir métodos (D43). El contrato de test es `db.memory.x(...) === db.x(...)`. La taxonomía de dominios (memory/graph/wiki/system; conversation y skills reservados) es normativa en [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md).

## Semántica verificada

`[HECHO]` `SearchHit.score` mayor-es-mejor en híbrida (desde W1/API-02; el rename `distance` de CODE-091 ya no existe); `searchVector()` devuelve `distance` menor-es-mejor. `[HECHO]` `u128` ids: strings decimales + `bigint` in/out. `[HECHO]` v2 wire fields (`as_of_ms`, `valid_window`, `include_quarantined`, `min_confidence`) pasan como passthrough del shape serde (matriz SCH-07). `[HECHO]` No expuestos en TS: `graph_page_rank`/`graph_degree_centrality` (solo Python), `bulk_import*` (wasm/Python), `hardware_profile` (Python).

`[HECHO]` Los tests del paquete (`vantadb-ts/src/__tests__/`, vitest) incluyen `parity.test.ts` corriendo el mismo escenario contra el artefacto real `vantadb-wasm/pkg` (emite `target/bindings-parity/wasm.json`).

## Fuentes

`vantadb-ts/src/` · [TS_SDK](../../api/TS_SDK.md) · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md) · [WASM_API](../../api/WASM_API.md) · [paridad](parity.md)
