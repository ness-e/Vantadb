---
title: "Auditoria profunda de la frontera FFI y los bindings - 2026-10-21"
kind: review
status: active
description: "Auditoria de solo lectura de vantadb-ffi-core, vantadb-node, vantadb-ts y vantadb-wasm. La frontera es memoria-segura por construccion (cero unsafe en los 3 transportes), pero hay divergencia silenciosa de contrato entre backends y stubs que fabrican datos. 7 hallazgos High, 10 Medium, 6 Low. Cierra FIND-504."
tags: [vantadb, audit, reviews, ffi, bindings, node, wasm, typescript]
---

# Auditoría profunda de la frontera FFI y los bindings

> **Campaña:** auditoría read-only del 2026-10-21, disparada por `FIND-504` (la campaña del
> 2026-10-20 declaró `vantadb-ffi-core/`, `vantadb-node/index.cjs`, `vantadb-ts/src/native.ts`
> completo y `vantadb-ts/examples/` como **no cubiertos**).
> **Alcance real:** `vantadb-ffi-core/`, `vantadb-node/`, `vantadb-ts/`, `vantadb-wasm/` y la
> frontera del core que exponen (`src/lib.rs`, `src/sdk/**`, `src/storage/vfile.rs`,
> `src/node/disk.rs`).
> **Método:** lectura + `grep` + `codegraph`; la semántica de `napi-rs` y de tokio se validó
> contra el código de `napi-3.12.0` en el registry local, y la saturación `f64 → f32` contra el
> Rust Reference oficial.
> **Sin `cargo` ni `npm`** (había otro agente compilando y la página de Windows se agota): los
> cuatro hallazgos de runtime que no se pudieron ejecutar van marcados como *sospechados*.
> **Skills:** `security-and-hardening`, `code-review-and-quality`, `doubt-driven-development`,
> `source-driven-development`.

## 0. Veredicto corto

La frontera FFI es **memoria-segura por construcción**: **cero `unsafe`** en los 3 transportes
(`grep` de `unsafe {` / `unwrap_unchecked` / `get_unchecked` / `transmute` / `from_raw_parts` en
`vantadb-ffi-core/`, `vantadb-node/src/`, `vantadb-wasm/src/`, `vantadb-python/src/` → **0 hits**;
los únicos `unsafe` del repo en la zona son `src/storage/vfile.rs:160,168` y
`src/index_port.rs:109`, con `SAFETY` completo). El puntero crudo lo maneja napi-rs internamente.

Los problemas reales **no son de corrupción de memoria** sino de:

1. **Divergencia silenciosa de contrato entre backends** (H1, H2, M2-M5).
2. **Stubs que fabrican datos en vez de fallar** (H5, H6, M1): el wrapper TS devuelve una métrica
   falsa, un camino de grafo inventado y una lista vacía cuando el binding no tiene el método.
3. **Documentación y ejemplos que mienten** (H7, M6, M7, M8, M9, L1, L2).

**7 High · 10 Medium · 6 Low.**

## 1. Inventario de la frontera

### 1.1 `vantadb-ffi-core/` — hoja std-only compartida

| Símbolo | Ubicación | Qué expone |
|---|---|---|
| `OpGate` / `OpGuard` | `vantadb-ffi-core/src/lib.rs:73-154` | Barrera de durabilidad: `try_enter()` (`:100-110`) rechaza tras `drain()`; `drain()` (`:122-137`) bloquea hasta `count == 0`. Clone comparte estado (`:72`) |
| `clamp_top_k(requested, max) -> (usize, bool)` | `:167-173` | Política ERR-022 *compare → cap → report*; el límite vive en el core (`vantadb::MAX_K`) |
| Consumidores | `vantadb-node/Cargo.toml:30`, `vantadb-wasm/Cargo.toml:26`, `vantadb-python/Cargo.toml:26` | Los 3 transportes. `publish = false` (`:8`), `[dependencies]` vacío (`:12`) → hoja real |

**Positivo:** `PoisonError::into_inner` en los 4 sitios de lock (`:102, :124, :133, :150`) →
recuperación de poisoning sin pánico; el `wasm32` está tratado explícitamente (`:126-136`).

### 1.2 `vantadb-node/` — napi-rs (43 métodos `#[napi]`)

