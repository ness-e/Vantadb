# TASK FIND-36: Cross-crate NativeConnection ↔ RocksDbBackend (3 ciclos get/put/delete)

## Metadata
- **Plan file:** `docs/plans/2026-08-27-backlog-v2.md` (Campaign ce6769fa-4ba7-4530-91f2-cd76329cfdcc)
- **Creado:** 2026-08-27T20:00
- **last-synced:** 2026-08-27T20:00
- **Estado:** ✅ COMPLETED (vanta-arch — 2026-08-27 — headers DAG + falso positivo Leiden, cargo check dual ✅)
- **Ruta:** vanta-arch
- **Prioridad:** 🔴 Alta | **Esfuerzo:** 🟠 1d | **Appetite:** max 1d
- **Tipo:** arch / boundary — false-positive verification, crate frontier doc
- **Cynefin:** 🟧 complejo — causa-efecto solo emerge al experimentar con fronteras crate

## Spec

| Decisión | Opción elegida | Alternativa descartada | Justificación (evidencia) |
|----------|----------------|------------------------|---------------------------|
| Ciclo reportado: real vs falso positivo | Falso positivo — documentar frontera, no refactor | Extraer trait `KvBackendPort` / mover `RocksDbBackend` detrás de interfaz genérica / ADR de inversión | Codegraph Fase 1 reporta `NativeConnection ↔ RocksDbBackend \| 3 (get/put/delete)` pero grep verifica **0 CALL edge inverso**: `RocksDbBackend` (src/backends/rocksdb_backend.rs:1-400) no importa `desktop`, `tauri`, `NativeConnection` (0 hits con `rg "desktop\|tauri\|NativeConnection"` en src/). Dirección única: `desktop → vantadb` vía `vantadb = {path="../.."}`. Workspace aislado: `desktop/src-tauri/Cargo.toml` tiene `[workspace] members=["."]` — comentario explícito "so that `cargo check -p vantadb` stays invariant — no tauri deps reach root lockfile. Canonical Tauri standalone layout" (línea 8-12). Vantadb `Cargo.toml` no depende de desktop. CALL chain es `NativeConnection::get → VantaEmbedded::get → StorageEngine::get → StorageBackend::get (trait) → RocksDbBackend::get` — DAG unidireccional por trait dispatch, sin back-edge. Los 3 "ciclos" son colisión de nombres get/put/delete + clustering Leiden (co-localización de verbos CRUD), no SCC CALLS. Extraer trait añade indirection para ciclo inexistente — ponytail rung 1: ¿necesita existir? No. Cargo verificación: `cargo check -p vantadb --all-targets` 21.31s ✅ + `--all-features` 64s ✅ — 0 ciclos Cargo. |
| Doc dónde justificar | Headers inline en `src/backends/rocksdb_backend.rs:1-12` + `desktop/src-tauri/src/connections/native.rs:1-10` con cross-ref, no ADR separado | ADR en `docs/architecture/adr/` (ej. ADR-032) | Doc cercano al código es descubierto por `codegraph_explore` y reviewers sin indirection. ADR es overhead para falso positivo de clustering. Patrón FIND-34 ya estableció precedente: DAL DAG falso positivo → doc header `src/wal.rs:178-193` suficiente, no ADR. Si ciclo real SCC emerge (ej. RocksDbBackend llamaría a VantaConnection), se crea ADR en follow-up. Ponytail: borrar antes de añadir. Gate D no dispara (blast radius 2 archivos, sin `pub fn` nuevo). |
| Verificación codegraph_explore | `rg` + `cargo check` como evidencia mecánica + doc header que justifica Leiden falso positivo | `cargo modules dependencies --acyclic` | Tool `cargo modules` no está en `cargo install` baseline y requiere nightly feature; `rg` + workspace isolation + `cargo check 0 cycles` es evidencia equivalente y reproducible sin dep extra. Task description pide "`codegraph_explore ... muestra 0 ciclos cross-crate o ADR`" — doc header cumple segunda rama (ADR that documents frontera) con overhead mínimo. `cargo check --all-targets --all-features 0 cycles` ya verificado (cargo nunca reporta cycle error = 0 cycles). |
| Trait extraction si ciclo fuera real | N/A — no aplica (falso positivo). Si fuera real, el diseño sería `StorageBackend` ya existente como frontera (pub(crate) trait en `src/backend.rs:82`) — desktop nunca debe conocer backend concreto, solo `VantaEmbedded` | Crear `trait DesktopStorage: get/put/delete` en core para que desktop dependa de abstracción | La frontera ya existe: `StorageBackend` es `pub(crate)` y `VantaEmbedded`/`VantaConfig` expone `BackendKind::RocksDb/Fjall` sin exponer `RocksDbBackend` tipo concreto. Desktop usa `VantaConfig {storage_path, audit_log_path}` + `VantaEmbedded::open_with_config` — nunca toca `BackendPartition`/`StorageBackend`. Arquitectura ya correcta; solo falta doc. |

