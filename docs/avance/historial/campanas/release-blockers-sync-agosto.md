# Release blockers y syncs (jul-agosto) + AUDIT-01/02

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-29 — Index Rebuild Optimization (4/4 tareas + 3 WI) ✅

**Fuente:** Plan `docs/plans/2026-07-29-index-rebuild-execution.md` — archivado en `docs/plans/archive/2026-07-29-index-rebuild-execution.md`.

**Objetivo:** Implementar Propuesta 1b (incremental threshold) + 3 (layer-wise) + 4 (flatten) de INDEX_REBUILD_OPTIMIZATION.md; NN-Descent diferido (fase posterior).

- **T1+: `InsertMode::Auto` + `incremental_threshold` en `BatchInsertOptions`** — `put_batch()` decide incremental vs rebuild por tamaño de chunk (`039b8c96`, `dad5b2fd`).
- **T2: Tests incremental** — `src/storage/engine/tests/incremental.rs` (8 tests: small/large batch, recall parity) + Criterion bench `benches/incremental_bench.rs` (`d1a6c62c`).
- **T3:** threshold configurable por opciones insert (auto default 1000).
- **T4:** `HnswNeighborIndex` flatten (`src/index/neighbor_index.rs`, DashMap de neighbor lists + inbound reachability) — `f94d71b1`; evolución posterior: inline `neighbor_lists` cache en `HnswNode` (`3f5e8416` E1, `b214434c` E2) tras regresión search_layer +66-90% por cloning.
- **Propuesta 2 (NN-Descent):** probada y revertida — regresión 7-1,300× (`f1b9ee03`).
- **WI-1/2/3:** fix comparador invertido B2, harness hnsw_recall_ef, competitive_bench.py fix.
- **Verificación:** `cargo test -p vantadb --lib index::search` 31 passed; recall harness real ef_400 → 0.9975; search 7× vs flat. Build benchmark pendiente de re-medición limpia (entorno sucio 2026-07-31).
- **Plan archivado:** `docs/plans/archive/2026-07-29-index-rebuild-execution.md` — 4/4 tareas + 3/3 WI completadas.

**Ids:** INDEX-REBUILD (T1-T4), WI-1, WI-2, WI-3

### 2026-08-05 — Sincronización release blockers Fase 3 (10 tareas) ✅

**Fuente:** Backlog (Phase 1/4 + Phase 8 — Auditoría) + plan `docs/plans/2026-08-05-backlog-validation-actions.md` (Fase 2-3, Tasks 20/22-28/31/33)

**Resueltas (commits en develop, wave F3 14:54-15:10):**
- **TECH-03:** corregidos 3 stale-docs reales (claim MCP+HTTP excluyente, API python real, tool `query`→`query_lisp`/`query_iql`) — `8530da3e`. *(plan Task 20 ✅)*
- **TECH-06:** CORS cerrada sin consumidor browser real (webview usa reqwest, no fetch); queda feature request futuro — `af812748`. *(plan Task 24 ✅)*
- **TECH-07:** API worker opfs documentada (`connect_worker`/`worker_read/write/delete`) + demo browser `worker-test.html` — `566e9369`. *(plan Task 22 ✅)*
- **TECH-08:** decisión documentada en `CI_POLICY.md:86` — mantener los 3 crates experimental, NO promover a default-members. *(plan Task 23 ✅)*
- **AUDIT-05:** housekeeping 3 fixes (gitignore `.playwright-cli/`, ADR `003` last-updated, task file GH-123 actualizado) — `9487073a`. *(plan Task 25 ✅)*
- **AUDIT-08:** P2 debt ledger refs actualizadas (P2-2/P2-3/P2-7/P2-8) + comentario LRU O(1)→O(n) corregido — `9487073a`. *(plan Task 26 ✅)*
- **AUD-002:** GRAPH_RAG.md reescrito con entrypoint real Rust (`VantaEmbedded::graphrag_search` + `GraphRagPipeline`); Python marcado no-implementado; cita `api-contract.md` actualizada — `05542105`. *(plan Task 27 ✅)*
- **AUD-003:** afirmación de verificación contra `src/governance` inexistente retractada; documento marcado como diseño propuesto — `bbcd3221` + `11d7944f`. *(plan Task 28 ✅)*
- **AUD-007:** ARCHITECTURE.md corregido con nombres de tipo y constantes reales (ef_construction 400→100, `HnswIndex`→`CPIndex`, `WalSharded`→`ShardedWal`) — `4a990366`. *(plan Task 31 ✅)*
- **AUD-009:** nota Vite→Next.js corregida en DESKTOP-01b; resto de menciones Vite correctas (desktop Tauri) no tocadas — `65125e35`. *(plan Task 33 ✅)*

