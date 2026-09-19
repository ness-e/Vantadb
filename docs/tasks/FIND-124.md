# FIND-124 — migrar `desktop/src-tauri` a la API actual (compila ×3 OS)

> **Plan:** `docs/plans/2026-09-19-ci-green.md` (Wave0, primera en secuencia) · **Campaign:** 0ad2d7e2-94e3-4313-8f5c-e8d57c08a6af
> **Estado:** ⏳ IN PROGRESS · **Ruta:** vanta-worker · **Branch:** develop · **Commit:** `fix: FIND-124 — ...`
> **SDP:** campaign-executor, frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (+ systematic-debugging manual para clasificar E0432)

## 1. TAREA

**Objetivo:** migrar `desktop/src-tauri` a la API actual del core — compila ×3 OS (Build & Test rojo en Linux/Windows/macOS del PR #182, log CI run 35413944127).

**Contrato exacto:** `cargo check/test -p desktop-src-tauri` (paquete real `vantadb-desktop`, workspace aislado `desktop/src-tauri`, comando equivalente: `cargo check/test -j 2` dentro de `desktop/src-tauri`) verde local + CI Build & Test ×3 en verde tras push (push lo hace vanta-lead; mi evidencia = verde local + commit).

**Acceptance criteria del plan:**
- (a) mapear TODOS los `Vanta*` a nombres actuales (post-AST-010);
- (b) check/tests tauri verdes local;
- (c) CI ×3 verde o re-run con evidencia (lo ejecuta el orquestador/lead tras mi commit — dejo recitation con el comando exacto).

**Gate Justificación (plan):** es el rojo más grande del PR (3 jobs); sin esto no hay verde posible.
**Appetite / Esfuerzo / Prioridad:** 2d / 🟡 / 🔴 Alta. **Cynefin:** 🟨 complicado (hasta mapear). **⬆️ 1 / ⬇️ 3 steps.**

## 2. ARCHIVOS

**Clave (con :línea — errores E0432/E0425/E0433 reales de `cargo check -j 2 --tests` local 2026-09-19):**
- `desktop/src-tauri/src/connections/native.rs:30` (`use vantadb::config::VantaConfig`), `:32` (`use vantadb::VantaError as CoreVantaError`), `:33-37` (bloque `VantaBm25TermContribution, VantaEmbedded, VantaMemoryFilterItem, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest, VantaNodeRecord, VantaQueryResult as CoreQueryResult, VantaSearchExplanationHit, VantaValue`), `:366-367` (`vantadb::VantaNamespaceStats`), resto de usos `Vanta*` en cuerpo (to_vanta_value, ingest_to_input, record_to_memory, core_query_to_wire, node_record_to_memory, hit_to_result, explanation_to_dto, search_request, open_with_audit, db field)
- `desktop/src-tauri/src/commands/connection.rs:13` (`use vantadb::VantaEmbedded`), `:39` (`vantadb::VantaError` en firma map_core_error), `:52` (`VantaEmbedded::open`)
- `desktop/src-tauri/src/error.rs:104` (doc `vantadb::VantaError::code()`), `:147-152` (`from_core(e: &vantadb::VantaError)`, `use vantadb::VantaError as Core`), `:249` (`VantaError::NodeNotFound(7)` en test), `:266-268` (`DatabaseBusy`, `Io` en test)
- `desktop/src-tauri/src/lib.rs:23` (`use vantadb::config::VantaConfig`), `:41` (`pub config: VantaConfig`), `:92` (`VantaConfig::default()`)
- `desktop/src-tauri/src/commands/memory.rs:15` (`sdk::{VantaMemoryListOptions, VantaMemoryListPage}`), `:16` (`VantaEmbedded`), `:100` (`downcast_ref::<vantadb::VantaError>`), `:119/:143` (`db: &VantaEmbedded`), `:486` (`VantaConfig::default()` en test state), `:877/:882/:893` (`vantadb::VantaError` en ChainedError + tests)
- `desktop/src-tauri/src/commands/metrics.rs:11-12` (`config::VantaConfig`, `VantaEmbedded, VantaOperationalMetrics`), `:24` (retorno `VantaOperationalMetrics`), `:30` (`VantaEmbedded::test_empty(VantaConfig::default())`)
- `desktop/src-tauri/src/connections/manager.rs:403` (retorno `vantadb::VantaEmbedded`)
- `desktop/src-tauri/src/connections/types.rs:231` (`sdk::VantaFilterOp`), `:656` (`VantaFilterOp::Eq` en test)
- `desktop/src-tauri/Cargo.toml` (deps — solo lectura; path `vantadb = { path = "../.." }`, sin cambios)

**Relacionados (fuente de verdad — solo lectura):**
- `src/sdk/mod.rs:15,24-33` (exports actuales: `Embedded`, `MemoryInput`, `MemoryRecord`, `MemoryListOptions`, `MemoryListPage`, `MemorySearchHit`, `MemorySearchRequest`, `MemoryFilterItem`, `NodeRecord`, `QueryResult`, `SearchExplanationHit`, `Bm25TermContribution`, `NamespaceStats`, `FilterOp`, `Value`, `OperationalMetrics`, `connect`)
- `src/sdk/builder.rs:15,71,99,143` (`Embedded::open/open_with_config/test_empty/close`)
- `src/config.rs:572,783` (`pub struct Config`, `audit_log_path`; `storage_path` en ambos StorageCfg:153 y Config:574 — el usado por desktop es `Config`)
- `src/error.rs:122,125,161,292,337` (`Error::NodeNotFound/Io/DatabaseBusy/code()`)
- `src/node/vector_data.rs:30` (`SparseVector` — import `vantadb::node::SparseVector` en native.rs NO se toca)
- `src/graph.rs:29` (`TraversalDirection` — import `vantadb::graph::TraversalDirection` NO se toca)
- Callers del blast radius (codegraph_explore 2026-09-19): `VantaError` desktop → 162 callers internos (commands/audit, data, metrics, connections/manager +10); `VantaQueryResult` desktop → 11 callers (connections/mod, server, trait, native +2); `VantaQueryResult` TS (`desktop/src/vanta.ts:344`) → 5 callers (vanta-http-map, IqlConsole) — el DTO desktop local NO se renombra (es contrato propio, no símbolo core)

**Prohibidos (WIP ajeno — NO se toca):**
`completions/*` (dirty en worktree), `.opencode` (submodule dirty), `reparacion.bat` (untracked), `Justfile`, `ocr-*`, `desktop/src-tauri/Cargo.lock` (solo lectura salvo que el fix lo exija con motivo — no lo exige), stash@{0} GOV-C4, `docs/Backlog.md`, plan file (solo recitation al cierre), `C:/Users/Eros/.vantadb*` (datos vivos), `src/` (cero cambios core — solo lectura como fuente de verdad), `web/`, `examples/`, `vantadb-mcp/`, `skills/`, resto de `docs/plans/`.

## 3. DEPENDENCIAS

**Wave:** Wave0 primera en secuencia (FIND-123 en paralelo-secuencial, disjuntos — no lo toco).
**Bloqueantes:** ninguno. **Task previa:** ninguna. **NextTask tras cierre:** FIND-123 (la ejecuta el orquestador, no yo).
**Stop del plan:** API nueva incompatible (no solo renombre) → DEFER con diagnóstico + alcance real (no re-diseñar desktop). **Veredicto DISCOVERY:** NO dispara — todo es renombre 1:1 verificado contra `src/` (ver §8). Sigo con la migración.

## 4. REFERENCIAS

- **Rules (lectura completa antes de codificar):** `.opencode/rules/core-engine.md` — R-3 (propagar con `?`, prohibido unwrap fuera de tests; mi cambio no añade unwraps), R-1/R-2/R-4/R-5 no aplican (no toco `src/`, no añado unsafe, no añado env vars). Leída completa 2026-09-19 ✅
- **Refs:** `definition-of-done.md`, `dev-tools.md`, `test-suite.md`, `skills-engineering.md` (SDP — ejecutado vía `campaign_discover_skills_v2` phase=BUILD, 8 skills, ver cabecera), `clean-code-clean-architecture.md` Ap. V (transversal MUST — rename puro, sin cambio de capas), `ocr-review.md` (gate VERIFY cierre)
- **Commands:** `pipeline.md` (ejecución), `audit.md` (verify L9/post-tarea)
- **SPEC.md raíz:** sin cambios (0 greenfield, todo fix sobre comportamiento existente)
- **Tabla Spec (símbolo público nuevo):** N/A — migración interna, sin símbolos públicos nuevos. Gate D spec-first no aplica (no es feature-add).

## 5. SKILLS (SDP Paso 0b — `campaign_discover_skills_v2` phase=BUILD keywords [tauri, desktop, Vanta, rename, E0432, build], 8 devueltas, todas cargadas)

- `campaign-executor` — base task-system (PLAN→ACT→VERIFY, recitation, RESULTADO)
- `frontend-ui-engineering` — base tipo Desktop-Tauri (contrato desktop, no UI web)
- `source-driven-development` — base tipo; API real de `src/` manda sobre memoria del modelo
- `incremental-implementation` — slices ~100 líneas, compilable tras cada slice
- `test-driven-development` — Prove-It: `cargo check/test` como RED (E0432 ya reproduce) → GREEN rename → REFACTOR n/a
- `context-engineering` — context pack por slice (rules → plan → source del slice)
- `doubt-driven-development` — stakes altos (3 jobs CI); verificación adversarial del mapa
- `api-and-interface-design` — boundaries: DTOs desktop locales NO se renombran, solo imports core
- (+ manual `systematic-debugging` — clasificar E0432: Fase 1 evidencia = log `cargo check` local; causa raíz = AST-010 renombró, desktop no migró; hipótesis única = renombre 1:1; test = `cargo check --tests`)

## 6. HERRAMIENTAS+MCP

- `codegraph_explore` PRIMERO ✅ (mapa `Vanta*`→actuales + blast radius — hecho en DISCOVERY)
- `cargo check -j 2 --tests` + `cargo test -j 2` en `desktop/src-tauri` (workspace aislado; `-j 2` siempre por riesgo OOM)
- `cargo fmt --check`, clippy del scope (`cargo clippy -j 2 --tests -- -D warnings`)
- `campaign_verify_cmd` para gates (BUG exit -1 conocido → bash directa + mención en RESULTADO)
- Internet N/A (todo local; nada que marcar)

## 7. INVESTIGACIÓN CÓDIGO — blast radius (generado en DISCOVERY)

**Grep exhaustivo `Vanta[A-Z]` en `desktop/src-tauri/src`:** ~100 matches; los que rompen son solo los que nombran símbolos del core (lista §2 Clave). Los `VantaError`/`VantaQueryResult`/`MemoryRecord`/`Bm25Term`/`ExplanationHit` **locales** (`error.rs`, `connections/types.rs`) son contrato propio del desktop y NO se renombran. `desktop/src/vanta.ts` (TS) tampoco se toca.

**Tabla renombre (verificada contra `src/` — cada fila con evidencia):**

| Desktop (roto) | Actual (core) | Evidencia |
|---|---|---|
| `vantadb::VantaEmbedded` | `vantadb::sdk::Embedded` | `src/sdk/mod.rs:15`, `builder.rs:15` |
| `vantadb::VantaError` | `vantadb::error::Error` | `src/error.rs:122` (`pub enum Error`), re-export? NO en raíz — usar path completo o `sdk`? Ver nota |
| `vantadb::config::VantaConfig` | `vantadb::config::Config` | `src/config.rs:572` |
| `vantadb::VantaMemoryInput` | `vantadb::sdk::MemoryInput` | `src/sdk/mod.rs:27` |
| `vantadb::VantaMemoryRecord` | `vantadb::sdk::MemoryRecord` | `src/sdk/mod.rs:28` |
| `vantadb::VantaMemoryListOptions` | `vantadb::sdk::MemoryListOptions` | `src/sdk/mod.rs:28` |
| `vantadb::VantaMemoryListPage` | `vantadb::sdk::MemoryListPage` | `src/sdk/mod.rs:28` |
| `vantadb::VantaMemorySearchHit` | `vantadb::sdk::MemorySearchHit` | `src/sdk/mod.rs:28` |
| `vantadb::VantaMemorySearchRequest` | `vantadb::sdk::MemorySearchRequest` | `src/sdk/mod.rs:29` |
| `vantadb::VantaMemoryFilterItem` | `vantadb::sdk::MemoryFilterItem` | `src/sdk/mod.rs:27` |
| `vantadb::VantaNodeRecord` | `vantadb::sdk::NodeRecord` | `src/sdk/mod.rs:29` (`types/graph.rs`) |
| `vantadb::VantaQueryResult` | `vantadb::sdk::QueryResult` | `src/sdk/mod.rs:29` |
| `vantadb::VantaSearchExplanationHit` | `vantadb::sdk::SearchExplanationHit` | `src/sdk/mod.rs:29` |
| `vantadb::VantaBm25TermContribution` | `vantadb::sdk::Bm25TermContribution` | `src/sdk/mod.rs:25` |
| `vantadb::VantaValue` | `vantadb::sdk::Value` | `src/sdk/mod.rs:32` |
| `vantadb::VantaNamespaceStats` | `vantadb::sdk::NamespaceStats` | `src/sdk/mod.rs:28` |
| `vantadb::VantaOperationalMetrics` | `vantadb::sdk::OperationalMetrics` | `src/sdk/types.rs:129` + re-export `mod.rs:30` (verificar path raíz vs sdk) |
| `sdk::VantaFilterOp` / `VantaFilterOp::Eq` | `sdk::FilterOp` / `FilterOp::Eq` | `src/sdk/mod.rs:26`, `types/record.rs:13-14` |
| `VantaConfig { storage_path, audit_log_path, ..Default }` | `Config { storage_path, audit_log_path, ..Default }` | `src/config.rs:574,783` — campos existen, struct literal compatible |
| `Error::DatabaseBusy/String, NodeNotFound(u128), Io(std::io::Error), code()` | idénticos en `error::Error` | `src/error.rs:125,161,292,337` — `from_core` y tests compatibles sin cambio de lógica |

**Nota Error path:** `vantadb::error::Error` es el path canónico (`src/error.rs:122`, `lib.rs:92 pub mod error`). Verificar si hay re-export en raíz (`VantaError` NO existe en raíz — E0432 lo prueba). Uso `vantadb::error::Error` en firmas + `use vantadb::error::Error as CoreError` en imports.
**Nota OperationalMetrics path:** `src/sdk/types.rs:129 pub struct OperationalMetrics`, re-exportado en `sdk/mod.rs:30`. Si no hay re-export raíz, uso `vantadb::sdk::OperationalMetrics`.

**Callers (codegraph):** desktop `VantaError` 162 callers internos — todos usan el tipo LOCAL (`crate::error::VantaError`), no se tocan; solo cambian `from_core`/`map_core_error` y tests que nombran `vantadb::VantaError`. `VantaQueryResult` local 11 callers — no se toca; solo `core_query_to_wire` (param `CoreQueryResult` → `QueryResult`).

## 8. INVESTIGACIÓN PROBLEMA — alcance real del drift

**Veredicto: SOLO RENOMBRE, API compatible.** Evidencia:
1. `cargo check -j 2 --tests` local reproduce E0432/E0425/E0433 en 8 puntos (imports + usos encadenados) — ningún error de tipo/firma/método faltante más allá de los nombres.
2. Cada símbolo nuevo existe con la misma forma: `Embedded::open/open_with_config/test_empty` (`builder.rs:71,99,143`), `Config{storage_path, audit_log_path}` (`config.rs:574,783`), `Error::{NodeNotFound, DatabaseBusy, Io, code()}` (`error.rs`), `FilterOp::Eq` (`record.rs:14`), `Value` variantes idénticas (String/Int/Float/Bool/Null/DateTime/List*), `NodeRecord{fields, vector, edges, id}`, `QueryResult::{Read, Write, StaleContext}` (usos en `core_query_to_wire` coinciden por construcción — el compilador lo confirma al pasar el check).
3. `graph::TraversalDirection` y `node::SparseVector` NO cambiaron — sus imports se quedan.
4. Stop del plan (API incompatible → DEFER) NO dispara. Sin rediseño de desktop.

## 9. INVESTIGACIÓN INTERNET

N/A — todo local; sin red. Nada que marcar (TSYS-13 no aplica: cero URLs citadas como evidencia).

## 10. VALIDACIÓN+CIERRE

- Verify contrato: `cargo check -j 2 --tests` + `cargo test -j 2` en `desktop/src-tauri` verdes local
- Full del scope: `cargo fmt --check` + `cargo clippy -j 2 --tests -- -D warnings` del workspace aislado
- OCR delegation (`pwsh dev-tools/ocr-review.ps1`; Critical/High = bloquea) — advisory
- DoD 3 niveles (contrato + task file + recitation; commit `fix:`; sin deuda)
- P2-01 lo hace el orquestador (no yo) · Gates D/V/C vía `question` (tool no disponible en este runtime → evaluados inline con motivo en RESULTADO)
- RESULTADO §7 obligatorio al final

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `connections/native.rs` (1241L), `error.rs` (271L), `commands/connection.rs` (123L), `lib.rs` (228L), `commands/metrics.rs` (32L), `connections/mod.rs` (32L) + parciales `commands/memory.rs` (§§ relevantes), `connections/manager.rs:380-439`, `connections/types.rs:210-269,640-679`
- **Referencias hacia dentro (lo que el cambio necesita):** `src/sdk/mod.rs`, `src/sdk/builder.rs`, `src/config.rs`, `src/error.rs`, `src/node/vector_data.rs`, `src/graph.rs` (todos solo lectura)
- **Referencias entrantes (quién usa lo que cambio):** 162 callers de `VantaError` local + 11 de `VantaQueryResult` local — no afectados (tipos locales intactos); tests tauri (`tests/`, `native.rs` tests, `memory.rs` tests, `error.rs` tests) — beneficiados (vuelven a compilar)
- **Veredicto:** impacto contenido a 7 archivos de `desktop/src-tauri/src`; cero cambios `src/`; cambio mecánico de nombres sin alteración de lógica ni de DTOs wire. Reversible por slice.

## Steps (~100 líneas c/u)

- [x] **Step 1 — `connections/native.rs` (imports + CoreQueryResult + NamespaceStats + Value + Memory* usos):** reescrito bloque `use vantadb::{...}` → root re-exports + `config::Config` + `error::Error as CoreError` + alias `CoreMemoryRecord`/`CoreMemoryFilterItem` (colisión con DTOs locales); verify check parcial ✅
- [x] **Step 2 — `commands/connection.rs` + `error.rs` + `lib.rs`:** `Embedded`, `error::Error`, `Config`; verify check parcial ✅
- [x] **Step 3 — `commands/memory.rs` + `commands/metrics.rs` + `connections/manager.rs` + `connections/types.rs` + `connections/trait.rs` (docs):** resto + `FilterOp`, `sdk::OperationalMetrics`, tests; `cargo check -j 2 --tests` 0 errores ✅
- [x] **Step 4 — VERIFY full + commit:** `cargo test -j 2` 106/106 ✅, `fmt --check` ✅ (1 reorder auto), `clippy -- -D warnings` ✅ (10 warnings pre-existentes `wal_sharded.rs`, fuera de scope), OCR advisory ✅ (solo inventario; cambio mecánico simétrico 125+/125-, sin trust-boundary), commit selectivo `fix: FIND-124 — ...`, recitation + RESULTADO

## Context Save Point (FINAL)

- **Postura:** COMPLETO 2026-09-19 — 9 archivos `desktop/src-tauri/src`, diff 125+/125- simétrico (renombre puro, cero lógica)
- **Verify contrato:** `cargo check -j 2 --tests` ✅ 0 errores · `cargo test -j 2` ✅ 106 passed/0 failed · `cargo fmt --check` ✅ · `cargo clippy -j 2 --tests -- -D warnings` ✅
- **Commit:** `fix: FIND-124 — ...` (solo 9 src + task file; WIP ajeno intacto; NO PUSH — lo hace vanta-lead)
- **CI ×3:** la corre el lead/orquestador tras push (`cargo test` en workspace aislado; toolkits OS ya resueltos en CI según pre-mortem)
- **Deuda:** ninguna. **Hallazgo:** `trait.rs`/`types.rs` tenían docs con nombres viejos — incluidos en el mapeo (contrato: TODOS).
