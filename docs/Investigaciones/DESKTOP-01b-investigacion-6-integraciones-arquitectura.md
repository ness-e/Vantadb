# DESKTOP-01b — Investigación de las 6 integraciones + Arquitectura Multi-Connection

- **Tipo:** Investigación / Arquitectura (sin código)
- **Fecha:** 2026-08-04
- **Fuente:** Extensión de `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` + 5 sub-agentes (4 de investigación de módulos + 1 definidor de arquitectura)
- **Decisión que informa:** Cómo construir la app de escritorio Tauri de VantaDB conectable a **cualquiera de las 6 integraciones** (crate nativa, `vantadb-server` HTTP, `vantadb-mcp` stdio, `vantadb-node` napi, `vantadb-python` PyO3, `vantadb-ts`/`vantadb-wasm` webview), individualmente o varias simultáneas.
- **Estado:** ✅ Investigación + arquitectura definida. Tareas en `docs/Backlog.md` → Phase 12 DESKTOP (DESKTOP-02..27).
- **Contenido:** este documento preserva íntegros los reportes de los sub-agentes (no solo el resumen del backlog).

---

## 1. Resumen ejecutivo

- **Las 6 integraciones no son alternativas del mismo tipo**: son vías de transporte distintas sobre el **mismo motor** `VantaEmbedded` (crate `vantadb`). server = red HTTP; mcp = subproceso stdio; node = FFI napi-rs; python = PyO3 in-process; ts/wasm = webview sandbox.
- **Vía nativa (crate `vantadb` directa en `src-tauri/`) es la óptima**: sin capa intermedia, WAL/fsync real, superficie API completa. Es la recomendación de DESKTOP-01 y de los 4 agentes.
- **Las otras 5 son conectables por el usuario** para compatibilidad universal (datos que ya existen en un server, Claude Desktop vía MCP, addon Node, runtime Python, demo web).
- **Arquitectura:** trait `VantaConnection` (async, object-safe) + un adaptador por vía + `ConnectionManager` (multi-conexión, "un escritor por path de DB") + `VantaError` unificado.
- **Hallazgos críticos para el equipo:** 3 bugs/gotchas reales del código y 3 stale-docs detectados (ver §6).

---

## 2. Reporte sub-agente — `vantadb-server/` (HTTP REST)

### 2.1 Arquitectura real
- `vantadb-server/` es una cáscara fina. El binario solo hace dispatch; **todo el servidor HTTP vive en el crate core `vantadb`, módulo `cli_server`**.
  - `vantadb-server/src/main.rs:26-57` — si arg `--mcp` → MCP stdio (`vantadb_mcp::run_stdio_server`); si no → `vantadb::cli_server::run(config)`.
  - `vantadb-server/src/server.rs:1-4` y `src/middleware.rs:1` — re-exports de `vantadb::cli_server`.
  - `vantadb-server/Cargo.toml:10` — `vantadb = { path = "..", features = ["cli", "server"] }`. El feature `server` del core activa axum/tower-governor/tower-http.
  - `src/api/mod.rs:1-4` — stub vacío.
- **Protocolo:** HTTP REST (axum 0.8) + JSON. **No hay gRPC, WebSocket ni SSE** (grep `DefaultBodyLimit|WebSocket|Sse|EventSource|text/event-stream` → 0). El streaming solo existe vía MCP stdio.

### 2.2 Endpoints (los únicos 3, rutas en `src/cli_server.rs:126-131`)
| Método | Path | Auth | Descripción |
|--------|------|------|-------------|
| GET | `/health` | Ninguna (exenta, `cli_server.rs:261-263`) | Liveness `{"success":true,"data":"OK"}` |
| POST | `/api/v2/query` | Bearer (si api_key configurada) | Ejecuta sentencia IQL |
| GET | `/metrics` | Bearer | Prometheus/OpenMetrics `text/plain; version=0.0.4` |

- Versión API **v2 hardcodeada** en la ruta (`cli_server.rs:129`). `docs/api/openapi.yaml:4` declara 0.4.0; workspace = 0.5.0.

### 2.3 Puerto / host / env vars (`src/config.rs:406-423`)
- Host: `VANTADB_HOST` → `HOST` → `127.0.0.1` (loopback por defecto).
- Puerto: `VANTADB_PORT` → `8080`. Storage: `VANTADB_STORAGE_PATH` → `vantadb_data`.
- Otras env: `VANTADB_API_KEY` (sin default = modo dev sin auth), `VANTADB_REQUIRE_AUTH` (false), `VANTADB_RATE_LIMIT_RPM` (100), `VANTADB_MAX_CONNECTIONS` (núcleos×2), `VANTADB_POOL_ACQUIRE_TIMEOUT_MS` (5000), `VANTADB_CIRCUIT_BREAKER_*` (5/30), `VANTADB_TLS_CERT/KEY`, `VANTADB_LOG_FORMAT`, `VANTADB_MAX_BLOCKING_THREADS`, `VANTA_BACKEND` (fjall|rocksdb|memory).
- **CLI:** `vanta-cli server --http [--mcp] [-p|--port] [--host] [--require-auth] [-d|--db]` (requiere feature `server`). El binario `vantadb-server` **no tiene flags salvo `--mcp`**; config 100% env.

### 2.4 Formato request/response
- Request: `{ "query": "<IQL>" }`, Content-Type `application/json` obligatorio. Body malformado → 400.
- Response envelope (`QueryResponse`, `cli_server.rs:58-71`): `{ success, data, node_id?, nodes? }`. `NodeDTO`: id:u128, semantic_cluster, relational:BTreeMap, hits, confidence_score.
- **⚠️ Gotcha clave:** HTTP **200 incluso cuando la query falla** — el error viaja en el body (`success:false`). El cliente debe tratar `success:false` como error de dominio.
- Status codes reales: 400 (JSON inválido), 401 (token mal/faltante, con `hint`), 403 (RBAC), 429 (rate limit), 503 + `Retry-After` (pool saturado / breaker abierto), 500 (panic).
- Sin CORS headers.

### 2.5 Operaciones (IQL — 6 tipos)
`FROM ... WHERE ... FETCH ... RANK BY`, vector/hybrid (`~` + min score), `INSERT NODE#id TYPE ...`, `UPDATE`, `DELETE NODE#id`, `RELATE NODE#a --"label"--> NODE#b` y `INSERT MESSAGE ... TO THREAD#id`.
`ExecutionResult`: `Read(Vec<UnifiedNode>) | Write{...} | StaleContext(u128)`.

### 2.6 Ejemplos reales (de `docs/api/HTTP_API.md:38-108` y tests e2e)
```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/metrics
curl -X POST http://127.0.0.1:8080/api/v2/query -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{"query": "(memory:get \"agent/main\" \"memory-1\")"}'
curl -X POST http://127.0.0.1:8080/api/v2/query -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{"query": "FROM memory WHERE text ~ \"neural network\", min = 0.75 FETCH text, score RANK BY score DESC"}'
```
Tests E2E sobre sockets: `vantadb-server/tests/e2e.rs` (insert 101, read, delete, auth 401/200, persistencia). TLS end-to-end: `tests/server.rs:405-495`.

### 2.7 Seguridad
- Bearer token comparado en tiempo constante (`subtle::ConstantTimeEq`). Sin key → modo dev (pasa todo).
- `--require-auth` falla al arrancar si no hay key (`cli_server.rs:714-729`).
- RBAC opcional (`token_role_map`: admin/writer/reader).
- Rate limiting doble (governor por IP + 5 fallos auth/60s por IP).
- TLS opcional (feature `tls`, rustls). Encriptación en reposo opcional `VANTADB_ENCRYPTION_KEY` (feature `encryption`).

### 2.8 Concurrencia / límites
- Pool (`src/connection_pool.rs:47-73`), `max_connections` default ≈ núcleos×4. Saturado → 503 + Retry-After:1. Queries en `spawn_blocking`.
- Circuit breaker: 5 fallos ≥500 → abierto 30s → 503.
- Payload: límite por defecto axum 0.8 = **2MB/body**. Sin límite de query explícito.
- **Sin streaming** en HTTP.

### 2.9 Cómo se levanta (3 vías)
1. `cargo build --bin vantadb-server` (workspace member) + env vars. Sin `--http` = HTTP por defecto; `--mcp` = MCP stdio.
2. `vanta-cli server --http ...` (feature `server`).
3. Docker `vantadb/server:latest`.

### 2.10 Recomendación para la app desktop Tauri
1. **Sidecar:** empaquetar `vantadb-server` como sidecar Tauri; spawn con `VANTADB_STORAGE_PATH=<app-data-dir>`, `VANTADB_PORT=18080`, `VANTADB_API_KEY=<generada>`, `VANTADB_REQUIRE_AUTH=true`.
2. **Cliente:** `reqwest` (rustls) desde el backend Rust de Tauri — NO desde el webview (evita CORS y expone la key).
3. Health-poll hasta 200, luego operar.
4. Respetar `Retry-After` ante 429/503; backoff exponencial ante 500/breaker. `VANTADB_RATE_LIMIT_RPM=0` para uso local single-client.
5. **Auth por defecto** (defensa en profundidad: otras apps locales podrían golpear el puerto).

---

## 3. Reporte sub-agente — `vantadb-mcp/` (MCP stdio)

### 3.1 Qué es
- Servidor MCP **hand-rolled** = **JSON-RPC 2.0 sobre stdio**, newline-delimited. **Solo stdio** — no hay SSE ni streamable HTTP (no hay socket/TCP/axum en el módulo).
- Punto de entrada único: `run_stdio_server(storage: Arc<StorageEngine>)` — `vantadb-mcp/src/lib.rs:347`.
- Lee por líneas (`BufReader::new(tokio::io::stdin()).lines()`, `lib.rs:384`); escribe a stdout con `\n` + flush (`lib.rs:462-465`).
- **Versión MCP `2024-11-05`** hardcodeada (`lib.rs:589`), verificada por tests.
- Métodos implementados: `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, `prompts/get`. **No** implementa `ping`, notificaciones, paginación de `tools/list`, capabilities logging/roots/sampling.

### 3.2 Crate MCP usado
**Ninguno.** Protocolo a mano (grep `rmcp|fastmcp|rust-mcp-schema` → 0 matches). Dependencias: solo `vantadb` (feature `cli`), `tokio`, `serde`, `serde_json`, `tracing`. Library-only (`publish=false`, excluido de releases binarias).

### 3.3 Tools (15, `lib.rs:808-964`)
| Tool | Params requeridos | Línea |
|---|---|---|
| `memory_put` | namespace, key, payload (+vector[], metadata{} opt) | 812 |
| `memory_get` | namespace, key | 827 |
| `memory_delete` | namespace, key | 836 |
| `memory_list` | namespace (limit 100, cursor, filters opt) | 845 |
| `memory_list_namespaces` | — | 859 |
| `query_lisp` | query (IQL; reads + mutaciones) | 864 |
| `search_semantic` | vector[] (f32), k (raw HNSW) | 873 |
| `search_memory` | namespace (+query_vector[], text_query, top_k 10, distance_metric cosine\|euclidean, explain, filters) | 883 |
| `get_node_neighbors` | node_id (number) | 900 |
| `inject_context` | content, thread_id | 909 |
| `read_axioms` | — (Devil's Advocate / Iron Axioms) | 919 |
| `collection_stats` | namespace | 924 |
| `collection_list` | — | 935 |
| `collection_delete` | namespace, confirm "yes" | 940 |
| `rehydrate` | summary_id (u128 as string) | 952 |

Handlers: `handle_tools_call` (`lib.rs:967-1482`). Límites: namespace ≤256B, key ≤512B, payload ≤1MB, vector ≤16384 dims, query ≤1MB.

**Resources (3):** `metrics://`, `memory://{ns}/{key}`, `namespace://{ns}`. ⚠️ Doc menciona `schema://` (`docs/api/MCP.md:79-81`) pero **no está implementado**.
**Prompts (4):** search_memory, analyze_namespace, summarize_context, query_builder (plantillas, no ejecutan).

### 3.4 Configuración y arranque (2 binarios)
- **`vanta-cli`**: `server --mcp` (sin `--http`) → `cmd_server_mcp` spawnea el binario `vantadb-server --mcp` con env `VANTA_DB=<db>`, stdio heredado (passthrough).
- **`vantadb-server`** (`main.rs`): detecta `--mcp` a mano (`args().any(|a| a == "--mcp")`, sin clap); abre `StorageEngine::open_with_config` con path de `VantaConfig::from_env()` → env `VANTADB_STORAGE_PATH` (default `vantadb_data`); `run_stdio_server` + `storage.flush()` al salir. En modo MCP todo el tracing va a **stderr** (stdout reservado al protocolo).

**⚠️ Gotcha (posible bug):** el wrapper `vanta-cli` setea `VANTA_DB` en el hijo (`server.rs:244`) pero `VantaConfig` lee `VANTADB_STORAGE_PATH` (`config.rs:408`) → `vanta-cli server --mcp --db /x` **puede no respetar /x** (DB caería en `vantadb_data` del CWD). Para la app desktop: setear `VANTADB_STORAGE_PATH` directo en el hijo.

- Build: `vantadb-server` **no está en default-members** (experimental) → `cargo build -p vantadb-server`.

### 3.5 Arquitectura: embebida
**El proceso MCP ES la base de datos.** Abre `StorageEngine` en su propio proceso; cada request crea `VantaEmbedded::from_engine(storage)` o `Executor::new(&storage)` para IQL. No hay red. Concurrencia: semaphore (max 32) + `spawn_blocking` + timeout 60s/request (`lib.rs:517-537`).

### 3.6 Conexión desde Tauri (recomendada)
1. Spawn `vantadb-server --mcp` con `VANTADB_STORAGE_PATH=<ruta-db>` y pipes.
2. Hablar JSON-RPC 2.0 por líneas sobre el pipe.
3. Handshake: `initialize` → `tools/list` → `tools/call`.
4. Matar el child al cerrar; SIGINT/Ctrl-C → graceful shutdown con flush.

**Crates Rust MCP client (2026):** `rmcp` (SDK oficial Rust, features `client` + `transport-child-process` — spawn del binario + stdio, `TokioChildProcess` con `graceful_shutdown`/kill-timeout). Alternativas: `rust-mcp-schema` (solo tipos), `fastmcp` (servidores), `mcp-protocol-sdk`. Alternativa mínima: cliente propio con `tokio::process::Command` + `BufReader::lines()` + `serde_json`.

**HTTP/SSE: NO es opción.** `docs/api/HTTP_API.md:124-125` ("Full MCP + HTTP") es engañoso: `mcp_mode = mcp && !http` (`server.rs:182`) → con ambos flags solo arranca el HTTP server (sin rutas MCP).

### 3.7 Limitaciones MCP
- **Single-client stdio** (un stdin/proceso); multi-IDE = un proceso por IDE sobre el mismo DB path.
- **Procesamiento estrictamente secuencial** (sin pipelining; el semáforo es casi irrelevante).
- Timeout 60s/request. Payload 1MB, key 512B, ns 256B, vector 16384 dims, top_k ≤1000, list ≤10000.
- **Sin autenticación** — el cliente stdio tiene control total de la DB (`query_lisp` ejecuta IQL arbitrario con mutaciones).
- Estado compartido: cada proceso embebido es dueño del lock file (`src/storage/engine/init.rs:40`).

### 3.8 Docs
`docs/api/MCP.md` (canónica, versiones impl 0.1.5 / protocolo 2024-11-05), `docs/book/src/api/MCP.md`, `docs/operations/CI_POLICY.md:65` (experimental), `docs/operations/DEPLOYMENT_GUIDE.md`, ADR `003_sync_async_decoupling.md`. Tests: `vantadb-mcp/tests/mcp_tests.rs` (1021 líneas), `vantadb-server/tests/mcp_integration.rs`.

**Staleness:** doc menciona tool `query` pero el código expone `query_lisp`; resource `schema://` no existe; `serverInfo.name = "vantadb"` (`src/metadata.rs:19`).

---

## 4. Reporte sub-agente — `vantadb-node/` + `vantadb-ts/`

