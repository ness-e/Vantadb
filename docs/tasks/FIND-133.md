# FIND-133 — Semver Checks rojo (develop vs crates.io 0.5.0): triage + veredicto

> **Plan:** `docs/plans/2026-09-19-cierre-total.md` (Wave A, archivos disjuntos con CODEX + FIND-128 — no tocar los suyos)
> **Estado:** 🟡 INCOMPLETO (triage completo, verde bloqueado por mecánica de versión — ver Step 5)
> **Gate P:** owner aprobó "Hacerlo pasar" (2026-09-19). Gates D/V/C vía question (sin tool → motivo registrado).
> **Contrato:** `cargo semver-checks -p vantadb` verde local + cada breaking con veredicto
> intencional-0.6.0 (con ADR Regla 5) o accidental-revertido/`#[non_exhaustive]`/`#[doc(hidden)]`
> + versión solo vía release-plz (no manual). Cero refactors.
> **SDP:** campaign-executor · source-driven-development · doubt-driven-development ·
> incremental-implementation · test-driven-development · context-engineering ·
> api-and-interface-design · systematic-debugging · documentation-and-adrs ·
> git-workflow-and-versioning (discover_v2 BUILD + keywords semver/breaking/non_exhaustive/ADR/release-plz/cargo;
> frontend-ui-engineering descartado — `web/` prohibido en esta tarea)

## Alcance

- **Clave (leídos):** `src/cli.rs` (§40-179, :416-424) · `src/query.rs:105,133,368-431` ·
  `src/sdk/types.rs` (head + re-exports) · `src/metrics/core/snapshot.rs` (completo) ·
  `src/backend.rs:34-55` · `src/node/flags.rs:113-134` · `src/error.rs:122` ·
  `src/agentic/thread.rs:106-292` · `src/wal_shipping.rs:160` · `src/config.rs:107-126,475` ·
  `src/storage/vfile.rs:112` · `src/server/state.rs:15,114-134` · `src/llm.rs` (vía semver) ·
  `src/graph.rs` (vía semver) · `src/planner.rs:180` · `src/migration.rs:13`.
- **Relacionados (leídos):** `docs/architecture/adr/ADR-014-pitr.md` (SUPERSEDED, removal documentado),
  `006_rrf_constant.md`, ADR-041 (major directo sin aliases), ADR-042 (fallo semver documentado),
  `release-plz.toml` (semver_check=true), `docs/CHANGELOG.md` ([Unreleased] ya acumula BREAKING 0.x),
  `.github/workflows/ci-rust-10.yml:90-126` (scope: main + PR-a-main; en develop rojo esperado),
  baseline `vantadb-0.5.0` del registry (tipos/firmas originales como evidencia).
- **PROHIBIDOS (intocados):** `vantadb-mcp/src/handlers/tools.rs`,
  `vantadb-server/docker-compose.yml`, `vanta-memory/src/services/conversation_hook.rs`,
  `web/`, `vantadb-ts/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`,
  `*.lock`, stash GOV-C4, `docs/Backlog.md`, plan file (solo recitation), `C:/Users/Eros/.vantadb*`.

## Baseline local (Step 1 ✅)

`cargo semver-checks -p vantadb` (v0.49.0, CARGO_BUILD_JOBS=2, baseline crates.io 0.5.0,
salida en `Temp/opencode/semver-clean.txt`): **21 categorías de fallo, ~100 ítems**
(>> "~20" del enunciado — el enunciado lista representantes). Proceso en background por
build rustdoc >10 min; resultado completo capturado.

## Triage intencional-vs-accidental (Step 2 ✅) — 0 accidentales