| Grupo | Métodos (Rust `src/lib.rs` → JS camelCase) |
|---|---|
| Ciclo de vida | `connect:240`, `reopen:288`, `flush:349`, `close:363` + `AbortHandle:78-104` + finalizador GC `finalize:210` |
| Memoria | `put:385`, `put_batch:398`, `get:417`, `delete:430`, `list:441`, `list_namespaces:455` |
| Búsqueda | `search:485` (+`abort`), `explain_search:737`, `similar_to_key:887`, `search_with_method:908`, `search_multi:928`, `graphrag_search:953` |
| Grafo | `insert_node:550`, `get_node:562`, `delete_node:575`, `add_edge:588`, `remove_edge:615`, `graph_bfs:633`, `graph_dfs:652`, `graph_topological_sort:670`, `graph_is_dag:680`, `graph_filtered_traversal:694`, `graph_degree:718` |
| Versiones | `versions:754`, `get_version:764`, `supersede:784` |
| Mantenimiento | `vacuum:797`, `rebuild_index:815`, `compact_layout:825`, `compact_wal:835`, `purge_expired:844` |
| Filtros/conteo | `delete_by_filter:855`, `count:869` |
| Fachada cognitiva (DIST-19) | `memory_recall:1007`, `context_assemble:1063`, `memory_consolidate:1119`, `memory_reinforce:1151` |
| Checkpoints | `checkpoint_save:1173`, `checkpoint_load:1194` |

**Positivo:** el `catch_unwind` del finalizador (`src/lib.rs:211`) con `SAFETY` documentado
(`:206-209`) evita desenrollar a través del `extern "C"`. Toda op entra por el gate
(`enter(&gate)`, `src/lib.rs:1355-1358`) y el `OpGuard` vive a través del `await` (R-4 de
`js-ecosystem.md`).

### 1.3 El resto

- `vantadb-ts/src/native.ts`: `class NativeVantaDB` (`:292-1169`) + `interface NativeClientBinding`
  (`:72-179`) que **declara 58 métodos**; el wrapper implementa 47 y **4 son degradaciones
  silenciosas** (H5, H6, M1, H2).
- `vantadb-ts/src/vantadb.ts`: `class Client` (`:181-1763`), superficie plana + sub-clientes
  `memory`/`graph`/`wiki`/`system` (`:316-430`).
- `vantadb-wasm/src/lib.rs`: `ListOptions` (`:226-248`), `SearchRequest` (`:181-215`),
  `memory_record_to_js` (`:2396-2525`).
- Core: `src/lib.rs:193-203` re-exporta `MemoryListOptions`, `MemorySearchRequest`, `MemoryRecord`,
  `ValidWindow`, `MemoryMetadata`…; los tipos viven en `src/sdk/types/record.rs:381-422` (10 campos)
  y `src/sdk/serialization/vector_types.rs:102-170` (18 campos).

## 2. Hallazgos

### 2.1 High

#### **H1 · `record.metadata` tiene forma distinta en WASM y en Node, y el tipo público dice una sola cosa**

- `vantadb-ts/src/types.ts:24` → `export type Metadata = Record<string, Value>;`
- `vantadb-ts/src/__tests__/flat-metadata.test.ts:14-17` → *"Runtime metadata is a Map<string, Value> — convert for assertions"*.
- Causa raíz: `vantadb-wasm/src/lib.rs:2515` serializa con `serde_wasm_bindgen::to_value(&rec.metadata)`
  (serializer por defecto) → el `BTreeMap` sale como `Map` de ES2015, no objeto plano. El propio repo
  lo documenta en `vantadb-wasm/src/lib.rs:2556-2558`.
- El backend Node sí devuelve objeto plano: `vantadb-node/src/lib.rs:393` `serde_json::to_value(&out)`.
- **El guard no lo detecta**: `isMetadata()` (`vantadb-ts/src/guards.ts:107-111`) hace
  `Object.values(new Map())` → `[]` → `every` → `true`.
- **Impacto:** `rec.metadata.lang` → `undefined` en WASM, `{String:"en"}` en Node;
  `rec.metadata.get("lang")` → funciona en WASM, revienta en Node.

→ **FIND-532**. Fix: `Serializer::json_compatible()` en el WASM (el mismo helper que ya usa el
camino de entrada, `lib.rs:2559-2562`) o `Object.fromEntries` en el SDK.

#### **H2 · `NativeVantaDB.searchMulti` lanza siempre**

- `vantadb-ts/src/native.ts:1016-1041` pasa la request por `_buildSearchRequest(req)` (`:1030`).
- `vantadb-ts/src/guards.ts:214-218` exige `namespace` no vacío → lanza `DbError(VALIDATION_ERROR)`.
- Forma **objeto** (`BatchSearchRequest` = `Omit<SearchRequest,"namespace">`, `types.ts:252-255`):
  `req.namespace` es `undefined` → **lanza**.
- Forma **array**: el segundo argumento debe traer `namespace` no vacío, y el ejemplo canónico
  (`docs/api/NODE_SDK.md:342`) usa `namespace: ""` → **lanza**.
