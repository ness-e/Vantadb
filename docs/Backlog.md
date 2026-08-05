---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-08-03
verified_by: "Historial de verificación: docs/progreso/BACKLOG_HISTORY.md"
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks — organized by execution order.
> **Execution state lives in:** `docs/plans/YYYY-MM-DD-<campaign>.md` (plan file) + task files — per campaign-executor RULES.md §2. This file is the task catalog; the plan file is the execution state.
> **Completed tasks moved to:** `docs/progreso/README.md`
> **Verification method:** All items cross-checked against actual codebase (Jul 27, 2026). 8 tareas ejecutadas en sesión: TSK-106, MKT-03, NUEVO-21, MKT-04, TSK-107, COM-03, COM-04, Good first issues (18 creadas).
> **Total open items:** ~134 (59 anteriores + 19 investigaciones INV-001..INV-017 + INV-024, -6 items migrados a completado REC-001/REC-010/INV-002/NUEVO-07/INV-019/TSK-104 + 15 GitHub issues GH-119..GH-144 convertidos a backlog Phase 11 + 26 DESKTOP-02..27 + DEBT-01 + 8 TECH-01..08)
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 1 (+6 ✅ completados, +7 removidos, 1 WONTFIX) | ~2-3d | 🔴 Bloqueante |
| **P1** 🛡️ Security & Critical | 0 (+2 INV investigación ✅) | ~3-5d | 🟡 Media |
| **P2** ⚡ Quick Wins Técnicos | 0 (7 ✅ + 24 stale removidos) | — | ✅ Cerrado |
| **P3** 🧪 Test Coverage (adapters) | 0 (7 ✅ + 7 stale removidos) | — | ✅ Cerrado |
| **P4** 🔧 Engineering Health | 4 (+4 INV investigación) | ~1-2 semanas | 🟡 Media |
| **P5** 📖 Docs & Community | 9 (+1 INV investigación) | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 9 (+1 INV investigación) | ~1-2 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 0 (todos completados) | — | ✅ Cerrado |
| **P8** 🔮 Post-Launch & Enterprise | 28 (+10 INV investigación) | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 13 (7 ✅ progreso) | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 20 (11 ✅ progreso) | — | 🗺️ Roadmap |
| **P12** 🖥️ DESKTOP App (Tauri) | 26 (DESKTOP-02..27) | ~3-5 semanas | 🔵 Futuro |

> **Historial de items removidos/completados:** ver `docs/progreso/BACKLOG_HISTORY.md`.
> **Nuevo 2026-08-04:** Fase 12 DESKTOP (26 tareas, app Tauri multi-connection sobre las 6 integraciones) + `DEBT-01` (gate docs-coverage roto, Fase 4) + `TECH-01..08` (hallazgos de investigación DESKTOP-01b: 2 bugs reales, 1 batch stale-docs, 1 ADR env-naming, 4 features/decisiones, todos en Phase 4).

---

## ✅ Definition of Ready / Done

> **DoR + DoD del proyecto (VantaDB-specific) viven en:** `.opencode/references/definition-of-done.md` — secciones "VantaDB — Definition of Ready" y "VantaDB — Project-specific DoD commands". Referencia única; no duplicar aquí.

---

## Phase 0: 🚀 Release Blockers

> Items que bloquean un release público seguro. Resolver antes de cualquier publicación.

| ID | Descripción | Archivos | Esfuerzo | Prio |
| ~~`DEVOPS-15`~~ | ~~**Reducir default features de 7 a 3** — ❌ **WONTFIX**. Analizado: remover `cli, memmap2, fs2, sysinfo` rompe UX "it just works". Las 7 features mantienen experiencia completa. | `Cargo.toml:89` | 🟡 — | ✅ Cerrado |~~
| ~~`META-001`~~ | ~~**🔍 Root Cause Analysis: Inconsistencias del Backlog** — Auditoría profunda. **Entregable:** Reporte de hallazgos.~~ | ~~`docs/audit-reports/meta-001-root-cause-analysis.md`~~ | ~~🟠 2-3d~~ | ✅ |

> **Items removidos (7) + WONTFIX:** ver `docs/progreso/BACKLOG_HISTORY.md` (P0). `META-001` era el único P0 activo.

---

## Phase 1: 🛡️ Security & Critical