| # | Breaking | Sitio actual | Veredicto | Evidencia (código, no opinión) |
|---|---|---|---|---|
| 1 | `Vanta*` structs/enums fuera (~25 structs + 6 enums: FilterOp, Value, MemoryInput/Record, OperationalMetrics, QueryResult, Error…) | `src/sdk/types.rs`, `graph.rs`, `builder.rs`, `serialization/*`, `src/error.rs:122` (`pub enum Error`) | INTENCIONAL-0.6.0 | `d524c67b refactor!: AST-002 tipos sin stutter + aliases` → `1a566ef3 refactor!: AST-010 quitar aliases (rename directo, 0 usuarios)`; cero ocurrencias `Vanta*` en `src/` |
| 2 | `wal_archiver` mod + `WalArchiver`/`PitrRestorer`/`WalArchiveConfig` + feature `pitr` fuera | eliminado (`Test-Path src/wal_archiver.rs` = False; `pitr` sin match en Cargo.toml) | INTENCIONAL (+ADR existe) | `bde2fc9e FIND-26 remove dead PITR` + ADR-014 SUPERSEDED (dead code, cero call sites) |
| 3 | `VantaConfig` fuera | `src/config.rs` (solo `HotReloadConfig` matchea) | INTENCIONAL (+ADR existe) | ADR-043 F3C split-config-datos |
| 4 | planner consts (`RRF_K`, `MAX/MIN_CANDIDATE_BUDGET`, `CANDIDATE_MULTIPLIER`) + fns (`trimmed_text_query`, `fuse_rrf[_with_report]`, `hybrid_candidate_budget`, `sort_hits`) fuera | `src/planner.rs` (solo mención en comentario ponytail :180) | INTENCIONAL | `6a50b8ee feat(planner): SearchProfileConfig per request (MEM-01)` los supersede; ADR-006 contexto RRF |
| 5 | `LogicalOperator` +`TextFilter`/+`Dedup` (shifts 4→5…8→10); `BackendPartition` +`SparseIndex`/+`Versions` (`InternalMetadata` 7→8); `DistanceMetric::SparseDot`; `Condition::TextMatch`; `WalRecord::Prepare`; `FormatKind::File` nuevo / `::VantaFile` fuera | `query.rs:368-411`, `backend.rs:50-55`, `vector_data.rs:20`, `migration.rs:13`, `wal.rs:80` | INTENCIONAL | features 0.6.0 (sparse ADR-011/019, VS-CORE-07 versions, dedup C2S6, migración formato); shifts = efecto colateral de inserción, no cambio semántico |
| 6 | `QueryResult` struct→enum | `sdk/types/graph.rs:14` | INTENCIONAL | `e3711dea FIND-49 split sdk types por dominio` |
| 7 | `Commands` growth (`Put.metadata` :58, `Export.format` :117, `Restore.dry_run` :164, `Server.allow_insecure/dashboard_dir`, `Doctor` unit→struct :168, `WalCommand::Salvage` + non-unit :416-424) | `src/cli.rs` | INTENCIONAL | CLI features (export md, doctor fix/force, salvage); leído :40-179 |
| 8 | Campos nuevos (`OperationalMetricsSnapshot` +14, `ServerState` +7, `Query`/`LogicalPlan.search_profile`, `Edge.created_at_ms`, `Cli.memory_limit`) | `snapshot.rs` (leído completo), `state.rs:114-134`, `query.rs:105,431`, `edge.rs:22`, `cli.rs:32` | INTENCIONAL | MEM-34/MEM-01/OLD-21 métricas; refactor server (pool vs semaphore) |
| 9 | `StorageEngine.volatile_cache` + `ServerState.semaphore` fuera | `engine/mod.rs` (hist. `836aece3 C2S3b extraer CacheLayer`), `state.rs:118` (`pool: Arc<ConnectionPool>`) | INTENCIONAL | extracción CacheLayer sin stutter; pool reemplaza semaphore |
| 10 | `ThreadStore::create/get/list/delete_thread` → `create(CreateThread)`/`get`/`list(limit,offset)`/`delete` + `send_message`/`purge_expired_threads`; `PrefetchMode::is_prefetch_enabled` fuera (enum vive :107) | `thread.rs:157-292`, `config.rs:107-126` | INTENCIONAL | rediseño agentic (builder + paginación) |
| 11 | Aridades (`cmd_server` 7→10, `cmd_put` 6→7, `cmd_restore` 5→3, `client_ip` 1→2; `bfs/dfs_traverse_filtered` 4→5; `LlmClient`/`Ollama`/`OpenAI::new` 0→1) | `cli_handlers/*`, `middleware.rs:26`, `graph.rs:121,215`, `llm.rs:699,784,895` | INTENCIONAL | crecimiento features + ADR-033 providers contract |
| 12 | `WalShipper::run_loop` `->!` → `->()` | `wal_shipping.rs:160` (base `:132` era `->!`) | INTENCIONAL | graceful shutdown (`shutdown_handle`, `test_run_loop_stops_on_shutdown`) — restaurar `!` es imposible (retorna) |
| 13 | `AccessTracker` +supertraits (`AccessStats`, `Pinnable`), métodos movidos | `node/flags.rs:113-134` (métodos viven :115-129) | INTENCIONAL | split sin cambio semántico |
| 14 | `release_mmap_vector` movido; `VantaFile`→`File`; `integrations::ollama_proxy_handler` fuera; `VantaEmbedded`/`VantaNode*`/`VantaSearch*` (alias removidos) | `vfile.rs:112`, AST-002/010, `integrations/` (adapters Python fuera del crate) | INTENCIONAL (+ADR-042 para mmap) | ADR-042 documenta el fallo como aceptado; resto = de-prefix + desacople |