### 4.1 `vantadb-node/` — addon nativo napi-rs (cdylib)
- **Naturaleza:** addon N-API generado con **napi-rs v3**. Crate standalone, **NO miembro del workspace** (`[workspace]` vacío — evita crash linker MSVC con cdylib). `crate-type = ["cdylib"]`, lib `vantadb_native`.
- Deps: `napi` (napi8, serde-json, tokio_rt), `napi-derive`, `serde_json`, `tokio`, `vantadb` (features `fjall, memmap2, rayon`).
- Binario presente: `vantadb-node/vantadb_native.win32-x64-msvc.node` (4.3 MB). Targets: win-x64-msvc, linux x86_64/aarch64, darwin aarch64/x86_64.
- **API (index.d.ts):** clase `VantaDb` (alias `VantaDB`): `connect(path, {read_only?, memory_limit?})`, `flush()`, `close()`, `put(record)`, `putBatch(records)`, `get(ns, key)`, `delete(ns, key)`, `list(ns, {filters?, limit?, cursor?})`, `listNamespaces()`, `search(request)`, `capabilities()`. Todas async (menos capabilities).
- Implementación: `VantaDB { engine: VantaEmbedded, op_gate: OpGate }`. Ops en `spawn_blocking` con `engine.clone()` (nunca bloquean el hilo JS). Boundary I/O = `serde_json::Value`. Guard `MAX_VEC_DIM = 10_000`. `:memory:` → `BackendKind::InMemory`.
- `OpGate` (Condvar/Mutex) = barrera de durabilidad: `close()` rechaza ops nuevas y drena las in-flight (un put fire-and-forget no se pierde tras close).
- **Persistencia: fjall + WAL + fsync nativo** (diferencial central vs WASM). Test: `tests/persistence.test.ts:49-71`.
- **No expone** graph, IQL, export/import ni mantenimiento.

### 4.2 `vantadb-ts/` — wrapper TS sobre WASM (+ backend nativo)
- Paquete npm `vantadb` v0.5.0. Dep runtime única: `vantadb-wasm: file:../vantadb-wasm/pkg`. `vantadb-node` es **solo devDependency** (no se publica). `dist/` NO construido.
- **Dos clases:** `VantaDB` (WASM, síncrono) — CRUD + search + **graph completo** + mantenimiento + export/import + text index + IQL + `generateSnippet`. `NativeVantaDB` (napi, async) — subconjunto isomórfico (connect/close/flush/capabilities/put/putBatch/get/delete/list/listNamespaces/search).
- **No hay auto-selección de backend:** el usuario elige `VantaDB` (WASM) o `NativeVantaDB` (napi). Si el `.node` no está para la plataforma → `VantaError` código `NATIVE_ERROR`.
- Persistencia WASM = in-memory por defecto; OPFS/IDB solo en browser. La persistencia real en archivos **solo** vía `NativeVantaDB`/vantadb-node.
- Nota: `reindexHnswFromText` lanza `WASM_ERROR` (no disponible en build WASM actual, `vantadb.ts:542-548`).

### 4.3 Relación entre ambos
Isomórficos en el subconjunto cubierto; **dos backends separados** del mismo SDK Rust, no uno sobre el otro. ADR: `docs/architecture/adr/COMP-029-napi-rs-node-bindings.md` ("backend ADICIONAL — no reemplazo"). Ambos envuelven `VantaEmbedded::open_with_config` (`src/sdk/builder.rs:91`).

### 4.4 ¿DB embebida?
**Sí, ambos.** node: el addon `.node` se carga en el proceso Node (FFI), `VantaEmbedded` vive en el mismo proceso. ts (WASM): corre en el mismo runtime JS. Matar el proceso mata la instancia (datos persisten por fjall en disco).

### 4.5 Evaluación Tauri (punto clave)
**Tauri (Rust) no puede `require()` un addon napi-rs** (no hay runtime Node en el proceso Rust). Opciones:
- **Opción A (recomendada):** crate `vantadb` directa en `src-tauri/`. Todo lo que expone `index.d.ts` es un wrapper delgado de serde_json sobre `VantaEmbedded` (sin lógica extra) → la API se replica 1:1 trivial. Además el crate da **más**: graph (BFS/DFS/toposort), IQL, export/import, `count`, `similar_to_key`, `delete_by_filter`.
- **Opción B:** spawn de proceso Node sidecar (plugin `shell` + `externalBin`) — añade runtime Node al bundle; estrictamente inferior a A.
- **Opción C:** WASM en webview (`vantadb-ts`) — demo in-memory; persistencia limitada a OPFS/IDB. No para memoria durable.

**Veredicto:** `vantadb-node` y `vantadb-ts` quedan como **referencia de API** para diseñar los `#[tauri::command]`, no como vía de integración directa.

---

## 5. Reporte sub-agente — `vantadb-wasm/` + `vantadb-python/`

### 5.1 `vantadb-wasm/`
- **Naturaleza:** crate que compila a WASM vía wasm-bindgen. **Incrusta el motor completo** (`VantaEmbedded`) dentro del binario WASM (no es wrapper del core). Dep: `vantadb` con `default-features=false, features=["wasm"]` (feature vacía, solo desactiva defaults).
- Features: `default=["tracing-wasm"]`, `opfs=[]` (habilita worker.rs). **El `pkg/` precompilado NO incluye worker** (sin `connect_worker`/`worker_read`...).
- **Inicialización (4):** `new(config)`, `static open(path)` (sync), `connect_persistent(path)` (motor + OPFS + load snapshot), `connect_idb(path)` (IDB fallback), `connect_worker(path)` (solo feature `opfs`).
- **API:** CRUD memoria (put/put_batch/get/delete/list/list_namespaces), búsqueda (search/search_vector/explain_memory_search), **grafo completo** (insert_node/get_node/delete_node/add_edge/graph_bfs/graph_dfs/graph_topological_sort/graph_is_dag), import/export (import_records/import_file/export_namespace/export_all/bulk_import/bulk_import_bytes), mantenimiento (rebuild_index/reindex_hnsw_from_text/compact_layout/flush/compact_wal/purge_expired/audit_text_index/audit_text_index_deep/repair_text_index/operational_metrics/capabilities/query/generate_snippet/close).
- `u64/u128` se serializan como **String** (precisión JS). Límites: vector 10M dims, batch 100k, export 1M records.

**Persistencia — punto clave:** el motor siempre se abre como **InMemory** (`build_config()` fuerza `backend_kind: BackendKind::InMemory`, `lib.rs:66`). **NO hay WAL ni fjall en WASM.** La "persistencia" es un **snapshot JSON completo** del estado en un solo `db_state.json`:
- `save()`: `collect_all_deduped()` recorre todos los namespaces, serializa a JSON, escribe. `load()`: lee y `import_records()`.
- **OPFS:** `navigator.storage.getDirectory()`, escritura atómica temp+move, footer CRC-32.
- **IndexedDB:** bridge JS inline en `globalThis.vantaIdbStorage`, DB "VantaDB"/store "state", BroadcastChannel sync cross-tab + Web Locks API.
- **Worker** (feature `opfs`): postMessage + MessageChannel, timeout 5000ms, 2 reintentos, backoff.

**Limitaciones de persistencia WASM:**
1. **Snapshot completo, no WAL** — O(DB) por guardado, sin fsync garantizado.
2. **Cap 1M de registros** para export (`lib.rs:420`).
3. Estado solo en el **sandbox del origen** (OPFS/IDB por-origen) — no hay ruta real al filesystem.
4. OPFS requiere browser moderno (Chrome 86+, Edge 86+, Firefox 111+, Safari 15.2+); **WebKit (WKWebView) soporte histórico incompleto**.
5. Build `pkg/` precompilado sin worker (recompilar con `--features opfs` + servir `opfs_bridge.js`).

**Evaluación Tauri:** para la mayoría de casos usar `vantadb-wasm` en el webview es **lo peor de ambos mundos**: el backend Rust ya tiene FS real (pasar por OPFS es un sandbox emulado dentro de otro), persistencia O(DB) inviable con DB grandes, costo de serialización WASM↔JS (comentario ponytail: ~2-5µs/vector, `lib.rs:1070-1073`), y el webview de Tauri no garantiza OPFS/Workers en todas las plataformas. **Solo tiene sentido** si se reutiliza la misma UI para web/desktop o modo demo.

### 5.2 `vantadb-python/` (PyO3)
- **Naturaleza:** crate `vantadb_py` expone el motor embebido a Python **in-process** vía PyO3 (sin red). `crate-type=["cdylib"]` (extensión CPython). Deps: `pyo3 0.29` (extension-module, abi3-py311), `chrono`, core `vantadb` (features `fjall, memmap2, rayon`). Build con `maturin`.
- **⚠️ Aclaración sobre "feature python_sdk":** hay DOS capas PyO3:
  1. `vantadb-python/` (moderna, la usada) — crate separado que **NO usa** `python_sdk`, usa `["fjall","memmap2","rayon"]`.
  2. `src/python.rs` (legacy) — bindings antiguos en el core gated por `#![cfg(feature="python_sdk")]`, expone `ClientEngine` mínima (`new/execute/insert_node`) — **obsoleta y distinta**.
  - → La afirmación de DESKTOP-01 (`:15`) de que vantadb-python usa `python_sdk` es **inexacta**. La integración real es el crate `vantadb_py` contra el SDK público `VantaEmbedded`.
- **API (clase `VantaDB`):**
  - Constructor: `VantaDB(db_path, memory_limit_bytes=None, read_only=False, backend=None)`; `backend`: `"rocksdb"|"memory"|None→Fjall`; `":memory:"`→in-memory. `connect(path, memory_limit=None)` alternativa.
  - Exporta: `VantaVector`, `VantaVectorIter`, `VantaSearchHit`, `VantaMemoryRecord`, `VantaListResult`, `__version__`. `AsyncVantaDB` = wrapper `asyncio.to_thread` + semáforo.
  - Memoria: `put/get_memory/delete_memory/list_memory/search_memory/explain_memory_search`.
  - Batch: `put_batch` (tuplas posicionales **deprecadas**) / `put_batch_raw` (numpy zero-copy PyBuffer f32/f64).
  - Nodo/grafo: `insert/get/delete/search/search_batch` (Rayon) / `add_edge/graph_bfs/graph_dfs/graph_topological_sort/graph_is_dag/graph_page_rank/graph_degree_centrality/recover_archived_nodes`.
  - Import/Export: `export_namespace/export_all/import_file/bulk_import/bulk_import_bytes`.
  - Mantenimiento: `rebuild_index/reindex_hnsw_from_text/flush/compact_wal/purge_expired/compact_layout/audit_text_index/repair_text_index/query(iql)/generate_snippet/capabilities/hardware_profile/operational_metrics/close/list_namespaces`.
  - **GIL:** casi todos los métodos usan `py.detach(...)` (GIL released durante I/O).
- **Persistencia real:** `BackendKind::Fjall` default (persistente). Fjall con `PersistMode::SyncAll` = **fsync de datos + metadata** (`fjall_backend.rs:220-223`). WAL propio con `sync_data()` al flush. `VantaConfig.force_fsync`. Layout on-disk verificado: `.vanta.lock`, `0.jnl`, `keyspaces/N/{current,vN}`, `data/vanta.wal`, `data/vector_store.vanta`, `data/vector_index.bin`. Lock multi-proceso: `read_only=True` para acceso concurrente.
- **¿API Python subconjunto de Rust?** **Sí — wrapper 1:1 delgado, subconjunto estricto de `VantaEmbedded`.** Métodos Rust que Python NO expone: `import_records` (Python solo `import_file`), `remove_edge`, `delete_by_filter`, `count`, `similar_to_key`, `search_multi`, `search_all`, `vacuum`, `pipeline`, `optimizer_config`, `graph_bfs_filtered`/`graph_dfs_filtered`, accumuladores de grafo, snapshots (`create_snapshot`/`list_snapshots`), threads/mensajes (`create_thread`/`send_message`/...), `graphrag_search`, todos los `debug_*`.
- **Docs:** `vantadb-python/README.md` **DESACTUALIZADO** (`put_memory` l.33, `search_hybrid` l.48, `memory_stats` l.59 — no existen). `docs/api/PYTHON_SDK.md` correcto y completo (746 líneas).

**Evaluación Tauri:**
- **Opción A (subproceso Python):** la peor vía. No existe CLI/REPL en el paquete (sin entry_points) → habría que escribir un driver script Python con JSON por stdio. Distribución: empaquetar runtime Python (~40-100MB+ con numpy) + `.pyd` por plataforma (solo hay wheel `cp311-abi3-win_amd64` local). Latencia/IPC innecesarios.
- **Opción B (crate directa):** la recomendada — el mismo `VantaEmbedded` es una crate Rust que Tauri ya puede linkear. Nada de lo que Python puede hacer se pierde.

### 5.3 Resumen ejecutivo WASM vs Python vs Rust directo
| | wasm | python | Rust directo |
|---|---|---|---|
| Motor | VantaEmbedded wasm32 | VantaEmbedded vía PyO3 | VantaEmbedded linkeado |
| Backend | Siempre InMemory | Fjall/RocksDB/Memory | Fjall/RocksDB/Memory |
| Persistencia | Snapshot JSON OPFS/IDB (cap 1M) | WAL+fjall con fsync | WAL+fjall con fsync |
| Costo Tauri | Alto | Alto (runtime+driver) | Bajo |
| Recomendación | ❌ solo reuso UI web | ❌ solo scripting Python | ✅ vía recomendada |

---

## 6. Hallazgos críticos para el equipo (bugs/gotchas/stale-docs)

1. **Bug potencial MCP:** `vanta-cli server --mcp --db /x` puede **no respetar `/x`** — el wrapper setea `VANTA_DB` pero `VantaConfig` lee `VANTADB_STORAGE_PATH` (`server.rs:244` vs `config.rs:408`). Usar `VANTADB_STORAGE_PATH` directo.
2. **HTTP: `success:false` con status 200** — el error de ejecución viaja en el body, no en el status code. El cliente HTTP debe validar el body.
3. **HTTP_API.md claim "Full MCP + HTTP" es falso** — `mcp_mode = mcp && !http`; con ambos flags solo arranca HTTP.
4. **`vantadb-python/README.md` desactualizado** — usa métodos inexistentes.
5. **DESKTOP-01 decía "python_sdk feature" — inexacto** — la capa moderna es el crate `vantadb_py` (sin `python_sdk`); `src/python.rs` es legacy.
6. **MCP staleness:** doc dice tool `query`, código dice `query_lisp`; resource `schema://` documentado pero no implementado.
7. **WASM `pkg/` precompilado sin worker** — para OPFS worker hay que recompilar con `--features opfs`.
8. **`reindexHnswFromText` en TS lanza WASM_ERROR** — no disponible en el build WASM actual.
9. **CORS no implementado en el server HTTP** — relevante solo si se usa webview con fetch (Tauri nativo lo evita).
10. **vantadb-server/mcp/wasm son experimentales y NO default-members** — construir con `-p` explícito; `desktop/` debe desacoplarse del workspace.

---

## 7. Arquitectura definida — vanta-arch

> **Decisión:** construir `desktop/` como app Tauri v2 (frontend React+Vite reutilizando design system de `web/`, backend Rust en `src-tauri/`) con un **ConnectionManager** que abstrae las 6 vías detrás de un trait común `VantaConnection`, **vía nativa como default** y las demás conectables a elección del usuario, soportando varias simultáneas bajo la regla **"un escritor por path de DB"**.

### 7.1 Alternativas consideradas (y rechazadas)
1. Solo vía nativa (DESKTOP-01) — no satisface el requisito de 6 vías.
2. Solo HTTP como capa de indirección — proceso extra y latencia para el caso nativo (80% del uso); el server no soporta graph API completa ni el mismo contrato que la crate.
3. Trait síncrono + spawn_blocking — descartado: rmcp/HTTP/subprocesos son naturalmente async. Se usa `async_trait` (Box<dyn> object-safe) y `spawn_blocking` solo en el adaptador nativo.
4. WASM como conexión Rust "proxy" — over-engineering: WASM es demo/read-only; el frontend lo usa directo en JS.

### 7.2 Trait común `VantaConnection` (`src-tauri/src/connections/trait.rs`)
```rust
#[async_trait]
pub trait VantaConnection: Send + Sync {
    fn kind(&self) -> ConnectionKind;          // Native | Server | Mcp | Node | Python | Wasm
    fn id(&self) -> &str;
    fn scope(&self) -> ConnectionScope;         // DbPath(PathBuf) | Url(String) | Webview
    async fn connect(&mut self) -> Result<(), VantaError>;
    async fn close(&mut self) -> Result<(), VantaError>;
    async fn health(&self) -> Result<HealthReport, VantaError>;
    async fn capabilities(&self) -> Vec<Capability>; // write, search, graph, vector, durability, batch
    async fn ingest(&self, item: IngestItem) -> Result<(), VantaError>;
    async fn ingest_batch(&self, items: Vec<IngestItem>) -> Result<IngestReport, VantaError>;
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VantaError>;
    async fn get(&self, key: &str, namespace: Option<&str>) -> Result<Option<MemoryRecord>, VantaError>;
    async fn delete(&self, key: &str, namespace: Option<&str>) -> Result<bool, VantaError>;
    async fn list(&self, namespace: Option<&str>, limit: u32, offset: u32)
        -> Result<Vec<MemoryRecord>, VantaError>;
}
```

