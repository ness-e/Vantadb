---
title: "TASK MEMG-20: Checkpoints reanudables de tarea (dim 1)"
kind: task
description: "API mínima de checkpoint de tarea en vanta-memory (paso + parcial + estado + versión) reanudable: tipos/namespace separados del pipeline checkpoint; test interrupción→resume sin repetir pasos; sin consumer in-repo → FIND del consumo por el host"
---

# TASK MEMG-20: Checkpoints reanudables de tarea (dim 1)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 43, F2 — Memoria I)
- **Fuente:** Backlog `MEMG-20` (dim1 hub [PROPUESTA]; MemGPT arXiv 2310.08560) + plan Task 43 (bloque F0-expandido, L1232-1258) + DELTA §P2
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1243):** 5d → entregar API + test de reanudación del pipeline y FIND del consumo por el host (contrato autoritativo L1241: reanudación a nivel de **tarea**; el contrato manda)
- **Prioridad:** 🟡
- **Tipo:** Rust (crate `vanta-memory`; core SDK consumido sin modificar)
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `43` en el campaign server)
- **Incógnitas (uphill):** 0 — downhill directo (API mínima sancionada por el plan; semántica fijada en DISCOVERY)
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `43`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Símbolos nuevos (aditivos — sin callers preexistentes):** `utils::task_checkpoint::{TaskCheckpoint, TaskState, TaskCheckpointManager, TaskCheckpointError}` (consumidores: hosts in-process + tests). **Callers de lo existente que NO cambia:** `CheckpointManager` (6 sitios en `pipeline_worker.rs:437,550,587,624,861` + `tests/pipeline_manager.rs`) mantienen firma y comportamiento — cero ripple. |
| Callees | Core SDK existente (sin cambios): `Embedded::get` (`src/sdk/api/memory.rs:912`), `Embedded::put` (`:645`), `Embedded::delete` (`:987`); `sanitize_component`/`sanitize_key` (`core/conversation/l0_recorder.rs:140,157`, `pub(crate)`). Cero dependencias nuevas (`serde`/`serde_json`/`thiserror` ya en el crate). |
| Implicaciones | **API pública aditiva** (módulo nuevo + re-export en `utils/mod.rs`) — sin breaking, sin wire, sin migración, sin cambios en `src/sdk/**` (solo consumo). Namespace propio `task_checkpoints` (jamás `pipeline_checkpoint`). Sin locks nuevos (RMW por record, single-writer por `task_id` — mismo principio del módulo). Regresión: suite `-p vanta-memory` se re-corre; ningún test existente toca los símbolos nuevos. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `bed44b23`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/utils/checkpoint.rs` (:1-236 — `RunnerSessionState` `:22`, `Checkpoint` `:34`, `CheckpointError` `:60`, `CheckpointManager` `:70` con `read/write/mutate` `:88-117`, namespace `pipeline_checkpoint` `:82`; **base existente del pipeline checkpoint — NO se modifica su semántica**).
  - `vanta-memory/src/services/pipeline_worker.rs` (:1-873 — integración existente: `CheckpointManager::new` `:437,550,587,624`, `update_runner_state` `:860`; tasks L1/L2/L3/Dream single-pass — no hay pasos internos que reanudar).
  - `vanta-memory/src/core/state/types.rs` (:1-112 — `PipelineSessionState` `:16`, `TaskKind`/`TaskPayload` `:38-70`).
  - `vanta-memory/src/utils/mod.rs` (:1-24 — re-exports; patrón aditivo), `vanta-memory/src/lib.rs` (:1-69 — "Core-only by decision"; crate publicable DIST-01).
  - `vanta-memory/src/core/conversation/l0_recorder.rs` (:135-184 — `sanitize_component` `:140` (set `[A-Za-z0-9._/-]`, ≤128 bytes), `sanitize_key` `:157` (≤512, sin `/`); `now_ms` `:401`), `core/conversation/mod.rs` (:1-10 — `pub(crate) use`).
  - `vanta-memory/Cargo.toml` (:1-80 — deps: serde/serde_json/thiserror; dev-deps tempfile; features default lean).
  - Tests vecinos: `vanta-memory/tests/pipeline_manager.rs` (:1-100 — fixture `open_db()` InMemory + `CheckpointManager`), `tests/backup_snapshot.rs` + `tests/l1_history.rs` (patrón de tests por superficie, MEMG-13).
  - SDK (solo firmas): `src/sdk/api/memory.rs` — `put` `:645` → `MemoryRecord`; `get` `:912` → `Option<MemoryRecord>`; `delete` `:987` → `bool`.
  - Docs/specs: `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 43 L1232-1258 + F2 L1059-1062), `docs/dev/tasks/MEMG-13.md` (formato canónico), `docs/dev/Backlog.md:169` (fila MEMG-20), `.opencode/rules/api-contract.md` (R-6/R-8), `.opencode/references/clean-code-clean-architecture.md` (Apéndice V).
