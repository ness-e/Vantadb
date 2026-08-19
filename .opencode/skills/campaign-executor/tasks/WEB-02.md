# WEB-02 — REST: resto del SDK (export/import, graph avanzado, mantenimiento, threads, snapshots)

> Plan: `docs/plans/2026-08-18-vanta-studio-fase3.md` (Task 3, Wave 1)
> Estado: ✅ COMPLETO (commit c856b3bd - fase 3)
> Pre-requisito: WEB-01 (commit c81bc23a) — patrón de handlers `run_db_op`, helpers `vanta_error_status`/`vanta_error_response`, tests `raw_request`/`json_request`/`parse_response` en `src/cli_server.rs`.

## Contrato (mapeo 1:1 con el SDK `VantaEmbedded`)

Errores: shape `{success:false,error}` con status de `vanta_error_status` (400/404/409/500). Todo engine work bajo pool permit en `spawn_blocking` vía `run_db_op`.

| Endpoint | SDK op | Request → Response |
|---|---|---|
| `POST /api/v2/export` | `export_namespace(path, ns, filter)` / `export_all(path)` | `{path, namespace?, filter?}` → `VantaExportReport` (200) |
| `POST /api/v2/import` | `import_records(Vec<VantaMemoryRecord>)` / `import_file(path)` / `bulk_import_file(path)` | `{records?[] , path?, format?:"jsonl"\|"bulk"}` → report (200); ambos reportes serializados a `serde_json::Value` para unificar T |
| `POST /api/v2/graph/bfs` | `graph_bfs(roots, max_depth, direction)` | `{roots[], max_depth, direction?:"forward"\|"reverse"\|"both"}` → `Vec<u128>` (200) |
| `POST /api/v2/graph/dfs` | `graph_dfs(...)` | igual → `Vec<u128>` |
| `POST /api/v2/graph/degree` | `graph_degree_centrality(roots)` | `{roots[]}` → `HashMap<u128,(usize,usize)>` |
| `POST /api/v2/graph/centrality` | `graph_degree_centrality(roots)` ⚠️ | idem degree — GDS solo expone degree_centrality y page_rank (ver Notas) |
| `POST /api/v2/graph/pagerank` | `graph_page_rank(roots, max_iter, damping, tol)` | `{roots[], max_iterations?=100, damping?=0.85, tolerance?=1e-6}` → `HashMap<u128,f64>` |
| `POST /api/v2/maintenance/purge` | `purge_expired()` | body vacío → `{purged: u64}` |
| `POST /api/v2/maintenance/compact` | `compact_layout()` | body vacío → `{freed_bytes: u64}` |
| `POST /api/v2/maintenance/flush` | `flush()` | body vacío → `{flushed: true}` |
| `POST /api/v2/maintenance/rebuild-index` | `rebuild_index()` | body vacío → `VantaIndexRebuildReport` |
| `POST /api/v2/threads` | `create_thread(title, ttl_secs)` | `{title, ttl_secs?}` → 201 `{thread_id: u128}` |
| `GET /api/v2/threads` | `list_threads(limit, offset)` ⚠️ | query `limit?=100, offset?=0` → `Vec<MessageThread>` (agregado: plan no lo listaba pero SDK lo expone) |
| `GET /api/v2/threads/{id}` | `get_thread(id)` | → `MessageThread` (200) / 404 si no existe |
| `POST /api/v2/threads/{id}` | `send_message(id, role, content)` | `{role, content}` → `{sent: true}`; 404 vía NodeNotFound |
| `DELETE /api/v2/threads/{id}` | `delete_thread(id)` | → `{deleted: true}` |
| `GET /api/v2/snapshots` | `list_snapshots()` | → `Vec<String>` |
| `POST /api/v2/snapshots/{name}` | `create_snapshot(name)` | → 201 `{name, path}` (wire propio — FsSnapshot no es Serialize, created_at es Instant) |

## Fases / Steps

### Fase 1 — DISCOVERY ✅
- [x] Leer plan file + pipeline-full.md
- [x] `codegraph_explore` sobre `cli_server.rs` (patrón run_db_op/vanta_error_status/vanta_error_response)
- [x] Leer `src/cli_server.rs` completo (2177L) + tests existentes
- [x] Mapear superficie real del SDK: api.rs, builder.rs, gds.rs, graph.rs, impl_export.rs, agentic/thread.rs, storage FsSnapshot
- [x] Impacto mapeado (Regla 0) — ver abajo