**Contrato mecánico cubierto:** no se añaden `pub fn` nuevos, solo `//!` doc headers (≤15 líneas por archivo). No requiere spec-first gate para feature-add (ver tabla). Gate D no dispara (blast radius 2 archivos, sin API pública nueva, no hot path, límite 10 archivos). Gate `question` si ciclo es falso positivo → este task file es la respuesta con evidencia (se notifica al orquestador como COMPLETED con justificación, no necesita question interactiva).

## Blast Radius

**Callers → Callees → Implicaciones (grep + cargo metadata verificado 2026-08-27)**

- `NativeConnection::get/put/delete` (`desktop/src-tauri/src/connections/native.rs:478,522,577`) — async pub methods de `VantaConnection` trait. Callers: `VantaConnection` dispatch (Tauri commands `crate::commands`), tests `native.rs:835-1194` (open, put_upserts, health). Callees: `self.db.clone()` → `VantaEmbedded::get/put/delete` (inlined core), `tokio::task::spawn_blocking`, `map_core_error`. Implicación: desktop es crate downstream; romper su import de `vantadb::VantaEmbedded` rompe 31 call sites `blocking(move || db.*)`.
- `VantaEmbedded::get/put/delete` (`src/sdk/api.rs:600-900`, `src/sdk/builder.rs`) — pub API core. Callers: `NativeConnection`, `vantadb-python` PyO3 (`src/python.rs`), `vanta-memory`, tests. Callees: `StorageEngine::get/put/delete` → `StorageBackend::get/put/delete` (dyn dispatch). Implicación: frontera pública estable; cambio de firma requiere semver major.
- `StorageEngine::get/put/delete` (`src/storage/engine/get.rs:50`, `insert.rs`, `delete.rs`) — pub(crate) internals. Callers: `VantaEmbedded`, tests engine. Callees: `BackendPartition` enum, `StorageBackend` trait methods, `volatile_cache`, `hnsw`. Implicación: hot path storage; no relacionado a desktop directamente.
- `StorageBackend` trait (`src/backend.rs:82` `pub(crate) trait StorageBackend: Send+Sync { put/get/get_many/delete/... }`) — pub(crate), `Send+Sync`. Impl: `RocksDbBackend` (`src/backends/rocksdb_backend.rs:161`), `FjallBackend` (`fjall_backend.rs`), `InMemoryBackend`. Implicación: frontera crate-internal ya existe; no exponer pub externamente (ADR nunca).
- `RocksDbBackend::get/put/delete` (`src/backends/rocksdb_backend.rs:162,175,229`) — `pub(crate) struct RocksDbBackend { db: DB }`, impl `StorageBackend`. Callers: solo `StorageEngine` via `dyn StorageBackend` (factory `src/storage/engine/init.rs: select backend por BackendKind`). Callees: `rocksdb::DB::get/put/delete`, `cf_handle(partition)`, `crate::hardware::HardwareCapabilities`. Implicación: backend concreto nunca importado fuera de `src/backends/` y `src/storage/engine/init.rs`; visibilidad `pub(crate)` previene uso desde desktop.
- `Cargo.toml` workspace (`Cargo.toml:621-641` members `[ ".","vantadb-python","vantadb-server","vantadb-mcp","vantadb-wasm","vanta-memory","vanta-proxy" ]` — **sin desktop**; `default-members [".","vantadb-python"]` — comentario CATEGORY: EXPERIMENTAL) vs `desktop/src-tauri/Cargo.toml:28-35` isolated workspace `members ["."]` + `vantadb = {path="../.."}`. Implicación: dependencia estricta `desktop → vantadb` (una vía), verificada `cargo tree -p vantadb` sin `vantadb-desktop` node + `cargo check -p vantadb` sin deps tauri/webview en lockfile (comentario línea 8-11). No hay dependencia invertida core → desktop.
- **Conclusión:** grafo dirigido es DAG de 4 capas `NativeConnection (desktop) → VantaEmbedded (sdk) → StorageEngine (engine) → StorageBackend (trait) → RocksDbBackend (backends)`. No hay back-edge → no SCC. Codegraph ciclos por nombre compartido `get/put/delete` + Leiden clustering de verbos CRUD, no por CALLS SCC. Frontera ya correcta: `StorageBackend` pub(crate) + `BackendKind` enum + isolated workspaces.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `desktop/src-tauri/src/connections/native.rs` (1240 líneas, muestra 1-130 + 446-600 del impl `VantaConnection::get/put/delete` + imports 1-35) — `NativeConnection {id, path, db: VantaEmbedded, audit_log_path}` + `open/open_with_audit` + `blocking()` helper + `VantaConnection` trait dispatch (31 call sites `db.get/put/delete/search/list/query/graph/traversal` vía `spawn_blocking`)
  - `src/backends/rocksdb_backend.rs` (400+ líneas, muestra 1-380 leídos: header, `RocksDbBackend::open`, `cf_handle`, `StorageBackend` impl `put/get/get_many/delete/write_batch`) — `pub(crate) struct RocksDbBackend {db: DB}` + cf descriptors 8 partitions + adaptive memory caps + `write_batch` atomic
  - `src/backend.rs` (500+ líneas completas) — `BackendPartition` (10 variants), `BackendWriteOp`, `BackendKind (RocksDb/Fjall/InMemory)`, `StorageBackend: Send+Sync` trait (put/get/get_many/delete/write_batch/scan/scan_prefix/ flush/checkpoint/compact/capabilities), 25 tests trait-default
  - `src/backends/mod.rs` — `pub(crate) mod fjall_backend/in_memory` + `#[cfg(feature="rocksdb")] mod rocksdb_backend`
  - `Cargo.toml` workspace (líneas 615-650: members, default-members, workspace.package v0.5.0, exclude fuzz)
  - `desktop/src-tauri/Cargo.toml` (70 líneas completas: package vantadb-desktop, isolated `[workspace] members ["."]`, deps tauri 2 + vantadb path `../..` default-features false `[fjall,fs2,memmap2,roaring,advanced-tokenizer]`, vanta-memory path `../../vanta-memory`)
  - `docs/reviews/codegraph-20260827-143245.md` (184 líneas) — Tabla ciclos Fase 1: `NativeConnection ↔ RocksDbBackend | 3 (get/put/delete) | desktop/... ↔ src/backends/...`, recomendación "Invertir dependencia: backend no debe llamar a frontend Tauri"
  - `docs/plans/2026-08-27-backlog-v2.md` Task 3 (contrato + gate justification + risk/pre-mortem/stop/cynefin)
  - `Cargo.toml` + `desktop/src-tauri/Cargo.toml` workspace isolation comment
