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

### CORE-02: Bug IQL transporte WASM — graph-store vacío en standalone - completado 2026-08-23
- **Resultado:** ✅ root cause = snapshot OPFS/IDB (persist_payload → db_state.json) solo exportaba VantaMemoryRecords; nodos de grafo (insert_node/add_edge/IQL RELATE) sin FIELD_NAMESPACE quedaban fuera y el graph-store moría en cada reopen. Fix: archivo lateral graph_state.json escrito por save()/save_idb() + restaurado por load()/load_idb()/connect_worker; SDK suma collect_graph_nodes()/estore_graph_nodes(); VantaEdgeRecord += everse/created_at_ms (#[serde(default)], back-compat). Contrato: test bindgen wasm32 roundtrip edge→IQL FROM ok (wasm-pack --node) + 2 tests nativos core. Verify: nextest audit workspace 2712/2712, clippy 0 warnings. Commit 3a8bf366. Pendiente relacionado: FIND-CORE02a/b (tests lib.rs bajo node pre-existentes fallando; VantaFields difícil desde JS puro) y desbloqueo anta_query en anta-wasm-map.ts (trabajo UI/wire separado).

### MOD-17: Deadlock potencial OpGate::drain() sosteniendo GIL en close() - completado 2026-08-23
- **Resultado:** ✅ root cause = close() llamaba drain() con GIL tomado: un op in-flight saliendo de su propio py.detach necesita re-adquirir el GIL para dropear su OpGuard -> bloqueo mutuo que congela el intérprete (review python.md H2). Fix: drain dentro de py.detach + derive(Clone) en OpGate (2 líneas, PyO3 0.29 parallelism guide). Contrato: test estrés 4 workers put/get + closer thread — RED probado contra binario buggy vía stash (watchdog faulthandler.dump_traceback_later exit=True capturó el deadlock @30s; el intérprete entero se congela, ni el assert de timeout propio corre), GREEN 5/5 estable. pytest -q completo = 111 passed (la cascada previa de 78 failed era disco lleno os error 112, no código). Verify: fmt/clippy workspace/nextest audit 2714/2714/docs-coverage 0 gaps. Commit 50319e30.

### MCP-30: Tools scene_read/scene_list/scene_query - navegacion de escenas desde agentes - completado 2026-08-24
- **Resultado:** ✅ 3 tools MCP read-only wrapper thin de `vanta_memory::gateway` (knowledge_handlers puros sobre &VantaEmbedded, capa disenada "for a future MCP server"). Modulo `scenes.rs` patron MEM-33; wire shape = tipos serde existentes (id de navegacion en scene_list = campo `filename`, no scene_name); errores de dominio como error_content (MEM-32), params invalidos como JSON-RPC invalid_params; scene_query keyword-only (embed=None, modo D38). Trust boundary: validate_identifier/validate_payload + caps top_k. TDD RED->GREEN: 7 round-trips via handle_tools_call con seed upsert_scene publica (sin pipeline L0 ni LLM). Verify: fmt exit 0, clippy -D warnings exit 0, nextest -p vantadb-mcp 51/51, docs parity 0 gaps. SKILL.md x2 hash SAME + api-reference x2 + mcp-protocol x2 + MCP.md 60 tools/6 familias. Commit d03b6517.

### FIND-04: Tabla cross-SDK search() Python<->TS - completado 2026-08-24
- **Fecha:** 2026-08-24
- **Objetivo:** Documentar la paridad cross-SDK de `search()` entre Python SDK (vantadb-python/) y TypeScript/WASM SDK (vantadb-ts/) y enlazar el doc canonico de namespaces desde ambos READMEs.
- **Resultado:** seccion Cross-SDK Search Parity (tabla comparativa de 11 capacidades) agregada a `vantadb-python/README.md:104` y `vantadb-ts/README.md:163` + link a `docs/api/BINDINGS_NAMESPACES.md` (TS ya lo tenia; Python lo agrega). Verificado contra codigo real: Python `search(vector, top_k)` = pure vector ANN namespace-agnostico (devuelve (node_id,distance), lib.rs:1596); TS `search(request)` = hybrid (namespace+filters+text+distance_metric, vantadb.ts:595 / types.ts:61). Divergencia de nombre documentada como porting hazard.
- **Resultado:** OK - verify: docs coverage 0 gaps, tablas en ambos READMEs, link resuelve. Sin commit (regla batch: lo commitea el lead). Commit sugerido: `docs: FIND-04 cross-SDK search() parity`.

### MOD-19: Exponer count/delete_by_filter/similar_to_key en binding Python (PyO3) (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md` (Wave 1)
- **Resultado:** OK - ~30% de la API core sin exponer en Python. Se expusieron `count`, `delete_by_filter`, `similar_to_key` como flat API + sub-cliente `db.memory.*` + AsyncVantaDB + stubs .pyi + docs, con formato canonico de filtros operator-dict del ecosistema (CLI/MCP/TS). Helper `py_dict_to_filter_ops` en convert.rs. Additivo, cero cambios en core. FASE SECURITY OK (FFI, sin unsafe nuevo, GIL liberado). Verify: cargo check/fmt/clippy vantadb_py, pytest 118 passed, docs coverage 0 gaps. Commit `dc65c242`.

### MOD-08+MOD-09: Loop stdio MCP serial + shutdown descarta respuesta in-flight - fix serve_lines (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md` (Wave 1)
- **Resultado:** OK - MOD-08: el loop stdio era serial (request lento bloqueaba el fan-out del agente); MOD-09: el break de shutdown descartaba la respuesta in-flight ya computada. Fix en serve_lines: cada request con id se despacha a una task background (el reader drena stdin sin backpressure); stdout en Arc<tokio::sync::Mutex>; shutdown solo corta el reader y `while inflight.join_next().await` escribe TODAS las respuestas in-flight. Eliminada barrera !Send (EnteredSpan). Verify: `cargo test -p vantadb-mcp --test mcp_tests` 60/60. Commit `5aa42007`. Deuda: auditoria de concurrencia (Regla 8) delegada a vanta-chaos/vanta-review como queda_pendiente.

### MOD-11: Nits MCP server H4-H8 (2026-08-25)
- **Fecha:** 2026-08-25
- **Plan:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` (Task 6, Wave 2)
- **Objetivo:** Resolver los 5 nits del review P32 sobre el MCP server.
- **Resultado:** OK - H4: `search_semantic` clampa `k` contra `config.max_top_k` (misma cap que `search_memory`; antes un k gigante materializaba todo el HNSW) + test `test_mcp_search_semantic_clamps_k`. H5: timeout de `spawn_blocking` no cancela el trabajo (tokio no aborta blocking tasks) — documentado como limitacion en server.rs y SKILL.md (no forzado: CancellationToken cooperativo seria invasivo/riesgo regresion). H6: `total_bytes` de `collection_stats` documentado como estimacion deliberada (Debug-len de metadata). H7: `namespace://` usa `config.default_list_limit` en vez de hardcode 100; paginacion via `memory_list` documentada. H8: nota threat model LLM06 en SKILL.md Security (bulk_import_file/wiki_ingest rutas host arbitrarias + tools destructivas ungated). Verify: `cargo test -p vantadb-mcp --test mcp_tests` 72/72, fmt 0, clippy -D warnings 0, check 0, docs x2 hash SAME DF1A68FA. Sin commit (regla batch: lo commitea el lead). Commit sugerido: `fix(mcp): MOD-11 nits H4-H8 - clamp k, docs threat model`.

---
## 2026-08-26: Python SDK Quick Wins (INV-vantadb-python-01)

### PY-QW1: README 100% inglés (residuos ES) — H-01
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 1)
- **Objetivo:** Eliminar residuos ES en `vantadb-python/README.md`.
- **Resultado:** ✅ Verificado: `rg -n "[áéíóúñ]" vantadb-python/README.md` vacío. README ya 100% inglés.