> Investigaciones de seguridad y dependencias críticas.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-001`~~ | ~~**🔍 Investigar dependencias con RUSTSEC activas** — Auditar `Cargo.lock` contra advisories conocidos. 3 crates reportadas: `atomic-polyfill` (RUSTSEC-2023-0089), `paste` (RUSTSEC-2024-0436), `rustls-pemfile` (RUSTSEC-2025-0134). **✅ COMPLETADA 2026-07-29 — Las 3 están gestionadas o son stale. Reporte: `docs/audit-reports/inv-001-rustsec-2026-07-29.md`. Cargo deny pasa limpio.**~~ | ~~`Cargo.lock`, `deny.toml`~~ | ~~🟢 2-4h~~ | ✅ |
| ~~`INV-024`~~ | ~~**🔍 Auditar bloqueos `unsafe` sin SAFETY docs** — Revisar todos los bloques `unsafe` en el código Rust (`src/node.rs`, `src/index/graph.rs`, `src/storage/vfile.rs`) que carecen de invariantes documentados. Verificar si hay UB potencial o si son seguros pero sin docs. Proponer: (a) agregar SAFETY comments documentando invariantes, o (b) reemplazar con alternativas seguras. **Sin implementación — solo auditoría + propuesta.** **✅ COMPLETADA 2026-07-30 — 39 bloques auditados (28 SAFE, 4 SAFE_BUT_UNDOCUMENTED, 7 UB_POTENTIAL). 1 High (panic-DoS sq8_similarity) + 1 Medium (UB alineación). Reporte: `docs/audit-reports/inv-024-unsafe-audit-2026-07-30.md`. cargo deny PASSED, cargo audit 0 vulns.**~~ | ~~`src/node.rs`, `src/index/graph.rs`, `src/storage/vfile.rs`, `src/index/search.rs`~~ | ~~🟡 3-5d~~ | ✅ |

| `AUDIT-01` | **?? C1 UAF en PyO3 `__array_interface__`** - El UAF es por los getters `__array_interface__` (`vantadb-python/src/vector.rs:59-73`, `types.rs:365-380`): exponen puntero zero-copy sin retener el objeto origen; el ndarray queda dangling al droppear/reasignar. `try_numpy_array` (`convert.rs:189-203`) COPIA y es seguro. Fix: congelar/clonar ante drop/`__setstate__`/mutación, + test repro. **Bloquea release del Python SDK.** *(premisa corregida 2026-08-05: getters, NO try_numpy_array; hallazgo audit-full 2026-08-04, C1)* | `vantadb-python/src/convert.rs`, `vantadb-python/src/vector.rs`, `vantadb-python/src/types.rs` | 🟡 2-4h | 🔴 Bloqueante |
| `AUDIT-02` | **?? Sparse hot-path per� + top-k heap** - Serialización JSON de sparse en ambos sentidos (`src/sdk/serialization/mod.rs:271-279,335-338`) + `sparse_memory_search` full-scan + `sort_hits` O(N·logN) (`src/sdk/search/mod.rs:721-746`). 1) `BinaryHeap`/`select_nth` (contained). 2) acceso lazy / rep binaria de sparse (toca WAL + `api.rs:751` → signoff Arch/Engine). **Index sparse → vanta-engine.** *(hallazgo audit-full 2026-08-04, F1/F2)* | `src/sdk/serialization/mod.rs`, `src/sdk/search/mod.rs`, `src/planner.rs` | 🟡 4-8h | 🟡 Media |
| `AUDIT-03` | **🛡️ Miri guard sobre el CORE Rust** — `vantadb-python` = 0 dev-deps, 0 tests Rust, cdylib PyO3 → Miri no puede correr FFI CPython/NumPy (premisa re-escalada 2026-08-05). **Alcance:** `cargo +nightly miri test -p vantadb` con `MIRIFLAGS=-Zmiri-tree-borrows` (ya existe `tests/miri_unsafe.rs`) para cubrir los 7 bloques UB_POTENTIAL de INV-024. Boundary Python cubierto con repro Python + ASAN/valgrind (AUDIT-04). **Ejecutar DESPUÉS del fix AUDIT-01.** | `src/`, `tests/miri_unsafe.rs` | 🟢 2-4h | 🔴 Bloqueante | ⏳ Pendiente — depende de AUDIT-01 |
| `AUDIT-04` | **🔍 Root-cause crash benchmark Python SDK (`0xC0000409`)** - Nocturnal suite: `Python Benchmark Suite` falló con exit -1073740791 (STATUS_STACK_BUFFER_OVERRUN) al ejecutar el benchmark 10K/128d/1000q. Determinar si es manifestación de C1 (UAF en `try_numpy_array`), stack overflow de Windows (histórico con crates grandes), o bug propio del benchmark. **Pipeline:** 🔍 Inv — aislar reproducción mínima (benchmark standalone + ASAN/valgrind si disponible); 📊 Análisis — atribuir causa; ✅ Verif — correr benchmark 3x sin crash; 🔧 Impl — fix según causa. **DoD:** benchmark Python pasa estable; crash atribuido y documentado. *(hallazgo audit-full 2026-08-04, Phase 8)* | `benchmarks/`, `vantadb-python/src/`, `heavy_nocturnal_tests.log` | 🟡 4-8h | 🟠 Alta |

> **Items previos resueltos (9):** ver `docs/progreso/BACKLOG_HISTORY.md` (P1).

---

> **Phase 2: ⚡ Quick Wins Técnicos** — **31 items removidos:** ver `docs/progreso/BACKLOG_HISTORY.md` (P2). No quedan items activos en P2.
> ⚠️ **Nota DRV-014 (tradeoff WAL batch):** ver ADR `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md` — el fix original fue revertido deliberadamente por `cae92db3` (batch-append por shard, 3-5× speedup). Tradeoff de performance, no deuda pendiente.

---

> **Phase 3: 🧪 Test Coverage (Adapters & Engine)** — **14 items removidos:** ver `docs/progreso/BACKLOG_HISTORY.md` (P3). No quedan items activos en P3.

---

## Phase 4: 🔧 Engineering Health & Architecture

> Investigaciones de salud de ingeniería — rendimiento, concurrencia, arquitectura.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `DEBT-01` | **🔧 Reparar `scripts/validate-docs-coverage.ps1` (roto en origen) + gaps de docs de API** — El gate de cobertura de docs está caído (exit=1 siempre). **Parte A (script):** línea 64 referencia `src\sdk\search.rs` que ahora es el directorio `src/sdk/search/` — corregir la ruta. **Parte B (gaps):** documentar los métodos de API pública sin doc en `docs/api/*` (detectados: `bulk_commit_interval`, `NoVectorForKey`, `create`, `bulk_import`, `graph_page_rank`, `graph_degree_centrality`, `recover_archived_nodes`; re-ejecutar el script tras arreglar la ruta para capturar el resto). **DoD:** `pwsh scripts/validate-docs-coverage.ps1` termina con exit 0. Origen: hallazgo deuda técnica DESKTOP-01 (2026-08-04). | `scripts/validate-docs-coverage.ps1`, `docs/api/*.md` | 🟡 1-2d | 🟡 |
| `TECH-01` | **🐛 Fix MCP: `--db` no respetado por el proceso hijo** — `vanta-cli server --mcp --db /x` setea `VANTA_DB` en el hijo (`src/cli_handlers/server.rs:244`) pero `VantaConfig` lee `VANTADB_STORAGE_PATH` (`src/config.rs:408`) → la DB cae en `vantadb_data` del CWD en vez de `/x`. **Pipeline:** 🔍 Inv — trazar el flujo spawn del child (`cmd_server_mcp`); 📊 Análisis — el fix es setear `VANTADB_STORAGE_PATH` en el hijo (no cambiar la lectura de config, que es contrato); ✅ Verif — e2e `vanta-cli server --mcp --db /tmp/x` y confirmar lock/persistencia en `/tmp/x`; 🔧 Impl — 1-línea en `server.rs`. **DoD:** `--db` respetado por el MCP server; tests e2e MCP pasan. Origen: hallazgo §6.1 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `src/cli_handlers/server.rs` | 🟢 2-4h | 🟡 |
| `TECH-02` | **🐛 WASM: `reindexHnswFromText` lanza WASM_ERROR** — **Premisa corregida 2026-08-05:** el `pkg/` YA exporta `reindex_hnsw_from_text` (`pkg/vantadb_wasm.d.ts:183`); el wrapper TS `vantadb-ts/src/vantadb.ts:542-548` está obsoleto (comentario stale) y lanza WASM_ERROR. **Pipeline:** 🔍 Inv — comparar `pkg/vantadb_wasm.d.ts` contra el wrapper; 📊 Análisis — fix 1-línea sin rebuild; ✅ Verif — test browser/TS llama a la función sin error; 🔧 Impl — `return this._wasm("reindexHnswFromText", () => this.inner.reindex_hnsw_from_text(namespace, pageSize))`. **DoD:** `reindexHnswFromText` funciona en TS/WASM; `npm test` en `vantadb-ts` pasa. Origen: hallazgo §6.8 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `vantadb-wasm/pkg/`, `vantadb-wasm/src/lib.rs` | 🟢 2-4h | 🟡 |
| `TECH-03` | **📝 Corregir 4 stale-docs detectados en investigación DESKTOP-01b** — (1) `docs/api/HTTP_API.md:124-125` claim falso "Full MCP + HTTP" (con `--http --mcp` solo arranca HTTP; `mcp_mode = mcp && !http`); (2) `vantadb-python/README.md` documenta `put_memory`/`search_hybrid`/`memory_stats` inexistentes; (3) `docs/Investigaciones/DESKTOP-01:15` afirma que Python usa feature `python_sdk` (la capa real es crate `vantadb_py`); (4) `docs/api/MCP.md:56` documenta tool `query`, el código expone `query_lisp`. **Pipeline:** 🔍 Inv — cotejar cada claim contra el código; 📊 Análisis — redactar corrección; ✅ Verif — grep que los métodos/tools existen; 🔧 Impl — editar docs. **DoD:** 4 docs corregidos; `pwsh scripts/validate-docs-coverage.ps1` verde. Origen: hallazgos §6.3-6.6 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `docs/api/HTTP_API.md`, `vantadb-python/README.md`, `docs/Investigaciones/DESKTOP-01-*.md`, `docs/api/MCP.md` | 🟢 2-4h | 🟢 |
| `TECH-04` | **🔄 ADR + unificar naming de env vars (`VANTA_DB` vs `VANTADB_*`)** — Root cause del bug TECH-01: coexistencia de `VANTA_DB` (CLI, `src/cli.rs:14-16`) y `VANTADB_STORAGE_PATH` (config, `src/config.rs:408`). **Pipeline:** 🔍 Inv — inventariar todas las env vars y sus lecturas; 📊 Análisis — diseñar esquema unificado (deprecar `VANTA_DB` con fallback o alias) y escribir ADR de contrato; ✅ Verif — test de compatibilidad (leer ambas, warning de deprecación); 🔧 Impl — aplicar solo si ADR lo aprueba; **no es urgente** (puede quedar como debt documentada). **DoD:** ADR publicado en `docs/architecture/adr/`; decisión tomada (migrar vs mantener alias). Origen: hallazgo §6.1 (root-cause) de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `src/cli.rs`, `src/config.rs`, `src/cli_handlers/server.rs`, `docs/architecture/adr/` | 🟡 3-5d | 🟠 |
| `TECH-05` | **✨ Feature MCP: implementar resource `schema://`** — Documentado en `docs/api/MCP.md:79-81` pero no implementado en código (solo existen `metrics://`, `memory://`, `namespace://`). **Pipeline:** 🔍 Inv — revisar `resources/list` + `resources/read` en `vantadb-mcp/src/lib.rs:605-706`; 📊 Análisis — definir shape del JSON de schema (estructura de namespaces/keys); ✅ Verif — tests `vantadb-mcp/tests/mcp_tests.rs`; 🔧 Impl — handler + tests. **DoD:** `resources/read schema://` devuelve schema; tests verdes. Origen: hallazgo §6.6 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. (Sin solape: los items MCP del backlog-guide no son tareas activas — referencia muerta limpiada 2026-08-05.) | `vantadb-mcp/src/lib.rs`, `vantadb-mcp/tests/mcp_tests.rs` | 🟡 4-8h | 🟠 |
| `TECH-06` | **✨ Feature: CORS en `vantadb-server` HTTP (opcional)** — No hay headers CORS (`docs/api/HTTP_API.md:148-150` recomienda reverse proxy). Solo necesario si el frontend webview fetchea directo; el plan desktop lo evita (reqwest desde Rust). **Pipeline:** 🔍 Inv — confirmar necesidad real (webview fetch vs Rust backend); 📊 Análisis — middleware tower-http CORS con origenes permitidos configurables; ✅ Verif — test e2e con Origin header; 🔧 Impl — feature-gated. **DoD:** CORS configurable; default off (sin cambio de comportamiento actual). Origen: hallazgos §2.7/§6.9 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `src/cli_server.rs`, `src/config.rs` | 🟡 4-8h | 🔵 |
| `TECH-07` | **✨ WASM: publicar `pkg/` con feature `opfs` (worker)** — El `pkg/` precompilado NO incluye `connect_worker`/`worker_read/write/delete` (falta recompilar con `--features opfs` + servir `opfs_bridge.js`). **Pipeline:** 🔍 Inv — verificar exports ausentes en `pkg/` vs `lib.rs:334-399`; 📊 Análisis — decidir si el worker OPFS es requerido (Tauri no lo necesita; browser sí); ✅ Verif — test browser con worker; 🔧 Impl — build + publish. **DoD:** `pkg/` con worker opcional documentado. Origen: hallazgo §6.7 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `vantadb-wasm/pkg/`, `vantadb-wasm/src/worker.rs`, `vantadb-wasm/src/opfs_bridge.js` | 🟡 4-8h | 🔵 |
| `TECH-08` | **📋 Decisión: promover `vantadb-server`/`vantadb-mcp`/`vantadb-wasm` a default-members** — Actualmente experimentales y NO default-members (Cargo.toml:593-599), hay que construir con `-p` explícito. **Pipeline:** 🔍 Inv — analizar impacto en CI/builds (¿qué rompería ser default?); 📊 Análisis — recomendación + consecuencias; ✅ Verif — `cargo check --workspace` con los 3 habilitados; 🔧 Impl — **solo si se decide**; es cambio de política de CI. **DoD:** decisión documentada (promover vs mantener experimental) con ADR o nota en CI_POLICY. Origen: hallazgo §6.10 de `docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`. | `Cargo.toml`, `.github/workflows/`, `docs/operations/CI_POLICY.md` | 🟢 2-4h | 🟠 |
| `AUDIT-05` | **🧹 Housekeeping audit-full 2026-08-04** - (1) Agregar `.playwright-cli/` a `.gitignore` (artefactos de sesión browser quedan untracked; riesgo de commit accidental). (2) ADR `003_sync_async_decoupling.md`: la nota SurrealDB añadida sin bump de status/versión se lee como parte de la decisión original — agregar sección Addendum o línea `last-updated`. (3) `GH-123.md`: campo `Estado: ⬜ PENDING` stale (trabajo ya committeado `d406feab`). | `.gitignore`, `docs/architecture/adr/003_sync_async_decoupling.md`, `.opencode/skills/campaign-executor/tasks/GH-123.md` | 🟢 30min | 🔵 Baja |
| ~~`AUDIT-06`~~ | ~~**⚡ RRF fusion: skip single-channel**~~ - ~~`fuse_rrf_many`/`apply_rrf_contributions` construyen `BTreeMap<(String,String),Hit>` (2 String clones por candidato) incluso en paths single-channel (sparse puro, dense puro) o ya-uniques.~~ *(hallazgo audit-full 2026-08-04, F3 — downgraded a low por ISO review, ≤750 candidates)* | ~~`src/planner.rs:144-185`~~ | ~~🟢 1-2h~~ | 🔁 FUSIONADA con AUDIT-07 (2026-08-05) → investigación de rendimiento delegada a vanta-tuner (Task 10 del plan): medición previa (flamegraph) antes de tocar código; DoD: si impacto <1%, cerrar WONTFIX con ADR ligero. |
| ~~`AUDIT-07`~~ | ~~**🔍 Evaluar SparseVector `BTreeMap<u32,f32>` vs sorted `Vec<(u32,f32)>`**~~ - ~~BTreeMap es correcto/determinístico (dot merge-lineal eficiente) pero mayor footprint por entrada + JSON más grande.~~ *(hallazgo audit-full 2026-08-04, F5)* | ~~`src/node.rs:409`~~ | ~~🟢 2-4h~~ | 🔁 FUSIONADA con AUDIT-06 (2026-08-05) → investigación de rendimiento delegada a vanta-tuner (Task 10 del plan): tradeoff BTreeMap ya decidido y razonado en doc comment `node.rs:404-407`; medición previa antes de cualquier cambio. |
| `AUDIT-08` | **📝 Actualizar P2 debt ledger (refs stale + comentario LRU)** - El ledger en AGENTS.md apunta a `lib.rs:1754/34-36/895-901/394-413` pero `src/lib.rs` ahora es 193 líneas de módulos. Refs reales: P2-2 → `vantadb-python/src/vector.rs:63` + `types.rs:365`; P2-3 → `vantadb-python/src/convert.rs:23-70`; P2-7 → `src/sdk/serialization/mod.rs:227-294`; P2-8 → `vantadb-wasm/src/lib.rs:402-433`. Además el comentario de eviction del LRU dice O(1) pero es O(n) `min_by_key`. *(hallazgo audit-full 2026-08-04, P2-2/P2-3/P2-7/P2-8)* | `AGENTS.md`, `vantadb-python/src/convert.rs` | 🟢 30min | 🔵 Baja |
| ~~`INV-002`~~ | ~~**🔍 Memory Telemetry Correction — investigación** — El reporte de RAM actual es inconsistente (mezcla core RAM, index RAM, page cache, mmap, ingest buffers). Audit dice "hasta que arregles la telemetría, no hables de eficiencia de memoria". **Alcance:** (1) Mapear qué mide cada métrica actual vs qué debería medir, (2) Diseñar esquema de telemetría con categorías separadas (core, index, page cache, mmap, ingest), (3) Identificar qué estructuras contribuyen a cada categoría, (4) Proponer implementación con `tracing::metrics` + labels. **Sin implementación — solo diseño + propuesta.** **✅ COMPLETADA 2026-07-30 — Esquema 5 categorías diseñado (core/index/page_cache/mmap/ingest) en `docs/operations/MEMORY_TELEMETRY.md`; IntGaugeVec con label `category` validado contra API oficial prometheus; contrato `OperationalMetrics` preservado.**~~ | ~~`src/metrics/`, `docs/operations/MEMORY_TELEMETRY.md`, `docs/operations/PERFORMANCE_TUNING.md`~~ | ~~🟡 3-5d~~ | ✅ |
| ~~`INV-003`~~ | ~~**🔍 Tokio Blocking Audit — auditoría** — Auditoría de `std::fs::*` y `std::sync::Mutex::lock()` en contextos `async`. **✅ COMPLETADA 2026-07-31 — Reporte: `docs/Investigaciones/INV-003-tokio-blocking-audit.md`. Se verificó que llamadas bloqueantes usan `spawn_blocking` correctamente.**~~ | ~~`src/`, `vantadb-server/`, `vantadb-mcp/`~~ | ~~🟡 2-3d~~ | ✅ |
| ~~`INV-004`~~ | ~~**🔍 mimalloc como Global Allocator — investigación** — Evaluación de `mimalloc` y allocators globales por plataforma. **✅ COMPLETADA 2026-07-31 — Reporte: `docs/Investigaciones/INV-004-mimalloc-global-allocator.md`. Feature `custom-allocator` y `mimalloc` validados y configurados.**~~ | ~~`Cargo.toml`, `src/bin/vanta-cli.rs`~~ | ~~🟢 4-6h~~ | ✅ |
| ~~`INV-005`~~ | ~~**🔍 ErrorBoundary en web frontend — investigación** — Auditoría de `react-error-boundary` y manejo de errores Next.js. **✅ COMPLETADA 2026-07-31 — Reporte: `docs/Investigaciones/INV-005-error-boundary-web.md`. Se propone adoptar `error.tsx` nativo de App Router.**~~ | ~~`web/src/app/layout.tsx`, `web/package.json`~~ | ~~🟢 2-4h~~ | ✅ |

> **Items previos completados (10):** ver `docs/progreso/BACKLOG_HISTORY.md` (P4) — movidos a `docs/progreso/README.md`.

---

## Phase 5: 📖 Docs & Community

> Preparación de documentación pública, comunidad, y onboarding.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|----------|------|-------------|
| ~~`MKT-14`~~ | ~~**Publicar 3 case studies** + ruta `/case-studies/` + `/case-studies/[slug]`~~ | ~~`web/src/components/vanta/vanta-data.ts` (CASE_STUDIES, 3 items), `web/src/app/case-studies/`~~ | ~~🟡 1-2d~~ | ~~🔴~~ | ✅ COMPLETADA 2026-08-02 — 3 CS con métricas, listing + detail pages, i18n (62 keys), rutas en navbar + LIVE_ROUTES. Validado por audit 2026-07-28. |
| ~~`TSK-106`~~ | ~~**Habilitar GitHub Discussions**~~ | — | ~~🟢 1h~~ | ~~🟠~~ | ✅ Ya estaba habilitado (`has_discussions: true`). 0 discussions creadas. |
| `NUEVO-01` | **README hero** con readme-aura + benchmark gráfico + GIF demo (incluye slice GH-139) | `README.md` (hoy solo texto + badges SVG — sin hero; premisa corregida 2026-08-05) | 🟡 2-3d | 🟠 | ❌ Desde cero |
| ~~`NUEVO-07`~~ | ~~**Migration tools: Chroma→Vanta, LanceDB→Vanta** — **✅ COMPLETADA 2026-08-02** — `vantadb_py/migrate/chroma.py` + `lancedb.py` + CLI, tests (4 migration + 42 regresión), tutoriales corregidos a API real `vantadb_py.VantaDB` (el audit 2026-07-28 reportó scripts inexistentes — falso positivo).~~ | ~~`vantadb-python/vantadb_py/migrate/`, `docs/tutorials/`~~ | ~~🟡 3-5d~~ | ~~🟠~~ | ✅ |
| ~~`NUEVO-08`~~ | ~~**Learning path estructurado** en tutorials/ (5-7 ejemplos)~~ — **✅ COMPLETADA 2026-08-02** — 6 tutoriales active con API real (`vantadb_py`), índice learning path (`docs/tutorials/index.md`), mdBook sync (SUMMARY 6). API inventada corregida en 01/02. Commits a460e4e4→cff2fb99. | ~~`docs/tutorials/`, `docs/book/src/tutorials/`~~ | ~~🟡 2-3d~~ | ✅ |
| ~~`NUEVO-10`~~ | ~~**Benchmark suite pública reproducible** — **✅ COMPLETADA 2026-08-02** — `benchmarks/requirements.txt` (path standalone `pip install vantadb-py` desde PyPI 0.5.0), hints corregidos en los 3 scripts (vantadb_local_bench, competitive_bench, batch_vs_sequential), `benchmarks/README.md` (guía pública), `docs/operations/BENCHMARKS.md` sección 3 reescrita (standalone antes que maturin). Smoke test exitoso (JSON 5/5 claves). Commit d0b1c7c6.~~ | ~~`benchmarks/`, `docs/operations/BENCHMARKS.md`~~ | ~~🟡 3-5d~~ | ✅ |
| ~~`TSK-107`~~ | ~~Community showcase page (`/showcase`, `/about/community`)~~ | ~~`web/src/app/showcase/page.tsx` (6 items mock)~~ | ~~🟢 4-6h~~ | ~~🟡~~ | ✅ 6 items actualizados a ejemplos reales (LangGraph, AutoGen, Haystack, CrewAI, Rust hybrid, GraphRAG) |
| `—` | Good first issues (18 open en GitHub) | GitHub Issues (#118-#145) | 🟢 2-4h | 🟠 | ✅ 18 issues creados (22 en total, 3 duplicados cerrados) |
| ~~`INV-006`~~ | ~~**🔍 Blog series completion — plan de finalización**~~ — **✅ COMPLETADA 2026-08-02** — `docs/strategy/BLOG_SERIES_PLAN.md`: inventario 4 web vs 3 docs/blog (6 mismatches M1-M6), revisión drafts, audiencia + keyword research (5 segmentos), calendario Show HN + cadencia 2/mes. Sin implementación (solo plan). Commit 042e8e50. | ~~`docs/blog/`, `docs/strategy/SHOW_HN_PREP.md`~~ | ~~🟢 2-4h~~ | ✅ |
| `COM-02` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/discord/todo.md` + assets SVG + server activo | 🟡 2-3d | 🟢 | ⚠️ Docs + assets OK. Config pendiente |
| `COM-03` | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 | ⚠️ Forums seedeado (9 threads: FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual — no API-accessible |
| `COM-04` | **Discord: ticketing system, stage channel, Server Discovery, Canny.io** | — | 🟢 4-6h | 🟢 | ⏸️ **ICEBOX 2026-08-05** — Server Discovery exige 1000+ miembros; Canny.io SaaS externo; ticketing requiere bot externo (Ticket Tool/Helper.gg). Nada accionable hoy. Dependencias documentadas en `docs/discord/todo.md`. No cuenta como activa. |

---

## Phase 6: 🚀 Launch Campaign

> Todo lo necesario para el Show HN y marketing de lanzamiento.

### 👤 Tareas Humanas (no-delegables a agentes)

> Requieren identidad legal, pago, o acceso manual a dashboards externos. `owner: human`. No ingresan al flujo de agentes. (Sección creada 2026-08-05 — LEG-01 movido aquí.)

| ID | Descripción | Estimación real | Prioridad | Estado Real |
|----|-------------|-----------------|-----------|-------------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** — Requiere abogado, pago (~$250-350/clase USPTO, ~€850 EUIPO), identidad legal. Estimación original "2-4h" irreal. | semanas, $2-5K | 🔴 | ❌ No iniciado — mover a `docs/strategy/GO_TO_MARKET.md` cuando exista |

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| ~~`MKT-03`~~ | ~~**Show HN post**~~ | ~~🟢 2h~~ | ~~🔴~~ | ✅ Draft actualizado a v0.4.0 en `docs/strategy/SHOW_HN_PREP.md` |
| ~~`MKT-04`~~ | ~~Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA)~~ | ~~🟢 2-4h~~ | ~~🟠~~ | ✅ 3 drafts en `docs/strategy/REDDIT_POSTS.md` |
| ~~`MKT-05`~~ | ~~Technical blog posts (5+ pre-launch) — 4/5 posts escritos~~ | ~~🟡 2-3d~~ | ~~🟠~~ | ✅ 5/5 — 5º post benchmarks añadido 2026-08-04 (`docs/blog/benchmarks_vs_lancedb_chroma.md`, run real competitive_bench.py) |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ Desde cero |
| ~~`MKT-15`~~ | ~~**Página de benchmarks competitivos** (`/benchmarks`) — Ruta existe (BenchmarksView + BenchmarkRace, 444L), sin tabla competitiva VantaDB vs Pinecone/Weaviate/Chroma~~ | ~~🟡 2-3d~~ | ~~🔴~~ | ✅ Tabla competitiva §03 añadida 2026-08-04 (cifras reales VantaDB/LanceDB/ChromaDB, filosofía citada Pinecone/Weaviate, commit 68e18405) |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** — Sin doc específico | 🟡 1-2d | 🟡 | ❌ Desde cero |
| ~~`MKT-17`~~ | ~~**Página de comparación competitiva interactiva** — Sin ruta `/compare` ni archivos~~ | ~~🟡 2-3d~~ | ~~🟢~~ | ❌ DUPLICADA 1:1 de INV-007-B (Task 47): `benchmarks-view.tsx:352-365` ya tiene tabla estática (MKT-15 ✅); INV-007-B especifica `competitive-table.tsx` + contrato JSON con más rigor. Tachada 2026-08-05, consolidada en INV-007-B. |
| ~~`TSK-103`~~ | ~~**Public benchmark site (`/benchmarks`)** — BenchmarksView + BenchmarkRace existen (BENCH01, SIFT1M). Falta script público reproducible~~ | ~~🟡 2-3d~~ | ~~🟠~~ | ~~⚠️ `/benchmarks` existe con datos benchmark reales. Sin script standalone público~~ ✅ RESUELTA por NUEVO-10 (2026-08-02, commit d0b1c7c6): `benchmarks/README.md` + `requirements.txt` + 3 scripts públicos reproducibles. Remanente = INV-007-B (JSON contrato). Tachada 2026-08-05 — sin duplicado en progreso (ya está ahí). |
| ~~`TSK-104`~~ | ~~**Demo agent: LangChain + Ollama + VantaDB** — **✅ COMPLETADA 2026-08-02** — `examples/python/langchain_ollama_rag.py` (151 líneas) con integraciones reales (`VantaDBVectorStore` + `OllamaEmbeddings`), fallback determinístico sin Ollama, smoke exit 0. Sketch legacy `langchain_rag.py` eliminado.~~ | ~~`examples/python/langchain_ollama_rag.py`, `docs/operations/EXPERIMENTAL_FEATURES.md`~~ | ~~🟡 1-2d~~ | ~~🟠~~ | ✅ |
| ~~`INV-007`~~ | ~~**🔍 Competitive benchmark vs LanceDB/Chroma — investigación y diseño** — El asset marketing #1 para audiencia técnica. **Alcance:** (1) Investigar `ann-benchmarks` y su conector para VantaDB, (2) Definir datasets: glove-100-angular + sift-128-euclidean, (3) Diseñar metodología: throughput, latencia p50/p95/p99, Recall@10, RAM, (4) Evaluar si benchmarks internos existentes (`/benchmarks`) pueden extenderse o si se necesita script standalone, (5) Proponer implementación mínima para página pública. **Sin implementación — solo diseño + propuesta.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-007-competitive-benchmark-lancedb-chroma.md`. Propuesta: harness standalone reproducible Python con glove-100-angular + sift-128-euclidean, protocolo Recall@10/QPS/latencia p50-p95-p99/RSS, 3 slices verticales, rechazado ann-benchmarks por desuso.)~~ | ~~🟡 2-3d~~ | ~~🟠~~ | ✅ |

---

## Phase 7: 🌐 WASM & Performance

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
> **Items removidos (5):** ver `docs/progreso/BACKLOG_HISTORY.md` (P7).

---

## Phase 8: 🔮 Post-Launch & Enterprise

> Features para después del lanzamiento público.

| ID | Descripción | Esfuerzo | Prio |
|----|-------------|----------|------|
| ~~`CLI-01`~~ | ~~**CLI polish: handlers backup/restore/doctor/stats/inspect existen pero no conectados al binary. REPL/TUI no existen**~~ — **✅ COMPLETADA** (5 handlers conectados: backup, restore, doctor, inspect, stats. 46 tests CLI pasan) | ~~🟡 2-3d~~ | ✅ |
| ~~`DEVOPS-HOMEBREW`~~ | ~~**Homebrew formula**~~ — **✅ COMPLETADA** (`Formula/vantadb.rb` existe con livecheck, 4 plataformas, install + test. Placeholder SHA256 — actualizar antes de publish.) | ~~🟢 4h~~ | ✅ |
| ~~`DEVOPS-PY313`~~ | ~~**Python 3.13 wheels en CI matrix**~~ — **✅ COMPLETADA** (`pyproject.toml` ya incluye `>=3.11` + classifier 3.13. CI verify jobs actualizados a Python 3.13 + build mantiene 3.11 con abi3) | ~~🟢 2h~~ | ✅ |
| ~~`DEVEX-DEMO`~~ | ~~**Demo app (Rust + Python)** — Phase 4.G~~ — **✅ COMPLETADA** (`examples/demo/demo.py` 239L con create → insert → search → delete, README, requirements.txt) | ~~🟡 2-3d~~ | ✅ |
| ~~`DEVEX-EXAMPLES`~~ | ~~**Rust examples en `examples/rust/`** (no `docs/examples/`)~~ — **✅ COMPLETADA** (4 ejemplos: basic, hybrid, graphrag, concurrent. Compilan clean.) | ~~🟢 4-6h~~ | ✅ |
| `NUEVO-16` | **Product Quantization (PQ) 96x** — compresión para datasets >RAM. RabitQ + TurboQuant + SQ8 existen, PQ real no | Alto | 🔵 |
| ~~`NUEVO-17`~~ | ~~**Segment LSM-style** — hot/warm/cold tiers. Fjall tiene LSM interno, tiers no~~ — **✅ COMPLETADA 2026-08-02** (TierPolicy SizeBased/FrequencyBased/AgeBased + TierPolicyConfig + L3 archive configurable en `LsmConfig`, promoción L0(hot)→L1(warm)→L2(cold)→L3(archive) en `compact_level`, 3 tests de promoción. `docs/architecture/STORAGE-TIERS.md`.) | ~~Muy alto~~ | ✅ |
| `NUEVO-22` | **Sparse indexed search — inverted index + posting lists** — Sparse search YA existe como brute-force O(n) sobre subset filtrado (`sparse_memory_search` sdk/search/mod.rs:721-746; hybrid 3 canales :748-780; `SparseVector` node.rs:409). Gap real: falta índice invertido + posting lists. *(premisa original NUEVO-18 "solo mención en test" FALSA — corregida 2026-08-05; ID renombrado de NUEVO-18)* | `src/sdk/search/mod.rs`, `src/node.rs`, `src/index/` | Alto | 🔵 |
| ~~`NUEVO-21`~~ | ~~**Vectara competitive research**~~ | ~~🟢 2-4h~~ | ✅ Hallazgo clave: Vectara cerró self-service tier → gap de mercado para soluciones local-first. Reporte en `docs/audit-reports/vectara-competitive-research-2026-07-27.md` |
| ~~`TSK-107b`~~ | ~~**✅ Audit logging enterprise (JSONL, timestamp + op)** — módulo `src/audit.rs` append-only JSONL (ISO 8601 + op + outcome + reason), opt-in via `audit_log_path`/`VANTADB_AUDIT_LOG_PATH`, hooks en put/put_batch/delete/delete_by_filter/export/import, no-op sin config. Migrada a progreso.~~ | ~~🟡 2-3d~~ | ~~🟡~~ | ~~✅ Hecho~~ |
| ~~`ENT-04`~~ | ~~**✅ Connection pooling + circuit breaker** — módulos `src/connection_pool.rs` + `src/circuit_breaker.rs` (feature-gated `server`), `ServerState` con breaker+pool en `cli_server.rs`, middleware breaker como capa más externa, 503 + `Retry-After` al abrir, probe half-open, config `server.*` (pool/min_connections/max_connections/breaker threshold/timeout). Migrada a progreso.~~ | ~~🟡 2-3d~~ | ~~🟡~~ | ~~✅ Hecho~~ |
| `BIZ-01` | **Enterprise features: encryption + RBAC ya en crate principal. Audit/replication/enterprise crate separado no existen** | 🟡 3-5d | 🟡 ⏳ |
| ~~`WEB-001`~~ | ~~**Implementar WASM demo en `/playground`** — El CodePlayground actual es un simulador, no corre WASM real. La demo WASM anterior estaba en `web_old/` (eliminada). Reconstruir requiere integrar `@vantadb/wasm` en el componente.~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ WASM real integrado 2026-08-04 — rebuild `--target no-modules` (initSync), assets en `web/public/vanta-wasm/`, commit ee310422 |
| ~~`WEB-18`~~ | ~~**⚠️ Definir pricing y estrategia de monetización** — El archivo `docs/web/standards/product-positioning.md` y `vanta-data.ts` tienen un plan "Team $49/mes por seat" que NO existe en `docs/strategy/GO_TO_MARKET.md`. Decidir: (a) agregar Team $49 a la estrategia GTM, (b) alinear vanta-data.ts con los planes reales de GO_TO_MARKET.md, o (c) eliminar pricing del sitio hasta definir.~~ | ~~🟡 2-4h~~ | ~~🔴~~ | ✅ Opción (b) aplicada 2026-08-04 — tier Team $49 eliminado, pricing alineado a GTM Phase 1 (Community + Enterprise), commit f90b4ec8 |

| ~~`DESKTOP-01`~~ | ~~**🔍 Investigar Tauri como plataforma desktop para VantaDB**~~ — **✅ COMPLETADA 2026-08-04** (Recomendación: SÍ — Tauri v2 con integración Rust nativa `vantadb` en `src-tauri/`, SIN WASM/OPFS. Reporte: `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md`; effort MVP ~8-13 días hábiles.) | ~~🟡 3-5d~~ | 🔵 |
| ~~`SDK-01`~~ | ~~**`delete_by_filter()` — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `VantaEmbedded::delete_by_filter` en SDK + PyO3/WASM + CLI `delete-by-filter`) | ~~🟡 3-5d~~ | ✅ |
| ~~`SDK-02`~~ | ~~**`similar_to_key()` — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `VantaEmbedded::similar_to_key` en SDK + CLI `similar-to-key`) | ~~🟡 3-5d~~ | ✅ |
| ~~`SDK-03`~~ | ~~**`count()` con filtros — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `VantaEmbedded::count` con soporte de metadatos en SDK + CLI `count`) | ~~🟡 1-2d~~ | ✅ |
| ~~`SDK-04`~~ | ~~**Multi-namespace search — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `search_multi` y `search_all` en SDK + CLI `search-multi`/`search-all`) | ~~🟡 4-7d~~ | ✅ |
| ~~`SDK-05`~~ | ~~**Expanded metadata filters en SDK — Exponer 6 operadores**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `matches_advanced_filters` con operadores Eq, Neq, Gt, Lt, Gte, Lte en listados y paginación) | ~~🟡 5-10d~~ | ✅ |
| ~~`REC-001`~~ | ~~**[Foundation] Definir `VantaFilterOp` + `VantaMemoryFilter` types** — `VantaFilterOp` enum con 6 variantes (Eq, Neq, Gt, Lt, Gte, Lte), `VantaMemoryFilterItem` struct, `VantaMemoryFilter = Vec<...>`. Creado en `src/sdk/types.rs:106-126`, re-exportado desde `src/sdk/mod.rs`. `cargo check + clippy` ✅. **✅ COMPLETADA 2026-07-29.**~~ | ~~🟢 2-3h~~ | ✅ |
| ~~`REC-007`~~ | ~~**WAL compaction + vacuum CLI** — Binding directo de funciones existentes (`VantaEmbedded::compact_wal()`, `VantaEmbedded::vacuum()`) a CLI. `vanta-cli wal compact` + `vanta-cli wal vacuum`. Sin lógica nueva. **Nuevo — identificado en auditoría 2026-07-28.** **✅ COMPLETADA 2026-07-29.**~~ | ~~🟢 1-2h~~ | ✅ |
| ~~`REC-008`~~ | ~~**[Diseño] Incremental backup + PITR CLI**~~ — **✅ COMPLETADA 2026-07-31** (Diseño documentado, Fase A implementada agregando `MANIFEST.json` con integridad CRC32C a backups) | ~~🟡 1d~~ | ✅ |
| ~~`REC-009`~~ | ~~**[Investigación] Analizar viabilidad PQ (Product Quantization)**~~ — **✅ COMPLETADA 2026-07-31** (Análisis completo documentado, viabilidad deferida) | ~~🟢 2-4h~~ | ✅ |
| ~~`REC-010`~~ | ~~**py.typed marker + config maturin** — 2 `.pyi` stubs existían pero `py.typed` no. PEP 561 non-compliant. Creado `py.typed` + configurado `[tool.maturin] include` en `pyproject.toml`. **✅ COMPLETADA 2026-07-29.**~~ | ~~🟢 30min~~ | ✅ |
| ~~`REC-999`~~ | ~~**Corregir `docs/progreso/README.md`**~~ — **✅ COMPLETADA 2026-07-31** (README de progreso actualizado y corregido con los hitos reales) | ~~🟢 30min~~ | ✅ |

### 🔍 Investigaciones Post-Consolidación

> Items agregados 2026-07-29 tras verificación de 19 hallazgos de 4 sub-agentes contra código actual. **Sin implementación — solo investigación + propuesta.**

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-008`~~ | ~~**🔍 Batch Queries Python SDK — diseño** — `VantaDB.search_batch()` para ejecutar múltiples queries en paralelo vía Rayon. **Alcance:** (1) Definir API: `search_batch(queries: List[SearchRequest]) -> List[SearchResult]`, (2) Diseñar conversión eager a Rust types + `py.allow_threads` + Rayon parallel map, (3) Evaluar target: batch 10 queries < 3× single query latency, (4) Identificar si `VantaEmbedded` necesita método nuevo o wrapper. **Sin implementación — solo diseño.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-008-batch-queries-python-sdk.md`. `search_batch(vectors, top_k)` vector-only ya existía (lib.rs:1181, Rayon + GIL release). Propuesta: `search_batch_requests` con dataclass SearchRequest completo, patrón GIL+Rayon existente, errores fail-fast v1, target batch 10 < 3× single.)~~ | ~~`vantadb-python/src/lib.rs`, `src/sdk/api.rs`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ |
| ~~`INV-009`~~ | ~~**🔍 Phrase Queries + Term Positions — diseño** — Implementar phrase query operator con almacenamiento de term positions para snippets destacados. **Alcance:** (1) Diseñar extensión del text index para almacenar term positions por documento, (2) Evaluar `tantivy` como backend vs storage custom en VantaFile, (3) Definir sintaxis de phrase query (comillas dobles en IQL), (4) Evaluar integración con `generate_snippet_with_highlighting` existente. **Sin implementación — solo diseño.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-009-phrase-queries-term-positions.md`. Infra phrase-ready YA existía: `TextQueryPlan.phrases`, `token_positions`, `posting_value`, test v3. Propuesta: sintaxis IQL comillas dobles vía `Condition::TextMatch`, matching con `text_positions_match_phrase`, rechazado tantivy (YAGNI), highlight de frase en snippets.)~~ | ~~`src/text_index.rs`, `src/sdk/search/snippet.rs`, `src/parser/`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ |
| ~~`INV-010`~~ | ~~**🔍 ACID rollback multi-capa completo — diseño** — VFY-010 y TASK-30 implementaron ACID Phase 1-2 (buffered writes + BEGIN/COMMIT/ABORT). Falta rollback coordinado entre WAL, VantaFile, HNSW y KV store. **Alcance:** (1) Mapear estado actual (qué capas tienen rollback y cuáles no), (2) Diseñar protocolo de two-phase rollback entre capas, (3) Evaluar Approach B del research `ACID_TRANSACTIONS.md` vs implementación actual, (4) Proponer plan de implementación por fases. **Sin implementación — solo diseño.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/ACID_ROLLBACK_DESIGN.md`. Research recuperado de git, protocolo 2-phase + crash-windows, plan 4 fases.)~~ | ~~`src/storage/engine/ops.rs`, `src/index/graph.rs`, `src/wal.rs`~~ | ~~🟡 2-3d~~ | ✅ |
| ~~`INV-011`~~ | ~~**🔍 Core-Server Separation — auditoría** — Verificar si el core embebido (`VantaEmbedded`) tiene dependencias no deseadas del modo servidor (axum, tower, MCP). **Alcance:** (1) Escanear features de `Cargo.toml` que mezclan core vs server, (2) Identificar imports de server-only en `src/` (no en `vantadb-server/`), (3) Verificar si `default` features incluyen server deps, (4) Proponer separación limpia con feature gates. **Sin implementación — solo auditoría.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-011-core-server-separation.md`. Separación YA limpia: server deps opcionales detrás de features, default NO incluye axum/tower/tokio, imports server-only gated en lib.rs. Verificado: `cargo tree -F cli -e normal` = 0 deps server, `cargo check -F cli` exit 0. Sin cambios requeridos.)~~ | ~~`Cargo.toml`, `src/lib.rs`, `vantadb-server/Cargo.toml`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ |
| ~~`INV-012`~~ | ~~**Anti-Locality Disk Layout — re-evaluación** — DRV-130 T3 concluyó WONTFIX (~9% mejora, <20% threshold). Revisar si con los cambios recientes (LSM compaction, multi-level storage) el BFS relabeling tiene más impacto. **Alcance:** (1) Re-ejecutar benchmark en la arquitectura actual (LSM + multi-level), (2) Comparar resultados con benchmark original de DRV-130 (2,440ms vs 2,221ms, ~9%), (3) Si mejora >15%, recomendar re-apertura. **Sin implementación — solo benchmark + recomendación.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-012-antilocality-reevaluation.md`. Re-run `benches/vfile_search.rs`: mejora locativa ~7.0% — inferior al 9% de DRV-130 y bajo el 15% requerido. **WONTFIX confirmado, NO re-abrir.** LSM/multi-level no alteraron el resultado; causa raíz greedy-still vigente. Nota: backlog apuntaba a `src/index/graph.rs`, real en `src/storage/archive.rs`+`maintenance.rs`.)**~~ | ~~`src/index/graph.rs`, `tests/certification/`, `benches/`~~ | ~~🟡 1-2d~~ | ~~🟢~~ | ✅ |
| ~~`INV-019`~~ | ~~**🔍 Advanced Tokenizer (Unicode + Stopwords) — investigación** — **✅ SKIP 2026-08-02 — YA IMPLEMENTADA.** Verificado vs código: `src/tokenizer.rs` (tokenize_advanced, stemming, stopwords, Unicode folding), feature `advanced-tokenizer` en default (Cargo.toml:94,108), wiring en `src/config.rs`, tests multilingües (ES/FR/DE), bench `benches/tokenizer_bench.rs`. Commits: `1a7c4d04`, `7459a558`. Único gap: `docs/api/ADVANCED_TOKENIZER.md` no existe — doc de API, ticket separado.**~~ | ~~`src/text_index.rs`, `src/tokenizer.rs`, `Cargo.toml`, `src/config.rs`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ SKIP (ya implementada) |
| `INV-025` | **🔍 Search Quality v2 Scope — investigación y diseño** — Hybrid Retrieval v1 es deliberadamente conservador. Snippets, highlighting, explicaciones de ranking estables, tokenizer evolution y mejoras de ranking necesitan scoping antes de implementar. **Alcance:** (1) Definir qué outputs son API pública SDK/CLI y cuáles quedan debug-only, (2) Decidir si snippets/highlighting van antes que cambios de tokenizer, (3) Documentar non-goals, incluyendo claims de paridad hybrid-search competitiva, (4) Proponer un corpus de validación pequeño para regression tests. **Sin implementación — solo diseño + propuesta.** Origen: borrador #3 de `PUBLIC_ISSUE_DRAFTS.md` (era ConnectomeDB), extraído en auditoría de documentación 2026-08-04 — no existía en Backlog ni roadmap vigente. | `src/sdk/search/snippet.rs`, `src/text_index.rs`, `src/tokenizer.rs`, `docs/api/` | 🟡 1-2d | 🟡 | ⏳ Pendiente |

### 🌐 Web Frontend — Auditorías

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-013`~~ | ~~**🔍 JSON-LD structured data — auditoría** — **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-013-jsonld-structured-data.md`. Hallazgo: JSON-LD **AUSENTE**; Next.js Metadata API NO genera JSON-LD — emitir `<script type="application/ld+json">` manualmente. Propuesta schema.org/SoftwareApplication + Google Rich Results Test.)~~ | ~~`web/src/app/layout.tsx`, `web/src/app/page.tsx`~~ | ~~🟢 2-4h~~ | ~~🟡~~ | ✅ |
| ~~`INV-014`~~ | ~~**🔍 Light mode (CSS muerto) — auditoría** — **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-014-light-mode-css.md`. Premisa invertida: sitio es **LIGHT-ONLY** — no hay CSS light muerto. Deuda real: plomería DARK inerte — ThemeProvider/ThemeToggle/next-themes nunca montados (consumer único en navbar.tsx, código muerto). Recomendación: eliminar.)~~ | ~~`web/src/app/globals.css`, `web/src/components/vanta/theme-toggle.tsx`, `web/src/components/vanta/theme-provider.tsx`~~ | ~~🟢 1-2h~~ | ~~🟢~~ | ✅ |
| ~~`INV-015`~~ | ~~**🔍 Touch targets < 44px — auditoría** — **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-015-touch-targets-44px.md`. ~23 componentes < 44px WCAG 2.5.8; 2 icon buttons clear-search de 14px < 24px mínimo (severo). Inventario priorizado P0 navbar → P4; fix `size-11` / `min-h-[44px] min-w-[44px]`.)~~ | ~~`web/src/components/vanta/*.tsx`~~ | ~~🟢 2-4h~~ | ~~🟡~~ | ✅ |
| ~~`INV-016`~~ | ~~**🔍 Motion-duration tokens — auditoría** — **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-016-motion-duration-tokens.md`. No existen tokens de duración/easing; easing hardcodeado en 4 lugares. Propuesta CSS vars `--duration-fast/normal/slow` + `--ease-default` + mapa JS `motion.ts` para framer-motion/animejs. Corrección: Reveal es CSS transition, no framer-motion.)~~ | ~~`web/src/app/globals.css`, `web/src/components/vanta/page-transition.tsx`, `web/src/components/vanta/reveal.tsx`~~ | ~~🟢 1-2h~~ | ~~🟢~~ | ✅ |

### ⚙️ CI & Tooling — Auditorías

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-017`~~ | **✅ 🔍 sccache en CI — investigación** — Los builds Rust en CI compilan desde cero cada vez (~8-10 min). sccache podría cachear compilaciones entre runs. **Alcance:** (1) Investigar compatibilidad de `sccache` con GitHub Actions + `Swatinem/rust-cache`, (2) Evaluar si son complementarios o redundantes, (3) Diseñar integración mínima (instalar sccache + configurar `RUSTC_WRAPPER`), (4) Medir impacto estimado en tiempo de CI. **Sin implementación — solo investigación + propuesta.** | `docs/Investigaciones/INV-017-sccache-ci.md` | 🟢 2-4h | 🟡 | ✅ Hecho |

### 🚀 Implementaciones derivadas de Investigaciones (agregadas 2026-08-04)

> Items creados tras la verificación de 13 INV contra código real (sub-agentes, 2026-08-04). Cada fila referencia su doc de investigación.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `INV-005-A` | **Error boundary nativo `error.tsx`** — Crear `web/src/app/error.tsx` (Next.js App Router error boundary) + eliminar dep muerta `@mdxeditor/editor` (0 imports, package.json:16). No instalar `react-error-boundary` (solo transitiva vía @lexical/react). | `web/src/app/error.tsx`, `web/package.json` | 🟢 1-2h | 🟡 | ⏳ Pendiente |
| `INV-007-B` | **Benchmark competitivo: emitir JSON contrato + Slice 2 web** — Hacer que `benchmarks/competitive_bench.py` (751 L, ya existe) emita `competitive_benchmark.json` versionado (fecha/hardware/versiones) y crear `web/src/components/vanta/competitive-table.tsx` renderizado bajo `<BenchmarkRace />` en `/benchmarks`. Sin números inventados (INV-007 §6). | `benchmarks/competitive_bench.py`, `web/src/components/vanta/competitive-table.tsx`, `web/src/lib/vanta-data.ts` | 🟡 1-2d | 🟡 | ⏳ Pendiente |
| `INV-008-B` | **Implementar `search_batch_requests` en Python SDK** — Método batch con `SearchRequest` dataclass completo, GIL-release eager (`py.allow_threads`), Rayon `par_iter` con fail-fast (`try_for_each`), closure cerrado en el binding. Target: batch 10 queries < 3× single. Extender `benchmarks/batch_vs_sequential_bench.py` (INV-008 §5). | `vantadb-python/src/lib.rs`, `src/sdk/api.rs`, `benchmarks/batch_vs_sequential_bench.py` | 🟡 1-2d | 🟡 | ⏳ Pendiente |
| `INV-009-B` | **Phrase queries (reescalada 2026-08-05)** — único gap real: `Condition::TextMatch` en parser IQL de grafo (`src/query.rs:121-126`); enforcement (`src/sdk/search/mod.rs:416-425`), tokenización literal (`src/text_index.rs:358-401`) y matched_phrases (`types.rs:438`) YA existen. (INV-009 §6) | `src/parser/mod.rs`, `src/query.rs`, `src/sdk/search/snippet.rs` | 🟡 1-2d | 🟡 | ⏳ Pendiente |
| `INV-013-B` | **JSON-LD structured data en el sitio** — Emitir `<script type="application/ld+json">` (schema.org/SoftwareApplication, VantaDB 0.2.0) en Server Component del `<head>` raíz; validar con Google Rich Results Test. Metadata API de Next.js 16 NO genera JSON-LD (INV-013). | `web/src/app/layout.tsx` | 🟢 2-4h | 🟡 | ⏳ Pendiente |
| `INV-014-B` | **Limpiar plomería dark inerte + corregir notas stale** — Eliminar `theme-provider.tsx`, `theme-toggle.tsx` y dep `next-themes` (sitio es light-only por diseño, INV-014). Corregir nota "class-based theme switching" en `web/AGENTS.md:72` y en tabla Stack decisions de `.opencode/AGENTS.md` raíz. | `web/src/components/vanta/theme-provider.tsx`, `web/src/components/vanta/theme-toggle.tsx`, `web/package.json`, `web/AGENTS.md`, `.opencode/AGENTS.md` | 🟢 1-2h | 🟢 | ⏳ Pendiente |
| `INV-015-B` | **Fix touch targets < 44px** — Ajustar ~24 componentes interactivos < 44px (WCAG 2.5.8), prioridad P0→P4; los 2 icon-buttons clear-search de 14px (< 24px mínimo) primero. Fix `size-11` / `min-h-[44px] min-w-[44px]` (INV-015). | `web/src/components/vanta/*.tsx` | 🟢 2-4h | 🟡 | ⏳ Pendiente |
| `INV-016-B` | **Motion tokens de duración/easing** — Definir CSS vars `--duration-fast/normal/slow` + `--ease-default` en `globals.css` `@theme inline`; reemplazar `cubic-bezier(0.2,0.8,0.2,1)` hardcodeado — **15 tokens reales (corregido 2026-08-05):** globals.css ×12 + `reveal.tsx:64` + `faq-section.tsx:79` + `benchmark-race.tsx:233` (page-transition y latency-comparator NO contienen el token) + mapa JS para framer-motion/animejs (INV-016). | `web/src/app/globals.css`, `web/src/components/vanta/*.tsx`, `web/src/components/vanta/benchmark-race.tsx` | 🟢 1-2h | 🟢 | ⏳ Pendiente |

### 🚨 Hallazgos de Auditoría Doc↔Código (agregados 2026-08-04)

> Items creados tras la auditoría multi-agente completa (7 sub-agentes + verificación dirigida del padre) que comparó docs vs código real. Cada tarea sigue el proceso **Investigación → Análisis → Implementación**. El backlog indica qué cada fase debe entregar.

| ID | Descripción (Investigación → Análisis → Implementación) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `AUD-001` | **🐛 Fix Dockerfile (MSRV + CRATE_NAMES inexistentes)** — **Investigación:** `Dockerfile:4` `ARG RUST_VERSION=1.94.0` < MSRV `Cargo.toml:5`=1.94.1; `Dockerfile:32-39` hace `COPY vantadb-mem0/-letta/-crewai/-dspy/-haystack/-litellm/-openai/-ollama` a `target/vantadb-*` pero esos dirs NO existen (crates movidos a `integrations/`). **Análisis:** confirmar ruta real de los crates de integración y su necesidad en la imagen final; decidir si la imagen debe incluirlos (registro `docker/vanta-db-##.png` parece copy-corrrupto). **Implementación:** subir RUST_VERSION a ≥1.94.1 y corregir los COPY a los paths reales o eliminarlos; validar `docker build` + smoke-test container. | `Dockerfile`, `Cargo.toml` | 🔴 2-4h | 🔴 | ⏳ Pendiente |
| `AUD-002` | **API fantasma `GRAPH_RAG.md`** — **Investigación:** `docs/api/GRAPH_RAG.md:18-19` documenta `vantadb_python.Client()` + `client.graphrag_search()` con 0 hits en bindings Python/TS/WASM. **Análisis:** decidir si la API existe bajo otro nombre/plano (vía `VantaDB` core y pipeline RAG config en `src/sdk/`) o es ficticia. **Implementación:** o bien documentar el entrypoint real, o marcar la doc como futuro/roadmap y arreglar ejemplo. | `docs/api/GRAPH_RAG.md`, `vantadb-python/src/lib.rs`, `src/sdk/` | 🟡 2-4h | 🟡 | ⏳ Pendiente |
| `AUD-003` | **`src/governance/` no existe (tabla falsa)** — **Investigación:** `EXPERIMENTAL_GOVERNANCE_DESIGN.md` afirma tabla "verificada contra src/governance 2026-07-21" pero ese dir no existe; exports reales en `src/gds.rs` (K2). **Análisis:** mapear qué del documento corresponde a implementación real (`src/gds.rs`) vs diseño futuro no realizado. **Implementación:** retractar la afirmación de verificación, renombrar como diseño propuesto no-implementado, y corregir referencias. | `docs/Investigaciones/EXPERIMENTAL_GOVERNANCE_DESIGN.md`, `src/gds.rs` | 🟢 1-2h | 🟡 | ⏳ Pendiente |
| `AUD-004` | **`query_lisp` tool MCP expone API que el core rechaza** — **Investigación (premisa corregida 2026-08-05):** `vantadb-mcp/src/lib.rs:864` registra tool `query_lisp`; la feature del parser LISP fue ELIMINADA (CUARENTENA-01, Jul 2026) — no gate por feature inexistente; test `test_execute_hybrid_rejects_lisp` (`src/executor.rs:577`). **Análisis:** eliminar/renombrar la tool (`query_iql`) o documentar que solo acepta IQL. **Implementación:** renombrar/eliminar la tool + documentar en MCP.md. | `vantadb-mcp/src/lib.rs`, `Cargo.toml` | 🟡 1-2h | 🔴 | ⏳ Pendiente |
| `AUD-005` | **Versionado inconsistente docs/api (3 fuentes)** — **Investigación (premisa corregida 2026-08-05):** único drift real = openapi.yaml=0.4.0 vs workspace=0.5.0; MCP.md=0.5.0 es correcto; HTTP_API.md=0.0.4 coincide con el content-type real (`cli_server.rs:368`). **Análisis:** elegir única fuente de versión (lineá a workspace definida en R-2 api-contract). **Implementación:** corregir openapi.yaml + gate CI de versión (opcional valioso). | `docs/api/openapi.yaml`, `docs/api/MCP.md`, `docs/api/HTTP_API.md` | 🟢 1-2h | 🟡 | ⏳ Pendiente |
| `AUD-006` | **MCP.md: 5 de 15 tools reales sin documentar** — **Investigación (premisa corregida 2026-08-05):** MCP.md documenta 11/15; de los 8 listados previamente, 3 SÍ están (`get_node_neighbors`, `inject_context`, `read_axioms`). Faltan reales: `query_lisp` (documentada como `query`), `collection_stats`, `collection_list`, `collection_delete`, `rehydrate` = 5. **Análisis:** extraer firmas + parámetros reales de las 5. **Implementación:** documentar las 5 en MCP.md + gate de paridad tool↔doc. | `docs/api/MCP.md`, `vantadb-mcp/src/lib.rs` | 🟡 2-4h | 🟡 | ⏳ Pendiente |
| `AUD-007` | **Drift ARCHITECTURE.md vs código** — **Investigación (premisa corregida 2026-08-05):** `src/storage/wal.rs` SÍ existe; el drift es de NOMBRES de tipo y constantes: ef_construction=400 vs real 100 (`src/index/graph.rs:259`); `HnswIndex`→`CPIndex`; `WalSharded`→`ShardedWal` (`src/wal_sharded.rs:9`). **Análisis:** confirmar cada drift contra código. **Implementación:** corregir ARCHITECTURE.md con nombres y constantes reales. | `docs/architecture/ARCHITECTURE.md`, `src/index/graph.rs`, `src/wal.rs` | 🟢 1-2h | 🟡 | ⏳ Pendiente |
| `AUD-008` | **Drift STORAGE_VERSIONING.md** — **Investigación:** VECTOR_INDEX_VERSION=7 vs real 8 (`graph.rs:142`); VFILE_VERSION=1 vs real 2 (`vfile.rs:26`); WAL declarado bincode vs real postcard. **Análisis:** confirmar constantes y serialización reales. **Implementación:** corregir doc + importar constantes en vez de hardcodear. | `docs/architecture/STORAGE_VERSIONING.md`, `src/index/graph.rs`, `src/storage/vfile.rs`, `src/wal.rs` | 🟢 1-2h | 🟡 | ⏳ Pendiente |
| `AUD-009` | **Doc dicta Vite pero sitio es Next.js** — **Investigación:** DOC3 §A.5 (stack web) dice "React + Vite"; realidad `web/` es Next.js 16 App Router (`next.config.ts`, `web/src/app/`). **Análisis:** localizar todas las notas erróneas Vite/React en la doc del proyecto. **Implementación:** corregir a Next.js + non_exhaustive check de stack. | `docs/Investigaciones/` (DOC3), `web/package.json`, `web/next.config.ts` | 🟢 1-2h | 🟡 | ⏳ Pendiente |
| `AUD-010` | **Env vars naming inconsistente** — **Investigación:** code usa `VANTA_DB` (`src/cli.rs:15`, `src/cli_handlers/server.rs:244`) pero config usa `VANTADB_STORAGE_PATH` (`src/config.rs:408`). **Análisis:** elegir prefijo único `VANTADB_*`. **Implementación:** renombrar `VANTA_DB` → prefijo único, mantener alias deprecado si aplica, documentar en docs/operations. | `src/cli.rs`, `src/cli_handlers/server.rs`, `src/config.rs` | 🟡 2-4h | 🟡 | ⏳ Pendiente |
| `AUD-011` | **Deuda unsafe/unwrap/OpGate no uniforme** — **Investigación (premisa corregida 2026-08-05):** 86 `unsafe` en 16 archivos (top: `index/distance.rs` 23, `storage/vfile.rs` 22; 0 en bindings); 1904 unwraps/expects en 62.7K LOC (~1/33 líneas); `ops.rs:1761` es `expect` infalible (guard `continue` previo — no alcanzable, deuda estilística; consumir reporte INV-024 en vez de re-contar); patrón `OpGate`+`drain()` solo en `vantadb-node/src/lib.rs:75-92,277-287` (falta en python/wasm → riesgo write-after-close). **Análisis:** portar OpGate a python/wasm bindings + reemplazar unwraps críticos en hot path. **Implementación:** añadir gate de cierre en bindings restantes + propagación de error en hot path. | `src/storage/engine/ops.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs` | 🟡 1-2d | 🔴 | ⏳ Pendiente |

> **Items removidos (1):** ver `docs/progreso/BACKLOG_HISTORY.md` (P8).

---

## Phase 9: 📚 Old Docs Rescue — Reference Catalog

> Recuperado de `VANTADB DOC OLD` (~280 archivos .md analizados vía 21 sub-agentes).
> **Total:** 21 items, **13 activos** (8 ✅ removidos a progreso). **Estado:** 1 ⚠️ parcial, 1 ❌ pendiente, 2 ❌ justificado.
> **Referencia completa:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7.
> **Items removidos a progreso (8):** ver `docs/progreso/BACKLOG_HISTORY.md` (P9).

### 🔴 Alta — Features perdidas con alto valor de mercado

| ID | Feature | Esfuerzo | Estado | Dependencias | Prioridad |
|----|---------|----------|--------|--------------|-----------|
> **8 items ✅ removidos a progreso:** ver `docs/progreso/BACKLOG_HISTORY.md` (P9).

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `OLD-01` | **PGWire (PostgreSQL wire protocol)** — Compatibilidad con psql, pgAdmin, ecosistema PG | 🟠 2-3 sem | ❌ No implementado | Ninguna | 🗺️ Roadmap |
| ~~`OLD-02`~~ | ~~**GraphRAG pipeline formal** — seed → expand → retrieve → generate context. Ejemplo en `examples/rust/graphrag.rs`, no pipeline formal~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | DRV-123 (auto-embedding) recomendado | 🗺️ Roadmap |
| ~~`OLD-03`~~ | ~~**Chaos testing (Jepsen/Maelstrom)** — `ChaosTestHarness` reutilizable, 6 failpoints (wal_append, storage_insert, mmap_flush, hnsw_serialize, edge_write, snapshot_serialize), docs `docs/chaos-testing.md`.~~ | ~~🟡 2-3 sem~~ | ~~✅ COMPLETADA~~ | Docker. WAL shipping existente | 🗺️ Roadmap |
| ~~`OLD-08`~~ | ~~Life Insurance / snapshots hard-link — `SnapshotManager` + `FsSnapshot` con hard-link POSIX, `StorageEngine::create_snapshot()`/`list_snapshots()`, CLI, VantaEmbedded API. +failpoint `snapshot_create_fail`. Tests: instant, multiple, independence.~~ | ~~🟡 3-4d~~ | ~~✅ COMPLETADA~~ | Ninguna. Solo syscalls POSIX |
| ~~`OLD-09`~~ | ~~Olvido Bayesiano (hit decay) — `EvictionPolicy` ahora soporta `BayesianDecay`: score Beta-Binomial α/(α+β), threshold configurable, 31 tests. Feature-gated `bayesian_decay`.~~ | ~~🟡 3-4d~~ | ~~✅ COMPLETADA~~ | Ninguna. `EvictionPolicy` existe |
| ~~`OLD-10`~~ | ~~Sinapsis eléctrica (index-free adjacency) — `edge_index.rs` usa DashSet, no index-free adjacency nativa~~ | ~~🟡 1 sem~~ | ~~❌ Ya existe: `UnifiedNode.edges: Vec<Edge>` es index-free adjacency nativa. EdgeIndex es auxiliar para cascade delete~~ | Post-HNSW multi-capa |
| ~~`OLD-11`~~ | ~~**CLI/TUI interactivo** — `vantadb tui` con ratatui + crossterm. 3 modos: Dashboard (stats engine), Monitor (queries live), REPL (queries interactivas con historial). Feature-gated `tui`.~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | Ninguna. Proyecto aparte |
| ~~`OLD-12`~~ | ~~**Pilot program formal** — `docs/operations/PILOT_PROGRAM.md` actualizado (9 secciones), +3 templates: agreement, feedback, onboarding checklist.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | PyPI publicado |
| ~~`OLD-14`~~ | ~~MessageThread / GcWorker para agentic chat — `GcWorker` en `src/gc.rs` existe, MessageThread no~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | Ninguna. `GcWorker` existe |
| ~~`OLD-16`~~ | ~~**WAL rotation a 256MB** — `WalWriter::try_auto_rotate()` en `append()`/`batch_append()`. 3 tests (trigger, no-trigger, data preservation). 52/52 WAL tests pass.~~ | ~~🟢 1d~~ | ~~✅ COMPLETADA~~ |
| ~~`OLD-19`~~ | ~~**Rehidratación desde shadow archive** — `VantaEmbedded::recover_archived_nodes()`, MCP tool `rehydrate`, Python binding. Conecta `StorageEngine::recover_archived_nodes()` (6 tests existentes) a SDK público.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | OLD-07 (AutoHot/Cold tiering) |
| ~~`OLD-20`~~ | ~~Contextual Priming (cache warming predictivo) — ✅ COMPLETADA. Auto-decay cada 1000 eventos, métricas exportables, co-access tracking ya conectado en hot path.~~ | ~~🟢 2-3d~~ | ~~✅ COMPLETADA~~ | ~~Ninguna~~ |
| ~~`OLD-21`~~ | ~~**CP-Index formal (query routing inteligente)** — `CostEstimator::select_index_strategy()` (Flat ≤ flat_threshold, IVF ≥ 10K, HNSW default; respeta config explícita). Conectado en `vector_memory_search` (métrica `record_vector_index_routing`). Admission budget de `Executor::execute_plan` ahora usa `ResourceGovernor::estimate_plan_cost` (removido dead_code) en vez de MIB fijo. 6 tests nuevos, 1816 tests ✅.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | COMP-028 ✅ |

---

## Phase 10: 🏗️ Competitive Features — Catalog

> **Fuente:** Análisis de 27 archivos de `VANTADB DOC OLD/` (9 vector DBs + 8 graph DBs + 10 arquitectura).
> **Total:** 30 items, **18 activos.** 12 ✅ implementados removidos a progreso: ver `docs/progreso/BACKLOG_HISTORY.md` (P10).
> **Reportes completos:** `docs/audit-reports/competitive-features-consolidated-report.md`, `docs/audit-reports/deep-analysis-{vector,graph,arch}.md`

### 🔴 Alta — Features competitivas críticas para adopción

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| ~~`COMP-006`~~ | ~~**Edge Label Interning (u32 label_id)** — `edge_label` es `String`, no u32 internado~~ | ~~🟢 ~2d~~ | ~~✅ COMPLETADA~~ | ~~Ninguna~~ |

### 🟠 Media-Alta — Features competitivas importantes

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| ~~`COMP-008`~~ | ~~**Pluggable index engine (VecIndex trait)** — `VecIndex` trait definido con `search`/`add`/`len`/`estimate_memory_bytes`. Implementado para CPIndex (HNSW) e IvfIndex. `vector_memory_search` usa `engine.vec_index()`. 1679 tests ✅. 🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | ~~Pre-COMP-027~~ |
| ~~`COMP-009`~~ | ~~**Binary bulk import (5-10x faster than INSERT)** — `bulk_import_stream()` + `bulk_import_file()` + `bulk_import_bytes()` en core Rust, Python, WASM. Formato `VDBJSON\n` header + serde_json body. Bypass per-record validation, batch commit, 3 tests. ✅ COMPLETADA~~ | ~~🟢 3-4d~~ | ✅ | Ninguna |
| ~~`COMP-010`~~ | ~~**Auto-embedding (embedding function abstraction)** — Trait `EmbeddingProvider` + `OllamaProvider` + `OpenAIProvider`. Factory `get_embedding_provider()` vía `VANTA_EMBEDDING_PROVIDER`. ✅ COMPLETADA | 🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | ~~DRV-123~~ |
| ~~`COMP-012`~~ | ~~**RoaringBitmaps for metadata indexing** — `FilterBitset` migrado a `croaring::Bitmap`. 19/19 tests ✅, serializado ~10× más pequeño. `DiskNodeHeader` intacto (u128).~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | Pre-COMP-003 ✅ |
| ~~`COMP-013`~~ | ~~**Segment optimizer pipeline (Vacuum/Merge/Index)** — `PipelineMode`, `VacuumReport`, `MergeReport`, `PipelineReport`, `SegmentOptimizerConfig`, `vacuum()`, `merge_segments()`, `run_pipeline()` + SDK API + 77 tests. ✅ COMPLETADA~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | ~~COMP-004, COMP-011~~ |
| ~~`COMP-014`~~ | ~~**FreshHNSW (background repair de enlaces huérfanos)** — `repair_orphan_links()` en CPIndex (three-phase: snapshot → scan → repair). `FreshHnswReport`, `PipelineMode::FreshHnswOnly`, pipeline phase, 4 tests. Fix: deadlock DashMap. ✅ COMPLETADA~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | ~~COMP-004, COMP-011~~ |
| ~~`COMP-016`~~ | ~~**Supernode mitigation (indexed relationships)** — `label_index: HashMap<u32, Vec<u128>>` en `UnifiedNode`. `bfs_traverse_filtered`/`dfs_traverse_filtered` en `GraphTraverser`. `graph_bfs_filtered`/`graph_dfs_filtered` en SDK, WASM, Python. 6 tests. ✅ COMPLETADA~~ | ~~🟢 3-5d~~ | ~~✅ COMPLETADA~~ | ~~COMP-006~~ |
| ~~`COMP-017`~~ | ~~**Accumulators for parallel graph algorithms** — `GraphAccumulator` con `AtomicU64` lock-free, `traverse_with_accumulator` en `GraphTraverser`, SDK API, 6 tests. ✅ COMPLETADA~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | Ninguna |
| ~~`COMP-018`~~ | ~~Double-linked relationship chains — Relaciones dirigidas simples, sin doble enlace — **✅ COMPLETADA** — Rust SDK (4 métodos con direction param), bindings WASM + Python, 33 graph tests pasan~~ | ~~🟡 1-2 sem~~ | ~~✅~~ | COMP-006 |
| ~~`COMP-019`~~ | ~~**Binary protocol (rkyv/FlatBuffers over gRPC)** — ❌ **WONTFIX** (ADR `COMP-019-binary-protocol-wontfix.md`). gRPC contradice posicionamiento embedded-first. rkyv ya cubre serialización interna. Sin demanda ni dependencias.~~ | ~~🟡 1-2 sem~~ | ~~✅ Cerrado — WONTFIX~~ | Ninguna |
| ~~`COMP-021`~~ | ~~**Temporal edges (timestamp-aware relationships)** — `Edge.created_at_ms` con custom `Deserialize` para backward-compat postcard, `bfs_traverse_filtered`/`dfs_traverse_filtered` con `time_range`, `add_edge` con `created_at_ms`, bindings Python/WASM/TS, 13 tests (7 unit + 6 certification). ✅ COMPLETADA~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | ~~Ninguna~~ |
| ~~`COMP-022`~~ | ~~**Graph Data Science library (PageRank, centrality)** — `GraphDataScience` con `page_rank()` + `degree_centrality()`, SDK API, Python bindings, 7 tests. ✅ COMPLETADA~~ | ~~🟡 2-3 sem~~ | ~~✅ COMPLETADA~~ | ~~COMP-017~~ ✅ |
| ~~`COMP-023`~~ | ~~**3 filtering strategies (pre/post/in-index)** — `FilterStrategy::PreFilter/InFilter/PostFilter` con selector por joint selectivity. PreFilter (< 1%): scan+brute-force. InFilter (1-10%): query_mask via HNSW. PostFilter (≥ 10%): vector→filter. 1589 tests ✅.~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | COMP-003 ✅, COMP-012 ✅, COMP-028 (diferida) |
| ~~`COMP-024`~~ | ~~**ACORN algorithm (second-hop filtered search)** — `acorn_expansion` param en + `search_layer()` con 2-hop expansion. 3 tests: exp | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | COMP-003 ✅, COMP-023 ✅, COMP-012 ✅ |

### 🟡 Medio — Features de madurez y ecosistema

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| ~~`COMP-025`~~ | ~~**JSON shredding (dynamic schema to columns)** — Phase 1: schema inference + columnar storage + filter integration. Phase 2: typed comparison filters (Gt/Lt/Gte/Lte/Neq). `matches_shredded` con 6 operadores, 13 tests.~~ | ~~🟡 2-3 sem~~ | ~~✅ COMPLETADA~~ | Ninguna |
| ~~`COMP-026`~~ | ~~**Multi-level LSM compaction (L0→L1→L2→L3)** — SegmentRegistry, compact_level(), PipelineMode::CompactOnly/L0Only in run_pipeline. 13+ archivos modificados. `cargo check -p vantadb` ✅~~  | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | COMP-013 |
| ~~`COMP-027`~~ | ~~**Multiple index types (IVF, DiskANN, SCANN)** — FlatIndex (brute-force), DiskAnnIndex (Vamana graph + robust pruning), ScannIndex (SQ8 scalar quantization). IndexType enum extendido: `Flat`, `DiskAnn`, `Scann`. `create_index()` factory. 15 tests pasan.~~ | ~~🟠 5-10d~~ | ~~✅ COMPLETADA~~ | ~~COMP-008~~ |
| ~~`COMP-028`~~ | ~~**Semantic Cost Estimator (SCE)** — módulo `src/cost_estimator.rs` unificado: `CostEstimator::selectivity`/`estimate_operator`/`estimate_plan`, `FilterStrategy` movido de search, `select_filter_strategy` delegada. CBO + `ResourceGovernor::estimate_plan_cost` (unwired, consumido por OLD-21). `get_estimated_selectivity` delega (firma pública intacta, 21 callers). 4 tests nuevos, 1776 tests ✅. Commit `f7cb46e4`.~~ | ~~🟡 2 sem~~ | ~~✅ COMPLETADA~~ | DRV-121/122 ✅ |
| ~~`COMP-029`~~ | ~~**Node.js/TS bindings via napi-rs** — **Implementado (2026-08-02):** crate `vantadb-node` (standalone, `vantadb_native` cdylib) como backend ADICIONAL a WASM. API isomórfica con `vantadb-ts`. Persistencia real (fjall/WAL/fsync) — WASM no puede. Tests vitest 3/3: put/get, persistencia cross-reconnect, search ordenado por score. Browser se queda con WASM.~~ | ~~🟡 2-3 sem~~ | ~~✅ Implementado~~ | ~~Ninguna~~ |

---

## Phase 11: 🐙 GitHub Issues — Backlog de Issues Abiertos

> Issues abiertos en `ness-e/Vantadb` convertidos a tareas del backlog (2026-08-01).
> **Total:** 14 tareas (issues #119–#144). Todos son `good first issue`.
> ⚠️ **Recordatorio obligatorio:** al completar cada tarea, **revisar el resultado contra el issue y cerrarlo en GitHub** (`gh issue close <NUM> --repo ness-e/Vantadb`), confirmando que se cumple su Definition of Done.
> Estado real se actualiza en la columna **Estado**.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| ~~`GH-144`~~ | ~~**i18n: Traducciones ES para showcase page**~~ — ~~Auditar `web/src/app/showcase/page.tsx` y completar claves de diccionario i18n sin traducción al español~~ | ~~`web/src/lib/dictionaries.ts`~~ | ~~🟢 2-3h~~ | ~~🟡~~ | ✅ RESUELTA 2026-08-05: 22 claves `showcasePage.*` ES (1370-1391) + EN (2856-2877) completas; página usa `tt()` con fallback. Issue #144 cerrado con evidencia. Migrada a progreso. |
| ~~`GH-143`~~ | **✅ ci: Acelerar CI con sccache y paralelización** — El pipeline CI corre build/test/clippy pero puede optimizarse. **Task:** (1) habilitar `sccache` para cachear compilación Rust, (2) evaluar reemplazar `cargo build` por `cargo check` donde baste, (3) paralelizar jobs independientes. **DoD:** CI ≥20% más rápido; todos los checks existentes pasan. **Cierre:** revisar y cerrar issue #143. Relacionado con `INV-017` (sccache investigación). | `.github/actions/rust-setup/action.yml`, `.github/workflows/ci-rust-10.yml` | 🟡 2-4h | 🟡 | ✅ Hecho |
| ~~`GH-142`~~ | **✅ ci: Smoke tests de examples en CI** — Workflow `ci-examples-12.yml`: 4 examples Rust (`cargo run --example`) + 10 Python (wheel maturin local, 1 step por example, sin `continue-on-error`). Se repararon 7 examples Python con drift de API (`db.list`→`list_memory`, `query_vector=None`→`[]`). Todos pasan. Migrada a progreso. | `.github/workflows/` (nuevo job) | 🟡 2-4h | 🟠 | ✅ Hecho |
| `GH-141` | **docs: Documentar integración webhook GitHub→Discord** — El webhook de Discord envía eventos push/PR/release a #announcements pero no está documentado. **Task:** (1) qué eventos se trackean, (2) cómo agregar nuevos tipos de evento, (3) dónde se configura (server settings). **DoD:** integración documentada con tipos de evento y mapeo de canales; instrucciones para agregar eventos. **Cierre:** revisar y cerrar issue #141. | `docs/discord/server-config.md` (sección Integrations) | 🟢 1-2h | 🟢 | ❌ Desde cero |
| `GH-140` | **chore: Auditar y eliminar CSS no usado** — `web/src/app/globals.css` puede tener clases sin uso. **Task:** (1) encontrar clases no referenciadas en componentes, (2) eliminar CSS muerto, (3) verificar sin regresiones visuales. **DoD:** ≥10% reducción de tamaño CSS; sin cambios visuales; método de auditoría documentado. **Cierre:** revisar y cerrar issue #140. | `web/src/app/globals.css` | 🟡 2-4h | 🟢 | ❌ Desde cero |
| ~~`GH-139`~~ | ~~**feat: GIF demo animado para README**~~ — ~~README hoy es solo texto + badges SVG (sin PNG estática; premisa corregida 2026-08-05). **Slice de NUEVO-01** (hero).~~ — ~~**Task:** crear GIF animado mostrando (1) `pip install vantadb-py`, (2) REPL Python con CRUD, (3) resultado de hybrid search.~~ | ~~`README.md` (hero), `assets/` (nuevo gif)~~ | ~~🟡 2-3h~~ | ~~🟢~~ | 🔁 FUSIONADO en NUEVO-01 (Task 41, 2026-08-05): NUEVO-01 gana el deliverable GIF <5MB como sub-paso. Tachada — cerrar issue #139 al completar NUEVO-01. |
| `GH-132` | **feat: Badge Google Colab + notebook quickstart** — Muchos devs descubren proyectos vía Colab. **Task:** (1) crear `examples/colab/vantadb_quickstart.ipynb` (install, CRUD básico, hybrid search), (2) agregar badge "Run in Colab" a README. **DoD:** notebook corre end-to-end en Colab; badge renderiza en README. **Cierre:** revisar y cerrar issue #132. | `examples/colab/vantadb_quickstart.ipynb` (nuevo), `README.md` | 🟡 2-4h | 🟢 | ❌ Desde cero |
| `GH-131` | **docs: Documentar integración mem0 en README** — Existe `examples/python/mem0_integration.py` sin documentar. **Task:** sección mem0 en README: descripción breve, snippet de código, link al ejemplo completo. **DoD:** integración mem0 documentada; snippet verificado funcionando. **Cierre:** revisar y cerrar issue #131. | `README.md` | 🟢 1-2h | 🟢 | ❌ Desde cero |
| `GH-129` | **feat: Ejemplo de integración Semantic Kernel** — Existe `examples/python/semantic_kernel_memory.py` sin conectar a docs/README. **Task:** (1) sección SK en README con snippet, (2) verificar con último SDK de Semantic Kernel, (3) agregar imports/setup faltantes. **DoD:** integración documentada; ejemplo verificado; CI corre smoke test del ejemplo. **Cierre:** revisar y cerrar issue #129. | `README.md`, `examples/python/semantic_kernel_memory.py` | 🟡 2-3h | 🟢 | ❌ Desde cero |
| `GH-128` | **docs: Ejemplo retriever DSPy en README** — Existe `examples/python/dspy_retriever.py` sin mencionar en README. **Task:** sección DSPy: descripción breve, snippet de uso básico, link al ejemplo completo. **DoD:** integración DSPy documentada; snippet preciso y testeado. **Cierre:** revisar y cerrar issue #128. | `README.md`, referencia `examples/python/dspy_retriever.py` | 🟢 1-2h | 🟢 | ❌ Desde cero |
| ~~`GH-124`~~ | **✅ docs: Ejemplos doc-test para API pública Rust** — 7 doc-tests agregados (`open`, `open_with_config`, `put`, `get`, `delete`, `search`, `VantaConfig`) + 2 doc-tests rotos reparados. `cargo test --doc` 11/11. Migrada a progreso. | `src/lib.rs` o módulos relevantes | 🟡 3-5h | 🟡 | ✅ Hecho |
| `GH-123` | **docs: Corregir typos y links rotos en docs** — `docs/` acumuló typos, links rotos y referencias desactualizadas en 167+ archivos. **Task:** (1) correr spell checker en `.md` de `docs/`, (2) verificar links internos, (3) corregir versiones desactualizadas. **DoD:** typos corregidos; links internos resuelven; referencias de versión actualizadas. **Cierre:** revisar y cerrar issue #123. | `docs/**` | 🟡 2-4h | 🟢 | ❌ Desde cero |
| ~~`GH-122`~~ | **✅ docs: Docstrings en API pública del Python SDK** — 12/12 métodos de `VantaDB` documentados (Args/Returns/Raises/ejemplo runnable ` ```python `) en `vantadb-python/src/lib.rs`, visibles vía PyO3. Docstring de clase con constructor. check/fmt/clippy clean. Migrada a progreso. | `vantadb-python/src/lib.rs` | 🟡 3-5h | 🟡 | ✅ Hecho |
| ~~`GH-119`~~ | ~~**docs: Guía de migración Vectara → VantaDB** — Vectara cerró su tier self-service en 2026; muchos equipos buscan alternativas local-first. **Task:** crear `docs/tutorials/migrate-from-vectara.md` cubriendo: diferencias de arquitectura (hosted vs embedded), exportar corpus (endpoint `corpus-export`), re-embedding (vectores Boomerang no portables), mapeo de API (corpus Vectara → namespace VantaDB). **DoD:** guía cubre workflow completo de migración; incluye ejemplos Python funcionales. **Cierre:** revisar y cerrar issue #119. Material de research: `docs/audit-reports/vectara-competitive-research-2026-07-27.md`.~~ | ~~`docs/tutorials/migrate-from-vectara.md` (nuevo)~~ | ~~🟡 1-2d~~ | ~~🟠~~ | ✅ Guía creada 2026-08-04 (259 líneas, workflow 4 pasos + ejemplos Python), issue #119 cerrado, commit ebfb3363 |

---

## Phase 12: 🖥️ DESKTOP — App de Escritorio Tauri Multi-Connection

> **Objetivo:** App de escritorio Tauri v2 en `desktop/` que conecta la UI con VantaDB a través de **cualquiera de las 6 integraciones** (crate nativa, `vantadb-server` HTTP, `vantadb-mcp` stdio, `vantadb-node` napi, `vantadb-python` PyO3, `vantadb-ts`/`vantadb-wasm` webview), individualmente o varias simultáneas, para máxima compatibilidad/rendimiento/seguridad.
> **Base de decisión:** `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` (✅ SÍ — Tauri v2, vía nativa óptima) + **`docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`** (investigación completa de las 6 integraciones + arquitectura multi-connection de vanta-arch: trait `VantaConnection` + `ConnectionManager`, default = crate `vantadb` embebida, regla "un escritor por path de DB").
> **Contexto de integraciones (investigado 2026-08-04):** server = HTTP REST `/api/v2/query` (IQL) en `127.0.0.1:8080`, auth Bearer, sin streaming; MCP = JSON-RPC 2.0 **solo stdio** (15 tools), proceso es la DB; node = addon napi-rs (Tauri no puede `require()`, solo sidecar); python = PyO3, sin CLI (requiere driver script); ts/wasm = snapshot JSON in-memory, read-only/demo.
> **Regla de fragmentación:** 1 tarea = 1 concepto; nunca mezclar 2 integraciones en una tarea. `desktop/` usa `[workspace]` vacío en `src-tauri/Cargo.toml` → desacoplado del workspace raíz (no toca CI/versiones de core).

### Fase 0 — Scaffold

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-02` | **Scaffold Tauri v2 + propio workspace** — `create-tauri-app` en `desktop/`; `src-tauri/Cargo.toml` con `[workspace]` vacío (desacopla del raíz); `tauri.conf.json`, capabilities mínimas, command `ping`; frontend React+Vite mínimo. **DoD:** `npm run tauri dev` abre ventana y el botón ping responde; `cargo check` en `src-tauri` pasa; `cargo check` raíz sigue igual (sin cambios en workspace). | `desktop/src-tauri/*`, `desktop/package.json` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-03` | **Integrar crate `vantadb` + managed state + healthcheck** — Dep `vantadb` con `default-features=false` + `fjall,fs2,memmap2,roaring,advanced-tokenizer` (nunca `cli`/`server`/`prometheus`); `AppState { manager, config }` managed; command `vanta_health` que abre `VantaEmbedded` en temp dir y reporta capabilities. **DoD:** `vanta_health` devuelve `HealthReport` con `backend=fjall`; abrir dos veces el mismo path falla con error de lock. | `src-tauri/Cargo.toml`, `src/lib.rs`, `src/commands/connection.rs` | 🟢 | 🔵 | ❌ Desde cero |

### Fase 1 — Trait + adaptador nativo + UI mínima

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-04` | **Trait `VantaConnection` + tipos + errores** — `async_trait` object-safe; tipos compartidos (`IngestItem`/`SearchQuery`/`SearchResult`/`MemoryRecord`/`HealthReport`/`ConnectionInfo`/`Capability`); `VantaError` unificado (`#[non_exhaustive]`, variant por vía: Native/Http/Mcp/Node/Python/Wasm + Lock/Timeout/Unsupported...). **DoD:** compila; tests unitarios de serde roundtrip de todos los tipos. | `src/connections/{trait,types}.rs`, `src/error.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-05` | **`NativeConnection`** — `VantaEmbedded` embebida, ops síncronas en `spawn_blocking`, mapeo de errores, `capabilities()`, lock del path. **DoD:** test integración put/search/get/delete en temp dir vía trait; segunda conexión mismo path → `VantaError::Lock`. | `src/connections/native.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-06` | **Commands CRUD async** — `vanta_connect/disconnect/list_connections/set_active/ingest/ingest_batch/search/get/delete/list` delegando al adaptador activo (solo nativo por ahora). Keys/namespaces como `String` (limitación `&str` en async). **DoD:** E2E manual: conectar nativo, ingest 3, search devuelve resultados ordenados. | `src/commands/{connection,data}.rs` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-07` | **Frontend MVP** — React+Vite en `desktop/` reusando tokens de `web/`; `ConnectionPanel`, `IngestForm`, `SearchBar`, `ResultsList`, hook `useConnectionState`; bridge `vanta.ts` (wrapper tipado de `invoke`). **DoD:** UI permite conectar nativo, ingresar y buscar; badge de health. | `desktop/src/*` | 🟡 | 🔵 | ❌ Desde cero |

### Fase 2 — Adaptador Server (HTTP)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-08` | **Cliente IQL tipado** — Wrapper reqwest (json): config url/port/token/timeout. **Premisa corregida 2026-08-05:** la API real tiene 3 endpoints (/health, /metrics, /api/v2/query IQL); put/get/delete/list/search van como statements IQL → diseñar 'cliente IQL tipado', NO client REST por-op — **validar contra `docs/api/HTTP_API.md` y `src/cli_server.rs`**. **DoD:** tests contra mock HTTP server (axum dev-deps): statements IQL mapeados y autenticados. | `src/connections/server_client.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-09` | **`ServerConnection`** — Implementa el trait sobre el client; connect valida auth/health; mapeo a `VantaError::Http`; timeouts; tratar `success:false` del body como error de dominio (el server devuelve 200 con body de fallo). **DoD:** integración contra `vantadb-server` real (spawn con `VANTADB_API_KEY` + `--require-auth`): health/put/search ok; server caído → error `Http` limpio. | `src/connections/server.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-10` | **Wire Server en commands + UI** — Selector muestra vía "Server" con campos url/puerto/token; conexión entra al registry y puede ser activa. **DoD:** desde la UI, conectar a server real, ingest + search por HTTP. | `src/commands/connection.rs`, `desktop/src/components/ConnectionSelector.tsx` | 🟢 | 🔵 | ❌ Desde cero |

### Fase 3 — Adaptador MCP (stdio)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-11` | **Spawn manager subproceso MCP** — Localizar binario `vantadb-server` (dev: `target/debug/`; release: bundled); confirmar flag `--mcp` en `vantadb-server/src/main.rs`; `tokio::process::Command` con stdio piped, stderr a log, timeout de arranque. **DoD:** spawn + kill limpio; flag MCP confirmado; stderr capturado. | `src/connections/child_process.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-12` | **Cliente rmcp** — Dep `rmcp` (`client`, `transport-child-process`); `TokioChildProcess` + init handshake, `list_tools`, `call_tool` con params serde_json. **DoD:** conecta al binario real; `list_tools` devuelve las 15 tools (memory_put/get/delete/list, search_memory, query_lisp, collection_*...). | `src/connections/mcp_client.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-13` | **`McpConnection`** — Mapea las 15 tools al trait (`memory_put`→ingest, `memory_get`→get, `memory_delete`→delete, `memory_list`/`collection_list`→list, `search_semantic`/`search_memory`→search, `query_lisp`+`collection_*` fuera del trait MVP); `VantaError::Mcp`. **DoD:** integración real: ingest + search vía MCP en temp dir; tool inexistente → error mapeado. | `src/connections/mcp.rs` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-14` | **Healthcheck/reconnect/UI MCP** — Healthcheck por tool trivial; `close` = `graceful_shutdown` con kill-timeout; selector "MCP" en UI. **DoD:** desconectar mata el proceso (verificado); reconectar funciona. | `src/connections/mcp.rs`, `ConnectionSelector.tsx` | 🟢 | 🔵 | ❌ Desde cero |

### Fase 4 — Node y Python (opcionales, feature-gate)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-15` | **JSON-RPC cliente + driver Node** — Framing newline-delimited, ids incrementales, mapa de respuestas pendientes, timeout (`connections/jsonrpc.rs`, compartido con Python); driver `drivers/node/driver.js` que `require('vantadb_native')` y sirve stdio. **DoD:** `node driver.js` responde put/search por stdio (test manual con pipe). | `src/connections/jsonrpc.rs`, `drivers/node/driver.js` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-16` | **`NodeConnection`** — Spawn node (dev: sistema; release: sidecar `externalBin`), IPC vía jsonrpc, mapeo `VantaError::Node`, capabilities. **DoD:** integración: put/search vía node en temp dir. | `src/connections/node.rs`, `tauri.conf.json` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-17` | **Driver Python + decisión runtime** — `drivers/python/python_driver.py` (import `vantadb_py`, JSON-RPC stdio); **decisión documentada aquí**: runtime python del sistema vs bundled (MVP = sistema primero). **DoD:** driver responde put/search con python local (si hay runtime; si no, skip documentado). | `drivers/python/python_driver.py` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-18` | **`PythonConnection`** — Spawn python + driver, reusa `jsonrpc.rs`, mapeo `VantaError::Python`. **DoD:** integración opcional (skippable sin runtime python). | `src/connections/python.rs` | 🟡 | 🔵 | ❌ Desde cero |

### Fase 5 — ConnectionManager multi-connection

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-19` | **ConnectionManager completo** — Registry multi (`HashMap<id, Arc<dyn VantaConnection>>`), active, `path_holders` (regla 1-escritor por path → `VantaError::Lock` + hint), routing por `connection` id, capability gate (write sobre read-only → `Unsupported`). **DoD:** Nativa + Server (paths distintos) conectadas simultáneamente; conectar segunda vía sobre el mismo path → rechazada con hint. | `src/connections/manager.rs` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-20` | **Lifecycle shutdown_all** — `shutdown_all` en `RunEvent::ExitRequested`: orden webview → subprocesos → nativa última (flush); timeout configurable + kill forzoso. **DoD:** cerrar app con MCP+Node+Python conectados no deja procesos huérfanos (verificado). | `src/lib.rs`, `src/connections/manager.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-21` | **UI multi-connection** — Selector con N vías conectadas, switch de activa, health badge por vía, warning de conflicto de path. **DoD:** UI muestra 2 vías vivas; la op va a la activa; warning al intentar conflicto. | `ConnectionSelector.tsx`, `ConnectionPanel.tsx` | 🟡 | 🔵 | ❌ Desde cero |

### Fase 6 — Streaming, config, empaquetado, CI, tests, docs

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-22` | **Eventos Tauri (streaming)** — `vanta://connection-state` (obligatorio) + `ingress-progress`/`search-progress` (flag `progress`); listeners en frontend. **DoD:** ingest batch de 1000 items emite progreso sin bloquear la UI. | `src/lib.rs`, `desktop/src/hooks/*` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-23` | **Persistencia de config** — JSON en `app_config_dir`, load en setup, save atómico (`temp + rename`), defaults, vías guardadas. **DoD:** reiniciar la app conserva vías guardadas y vía activa. | `src/config.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-24` | **Empaquetado** — Bundle NSIS/MSI (Windows primero), `externalBin` (node + `vantadb-server.exe` + runtime python si procede), identifier, icons, auto-update opcional. **DoD:** instalador produce una app que conecta nativo + server + node sin entorno de dev. | `tauri.conf.json`, `src-tauri/build.rs` | 🔴 | 🔵 | ❌ Desde cero |
| `DESKTOP-25` | **CI GitHub Actions** — Build Windows (tauri-action), `cargo test` en `src-tauri` (workspace desacoplado), `npm build` frontend, artefacto instalador; matrix por features de vías (con/sin server/mcp). **DoD:** pipeline verde; artefacto instalador subido. | `.github/workflows/desktop.yml` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-26` | **Tests** — Unit: tipos, mapping de errores, framing jsonrpc; integración por adaptador (mock HTTP, MCP real, nativa temp); **contrato de errores**: misma op en N vías → mismo shape `VantaError`. **DoD:** `cargo test` + integraciones en CI. | `src/**/*_tests.rs` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-27` | **Docs + ADR** — README desktop, `ARCHITECTURE.md` (modelo conexión), **ADR** multi-connection + regla 1-escritor (siguiente número libre en `docs/architecture/adr/`), guía de usuario por vía, actualizar DESKTOP-01 con decisiones. **DoD:** ADR revisado por vanta-arch; guía cubre las 6 vías. | `docs/desktop/*`, `docs/architecture/adr/ADR-0XX.md` | 🟢 | 🔵 | ❌ Desde cero |

> **Total:** 26 tareas (DESKTOP-02..27) — 13 🟢, 10 🟡, 1 🔴, 2 condicionales (Node/Python feature-gate). Secuencia: Fase 0 → 1 → (2/3/4 en paralelo) → 5 → 6.

---

## Referencias Cruzadas

- **RC items:** `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md` (generado por `vantadb-full-review` skill)
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados

---


