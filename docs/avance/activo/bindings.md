---
title: "Avance — Bindings (SDK, Adapters, MCP)"
type: domain-log
status: active
tags: [vantadb, avance, bindings, python, wasm, typescript, mcp, adapters]
last_reviewed: 2026-08-07
aliases: []
---

# Avance — Bindings (SDK, Adapters, MCP)

> Registro consolidado del trabajo completado sobre bindings: Python (vantadb-python), WASM (vantadb-wasm), TypeScript, adapters (LangChain/LlamaIndex/CrewAI/DSPy/OpenAI), y MCP. IDs originales conservados.

## Python SDK

### PERF-15: PyBuffer zero-copy batch
- **Resultado:** ✅ `PyBuffer<Vec<u8>>` evita copia en batch put. +42.3% throughput.

### PERF-24: GIL scope optimization
- **Resultado:** ✅ `py.attach()`/detach en loops hot de `add_to_memory`, `vector_memory_search`, `hybrid_search`, `lexical_search`. +178% throughput (7,257 → 20,190 op/s).

### PERF-25: PyDict object pool
- **Resultado:** ✅ PyDict cache en convert.rs, +11.4% throughput.

### PERF-26: Lazy serialization
- **Resultado:** ✅ Devuelve `VantaPyMemoryRecord` en lugar de PyDict built eager; Python convierte con `#type: ignore` fallback. +24.8% throughput.

### AUD-037: error explícito de backend + unificar new()/connect() (Python)
- **Fecha:** 2026-08-16
- **Resultado:** ✅ `vantadb-python/src/lib.rs`: `parse_backend_kind` + `open_vantadb` — backend desconocido → `ValueError` (antes fallback silencioso a Fjall); `new()`/`connect()` delegan en `open_vantadb` (connect normaliza `""`/`":memory:"` + `py.detach`); docstrings actualizados. pytest 89 passed. Commit `47153977`. (ver docs/progreso/README.md)

### AUD-049: shim `import vantadb` (Python)
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `vantadb-python/vantadb/__init__.py` (nuevo): shim delgado re-exporta `vantadb_py` (import canónico `import vantadb`, compat `vantadb_py` intacto, zero breaking). `pyproject.toml`: include maturin `vantadb/__init__.py`; quickstart/docs con import canónico. Fix colateral `.gitignore:136` `*db/` matcheaba `vantadb-python/vantadb/` → excepción `!vantadb-python/vantadb/`. Commits `9a5e5305`. Wheel verificado con `import vantadb`. (ver docs/progreso/README.md)

### PERF-31: NumPy output batch
- **Resultado:** ✅ `np_per_query_batch` vía `__array_interface__` zero-copy (sin GIL), CSV header skip, `np.asarray(..., dtype=np.float32)`. +26.4% throughput.

### PERF-29: Cosine→Euclidean mapping
- **Resultado:** ✅ `MetricMapper` + `MetricCache` en `vantadb-python/src/lib.rs` mapean cosine_space→euclidean para kernels SIMD.

### PERF-36: Config hot-reload
- **Resultado:** ✅ `update_config()` reconfigura engine runtime. En config.py: `update_config()`.

### SDK Gap rec: SDK-03 / SDK-05
- SDK-03 (`delete_batch` batch delete — WASM exists): ✅ verificado.
- SDK-05 (`count()`): ✅ 2026-07-31.

### COMP-009 (Python side)
- ✅ `VantaDB.bulk_import()` + `bulk_import_bytes()` async wrappers (ver core-engine.md).