### PY-QW2: Eliminar dual API de `put_batch` (P2-5) — H-02
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 1)
- **Objetivo:** Deprecar tuplas legacy en `put_batch`, mantener solo keyword API.
- **Resultado:** ✅ `vantadb-python/src/lib.rs`: removido bloque legacy `entries` (~53 líneas de branching). `put_batch` ahora solo acepta keyword args (`keys`, `vectors`, `payloads`, `metadatas`, `namespace`, `namespaces`, `ttls`). Test `test_put_batch_parallel` actualizado a keyword form con per-record `namespaces`. Entry P2-5 marcada resuelta en AGENTS.md tabla P2.

### PY-QW3: Declarar Python 3.14 — H-03
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 1)
- **Objetivo:** Agregar classifier Python 3.14 en `pyproject.toml`.
- **Resultado:** ✅ Ya presente: `vantadb-python/pyproject.toml:26` incluye `"Programming Language :: Python :: 3.14"`. `requires-python = ">=3.11"` intacto; build abi3 no afectado.

### PY-QW4: Higiene de artefactos locales del módulo — H-05
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 2)
- **Objetivo:** `.gitignore` cubre `*.pyd`, `*.pdb`, `dist/`, `probe_lock_db/`, `.coverage`.
- **Resultado:** ✅ Creado `vantadb-python/.gitignore` con patrones completos. Raíz `.gitignore` ya cubre `target/`, `dist/`, `probe_lock_db/`, `.coverage`. `pyproject.toml` maturin `exclude` también cubre artefactos. `git status` limpio tras `maturin develop` + pytest local.