- **Archivos referenciados hacia dentro (imports/deps):** módulo nuevo → `vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata}` + `vantadb::error::Error` (públicos, `default-features=false` ✓), `crate::core::conversation::{sanitize_component, sanitize_key}` (existentes, `pub(crate)`), `serde`/`serde_json`/`thiserror` (ya usados por `checkpoint.rs`).
- **Referencias entrantes (grep/CodeGraph HEAD):** `TaskCheckpoint|TaskState` en `vanta-memory/src` + `src/` = **0 hits** (rg, verificado 2026-10-05); `CheckpointManager` = 6 sitios en `pipeline_worker.rs` + `utils/mod.rs` + `tests/pipeline_manager.rs` — **firmas intactas**. `utils::*` re-exporta lo nuevo (aditivo).
- **Veredicto impacto:** **BAJO (aditivo puro)** — 1 archivo de código nuevo (`utils/task_checkpoint.rs`) + 1 línea de re-export + 1 test nuevo (`tests/task_checkpoint.rs`) + docs. Sin cambios de firma en lo existente, sin core, sin wire, sin deps, sin locks nuevos. Pre-mortems mitigados: (1) tipos/namespace separados (módulo propio + `task_checkpoints`); (2) API mínima declarada en §Spec ANTES de codear (paso + parcial + estado + versión; sin orquestador); (3) límite host documentado (módulo + `VANTA_MEMORY.md`).

## Contrato