- El backend WASM sí tiene el workaround (`vantadb-ts/src/vantadb.ts:835-838`).
- **Cobertura:** el test de `searchMulti` (`integration.test.ts:190-210`) solo ejercita WASM
  (`Client`).

→ **FIND-533**.

#### **H3 · `OpGate::drain()` bloquea un worker de tokio dentro de un método async → deadlock con runtime de 1 worker**

- `vantadb-ffi-core/src/lib.rs:122-137`: `drain()` hace `cvar.wait(state)` (`:133`) → **bloquea el
  hilo del SO**, no cede el scheduler.
- Se llama desde `#[napi] pub async fn`: `vantadb-node/src/lib.rs:300` (`reopen`), `:365` (`close`),
  `:221` (`finalize`, hilo de GC de V8).
- El future de un `async fn` de napi-rs se **spawlea al runtime de tokio**
  (`napi-3.12.0/src/tokio_runtime.rs:1243` → `spawn(inner)`); el runtime por defecto es
  **multi-thread** (`:853`), con `worker_threads = available_parallelism()`.
- **Mecanismo:** `drain()` espera a que caiga `count`, y `count` baja cuando el `OpGuard` de la op
  en vuelo se dropea → lo cual exige que su future (el `await` del `spawn_blocking`) **sea polled**.
  Si el único worker está parqueado en `cvar.wait()` dentro de `close()`, nadie lo pollea →
  `count` nunca llega a 0 → **deadlock permanente** del proceso Node.
- Con ≥2 workers el work-stealing lo salva casi siempre; con **1 worker** (contenedor o CI de 1 CPU)
  el deadlock es determinista.

→ **FIND-534**. Fix: `tokio::task::block_in_place(|| gate.drain())` (feature `rt` ya presente en
`vantadb-node/Cargo.toml:23`).

#### **H4 · El camino de vector denso acepta `f64` que desborda a `f32::INFINITY`; el sparse ya fue corregido**

- `vantadb-node/src/lib.rs:1650-1657` y `:1675-1684`:
  `v.as_f64().filter(|f| f.is_finite()).map(|f| f as f32)` → el filtro es a nivel **f64**.
  `1e300` es finito en f64 → pasa → `1e300f64 as f32` = `f32::INFINITY` (definido en el Rust
  Reference: *"on overflow, infinity (of the same sign as the input) is produced"*).
- El mismo bug en `:971-974` (`graphrag_search`).
- El camino **sparse** sí lo rechaza, con un comentario que describe este bug exacto
  (`vantadb-node/src/lib.rs:1437-1444`).
- El **core no valida finitud**: `src/sdk/api/memory.rs:712-714` solo valida la dimensión;
  `usable_vector` (`:260-262`) solo comprueba `!v.is_empty() && any(|x| *x != 0.0)` → `inf != 0.0`
  → el vector se considera usable y **se indexa**.
- El TS tampoco lo atrapa (`guards.ts:149-155`, `:247-255` chequean `Number.isFinite` a nivel f64).

→ **FIND-535**. Impacto: la similitud coseno da **NaN** → registro imrancable; en `search`, un
vector con `inf` produce NaN en todos los scores → `top_k` devuelve un conjunto arbitrario.

#### **H5 · `NativeVantaDB.metricsPrometheus()` fabrica métricas**

`vantadb-ts/src/native.ts:1150-1168`: si el método no existe en el binding nativo (y no existe:
no está en la lista de `#[napi] pub async fn` ni en `index.d.ts:388-657`), devuelve la constante
`"# HELP vantadb_up ...\nvantadb_up 1\n"`.

→ **FIND-536**. Es **fabricación de datos** en una superficie de confianza: un scraper ve el motor
"up" para siempre, incluso con la base caída.

#### **H6 · `NativeVantaDB.graphShortestPath()` devuelve un camino inventado**

`vantadb-ts/src/native.ts:915-955`: si el método no existe, hace un BFS y **fabrica**:
`if (visited.includes(tgt)) { return [src, tgt]; }` (`:950-951`) → un camino de **2 aristas** sea
cual sea el real.

→ **FIND-537**.

#### **H7 · Los 6 ejemplos de `vantadb-ts/examples/` importan un símbolo que el paquete no exporta**

Los 6 hacen `import { VantaDB } from "vantadb"`
(`examples/vercel-ai/index.ts:1`, `examples/llamaindex/index.ts:1`, `examples/langchain/index.ts:1`,
`examples/vercel-ai-memory.mjs:12`, `examples/llamaindex-rag.mjs:12`, `examples/langchain-rag.mjs:12`).
El paquete exporta `Client` (`vantadb.ts:181`), `DbError`, `ERROR_CODES`, los guards y
`export * from "./native.js"` (`:1765-1775`): **no existe ningún `export` de `VantaDB`**
(0 coincidencias en `vantadb-ts/src/*.ts`). El README del mismo paquete usa lo correcto
(`vantadb-ts/README.md:10,36`).

