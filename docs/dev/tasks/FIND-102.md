# FIND-102 — `tests/sdk_serialization.rs` compila (39 errores `QueryResult` pre-existentes)

- **Objetivo:** test roto pre-existente ensucia `cargo check --tests`. Evidencia EMB-10 `docs/dev/tasks/EMB-10.md:174`: "`tests/sdk_serialization.rs` no compila (39 errores, `QueryResult` no declarado) — pre-existente, ajeno a este task". Fix probablemente mecánico (imports/tipos).
- **Contrato:** `cargo check -p vantadb --tests` verde (o el scope mínimo que incluya ese archivo) + sin cambiar asserts existentes salvo que estén mal (y entonces documentar por qué).
- **AC:** (1) check verde en el scope del contrato; (2) asserts intactos salvo mal-documentado con motivo; (3) prohibido borrar tests para "arreglar" (Regla 2) y prohibido reescribir la suite; (4) si no es mecánico → DEFER con diagnóstico, no reescribir.
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟢 Baja · **Wave:** Wave2 primera en secuencia (Wave0 ✅ ×3: FIND-109/110/111; Wave1 ✅ ×3: FIND-112/113/101; FIND-114 + FIND-115 después) · **NextTask:** FIND-114 (orquestador).
- **Branch:** develop · **Commit:** `test: FIND-102 — ...` (NO PUSH, staging selectivo).
- **SDP:** `campaign_discover_skills_v2 archivosClave="tests/sdk_serialization.rs" phase="BUILD" contractKeywords=["cargo check tests verde","QueryResult","sdk serialization","bug fix test roto"] maxSkills=8` → campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, frontend-ui-engineering, api-and-interface-design (todas score 1.00 lifecycle/base). Cargadas en sesión: systematic-debugging (clasificar 39 errores), test-driven-development (Prove-It/RED-GREEN), incremental-implementation (slice único verify→commit). `frontend-ui-engineering` descartada: sin superficie `web/` en el slice. `source-driven-development`/`doubt-driven-development`/`api-and-interface-design` registradas pero no activadas: sin API externa nueva ni trust boundary. SKILLS_CARGADAS en RESULTADO §7.
- **Tipo detectado:** `campaign_detect_task_type archivosClave="tests/sdk_serialization.rs"` → `unknown` ("No detectable", checks: `cargo check -p vantadb`). Workflow cargado: `campaign_get_workflow bug-fix` (localizing → planning → implementing → testing → review → accept → close).

## Archivos

- **Clave:** `tests/sdk_serialization.rs` (371L, 16 tests; `QueryResult` usado en `test_query_result_serialize:178-203`).
- **Relacionados (símbolos que referencia — ubicados en DISCOVERY):** `src/sdk/types/graph.rs:14` (`pub enum QueryResult { Read(Vec<NodeRecord)), Write { affected_nodes, message, node_id }, StaleContext { node_id } }`) · re-export `src/sdk/types.rs:11` (`pub use graph::{EdgeRecord, NodeInput, NodeRecord, QueryResult}`) · re-export `src/sdk/mod.rs:24-33` (`QueryResult` en lista `pub use types::{...}`) · re-export raíz `src/lib.rs:183` (crate-root `QueryResult` = SDK graph result; `engine::QueryResult` queda namespaced por AST-002, `lib.rs:170-171`). Símbolos vecinos del archivo: `MemoryInput`, `MemoryRecord`, `MemorySearchRequest`, `MemorySearchHit`, `MemoryListPage`, `NodeRecord`, `EdgeRecord`, `Capabilities`, `RuntimeProfile`, `ExportReport`, `ImportReport`, `IndexRebuildReport`, `TextIndexAuditReport`, `OperationalMetrics`, `SearchExplanation`, `StorageTier`, `Value`, `DistanceMetric` — todos resueltos vía `vantadb::sdk::*`.
- **Prohibidos (intocables):** reescritura de suite; cambios fuera del archivo salvo imports necesarios; resto del repo (`src/`, otros tests); `reparacion.bat`; `.opencode`; `Justfile`; `ocr-*`; `completions/*`; `desktop/src-tauri/Cargo.lock`; stash@{0} GOV-C4; `docs/dev/Backlog.md`; plan file (solo recitation); `C:/Users/Eros/.vantadb*`. WIP ajeno verificado intocable (`git status`: `M .opencode`, `M completions/*` ×4, `M docs/dev/Backlog.md`, `M docs/dev/avance/activo/core-engine.md`, `?? reparacion.bat`, `?? docs/dev/plans/2026-09-17-seguimiento-mvp.md`).