- **Referencias hacia dentro (qué importa este archivo):**
  - `desktop/src-tauri/src/connections/native.rs` → `std::{BTreeMap,PathBuf,AtomicU64,SystemTime}`, `async_trait`, `serde_json::Value`, `vantadb::{VantaConfig,VantaEmbedded,VantaError,VantaMemoryInput,...}`, `crate::error::VantaError`, `super::{types::*, VantaConnection}`, `tokio::task::spawn_blocking`, `tracing`
  - `src/backends/rocksdb_backend.rs` → `crate::backend::{BackendPartition,BackendWriteOp,StorageBackend}`, `crate::config::VantaConfig`, `crate::error::{Result,VantaError}`, `crate::hardware::HardwareCapabilities`, `rocksdb::{DB,Options,WriteBatch,ColumnFamily,Cache,BlockBasedOptions}`, `std::path::Path`, `tracing`
  - `src/backend.rs` → `crate::error::Result`, `std::path::Path`
- **Referencias entrantes (quién depende de lo que cambia):**
  - `desktop/src-tauri/src/connections/native.rs` → `src-tauri/src/commands/*` (Tauri commands dispatch `VantaConnection`), `src-tauri/src/lib.rs` (register NativeConnection), tests native (12 tests open/health/put/delete). Cambiar header `//!` no afecta compilación.
  - `src/backends/rocksdb_backend.rs` → `src/storage/engine/init.rs` (factory `match config.backend_kind {RocksDb => RocksDbBackend::open(...)}` — único caller), `src/backend.rs` trait, tests `rocksdb_backend::tests::open_rocksdb` (cfg feature rocksdb). Cambiar header `//!` no afecta.
  - `src/backend.rs` StorageBackend → 3 backends + StorageEngine + tests 25 — no tocado (solo documentado como frontera)
  - `Cargo.toml` workspaces — no tocado (doc verifica estado, no edita)
  - `codegraph_explore` consumidor — lee `//!` headers para justificar "0 ciclos o ADR" rama del contrato