Bug secundario: `examples/vercel-ai/index.ts:50` usa `db.get("context", "kb_calendar")` (posicional)
pero `Client.get(input: GetInput)` toma un **objeto** (`vantadb.ts:558`).

**Por qué el CI no lo detecta:** `ci-fleet.yml` corre los ejemplos de la **raíz**
(`examples/python/*`, `examples/demo/*`), no `vantadb-ts/examples/`; el gate de docs valida
ejemplos **Python**; y `vantadb-ts/examples/*/package.json` no tiene ningún script de test.

→ **FIND-538**.

### 2.2 Medium

| # | Hallazgo | Ubicación | Fila |
|---|---|---|---|
| **M1** | `findNodesByLabel()` devuelve `[]` incondicionalmente → "no existe la etiqueta" y "no soportado" son el mismo resultado | `native.ts:823-831` | `FIND-539` |
| **M2** | El `.d.ts` del WASM **no** declara `exclude_superseded` en `ListOptionsInput` (`:234-253`) pero el runtime sí lo honra (`vantadb-wasm/src/lib.rs:233-235`, propagado en `:1229`); el `SearchRequestInput` del **mismo fichero** sí lo declara (`:270`) → asimetría dentro del mismo `.d.ts`. El SDK TS lo salva por un `spread` con cast (`vantadb.ts:628`) → el SDK depende de un **cast** para propagar un campo que el `.d.ts` niega | `vantadb-wasm/src/vantadb_wasm.d.ts:234-253` | actualiza `FIND-482` |
| **M3** | `memory_record_to_js` **no emite** `superseded_by` ni `superseded_at_ms` (sí existen en el core: `src/sdk/types/record.rs:283`; el backend Node los emite). Tras `supersede(ns, old, new)`, un cliente WASM no puede saber qué registro quedó supersedido leyendo el registro | `vantadb-wasm/src/lib.rs:2396-2525` | actualiza `FIND-483` |
| **M4** | El `.d.ts` del WASM declara `EdgeRecord.target_id` pero el campo real es `target` (`src/sdk/serialization/graph_types.rs:9-26`; el test del WASM lee `"target"`, `vantadb-wasm/src/lib.rs:2121`), y **omite `reverse`**, que es load-bearing para `TraversalDirection::Reverse` (`graph_types.rs:16-21`) | `vantadb_wasm.d.ts:342-347`, `vantadb-ts/src/types.ts:300-309` | `FIND-540` |
| **M5** | El `.d.ts` del WASM describe el metadata como JSON plano (`:188,192-198`), pero en el cable es la forma **etiquetada** (`{String:"en"}`, `enum VantaValue` externally-tagged) y dentro de un `Map`. El SDK TS sí usa la etiquetada (`types.ts:11-22`) → los tres sitios no coinciden | `vantadb_wasm.d.ts:188` | `FIND-541` |
| **M6** | El ejemplo del README de `vantadb-node` revienta: `search()` devuelve `Promise<MemorySearchHit[]>` (array), y el README hace `hit.hits[0]` → `TypeError`. El doc canónico lo hace bien (`docs/api/NODE_SDK.md:60-66`) | `vantadb-node/README.md:44-49` | `FIND-542` |
| **M7** | `docs/api/NODE_SDK.md` omite 9 métodos publicos reales (0 hits para `exclude_superseded`, `reopen`, `checkpointSave`, `checkpointLoad`, `memoryRecall`, `contextAssemble`, `memoryConsolidate`, `memoryReinforce`, `AbortHandle`) | `NODE_SDK.md:151-188`, `:238-254` | `FIND-543` |
| **M8** | `NODE_SDK.md:190-201` afirma que el `.d.ts` declara `Promise<number>` "stale" — pero ya declara `bigint` (0 hits de `Promise<number>`) | `NODE_SDK.md:190-201`, `index.d.ts:566,569,578,583` | `FIND-544` |
| **M9** | `DiskNodeHeader::vector_offset` (campo `pub` de un struct `pub`) cambió de significado en v3 (offset en-blob) y su doc-comment no lo refleja; `ADR-0032:63,91,150` y `ADR-0049:243` describen la semántica vieja | `src/node/disk.rs:16-17` | `FIND-545` |
| **M10** | `index.cjs:705` y `index.js:709` exportan `VantaDB` = `undefined` (la clase real es `VantaDb`; `index.d.ts:658` `export type VantaDB = VantaDb` es un alias de **tipo**) | `vantadb-node/index.cjs:702-705` | `FIND-546` |