### Fase 2 — Implementación
- [ ] Step 2.1: imports (`VantaMemoryRecord`, referencias `crate::graph::TraversalDirection`) + tipos wire (ExportRequest, ImportRequest, GraphTraversalRequest, GraphDirection, GraphPageRankRequest, ThreadCreateRequest, ThreadMessageRequest, ThreadsListParams)
- [ ] Step 2.2: rutas nuevas en `app_with_cors` (protected router)
- [ ] Step 2.3: handlers export/import
- [ ] Step 2.4: handlers graph (bfs/dfs/degree/centrality/pagerank)
- [ ] Step 2.5: handlers maintenance (purge/compact/flush/rebuild-index)
- [ ] Step 2.6: handlers threads (list/create/get/send/delete)
- [ ] Step 2.7: handlers snapshots (list/create)
- [ ] Step 2.8: tests `v2_export_import_roundtrip`, `v2_threads_roundtrip`, `v2_graph_roundtrip`, `v2_maintenance_roundtrip`, `v2_snapshots_roundtrip`

### Fase 3 — Verify
- [ ] `cargo fmt`
- [ ] `cargo check --features server`
- [ ] `cargo test -p vantadb --features server --lib -- cli_server`

### Fase 4 — Smoke
- [ ] Arrancar `vanta serve` con DB temp (fjall en disco)
- [ ] `Invoke-RestMethod` por endpoint: export→import roundtrip, graph con nodos+edges, maintenance, threads CRUD, snapshots
- [ ] Devolver RESULTADO al lead

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `src/cli_server.rs` (2177L), `src/sdk/api.rs` (§1480-1599 bulk + grep pub fn), `src/sdk/builder.rs` (§130-246 threads/snapshots), `src/sdk/gds.rs` (37L), `src/sdk/graph.rs` (234L), `src/sdk/serialization/impl_export.rs` (710L), `src/agentic/thread.rs` (327L), `src/graph.rs` (§1-60 TraversalDirection), `src/gds.rs` (§1-120), `src/storage/engine/mod.rs` (§140-219 FsSnapshot, §455-539 create/list_snapshots), `src/sdk/serialization/graph_types.rs` (VantaNodeInput).

**Referencias hacia dentro (dependen de cli_server.rs):** `src/lib.rs:72` (`pub mod cli_server`, `#[cfg(feature="server")]`), `src/cli_handlers/server.rs` (invoca `cli_server::run`/`app`), tests `#[cfg(test)] mod tests` dentro del propio archivo.

**Referencias salientes (lo que cli_server usa):** `VantaEmbedded` (sdk), `ServerState` campos (`storage`, `db`, `circuit_breaker`, `pool`, `api_key`, `rbac_config`, `trusted_proxies`), helpers `run_db_op`/`vanta_error_response`/`vanta_error_status`, `VantaMemoryInput`/`VantaMemoryRecord`/`VantaMemoryFilter`, `crate::graph::TraversalDirection`, `MessageThread` (agentic), `FsSnapshot` (storage).

**Veredicto de impacto:** Editar SOLO `src/cli_server.rs` (agregar handlers + rutas + tests). No se modifica ningún tipo del SDK (`src/sdk/` read-only, protegido). `TraversalDirection` no es Serialize/Deserialize → wire enum local `GraphDirection` en cli_server.rs (no tocar `src/graph.rs`). `FsSnapshot` no es Serialize → respuesta wire manual `{name, path}`. Los handlers son nuevas funciones — no cambia ningún contrato existente (rutas WEB-01 intactas).

## Notas de divergencia (contrato del plan vs SDK real)

1. **`/graph/centrality`** no tiene algoritmo dedicado — `src/gds.rs` expone SOLO `page_rank` y `degree_centrality`. Se mapea a `graph_degree_centrality` (misma op que `/graph/degree`). Documentado; si se quiere betweenness/closeness hay que implementarlo en el core primero.
2. **`GET /api/v2/threads`** agregado (plan solo listaba POST /threads + GET/POST/DELETE /threads/{id}) — `list_threads` es la única vía de descubrimiento del SDK.
3. **`compact_wal()`** existe en el SDK pero no está en el contrato (solo `compact`) → `compact` = `compact_layout()`.
4. **`bulk_import_stream`** no se expone (requiere reader binario) — se expone `bulk_import_file` con `format:"bulk"`.
5. **`create_snapshot` no funciona con backend InMemory** (`data_dir` inexistente) → tests/smoke de snapshots usan DB fjall en disco.
6. **import_records** acepta `Vec<VantaMemoryRecord>` (formato export), no `VantaMemoryInput` — wire 1:1 con el SDK.