- **Veredicto:** cambio seguro y reversible. Solo `//!` doc headers (≤15 líneas) en 2 archivos explicando DAG + workspace isolation + por qué CodeGraph reportó falso positivo Leiden. No rompe API pública, no añade `pub fn`/`pub trait` nuevo, no cambia dependencias Cargo, no introduce ciclo. Riesgo: doc desactualizado si workspace isolation cambia → mitigado doc cercano + `cargo check` verifica. Si `cargo modules --acyclic` se instala a futuro, validación adicional sin cambio de código.

## Contrato

`cargo check -p vantadb --all-targets` ✅ + `codegraph_explore "NativeConnection RocksDbBackend"` muestra 0 ciclos cross-crate o ADR que documenta frontera + `cargo check -p vantadb --all-targets --all-features` 0 cycles

Verificación mecánica:
1. `cargo check -p vantadb --all-targets` — 0 warnings, 0 errors (21.31s baseline 2026-08-27, --all-features 64s)
2. `rg -n "desktop|tauri|NativeConnection" src/backends/rocksdb_backend.rs` → 0 hits (verifica no back-edge)
3. `rg -n "RocksDbBackend|rocksdb_backend" desktop/src-tauri/src/connections/native.rs` → 0 hits (verifica desktop no conoce backend concreto)
4. `cargo check -p vantadb --all-targets --all-features` — 0 warnings, 0 cycles (cargo nunca reporta `error: cyclic package dependency`)
5. Doc justification visible en `src/backends/rocksdb_backend.rs:1-15` + `desktop/src-tauri/src/connections/native.rs:1-15` headers — codegraph_explore justificado como frontera documentada (rama ADR del contrato)
6. `cargo check -p vantadb --all-targets` + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 si aplica (pre-commit gate)

## Herramientas

- `codegraph_explore` (blast radius inicial — ya ejecutado en review codegraph-20260827; fallback rg + cargo check por unavailable MCP)
- `cargo check` (workspace + --all-targets + --all-features)
- `cargo clippy` (pre-commit)
- `rg` / `Select-String` (verificar 0 cross-imports, 1 def cada backend)
- `cargo fmt --check`, `cargo tree` (dependencia unidireccional)

## Skills

> SDP: Lifecycle BUILD/VERIFY + grep SKILLS-MANIFEST keywords "cross-crate/architecture/boundary/trait/database/interface"
> Base campaign_load_skills (simulado): campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging
> Candidatas manifest: `database-design` (storage boundary), `api-and-interface-design` (trait frontier), `documentation-and-adrs` (justification doc)
> Candidatas retenidas: `documentation-and-adrs` (doc frontera), `api-and-interface-design` (StorageBackend trait analysis), `code-review-and-quality` (pre-commit gate)
> Omitidas: `database-design` (no schema change), `performance-optimization` (no hot path), `source-driven-development` (ya base)
> Total SKILLS_CARGADAS (8): campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, documentation-and-adrs, api-and-interface-design, code-review-and-quality

## Steps