### 2.3 Low

| # | Hallazgo | Ubicación | Fila |
|---|---|---|---|
| **L1+L2** | Los `index.cjs`/`index.js` commiteados llevan `0.8.0` hardcodeado en **27 sitios** (el `package.json` dice `0.9.0`); el artefactado publicado no sufre esto (`release-npm-node.yml:104-105` lo regenera), pero un `npm i git+…` sí lo recibe, y con `NAPI_RS_ENFORCE_VERSION_CHECK` (opt-in) el check falla. Comentario obsoleto en `vantadb-node/Cargo.toml:26` | `vantadb-node/index.cjs`, `index.js`, `Cargo.toml:26` | `FIND-547` |
| **L3** | `read_header` reporta un fallo de descifrado como "no hay header" (`.ok()` se come el error) — **FIND-509 (a), sigue abierto** | `src/storage/vfile.rs:581-597` | actualiza `FIND-509` |
| **L4** | El `madvise` de prelectura usa `h.vector_offset` como dirección de fichero → en segmentos cifrados el hint es inútil (no incorrecto) — **FIND-509 (c), sigue abierto**. El resto de consumidores son correctos | `src/index/search/layer.rs:191-202` | actualiza `FIND-509` |
| **L5** | `reopen()` libera la reserva de la ruta antes de volver a reservarla → carrera TOCTOU: otra hebra puede cogerla y el `reopen` falla con `VANTADB_BUSY` | `vantadb-node/src/lib.rs:302-306`, `:318-321` | `FIND-548` |
| **L6** | El "zero-copy" del vector WASM **no** es un zero-copy: `Float32Array::new_with_length` + `copy_from` **copia**. Sin riesgo de aliasing (0 vistas en el crate), pero el comentario (`:2416`) y el bloque `ponytail:` (`:2534-2539`) describen un beneficio que no se entrega | `vantadb-wasm/src/lib.rs:2440-2442` | `FIND-549` |

## 3. Sospechados (no verificados — requieren ejecución o un entorno no disponible)

| # | Sospecha | Por qué no se confirma | Cómo verificarla |
|---|---|---|---|
| **S1** | El deadlock de H3 se manifiesta con 1 worker | El mecanismo está confirmado leyendo (`spawn(inner)`, `cvar.wait`, `worker_threads` por defecto), pero no se ejecutó | Runner de 1 CPU: `connect(":memory:")` → `put` sin await → `await close()`; o `TOKIO_WORKER_THREADS=1` |
| **S2** | Un vector con `inf` *envenena* el grafo HNSW (no solo produce NaN) | Confirmado que el `inf` **se persiste** y que la similitud da NaN; no seVWraceó el comparador del heap con NaN | Test en el core: `put` con `vector: [f32::INFINITY, 0.0]` → inspeccionar grafo y scores |
| **S3** | `NativeVantaDB.getEdges()` (`:809-818`) devuelve edges incompletos | El fallback hace `getNode(id).edges` y el core sí puebla `edges` (`src/sdk/serialization/graph_types.rs:88-127`) → parece correcto, no verificado en runtime | Test nativo: `insertNode` + `addEdge` → `getEdges` vs `getNode().edges` |
| **S4** | El paquete `vantadb` publicado en npm **no** puede usar el backend nativo | `release-npm-61.yml:307` **borra** `vantadb-node` de `dependencies` (es `file:`) sin re-agregar como `optionalDependency` con versión → el `await import("vantadb-node")` (`vantadb-ts/src/native.ts:327`) no resolvería desde el registry | `npm view vantadb@0.9.0 dependencies` + instalación limpia → `await import("vantadb-node")` |

## 4. Veredicto de paridad de superficie (core ↔ node ↔ ts ↔ wasm)