**Reorder append-only (shifts #5) evaluado y DESCARTADO:** mataría 6 lints de discriminante
pero `enum_variant_added` persiste en los mismos enums → 21/21 categorías seguirían rojas;
churn en archivos hot sin cambio de outcome = anti-ponytail. `#[non_exhaustive]`/`#[doc(hidden)]`
tampoco restauran compat vs 0.5.0 (solo valen para tipos NUEVOS). Aliases de compat rechazados
por precedente AST-010/ADR-041.

## Impacto mapeado (Regla 0)

- **Leídos completos:** lista en Alcance + plan file · pipeline-full.md · rules core-engine/release-ci ·
  definition-of-done · dev-tools · release-plz.toml · Cargo.toml (workspace v0.5.0, tag v0.5.0) ·
  ADR-006/014/041/042 · `ci-rust-10.yml:85-144` · baseline registry 0.5.0.
- **Hacia dentro:** revertir cualquier ítem = borrar features de develop (MEM-01/34, OLD-21, FIND-49,
  AST-002/010, FIND-26, F3C, C2S3b…) o reintroducir estado eliminado; compilación interna colapsaría.
- **Entrantes:** `QueryResult` ← `sdk/types.rs`; `AccessTracker` ← `node/mod.rs`;
  `OperationalMetricsSnapshot` ← `metrics/core/mod.rs`, `sdk/serialization/conversions.rs` (codegraph).
- **Veredicto:** impacto de NO-TOCAR. Cambio propuesto: 0 líneas Rust + 1 ADR + este file.
  **Gate D: no disparado** (sin símbolos públicos nuevos; fix-type, scope listado).

## Steps

### Step 1 — Baseline local ✅
Hecho: 21 categorías / ~100 ítems en `Temp/opencode/semver-clean.txt`. (10 min build rustdoc,
background por timeouts de 300/600s en foreground.)

### Step 2 — Triage ✅
Tabla arriba: 14 grupos, todos INTENCIONAL-0.6.0 con evidencia código/commits/ADRs. 0 accidentales.

### Step 3 — ADR Regla 5 ✅
`docs/architecture/adr/ADR-044-acumulado-breaking-0.6.0-find-133.md` (Contexto/Decisión/Consecuencias;
sigue plantilla `docs/_templates/adr.md`; detalla mecanismo release-plz + precedente aliases).

### Step 4 — Fix accidentales ✅ (vacuo documentado)
Sin accidentales → sin slices de código. Reorder append-only y aliases descartados con motivo arriba.
Cero refactors, WIP ajeno intacto.

### Step 5 — Verify contrato 🟡 BLOQUEADO (ver motivo)
- `cargo check -p vantadb -j 2`: corre (árbol sano tras no-touch).
- `cargo semver-checks -p vantadb`: seguirá ROJO en develop con versión 0.5.0 = baseline
  **por diseño del repo** (CI scope main-only: "en develop es ruido"; `--release-type` deriva del
  número de versión — el verde llega con el bump 0.5.0→0.6.0 que hace release-plz en main vía
  `refactor!:`/`feat!:` + `semver_check=true`). Bump manual prohibido por contrato.
- `campaign_verify_cmd`: bug exit -1 conocido → bash directa + mención (esta).
- 2-fallas-mismo-error → Gate V N/A (0 fallas de edición; 0 ediciones Rust).

### Step 6 — Commit selectivo NO PUSH ✅
Commit `3c4f146c` (2 files, +157: este file + ADR-044; pre-commit hook verde).
NO PUSH (solo vanta-lead). Recitation in-progress + RESULTADO §7 abajo.

## Deuda / Notas
- Notion Paso 0c (4 páginas fetch): pendiente — Internet N/A declarado; deuda registrada, no bloquea.
- Logs CI runs 35417674114/35467546726: no consultados (Internet N/A); baseline = run local + registry 0.5.0.
- **Gate C (pregunta sin tool → motivo):** owner elige (A) aceptar rojo-en-develop hasta release 0.6.0
  vía release-plz (=也会 hacer pasar el job en main post-bump), o (B) exigir verde-ya (= revert masivo
  destructivo, desaconsejado). Sin `question` tool disponible → se registra y se devuelve INCOMPLETO.
- Precedentes intencional-major: ADR-041, ADR-042, AST-010, CHANGELOG [Unreleased] "BREAKING (0.x)".