## Dependencias

- **Wave2 primera en secuencia.** Wave0 ✅ ×3 (FIND-109 commit 9c881ac5, FIND-110 commit 1b3b3409, FIND-111 commits 234f0627+e61a15a3). Wave1 ✅ ×3 (FIND-112 commit 062300a8, FIND-113 commit b2b20073, FIND-101 commit de3bd119). FIND-114 + FIND-115 después (disjuntos: examples · docs/web).
- **Stop:** no-mecánico (rediseño escondido) → DEFER con diagnóstico (no reescribir). Pre-mortem plan: (1) rediseño escondido → DEFER; (2) borrar tests → prohibido (Regla 2).
- **NextTask:** FIND-114 (orquestador).

## Referencias

- **Refs:** `test-suite.md` (runner `cargo nextest run --profile audit --workspace --build-jobs 2`; single-test `cargo nextest run --profile audit -p vantadb --test <name>`), `definition-of-done.md` (standing checklist + DoD VantaDB: check/tests/clippy/fmt/OCR/docs/dev/avance + v1 baseline + progreso Trigger 1 A-G). `clean-code-clean-architecture.md` leído completo incl. Apéndice V (mapa capas→repo, severidades; sin deuda nueva 🟡/🔴 en este slice — cero código tocado).
- **Rules:** N/A con motivo — el slice no toca `src/` (cero ediciones de código; veredicto DISCOVERY: ya verde, fix innecesario). Si hubiera tocado `src/sdk/` habría aplicado `.opencode/rules/api-contract.md` + `core-engine.md`; al ser solo tests, no dispara regla de área.
- **Commands:** `pipeline.md` (vía `pipeline-full.md` exacto).
- **SPEC.md raíz:** leído (MVP memoria automática; F1-F8). Tabla Spec: N/A (tests, sin símbolos públicos nuevos — 0 `pub fn`/tool/endpoint/binding; Gate D spec-first no aplica a bug-fix/test-only).
- **Evidencia previa:** `docs/dev/tasks/EMB-10.md:174` (FIND-D: 39 errores pre-existentes).

## Skills (base en sesión + sugeridas)

- Base sesión (plan): campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, ponytail(full).
- Sugeridas plan FIND-102: systematic-debugging (clasificar los 39 errores) · test-driven-development. Ambas cargadas vía `skill`. SDP real Paso 0b ejecutado (ver línea SDP arriba), ≤8.

## Herramientas + MCP

- `codegraph_explore "QueryResult sdk definition sdk_serialization"` — ubicar `QueryResult` y símbolos (blast radius + source verbatim).
- `cargo check -p vantadb --tests -j 2` (contrato; Cargo `-j 2` siempre por riesgo OOM).
- `cargo test -p vantadb --test sdk_serialization -j 2` (runtime de los 16 tests).
- `cargo fmt --check -p vantadb` + `cargo clippy -p vantadb --tests -j 2 -- -D warnings` (DoD Quality).
- `campaign_verify_cmd` (bug exit -1 → bash directa; documentado en RESULTADO si aplica).
- Internet N/A (sin ambigüedad de API externa; todo local).

## Investigación código (DISCOVERY)