| Capacidad | Core | Node | TS nativo | TS WASM | WASM directo | Paridad |
|---|---|---|---|---|---|---|
| `put`/`putBatch`/`get`/`delete` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `list` + `filters`/`limit`/`cursor` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `list` + `exclude_superseded` | ✅ | ✅ | ✅ | ✅ | ✅ runtime / ❌ **tipo** | ⚠️ M2 |
| `list` + `as_of_ms`/`valid_window`/`include_quarantined`/`min_confidence` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `search` + `exclude_superseded` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `search` + `query_sparse` | ✅ | ✅ | ✅ | ❌ **lanza** a propósito (`vantadb.ts:704-709`) | ❌ | ⚠️ divergencia **deliberada y documentada** |
| `search` + `range`/`group_by`/`mmr`/`cursor`/`search_profile` | ✅ | ❌ | ❌ | ❌ | ❌ | ⚠️ gap documentado (SCH-04 §3) |
| `record.metadata` (forma) | `BTreeMap<String,Value>` | **objeto plano** | objeto plano | **`Map`** | **`Map`** | ❌ **H1** |
| `record.superseded_by` | ✅ | ✅ | ✅ | ❌ **no emitido** | ❌ | ❌ **M3** |
| `record.vector` (tipo) | `Vec<f32>` | `number[]` | `number[]` | `Float32Array` | `Float32Array` | ✅ documentado |
| `record` u64 (`created_at_ms`…) | number | number | number | **string** | **string** | ✅ (policy string-u64) |
| `graphShortestPath` | ✅ (`src/gds.rs`) | ❌ | ⚠️ **inventa** | ❌ | ❌ | ❌ **H6** |
| `findNodesByLabel` | ✅ | ❌ | ⚠️ **`[]`** | ❌ | ❌ | ❌ **M1** |
| `metricsPrometheus` | ✅ (`OperationalMetrics`) | ❌ | ⚠️ **métrica falsa** | ✅ | ✅ | ❌ **H5** |
| `getEdges` | ✅ | ⚠️ vía `getNode` | ⚠️ idem | ✅ | ✅ | ⚠️ |
| `searchMulti` | ✅ | ✅ | ❌ **lanza siempre** | ✅ | ✅ | ❌ **H2** |
| `reopen` | ✅ | ✅ | ✅ | ❌ | ❌ (`connect_persistent`) | ⚠️ |
| `checkpointSave/Load` | ✅ | ✅ | ❌ | ❌ | ❌ | ⚠️ |
| `memoryRecall`/`contextAssemble`/`memoryConsolidate`/`memoryReinforce` | ✅ (`vanta-memory`) | ✅ | ❌ | ❌ | ❌ | ⚠️ |
| `supersede` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `versions`/`getVersion` | ✅ | ✅ | ✅ | ❌ | ❌ | ⚠️ |
| `vacuum`/`rebuildIndex`/`compactLayout`/`compactWal`/`purgeExpired` | ✅ | ✅ | ✅ | parcial | parcial | ✅ |
| `count`/`deleteByFilter` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `AbortHandle` (cancelación) | n/a | ✅ | ❌ | ❌ | ❌ | ⚠️ |

**Lectura de conjunto:** la paridad **funcional** de los flags de consulta (SCH-03/04/05/07) está
**completa y bien resuelta** — S3-A1 está realmente corregido. La paridad rota está en (a) **la
forma de `metadata`** (H1, transversal WASM↔Node), (b) **métodos que el wrapper TS promete y el
binding no tiene**, resueltos con stubs silenciosos (H5, H6, M1) en vez de con un error, y (c)
**superficie pública del core que ningún binding expone** (`range`/`group_by`/`mmr`/`search_profile`).

## 5. Impacto de `vector_offset`

### 5.1 La premisa del encargo, revisada

> *"cualquier binding que exponga ese campo se estaría equivocando"*

**Confirmado por lectura: ningún binding expone `vector_offset`.** `grep vector_offset` sobre
`*.{ts,js,cjs,mjs,py,pyi}` en todo el repo → **0 hits** fuera de `docs/`; sobre
`vantadb-node/`, `vantadb-wasm/`, `vantadb-python/` → 0 hits.

### 5.2 Consumidores reales del campo

| Consumidor | Ubicación | ¿Trata el valor nuevo? | Veredicto |
|---|---|---|---|
| Escritor (plaintext) | `src/storage/ops.rs:86` | solo plaintext | ✅ |
| Escritor (cifrado) | `src/storage/vfile.rs:958-960` | escribe 64 (offset en-blob) | ✅ |
| `File::read_header` | `src/storage/vfile.rs:581-609` | valida `is_multiple_of(4)` (`:605`): 64 % 4 == 0 en ambos formatos | ✅ (pero ver L3) |
| `File::node_bytes` | `src/storage/vfile.rs:709-747` | bifurca por `is_encrypted()` → `NodeBytes::Blob` | ✅ |
| `File::read_full_vector` | `src/storage/vfile.rs:1046-1066` | usa `ENCRYPTED_PAYLOAD_OFFSET` explícitamente (`:1058`) | ✅ |
| `index_port::full_vector_from_bytes` | `src/index_port.rs:98-114` | solo recibe el mapping **plaintext** | ✅ (pero `pub` y latente) |
| Lectores del engine | `src/storage/engine/get.rs:76-178`, `txn.rs:569-642`, `ops.rs:296-313` | indexan dentro del `NodeBytes` | ✅ |
| `src/storage/archive.rs:370-478` | compactación | headers del mapping plaintext | ✅ |
| **Prefetch `madvise`** | `src/index/search/layer.rs:191-202` | usa `h.vector_offset` como dirección de fichero | ❌ hint inútil (sin impacto de correctitud) |
| **Doc-comment público** | `src/node/disk.rs:16-17` | dice solo "Offset to vector data in the MMap file" | ❌ |
| **ADR-0032 / ADR-0049** | `ADR-0032:63,91,150`, `ADR-0049:243` | solo semántica plaintext | ⚠️ desactualizado para v3 |
| **Plan de la tanda** | `docs/dev/plans/2026-10-20-wiring-encryption-feature.md:274` | documenta el significado nuevo | ✅ referencia canónica |