**Tipos de datos (todos Serialize+Deserialize+Clone, `connections/types.rs`):**
- `IngestItem { key, text?, vector?, payload?, namespace?, expires_at? }`
- `SearchQuery { text?, vector?, namespace?, top_k, filters?, hybrid }`
- `SearchResult { key, score, payload?, snippet? }`
- `MemoryRecord { key, payload, vector?, created_at, expires_at }`
- `HealthReport { ok, kind, backend, records?, db_path?, latency_ms?, message? }`
- `Capability`, `ConnectionKind`, `ConnectionScope`, `IngestReport { ingested, failed }`

**`VantaError` unificado (`#[non_exhaustive]`):** `Native|Http|Mcp|Node|Python|Wasm(String)` + `Connection|Lock|Timeout|NotFound|Config|Io|Unsupported(String)`. Los comandos Tauri devuelven `Result<T, VantaError>`.

### 7.3 Adaptadores (uno por vía)
| Adaptador | Vía real | Naturaleza | Notas |
|---|---|---|---|
| `NativeConnection` | crate `vantadb` embebida | sync → spawn_blocking | Default óptimo; toma lock fs2 del path |
| `ServerConnection` | HTTP REST (reqwest json) | async | config url/port/token/timeout; healthcheck+auth en connect |
| `McpConnection` | spawn `vantadb-server --mcp` + rmcp (`TokioChildProcess`) | async | 15 tools → trait; graceful_shutdown con kill-timeout |
| `NodeConnection` | sidecar `node driver.js` + JSON-RPC stdio | async | reusa `jsonrpc.rs`; dev=node sistema, release=externalBin |
| `PythonConnection` | spawn `python python_driver.py` + JSON-RPC stdio | async | reusa `jsonrpc.rs`; runtime: decisión DESKTOP-17 |
| `WasmConnection` | conexión vive en el webview (JS `vantadb`) | — | Rust solo metadata + health; CRUD → `Unsupported` |

### 7.4 ConnectionManager (multi-conexión)
```rust
pub struct ConnectionManager {
    registry: Mutex<HashMap<String, Arc<dyn VantaConnection + Send + Sync>>>,
    active: Mutex<Option<String>>,
    path_holders: Mutex<HashMap<PathBuf, String>>, // path -> connection id (regla 1-escritor)
}
```
- **Registry:** N conexiones vivas de vías distintas. **Routing:** cada comando acepta `connection: Option<String>`; None → activa. Routing por namespace = YAGNI en MVP.
- **Regla "un escritor por path":** conectar una segunda vía embebida (o el server) sobre el mismo path → `VantaError::Lock` con hint. Multi-escritura real = paths distintos por vía.
- **Capability gate:** write sobre conexión read-only (WASM) → `Unsupported`.
- **Ciclo de vida:** `Disconnected → Connecting → Connected → Error → Disconnected`, cada transición emite `vanta://connection-state`.
- **Shutdown:** `shutdown_all()` en `RunEvent::ExitRequested`, orden: webview → subprocesos (graceful_shutdown) → nativa última (flush); timeout + kill forzoso.

**Mapeo MCP → trait (referencia DESKTOP-13):** memory_put→ingest, memory_get→get, memory_delete→delete, memory_list/collection_list→list, search_semantic/search_memory/inject_context→search; query_lisp + collection_* restantes fuera del trait MVP.

### 7.5 Estructura `src-tauri/`
```
desktop/
├─ package.json / vite.config.ts / src/          # frontend React+Vite (reusa tokens de web/)
├─ drivers/
│  ├─ node/driver.js                             # require('vantadb_native') + JSON-RPC stdio
│  ├─ python/python_driver.py                    # import vantadb_py + JSON-RPC stdio
│  └─ bin/                                       # sidecars generados (fuera de git)
└─ src-tauri/
   ├─ Cargo.toml            # [workspace] vacío → propio workspace (NO miembro del raíz)
   ├─ tauri.conf.json       # identifier, frontendDist, externalBin
   ├─ capabilities/default.json
   └─ src/
      ├─ main.rs / lib.rs   # Builder: .manage(AppState), invoke_handler, setup, on exit
      ├─ commands/{mod,connection,data,config}.rs
      ├─ connections/{mod,trait,types,native,server,mcp,node,python,wasm,manager,jsonrpc}.rs
      ├─ config.rs          # config JSON en app_config_dir
      └─ error.rs           # VantaError
```
**Deps `src-tauri/Cargo.toml`:** `tauri` v2, `tauri-plugin-shell`, `vantadb` (`default-features=false` + `fjall,fs2,memmap2,roaring,advanced-tokenizer` — **nunca** cli/server/prometheus/python_sdk), `reqwest` (json), `rmcp` (client, transport-child-process), `async-trait`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`.
**State:** `AppState { manager: Arc<ConnectionManager>, config: Arc<ConfigStore> }`. Async commands usan `tauri::State<'_, AppState>` (owned wrapper), nunca `&State`/`&str`.

### 7.6 Commands Tauri
`vanta_connect`, `vanta_disconnect`, `vanta_list_connections`, `vanta_set_active`, `vanta_health`, `vanta_ingest`, `vanta_ingest_batch`, `vanta_search`, `vanta_get`, `vanta_delete`, `vanta_list`, `vanta_stats`, `vanta_register_webview`, `vanta_config_get`/`vanta_config_set`. Keys/namespaces siempre `String`.

### 7.7 Config persistida (`config.json` en `app_config_dir`, escritura atómica temp+rename)
```json
{
  "active_connection": "native-1",
  "connections": [
    { "name": "native-1", "kind": "native", "db_path": "..." },
    { "name": "server-1", "kind": "server", "url": "http://127.0.0.1:7788", "auth_token": "...", "timeout_ms": 5000 }
  ],
  "ui": { "last_namespace": "default" }
}
```

### 7.8 Eventos / streaming
- `vanta://connection-state` (obligatorio)
- `vanta://ingress-progress` (flag `progress`)
- `vanta://search-progress` (opcional)
- Implementación: `tauri::ipc::Channel` o `AppHandle::emit`; frontend escucha con `@tauri-apps/api/event`.

### 7.9 Frontend MVP
React + Vite; bridge `src/lib/vanta.ts` (wrapper tipado de invoke); hook `useConnectionState`. Componentes: `<ConnectionPanel>` (health badges), `<ConnectionSelector>` (selector vía activa + warning conflicto path), `<IngestForm>`, `<SearchBar>` + `<ResultsList>`, `<ConfigModal>`. Estado: React context local (YAGNI).

---

## 8. Plan de tareas DESKTOP (resumen)

26 tareas (DESKTOP-02..27) — detalle completo en `docs/Backlog.md` → **Phase 12**. Regla: 1 tarea = 1 concepto; ninguna mezcla 2 integraciones; `desktop/` no toca el workspace raíz.

| Fase | Tareas | Contenido |
|------|--------|-----------|
| 0 — Scaffold | DESKTOP-02, 03 | Tauri v2 + workspace propio; crate `vantadb` + managed state + healthcheck |
| 1 — Trait + nativo + UI | DESKTOP-04..07 | Trait VantaConnection + tipos + errores; NativeConnection; commands CRUD; frontend MVP |
| 2 — Server HTTP | DESKTOP-08..10 | HTTP client tipado; ServerConnection; wire en commands + UI |
| 3 — MCP stdio | DESKTOP-11..14 | Spawn manager; cliente rmcp; McpConnection; healthcheck/reconnect/UI |
| 4 — Node + Python | DESKTOP-15..18 | JSON-RPC + driver Node; NodeConnection; driver Python + decisión runtime; PythonConnection |
| 5 — Multi-connection | DESKTOP-19..21 | ConnectionManager; shutdown_all; UI multi-connection |
| 6 — Empaquetado/CI/docs | DESKTOP-22..27 | Eventos streaming; config persistida; empaquetado; CI; tests; docs + ADR |

Secuencia: Fase 0 → 1 → (2/3/4 en paralelo) → 5 → 6.

---

## 9. Riesgos técnicos (arquitectura)

1. **Lock de archivo de la DB (1-escritor por path).** Fjall toma lock exclusivo (fs2). Mitigación: `path_holders` (DESKTOP-19) + warning UI (DESKTOP-21) + hint `VantaError::Lock`. Multi-escritura = paths distintos.
2. **CORS/seguridad del HTTP server.** Validar modelo auth real (DESKTOP-08); nunca exponer el puerto a no-loopback.
3. **Sidecar Node.** Empaquetar runtime node + addon `.node` (ABI napi por plataforma) es frágil. Dev=node sistema; release=externalBin; feature-gate si pesa.
4. **Driver Python.** Runtime + wheel por plataforma pesado. MVP=python sistema; decisión explícita DESKTOP-17; vía opcional.
5. **rmcp cliente stdio.** `TokioChildProcess::graceful_shutdown` ya maneja close+kill-timeout; capturar stderr a log (DESKTOP-11); confirmar flag `--mcp`.
6. **WASM/TS.** Solo demo/read-only (snapshot, cap 1M). La conexión vive en JS; Rust devuelve `Unsupported`. Health vía `vanta_register_webview`.
7. **Async commands Tauri.** No usar `&str`/`&State` en async (future no `'static`). Keys/namespaces `String`; state `tauri::State<'_, AppState>`.
8. **Workspace raíz y CI.** `vantadb-server`/`mcp`/`wasm` NO default-members; `[workspace]` vacío en `src-tauri/Cargo.toml` desacopla; CI desktop en workflow propio (DESKTOP-25).
9. **Tiempo de build.** `vantadb` + fjall es pesado; `src-tauri` fuera del CI de core.
10. **Durabilidad heterogénea.** `capabilities().durability` por conexión; la UI muestra el nivel (no asumir sync donde no lo hay).

---

## 10. ADR (inline — persistir en DESKTOP-27)

**Estado:** Aceptado (propuesta) · **Fecha:** 2026-08-04 · **Área:** desktop
**Contexto:** DESKTOP-01 identificó la vía nativa como óptima, pero el producto debe poder conectarse a las 6 vías individualmente o en simultáneo.
**Decisión:**
1. Backend Tauri v2 en `desktop/` con `src-tauri` desacoplado del workspace raíz (`[workspace]` vacío).
2. Trait `VantaConnection` (async, object-safe con async_trait) + adaptador por vía; `ConnectionManager` como único punto de entrada de comandos.
3. Vía nativa (crate `vantadb` embebida) como default; selector de vía por usuario; N conexiones simultáneas con la invariante "un escritor por path de DB".
4. WASM/TS como conexión de webview (read-only, metadata en Rust, ops en JS) — sin proxy Rust.
5. Persistencia de config en `app_config_dir`; eventos Tauri para estado/progreso; shutdown ordenado con kill-timeout.
**Consecuencias:** + compatibilidad universal y testabilidad por vía; − complejidad de 6 adaptadores y empaquetado de runtimes (Node/Python) detrás de feature-gates; − educar al usuario sobre la regla 1-escritor.

---

## Anexo A — Reportes completos verbatim de los sub-agentes

> Los §2-§5 del cuerpo son versiones condensadas. Este anexo preserva los reportes íntegros tal como los entregaron los sub-agentes, para no perder ningún detalle (números de línea, firmas, tablas, veredictos).

### A.1 Reporte: módulo vantadb-mcp/ — VantaDB

**1. Qué implementa y qué transporte**

Implementación: servidor MCP hand-rolled = JSON-RPC 2.0 sobre stdio, mensajes delimitados por línea (un JSON por línea, terminado en \n).
- Punto de entrada único: `run_stdio_server(storage: Arc<StorageEngine>)` — vantadb-mcp/src/lib.rs:347
- Lee de stdin por líneas: `BufReader::new(tokio::io::stdin()).lines()` — lib.rs:384
- Escribe respuestas a stdout con `\n` + `flush()` — lib.rs:462-465
- Transporte soportado: **SOLO stdio**. No hay SSE ni streamable HTTP en ningún lado del módulo (no hay socket/TCP/axum en vantadb-mcp/).
- Versión MCP: **2024-11-05** — hardcodeada en el handshake — lib.rs:589; verificada por tests (tests/mcp_tests.rs:25, vantadb-server/tests/mcp_integration.rs:20).

Métodos implementados (dispatcher en lib.rs:513-567): `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, `prompts/get`. No implementa `ping`, `notifications/initialized`, paginación de `tools/list`, ni capabilities de logging/roots/sampling.

**2. Qué crate MCP usa**

Ninguno. No hay rmcp, fastmcp ni rust-mcp-schema (grep en todos los *.toml del repo: 0 matches). Es protocolo a mano:
- Tipos de wire: `RpcRequest`/`RpcResponse` serde — lib.rs:167-183
- Errores JSON-RPC estándar (-32700, -32602, -32601, -32603, -32600) — lib.rs:106-163
- Dependencias del crate: solo vantadb (feature cli), tokio, serde, serde_json, tracing — vantadb-mcp/Cargo.toml:10-14
- El crate es library-only (`publish = false`; docs/CHANGELOG.md:42: "excluded from binary releases")

**3. Tools, Resources y Prompts expuestos**

Tools (15) — definidas en lib.rs:808-964

| Tool | Params (requeridos en bold) | lib.rs |
|---|---|---|
| memory_put | namespace, key, payload, vector[] (opt), metadata{} (opt) | 812 |
| memory_get | namespace, key | 827 |
| memory_delete | namespace, key | 836 |
| memory_list | namespace, limit (default 100), cursor (opt), filters{} (opt) | 845 |
| memory_list_namespaces | ninguno | 859 |
| query_lisp | query (VantaLISP/IQL; permite reads y mutaciones) | 864 |
| search_semantic | vector[] (f32), k (raw HNSW) | 873 |
| search_memory | namespace, query_vector[] (opt), text_query (opt), top_k (default 10), distance_metric (cosine\|euclidean), explain (bool), filters{} (opt) | 883 |
| get_node_neighbors | node_id (number) | 900 |
| inject_context | content, thread_id | 909 |
| read_axioms | ninguno (Devil's Advocate / Iron Axioms) | 919 |
| collection_stats | namespace | 924 |
| collection_list | ninguno | 935 |
| collection_delete | namespace, confirm ("yes") | 940 |
| rehydrate | summary_id (u128 as string; recupera nodos archivados) | 952 |

Handlers de ejecución: `handle_tools_call` en lib.rs:967-1482 (validación de límites: namespace ≤256B, key ≤512B, payload ≤1MB, vector ≤16384 dims, query ≤1MB — lib.rs:196-252, `McpConfig` en lib.rs:23-64).

Resources (3 URIs) — lib.rs:605-706
- `metrics://` — métricas operacionales (HNSW, memoria, storage)
- `memory://{namespace}/{key}` — un record
- `namespace://{namespace}` — listado de un namespace
- ⚠️ La doc menciona `schema://` (docs/api/MCP.md:79-81) pero **no está implementado** en el código.

Prompts (4) — lib.rs:711-749: `search_memory`, `analyze_namespace`, `summarize_context`, `query_builder` (plantillas de prompt, no ejecutan nada).

**4. Configuración y arranque**

Hay dos binarios en la cadena:

a) `vanta-cli` (src/bin/vanta-cli.rs) — wrapper que hace spawn:
- Subcomando server con flags `--http`, `--mcp`, `--port`, `--host`, `--require-auth` — src/cli.rs:286-307
- `--db` es global, env `VANTA_DB`, default `./db` — src/cli.rs:14-16
- Con `--mcp` (sin `--http`): `cmd_server_mcp` — src/cli_handlers/server.rs:182-185, 227-319
- Ese handler spawnea el binario `vantadb-server` con arg `--mcp`, env `VANTA_DB=<db>`, `VANTADB_PORT/HOST/REQUIRE_AUTH` opcionales, y stdio heredado (`Stdio::inherit()`) — server.rs:242-258. Es un **passthrough**: el MCP client habla con vanta-cli y éste reenvía al hijo.

b) `vantadb-server` (vantadb-server/src/main.rs) — el servidor MCP real:
- Detecta `--mcp` a mano (`args().any(|a| a == "--mcp")`) — main.rs:27 (sin clap)
- Abre la DB: `StorageEngine::open_with_config(config.storage_path, ...)` — main.rs:33
- Path de DB desde `VantaConfig::from_env()` (= `default()`) → env `VANTADB_STORAGE_PATH`, default "vantadb_data" — src/config.rs:406-411, 657-659
- `run_stdio_server` + `storage.flush()` al salir — main.rs:44-51
- Telemetría: en modo MCP todo el tracing va a **stderr** (stdout queda reservado al protocolo) — src/cli_server.rs:555-588