- **Clasificación de los 39 errores: MECÁNICO-RESUELTO, no rediseño.** A fecha de ejecución (2026-09-18, branch develop @ de3bd119) el archivo compila y pasa: `cargo check -p vantadb --tests -j 2` ✅ (Finished dev profile, 33.29s, 0 errores); `cargo test -p vantadb --test sdk_serialization -j 2` ✅ 16/16; `cargo fmt --check -p vantadb` ✅ exit 0; `cargo clippy -p vantadb --tests -j 2 -- -D warnings` ✅ 0 warnings.
- **Dónde vive cada símbolo (todos resueltos, 0 no-declarados):** `QueryResult` = `src/sdk/types/graph.rs:14` (enum SDK Read/Write/StaleContext) re-exportado en `types.rs:11` → `sdk/mod.rs:29` → `lib.rs:183`. Distinto de `src/engine.rs:40` (`pub struct QueryResult { nodes, is_partial, exhaustivity, source_type }` — engine interno, namespaced `engine::QueryResult` por AST-002, no colisiona). Resto de símbolos del test: `Value` (`types.rs:81`), `MemoryInput/MemoryRecord` (`types/record.rs` vía `types.rs:12-16`), `MemorySearchRequest/Hit` (`types/search.rs` vía `types.rs:22-26`), `NodeRecord/EdgeRecord` (`serialization::graph_types` vía `types/graph.rs:8`), `Capabilities/RuntimeProfile/StorageTier/OperationalMetrics` (`types.rs:61-245`), reports (`ExportReport/ImportReport/IndexRebuildReport/TextIndexAuditReport` en `types/record.rs`/`types/search.rs`), `SearchExplanation` (`types/search.rs`), `DistanceMetric` (`crate::DistanceMetric` raíz).
- **Blast radius (codegraph):** `QueryResult` SDK tiene 1 caller en `src/sdk/types.rs` + conversiones `src/sdk/serialization/conversions.rs:116` (`From<ExecutionResult>`) + API `src/sdk/api/graph.rs:244` (`query() -> Result<QueryResult>`) + binding `vantadb-python/src/convert.rs:325-390` (format + pydict). Cero cambios en este slice → blast radius no activado; si se hubiera editado el test, afectados: solo el harness de ese archivo (16 tests), sin callers productivos.
- **Historial:** `git log --follow -- tests/sdk_serialization.rs` → último toque `d524c67b refactor!: AST-002 Rust tipos sin stutter + aliases deprecated`. El drift EMB-10 (2026-09-16) es anterior a los refactors de estabilización del boundary SDK (FIND-49 split `sdk/types` por dominio + AST-002 renames + re-exports); a HEAD el boundary expone `QueryResult` y el test compila sin tocarlo.

## Investigación problema (causa del drift)

- **Causa:** símbolo renombrado/reubicado sin actualizar test en su momento (drift test-vs-SDK): el test referenciaba `QueryResult` vía `vantadb::sdk::*` cuando el boundary aún no lo re-exportaba (o lo exponía con otro path/nombre pre-FIND-49/AST-002). Los refactors intermedios (split `src/sdk/types/` por dominio FIND-49 `e3711dea`, renames sin stutter AST-002 `d524c67b` + aliases deprecated, re-exports `types.rs:11`/`mod.rs:29`/`lib.rs:183`) restauraron el path estable y el test volvió a compilar sin intervención del test. **Evidencia:** a HEAD `cargo check` 0 errores + `QueryResult` definido y re-exportado en las 3 capas (def-site, `types`, `sdk`, raíz) + `engine::QueryResult` explícitamente namespaced para evitar la colisión que probablemente originó el "no declarado".
- **Por qué no es rediseño escondido:** 0 errores actuales, 16/16 asserts pasan sin modificarlos, 0 warnings clippy, fmt limpio. No hay síntoma residual que pida rediseño. Stop DEFER no dispara.

## Investigación internet