### 5.3 Conclusión

1. **No hay binding roto.** La premisa de `FIND-504` es *correcta pero vacía*: no existe tal FFI
   hoy. El riesgo es para un binding **futuro** o para un consumidor Rust directo de
   `vantadb::node::DiskNodeHeader`.
2. **El daño real es de API pública + docs**, no de FFI: un campo `pub` de un struct `pub` cambió
   de significado entre 0.8.0 y 0.9.0 **sin** que su doc-comment lo refleje, y dos ADRs describen
   la semántica vieja.
3. **El único consumidor funcionalmente afectado** es el hint de `madvise`, sin impacto de
   correctitud.
4. **El riesgo latente mayor**: `full_vector_from_bytes` (`src/index_port.rs:98`) es `pub` y asume
   que `vector_offset` es una dirección de fichero; si alguien la llama con un header de un
   segmento cifrado, lee **bytes equivocados en silencio** (no UB: el slice está bounds-checked y
   64 % 4 == 0 mantiene la alineación). Recomendación: renombrarla a
   `full_vector_from_plaintext_bytes` o pedir explícitamente el `NodeBytes`.

## 6. Lo que no se pudo verificar

| Ítem | Motivo |
|---|---|
| **SAST (semgrep / ast-grep)** — gate L7/L10 | **No están instalados** en esta máquina (`Get-Command` → vacío). **No se afirma "scan limpio"**: no se ejecutó. Se hizo el equivalente manual: `grep` de `unsafe {` / `unwrap_unchecked` / `get_unchecked` / `transmute` / `from_raw_parts` en los 4 transportes → **0 hits** |
| **Compilación / tests / Miri** | El encargo prohíbe `cargo` y `npm` (otra compilación en curso, página de Windows agotada). No hay evidencia de compilación: todos los hallazgos son de lectura |
| **Runtime de H2, H5, H6, M1, H7** | Derivados de lectura con evidencia textual directa (el `throw` de `guards.ts:218`, el `return []` de `native.ts:829`, la constante de `native.ts:1166`, el `return [src, tgt]` de `native.ts:950`, el `export` inexistente). Un `tsc --noEmit` sobre `vantadb-ts/examples/` y un `vitest` de `searchMulti` nativo confirmarían en minutos |
| **Muerte por deadlock de H3** | Mecanismo confirmado leyendo napi-rs 3.12.0 y tokio; la reproducción necesita un runner de 1 CPU (S1) |
| **`cargo audit` / `cargo deny` / `cargo machete`** | Requerían `cargo`. Revisión estática de `Cargo.toml`: `vantadb-ffi-core` con `[dependencies]` **vacío** → hoja real, cumple su contrato documentado (`src/lib.rs:29-33`); `vantadb-node` tiene 7 deps runtime + `napi-build` (build) — sin typosquats evidentes; `vantadb-ts` declara `file:` deps reescritas en el publish (correcto, pero `vantadb-node` **se borra** → S4) |
| **Contenido publicado en npm** | Requiere `npm view` (prohibido). Verificado solo por lectura de los workflows |
| **Superficie de `vantadb-python`** | Fuera del encargo explícito; solo se tocó para confirmar que comparte `OpGate`/`clamp_top_k` y que `exclude_superseded` existe. No es un binding con `unsafe` (0 hits) |

## 7. Estado de los hallazgos de la campaña del 2026-10-20