⚠️ **Gotcha importante (posible bug):** el wrapper setea `VANTA_DB` en el hijo (server.rs:244), pero `VantaConfig` lee `VANTADB_STORAGE_PATH` (config.rs:408). Es decir, `vanta-cli server --mcp --db /x` **puede no respetar /x** en el proceso hijo — la DB caería en `vantadb_data` relativo al CWD. Para una app desktop, setear `VANTADB_STORAGE_PATH` directamente en el hijo es lo confiable.

Build: `vantadb-server` NO está en default-members del workspace (experimental) — Cargo.toml:593-599. Construir con `cargo build -p vantadb-server` (binario vantadb-server / vantadb-server.exe — server.rs:235-240).

Config del MCP server (`McpConfig`, lib.rs:23-74): valores hardcodeados con default; solo `max_concurrency` se toma del storage (lib.rs:68-73). No expuestos por CLI/env.

**5. Arquitectura: ¿embebida o servidor externo?**

**Embebida. El proceso MCP ES la base de datos.**
- `vantadb-server` abre `StorageEngine` en su propio proceso — main.rs:33
- Cada request crea un `VantaEmbedded::from_engine(storage)` sobre ese engine (ej. lib.rs:1020, 1039, 1095, 1224) o un `Executor::new(&storage)` para IQL (lib.rs:531)
- No hay conexión a ningún servidor externo; no hay red de por medio.
- Concurrencia interna: semaphore (max 32) + `spawn_blocking` + timeout 60s por request — lib.rs:517-537 (mismo patrón documentado en docs/Investigaciones/INV-003-tokio-blocking-audit.md:37).

**6. Cómo se conectaría una app desktop Tauri**

Opción recomendada — **spawn stdio** (es exactamente el patrón que usan los IDEs, docs/api/MCP.md:113-196):
1. La app Tauri (Rust backend) hace spawn de `vantadb-server --mcp` con env `VANTADB_STORAGE_PATH=<ruta-db>` y pipes en stdin/stdout/stderr.
2. Habla JSON-RPC 2.0 por líneas (newline-delimited) sobre ese pipe.
3. Handshake: `initialize` → `tools/list` → `tools/call`.
4. Ciclo de vida: matar el child al cerrar la app; SIGINT/Ctrl-C hace graceful shutdown con flush (lib.rs:353-360, main.rs:46-51).

Crates MCP client en Rust para Tauri (2026):
- **rmcp** — SDK oficial de Rust MCP (modelcontextprotocol/rust-sdk), ~4.7M+ descargas, v3.x (implementa spec 2026-07-28, compatible con 2025-11-25 y anteriores, incluida 2024-11-05). Para cliente stdio: features = ["client", "transport-child-process"] (spawn del binario servidor y habla por stdio) o transport-io. Es la opción más limpia.
- rust-mcp-schema — solo tipos del protocolo.
- fastmcp (Rust) — enfocado a servidores.
- mcp-protocol-sdk — server+client, spec 2025-06-18.
- Alternativa mínima: cliente propio con `tokio::process::Command` + `BufReader::lines()` + `serde_json` — el protocolo es tan simple (líneas + JSON-RPC) que es viable sin SDK; útil si solo se quieren llamar tools puntuales y no se necesita negociación completa.

**HTTP/SSE: NO es opción hoy.** El servidor no implementa transporte HTTP ni SSE. La línea docs/api/HTTP_API.md:124-125 ("Full MCP + HTTP", `vanta-cli server --http --mcp`) es engañosa: `mcp_mode = mcp && !http` (server.rs:182) → con ambos flags solo arranca el HTTP server, que no tiene rutas MCP (grep de cli_server.rs: solo referencias a is_mcp de telemetría).

**7. Limitaciones**

- **Single-client stdio**: una conexión por proceso (un stdin). Multi-IDE se logra con un proceso por IDE, todos abriendo la misma DB path (docs/api/MCP.md:201).
- **Procesamiento estrictamente secuencial**: el loop lee una línea, espera `dispatch_request` completo (await), escribe respuesta, y recién lee la siguiente — lib.rs:386-472. No hay pipelining; el semáforo max_concurrency es prácticamente irrelevante dado el loop serial.
- Timeout de 60s por request — lib.rs:527-536.
- Límites de payload: 1MB payload/query, key 512B, namespace 256B, vector 16384 dims, top_k ≤1000, list limit ≤10000 — lib.rs:51-62.
- **Subconjunto del protocolo**: sin ping, sin manejo de notificaciones, sin paginación en tools/list, sin streaming/batch (listados en "Future Enhancements", docs/api/MCP.md:302-306).
- **Seguridad**: `query_lisp` ejecuta IQL arbitrario (mutaciones incluidas); hay sanitización de null bytes/longitud (lib.rs:1118-1132) pero **no autenticación** — el cliente stdio tiene control total de la DB.
- **Estado compartido**: cada proceso embebido es un dueño de la DB (con lock file — src/storage/engine/init.rs:40). Dos instancias simultáneas sobre el mismo path dependen del locking del storage; la doc afirma que es soportado (MCP.md:201).

**8. Docs existentes sobre MCP en el repo**

| Doc | Contenido |
|---|---|
| docs/api/MCP.md | La doc canónica: tools, resources, prompts, arranque (`vanta-cli server --mcp --db`), config per-IDE (Cursor/Claude/Windsurf/OpenCode/Cline), ejemplos JSON-RPC, versiones (impl 0.1.5, protocolo 2024-11-05 — MCP.md:310-312) |
| docs/book/src/api/MCP.md | Incluye la anterior en el mdbook |
| docs/api/HTTP_API.md:124-125 | Claim "Full MCP + HTTP" — **no implementado** (ver §6) |
| docs/operations/CI_POLICY.md:65 | vantadb-mcp = Experimental |
| docs/operations/DEPLOYMENT_GUIDE.md:14,56 | Un solo binario, modos embedded/HTTP/MCP |
| docs/architecture/adr/003_sync_async_decoupling.md:31,48 | MCP sobre tokio, pool bloqueante |
| docs/Investigaciones/INV-003-tokio-blocking-audit.md:37 | Patrón semaphore+spawn_blocking+timeout |
| docs/backlog-guide.md:68-75 + docs/Backlog.md | MCP-02..05 (estabilizar a GA, tools collection, test suite) |
| Tests: vantadb-mcp/tests/mcp_tests.rs (suite completa, 1021 líneas), vantadb-server/tests/mcp_integration.rs | Certificación del protocolo |

**Staleness detectada:** la doc menciona tool `query` (MCP.md:56) pero el código expone `query_lisp`; resource `schema://` no existe; la versión "0.1.5" es de la doc, el protocolo reportado es siempre 2024-11-05 con `serverInfo.name = "vantadb"` (src/metadata.rs:19).

### A.2 Investigación: vantadb-node/ y vantadb-ts/ — reporte completo

Los 6 módulos de integración del repo son: vantadb-mcp, vantadb-node, vantadb-python, vantadb-server, vantadb-ts, vantadb-wasm. Este reporte cubre a fondo los dos pedidos, con la evaluación Tauri al final.

**1. vantadb-node/ — addon nativo napi-rs (cdylib)**

*Naturaleza*

Addon nativo N-API para Node.js generado con napi-rs v3 (napi + napi-derive). Es una crate standalone que **NO es miembro del workspace** (evita crash del linker MSVC con cdylib en workspace):
- vantadb-node/Cargo.toml:10-17 — `[workspace]` vacío + `[lib] name = "vantadb_native"`, `crate-type = ["cdylib"]`.
- vantadb-node/Cargo.toml:19-24 — deps: napi (features napi8, serde-json, tokio_rt), napi-derive, serde_json, tokio, y vantadb = { path = "..", features = ["fjall", "memmap2", "rayon"] }.
- vantadb-node/build.rs:1-3 — `napi_build::setup()`.
- vantadb-node/.cargo/config.toml — target-dir = "../target" (comparte el target del workspace).
- Binario compilado presente: `vantadb-node/vantadb_native.win32-x64-msvc.node` (4.3 MB).
- vantadb-node/package.json:24 — build: `napi build --platform --release --js index.cjs && napi build --platform --release --esm --js index.js`.
- vantadb-node/package.json:32-41 — `napi.binaryName = "vantadb_native"`, targets: x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, aarch64-apple-darwin, x86_64-apple-darwin.

index.js/index.cjs son glue generado por NAPI-RS que hace require() del .node por plataforma (vantadb-node/index.js:67-80).

*API pública (firmas exactas, de vantadb-node/index.d.ts)*

Clase `VantaDb` (alias `VantaDB`), index.d.ts:7-62:
- `static connect(path: string, options?: { read_only?: boolean, memory_limit?: number }): Promise<VantaDb>` — `:memory:` o directorio real (persistente) (index.d.ts:16).
- `flush(): Promise<void>` — index.d.ts:18.
- `close(): Promise<void>` — index.d.ts:28.
- `put(record: { namespace, key, payload, metadata?, vector?, ttl_ms? }): Promise<any>` — index.d.ts:35.
- `putBatch(records: any): Promise<any>` — index.d.ts:37.
- `get(namespace: string, key: string): Promise<any | null>` — index.d.ts:39.
- `delete(namespace: string, key: string): Promise<boolean>` — index.d.ts:44.
- `list(namespace: string, options?: { filters?, limit?, cursor? }): Promise<{ records, next_cursor? }>` — index.d.ts:51.
- `listNamespaces(): Promise<Array<string>>` — index.d.ts:53.
- `search(request: any): Promise<any>` (hits { record, score, explanation? }) — index.d.ts:59.
- `capabilities(): any` (síncrono) — index.d.ts:61.

*Implementación (src/lib.rs)*
- `VantaDB { engine: VantaEmbedded, op_gate: OpGate }` — vantadb-node/src/lib.rs:35-39.
- Todas las ops son async sobre `tokio::task::spawn_blocking` con `engine.clone()` (el SDK dice VantaEmbedded es Clone, confirmado en src/sdk/builder.rs:14), nunca bloquea el hilo JS — lib.rs:278-287.
- Boundary de I/O = `serde_json::Value`: inputs parseados a mano, outputs serializados — lib.rs:12-15.
- Guard FFI: `MAX_VEC_DIM = 10_000` — lib.rs:25.
- connect mapea `""`/`:memory:` a `BackendKind::InMemory`, y lee read_only/memory_limit — lib.rs:289-317.
- `OpGate` (Condvar/Mutex) = barrera de durabilidad: `close()` rechaza ops nuevas y drena las in-flight para que un put fire-and-forget no se pierda tras el close — lib.rs:197-275.

*Persistencia*

fjall + WAL + fsync nativo (features fjall, memmap2, rayon), o in-memory para `:memory:`. Esto es el **diferencial central vs WASM**: lib.rs:1-6 ("the native .node module gives Node.js real filesystem persistence (fjall/WAL/fsync), which WASM cannot provide"). La prueba que lo demuestra: vantadb-node/tests/persistence.test.ts:49-71 (persistencia cross-close/reconnect).

*Instalación/uso desde Node*

`npm install` en vantadb-node/ y build previo `napi build` (o descargar el .node precompilado por target). Uso: `import { VantaDb } from "vantadb-node"` → `await VantaDb.connect(path)` (tests/persistence.test.ts:6,29).

**2. vantadb-ts/ — wrapper TypeScript sobre WASM (backend adicional: napi)**

*Naturaleza*

Wrapper TS sobre el build WASM vantadb-wasm (wasm-bindgen). Paquete npm `vantadb` v0.5.0:
- vantadb-ts/package.json:1-5 — name `vantadb`, main: dist/vantadb.js, types: dist/vantadb.d.ts.
- vantadb-ts/package.json:46-48 — dependencia runtime única: `vantadb-wasm: file:../vantadb-wasm/pkg`.
- vantadb-ts/package.json:49-57 — vantadb-node es **SOLO devDependency** (no se publica como dep del paquete).
- dist/ NO está construido (solo src/, examples/, node_modules/).

*API pública*

Dos clases exportadas (vantadb-ts/src/vantadb.ts:914-924):
- `VantaDB` (WASM, síncrono): connect(path?)/create(config?)/open(path) + CRUD + search + graph completo + mantenimiento + export/import + text index + IQL (query) + generateSnippet. Firmas exactas en src/vantadb.ts y tabla en README.md:61-137.
- `NativeVantaDB` (napi, async): src/native.ts:80-283 — `static async connect(path = ":memory:", options?)`, close, flush, capabilities, put, putBatch, get, delete, listNamespaces, list, search. Solo este subconjunto isomórfico.
- Types: src/types.ts — MemoryInput, MemoryRecord, SearchRequest, SearchHit, NodeRecord, GraphBfsResult, GraphDfsResult, GraphTopologicalSortResult, OperationalMetrics, Capabilities, etc.

*Cómo elige backend*

No hay auto-selección. El usuario elige explícitamente:
- `import { VantaDB } from "vantadb"` → WASM (browser/Node/Bun/Deno), persistencia in-memory/OPFS/IDB.
- `import { NativeVantaDB } from "vantadb"` → carga dinámica `await import("vantadb-node")` (src/native.ts:112); si el .node no está para la plataforma, lanza `VantaError` con código `NATIVE_ERROR` — src/native.ts:75-79,116-121. "Browsers should use the WASM wrapper instead."

*Persistencia*

WASM = in-memory por defecto; OPFS/IDB solo en browser (vantadb-wasm/src/lib.rs:5-7,19-33,66). La persistencia real en archivos solo llega vía `NativeVantaDB`/vantadb-node.

**3. Relación entre ambos**

Isomórficos en el subconjunto cubierto; no es que vantadb-ts use vantadb-node como backend por defecto. Son dos backends separados del mismo SDK Rust:
- vantadb-node = backend adicional nativo a WASM, API isomórfica con vantadb-ts/src/vantadb.ts — vantadb-node/src/lib.rs:1-6.
- ADR: "backend ADICIONAL — no reemplazo" — docs/architecture/adr/COMP-029-napi-rs-node-bindings.md:8-10.
- Ambos envuelven `VantaEmbedded::open_with_config` (src/sdk/builder.rs:91) con la misma semántica de records/search, cambiando solo el transporte (serde_json vía FFI vs wasm-bindgen).

**4. ¿DB embebida en el proceso?**

**Sí, en ambos casos.**
- vantadb-node: el addon .node se carga en el proceso Node.js (napi-rs FFI). La crate vantadb se compila DENTRO del cdylib; el VantaEmbedded vive en el mismo proceso/espacio de memoria que el host. Las ops corren en threads del tokio runtime embebido en el addon (vantadb-node/src/lib.rs:283).
- vantadb-ts (WASM): el módulo WASM corre en el mismo runtime JS.
- Implicación: hay un solo proceso con la DB dentro — no hay servidor, no hay socket. Matar el proceso mata la instancia (los datos persisten por fjall en disco).

**5. Evaluación Tauri (Rust) — punto clave**

**Tauri no puede `require()` un addon napi-rs** (no existe runtime Node en el proceso Rust). Opciones reales:

- **Opción A (recomendada): crate `vantadb` directa en src-tauri/.**
  - Todo lo que expone index.d.ts es un wrapper delgado de serde_json sobre `VantaEmbedded` (nada de lógica extra en Rust del addon — vantadb-node/src/lib.rs:86-163). Por tanto la API se replica 1:1 en Rust de forma trivial: `VantaEmbedded::open_with_config(VantaConfig)` (src/sdk/builder.rs:91), guardarlo con `tauri::Builder::manage()` y exponer `#[tauri::command]` async. Además el crate da **más** que el addon: graph (BFS/DFS/toposort, src/sdk/graph.rs:50-110), IQL (src/sdk/api.rs:1037), export/import (src/sdk/serialization/impl_export.rs:121-251), count, similar_to_key, delete_by_filter (src/sdk/api.rs:1125,1193,1237) — que el addon NO expone.
  - Esta es exactamente la conclusión del informe existente: docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md:104-140 y su recomendación final :178-184 ("SÍ — Tauri v2 … vantadb como dependency directa en src-tauri/, VantaEmbedded en managed state, commands async delgados. Sin bridge WASM ni OPFS"). Esfuerzo MVP estimado: ~8-13 días (:157-172).
- **Opción B: spawn de proceso Node sidecar.**
  - Empaquetar un binario Node + vantadb-node como sidecar (plugin shell de Tauri) y comunicarse por stdio/IPC. Útil solo si se quiere la paridad exacta con el SDK TS sin reescribir commands — pero añade un runtime Node al bundle (contra el espíritu de Tauri) y es estrictamente inferior a A porque A es el mismo código Rust sin capa FFI.
- **Opción C: WASM en el webview (vantadb-ts).**
  - Funciona para demo in-memory, pero la persistencia queda limitada a OPFS/IDB del browser — no FS real. No recomendable para memoria local durable (DESKTOP-01:16-17 confirma que vantadb-ts es wrapper WASM).

*Veredicto por módulo:*

