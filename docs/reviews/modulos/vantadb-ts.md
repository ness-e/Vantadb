# Review profunda — `vantadb-ts` (SDK TypeScript puro)

**Fecha:** 2026-08-22 · **Alcance:** lectura completa de `src/*.ts` (types, vantadb, errors, guards, native), `package.json`, y los 7 archivos de test. Verificación cruzada contra la superficie real del core (`vantadb/src/sdk/**`) y el binding WASM.

---

## 1. Resumen

SDK TypeScript que envuelve el binding WASM (`vantadb-wasm/pkg`) con una capa fina: validación de records de salida (`_mapRecord`), frontera de error uniforme (`VantaError`), sub-clientes agrupados por dominio (SDKB-02) y un backend alternativo nativo (`NativeVantaDB` sobre `vantadb-node`). Es el SDK más completo y mejor documentado de los tres transportes JS, pero arrastra **tipos de resultado de grafo ficticios**, un **bug de captura de errores async en `NativeVantaDB._native`** y duplicación TS↔native que ya empieza a divergir.

## 2. Arquitectura

```
usuario → VantaDB (vantadb.ts)  ──delega──> WasmVantaDB (vantadb-wasm/pkg)
        → NativeVantaDB (native.ts) ──await──> VantaDb (vantadb-node napi)
sub-clientes memory/graph/wiki/system = vistas congeladas sobre métodos flat (delegación pura, D43)
errors.ts: wrapWasmError/wrapNativeError → VantaError{code,message,details}
guards.ts: type-guards is* para validar shapes de salida
```

Puntos de diseño correctos: sub-clientes como getters memoizados + `Object.freeze` con flechas léxicas (sin drift de `this`); `_assertOpen()` en cada método público; `close()` idempotente; la decisión documentada de NO tomar decisiones de búsqueda en la capa glue (ERR-028 / api-contract R-8).

## 3. Fortalezas

- **JSDoc exhaustiva con ejemplos** en casi todos los métodos públicos — el mejor DX de los tres módulos.
- **Sub-clientes (SDKB-02)** bien ejecutados: delegación pura, tipos reutilizados de `types.ts`, tests de equivalencia flat↔agrupado (`subclients.test.ts`).
- **Manejo u128**: `insertNode` acepta `bigint` con guard de safe-integer; edges normalizados a `bigint` en `getNode` (contrato documentado).
- Tests reales (7 archivos, incl. integración contra WASM y hardening), coverage script, ESLint.
- `guards.ts` exportado públicamente — los consumidores pueden validar shapes sin reimplementar.

## 4. Hallazgos

### Crítico / Required

1. **Tipos de grafo ficticios — `GraphBfsResult`/`GraphDfsResult`/`GraphTopologicalSortResult` no corresponden al wire format real.**
   El core devuelve `Vec<u128>` plano en los tres casos (`src/sdk/graph.rs:50`, `graph_dfs`, `graph_topological_sort`). El binding WASM serializa eso directo (`to_js(&result)`, `lib.rs:1353/1382/1396`) → llega a JS **un array de BigInt**, no `{visited, levels, path}` ni `{sorted, has_cycle}`. `vantadb.ts` hace blind-cast `as GraphBfsResult` (`vantadb.ts:1057/1081/1103`). Los tests no lo detectan porque solo afirman `toBeDefined()` (`integration.test.ts:90-91`). Cualquier consumidor que use `result.visited` recibe `undefined`. **Este es probablemente un bug real de producción, no cosmético.**

2. **`NativeVantaDB._native` nunca envuelve rechazos asíncronos** (`native.ts:88-94`):
   ```ts
   try { return Promise.resolve(fn()); } catch (e) { throw wrapNativeError(e, method); }
   ```
   El `catch` solo captura throws síncronos. Todas las llamadas al engine son promesas (`spawn_blocking` del napi): sus rechazos escapan **sin** envolver en `VantaError`, rompiendo la promesa de "uniform error boundary" del backend nativo. Fix: `async _native(...) { try { return await fn(); } catch (e) { throw wrapNativeError(e, method); } }`.

3. **Semántica invertida `distance` vs `score`:** `search()` mapea `h.score → hit.distance` (`vantadb.ts:579`) y `SearchHit.distance` documenta "lower = more similar". Pero el test del nodo afirma que `score` es **similitud** (mayor = más cercano, `vantadb-node/tests/persistence.test.ts:96-98`). Uno de los dos documentos miente; hoy un usuario de TS ordena mal los hits si confía en el JSDoc. Requiere verificación contra core y unificación (ver §consistencia).

### Nit / Optional

4. `guards.validateVector(v): asserts v is Float32Array` (`guards.ts:82`) — valida `number[]` pero *asserts* `Float32Array`. Type-lie sin uso aparente en el código interno; corregir o eliminar.
5. `_mapRecord` y `_buildSearchRequest` están duplicados casi verbatim entre `vantadb.ts` y `native.ts` — extraer a módulo compartido antes de que diverjan más.
6. Ejemplo JSDoc de `put()` usa metadata `{ source: { type: "String", value: "manual" } }` (`vantadb.ts:408`) — el shape real es tagged-enum `{ String: "manual" }`. El ejemplo tal cual fallaría la deserialización serde.
7. Ejemplo de paginación en `list()` reasigna un `const page` (`vantadb.ts:511-516`) — no compila tal cual.
8. Solo exports ESM (`package.json`: `"type": "module"`, condición `import` única). Consumidores CJS (`require("vantadb")`) fallan. Si es deliberado, documentarlo en README.
9. `dependency: "vantadb-wasm": "file:../vantadb-wasm/pkg"` — dependencia de un artefacto de build commiteado; riesgo de drift pkg↔src si alguien edita `pkg/` a mano o lo regenera con otra versión.
10. Carpeta `coverage/` y un dataset de test residual (`test_perf_15_16_db_39c9d387/`) commiteados en el árbol del paquete — ruido; `.gitignore`.