- N/A (sin APIs externas ambiguas; todo el diagnóstico es local: código + git + toolchain).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `tests/sdk_serialization.rs` (371L), `src/sdk/mod.rs` (33L), `src/sdk/types.rs` (head + re-exports), `src/sdk/types/graph.rs:1-32` (def-site `QueryResult`, vía codegraph verbatim), `docs/dev/tasks/EMB-10.md:160-187` (evidencia FIND-D + spec gates), `SPEC.md` (head 60L), `docs/dev/tasks/FIND-101.md` (patrón task file), plan file `docs/dev/plans/2026-09-17-seguimiento-mvp.md` §Task 7 + recitations.
- **Referencias hacia dentro (el slice usa):** `cargo check/test/fmt/clippy` (solo lectura + ejecución); `codegraph_explore`; `git log/status/diff`.
- **Referencias entrantes (dependen del cambio):** ninguna — el slice no edita código ni tests (cero diff productivo). Único artefacto nuevo: este task file.
- **Veredicto:** impacto nulo en runtime (verify-only). Sin símbolos públicos nuevos. Sin hot path (`vector/`, `engine.rs` search loop no tocados). Sin trust boundary (sin input/auth/storage/FFI/red). Rollback: borrar este task file del commit (`git revert` limpio, aditivo docs-only).
- **Gate D (question-gates.md):** evaluado pre-task-file — **no disparado** (1 archivo test, 0 símbolos públicos nuevos, sin hot path/API pública, contrato mecánico no ambiguo, blast radius ≤10).

## Steps

- [x] **Step 1 — DISCOVERY:** zero-code planning + clasificación 39 errores (mecánico-resuelto) + ubicación `QueryResult` + task file. Verify: task file existe con las 10 secciones de detalle obligatorio.
- [x] **Step 2 — VERIFY CONTRATO (ACT verify-only, TDD N/A — sin código que fijar):** `cargo check -p vantadb --tests -j 2` ✅ + `cargo test -p vantadb --test sdk_serialization -j 2` ✅ 16/16 (asserts intactos, 0 modificados) + `cargo fmt --check` ✅ + `clippy --tests -D warnings` ✅. RED/GREEN no aplica: no hay reproducción pendiente (el "failing test" histórico ya pasa a HEAD); forzar un RED artificial violaría Scope Discipline.
- [x] **Step 3 — CLOSE:** OCR delegation ✅ (docs-only excluido, sin bloqueos) + DoD 3 niveles ✅ + commit `test:` (solo task file; plan recitation en worktree, precedente FIND-101) + `campaign_update_task_state` + RESULTADO §7.

## Validación + cierre

- **Verify contrato:** `cargo check -p vantadb --tests -j 2` ✅ · `cargo test -p vantadb --test sdk_serialization -j 2` ✅ 16/16 · asserts intactos (0 cambios; nada mal que documentar).
- **OCR delegation:** ✅ ejecutado `pwsh dev-tools/ocr-review.ps1 -Format json` — `docs/dev/tasks/FIND-102.md` sale como `excluded_files/unsupported_ext` (.md advisory, sin regla aplicable); único `rule group default` cubre WIP ajeno `completions/*` (no del slice, no commitear). 0 Critical/High del slice → no bloquea commit.
- **DoD 3 niveles:** Correctness (contrato ✅, runtime verificado 16/16 real no solo check, sin regresiones — suite del archivo verde; edge/error paths N/A verify-only) · Quality (scope estricto: 0 archivos productivos tocados; fmt+clippy ✅; sin dead code/duplicación) · Integration (sin firmas tocadas; paridad SDK intacta) · Documentation (este task file; `docs/api/` N/A — sin API cambiada) · Ship-readiness (sin trust boundary nuevo, sin deuda nueva, rollback `git revert` docs-only; Regla 6: deuda neta 0).
- **P2-01 orquestador:** `vanta-review` fresh-context al cierre (veredicto en recitation/RESULTADO; el implementador no se auto-audita).
- **Gates D/V/C vía `question`:** D evaluado (no disparado, motivo ≤6 palabras: "test-only, sin API nueva"); V solo si 2 fallas mismo-error (no ocurrido); C al cierre sobre colaterales (propuesta: ninguno — `git status` ajeno pre-existente, no incluir).

## Notas

- `ponytail:` fix mínimo = cero código; el drift ya lo pagaron FIND-49/AST-002 (re-exports estables). Forzar edición del test para "dejar huella" sería scope-creep.
- NOTICED BUT NOT TOUCHING: WIP ajeno en `git status` (`.opencode`, `completions/*`, `docs/dev/Backlog.md`, `docs/dev/avance/activo/core-engine.md`, `reparacion.bat`, plan file untracked) → no incluir en staging (staging selectivo solo `docs/dev/tasks/FIND-102.md` + recitation del plan).
- Secrets: ninguno tocado ni a disco.