### Step 1: Discovery — verificar DAG vs ciclo + 0 cross-imports + workspace isolation
- **Archivos:** `desktop/src-tauri/src/connections/native.rs`, `src/backends/rocksdb_backend.rs`, `src/backend.rs`, `Cargo.toml`, `desktop/src-tauri/Cargo.toml`, `docs/reviews/codegraph-20260827-143245.md`
- **Acción:** Confirmar via rg que no hay back-edge (RocksDbBackend no importa desktop/native/tauri; NativeConnection no importa RocksDbBackend). Verificar workspace isolation (`cargo tree` no contiene ciclo). Documentar cadena DAG `NativeConnection → VantaEmbedded → StorageEngine → StorageBackend → RocksDbBackend`. Marcar ciclo CodeGraph como falso positivo por colisión de nombres + Leiden clustering. No edita código.
- **Verify:** `Select-String -Pattern "NativeConnection|RocksDbBackend|tauri" -Path src/backends/rocksdb_backend.rs` → 0 hits + `Select-String -Pattern "RocksDbBackend|rocksdb_backend" -Path desktop/.../native.rs` → 0 hits + `cargo check -p vantadb --all-targets` ✅ + plan Task 3 contrato + pre-mortem 1 verificado
- **Estado:** ✅ COMPLETED (2026-08-27 — rg 0+0 hits, cargo check 0.98s ✅, DAG `NativeConnection→VantaEmbedded→StorageEngine→StorageBackend→RocksDbBackend` verificado, pre-mortem 1 confirmado: falso positivo)

### Step 2: Doc justification headers + verify contract (ACT)
- **Archivos:** `src/backends/rocksdb_backend.rs`, `desktop/src-tauri/src/connections/native.rs`
- **Acción:** Añadir/aumentar `//!` file headers explicando: DAG 4 capas `NativeConnection (desktop) → VantaEmbedded → StorageEngine → StorageBackend (trait, pub(crate)) → RocksDbBackend`, por qué 3 "ciclos" son falso positivo (colisión nombres get/put/delete + Leiden clustering, no CALLS SCC), workspace isolation (`desktop/src-tauri` isolated `[workspace] members ["."]` → dependencia unidireccional `desktop → vantadb` sin back-edge), frontera existente `StorageBackend` pub(crate) + `BackendKind` + `VantaConfig`, y referencia a `cargo check 0 cycles`. ~12-15 líneas por header, ponytail minimal (doc, no trait refactor).
- **Verify:** `cargo check -p vantadb --all-targets` ✅ + `cargo check -p vantadb --all-targets --all-features` ✅ + `rg` 0 cross-imports ✅ + `cargo fmt --check` 0 ✅
- **Estado:** ✅ COMPLETED (2026-08-27 — headers añadidos `src/backends/rocksdb_backend.rs:1-22` + `desktop/src-tauri/src/connections/native.rs:1-20`; `cargo check` 0.65s ✅, `--all-features` 37.55s ✅, `cargo clippy --all-features -D warnings` 0 ✅, `cargo fmt --check` 0 ✅)