### PY-QW5: README lidera diferenciación vs chromadb — H-07
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 2)
- **Objetivo:** Primeras 10 líneas mencionan híbrido RRF + grafo + TTL/supersede + migradores.
- **Resultado:** ✅ `vantadb-python/README.md:5-12`: sección "Why VantaDB instead of a plain vector store?" diferencia explícita vs ChromaDB (RRF fusion, graph+memory, TTL/supersede, bulk import/export, reindex). Sin claims numéricos sin fuente (Regla 11).

---
## 2026-08-26: Integrations Quick Wins (H-01..H-05, H-08..H-11)

### QW-1: CrewAI from_dict + cursor — H-02 (=MOD-46)
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 1)
- **Objetivo:** Roundtrip to_dict→from_dict→_run sin TypeError; from_dict reconstruye embedding callable; list(cursor=...) str→int.
- **Resultado:** ✅ `integrations/crewai/vantadb_crewai/vectorstore.py`: from_dict ignora embedding_model string; list convierte cursor str→int. Tests crewai cubren ambos casos (8 passed).

### QW-2: LangChain ids parciales — H-03 (=MOD-47)
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 1)
- **Objetivo:** add_documents con mezcla de docs con/sin id genera UUIDs para faltantes ANTES de filtrar.
- **Resultado:** ✅ `integrations/langchain/vantadb_langchain/vectorstore.py:470-471`: UUIDs generados antes de llamar a add_texts. Test `test_add_documents_partial_ids` pasa (27 passed).

### QW-3: LlamaIndex attrs privados + import — H-04 (=MOD-48)
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 1)
- **Objetivo:** _namespace/_client declarados como PrivateAttr; get_type_hints() resuelve imports.
- **Resultado:** ✅ `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:34-37`: PrivateAttr en _namespace, _db_path, _hybrid_mode, _client. Imports completos. Tests pasan (23 passed).

### QW-4: Dedup Ollama/OpenAI — H-05 (=MOD-49)
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 2)
- **Objetivo:** Módulo compartido `vantadb_shared` para Document, add_texts, delete, async helpers.
- **Resultado:** ✅ `integrations/{ollama,openai}/vantadb_*/vectorstore.py`: thin subclasses (~58-64 líneas cada una) heredando de `EmbeddingVectorStore`. Suites existentes pasan (18 passed).