### COMP-029: Bindings Node.js/TS mediante napi-rs (backend adicional)
- **Fecha:** 2026-08-02
- **Resultado:** ✅ Crate standalone **`vantadb-node/`** (NO workspace member): `lib = "vantadb_native"` (cdylib), `napi 3` + `napi-derive` sobre `vantadb` (features `fjall, memmap2, rayon`). Aislamiento standalone evita crash del linker MSVC con cdylib en workspace. API isomórfica con wrapper WASM: `connect`, `flush`, `close`, `put`, `put_batch`, `get`, `delete`, `list`, `list_namespaces`, `search`, `capabilities` (patrón `engine.clone()` + `spawn_blocking`). Persistencia real (fjall/WAL/fsync) en Node.js — WASM no puede. Wrapper TS `vantadb-ts/src/native.ts` + dep `vantadb-node`. `npm test` vitest 3/3 (put/get, persistencia cross-reconnect, search ordenado). ADR `docs/architecture/adr/COMP-029-napi-rs-node-bindings.md`.

---

## WASM & TypeScript

### CODE-018: WASM serialization panic con NaN/Inf
- **Resultado:** ✅ Sanitización NaN/Inf→0.0 en `memory_record_to_js` y `search_hit_to_js`. Feature: javascript.

### CODE-019: TS close() debe llamar WAL flush
- **Resultado:** ✅ `this.inner.close()` en vez de `free()`; `_closed` + `_assertOpen()` en todos los métodos.

### CODE-045: OperationalMetrics 70% incompleto
- **Resultado:** ✅ 10/14 métodos implementados (solo indexed getis state → TODO documentado).

### CODE-046/087/088: _mapRecord / O(n) copy / Object reconstruction
- **Resultado:** ✅ TS records: `_mapRecord` identity fallback, copy-on-write, Object.assign reconstruction refactor.

### AUD-034: dedupe transacción IDB en helper único (WASM)
- **Fecha:** 2026-08-16
- **Resultado:** ✅ `vantadb-wasm/src/idb.rs`: 4 bloques IDBTransaction (write/del × lock/no-lock) → helper `runWriteTx` + 2 call sites; diff +15/−32. Lock y notify preservados; API `IdbStorage::write_file`/`delete_file` intacta. Commit `b255f982`. (ver docs/progreso/README.md)

### AUD-043: collect_all_deduped — dedup u128 node-ids (WASM)
- **Fecha:** 2026-08-16
- **Resultado:** ✅ `vantadb-wasm/src/lib.rs:556`: dedup `HashSet<(String,String)>` → `HashSet<u128>` por `record.node_id` (XxHash3_128 sobre `namespace\0key`); cero alocación por record. + test `test_collect_all_deduped_no_duplicates`. Commit `9dcbff5a`. (ver docs/progreso/README.md)

### FND-18: Time-to-first-query <5 min en SDKs Python/TS
- **Fecha:** 2026-08-16
- **Resultado:** ✅ quickstarts corregidos (metadata shape discriminated union en TS, PyPI primario + `hit.key`/`hit.score` en Python, QUICKSTART.md desactualizado) — medido: Python 6.2s, TS 1.6s (objetivo <5 min). Commit `ae39516e`. (ver docs/progreso/README.md)

### CODE-047: Tests con catch vacío
- **Resultado:** ✅ aserciones de errores en catches, 11 archivos.

### CODE-086: TS async sin async real
- **Resultado:** ✅ 13 métodos `async` real (borran manual wrapper).

### CODE-089: storage_path sin efecto en WASM
- **Resultado:** ✅ Constructor `QdrantWasmConfig` acepta storage_path (memoria solo; warning en consola).

### CODE-090: insertNode BigInt overflow
- **Resultado:** ✅ u128 JSON string parsing BigInt; `Number` fallback para valores u64 siendo u128 json string.

### CODE-091: hit.distance etiquetado score
- **Resultado:** ✅ tipo separado metadata vs metrics del vector; `score` = `1 - distance`.

### PERF-08: WASM serialización zero-copy en hot path
- **Resultado:** ✅ `memory_record_to_js` emite `record.vector` como `Float32Array` zero-copy (`js_sys`) en vez de `serde_wasm_bindgen::to_value` por elemento; cierra P2-7. Host compat: `vantadb-ts/src/types.ts` `vector?: Float32Array | number[]`. Persist-delta (H3-SER-001) diferido (requiere dirty-tracking en core).