## 5. API coverage vs core (verificado leyendo `vantadb/src/sdk/**`)

**Expuesto (35 ops):** put, putBatch, get, delete, deleteByFilter, list, listNamespaces, search, searchVector, explainSearch, generateSnippet, purgeExpired · insertNode, getNode, deleteNode, addEdge, graphBfs/Dfs/TopologicalSort/IsDag/FilteredTraversal/Degree · query (IQL), flush, compactWal, compactLayout, rebuildIndex, reindexHnswFromText, repairTextIndex, auditTextIndex(+Deep), exportAll/exportNamespace(+filtered), importRecords/importFile · close, capabilities, operationalMetrics.

**Faltante vs core (verificado):**
- Versionado: `get_version`, `versions` (VS-CORE-07), `supersede`.
- Consultas/métricas: `count`, `namespace_stats`, `similar_to_key`, `search_multi`, `search_all`, `search_with_method`, `graphrag_search`, `debug_memory_breakdown`.
- Grafo: `remove_edge` (¡hay add sin remove!), `graph_dfs_filtered` (solo BFS filtrado está bindeado), acumuladores GDS, `graph_page_rank`.
- Hilos/wiki: `create_thread/send_message/get_thread/list_threads/delete_thread/purge_expired_threads`, `recover_archived_nodes` (wiki client vacío — documentado).
- Portabilidad: `bulk_import_file/bulk_import_stream/bulk_import_bytes`; snapshots (`create_snapshot/list_snapshots`); `vacuum`, `pipeline`, optimizer config.
- Parámetros perdidos: `filter_ops` avanzados en `list` (solo pasa el path legacy `filters`), `exclude_superseded`, `sparse_vector` (siempre `None`), `search_profile`.

## 6. Incompletudes

- `NativeVantaDB` cubre solo ~11 métodos (subset memoria); sin grafo, IQL, export/import ni métricas. El docstring lo reconoce ("exposed subset") pero no hay roadmap en el código.
- Sin tests para `NativeVantaDB` dentro de este paquete (la cobertura vive en vantadb-node).
- No hay changelog propio ni versión sincronizada visible con wasm/node (0.5.0 en ambos package.json — ok hoy, frágil mañana).

## 7. Propuestas (ordenadas por leverage)

1. Corregir `GraphBfsResult/DfsResult/TopoSort` para que reflejen `u128[]` (o envolver en el binding WASM con el shape documentado) + test que afirme el shape real, no `toBeDefined()`.
2. Fix de 3 líneas en `NativeVantaDB._native` (await + catch).
3. Decidir y documentar la semántica de `distance`/`score` una sola vez, en core, y replicarla (ver reporte wasm §consistencia).
4. Unificar `_mapRecord`/`_buildSearchRequest`/wrappers de error en un `internal/shared.ts`.
5. Exponer `remove_edge` y `count` (baratos de bindear, cierran huecos obvios).

## 8. Consistencia con otros SDKs

| Aspecto | TS | WASM | node | Python |
|---|---|---|---|---|
| Nombres de método | camelCase | snake_case crudo | camelCase | snake_case |
| IDs de nodo | number\|bigint in, bigint edges out | strings decimales | **no existe grafo** | strings (ERR-023) |
| Errores | `VantaError{code}` local; core colapsado a string | `js_sys::Error` string | `napi::Error` string → NATIVE_ERROR | PyErr |
| Hit de búsqueda | `.distance` (¿invertido?) | campo `score` | campo `score` (similitud según test) | — |
| Cap vector | hereda wasm 10M | 10_000_000 | 10_000 | — |

Divergencias activas: semántica score/distance (#3), cap de dimensión de vector (10M vs 10k entre wasm y node), ausencia total de grafo/IQL en node. La duplicación de cliente (retry/cursor/parsing) está bien resuelta aquí: parsing y cursor viven en core/Rust; TS no reimplementa retry (solo el worker proxy de wasm lo hace).

## 9. Score

**7 / 10** — Excelente DX y arquitectura de wrapper; penaliza fuerte el hallazgo #1 (contrato de tipos de grafo roto y no testeado), el bug #2 y la semántica distance/score ambigua. Con esos tres fixes sería 8.5+.

---
*Siguiente:* [`vantadb-wasm.md`](./vantadb-wasm.md) · [`vantadb-node.md`](./vantadb-node.md)

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| #1 — Tipos de grafo ficticios (`GraphBfsResult`/`GraphDfsResult`/`GraphTopologicalSortResult` vs `u128[]` real) | **MOD-22** |
| #2 — `NativeVantaDB._native` no envuelve rechazos async (catch solo-síncrono) | **MOD-23** |
| #3, #4–#10 — nits (semántica invertida distance/score en JSDoc, guard type-lie, duplicación `_mapRecord`, ejemplos JSDoc que no compilan, dependencia `pkg/`) | **MOD-24** |