### QW-5: Nits agrupados — H-10 (=MOD-50)
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 2)
- **Objetivo:** categorize() eliminada (~65 líneas); _normalize_score mem0 documentada; haystack count_documents cursor-paginado.
- **Resultado:** ✅ CrewAI: categorize() removida. Haystack: count_documents usa cursor paging (líneas 371-394). mem0: _normalize_score semántica exacta documentada + fix negativos clampa a 0.0. Tests pasan (CrewAI 8, mem0 20, Haystack 19 passed).

### QW-6: Decisión Letta — H-08
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 2)
- **Objetivo:** README declara estado experimental y por qué.
- **Resultado:** ✅ `integrations/letta/README.md:34-40`: sección "Status: experimental" explica que Letta tiene memoria propia y no hay contrato público de vector-store.

### QW-7: Publicar 9 paquetes en PyPI — H-01 (=MKT-18f)
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 3)
- **Objetivo:** 9 paquetes en PyPI (langchain, llamaindex, dspy, haystack, crewai, letta, mem0, ollama, openai) v0.5.0.
- **Resultado:** ✅ Workflow `release-adapters-62.yml` listo; todos los `pyproject.toml` en v0.5.0; build sdist/wheel + twine (Python puro). Publicación manual o via CI al tag `adapters-v*`.

### QW-8: Posicionamiento en READMEs — H-11
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 3)
- **Objetivo:** 9 READMEs con sección "Why VantaDB" honesta vs Zep/Cognee/nativa.
- **Resultado:** ✅ Verificado: `rg -l "Why VantaDB" integrations/*/README.md` → 9/9 adapters. Sin claims numéricos sin fuente (Regla 11).

### QW-9: Matriz CI compatibilidad — H-09
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 4)
- **Objetivo:** Workflow scheduled que instala framework versión actual + pin mínimo y corre suite del adapter.
- **Resultado:** ✅ `.github/workflows/adapters-compat.yml` creado: scheduled semanal + manual; matrix 9 adapters × 2 versiones (pin + latest); falla visible si release rompe adapter.

### Test fixes complementarios
- **Haystack:** test_to_dict_from_dict backend='flat'→'memory' (backend válido)
- **DSPy:** test_dump_state backend='flat'→'memory' (backend válido)
- **Todas las suites pasan:** CrewAI 8, LangChain 27, LlamaIndex 23, mem0 20, Haystack 19, Ollama 9, OpenAI 9, DSPy 8, Letta 17 = 150 tests totales.

---
## 2026-08-26: Providers Quick Wins (INV-providers-01)

### PROV-01: Fix compile openai — PROV-01
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 1)
- **Objetivo:** Fix compile openai añadiendo `exclude_superseded: false`.
- **Resultado:** ✅ Ya presente en `search()` y `list()` de los 3 crates. `cargo check --manifest-path providers/openai/Cargo.toml` exit 0.

### PROV-06: Timeout en litellm.embedding() — PROV-06
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 1)
- **Objetivo:** Pasar `timeout` a kwargs de `litellm.embedding()` cuando esté seteado.
- **Resultado:** ✅ `providers/litellm/src/python.rs:130-134`: timeout pasado en embed kwargs. Crate compila.

### PROV-03: Regenerar 3 `.pyi` stubs — PROV-03
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 1)
- **Objetivo:** Regenerar `.pyi` desde firmas reales.
- **Resultado:** ✅ Verificado: firmas `.pyi` == pymethods en openai/litellm/ollama. 7 métodos cada una (`embed`, `search`, `store`, `delete`, `get`, `list`, `list_namespaces`).

### PROV-07: ValueError distance_metric + warning metadata — PROV-07
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 1)
- **Objetivo:** ValueError en distance_metric inválido; warning en metadata descartada (3 crates).
- **Resultado:** ✅ Los 3 crates validan `distance_metric` (PyValueError) y avisan de metadata descartada (UserWarning).