### PERF-04: TS stream-based Large Object handling
- **Resultado:** ✅ Streams en `put`/`get` (WASM BinaryLargeObject 2 stages).

### VFY-002 (bloqueado WASM section): CDV->VitaBuf
- El objeto CODeXCE/Jobs quedó deprecado; fix definitivo del vm debía venir de CDV->VitaBuf (crít. AHORA, sin completar en sesión). Ver `historial/autopsias-2026-06-19.md`.

### VFY-004 (WASM near)
- FASE W2: 1316 tests, 40 para WASM Build. FASE W3/4: TS SDK axioms WASM. Ver BACKLOG_HISTORY.

---

## Adapters IA

### DRV-102: LangChain add_texts GIL release
- **Fecha:** 2026-07-14
- **Resultado:** ✅ `detach()` + threading in add_texts/similarity_search_by_vector/delete. `cargo check -p vantadb-langchain` ✅. Commit `3cc6888`.

### DRV-103: LangChain metadata no-string fallthrough
- **Fecha:** 2026-07-14
- **Resultado:** ✅ Fallthrough String→bool→i64→f64. Commit `b83f0f9`.

### DRV-086: CrewAI metadata no-string fallthrough
- **Fecha:** 2026-07-14
- **Resultado:** ✅ to_string() fallback. Commit `b83f0f9`.

### DRV-092: DSPy metadata no-string fallthrough
- **Fecha:** 2026-07-14
- **Resultado:** ✅. Commit `b83f0f9`.

### DRV-110: LlamaIndex metadata no-string fallthrough
- **Fecha:** 2026-07-14
- **Resultado:** ✅. Commit `b83f0f9`.

### DRV-099: (pendiente detalle en fuente bitacora §BINDINGS)
- **Estado:** Verificado en bitacora línea ~1745.

### DRV-109: LlamaIndex GIL release — no-op
- **Resultado:** ✅ CR.: Already correct since start; commit no-op. → `historial/no-ops.md`.

### NUEVO-01 / NUEVO-02 / NUEVO-03
- NUEVO-01: LangChain vectorstore (mem_as_base64). ✅
- NUEVO-02: WebLoader LLMs (MCP) → RS de Vantada. ✅
- NUEVO-03: OpenAI handler con memoria. ✅

### NUEVO-13/14: OpenAI embedding API
- **Resultado:** ✅ Async engine + batch embeddings, `VantaDB.search(..., embeddings=...)` + `summarize_context`.

---

## MCP

### MOD-07: Notifications JSON-RPC sin `id` aceptadas (handshake clientes estrictos)
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/plans/2026-08-23-backlog-triage.md` (Wave 1) · Backlog · mcp.md H1
- **Resultado:** ✅ `RpcRequest.id` requerido rechazaba notificaciones (`notifications/initialized`, `notifications/cancelled`) con -32700 espurio. Fix: `#[serde(default, deserialize_with = keep_explicit_null)] id: Option<Value>` + routing en `serve_lines` — notification → log + drop SIN respuesta (JSON-RPC 2.0 §4.1); `"id": null` explícito sigue siendo request. Loop extraído a `serve_lines<R, W>` genérica + `write_json` genérica para testabilidad con duplex pipes. TDD: RED reprodujo el bug exacto; 4 tests nuevos de wire-format. Tests MCP 37/37 ✅; fmt+clippy `-D warnings` ✅. Commit `4cb3abec`. (ver `.opencode/skills/campaign-executor/tasks/MOD-07.md`)