| Módulo | Rol en Tauri |
|---|---|
| vantadb-node | Solo vía sidecar Node (Opción B); el addon .node no es cargable desde Rust. Su valor es replicar lo que el crate ya hace mejor. |
| vantadb-ts | Solo en el frontend webview (WASM), persistencia limitada; o como referencia de API para diseñar los commands. |

**6. Ops soportadas (resumen de firmas)**

- vantadb-node (napi, async): connect, flush, close, put, putBatch, get, delete, list, listNamespaces, search, capabilities — vantadb-node/index.d.ts:7-62. No expone graph, IQL, export/import ni mantenimiento.
- vantadb-ts WASM (sync): todo lo anterior + searchVector, explainSearch, graph completo (insertNode, getNode, deleteNode, addEdge, graphBfs, graphDfs, graphTopologicalSort, graphIsDag), mantenimiento (compactWal, purgeExpired, rebuildIndex, compactLayout, operationalMetrics), export/import (exportNamespace, exportAll, importRecords, importFile), text index (auditTextIndex, auditTextIndexDeep, repairTextIndex), query(IQL), generateSnippet — vantadb-ts/src/vantadb.ts:213-911. Nota: `reindexHnswFromText` lanza `WASM_ERROR` (no disponible en el build WASM actual) — :542-548.
- vantadb-ts native (napi, async): subconjunto isomórfico — vantadb-ts/src/native.ts:110-282.

**7. Docs existentes**

- docs/architecture/adr/COMP-029-napi-rs-node-bindings.md — ADR de los bindings nativos (decisión, consecuencias, verificación 3/3 tests).
- docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md — investigación Tauri (recomienda Opción A).
- docs/Backlog.md — entradas COMP-029 (l.281) y DESKTOP-01 (l.166).
- docs/progreso/README.md:309-319 — bitácora 2026-08-02 de COMP-029.
- docs/api/TS_SDK.md + docs/master-index.md:62 — docs del SDK TS.
- vantadb-ts/README.md — API completa; vantadb-node no tiene README propio.
- llms.txt — ecosistema (9 SDKs: Python, TypeScript, Rust, WASM, MCP, REST, LangChain, LlamaIndex, DSPy).

**Conclusión práctica para la app Tauri:** ignora vantadb-node y vantadb-ts como vía de integración directa; usa la crate `vantadb` como dependency del backend Rust de Tauri — es exactamente el mismo motor que ambos envuelven, sin capas intermedias. Los dos módulos JS quedan como referencia de API para diseñar los `#[tauri::command]` (vanta_ingest, vanta_search, etc.).

### A.3 Reporte de Investigación: vantadb-wasm/ y vantadb-python/

Investigación completada (solo lectura, nada modificado). Todos los paths son absolutos. Citas con `file_path:line`.