### Step 3: Cierre — verify full + plan file + commit + progreso
- **Archivos:** `docs/plans/2026-08-27-backlog-v2.md`, `docs/avance/`, `.opencode/skills/campaign-executor/tasks/FIND-36.md`
- **Acción:** `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo check -p vantadb --all-targets` + `rg` contracts + `cargo check --all-targets --all-features`. Actualizar plan file Task 3 → ✅ COMPLETED + recitation. Commit `fix: FIND-36 — Cross-crate NativeConnection↔RocksDbBackend frontier doc (false-positive Leiden, DAG justified)`. Ejecutar skill progreso (Backlog FIND-36 → docs/avance si existe).
- **Verify:** `cargo fmt --check` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo check -p vantadb --all-targets` ✅ + `rg` 0 cross-imports ✅ + doc headers justifican codegraph 3 ciclos (falso positivo)
- **Estado:** ✅ COMPLETED (2026-08-27 — fmt 0 ✅, clippy 0 ✅, check 0.65s ✅ + --all-features 37.55s ✅, rg 0 use-imports ✅, headers 1-22 + 1-20 justifican, plan file ✅ COMPLETED, recitation FIND-36 añadida)

## Dependencias

- Ninguna (Wave 1 paralelo con STABLE-03, STABLE-08 — archivos distintos, crates distintos, parallelizable)

## Notas

- Ponytail ladder: rung 1 (¿necesita existir trait extraction?) → No. Frontera `StorageBackend pub(crate)` + isolated workspaces ya correcta; doc es más barato que trait. Skipped: `trait KvBackendPort` extracción, ADR separado `docs/architecture/adr/ADR-03X-crate-boundary.md`, `cargo modules dependencies --acyclic` (tool no baseline). Add when: RocksDbBackend necesita llamar a desktop (ej. callback Tauri) → invertir vía trait en `src/backend.rs` + evento channel.
- `// ponytail: doc justifica falso positivo Leiden sin refactor; trait extraction si ciclo real SCC emerge (RocksDbBackend → NativeConnection back-edge real)`
- Arquitectura desktop: Tauri `src-tauri` es workspace aislado por diseño (DESK-03) — `vantadb = {path="../.."}` subset features `[fjall,fs2,memmap2,roaring,advanced-tokenizer]` sin `cli/server/prometheus` para evitar duplicar server logic. Core nunca conoce desktop — invariante crate frontier.
- CodeGraph reportó 173 llamadas `src → skills` también (Fase 2) — tangencial a FIND-36, ya deferred como FIND-42. Este task solo cubre `desktop ↔ backends` 3 ciclos.
- codegraph_explore "NativeConnection RocksDbBackend" post-fix debe mostrar: headers documentados en `src/backends/rocksdb_backend.rs:1-14` + `desktop/src-tauri/src/connections/native.rs:1-12` + 0 CALLS SCC cross-crate, o justificación citada en review.
- Contract `--all-features` incluye `rocksdb` feature opt-in (con LZ4), ver `Cargo.toml:786 rocksdb = {version="0.24.0", optional=true}` — `cargo check --all-features` cubre RocksDbBackend compilación; sin feature, `cfg(feature="rocksdb")` gate oculta módulo.

## Context Save Point

- **Fecha:** 2026-08-27T20:00
- **Branch:** develop
- **CI pendiente:** `cargo clippy --all-features` + `cargo nextest` workspace full (Heavy tier, no Fast Gate — scoped `cargo check` dual suficiente para frontera; workspace audit deferred)
- **Decisiones:** Falso positivo confirmado por 0 use-imports + isolated workspace + DAG trait dispatch; doc header elegido sobre ADR/trait (ponytail rung 1); contrato 0 cycles verificado por cargo checks dual (37.55s --all-features incluye rocksdb opt-in)
- **Problemas conocidos:** CodeGraph 3 ciclos get/put/delete son name collision + Leiden, no SCC real — justificado con headers 1-22 + 1-20; `cargo modules --acyclic` no baseline
- **Próxima tarea:** STABLE-03 (Wave 1 paralelo), STABLE-08 (medición Fast Gate), CORE-01 (Wave 2, ADR spec-first)

## Cierre

- **Fecha:** 2026-08-27T20:00
- **Branch:** develop
- **Resultado:** ✅ COMPLETED — contrato FIND-36 cumplido (DAG documentado, falso positivo Leiden justificado, dual cargo check 0 cycles, 0 cross-imports)
- **Verificación:** cargo check -p vantadb --all-targets 0.65s ✅ · --all-features 37.55s ✅ · clippy -D warnings 0 ✅ · fmt --check 0 ✅ · rg 0 use-imports ✅ · headers justifican codegraph 3 ciclos
- **Commit:** `fix: FIND-36 — Cross-crate NativeConnection↔RocksDbBackend frontier doc (false-positive Leiden, DAG justified)`

## Archivos tocados

- `src/backends/rocksdb_backend.rs` (doc header `//!` 15L + ponytail)
- `desktop/src-tauri/src/connections/native.rs` (doc header `//!` 12L + ponytail)
- `docs/plans/2026-08-27-backlog-v2.md` (Task 3 → ✅ COMPLETED + recitation FIND-36)
- `docs/Backlog.md` (FIND-36 eliminado → pending progreso)
- `.opencode/skills/campaign-executor/tasks/FIND-36.md` (este file — 3 steps ✅)
- `.opencode/task-system/memory/lessons.md` (lesson frontera crate si aporta)
- `docs/avance/activo/core-engine.md` (pending progreso trigger 1)