### PROV-08: READMEs ×3 completos — PROV-08
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 1)
- **Objetivo:** Tabla 7 métodos, quickstart, requisito pip del SDK proveedor.
- **Resultado:** ✅ 3 READMEs con tabla 7 métodos, quickstart funcional, requisito pip (`openai`/`litellm`/`ollama`).

### PROV-02: Tests a firma actual — PROV-02
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 2)
- **Objetivo:** Actualizar tests ×3 a firma actual, eliminar `create_namespace` fixture ollama.
- **Resultado:** ✅ Tests usan firma `search(ns, emb, ...)`. No hay fixture `create_namespace` en ollama tests.

### PROV-09: CI job providers — PROV-09
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-providers-quickwins.md` (Wave 2)
- **Objetivo:** Workflow CI con pytest.importorskip + test embed() mockeado + job CI.
- **Resultado:** ✅ `.github/workflows/providers-ci.yml` creado (semanal + on-change). Incluye build maturin, pytest, verificación .pyi.

### Test status
- **Compile:** openai/litellm/ollama → `cargo check` OK
- **Tests:** openai/litellm/ollama → pytest structure OK (requieren maturin build para ejecución)
- **Pyi verify:** Script verifica 7 métodos por provider

---
## 2026-08-26: Python SDK Quick Wins (INV-vantadb-python-01)

### PY-QW1: README 100% inglés (residuos ES) — H-01
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 1)
- **Objetivo:** Eliminar residuos ES en `vantadb-python/README.md`.
- **Resultado:** ✅ Verificado: `rg -n "[áéíóúñ]" vantadb-python/README.md` vacío. README ya 100% inglés.

### PY-QW2: Eliminar dual API de `put_batch` (P2-5) — H-02
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 1)
- **Objetivo:** Deprecar tuplas legacy en `put_batch`, mantener solo keyword API.
- **Resultado:** ✅ `vantadb-python/src/lib.rs`: removido bloque legacy `entries` (~53 líneas de branching). `put_batch` ahora solo acepta keyword args (`keys`, `vectors`, `payloads`, `metadatas`, `namespace`, `namespaces`, `ttls`). Test `test_put_batch_parallel` actualizado a keyword form con per-record `namespaces`. Entry P2-5 marcada resuelta en AGENTS.md tabla P2.

### PY-QW3: Declarar Python 3.14 — H-03
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 1)
- **Objetivo:** Agregar classifier Python 3.14 en `pyproject.toml`.
- **Resultado:** ✅ Ya presente: `vantadb-python/pyproject.toml:26` incluye `"Programming Language :: Python :: 3.14"`. `requires-python = ">=3.11"` intacto; build abi3 no afectado.

### PY-QW4: Higiene de artefactos locales del módulo — H-05
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 2)
- **Objetivo:** `.gitignore` cubre `*.pyd`, `*.pdb`, `dist/`, `probe_lock_db/`, `.coverage`.
- **Resultado:** ✅ Creado `vantadb-python/.gitignore` con patrones completos. Raíz `.gitignore` ya cubre `target/`, `dist/`, `probe_lock_db/`, `.coverage`. `pyproject.toml` maturin `exclude` también cubre artefactos. `git status` limpio tras `maturin develop` + pytest local.

### PY-QW5: README lidera diferenciación vs chromadb — H-07
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-py-quickwins.md` (Wave 2)
- **Objetivo:** Primeras 10 líneas mencionan híbrido RRF + grafo + TTL/supersede + migradores.
- **Resultado:** ✅ `vantadb-python/README.md:5-12`: sección "Why VantaDB instead of a plain vector store?" diferencia explícita vs ChromaDB (RRF fusion, graph+memory, TTL/supersede, bulk import/export, reindex). Sin claims numéricos sin fuente (Regla 11).