**MÓDULO 1: vantadb-wasm/**

*1.1 Naturaleza y build*

Crate Rust vantadb-wasm que compila a WebAssembly vía wasm-bindgen y expone VantaDB a JavaScript/TypeScript. **No es un wrapper del core: incrusta el motor completo (VantaEmbedded) dentro del binario WASM.**
- Cargo.toml (vantadb-wasm\Cargo.toml:2-12): `crate-type = ["cdylib", "lib"]`, version workspace (0.5.0), wasm-pack con wasm-opt -Oz (:14-18).
- Dependencia del core (vantadb-wasm\Cargo.toml:21): `vantadb = { path = "../", default-features = false, features = ["wasm"] }`. La feature `wasm` del core es vacía (Cargo.toml:107 del repo raíz: `wasm = []`) — simplemente desactiva las features por defecto (cli, fjall, rocksdb, etc.) para no arrastrar deps no compilables a wasm32.
- Features del crate (vantadb-wasm\Cargo.toml:36-38):
  - `default = ["tracing-wasm"]` — logging a console.
  - `opfs = []` — habilita el módulo worker.rs (Web Worker bridge). **No está activada en el pkg/ precompilado** (ver 1.4).
- Estructura (vantadb-wasm\src\): lib.rs (API wasm_bindgen, 1281 líneas), opfs.rs (persistencia OPFS), idb.rs (persistencia IndexedDB con bridge JS inline), worker.rs (bridge Web Worker, gated por opfs), opfs_bridge.js (helpers JS importables desde Rust).
- Tests (vantadb-wasm\tests\wasm_tests.rs:1-14): solo corren en browser (wasm-pack test --chrome), run_in_browser.
- Precompilado (vantadb-wasm\pkg\): vantadb_wasm.js + .d.ts + .wasm + package.json v0.5.0 (pkg\package.json:5). vantadb-ts/ es el wrapper TS sobre este build (vantadb-ts\src\native.ts:61-64 documenta que el wrapper WASM es vantadb.ts y que el backend nativo de Node (vantadb-node, napi-rs) es el que da persistencia real fjall/WAL — el WASM no).

*1.2 API pública expuesta al JS*

Tipo principal: `#[wasm_bindgen] pub struct VantaDB { inner: VantaEmbedded, opfs: Option<OpfsStorage>, #[cfg(feature="opfs")] worker: Option<OpfsWorkerProxy> }` (vantadb-wasm\src\lib.rs:247-253). Firmas completas en vantadb-wasm\pkg\vantadb_wasm.d.ts.

*Inicialización (4 constructores)*
- `new(config_val?: any)` — config {storage_path, read_only, rss_threshold, memory_limit} (lib.rs:260-275).
- `static open(path: string): VantaDB` — sincrónico (lib.rs:278-292).
- `static connect_persistent(path: string): Promise<VantaDB>` — abre motor + OPFS y hace load() del snapshot (lib.rs:295-312).
- `static connect_idb(path: string): Promise<VantaDB>` — IndexedDB como fallback (lib.rs:315-331).
- `static connect_worker(path: string): Promise<VantaDB>` — OPFS vía Web Worker, solo con feature opfs (lib.rs:334-372); requiere importar opfs_bridge.js que expone spawnOpfsWorker() (opfs_bridge.js:66-85).

Métodos de persistencia: `save()`, `load()`, `save_idb()`, `load_idb()`, `delete_idb()` (lib.rs:436-490), más worker_read/write/delete (solo feature opfs, lib.rs:376-399).

CRUD memoria: `put(input: object)` (lib.rs:507-529), `put_batch(inputs: array)` (lib.rs:532-570), `get(namespace, key)` (lib.rs:573-580), `delete(namespace, key): boolean` (lib.rs:583-585), `list(namespace, options)` con cursor (lib.rs:594-614), `list_namespaces()` (lib.rs:588-591).

Búsqueda: `search(request: {namespace, query_vector, filters, text_query, top_k, distance_metric, explain})` (lib.rs:640-669; distancia: "Cosine" default, "Euclidean" alternativo, lib.rs:649-652), `search_vector(vector: Float32Array, top_k)` (lib.rs:672-692), `explain_memory_search(request)` (lib.rs:695-723).

Grafo: `insert_node(id: bigint, content, vector, fields)` (lib.rs:858-886), `get_node(id: bigint)` (lib.rs:889-893), `delete_node(id, reason)` (lib.rs:896-898), `add_edge(source, target, label, weight, created_at_ms)` (lib.rs:902-919), `graph_bfs/graph_dfs(roots: BigUint64Array, max_depth, direction)` (lib.rs:922-969), `graph_topological_sort` (lib.rs:972-979), `graph_is_dag` (lib.rs:982-985).

Import/Export: `import_records(records)`, `import_file(path)`, `export_namespace(path, ns)`, `export_all(path)`, `bulk_import(path)`, `bulk_import_bytes(Uint8Array)` (lib.rs:726-776).

Mantenimiento/diagnóstico: `rebuild_index()`, `reindex_hnsw_from_text(ns, page_size)`, `compact_layout(): bigint`, `flush()`, `compact_wal()`, `purge_expired(): bigint`, `audit_text_index(ns?)`, `audit_text_index_deep(ns?)`, `repair_text_index()`, `operational_metrics()`, `capabilities()`, `query(q)`, `generate_snippet(payload, query, highlight)`, `close()` (lib.rs:779-996).

Tipos: u64/u128 se serializan como **String** para evitar pérdida de precisión JS (JsNodeRecord, lib.rs:126-159; memory_record_to_js convierte timestamps/version/node_id a strings, lib.rs:1013-1061). Límites: vector máx 10M dims (lib.rs:36), batch máx 100k (lib.rs:37), export máx 1M registros (lib.rs:255).

*1.3 Persistencia: OPFS / IndexedDB / Worker*

**Punto clave: el motor interno siempre se abre como InMemory** — `build_config()` fuerza `backend_kind: BackendKind::InMemory` (vantadb-wasm\src\lib.rs:66). **NO hay WAL ni fjall en WASM.** La "persistencia" es un snapshot JSON completo del estado, serializado a un único archivo `db_state.json`:
- `save()`: `collect_all_deduped()` recorre todos los namespaces con list paginado (páginas de 10k, lib.rs:402-433), serializa a JSON (`serde_json::to_vec`) y escribe `db_state.json` (lib.rs:436-445).
- `load()`: lee el JSON y hace `import_records()` (lib.rs:475-490). Mismo patrón para IDB (lib.rs:448-472).

**OPFS** (vantadb-wasm\src\opfs.rs): acceso vía `navigator.storage.getDirectory()` → `getDirectoryHandle(name, {create:true})` (opfs.rs:153-165). Escritura atómica: archivo temp + move() (rename) + footer CRC-32 de 4 bytes para detectar corrupción (opfs.rs:172-186, verificación :193-214). Operaciones de archivo vía FileSystemFileHandle (getFile/createWritable/arrayBuffer, opfs.rs:46-97). Detección de disponibilidad: `navigator.storage` presente (opfs.rs:246-255).

**IndexedDB** (vantadb-wasm\src\idb.rs): bridge JS inline (no requiere import externo) registrado en `globalThis.vantaIdbStorage` (idb.rs:5-78). DB "VantaDB", store "state" (idb.rs:8-9). Sincroniza cross-tab con BroadcastChannel("vantadb-sync") y coordina escrituras con Web Locks API (`navigator.locks`, idb.rs:40-46, 60-66). Feature detection: `IdbStorage::is_available()`, `has_broadcast_channel()`, `has_web_locks()` (idb.rs:113-138).

**Worker** (vantadb-wasm\src\worker.rs, solo feature opfs): el I/O de OPFS se mueve a un thread dedicado. Protocolo postMessage con MessageChannel por petición (worker.rs:224-285), timeout 5000ms con 2 reintentos y backoff exponencial (worker.rs:27, 168-222). Mensajes: Init/Read/Write/Append/Delete (worker.rs:30-44). El worker JS es creado por `spawnOpfsWorker()` desde un Blob URL (opfs_bridge.js:66-85).

*Limitaciones de la persistencia WASM:*

1. **Snapshot completo, no WAL**: cada save() serializa TODOS los registros a JSON. O(DB) por guardado, no incremental. Sin fsync garantizado, sin durabilidad punto-a-punto.
2. **Cap de 1M de registros** para exportar (lib.rs:420); un DB mayor no puede persistirse vía save().
3. El estado persiste solo en el **sandbox del origen** (OPFS/IDB son por-origen del navegador). No hay ruta real al filesystem.
4. OPFS requiere browser moderno (Chrome 86+, Edge 86+, Firefox 111+, Safari 15.2+ — vantadb-wasm\demo\README.md:21). **WebKit (WKWebView) tiene soporte histórico incompleto**; en Tauri la disponibilidad depende del webview del SO.
5. El build precompilado `pkg/` **NO incluye el worker**: verifiqué que pkg\vantadb_wasm.js no contiene connect_worker/spawnOpfsWorker (grep sin coincidencias), y pkg\vantadb_wasm.d.ts no declara connect_worker/worker_read/worker_write/worker_delete (compare contra lib.rs:334-399). Para usar worker hay que recompilar con `--features opfs` y servir opfs_bridge.js.

*1.4 Evaluación para una app desktop Tauri*

**Cuándo NO tiene sentido:** para la mayoría de casos Tauri, usar vantadb-wasm en el webview es lo peor de ambos mundos:
- El backend Rust de Tauri ya tiene acceso al filesystem real (via `app_handle.path().app_data_dir()`) — pasar por OPFS/IDB es usar un sandbox emulado dentro de un sandbox real, con datos difíciles de ubicar, respaldar o compartir con otros procesos (docs\Investigaciones\DESKTOP-01-tauri-plataforma-desktop.md:134-136).
- Persistencia por snapshot JSON O(DB): inviable con DB grandes; en Rust nativo la persistencia es WAL incremental + fjall.
- Costo de serialización WASM↔JS en el hot path de búsqueda, vectores pasando por serde_wasm_bindgen (hay un comentario ponytail documentando el overhead de ~2-5µs por vector, lib.rs:1070-1073).
- El webview de Tauri (WebView2 en Windows, WKWebView en macOS, WebKitGTK en Linux) no garantiza OPFS ni Workers en todas las plataformas.

**Cuándo SÍ tiene sentido:** solo si el objetivo fuera reusar la misma UI web (mismo frontend) para targets web y desktop, o si se quisiera un modo "demo" incrustado en el webview. El propio repo ya documenta esta conclusión: la investigación docs\Investigaciones\DESKTOP-01-tauri-plataforma-desktop.md:180-182 recomienda explícitamente la vía nativa Rust sin bridge WASM ni OPFS (y vantadb-ts\src\native.ts:61-64 confirma que el backend WASM no puede dar persistencia real — por eso existe vantadb-node).

**MÓDULO 2: vantadb-python/**

*2.1 Naturaleza y build (PyO3)*

Crate `vantadb_py` que expone el motor embebido a Python in-process vía PyO3 (sin red, sin servidor).
- Cargo.toml (vantadb-python\Cargo.toml:10-12): `[lib] name = "vantadb_native"`, `crate-type = ["cdylib"]` (extensión CPython).
- Deps (vantadb-python\Cargo.toml:14-21): pyo3 0.29 con extension-module, abi3-py311, chrono; y el core: `vantadb = { path = "../", default-features = false, features = ["fjall", "memmap2", "rayon"] }`.
- Build (vantadb-python\pyproject.toml:1-3,37-42): maturin, `module-name = "vantadb_py"`. Wheels precompilados presentes: `dist\vantadb_py-0.4.0-cp311-abi3-win_amd64.whl` (y 0.1.5). El .pyd compilado está en `vantadb_py\vantadb_py.pyd`.
- ⚠️ **Aclaración sobre "feature python_sdk": hay DOS capas PyO3 en el repo.**
  1. `vantadb-python/` (moderna, la que usas): crate separado que **NO usa la feature python_sdk** — usa `["fjall", "memmap2", "rayon"]` (Cargo.toml:21).
  2. `src/python.rs` (legacy): bindings antiguos en el core gated por `#![cfg(feature = "python_sdk")]` (src\python.rs:1), feature definida en Cargo.toml:104 (`python_sdk = ["pyo3"]`). Expone una clase `ClientEngine` mínima (new(), execute(query), insert_node(id, vec), src\python.rs:13-66) — obsoleta y distinta. El doc docs\Investigaciones\DESKTOP-01-tauri-plataforma-desktop.md:15 afirma que vantadb-python usa la feature python_sdk; **eso es inexacto** — la integración real es el crate `vantadb_py` contra el SDK público VantaEmbedded.
- Test de frontera: tests\api\python.rs (declarado en Cargo.toml:397-399 como `python_sdk_boundary`) valida la capa legacy ClientEngine, no la nueva.

*2.2 API pública (clase VantaDB)*

Definida en vantadb-python\src\lib.rs:74-76 como `pub struct VantaDB { engine: VantaEmbedded }`. Firmas exactas en vantadb_py\vantadb_py.pyi (generado) y vantadb_py\__init__.pyi.

Constructor (lib.rs:113-146):
```
VantaDB(db_path: str, memory_limit_bytes: int|None = None,
        read_only: bool = False, backend: str|None = None)
```
- `backend`: "rocksdb" | "memory" | None→Fjall (default persistente); desconocido → warning y fallback Fjall (lib.rs:122-133).
- `db_path=":memory:"` o `""` → in-memory.
- Función módulo alternativa: `connect(path, memory_limit=None)` (lib.rs:1565-1581).
- Exporta además: `VantaVector`, `VantaVectorIter`, `VantaSearchHit`, `VantaMemoryRecord`, `VantaListResult`, `__version__` (lib.rs:1586-1595). `AsyncVantaDB` es un wrapper Python puro vía `asyncio.to_thread` con semáforo de concurrencia (vantadb_py\__init__.py:24-44, 57-77).

Métodos (firmas resumidas de vantadb_py\vantadb_py.pyi:28-144)

| Grupo | Métodos |
|---|---|
| Memoria CRUD | `put(namespace, key, payload, metadata=None, vector=None, ttl_ms=None) -> VantaMemoryRecord` (lib.rs:589-615) · `get_memory(ns, key) -> VantaMemoryRecord\|None` (lib.rs:644-658) · `delete_memory(ns, key) -> bool` (lib.rs:685-690) · `list_memory(ns, filters=None, limit=100, cursor=None) -> VantaListResult` (lib.rs:726-760) |
| Batch | `put_batch(entries\|keys=..., vectors=..., payloads=..., metadatas=..., namespace=..., ttls=...) -> list` (lib.rs:227-371; tuplas posicionales deprecadas) · `put_batch_raw(vectors_2D_numpy, keys, ...) -> list` — zero-copy PyBuffer f32/f64 (lib.rs:383-544) |
| Búsqueda | `search_memory(ns, query_vector, filters=None, text_query=None, top_k=10, distance_metric=None, explain=False) -> list[VantaSearchHit]` (lib.rs:807-856) · `explain_memory_search(ns, qv, ...) -> dict` (lib.rs:1513-1555) |
| Nodo/grafo | `insert(id: u128, content, vector, fields=None)` (lib.rs:176-202) · `get(id) -> dict\|None` (lib.rs:1119-1126) · `delete(id, reason="manual deletion")` (lib.rs:1131-1140) · `search(vector, top_k=10) -> list[(id, dist)]` (lib.rs:1149-1171) · `search_batch(vectors, top_k) -> list[list[(id, dist)]]` (Rayon paralelo, lib.rs:1180-1209) · `add_edge(source_id, target_id, label, weight=None, created_at_ms=None)` (lib.rs:1301-1318) · `graph_bfs/graph_dfs(roots, max_depth=999999, direction="Forward")` (lib.rs:1342-1378) · `graph_topological_sort, graph_is_dag, graph_page_rank, graph_degree_centrality` (lib.rs:1384-1451) · `recover_archived_nodes(summary_id)` (lib.rs:1472-1483) |
| Import/Export | `export_namespace(path, ns)`, `export_all(path)`, `import_file(path)`, `bulk_import(path)`, `bulk_import_bytes(bytes)` (lib.rs:943-1053) |
| Mantenimiento | `rebuild_index()`, `reindex_hnsw_from_text(ns, page_size=1000)`, `flush()`, `compact_wal()`, `purge_expired() -> int`, `compact_layout() -> int`, `audit_text_index(ns=None, deep=False)`, `repair_text_index()`, `query(iql) -> str`, `generate_snippet(...)`, `capabilities()`, `hardware_profile()`, `operational_metrics()`, `close()`, `list_namespaces()` (lib.rs:885-1555) |

**GIL**: prácticamente todos los métodos envuelven la llamada en `py.detach(...)` (GIL released durante I/O e índices), p. ej. lib.rs:613, lib.rs:845, lib.rs:1193.

Tipos devueltos: VantaMemoryRecord/VantaSearchHit son `#[pyclass]` con getters tipados y `__getitem__` (sin PyDict en el hot path, vantadb-python\src\types.rs:46-156, 274-363). Vectores: numpy array si disponible, si no `VantaVector` (wrapper zero-copy, types.rs:86-97).

*2.3 Persistencia real embebida (fjall/WAL/fsync)*

A diferencia del WASM, aquí el motor es el verdadero VantaEmbedded con backend persistente:
- Config por defecto `BackendKind::Fjall` (vantadb-python\src\lib.rs:122-133). Fjall es el default global del core (src\backend.rs:94-99; src\config.rs:484-492).
- El backend fjall persiste con `PersistMode::SyncAll` = **fsync de datos + metadata**, la garantía más fuerte de Fjall (src\backends\fjall_backend.rs:220-223; "SyncAll = fsync(data + metadata)" en :15).
- El WAL propio de VantaDB hace `sync_data()` (fsync) al flush (src\wal.rs:340-343, 400). `VantaConfig.force_fsync` fuerza fsync en cada operación (src\config.rs:61).
- Layout on-disk verificado en vantadb-python\test_smoke\: `.vanta.lock` (lock de proceso), `0.jnl` (journal fjall), `keyspaces\N\{current,vN}` (versiones), `keyspaces\N\tables\N` (fjall LSM), `data\vanta.wal` (WAL), `data\vector_store.vanta`, `data\vector_index.bin` (HNSW). El lock multi-proceso (`read_only=True` para acceso concurrente, lib.rs:50-53) está documentado en el docstring del constructor.
- `close()` hace flush + cierre (lib.rs:1321-1324); `flush()` garantiza durabilidad antes de shutdown (lib.rs:1230-1233).

*2.4 ¿Es la API Python un subconjunto de la API Rust vantadb?*

**Sí — la API Python es un wrapper 1:1 delgado y un subconjunto estricto de `VantaEmbedded`.** Cada método Python delega directo en el método homónimo del SDK Rust: `use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, ...}` (lib.rs:15-18) y llamadas como `engine.put(input)`, `engine.search(request)`, `engine.graph_bfs(...)`. El SDK Rust completo está en src\sdk\ (VantaEmbedded en src\sdk\builder.rs:91; superficie en src\sdk\api.rs, src\sdk\search\mod.rs, src\sdk\graph.rs, src\sdk\gds.rs; re-exports en src\sdk\mod.rs:13-28; doc en docs\api\EMBEDDED_SDK.md).

Métodos Rust que Python **NO** expone (subconjunto → pérdida de superficie): `import_records` (en-memoria; Python solo tiene import_file), `remove_edge`, `delete_by_filter`, `count`, `similar_to_key`, `search_multi`, `search_all`, `vacuum`, `pipeline`, `optimizer_config`/`set_optimizer_config`, `graph_bfs_filtered`/`graph_dfs_filtered`, accumuladores de grafo (`graph_create_accumulator` etc.), snapshots (`create_snapshot`/`list_snapshots`), threads/mensajes (`create_thread`/`send_message`/`get_thread`/`list_threads`/`delete_thread`/`purge_expired_threads`), `graphrag_search`, y todos los `debug_*`. Todos en src\sdk\api.rs:72-1393, src\sdk\graph.rs:12-134, src\sdk\gds.rs:15-32, src\sdk\builder.rs:139-242, src\sdk\search\mod.rs:1390-1446.

WASM vs Python: WASM añade `import_records`, `insert_node`/`get_node`/`delete_node` explícitos, `bulk_import_bytes` (Python sí lo tiene); Python añade `put_batch_raw` numpy zero-copy, `page_rank`, `degree_centrality`, `recover_archived_nodes`, `hardware_profile`. Ambos fijan `query_sparse: None` (Python no pasa sparse; WASM lib.rs:656).

*2.5 Docs existentes*

- `vantadb-python\README.md` — **DESACTUALIZADO**: usa métodos que no existen (`put_memory` línea 33, `search_hybrid` línea 48, `memory_stats` línea 59). No usarlo como referencia.
- `docs\api\PYTHON_SDK.md` — correcto y completo (746 líneas): constructor, connect(), API memoria (put, put_batch kw vs posicional deprecado, get_memory, delete_memory, list_memory, search_memory, explain_memory_search), API nodo/grafo, mantenimiento, VantaMemoryRecord/VantaSearchHit/VantaListResult, AsyncVantaDB, error handling. Funciones no expuestas documentadas en :369-376 (delete_by_filter, similar_to_key, count).
- `docs\api\EMBEDDED_SDK.md` — referencia Rust VantaEmbedded que es la fuente de verdad de la API.

*2.6 Evaluación para una app desktop Tauri*

- **Opción A — Subproceso Python (spawn):** técnicamente posible pero es la peor vía:
  - No existe ningún CLI/REPL en el paquete — vantadb_py es solo una extensión Python (pyproject.toml no define entry_points, solo el módulo). Un cliente Tauri tendría que escribir un driver script Python que hable JSON sobre stdin/stdout con el proceso Rust de Tauri.
  - Distribución: requiere empaquetar un runtime Python (~40-100MB+ con numpy) + el .pyd correcto por plataforma (solo hay wheel cp311-abi3-win_amd64 local; PYTHON_SDK.md:18 menciona wheels linux/macOS pero no windows-arm). Frágil para instaladores desktop.
  - Latencia y complejidad de IPC innecesarias: el mismo VantaEmbedded que usa el SDK Python es una crate Rust que Tauri ya puede linkear directamente.
- **Opción B — Crate vantadb directa desde Rust (la recomendada, ya documentada en el repo):**
  - docs\Investigaciones\DESKTOP-01-tauri-plataforma-desktop.md:104-141 (arquitectura) y :180-182 (recomendación final): vantadb como dependency en src-tauri/Cargo.toml con features ["fjall", "memmap2"] (o rocksdb), `VantaEmbedded::open_with_config(VantaConfig)` en `tauri::Builder::manage()`, y `#[tauri::command]` async delgados (vanta_ingest/vanta_search) que delegan en el managed state.
  - Como la API Python es subconjunto de la API Rust (sección 2.4), nada de lo que el SDK Python puede hacer se pierde: es la misma VantaEmbedded, sin la capa PyO3.
  - Beneficios concretos: persistencia real fjall/WAL/fsync en el filesystem del usuario (no snapshot, no sandbox), cero overhead de serialización FFI, y acceso a la superficie Rust completa (snapshots, threads, delete_by_filter, count, etc.) que Python no expone.
  - La crate root ya se usa como lib desde vantadb-server, vantadb-mcp, vantadb-wasm y vantadb-python — es el patrón establecido del workspace (Cargo.toml:582-599).

*Resumen ejecutivo*

|  | vantadb-wasm | vantadb-python | Rust directo (vantadb crate) |
|---|---|---|---|
| Motor | VantaEmbedded compilado a wasm32 | VantaEmbedded vía PyO3 cdylib | VantaEmbedded linkeado directo |
| Backend | Siempre InMemory (lib.rs:66) | Fjall (default) / RocksDB / Memory | Fjall / RocksDB / Memory |
| Persistencia | Snapshot JSON db_state.json en OPFS/IDB (cap 1M records, lib.rs:420) | WAL + fjall con fsync (fjall_backend.rs:220) | WAL + fjall con fsync |
| Costo de integración en Tauri | Alto (webview sandbox, snapshots O(DB), OPFS inconsistente en WebKit) | Alto (runtime Python + driver IPC inexistente) | Bajo (managed state + commands) |
| Recomendación | ❌ Solo si se reutiliza la misma UI para web | ❌ Solo si hay requisito de scripting Python | ✅ Vía recomendada (DESKTOP-01:180) |

**Hallazgo adicional importante:** docs\Investigaciones\DESKTOP-01-tauri-plataforma-desktop.md ya existe en el repo y llega exactamente a la misma conclusión que este análisis — integración nativa Rust, sin WASM/OPFS ni Python intermedio.

### A.4 VantaDB Server — Investigación profunda (solo lectura)

**1. Arquitectura: dónde vive el servidor realmente**

vantadb-server/ es una cáscara fina. El binario `vantadb-server` solo hace dispatch; **TODO el servidor HTTP vive en el crate core `vantadb`, módulo `cli_server`**:
- vantadb-server/src/main.rs:26-57 — entrypoint: si el arg es `--mcp` → MCP stdio server (`vantadb_mcp::run_stdio_server`); si no → `vantadb::cli_server::run(config)`.
- vantadb-server/src/server.rs:1-4 y src/middleware.rs:1 — meros re-exports de `vantadb::cli_server`.
- vantadb-server/Cargo.toml:10 — `vantadb = { path = "..", features = ["cli", "server"] }`. El feature `server` del core activa axum/tower-governor/tower-http (root Cargo.toml — `axum = { version = "0.8", optional = true }`).
- src/cli_handlers/server.rs:199-204 — `vanta-cli server --http` sin el feature server falla con "Rebuild with: cargo build --features server".
- src/api/mod.rs:1-4 — stub vacío (solo doc comment). No hay routers ahí.

Protocolo: HTTP REST (axum 0.8) + JSON. No hay gRPC. No hay WebSocket ni SSE en todo src/ (grep de `DefaultBodyLimit|WebSocket|Sse|EventSource|text/event-stream` → 0 resultados). El streaming solo existe vía MCP stdio (JSON-RPC 2.0 por stdin/stdout).

**2. Endpoints reales (los únicos tres, rutas en src/cli_server.rs:126-131)**

| Método | Path | Auth | Descripción |
|---|---|---|---|
| GET | /health | Ninguna (exenta explícitamente en cli_server.rs:261-263) | Liveness: `{"success":true,"data":"OK"}` |
| POST | /api/v2/query | Bearer (si api_key configurada) | Ejecuta una sentencia IQL |
| GET | /metrics | Bearer | Texto Prometheus/OpenMetrics `text/plain; version=0.0.4` (cli_server.rs:364-378) |

Versionado de API: el path es `/api/v2/query` — versión v2 hardcodeada en la ruta (cli_server.rs:129). docs/api/openapi.yaml:4 declara versión API 0.4.0; workspace version = "0.5.0". No hay otros endpoints ni parámetros de query.

**3. Puerto, host, CLI flags, env vars**

Defaults (src/config.rs:406-423):
- Host: `VANTADB_HOST` → fallback `HOST` → `127.0.0.1`
- Puerto: `VANTADB_PORT` → `8080`
- Storage: `VANTADB_STORAGE_PATH` → `vantadb_data`

CLI vanta-cli server (src/cli.rs:286-307 y src/cli_handlers/server.rs:216-224):
```
vanta-cli server --http [--mcp] [-p|--port <u16>] [--host <str>] [--require-auth] [-d|--db <path>]
```
- `cmd_server_http` fuerza `port.unwrap_or(8080)` y `host.unwrap_or("127.0.0.1")` (cli_handlers/server.rs:218-219).

Binario vantadb-server: NO tiene CLI args salvo `--mcp`; configuración 100% vía env vars (vantadb-server/src/main.rs:28 → `VantaConfig::from_env()`).

Env vars relevantes (src/config.rs):

| Env | Default | Línea |
|---|---|---|
| VANTADB_HOST / HOST | 127.0.0.1 | 412-415 |
| VANTADB_PORT | 8080 | 420 |
| VANTADB_STORAGE_PATH | vantadb_data | 407-409 |
| VANTADB_API_KEY | none (modo dev sin auth) | 533-536 |
| VANTADB_REQUIRE_AUTH | false | 538-541 |
| VANTADB_RATE_LIMIT_RPM | 100 (0 = desactivado) | 543-546 |
| VANTADB_MAX_CONNECTIONS | max_blocking_threads * 2 | 502-505 |
| VANTADB_POOL_ACQUIRE_TIMEOUT_MS | 5000 | 507-510 |
| VANTADB_CIRCUIT_BREAKER_FAILURE_THRESHOLD | 5 | 512-515 |
| VANTADB_CIRCUIT_BREAKER_OPEN_TIMEOUT_SECS | 30 | 517-520 |
| VANTADB_TLS_CERT / VANTADB_TLS_KEY | none | 580-588 |
| VANTADB_LOG_FORMAT | compact (json\|full) | 590-599 |
| VANTADB_MAX_BLOCKING_THREADS | núcleos×2 | 497-499 |
| VANTA_LLM_URL / VANTA_LLM_MODEL / VANTA_LLM_SUMMARIZE_MODEL | http://localhost:11434, all-minilm, llama3 | 424-439 |
| VANTA_BACKEND | fjall (rocksdb\|memory) | 480-495 |

**4. Formato request/response**

Request (cli_server.rs:52-56):
```
{ "query": "<sentencia IQL>" }
```
Content-Type `application/json` obligatorio. Body JSON malformado → 400 (test e2e.rs:301-315).

Response envelope (QueryResponse, cli_server.rs:58-71):
```
{
  "success": true,
  "data": "Read 1 nodes.",
  "node_id": 42,          // solo writes / stale-context; null en reads multi-nodo
  "nodes": [              // solo reads; null en writes
    { "id": 1, "semantic_cluster": 0,
      "relational": { "key": "memory-1", "namespace": "agent/main" },
      "hits": 3, "confidence_score": 0.95 }
  ]
}
```
NodeDTO (cli_server.rs:74-98): id: u128, semantic_cluster: u32, relational: BTreeMap<String, FieldValue>, hits: u32, confidence_score: f32. FieldValue es un enum tipado (src/node.rs:880-903): String, Int(i64), Float(f64), Bool, DateTime, ListString/ListInt/ListFloat/ListBool/ListDateTime, Null.

Errores HTTP vs errores de ejecución:
- **HTTP status code 200 incluso cuando la query falla** — el fallo viaja en el body como `success: false, data: "Execution Error: ..."` (cli_server.rs:513-519). ⚠️ Importante para el cliente.
- Status codes reales: 400 (JSON inválido), 401 (token faltante/incorrecto, con hint, cli_server.rs:339-350), 403 (RBAC, cli_server.rs:324-335), 429 (rate limit auth-failure por IP, 5 fallos/60s, cli_server.rs:231 y governor), 503 + Retry-After (pool saturado cli_server.rs:440-453; circuit breaker abierto cli_server.rs:404-417), 500 (panic del task, cli_server.rs:466-478).
- Sin WWW-Authenticate estándar (solo body hint), sin CORS (ver §8).

**5. Operaciones soportadas (IQL — las 6)**

El endpoint único POST /api/v2/query ejecuta IQL (parser Nom en src/parser/mod.rs). Sentencias (parser/mod.rs:220-316, doc docs/api/IQL.md):
1. Query/search: `FROM <entity> [SIGUE <min>-<max> "<label>"] WHERE <cond> AND <cond> FETCH <f> RANK BY <f> [DESC] WITH TEMPERATURE <f> ROLE "<r>"`
2. Vector/hybrid: `WHERE <field> ~ "<text>", min = <score>` (embeddings vía LLM local)
3. Insert: `INSERT NODE#<id> TYPE <t> { <k>: <v>, ... } [VECTOR [x,y,z]]`
4. Update: `UPDATE NODE#<id> SET <field> = <v>, ... [SET VECTOR [...]]`
5. Delete: `DELETE NODE#<id>`
6. Graph: `RELATE NODE#<a> --"<label>"--> NODE#<b> [WEIGHT <n>]` y `INSERT MESSAGE <SYSTEM|USER|ASSISTANT> "<content>" TO THREAD#<id>`

ExecutionResult (src/executor.rs:17-31): `Read(Vec<UnifiedNode>)`, `Write { affected_nodes, message, node_id }`, `StaleContext(u128)` (confianza crítica → rehidratación disponible). Ejecución híbrida vía `Executor::execute_hybrid` (cli_server.rs:459-462).

**6. Ejemplos reales encontrados en el repo (curl y tests)**

Del doc oficial docs/api/HTTP_API.md:38-108 y docs/api/IQL.md:184-189:
```bash
# Health (sin auth)
curl http://127.0.0.1:8080/health

# Metrics
curl http://127.0.0.1:8080/metrics

# Query con auth (read)
curl -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{"query": "(memory:get \"agent/main\" \"memory-1\")"}'

# Hybrid search
curl -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{"query": "FROM memory WHERE text ~ \"neural network\", min = 0.75 FETCH text, score RANK BY score DESC"}'
```

Tests E2E reales sobre sockets (vantadb-server/tests/e2e.rs):
- Insert → POST /api/v2/query body `{"query": "INSERT NODE#101 TYPE Test { content: \"e2e-http\" }"}` → espera node_id == 101 (e2e.rs:108-123)
- Read → `{"query": "FROM Test FETCH content"}` (e2e.rs:126-139)
- Delete → `{"query": "DELETE NODE#101"}` (e2e.rs:141-155)
- Auth sobre HTTP: sin token → 401; `Authorization: Bearer e2e-secret` → 200; token erróneo → 401 (e2e.rs:158-202)
- Persistencia entre reinicios del server (e2e.rs:204-259)

Tests unitarios con `Authorization: Bearer` (vantadb-server/tests/server.rs:86-128) y TLS end-to-end con reqwest (server.rs:405-495). Benchmarks de inserción concurrente en vantadb-server/tests/benchmarks.rs:60-66.

**7. Cómo se levanta (3 vías)**

1. `cargo build --bin vantadb-server` (workspace member, Cargo.toml root members), luego ejecutar con env vars (VANTADB_STORAGE_PATH, VANTADB_PORT, VANTADB_API_KEY...). Sin `--http` flag — es el modo por defecto; `--mcp` cambia a MCP stdio (vantadb-server/src/main.rs:27).
2. `vanta-cli server --http [--port] [--host] [-d <db>] [--require-auth]` — requiere feature `server` en el build del core (cli_handlers/server.rs:188-224). Es el path documentado (docs/api/HTTP_API.md:120-130).
3. Docker: imagen `vantadb/server:latest` con args `server --http --port 8080 -d /data` (docs/operations/DEPLOYMENT_GUIDE.md:246-256).

Nota: `vanta-cli server --mcp` (sin --http) spawnea el binario vantadb-server como subproceso con `--mcp` y env vars (cli_handlers/server.rs:227-298). En `server --http --mcp` ambos modos coexisten.

**8. Autenticación / seguridad**

- **Bearer token**: `Authorization: Bearer <VANTADB_API_KEY>`, comparado en tiempo constante (`subtle::ConstantTimeEq`, cli_server.rs:299-312).
- Sin api_key configurada → **modo dev**: el middleware deja pasar todo (cli_server.rs:280-282). Solo /api/v2/query y /metrics están protegidos; /health siempre público.
- `--require-auth` / VANTADB_REQUIRE_AUTH: arranque falla con error claro si no hay key (cli_server.rs:714-729).
- **RBAC opcional** (RbacConfig, cli_server.rs:116-124, 314-338): roles admin (Admin), writer (Read+Write), reader (Read); mapeo token→rol en token_role_map (config, sin env var dedicada — solo API). Writes = POST/PUT/PATCH/DELETE.
- **Rate limiting doble**: governor por IP (burst = rpm/10, cli_server.rs:133-147) + limitador de fallos de auth 5/60s por IP (cli_server.rs:231).
- **TLS opcional** (feature tls): rustls TLS 1.2+1.3, ALPN h2,http/1.1 (cli_server.rs:845-854); sin certs → cae a HTTP plano con warning (cli_server.rs:692-696). Encriptación en reposo opcional `VANTADB_ENCRYPTION_KEY` (feature encryption, config.rs:336-341).
- **CORS: NO hay headers CORS** (docs/api/HTTP_API.md:148-150 — recomiendan reverse proxy). Irrelevante para Tauri nativo, relevante si se usa webview con fetch.
- Default bind `127.0.0.1` = loopback, no expuesto externamente.

**9. Concurrencia, límites de payload, streaming**

- **Concurrencia**: semáforo ConnectionPool (src/connection_pool.rs:47-73), max_connections default = max_blocking_threads × 2 (≈ núcleos×4). Pool saturado → 503 + Retry-After: 1 (cli_server.rs:440-453). Cada query corre en spawn_blocking (cli_server.rs:459).
- **Circuit breaker**: 5 fallos consecutivos (status ≥500) → abierto 30s → 503 + Retry-After (cli_server.rs:399-426, src/circuit_breaker.rs).
- **Payload**: NO hay DefaultBodyLimit custom → límite por defecto de axum 0.8 = **2 MB por body**. Sin límite de longitud de query explícito en el handler HTTP.
- **Streaming**: NO existe SSE ni WebSocket en el server HTTP. Las queries son request/response bloqueantes (el LLM/embeddings se ejecutan en el handler; una query pesada puede tardar sin streaming). El "streaming" de VantaDB es solo el modo MCP stdio.
- **MCP (modo alternativo)**: límites McpConfig (vantadb-mcp/src/lib.rs:22-64): payload 1 MB, key 512 B, namespace 256 B, vector dim 16384, query 1 MB, timeout 60 s, concurrency 32 (o max_blocking_threads del engine), list limit 10 000, top_k 1000. Tools: memory_put/get/delete/list/list_namespaces, query_lisp, search_semantic, search_memory, get_node_neighbors, inject_context, read_axioms, collection_stats/list/delete, rehydrate (vantadb-mcp/src/lib.rs:808-964).

**Otras 4 integraciones** — todas embebidas in-process, NO clientes HTTP: vantadb-node (addon napi-rs nativo, vantadb-node/package.json:4), vantadb-python (PyO3, docs/api/EMBEDDED_SDK.md:12), vantadb-ts (WASM, vantadb-ts/README.md:3 "WASM-powered embedded"), vantadb-wasm (in-browser). Ninguna habla el protocolo HTTP del server; no sirven como modelo para conectar una app desktop a un server remoto — sí para embeder el engine.

**10. Recomendación para la app desktop Tauri**

1. **Vía de integración**: la única opción de red es el HTTP REST de vantadb-server. (MCP sería subproceso, no red.)
2. **Sidecar**: empaquetar vantadb-server como sidecar de Tauri (`cargo build --bin vantadb-server` con features server + opcionalmente tls/prometheus). Al arrancar la app: spawnear el binario con env `VANTADB_STORAGE_PATH=<app-data-dir>`, `VANTADB_PORT=18080` (puerto local fijo o efímero), `VANTADB_API_KEY=<generada>` y `VANTADB_REQUIRE_AUTH=true`.
3. **Cliente**: reqwest (rustls) desde el backend Rust de Tauri — NO desde el webview (evita CORS y expone la key). Base URL `http://127.0.0.1:18080` (o https si se activa TLS). Health-poll hasta 200, luego operar.
4. **Concurrencia**: mantener ≤ VANTADB_MAX_CONNECTIONS (default ≈ núcleos×4) requests concurrentes; ante 429/503 respetar Retry-After; ante 500/breaker abierto, backoff exponencial. Considerar `VANTADB_RATE_LIMIT_RPM=0` para desactivar el governor en uso local de un solo cliente.
5. **Semántica de errores**: tratar `success:false` del body como error de dominio aunque el status sea 200; 400 = JSON malformado.
6. **Sin streaming**: para UX "en vivo" (logs, progreso), usar polling de /metrics o queries periódicas; no hay SSE.
7. **Auth por defecto**: como el server enlaza 127.0.0.1 por defecto, HTTP plano local es aceptable, pero igual setear VANTADB_API_KEY + --require-auth como defensa en profundidad (otras apps locales podrían golpear el puerto).
8. **Alternativa embebida**: si la desktop no necesita servidor externo, vantadb-ts (WASM) o vantadb-node (napi) se pueden usar in-process desde el lado JS de Tauri — sin puerto, sin auth, sin límites de red; útil para modo offline.

Fuentes clave: src/cli_server.rs (todo el server HTTP), src/config.rs (env/defaults), src/cli.rs + src/cli_handlers/server.rs (CLI), docs/api/HTTP_API.md + docs/api/openapi.yaml + docs/api/IQL.md (documentación), vantadb-server/tests/{server,e2e,benchmarks}.rs (ejemplos reales), vantadb-mcp/src/lib.rs (MCP), vantadb-server/Cargo.toml + root Cargo.toml (features/workspace).

### A.5 Validación rápida vanta-arch (entorno)

Validación rápida completada (workspace del repo, binarios, rmcp, estructura). Confirmo lo relevante para el diseño: `vantadb-server`/`vantadb-mcp`/`vantadb-wasm` son experimentales y **NO default-members**; `vantadb-node`/`vantadb-ts` son standalone; **no existe src-tauri/ ni desktop/ aún**; el frontend vive en `web/` (React+Vite). **rmcp 3.x** valida `client` + `transport-child-process` (TokioChildProcess con spawn + graceful_shutdown con kill-timeout).

### A.6 VantaDB Desktop — Arquitectura Tauri v2 Multi-Connection (reporte íntegro vanta-arch)

**Autor:** vanta-arch · **Estado:** Propuesto · **Relacionado:** docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md · **Gama de tareas:** DESKTOP-02..DESKTOP-27

**Architecture Decision**

Construir `desktop/` como app Tauri v2 (frontend React+Vite reutilizando el design system de web/, backend Rust en src-tauri/) con un **ConnectionManager** que abstrae las 6 vías de integración (nativa, HTTP server, MCP stdio, Node sidecar, Python driver, WASM/TS webview) detrás de un trait común `VantaConnection`, con la vía nativa (crate `vantadb` embebido) como default y las demás como conectables a elección del usuario, soportando varias simultáneas bajo la regla "un escritor por path de DB".

**Rationale:** DESKTOP-01 concluyó que la vía nativa es la óptima (latencia, sin subprocesos, WAL real). Pero el valor del producto es la compatibilidad universal: un usuario puede tener un vantadb-server corriendo, querer conectar Claude Desktop vía MCP, o tener datos en el addon Node/Python. Un trait único + adaptadores por vía da esa flexibilidad sin duplicar la lógica de comandos y UI.

**Alternativas consideradas:**
1. Solo vía nativa (recomendada por DESKTOP-01) — rechazada: no satisface el requisito de 6 vías.
2. Solo HTTP como capa de indirección (la app habla HTTP con el server y punto) — rechazada: añade un proceso extra y latencia para el caso nativo que es el 80% del uso; el server no soporta graph API completa ni el mismo contrato que la crate.
3. Un trait síncrono + spawn_blocking — considerado; descartado porque rmcp/HTTP/subprocesos son naturalmente async. Se usa `async_trait` (Box<dyn> object-safe) y spawn_blocking internamente en el adaptador nativo (VantaEmbedded es síncrono).
4. WASM como conexión Rust "proxy" (comandos Rust redirigidos al webview) — rechazado por over-engineering: WASM es demo/read-only; el frontend lo usa directo en JS y Rust solo registra metadata + devuelve `Unsupported`.

**Impact Analysis**
- **Modules affected:** nuevo directorio desktop/ (self-contained, NO toca workspace raíz); cero cambios en vantadb, vantadb-server, vantadb-mcp, vantadb-node, vantadb-python, vantadb-ts, vantadb-wasm salvo que una tarea de empaquetado necesite publicar binarios.
- **Concurrency model:** async_trait connections; parking_lot::Mutex/RwLock en ConnectionManager (registry frío, ops por conexión); subprocesos hijos (MCP/Node/Python) con stdio piped; spawn_blocking para VantaEmbedded.
- **Durability guarantee:** la que dé cada vía (nativa/MCP/Node/Python = WAL+fsync real de fjall; HTTP = la del server; WASM = snapshot JSON). El trait expone `capabilities()` con nivel de durabilidad para que la UI advierta.
- **Backward compatibility:** aditivo 100%. El único toque al repo raíz sería ninguno (el `[workspace]` vacío en src-tauri/Cargo.toml desacopla del workspace padre automáticamente).
- **Send+Sync:** el trait object `Box<dyn VantaConnection + Send + Sync>`; los estados internos (child handles) se guardan en Arc<Mutex<_>> por conexión.

**1.3 Elección de vía**
- **Default:** vía nativa (crate) — la más rápida, sin procesos extra, WAL/fsync real. Es la que se conecta automáticamente al arrancar si hay config guardada.
- **Selector por usuario:** la UI permite conectar/desconectar cualquiera de las 6 y marcar una activa (a la que delegan los comandos sin connection explícito). La elección se persiste en config.

**1.6 Estado de conexión y ciclo de vida**
- Estados: `Disconnected → Connecting → Connected → Error (con detalle) → Disconnected`. Cada transición emite `vanta://connection-state`.
- Healthcheck: comando explícito + sondeo perezoso (solo cuando la UI lo pide o antes de una op); MCP usa una tool trivial, HTTP un GET de health, nativos capabilities().
- Shutdown: `ConnectionManager::shutdown_all()` en `RunEvent::ExitRequested` con orden: webview (wasm) → subprocesos (MCP graceful_shutdown, Node/Python close+kill) → nativa última (flush). Timeout configurable; si se excede → kill forzoso de hijos (constraint 8 del agente).

**2.2 Commands Tauri (firmas completas)**

| Command | Firma (params → Result) | Delega a |
|---|---|---|
| vanta_connect | (kind: ConnectionKind, name: String, config: ConnectionConfig) → ConnectionInfo | manager.connect |
| vanta_disconnect | (connection: Option\<String\>) → () | manager.disconnect |
| vanta_list_connections | () → Vec\<ConnectionInfo\> | registry |
| vanta_set_active | (connection: String) → () | manager |
| vanta_health | (connection: Option\<String\>) → HealthReport | adaptador activo |
| vanta_ingest | (item: IngestItem, connection: Option\<String\>) → () | adaptador activo |
| vanta_ingest_batch | (items: Vec\<IngestItem\>, connection: Option\<String\>) → IngestReport | adaptador activo |
| vanta_search | (query: SearchQuery, connection: Option\<String\>) → Vec\<SearchResult\> | adaptador activo |
| vanta_get | (key: String, namespace: Option\<String\>, connection: Option\<String\>) → Option\<MemoryRecord\> | adaptador activo |
| vanta_delete | (key: String, namespace: Option\<String\>, connection: Option\<String\>) → bool | adaptador activo |
| vanta_list | (namespace: Option\<String\>, limit: u32, offset: u32, connection: Option\<String\>) → Vec\<MemoryRecord\> | adaptador activo |
| vanta_stats | (connection: Option\<String\>) → ConnectionStats | adaptador activo |
| vanta_register_webview | (meta: WebviewConnectionMeta) → () | manager (WASM) |
| vanta_config_get / vanta_config_set | () → AppConfig / (patch: serde_json::Value) → AppConfig | ConfigStore |

Keys/namespaces siempre String (nunca &str), errores VantaError (Serialize).

**3. Frontend (detalle)**
- Stack: React + Vite (mismo que web/), reutilizando el design system y tokens de web/ (verificado: web/package.json es el frontend existente; la tarea DESKTOP-07 valida la ruta exacta de tokens). `@tauri-apps/api` (invoke + event).
- Bridge: `src/lib/vanta.ts` — wrapper tipado de invoke que espeja los comandos; hook `useConnectionState` suscribe a vanta://connection-state.
- Componentes mínimos MVP:
  - `<ConnectionPanel>` — lista de conexiones con health badge por vía (estado por eventos).
  - `<ConnectionSelector>` — selector de vía activa + formulario de config por vía (path, url/puerto/token) + warning de conflicto de path (por la regla 1-escritor).
  - `<IngestForm>` — key/text/vector/namespace, batch simple.
  - `<SearchBar>` + `<ResultsList>` — query, top_k, resultados con score/snippet.
  - `<ConfigModal>` — persistir vías y activa.
- Estado: React context local (ConnectionContext); sin librería de estado (YAGNI).

**4. Plan de tareas DESKTOP — tabla completa (archivos clave, esfuerzo, deps, DoD)**

Regla de oro: 1 tarea = 1 concepto; ninguna mezcla dos integraciones. Las fases 2-4 son por vía. `desktop/` NO toca el workspace raíz (el `[workspace]` vacío en src-tauri/Cargo.toml lo desacopla; sin cambios al manifest raíz ni a CI de core).

*Fase 0 — Scaffold*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-02 | Scaffold Tauri v2 + propio workspace | create-tauri-app en desktop/; src-tauri/Cargo.toml con `[workspace]` vacío (desacopla del raíz); tauri.conf.json, capabilities mínimas, comando ping; frontend React+Vite mínimo. | desktop/src-tauri/*, desktop/package.json | 🟢 | — | `npm run tauri dev` abre ventana y el botón ping responde; `cargo check` en src-tauri pasa; `cargo check` raíz sigue igual (no hay cambios en workspace). |
| DESKTOP-03 | Integrar crate vantadb + managed state + healthcheck | Dep vantadb con default-features=false + fjall,fs2,memmap2,roaring,advanced-tokenizer; AppState { manager, config } managed; command vanta_health que abre VantaEmbedded en temp dir y reporta capabilities. | src-tauri/Cargo.toml, src/lib.rs, src/commands/connection.rs | 🟢 | 02 | vanta_health devuelve HealthReport con backend=fjall; abrir dos veces el mismo path falla con error de lock. |

*Fase 1 — Trait + adaptador nativo + UI mínima*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-04 | Trait VantaConnection + tipos + errores | async_trait, tipos compartidos (IngestItem/SearchQuery/SearchResult/MemoryRecord/HealthReport/ConnectionInfo/Capability), VantaError unificado (#[non_exhaustive]). | src/connections/{trait,types}.rs, src/error.rs | 🟢 | 03 | Compila; tests unitarios de serde roundtrip de todos los tipos. |
| DESKTOP-05 | NativeConnection | VantaEmbedded embebida, ops síncronas en spawn_blocking, mapeo de errores, capabilities(), lock del path. | src/connections/native.rs | 🟢 | 04 | Test integración: put/search/get/delete en temp dir vía trait; segunda conexión mismo path → VantaError::Lock. |
| DESKTOP-06 | Commands CRUD async | vanta_connect/disconnect/list_connections/set_active/ingest/ingest_batch/search/get/delete/list delegando al adaptador activo (por ahora solo nativo). | src/commands/{connection,data}.rs | 🟡 | 05 | E2E manual: conectar nativo, ingest 3, search devuelve resultados ordenados. |
| DESKTOP-07 | Frontend MVP | Scaffold React+Vite en desktop/ reusando tokens de web/; ConnectionPanel, IngestForm, SearchBar, ResultsList, hook useConnectionState; bridge vanta.ts. | desktop/src/* | 🟡 | 06 | UI permite conectar nativo, ingresar y buscar; badge de health. |

*Fase 2 — Adaptador Server (HTTP)*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-08 | HTTP client tipado | Wrapper reqwest (json): config url/port/token/timeout; métodos por endpoint de docs/api/HTTP_API.md (health, put, get, delete, list, search) — validar rutas/auth reales contra el doc (requisito del agente: validar APIs, no asumir). | src/connections/server_client.rs | 🟢 | 04 | Tests contra mock HTTP server (axum en dev-deps): cada endpoint mapeado y autenticado. |
| DESKTOP-09 | ServerConnection | Implementa el trait sobre el client; connect valida auth/health; mapeo a VantaError::Http; timeouts. | src/connections/server.rs | 🟢 | 08 | Integración contra vantadb-server real: health/put/search ok; server caído → error Http limpio. |
| DESKTOP-10 | Wire Server en commands + UI | Selector muestra vía "Server" con campos url/puerto/token; conexión entra al registry y puede ser activa. | src/commands/connection.rs, desktop/src/components/ConnectionSelector.tsx | 🟢 | 09, 07 | Desde la UI: conectar a server real, ingest + search por HTTP. |

*Fase 3 — Adaptador MCP (stdio)*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-11 | Spawn manager subproceso MCP | Localiza el binario (vantadb-server — dev: target/debug/vantadb-server.exe; release: bundled); validar flag MCP real en vantadb-server/src/main.rs; tokio::process::Command con stdio piped, stderr a log, timeout de arranque. | src/connections/child_process.rs | 🟢 | 04 | Spawn + kill limpio; arranque con flag MCP confirmado; stderr capturado. |
| DESKTOP-12 | Cliente rmcp | Dep rmcp (client,transport-child-process); TokioChildProcess + ClientInfo::serve; init handshake, list_tools, call_tool con params serde_json. | src/connections/mcp_client.rs | 🟢 | 11 | Conecta al binario real; list_tools devuelve las 15 tools. |
| DESKTOP-13 | McpConnection | Mapea las 15 tools al trait (tabla 1.5); VantaError::Mcp; session por conexión. | src/connections/mcp.rs | 🟡 | 12 | Integración real: ingest + search vía MCP en temp dir; tool inexistente → error mapeado. |
| DESKTOP-14 | Healthcheck/reconnect/UI MCP | Healthcheck por tool trivial; close = graceful_shutdown con kill-timeout; selector "MCP" en UI. | src/connections/mcp.rs, ConnectionSelector.tsx | 🟢 | 13, 07 | Desconectar mata el proceso (verificado); reconectar funciona. |

*Fase 4 — Node y Python (opcionales)*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-15 | JSON-RPC cliente + driver Node | Framing newline-delimited, ids incrementales, mapa de respuestas pendientes, timeout (connections/jsonrpc.rs, compartido con Python); driver drivers/node/driver.js que require('vantadb_native') y sirve stdio. | src/connections/jsonrpc.rs, drivers/node/driver.js | 🟡 | 04 | `node driver.js` responde put/search por stdio (test manual con pipe). |
| DESKTOP-16 | NodeConnection | Spawn node (dev: sistema; release: sidecar externalBin), IPC vía jsonrpc, mapeo VantaError::Node, capabilities. | src/connections/node.rs, tauri.conf.json | 🟡 | 15 | Integración: put/search vía node en temp dir. |
| DESKTOP-17 | Driver Python + decisión runtime | drivers/python/python_driver.py (import vantadb_py, JSON-RPC stdio); decisión documentada: runtime python del sistema para MVP vs bundled embeddable (se decide aquí, no antes). | drivers/python/python_driver.py | 🟡 | 04 | Driver responde put/search con python local (si disponible en CI, se testea; si no, skip documentado). |
| DESKTOP-18 | PythonConnection | Spawn python + driver, reusa jsonrpc.rs, mapeo VantaError::Python. | src/connections/python.rs | 🟡 | 17 | Integración opcional (skippable sin runtime python). |

*Fase 5 — ConnectionManager multi-connection*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-19 | ConnectionManager completo | Registry multi, active, path_holders (1-escritor por path → VantaError::Lock + hint), routing por connection id, capability gate (write sobre read-only → Unsupported). | src/connections/manager.rs | 🟡 | 05, 09, 13 | Nativa + Server (paths distintos) conectadas simultáneamente; conectar segunda vía sobre el mismo path → rechazada con hint. |
| DESKTOP-20 | Lifecycle shutdown_all | shutdown_all en RunEvent::ExitRequested: orden webview → subprocesos → nativa última (flush); timeout configurable + kill forzoso. | src/lib.rs, src/connections/manager.rs | 🟢 | 19 | Cerrar app con MCP+Node+Python conectados no deja procesos huérfanos (verificado). |
| DESKTOP-21 | UI multi-connection | Selector con N vías conectadas, switch de activa, health badge por vía, warning de conflicto de path. | ConnectionSelector.tsx, ConnectionPanel.tsx | 🟡 | 19, 07 | UI muestra 2 vías vivas; la op va a la activa; warning al intentar conflicto. |

*Fase 6 — Streaming, config, empaquetado, CI, tests, docs*

| ID | Título | Descripción | Archivos clave | Esf. | Deps | DoD |
|---|---|---|---|---|---|---|
| DESKTOP-22 | Eventos Tauri (streaming) | vanta://connection-state (obligatorio) + ingress-progress/search-progress (flag progress); listeners en frontend. | src/lib.rs, desktop/src/hooks/* | 🟢 | 19 | Ingest batch de 1000 items emite progreso sin bloquear la UI. |
| DESKTOP-23 | Persistencia de config | JSON en app_config_dir, load en setup, save atómico ante cambios, defaults, vías guardadas. | src/config.rs | 🟢 | 19 | Reiniciar la app conserva vías guardadas y vía activa. |
| DESKTOP-24 | Empaquetado | Bundle NSIS/MSI (Windows primero), externalBin (node + vantadb-server.exe + runtime python si procede), identifier, icons, auto-update opcional. | tauri.conf.json, src-tauri/build.rs | 🔴 | 16, 18, 23 | Instalador produce una app que conecta nativo + server + node sin entorno de dev. |
| DESKTOP-25 | CI GitHub Actions | Build Windows (tauri-action), cargo test en src-tauri (workspace desacoplado), npm build frontend, artefacto instalador; matrix por features de vías (con/sin server/mcp). | .github/workflows/desktop.yml | 🟡 | 24 | Pipeline verde; artefacto instalador subido. |
| DESKTOP-26 | Tests | Unit: tipos, mapping de errores, framing jsonrpc; integración por adaptador (mock HTTP, MCP real, nativa temp); contrato de errores: misma op en N vías → mismo shape VantaError. | src/**/*_tests.rs | 🟡 | 19 | cargo test + integraciones en CI. |
| DESKTOP-27 | Docs + ADR | README desktop, ARCHITECTURE.md (modelo conexión), ADR (multi-connection + regla 1-escritor; siguiente número libre en docs/architecture/adr/), guía de usuario por vía, actualizar DESKTOP-01 con decisiones. | docs/desktop/*, docs/architecture/adr/ADR-0XX.md | 🟢 | 19..26 | ADR revisado por vanta-arch; guía cubre las 6 vías. |

**Total: 26 tareas (DESKTOP-02..DESKTOP-27) — 13 🟢, 10 🟡, 1 🔴, 2 condicionales (16/18).**

**5. Riesgos técnicos (verbatim vanta-arch)**
 1. **Lock de archivo de la DB (1-escritor por path).** Fjall toma lock exclusivo (fs2) al abrir. Nativa/MCP/Node/Python/Server compiten por el mismo path. Mitigación: path_holders en el manager (DESKTOP-19) + warning en UI (DESKTOP-21) + hint en VantaError::Lock. Multi-escritura real = paths distintos por vía.
 2. **CORS/seguridad del HTTP server.** El server puede requerir auth; la app debe mandar token y validar el modelo de auth real en HTTP_API.md (DESKTOP-08). Nunca exponer el puerto a interfaces no-loopback por defecto; documentar en guía.
 3. **Sidecar Node.** Empaquetar runtime node + addon .node (ABI napi por plataforma) + versionado es frágil. Mitigación: dev usa node del sistema; release usa externalBin; el addon se descarga/empaqueta como artefacto de build (DESKTOP-16/24). Si el peso es inaceptable → feature-gate de la vía.
 4. **Driver Python.** Empaquetar runtime python + wheel por plataforma es pesado. Mitigación: MVP con python del sistema + decisión explícita en DESKTOP-17 (bundled embeddable o sistema); la vía es opcional y se desactiva si no hay runtime.
 5. **rmcp cliente stdio.** Gestión de subproceso, timeouts de arranque, stderr. `TokioChildProcess::graceful_shutdown` ya implementa close + wait con kill-timeout (validado en docs.rs); capturar stderr a log para diagnóstico (DESKTOP-11). Flag MCP exacto a confirmar contra vantadb-server/src/main.rs.
 6. **WASM/TS.** Solo demo/read-only (persistencia snapshot, cap 1M). Rust no puede invocar el webview → la conexión vive en JS; comandos Rust devuelven Unsupported y la UI usa la vía directa. Health se registra con vanta_register_webview.
 7. **Async commands Tauri.** Limitación conocida: en comandos async no se puede usar &str ni &State<'_, _> (el future no es 'static). Mitigación: keys/namespaces como String, state como `tauri::State<'_, AppState>` (owned wrapper sobre Arc<ConnectionManager>), errores VantaError: Serialize.
 8. **Workspace raíz y CI.** vantadb-server/mcp/wasm son experimentales y NO default-members; meter src-tauri al workspace rompería esa política. Mitigación: `[workspace]` vacío en src-tauri/Cargo.toml → crates desacopladas automáticamente; CI de desktop en workflow propio (DESKTOP-25). No activar features pesadas de vantadb (cli/server) en la dependencia de la app.
 9. **Tiempo de build.** vantadb + fjall es pesado; src-tauri usa su propio perfil optimizado y queda fuera del CI de core (mitigado por 8). Perfil dev del repo (opt-level=1, debug=0) aplica igualmente.
10. **Durabilidad heterogénea.** Cada vía tiene distinta garantía (WAL real vs snapshot JSON vs la del server). El trait expone `capabilities().durability`; la UI muestra el nivel por conexión para que el usuario no asuma sync donde no lo hay.

**ADR (inline — persistir en DESKTOP-27 como docs/architecture/adr/ADR-0XX.md)**
- Estado: Aceptado (propuesta) · Fecha: 2026-08-04 · Área: desktop
- Contexto: DESKTOP-01 identificó la vía nativa como óptima, pero el producto debe poder conectarse a las 6 vías (crate, HTTP server, MCP stdio, Node napi, Python PyO3, WASM) individualmente o en simultáneo.
- Decisión:
  1. Backend Tauri v2 en desktop/ con src-tauri desacoplado del workspace raíz ([workspace] vacío) para no afectar CI/versiones de core.
  2. Trait VantaConnection (async, object-safe con async_trait) + adaptador por vía; ConnectionManager como único punto de entrada de comandos.
  3. Vía nativa (crate vantadb embebida) como default; selector de vía por usuario; N conexiones simultáneas con la invariante "un escritor por path de DB".
  4. WASM/TS se modela como conexión de webview (read-only, metadata en Rust, ops en JS) — sin proxy Rust.
  5. Persistencia de config en app_config_dir; eventos Tauri para estado y progreso; shutdown ordenado con kill-timeout.
- Consecuencias: + compatibilidad universal y testabilidad por vía; − complejidad de mantenimiento de 6 adaptadores y de empaquetado de runtimes (Node/Python) que queda detrás de feature-gates; − necesidad de educar al usuario sobre la regla 1-escritor.
- **Siguiente paso recomendado:** aprobar este documento, asignar números de ADR y arrancar por DESKTOP-02/03 (scaffold + crate) en una sola sesión de vanta-worker, con vanta-lead revisando el desacople del workspace y vanta-audit el manejo de subprocesos/unsafe de los sidecars al llegar a las fases 3-4.

## Referencias

- `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` — investigación Tauri base (recomienda vía nativa)
- `docs/Backlog.md` Phase 12 — tareas DESKTOP-02..27
- `docs/api/HTTP_API.md`, `docs/api/MCP.md`, `docs/api/IQL.md`, `docs/api/EMBEDDED_SDK.md`, `docs/api/PYTHON_SDK.md`, `docs/api/TS_SDK.md`, `docs/api/openapi.yaml`
- `docs/architecture/adr/COMP-029-napi-rs-node-bindings.md`, `docs/architecture/adr/003_sync_async_decoupling.md`
- `vantadb-server/src/main.rs`, `src/cli_server.rs`, `src/config.rs`, `vantadb-mcp/src/lib.rs`, `vantadb-node/src/lib.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`