**Ids:** `TECH-03`, `TECH-06`, `TECH-07`, `TECH-08`, `AUDIT-05`, `AUDIT-08`, `AUD-002`, `AUD-003`, `AUD-007`, `AUD-009`

### 2026-08-05 — Sync Docs/Auditoría/Community/Webhook + Marketing (7 tareas) ✅

**Fuente:** Backlog (Phase 5/6/8/11) + plan `docs/plans/2026-08-05-backlog-validation-actions.md` (Tasks 29/30/32/34/51/52/53)

**Resueltas:**
- **AUD-005:** único drift real = openapi.yaml 0.4.0→0.5.0 (MCP.md=0.5.0 correcto, HTTP_API.md 0.0.4 coincide con `cli_server.rs:368`); gate CI de versión contra workspace. *(plan Task 29 ✅)*
- **AUD-006:** 5 tools MCP reales faltantes documentadas (`query_lisp`→`query_iql`, `collection_stats`, `collection_list`, `collection_delete`, `rehydrate`) — 15/15 con nombre real + gate de paridad tool↔doc. *(plan Task 30 ✅)*
- **AUD-008:** STORAGE_VERSIONING.md corregido a constantes reales (VECTOR_INDEX_VERSION=8, VFILE_VERSION=2, WAL postcard), importadas del código; contradicción interna bincode/postcard resuelta. *(plan Task 32 ✅)*
- **GH-123:** claim "167+ archivos" desmentido (341 .md en docs/); ~4 links rotos reales corregidos + método de auditoría documentado (wiki-links `[[..]]` = falsos positivos). Issue #123 cerrado con evidencia del inventario. *(plan Task 34 ✅)*
- **GH-141:** webhook GitHub→Discord documentado en `docs/discord/server-config.md` (4 tipos de evento: push, pull_request, issues, release → #announcements) + procedimiento para añadir eventos. Issue #141 cerrado. *(plan Task 51 ✅)*
- **MKT-16:** metodología benchmark GraphRAG publicada con números reales de un run reproducible (prohibido inventar cifras; ejemplo `examples/rust/graphrag.rs` citado). *(plan Task 52 ✅)*
- **MKT-10:** "AI Agent Memory" campaign rescatada con DoD de deliverables medibles (landing "agent memory" + 1 blog benchmark vs full-context + demo); contenido base tutorial 01-ai-agent-memory + 3 blogs; cubierta por INV-006/BLOG_SERIES_PLAN. *(plan Task 53 ✅)*

**Ids:** `AUD-005`, `AUD-006`, `AUD-008`, `GH-123`, `GH-141`, `MKT-16`, `MKT-10`

### 2026-08-05 — Sincronización release blockers Fase 2 (6 tareas) ✅

**Fuente:** Backlog (Phase 1/4) + plan `docs/plans/2026-08-05-backlog-validation-actions.md` (Fase 2-3)

**Resueltas (commits en develop):**
- **AUDIT-04:** crash benchmark `0xC0000409` atribuido a `cache_warmer.co_access` (OOM 270KB), NO UAF — fix `d2c7b0a5` acota access-time. *(plan Task 14 ✅)*
- **DEBT-01:** gate `validate-docs-coverage.ps1` reparado (ruta `src/sdk/search/` corregida) + 13 gaps de API documentados — `1a0cb79a`.
- **TECH-01:** MCP child respeta `--db` (setea `VANTADB_STORAGE_PATH`) — `9d085d00`; ver ADR-012. *(plan Task 17 ✅)*
- **TECH-02:** wrapper TS `reindexHnswFromText` usa export real del pkg (1-línea, sin rebuild) — `274edcf9`. *(plan Task 18 ✅)*
- **TECH-04:** ADR-012 publicado (`012_env_var_naming.md`) — naming env vars unificado, AUD-010 absorbida. *(plan Task 10 ✅)*
- **TECH-05:** resource MCP `schema://` implementado (list + read) — `4dff484c`.

**Ids:** `AUDIT-04`, `DEBT-01`, `TECH-01`, `TECH-02`, `TECH-04`, `TECH-05`

### 2026-08-05 — Sincronización release blockers Fase 2 wave 2 (3 tareas) ✅

**Fuente:** Backlog (Phase 8 — Auditoría) + plan `docs/plans/2026-08-05-backlog-validation-actions.md` (Fase 2)

**Resueltas (commits en develop):**
- **AUD-001:** Dockerfile reparado — MSRV subido a ≥1.94.1 y 8 `COPY` a crates inexistentes (integración movida a `integrations/`) eliminados — `1ffe523c`. *(plan Task 16 ✅)*
- **AUD-004:** tool MCP `query_lisp` renombrada a `query_iql` (feature LISP eliminada en CUARENTENA-01) + MCP.md actualizado — `f097bac7`. *(plan Task 13 ✅)*
- **AUD-011:** patrón OpGate portado a bindings python/wasm (riesgo write-after-close); `ops.rs:1761` documentado como expect infalible — `ef155f9c`. *(plan Task 15 ✅)*

**Ids:** `AUD-001`, `AUD-004`, `AUD-011`

### 2026-08-05 — AUDIT-01: Fix UAF PyO3 `__array_interface__` (release-blocker) ✅

**Fuente:** Backlog (Phase 1 — Security & Critical) `AUDIT-01`

**Resuelto por (vanta-worker):**
- **Root cause:** getters `__array_interface__` (`vector.rs:59-73`, `types.rs:365-380`) exponían puntero raw zero-copy al `Box<[f32]>` del pyclass; el UAF real se abría con mutación (`__setstate__` libera el buffer viejo con views NumPy vivas → lectura de memoria liberada). `try_numpy_array` COPIA y era seguro — premisa corregida 2026-08-05.
- **Fix:** `get_array_interface` devuelve `PyBytes` (copia little-endian f32, sin `unsafe`); NumPy pinna el snapshot bytes inmutable → nunca aliasa `self.data`. Cubre drop + `__setstate__` + cualquier mutación futura.
- **Tests:** `TestArrayInterfaceMemorySafety` 3/3 (no-alias determinístico, sobrevive drop+hammering 2000 allocs, sobrevive `__setstate__` mutation) + suite `test_sdk.py` 45 passed, 0 regresiones.
- **Verify:** `cargo check -p vantadb_py` ✅; MIRI no operativo en Windows (documentado, alcance Miri re-escalado a AUDIT-03 sobre core).
- Commit `bff30d38` (develop). Deuda P2-2 (raw pointer UB) liquidada — saldo neto negativo.
- **Follow-up:** pickle roto pre-existente (`__module__ == 'builtins'`) — agregar `module = "vantadb_py"` al `#[pyclass]`.

**Ids:** `AUDIT-01`

### 2026-08-06 — AUDIT-02: Sparse hot-path micro-opt (gate de medición) — WONTFIX ✅

**Fuente:** Backlog `AUDIT-02`

**Resuelto por (vanta-tuner):**
- **Premisa corregida:** "sparse_memory_search full-scan" era falsa desde NUEVO-22 (SparseIndex invertido + posting lists ya implementado).
- **Medición (gate):** bench `sparse_hot_path` (criterion, 5.000 docs, ~24 dims/2000 vocab, 5.000 candidatos, top_k=10). Hot-path total 464 ms.
- **Candidato sort:** `sort_hits` (vive en `src/planner.rs:190`, no en search:775) = 0.51% del hot-path; fix `select_nth` ahorra solo 0.31% → < 1%.
- **Candidato serialización-J:** parse por hit = 1.49% cruza umbral nominal, pero eliminarlo exige migración del formato persistido (storage + compat) y ya está indexado como deuda P2-7 — no es diff mínimo.
- **Decisión:** WONTFIX. Mediciones en `docs/research/AUDIT-02-2026-08-06.md`.
- **Verify:** `cargo bench --bench sparse_hot_path --no-run` ✅; no se tocó `src/` → no aplica check/nextest.

**Ids:** `AUDIT-02`