| ID | Enunciado | Estado tras esta auditoría |
|---|---|---|
| **S3-A1** (P1) | `exclude_superseded` se honraba solo en WASM | **CORREGIDO — confirmado.** `vantadb-node/src/lib.rs:1503` (list) y `:1541` (search) leen el flag; `vantadb-ts/src/native.ts:498` y `:541` lo propagan. Tests de regresión en `vantadb-node/tests/find412.test.ts:34-69` y `vantadb-ts/src/__tests__/find412.test.ts:43-124`, **con precondición explícita** (`:27-31` — sin el supersede las aserciones pasarían vacuamente). **La premisa del encargo de que "el `.cjs` no lo propaga" es incorrecta**: `index.cjs:702-705` es un loader puro (`module.exports = nativeBinding`), sin wrappers por método → propaga por referencia. Su no-modificación **no es un hallazgo**. *Pendiente menor:* la nota del fix ("añadir el caso a `docs/api/SEARCH_PARITY.md`") apunta a un fichero que es una comparación Milvus/Qdrant; la tabla correcta es `docs/api/BINDINGS_NAMESPACES.md:187-188`, que **ya** lista `exclude_superseded` ✅ para los 3 bindings |
| **S3-A8** | (gaps de capacidades en Node, `:315`) | **PARCIALMENTE CONFIRMADO / AGRAVADO.** 4 gaps de capacidad en Node con degradación silenciosa en el wrapper TS: `getEdges` (⚠️), `findNodesByLabel` (**M1**), `graphShortestPath` (**H6**), `metricsPrometheus` (**H5**). Ninguno está marcado como "no soportado" en `NativeClientBinding` (`native.ts:72-179`), que los **promete** — y dos devuelven un valor falso en vez de un error. Material **nuevo** (no estaba en los 141 hallazgos) |
| **S3-A1 (sibling)** | `searchMulti` nativa rota | **HALLAZGO NUEVO — H2.** Divergencia cross-backend de la misma clase, en otro método, y **sin cobertura de test** |
| **VDB-02..VDB-05** | (escala de score Euclidean, bitset truncado, WAL >10MB, `MAX_K` sin clampear) | El primero tiene un eco en la frontera: el clamp de `MAX_K` **sí** está unificado en `vantadb-ffi-core/src/lib.rs:167-173` y lo usan node, wasm y python → **confirmado corregido en la frontera**. Los otros tres requieren `cargo` |
| **FIND-412** | `exclude_superseded` descartado en la ruta nativa | **CORREGIDO** (ver S3-A1). La premisa "no lo propagó en `index.cjs`" queda **invalidada** |
| **FIND-504** | `vantadb-ffi-core/`, `index.cjs`, `native.ts` completo y `examples/` no analizados | **CERRADO por esta auditoría.** Resultado neto: 7 High + 10 Medium + 6 Low. La hipótesis central ("un FFI que exponga `vector_offset` se estaría equivocando") queda **invalidada por evidencia** (0 bindings exponen el campo; el daño real es API pública + docs). La hipótesis "el `.cjs` es el último bloqueo de cadena de suministro" queda **invalidada**: el `.cjs` **falla ruidosamente** cuando no encuentra el binding (`index.cjs:687-700` → `throw`), no hay stub no-op; el riesgo de fallo silencioso está **en `vantadb-ts/src/native.ts`** (H5/H6/M1), no en el `.cjs` |
| **FIND-509** | `read_header` → `None`; `vector_offset` cambia de significado; `madvise` usa el offset viejo | **CONFIRMADO, sigue ABIERTO (3/3).** (a) `src/storage/vfile.rs:581-597` — L3. (b) el cambio de significado es real y el doc-comment quedó obsoleto — M9. (c) `src/index/search/layer.rs:191-202` — L4. Ninguna de las 3 se corrigió desde el 2026-10-20 |
| **FIND-482** | El WASM no declara `exclude_superseded` en `ListOptionsInput` | **CONFIRMADO** (M2): el runtime lo honra, el `.d.ts` no, y el SDK TS depende de un cast para propagarlo |
| **FIND-483** | El WASM no emite `superseded_by` | **CONFIRMADO** (M3): el campo existe en el core y el backend Node lo emite; el WASM no |

## 8. Nota de cierre (Ponytail lite)

La lección de esta frontera no es de memoria: es que **un stub silencioso es peor que un hueco**.
`findNodesByLabel` → `[]`, `graphShortestPath` → `[src, tgt]`, `metricsPrometheus` → `vantadb_up 1`:
tres métodos que **no existen** en el binding devuelven una respuesta plausible en vez de un
error. El fix es de 3 líneas por método (lanzar `DbError` de "no soportado en este backend"), y
elimina de raíz H5, H6 y M1. H2 es el mismo patrón por el otro lado (el guard exige `namespace` y
el wrapper no lo pone): 2 líneas. H7 son 6 imports. H1 y H4 son un cambio de serializer y una
comprobación de finitud. El resto (M2-M10, L1-L6) es drift de contrato y de docs: barato, pero
que es exactamente lo que hace que un binding "funcione" y no se pueda confiar en el.