### P22-MCP: Certificación del MCP server vs skill `vantadb-mcp` (14 tareas, 2026-08-17)
- **Resultado:** ✅ Bloque 1 (código): MCP-01 text search fix (`ensure_indexes_current` en arranque `run_stdio_server`), MCP-02 `distance_metric` per-request propagado, MCP-03 `distance` = 1−cosine, MCP-04 validación `DimensionMismatch` (isError content). Bloques 2-5 (docs): skill sync — IQL Syntax, Response Envelope, Error Channels, Behavior Notes, dead refs, contradicciones. **Cierre 2026-08-17:** MCP-15 stack overflow resuelto (`PrefetchGuard` thread_local+RAII single-level — root cause recursión infinita get→prefetch_related→get en pares co-accesados cache-miss; GATE vanta-audit aprobado 0 C/H/M; commit `cd8dd129`) y T15 explain shape (doc alineada a realidad + test `test_mcp_search_memory_explain_shape`; commit `a7c0a00c`). Commits `d8f720f9`, `d24fb663`, `04840079`. Tests MCP 34/34 ✅; test-busqueda.py 20/20 ✅; hash SAME skills↔.opencode/skills. (ver docs/progreso/README.md)

### MCP-01: MCP server crate (vantadb-mcp)
- **Fecha:** 2026-07-25
- **Resultado:** ✅ crate nueva `vantadb-mcp`, tools `insert`, `search`, `list_namespaces`, `create_namespace`, `delete_namespace`, `compact` — 6 tools MCP. MCP se comunica vía JSON-RPC. En `crates/vantadb-mcp/`. Test `tools_all_reachable`. Solo Linux/macOS.

### MCP-02: MCP async I/O parallel
- **Fecha:** 2026-07-11
- **Resultado:** ✅ ThreadPools 4 workers: `query_async`, `compact_async`, `import_async`, los retorna JoinHandle. 4 tests.

### AUD-032: Split del monolito vantadb-mcp en 12 módulos
- **Fecha:** 2026-08-14
- **Resultado:** ✅ `src/lib.rs` → facade (`#![warn(missing_docs)]`, 8 mods, 10 `pub use`) + 12 módulos (`config,axioms,error,protocol,metrics,validation,server` + `handlers/{initialize,resources,prompts,tools}`); slicing 1:1, internals `pub(crate)`; tests migrados; `version_coherence.rs:97` → `src/handlers/initialize.rs`. Review P2-01 approve. Commit `1099bfe4`. (ver docs/progreso/README.md)

### AUD-045: MCP `memory_put` acepta `expires_at_ms` + `sparse_vector` (2026-08-18)
- **Resultado:** ✅ schema `memory_put` + handler parsean TTL (absoluto → relativo con `saturating_sub(now)`) y sparse (formato real `{"0": 0.5}` vía `parse_sparse_vector` en validation.rs). Backward compat (campos opcionales, `required` intacto); inválidos → `-32602`. mcp_tests 37/37. Commit `27f3770e`. (ver docs/progreso/README.md)

### AUD-046: MCP `memory_put` valida dims antes de insertar (2026-08-18)
- **Resultado:** ✅ reusa `index_vector_dim` + `DimensionMismatch` de search — dim ≠ esperada → error JSON-RPC ANTES de insertar (nunca corrupción silenciosa HNSW); primer put define dim. mcp_tests 38/38. Commit `4936418a`. (ver docs/progreso/README.md)

### AUD-048: semántica filtros unificada CLI↔MCP (2026-08-18)
- **Resultado:** ✅ ambos canales aceptan plano (`{"field": v}` = `$eq`) y operadores `$eq/$neq/$gt/$gte/$lt/$lte` (normalizados en parseo: `parse_filter_ops` MCP, `parse_filter_json` CLI); `memory_list` rangos OK, `search_memory` fold `$eq`→plano. Zero breaking. cli_tests 79/79 + mcp_tests 40/40 + review APPROVE. Commits `8dbe07a8`, `e6f43f3b`. (ver docs/progreso/README.md)

### AUD-050: `inject_context` error claro thread_id (2026-08-18)
- **Resultado:** ✅ distingue `Missing 'thread_id'` (ausente/null) vs `'thread_id' must be a numeric id (integer), got string` (tipo inválido — el error anterior decía "Missing" con el campo presente). mcp_tests 41/41. Commit (wave 4). (ver docs/progreso/README.md)