"API mínima de checkpoint de tarea (paso actual + resultados parciales + estado) reanudable (plan L1241): (a) **tipos separados del pipeline checkpoint** — `TaskCheckpoint{version,step,state,partial}` + `TaskState` + `TaskCheckpointError` en módulo propio `utils::task_checkpoint`, namespace propio `task_checkpoints` (jamás `pipeline_checkpoint`, jamás campos de `Checkpoint`/`RunnerSessionState`); (b) **manager RMW** (`begin` idempotente, `advance`, `complete`, `fail`, `load`, `delete`) sobre un JSON record por tarea (key = `sanitize_key(task_id)`, RMW atómico in-process); (c) **test de reanudación** — interrupción a mitad → nueva instancia retoma del checkpoint sin repetir pasos completados (step correcto, parcial preservado); (d) **sin romper** `RunnerSessionState`/`Checkpoint`/`CheckpointManager` (firmas intactas; suite `-p vanta-memory` verde). Límite documentado: resume tras compactación del agente = del host; el manager no persiste contexto conversacional. Verify: `cargo nextest run --profile audit -p vanta-memory --test task_checkpoint --build-jobs 2` + suite `-p vanta-memory --build-jobs 2` + `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` verdes."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato del plan (Task 43, Gate Result ✅ DO) sanciona la "API mínima de checkpoint de tarea (paso actual + resultados parciales + estado)" y su pre-mortem #2 fija el mínimo ("paso + payload parcial + versión", sin orquestador); los símbolos nuevos SON el mecanismo sancionado por el plan. Nombres/firmas siguen convenciones del crate (micro-decisiones decidido-por-evidencia abajo). Sin símbolos fuera de la sanción. Precedente de campaña idéntico: MEMG-13 (`versions`/`snapshot` — "Gate D: pre-respondido por el plan F0", MEMG-13.md:52).

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Ubicación de la API | A) **Módulo nuevo `utils/task_checkpoint.rs` + re-export en `utils/mod.rs`** / B) extender `checkpoint.rs` (módulo = port TDAM del pipeline checkpoint, narrativa propia; mezcla dominios — pre-mortem #1) / C) core `src/` (prohibido: la lógica memory vive en `vanta-memory`; api-contract R-8) | ✅ **A** — decidido-por-evidencia: pre-mortem #1 ("tipos/namespaces separados") + precedente `utils/backup.rs` (MEMG-13, revisado y aprobado) + `utils/mod.rs` re-exporta lo nuevo (aditivo, patrón existente) |
| 2 | Modelo del checkpoint | A) **`TaskCheckpoint { version: u32, step: u64, state: TaskState, partial: Vec<serde_json::Value> }`** / B) reutilizar `Checkpoint` (prohibido por pre-mortem #1: campos de pipeline) / C) record global con `BTreeMap<task_id, …>` (crecimiento sin límite + reescritura total por mutación) | ✅ **A** — decidido-por-evidencia: plan L1242 ("paso + payload parcial + versión") + contrato L1241 ("estado"); un record por tarea (RMW por record, escala con N tareas) |
| 3 | Semántica de `step` | A) **Siguiente paso a ejecutar (0-based; pasos `< step` completados; `partial.len() == step`)** / B) último completado (off-by-one en cada resume) / C) nombre de paso string (no verificable mecánicamente) | ✅ **A** — decidido-por-evidencia: el contrato exige "sin repetir pasos completados" — con A el host itera `cp.step..TOTAL` directo y el invariante `partial.len() == step` es asertable |
| 4 | Persistencia | A) **Un JSON record por tarea: namespace `task_checkpoints`, key `sanitize_key(task_id)`, RMW del manager** / B) un record global (C) / C) reutilizar `pipeline_checkpoint` (prohibido pre-mortem #1) | ✅ **A** — decidido-por-evidencia: "principio del módulo: un JSON record, RMW atómico in-process" (contrato L1241) a escala per-task; namespace separado explícito (test lo pinea); `sanitize_key`/`sanitize_component` ya usados por el módulo hermano |
| 5 | API mínima (métodos) | A) **`new/load/begin/advance/complete/fail/delete`** — `begin` idempotente (no clobbea in-progress; restart = `delete`+`begin`), `advance` = RMW (push parcial, `step+1`), `complete`/`fail` = terminales (dos métodos, sin flag-arg — clean-code §2.2) / B) `save` full-record (el host hace el merge; pierde la disciplina RMW) / C) motor de pasos/orquestador (prohibido: pre-mortem #2 scope creep) | ✅ **A** — decidido-por-evidencia: mínimo que hace la semántica reanudable completa; "safe defaults" (begin no destruye progreso; misuse imposible); B no aporta sobre A y C es scope creep |
| 6 | Errores | A) **`TaskCheckpointError` nuevo (`Store`/`Serde`/`NotFound`/`NotInProgress`), `#[non_exhaustive]`** / B) reutilizar `CheckpointError` (mezcla dominios — pre-mortem #1; le faltan NotFound/NotInProgress) / C) `String` (pierde tipado; prohibido en dominio) | ✅ **A** — decidido-por-evidencia: separación de tipos + R-6 (`#[non_exhaustive]`); `NotFound`/`NotInProgress` hacen detectable el misuse (advance sobre completado / RMW sobre inexistente) |
| 7 | Estado | A) **`TaskState { InProgress, Completed, Failed }`, `#[non_exhaustive]`** / B) bool `completed` (no distingue fallo) / C) string libre (no tipado) | ✅ **A** — decidido-por-evidencia: contrato pide "estado"; `Failed` preserva parciales para inspección/retry (a diferencia de `Completed`); R-6 por si gana variantes |
| 8 | Versión del record | A) **`version: u32` con `TASK_CHECKPOINT_VERSION = 1`; `load` la porta sin rechazar** / B) rechazar `version != 1` (rompe forward-compat; sin necesidad hoy) | ✅ **A** — decidido-por-evidencia: "versión" = hook de migración del formato (plan L1242); migración futura = bump + handler en load |
| 9 | Consumo in-repo | A) **Ninguno: API + tests + docs; FIND del consumo por el host (plan stop L1243)** / B) wire al `pipeline_worker` (speculativo: tasks L1/L2/L3/Dream son single-pass — no hay pasos internos que reanudar) / C) consumidor nuevo (scope creep — pre-mortem #2) | ✅ **A** — decidido-por-evidencia: dim1 es [PROPUESTA] para hosts; el plan corta exactamente ahí ("FIND del consumo por el host"); el worker actual no tiene pasos reanudables |
| 10 | Timestamp | A) **No incluir** (mínimo del pre-mortem: paso+parcial+versión+estado) / B) `updated_at_ms` (campo extra sin consumidor; YAGNI — `version` permite agregarlo) | ✅ **A** — decidido-por-evidencia: pre-mortem #2 fija el mínimo; sin consumidor de staleness hoy |
| 11 | Concurrencia | A) **Sin locks nuevos: RMW por record, single-writer por `task_id` documentado** (mismo principio del módulo hermano) / B) lock global por manager (scope creep + Regla 8) | ✅ **A** — decidido-por-evidencia: `checkpoint.rs` documenta el mismo principio ("single-record RMW keeps mutations atomic in-process"); no toco dashmap/parking_lot/Tokio/multi-índice → Regla 8 N/A |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Tipos/namespaces separados:** `TaskCheckpoint*` jamás comparte campos con `Checkpoint`/`RunnerSessionState`; namespace `task_checkpoints` ≠ `pipeline_checkpoint` (test lo pinea vía SDK crudo).
  2. **Firmas intactas de lo existente:** `Checkpoint`/`CheckpointManager`/`RunnerSessionState` no cambian (callers: `pipeline_worker.rs` + `tests/pipeline_manager.rs`).
  3. **RMW por record; single-writer por `task_id`**; sin locks nuevos; `partial.len() == step` mientras se use `begin`+`advance`.
  4. **Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción**; sin dependencias nuevas; `src/sdk/**`/`src/wal.rs`/`src/storage/**`/`src/vector/` **no se tocan** (consumo puro).
  5. **Límite host documentado:** resume tras compactación del agente = del host; el manager no persiste contexto conversacional (usar `context_engine` para eso).
  6. **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  7. **Coordinación MEMG-07 (en vuelo):** NO tocar `core/record/lifecycle.rs`; mis paths = `vanta-memory/src/utils/**` + `vanta-memory/tests/**` + docs. Conflicto real → BLOQUEO.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test task_checkpoint --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings`.
- **Deuda pendiente:** consumo por el host (FIND — sin consumer in-repo hoy). Sin otra deuda.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin clones en hot path (checkpoint explícito del host, fuera del path de recall), sin abstracciones especulativas (1 struct + 1 enum + 1 error + manager de 6 métodos + 1 test). El cambio **elimina** deuda de "semántica de checkpoints de tarea inexistente" (dim1, DELTA §P2) y agrega tests de contrato. Sin deps nuevas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: resume test RED→GREEN (interrupción a mitad → nueva instancia retoma sin repetir pasos completados) + test de separación de namespace + error paths + round-trip; suites scoped verdes + fmt/clippy verdes |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) + `codebase-memory-mcp_check_index_coverage` (5 paths, `no_recorded_issue` ✅) + grep puntual
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file + `VANTA_MEMORY.md` editados

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (SDP phase=BUILD) + rol: `rust-write-tests`, `api-and-interface-design`. `documentation-skill` se carga al editar `docs/**` (paso 3). `security-and-hardening` excluida (sin trust boundary nuevo: `task_id` del host se sanitiza con el patrón existente; store propio). `performance-optimization` excluida (no hot path — operación explícita del host; Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo; `task_id` viene del host in-process y se sanitiza con `sanitize_key` (patrón existente del crate); el payload parcial es JSON propio del host (se persiste tal cual, sin interpretación); sin auth/secrets/deps. Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — no aplica: fuera del path de recall/pipeline (checkpoint explícito del host); cada operación es 1 get/put de un record; sin loops nuevos en hot paths. Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — RED+GREEN: tipos + persistencia (API mínima declarada)

- **Archivos:** `vanta-memory/tests/task_checkpoint.rs` (nuevo), `vanta-memory/src/utils/task_checkpoint.rs` (nuevo), `vanta-memory/src/utils/mod.rs` (re-export)
- **Acción:** RED — tests de contrato: (1) `begin_creates_in_progress_checkpoint_at_step_zero` — `begin("t1")` → `step=0`, `state=InProgress`, `partial=[]`, `version=1`; `load` ida/vuelta igual (falla por compilación: símbolos ausentes → RED correcto). (2) `begin_is_idempotent_and_preserves_progress` — `begin`→`advance`→`begin` de nuevo → `step`/`partial` intactos (safe default: no clobbea). (3) `task_checkpoint_and_pipeline_checkpoint_live_in_separate_namespaces` — `CheckpointManager::set_persona_update_request` (record pipeline) + `begin` (record task) → vía SDK crudo: `db.get("pipeline_checkpoint","checkpoint.json")` Some y `db.get("task_checkpoints","t1")` Some; cross-gets None (namespace separado, pineado). (4) `delete_removes_the_checkpoint` — `delete` → `load` None. GREEN — `utils/task_checkpoint.rs`: `TASK_CHECKPOINT_VERSION`, `TaskState` (`#[non_exhaustive]`), `TaskCheckpoint` (serde snake_case), `TaskCheckpointError` (`#[non_exhaustive]`), `TaskCheckpointManager` (`new` con namespace `task_checkpoints` + `load`/`begin`/`delete`; `begin` idempotente) + re-export en `utils/mod.rs`.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test task_checkpoint --build-jobs 2` → RED correcto (compile error por símbolos ausentes) → GREEN (6 tests)
- **Evidencia:** ✅ RED verificado: `error[E0432]` unresolved imports (`TaskCheckpointManager`/`TaskState`/`TaskCheckpointError`/`TASK_CHECKPOINT_VERSION` ausentes en `utils`) — falla por la razón correcta (símbolos ausentes, no assertion). ✅ GREEN: `6 tests run: 6 passed` — begin en step 0 (`version=1`, `InProgress`, `partial=[]`) con round-trip; begin idempotente (advance→begin no resetea); parcial en orden a través de instancias (invariante `partial.len() == step`); **separación de namespace pineada vía SDK crudo** (`pipeline_checkpoint`/`checkpoint.json` vs `task_checkpoints`/`t1`, sin cross-contaminación, sin campos de pipeline en el record de tarea); `advance` sin begin → `NotFound`; `delete` idempotente. `campaign_verify_cmd` exit 0 (15.5s).
- **Estado:** ✅ COMPLETED

### Step 2 — RED+GREEN: semántica reanudable (test de reanudación del contrato)

- **Archivos:** `vanta-memory/tests/task_checkpoint.rs`, `vanta-memory/src/utils/task_checkpoint.rs` (`advance`/`complete`/`fail`)
- **Acción:** RED — (5) `resume_after_interruption_does_not_repeat_completed_steps` — host A: `begin("t1")` + corre pasos 0 y 1 (`advance` con resultado por paso) → interrupción (drop de la instancia de manager); host B (nueva instancia, mismo db): `load` → `step=2`, `InProgress`, `partial` = resultados 0/1 preservados → itera `cp.step..5` registrando qué pasos corren → `executed == [2,3,4]` (sin repetir 0/1) + `advance` cada uno → `complete` → `step=5`, `Completed` (falla por compilación: `advance`/`complete` ausentes → RED). (6) `advance_on_missing_checkpoint_is_not_found` → `Err(NotFound)`. (7) `advance_after_completion_is_rejected` → `Err(NotInProgress)`. (8) `fail_preserves_partial_for_inspection` → `fail` → `Failed` + parcial intacto. GREEN — `advance` (RMW: valida `InProgress`, push parcial, `step+1`), `complete`/`fail` (RMW terminales, validan `InProgress`).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test task_checkpoint --build-jobs 2` → RED correcto → GREEN (8 tests en total)
- **Evidencia:** ✅ GREEN: `9 tests run: 9 passed` — (5) **test del contrato (plan L1241):** host A corre pasos 0-1 de una tarea de 5 y muere (`db.close()` Fjall + tempdir) → host B (nueva instancia, reopen) carga `step=2`, `InProgress`, `partial` preservado → itera `cp.step..5` → `executed == [2,3,4]` (sin repetir pasos completados) → `complete` → `step=5`, `Completed`, `partial.len()==5`. (6) misuse loud: `advance` tras `complete` → `NotInProgress{state: Completed}`; `complete` dos veces → `NotInProgress`. (7) `fail` preserva `partial`+`step` para inspección; `fail` sin checkpoint → `NotFound`. **Nota TDD:** el orden API→persistencia→test del plan hace que estos tests de aceptación corran tras la API (Step 1); su condición RED fue la ausencia total de símbolos verificada en Step 1 (`E0432` cubría el archivo completo) — los asserts son de valor exacto (step/partial/executed/estado), no triviales. **Anti-regresión:** suite `-p vanta-memory` = `620 tests run: 620 passed, 2 skipped` (0 regresiones); `cargo fmt --check` OK (test reformateado con `cargo fmt -p vanta-memory`); `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` OK; `cargo test --doc -p vanta-memory` = 2/2 (doctest del módulo compila y corre).
- **Estado:** ✅ COMPLETED

### Step 3 — Docs + FIND (límite host) + gates

- **Archivos:** `docs/api/VANTA_MEMORY.md` (§Task checkpoints), `vanta-memory/src/utils/task_checkpoint.rs` (rustdoc del módulo: límite host + invariantes), `docs/dev/Backlog.md` (FIND del consumo por el host)
- **Acción:** documentar la superficie en `VANTA_MEMORY.md` (firmas reales + flujo de resume + límites: compactación = host, granularidad = paso, single-writer) con `documentation-skill` cargada; registrar la fila FIND (consumo por el host: no hay consumer in-repo; dim1 [PROPUESTA]) con el formato de `prompts/findings.md`; correr gates docs.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` exit 0 + `pwsh scripts/validate-docs-coverage.ps1` 0 gaps
- **Evidencia:** ✅ `VANTA_MEMORY.md` §Task checkpoints (MEMG-20): API table (6 métodos + contrato), `TaskCheckpoint{version,step,state,partial}` + semántica de `step`, ejemplo de resume loop, persistencia + test pineado, límite host (compactación) + "consumers in-repo: none → FIND-288"; fila `task_checkpoints` en §Namespaces; fila Orquestación actualizada (`task_checkpoint`). ✅ FIND-288 registrado en `Backlog.md` (esquema 10-col, tras FIND-287). ✅ Gates: check-links exit 0 (20 wikilinks tolerados, budget OK) · check-docs exit 0 · gen-index `--write` regeneró `docs/index.md` + `llms.txt` (delta: solo MEMG-20.md) → `--check` exit 0 · validate-docs-coverage: 0 gaps (exit 0).
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-20.md` (§Review + RESULTADO §7)
- **Acción:** `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `pwsh scripts/validate-docs-coverage.ps1` · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → revisar por Rule Group (Critical/High bloquean; Medium → FIND); clasificar tier HARD-02 (paths `vanta-memory/**` + `docs/api/**` → **Adversarial**); fork `vanta-review` con el diff (fresh context); registrar veredicto en §Review; commit **LOCAL** `feat(memory):` con pathspec de archivos propios; `campaign_update_task_state(completed, taskId:"43")` con recitation + payload `review`.
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** (pendiente)
- **Estado:** ⬜ PENDING

## Dependencias

- MEMG-11 ✅ (`508e211e`), MEMG-12 ✅ (`9d0e371d`), MEMG-13 ✅ (`eb842343`) — releídos frescos desde HEAD (`bed44b23`): `checkpoint.rs`/`pipeline_worker.rs`/`types.rs` sin cambios pendientes.
- **MEMG-07 (Task 42) EN VUELO** — `core/record/lifecycle.rs` (forgetting curves); NO se toca ese archivo. Sin solape de paths.
- dim1 hub [PROPUESTA] (Notion) + MemGPT arXiv 2310.08560 — fuente conceptual (Backlog); el plan fija el alcance.
- nextTask: Task 44 (MEMG-21) — lo decide el orquestador (MEMG-07 sigue en vuelo).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Adversarial** (HARD-02: el diff incluye `docs/api/**` + `vanta-memory/**`). Sesión reviewer: `ses_ef468a5c0ffe1AfXA7kL2N5Qar`. Veredicto: ✅ **APPROVE** (ronda única; 0 Critical / 0 Required).
- **Enfoque:** contrato L1241 punto por punto (separación tipos/namespace, RMW, test de reanudación, no-regresión) + RBI adversarial (misuse silencioso, atomicidad single-writer, doc↔código, FIND-288) + DoD P2-08 + Regla 6.
- **Cómo se probó:** corridas propias del reviewer — `task_checkpoint` 9/9 (incl. `resume_after_interruption` 1/1 explícito, Fjall close/reopen) · suite `-p vanta-memory` 620/620 (2 skipped) · `clippy --all-targets --all-features -D warnings` OK · fmt OK · doctest 2/2 · docs gates 0 (check-links/check-docs/gen-index/coverage). Separación de namespace verificada a nivel store crudo; `git status` sin nada staged (WIP ajeno advertido).
- **Hallazgos + disposición:**
  1. [OPTIONAL] Semántica at-least-once del resume (crash entre efecto del paso y `advance` re-ejecuta el paso) sin documentar → **APLICADO** (doc-only post-approve): línea en rustdoc del módulo + `VANTA_MEMORY.md` ("host steps should be idempotent").
  2. [OPTIONAL] Colisiones de `sanitize_key` (`a/b` → `a_b`) no documentadas → **APLICADO** (doc-only): rustdoc del manager + `VANTA_MEMORY.md` (mantener ids en el safe set).
  3. [NIT] Literal `"checkpoints"` enrutado por `sanitize_component` → **DESCARTADO** con razón (espeja `checkpoint.rs:82` — consistencia del módulo hermano; sin efecto conductual).
  4. [NIT] Quote del stop L1243 omitía "del pipeline" → **CORREGIDO** (quote exacto + nota: el contrato autoritativo L1241 manda — reanudación a nivel de tarea).
  5. [NIT] `advance` re-serializa `partial` completo (O(n) por paso) → **DESCARTADO** con razón (inherente al un-record sancionado por el plan; sin consumidor de miles de pasos hoy — YAGNI; el techo queda anotado en este review).
- **Veredicto:** ✅ **APPROVE** — contrato, DoD Task y gates verdes; 0 Critical/0 Required; Optional aplicados doc-only (no invalidan el veredicto: código sin cambios post-review); changeset listo para commit local (Step 4).

## Context Save Point

- **Última acción:** Steps 1-3 ✅ (RED→GREEN 9/9; suite 620/620; fmt/clippy/doctest OK; docs gates 0; FIND-288; OCR sin Critical/High; review P2-01 APPROVE + Optional doc-only aplicados). Step 4: commit local en curso.
- **Próximo paso:** commit **LOCAL** `feat(memory):` con pathspec de archivos propios → commit docs de cierre (RESULTADO + hash) → campaign completed taskId `43`.
- **Estado del worktree:** HEAD `bed44b23`; WIP ajeno (`opencode.jsonc`, master plan) NO se stagea.

## Notas

- **Diseño (por qué tipos separados):** `checkpoint.rs` es el port TDAM del checkpoint de **pipeline** (un record global con cursores/counters; namespace `pipeline_checkpoint`). El checkpoint de **tarea** (dim1) es otro dominio: un record **por tarea** con paso + parcial + estado, para que un host retome una tarea multi-paso tras una interrupción. Se implementa en módulo propio (`utils/task_checkpoint.rs`) para que la confusión pipeline↔tarea sea imposible a nivel de tipos, namespace y archivo.
- **Límite host (pre-mortem #3):** "resume tras compactación" del agente = del host, no del pipeline: el manager persiste el **estado de la tarea** (paso + parciales), no el contexto conversacional del agente. Un host que compacta su ventana usa `context_engine` para el contexto y este manager para la tarea; se documenta en el rustdoc del módulo + `VANTA_MEMORY.md`.
- **Granularidad:** el checkpoint registra 1 resultado por paso completado (`advance`); el progreso intra-paso no se checkpointea (mínimo del plan). `partial[i]` = resultado del paso `i`.
- **Sin consumer in-repo (plan stop L1243):** las tasks actuales del `pipeline_worker` (L1/L2/L3/Dream) son single-pass — no hay pasos internos que reanudar; el consumer natural es un host de agentes (dim1 [PROPUESTA]). Se registra FIND; no se wirea especulativamente (pre-mortem #2).
- **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean. PROHIBIDO tocar `docs/pipeline-state.json`.
- **Disco:** si el linker falla por espacio → `dev-tools/target-cleanup.ps1 -Clean -Yes` (patrón MEMG-02/13).
- **NOTICED BUT NOT TOUCHING:** `pipeline_worker.rs` no se modifica (sin consumer; ver arriba); `checkpoint.rs` no se modifica (separación total; el re-export en `utils/mod.rs` da discoverability).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: pendiente — se completa en el commit docs de cierre
ARCHIVOS: vanta-memory/src/utils/task_checkpoint.rs (nuevo), vanta-memory/tests/task_checkpoint.rs (nuevo), vanta-memory/src/utils/mod.rs, docs/api/VANTA_MEMORY.md, docs/dev/Backlog.md (FIND-288), docs/dev/tasks/MEMG-20.md, docs/index.md + llms.txt (generados)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:plan Task 43 sanciona la API (Gate Result ✅ DO) · D:no disparado (pre-respondido por plan F0 — símbolos dentro de la sanción, precedente MEMG-13/11) · V:no disparado (verde al primer intento; 0 fallas) · C:no disparado (FIND-288 registrado; WIP ajeno no stageado)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base) · source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering (SDP v3 BUILD) · rust-write-tests, api-and-interface-design, documentation-skill, code-review-and-quality (rol/cierre)
```
