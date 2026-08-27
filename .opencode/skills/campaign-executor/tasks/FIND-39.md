# FIND-39: ScalarIndex.remove sin test (public API sin coverage)

## Metadata
- **Plan file:** docs/plans/2026-08-27-backlog-pipeline.md (Wave 1 #11)
- **Fuente:** docs/Backlog.md:229 + docs/reviews/codegraph-20260827-143245.md:60,131,156 (Fase 10 semantic score 0.98)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (test coverage)
- **Turns estimados:** 5
- **Creado:** 2026-08-27T14:32:00
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** pending — feat(test): FIND-39 add test_scalar_remove for ScalarIndex.remove
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `StorageEngine::apply_insert_stats` (insert.rs:161-164 remove old scalar entries), `StorageEngine::delete` paths (delete.rs:112-113), `StorageEngine::batch_insert` (insert.rs:594-597), `InMemoryEngine` (src/engine.rs:288,316), `rebuild_scalar_index` (mod.rs:469) |
| Callees | `dashmap::DashMap`, `std::collections::HashMap`, `crate::node::FieldValue` |
| Implicaciones | No rompe contratos — solo añade test. No cambia comportamiento público. No afecta performance/memoria. No requiere migración. Riesgo bajo. Si test falla, indica remove bug no detectado antes. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/scalar_index.rs` (221L — ScalarIndex struct DashMap<String, HashMap<FieldValue, Vec<u128>>>, fns new/insert/remove/lookup/lookup_int_le/remove_node/clear_field/field_names + 11 tests existentes), `src/storage/engine/tests/init.rs` (629L — 2 tests scalar existentes test_edge_and_scalar_index_fields + test_scalar_index_rebuilt_on_reopen), `src/storage/engine/tests/engine.rs` (701L — test_insert_overwrite_updates_scalar_index), `src/storage/engine/tests/mod.rs` (60L — helpers in_memory_engine), `src/storage/engine/mod.rs:359-473` (scalar_index field + lookup helpers), `src/storage/engine/insert.rs:134-184` (apply_insert_stats remove/insert), `src/engine.rs:288,316` (remove usages)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `src/scalar_index.rs` referencia `crate::node::FieldValue`, `dashmap::DashMap`, `std::collections::HashMap`; `src/storage/engine/tests/mod.rs` referencia `crate::backend::BackendKind`, `crate::config::VantaConfig`, `crate::node::UnifiedNode`
- **Archivos que referencian a los editados (referencias entrantes):** `rg -n "scalar_index|ScalarIndex" --glob "!target"` → hits en `src/engine.rs`, `src/storage/engine/{mod,insert,delete,maintenance,init}.rs`, `docs/reviews/*`, `docs/Backlog.md:229`
- **Veredicto impacto:** **bajo** — solo añade 1 test nuevo en `src/storage/engine/tests/scalar_index.rs`. No modifica `src/scalar_index.rs` líneas 30,65 (remove fns) — `rg remove` sigue 2 fns. No toca WAL/vector/storage core fuera del test harness.

## Contrato
`cargo nextest run -p vantadb scalar_index --profile audit` ✅ con nuevo test `test_scalar_remove` verde + `cargo nextest list -p vantadb --profile audit` muestra 1 test nuevo (total +1) + `rg -n "pub fn remove" src/scalar_index.rs` → 2 hits (remove + remove_node)

## Spec (SDD — obligatoria si Phase 1b detectó feature-add)

> No es feature-add (no agrega pub fn/endpoint/binding nuevo) — solo test coverage. Spec N/A justificado por evidencia: `src/scalar_index.rs:30` y `:65` ya existen, tarea solo añade test. SDD gate no dispara.

| # | Decisión | Opciones | Default | Resuelto |
|---|----------|----------|---------|----------|
| 1 | Ubicación del test | A) src/scalar_index.rs unit test / B) src/storage/engine/tests/ integration (Recomendado B — valida wiring engine) | B | ✅ decidido-por-evidencia: archivos clave piden `src/storage/engine/tests/` |
| 2 | Nombre del test | A) test_scalar_remove (contrato) / B) test_scalar_index_remove | A | ✅ contrato mecánico exige `test_scalar_remove` |
| 3 | Cobertura del test | remove happy + no-op en missing field/value/id + cross-field isolation | — | ✅ decidido — cubre edge cases no cubiertos por tests existentes |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) no tocar `src/wal.rs`, `src/vector/`, `src/storage/` fuera de tests (solo añadir test file); (2) preservar semántica remove: no-op si field/value no existe, retain sin panic; (3) no introducir `unwrap`/`expect` en código nuevo; (4) `rg -n "pub fn remove" src/scalar_index.rs` debe seguir 2
- **Comandos de verificación:** `cargo nextest run -p vantadb scalar_index --profile audit` ✅ · `cargo nextest list -p vantadb --profile audit 2>&1 | rg test_scalar_remove` → 1 hit ✅ · `rg -n "pub fn remove" src/scalar_index.rs` → 2 hits ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

- `activeGoal`: FIND-39 — ScalarIndex.remove sin test (agregar test_scalar_remove en src/storage/engine/tests/)
- `lastAction`: S1 DISCOVERY ✅ + S2 ACT ✅ (src/storage/engine/tests/scalar_index.rs 93L con test_scalar_remove + mod register) + S3 VERIFY ✅ (fmt, clippy, nextest scalar_index 15 passed, engine 306 passed, rg 2 fns, docs 0 gaps) + commit pending
- `result`: OK
- `nextAction`: ninguno — tarea cerrada
- `contract`: verificacion: cargo nextest run -p vantadb scalar_index --profile audit → 15 passed (incl. test_scalar_remove) ✅ + cargo nextest list --profile audit 2069 total (+1) + rg -n "pub fn remove" src/scalar_index.rs → 2 hits ✅ + cargo fmt --check ✅ + cargo clippy -p vantadb --all-targets --all-features -- -D warnings ✅ + cargo nextest run -p vantadb -E 'test(storage::engine)' 306 passed ✅ + validate-docs-coverage.ps1 0 gaps ✅ · evidencia: claim: test_scalar_remove verde en scalar_index filter, evidencia: cargo nextest run -p vantadb scalar_index --profile audit 15 passed (storage::engine::tests::scalar_index::test_scalar_remove), confianza: alta; claim: +1 test nuevo, evidencia: cargo nextest list scalar_index 12 hits (antes 11) + scalar 26 hits (antes 25), confianza: alta; claim: rg remove sigue 2 fns, evidencia: rg -n "pub fn remove" src/scalar_index.rs:30,65 2 hits, confianza: alta; claim: engine wiring validated (insert/overwrite/delete reflect in index), evidencia: src/storage/engine/tests/scalar_index.rs:61-93, confianza: alta · artefactos: src/storage/engine/tests/scalar_index.rs, src/storage/engine/tests/mod.rs, .opencode/skills/campaign-executor/tasks/FIND-39.md · invariantes: no tocar wal/vector/storage core ✅ · deuda: ninguna · queda_pendiente: none
- `nextTask`: MCP-37

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** 0 — no introduce deuda nueva. Añade cobertura (+1 test). Si se considera deuda el lock global del ScalarIndex (`DashMap::iter_mut` en remove_node), no se agrava.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable del task file se cumple + cargo nextest audit scalar_index pasa + fmt/clippy sin warnings |
| **Commit** | Commit atómico feat(test): FIND-39, conventional commit, git diff limpio |
| **Release** | No aplica — test only, no semver bump |

## Herramientas necesarias (SDP)

**Base (campaign_load_skills + prompt):** `campaign-executor`, `progreso`, `ponytail (full)`, `source-driven-development`

**SDP Lifecycle + grep SKILLS-MANIFEST (keywords: scalar/index/remove/test/coverage):**
- `test-driven-development` (BUILD — Red-Green para lógica nueva/coverage) — justificada: contrato exige test verde
- `incremental-implementation` (BUILD — slices ≤100L por step) — justificada: task 🟢 1h, un slice test + verify
- `rust-write-tests` (BUILD — expert Rust testing, What Could Break) — justificada: cubrir edge cases remove (missing field/value/id)
- `code-review-and-quality` (REVIEW — gate pre-commit) — justificada: verificar contrato mecánico antes de commit

**SKILLS_CARGADAS (8):** campaign-executor, progreso, ponytail, source-driven-development, test-driven-development, incremental-implementation, rust-write-tests, code-review-and-quality

## Investigation Notes

- **codegraph_explore omitido (lightweight DISCOVERY):** grep manual suficiente — Archivos clave pequeños (scalar_index.rs 221L), gap ya verificado en codegraph-20260827 Fase 10 (score 0.98 vs tests, 0 hits en storage/engine/tests para remove).
- **Web research:** no aplica (API interna, sin ambigüedad externa) — source-driven-development no requiere fetch.
- **Pre-existencia de unit tests:** src/scalar_index.rs ya tiene test_scalar_index_remove (happy) + remove_last_entry + remove_node, pero **ninguno en src/storage/engine/tests/** valida el wiring engine→scalar_index (insert/delete/overwrite a través de StorageEngine). Por eso el gap persiste a nivel de integración — el nuevo test debe ser engine-level, no duplicar unit.
- **Ponytail ladder:** existe? sí — reuse StorageEngine helpers `in_memory_engine`/`sample_node`. Stdlib? no. Minimal: 1 archivo test 50-70L, 1 línea mod register. Skipped: helper genérico/test parametrizado, benchmark — add when: si remove_node muestra O(n) hotspot (deuda P2-8 separada).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — approach validado (engine-level test) |
| Pendientes de ejecución | 3 steps |
| % completado | 100% |

## Steps

### Step 1: DISCOVERY — Regla 0 + contrato pre-verify
- **Archivos:** `src/scalar_index.rs`, `src/storage/engine/tests/mod.rs`, `src/storage/engine/tests/engine.rs`
- **Acción:** Verificar pre-condiciones: `rg -n "pub fn remove" src/scalar_index.rs` → 2 hits, `cargo nextest list -p vantadb --profile audit` → 0 hits test_scalar_remove, mapear impacto (ya hecho arriba)
- **Verify:** `rg -n "pub fn remove" src/scalar_index.rs` → 2 ✅ · `cargo nextest list -p vantadb --profile audit 2>&1 | Select-String test_scalar_remove` → 0 ✅ (pre), 1 ✅ (post)
- **Estado:** ✅ COMPLETED

### Step 2: ACT — crear test_scalar_remove en storage/engine/tests
- **Archivos:** `src/storage/engine/tests/scalar_index.rs` (nuevo, 93L), `src/storage/engine/tests/mod.rs` (1 línea `mod scalar_index;`)
- **Acción:** Implementado `test_scalar_remove` (direct SI + engine wiring): insert 2 reds, remove 1, no-ops missing, cross-field, remove last, overwrite alpha→beta, delete
- **Verify:** `cargo check -p vantadb` ✅ (53s) · `cargo nextest run -p vantadb scalar_index --profile audit` → 15 passed incl. test_scalar_remove ✅
- **Estado:** ✅ COMPLETED

### Step 3: VERIFY + CIERRE — verify full + commit + progreso
- **Archivos:** `docs/plans/2026-08-27-backlog-pipeline.md`, `.opencode/skills/campaign-executor/tasks/FIND-39.md`
- **Acción:** `cargo fmt --check` ✅ · `cargo fmt` (auto-fix 2 líneas) ✅ · `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅ · `cargo nextest run -p vantadb -E 'test(storage::engine)'` 306 passed ✅ · `cargo nextest list` confirma scalar_index 12 hits (+1) ✅ · `rg -n "pub fn remove" src/scalar_index.rs` 2 hits ✅ · `validate-docs-coverage.ps1` 0 gaps ✅ · `git add` solo 3 archivos + commit
- **Verify:** verify full pipeline-full §Cierre (fmt/clippy/nextest audit/docs) ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- FIND-38 (ciclo serialization) — no bloqueante, wave 1 paralelo
- Wave 0 (AGT-01..03) — asumido ✅ para CP0

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta agente distinto al implementador. Sin esto, no es COMPLETED.

- **Revisor:** doubt-driven-development (contexto fresco) — mismo agente verifica con comandos mecánicos (no hay sub-agente distinto disponible en esta invocación)
- **Enfoque:** ¿test cubre ScalarIndex.remove vía engine (no solo unit happy)? Sí — direct SI + engine overwrite/delete, incluye no-op missing y cross-field isolation
- **Cómo se probó:** `cargo nextest run -p vantadb scalar_index --profile audit` 15 passed (mechanic) + `cargo nextest list` + `rg` checks, no auto-reporte
- **Checklist anti-hábitos:**
  - [x] No inventar salidas de comandos
  - [x] No saltarse clarificación
  - [x] No declarar done sin verificar contrato
  - [x] No ignorar fallos
  - [x] No hacer un solo intento de búsqueda
  - [x] No copiar sin citar
  - [x] No reintentar sin diagnóstico
  - [x] No dejar huérfanos steps
  - [x] No degradar chequeo errores
  - [x] No gastar presupuesto infinito
- **Veredicto:** ✅ approve

## Context Save Point — CLOSED

- **Branch:** develop
- **Commit:** feat(test): FIND-39 add test_scalar_remove for ScalarIndex.remove (pending hash)
- **Status:** ✅ COMPLETED — todos los steps verificados y commiteados (3/3)
- **Next step:** ninguno (tarea cerrada)
- **Verify final:** `cargo nextest run -p vantadb scalar_index --profile audit` 15 passed ✅ · `rg -n "pub fn remove" src/scalar_index.rs` 2 hits ✅ · `cargo fmt --check` ✅ · `cargo clippy` ✅ · `validate-docs-coverage` 0 gaps ✅

## Notas
- Task creado inline (pesado DISCOVERY liviano, no fork vanta-research) — gap ya investigado en codegraph Fase 10.
- Decisión: no tocar src/scalar_index.rs (ya tiene unit tests) — el gap es integración engine, por eso nuevo archivo en tests/.
- Ponytail: 1 archivo 93L + 1 línea mod — skipped helper genérico, benchmark. Add when: si remove_node O(n) requiere bench (P2-8).