### MEM-21: F4 Tools MCP scene_read/list/query — gateway handlers (2026-08-20)
- **Resultado:** ✅ `vanta-memory/src/gateway/knowledge_handlers.rs` (nuevo): capa de entrada tipada serde para `scene_read`/`scene_list`/`scene_query` sobre el store de escenas (MEM-12/MEM-15); server MCP la expone después. Soft-delete respetado (read→NotFound, list/query excluidos); query LLM-free (`overlap_score`, techo documentado); `KnowledgeError` non_exhaustive. 10 tests D19; suite 361 ✅. Commit `31e676b1`. (ver docs/progreso/README.md)

### MCP-16 (edge? — ver fuente)
- **Estado:** Pendiente verificar.

---

## Compatibilidad API & docs de bindings

### CODE-074: Python long compatibility (u128 → int Python native)
- **Resultado:** ✅ Compatibilidad Python: `u128` server-side como `str` con tipo dual; Python usa int nativo.

### DRV-016: (relacionado bindings — ver detalle) 
- **Estado:** Ver fuente README §bindings.

> **Cruce:** cada binding público debe mantener el contrato definido en `docs/api/`; los cambios de firma se auditan en `auditoria/seguridad.md` FFI y en `docs/avance/operaciones.md` (API contract sync).
### ERR-026 (MCP parse_metadata), ERR-033 (MCP list limit=0) — migrados 2026-08-12 (ver docs/progreso/README.md)
### COV-001 (Python AsyncVantaDB async smoke, 3 tests) + COV-002 (vantadb-ts coverage vía c8) — migrados 2026-08-12 (ver docs/progreso/README.md)

### FND-04: Zero-copy Arrow en bindings — DIFERIDO — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ DIFERIDO con ADR-021 + señal de reapertura explícita en `docs/research/FND-04-arrow-zero-copy.md` (umbrales documentados). Commit `95a67fd3`.

### FND-05: SDK idiomático (no wrapper 1:1 de Rust) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ análisis en `docs/research/FND-05-sdk-idiomatico.md` (gaps PY-*/TS-*) + prototipos `with VantaDB(path) as db` (Python) y `await using db` (TS, ejemplos en `docs/examples/`). Sin rewrite; async nativo NO (cubre FND-04). Commit `14183fc4`.

### FND-06: Regla de boundaries core ↔ bindings (Ports & Adapters) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ regla R-8 core-bindings (lógica de negocio NUNCA en PyO3/WASM/server) en `.opencode/rules/api-contract.md` + TODO(core) + drift ERR-028 documentado. Commit `bea0f513`.

### VS-CORE-04: Exportar selección/query con filtro — migrado 2026-08-19 (ver docs/progreso/README.md)
- **Resultado:** ✅ `export_namespace_filtered` WASM + `exportNamespace(path, namespace, filter?)` TS + comando Tauri `vanta_export_namespace`. Aditivo sobre `export_namespace` (None = export completo). Commits `a62088b7`/`7429f81a`.

### VS-CORE-05: Batch delete con filtro — migrado 2026-08-19 (ver docs/progreso/README.md)
- **Resultado:** ✅ `delete_by_filter` expuesto WASM → TS → bridge Tauri (`vanta_delete_by_filter(namespace, filter) -> u64`), protección anti borrado total (filtro vacío rechazado) propagada a todos los bindings. Commits `15172349`/`39a6369c`.

### VS-CORE-06: IQL bridge + autocompletado — migrado 2026-08-19 (ver docs/progreso/README.md)
- **Resultado:** ✅ comando Tauri `vanta_query` + `vanta_iql_autocomplete` (shim core-side sobre `parse_statement`); wrapper `queryIql()` en `vanta.ts`. Commit `ebf9acc1`.
