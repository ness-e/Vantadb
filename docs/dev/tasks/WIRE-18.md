---
title: "TASK WIRE-18: T5 — Verificación adversarial del scheduler (crash mid-pass + restart)"
kind: task
description: "T5 de ADR-0054: test de crash mid-pass determinista (hook = panic del runner, no timing) + restart sobre Fjall real → DB íntegra (L0/L1 pre-crash byte-idénticos), cola efímera verificada, re-encolado desde L0 persistido sin doble procesamiento corrupto (dedup skip) + reclaim in-process del claim huérfano tras lease; review P2-01 por agente distinto"
---

# TASK WIRE-18: T5 — Verificación adversarial del scheduler (crash mid-pass + restart)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 49, bloque F0-expandido L1403-1430)
- **Fuente:** plan Task 49 + ADR-0054 T5 (`docs/dev/architecture/adr/ADR-0054-scheduler-host-vantadb-server.md:181`) + handoffs WIRE-15 (`docs/dev/tasks/WIRE-15.md:191-196`) y WIRE-16 (`docs/dev/tasks/WIRE-16.md:177-183`)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1414):** 5d → entregar crash mid-pass + restart del pass + FIND del crash del loop completo
- **Prioridad:** 🟠
- **Tipo:** Rust — **test-only** (`vanta-memory/tests/`; cero cambios de producción)
- **Turns estimados:** 6-10 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `49` en el campaign server)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY (§Spec: crash determinista por hook, backend Fjall, semántica de cola efímera ya aceptada)
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `49`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Ninguno** — archivo de test nuevo (`vanta-memory/tests/scheduler_crash.rs`); ningún símbolo de producción nuevo ni modificado; ningún otro test lo referencia. |
| Callees | `vanta_memory::services::scheduler::run_pass` (WIRE-15 :80), `vanta_memory::utils::{LocalStateBackend, FakeClock}` (claim/lease :241-318, reclaim :294), `vanta_memory::core::conversation::L0Recorder` (:179,266), `vanta_memory::core::record::read_session_records` (l1_reader.rs:29), `vanta_memory::core::state::{TaskKind, TaskPayload}`, `vantadb::{config::Config, sdk::Embedded, storage::BackendKind}` (dev-dep ya presente, `vanta-memory/Cargo.toml:43-48`). |
| Implicaciones | 0 firmas cambiadas, 0 wire, 0 migración, 0 producción tocada. El test usa Fjall + tempdir + reopen real (precedente `vanta-memory/tests/task_checkpoint.rs:165-186` "Fjall + close/reopen = real host restart"). Riesgo de flake mitigado: FakeClock determinista (sin sleeps para asserts positivos), crash hook determinista (panic, no timing), reopen con retry bounded 10s (precedente WIRE-16 `scheduler_e2e.rs:110-127`). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `7c5ff1b6`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/services/scheduler.rs` (:1-262 completo vía explore + :180-262 — `run_pass` :80, `MemoryScheduler` :138, `Drop` :171, `spawn_memory_scheduler` :198-262).
  - `vanta-memory/src/utils/local_backend.rs` (:185-234 cola/claim; :238-470 claim/complete/renew/reclaim/requeue/locks/destroy/snapshot — `queue_depth` :217, `claim_task` :241, `complete_task` :261, `claim_stale_tasks` :294, `requeue_task` :324).
  - `vanta-memory/src/services/pipeline_worker.rs` (:64-83 `WorkerConfig::default` — lock_ttl_ms 60_000; :269-508 `run_task` :272-324 claim→lock→handle→settle, `run_l1_inner` :431-489, `run_l1_batch` :497+).
  - `vanta-memory/src/core/record/l1_dedup.rs` (:1-337 — `batch_dedup` :148 "no candidates → store all", `run_l1_dedup` :223, `parse_batch_result` :199), `l1_reader.rs` (:29-75 `read_session_records`, `recall_candidates` :265), `l1_writer.rs` (grep: `apply_dedup_batch` :385, `DedupAction::Skip => Ok(None)` :191).
  - `vanta-memory/src/core/conversation/l0_recorder.rs` (:154-407 — `record_turn` :179, `read_messages` :266).
  - `vanta-memory/tests/scheduler.rs` (:1-178 completo — pass tests + FakeClock), `tests/scheduler_loop.rs` (:1-226 completo — loop tests), `tests/conversation_hook.rs` (patrón ScriptedRunner), `tests/e2e_flow.rs` (:59-171 — `run_full_pass` :144), `tests/task_checkpoint.rs` (:140-219 — precedente Fjall close/reopen).
  - `vantadb-server/tests/scheduler_e2e.rs` (:1-456 completo — restart test :254-347, `open_storage_with_retry` :110-127), `vantadb-server/src/scheduler.rs` (:1-192 completo — `wire_memory` :124).
  - `tests/durability_recovery.rs` (:1-454 completo — precedente crash/recovery: drop sin flush :50-58, WAL replay idempotente :188-227, failpoints :383-447).
  - `src/sdk/builder.rs` (:60-179 — `open_with_config` :109-128 abre engine + `ensure_indexes_current`), `src/storage/engine/init.rs` (:1-50 — `open`/`open_with_config`), `src/config.rs` (:317 default `BackendKind::Fjall`).
  - `vanta-memory/Cargo.toml` (:1-87 — dev-deps :43-48 con `vantadb` default features → fjall en tests; features :50-84).
  - Docs/specs: ADR-0054 (T5 :181; restricciones de cola :45-59), `docs/dev/tasks/FIND-113-spec.md` (:189 "tasks re-enqueue from persisted sessions"; :219 "pérdida acotada a un pass no corrido, regenerable"), plan Task 49 (L1403-1430), WIRE-15.md completo, WIRE-16.md completo, `.opencode/references/{clean-code-clean-architecture.md (Apéndice V),definition-of-done.md}`.
- **Archivos referenciados hacia dentro (imports/deps):** el test nuevo importa los símbolos listados en §Callees; ningún archivo de producción importa al test (crate de test aislado).
- **Referencias entrantes (grep HEAD):** `scheduler_crash` = **0 hits** (archivo nuevo). `run_pass` = callers producción `scheduler.rs:223` + tests `tests/scheduler.rs` (intactos — no se tocan). `read_session_records`/`L0Recorder` = consumidos por el test sin modificarlos.
- **Veredicto impacto:** **BAJO (aditivo puro, test-only)** — 1 archivo de test nuevo + task file. 0 producción, 0 firmas, 0 wire, 0 migración. No rompe contratos ni tests existentes. Pre-mortems del plan mitigados: (1) crash determinista → hook de test (panic del runner), no timing; (2) expectativa de durabilidad de cola → se verifica la semántica aceptada (efímera + re-encolable desde L0 persistido), no se promete persistencia; (3) review del mismo agente → prohibido (P2-01, agente distinto al cierre).

## Contrato

"Crash mid-pass + restart **verde y determinista**: (a) matar a mitad de pass con hook determinista (panic del runner en la sesión marcada; pre-mortem 1 del plan) → el pass muere mid-flight dejando evidencia: 1 task completada (L1 escrita) + 1 task reclamada en `pending` sin escribir; (b) reiniciar (drop sin close → reopen Fjall del mismo path) → **DB íntegra**: L0 de ambas sesiones y L1 pre-crash **byte-idénticos** (sobreviven WAL replay); **cola efímera** verificada: cola nueva `(0,0)` + `pending 0`; **re-encolable desde L0 persistido**: re-enqueue de las sesiones capturadas → el pass procesa ambas; **sin doble procesamiento corrupto**: la re-entrega del trabajo ya procesado recibe `skip` del judge de dedup → sin duplicados; segunda pasada de reconstrucción = no-op (idempotente); (c) **sin pérdida in-process**: un pass que paniquea deja su claim huérfano en `pending` y el siguiente pass lo reclama tras expirar el lease (MEM-66) y completa el trabajo. Verify: `cargo nextest run --profile audit -p vanta-memory --test scheduler_crash --build-jobs 2` verde + suite scoped del crate + `cargo fmt --check` + clippy scoped. **Review P2-01 por agente distinto (vanta-review) con veredicto; hallazgos → FINDs (no fixes silenciosos).**"

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): NO disparado** — la solución es **test-only** (cero símbolos públicos nuevos, cero cambios de producción); el contrato del plan F0 (Task 49, Gate Result ✅ DO) ya sanciona la verificación y su forma (hook determinista, cola efímera aceptada, review P2-01). Worker sin `question` (question-gates §Routing); precedente de campaña idéntico: WIRE-15/16 ("Gate D pre-respondido por el plan F0").

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Punto de crash determinista | A) **panic del runner (hook de test)** (pro: determinista, sin timing, mismo path que el crash real — el unwind mata el pass mid-flight; contra: in-process no mata el proceso) / B) kill de subproceso (contra: timing/flaky — prohibido por pre-mortem 1) / C) failpoints (contra: feature extra; el punto de crash queda en storage, no en el pass) | ✅ **A** — decidido-por-evidencia: pre-mortem 1 del plan (L1413 "punto de crash con test hook/feature de test, no timing") + precedente `durability_recovery.rs` (drop sin flush como shutdown simulado) |
| 2 | Backend del restart | A) **Fjall persistente + tempdir + reopen real** (pro: restart real, WAL replay; contra: test medium) / B) InMemory (contra: no persiste → "restart" trivial, no verifica nada) | ✅ **A** — decidido-por-evidencia: precedente exacto `task_checkpoint.rs:165-186` ("Fjall + close/reopen = a real host restart (durability)") + default del core = Fjall (`config.rs:317`) |
| 3 | Semántica de cola tras restart | A) **verificar la semántica aceptada: cola efímera (nace vacía) + re-encolado manual desde L0 persistido** (pro: fiel a ADR/FIND-113; contra: —) / B) inventar re-enqueue automático (contra: mecanismo inexistente; WIRE-16 §Spec #12 "no se inventa"; scope creep) | ✅ **A** — decidido-por-evidencia: FIND-113-spec :189/:219 + WIRE-16 §Spec #12 + contrato del plan ("cola re-encolable", no "cola durable") |
| 4 | "Sin doble procesamiento corrupto" | A) **re-entrega de trabajo ya procesado → judge de dedup `skip` → sin duplicados** (pro: ejercita el path real de dedup (recall de candidatos + decisión); contra: —) / B) solo asertar que la DB no corrompe (contra: no cubre la re-entrega, que es el riesgo real de at-least-once) | ✅ **A** — decidido-por-evidencia: contrato del plan L1412 + `batch_dedup` (l1_dedup.rs:148-193: candidatos → judge; `skip` → `Ok(None)` sin escritura, l1_writer.rs:191) |
| 5 | Ubicación y gating | A) **`vanta-memory/tests/scheduler_crash.rs` nuevo, ungated** (pro: `run_pass` es always-on sin features; aislamiento del suite WIRE-15; contra: —) / B) extender `tests/scheduler.rs` (contra: mezcla suite de pass con suite de crash; el archivo ya tiene su contrato) | ✅ **A** — decidido-por-evidencia: WIRE-15 §Spec #12 (2 files por contrato) + `run_pass` disponible sin `http-server` |
| 6 | Crash del loop completo (proceso) | A) **FIND si no se entrega** (pro: stop condition del plan L1414; in-process no se puede matar el proceso de forma fiel sin subproceso flaky; contra: —) / B) test de subproceso kill (contra: timing/flaky + infra desproporcionada) | ✅ **A** — decidido-por-evidencia: stop condition del plan ("5d → ... + FIND del crash del loop completo") + el pass es la unidad que el loop ejecuta en `spawn_blocking` (cubierto); el loop cubre su resiliencia en WIRE-15/16 |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Cero cambios de producción:** solo se agrega `vanta-memory/tests/scheduler_crash.rs` (test) + task file + docs de cierre. `run_pass`/`MemoryScheduler`/`LocalStateBackend`/`PipelineWorker` intactos.
  2. **Determinismo:** sin sleeps para asserts positivos (FakeClock + crash hook); el único sleep es el retry bounded de reopen (≤10s).
  3. **Semántica de cola:** el test verifica la semántica aceptada (efímera + re-encolable desde L0) — no promete durabilidad de cola ni inventa re-enqueue automático.
  4. **No tocar** `src/wal.rs`, `src/vector/`, `src/storage/` (dominio Arch/Engine).
  5. **WIP ajeno** (`opencode.jsonc`, master plan) NO se stagea; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  6. **Sin `unwrap`/`expect`/`unsafe` en producción** (no hay producción nueva); el test usa `#![allow(clippy::expect_used, clippy::unwrap_used)]` con invariantes documentados (convención del repo: scheduler.rs:1-2, scheduler_e2e.rs:1-2).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test scheduler_crash --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets -- -D warnings`.
- **Deuda pendiente:** ninguna del contrato. (Crash del loop completo (proceso) = FIND si no se entrega — stop condition del plan; persistencia de cola = solo por ADR, fuera de appetite.)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin producción, sin deps nuevas. El cambio **elimina** la deuda "T5 sin verificación adversarial" (ADR-0054 :181, gate de cierre de la cadena WIRE-14→16) y agrega 2 tests de regresión de crash/restart. `NOTICED BUT NOT TOUCHING`: crash del loop completo (proceso) → **FIND-293** registrada en Backlog (stop condition L1414); persistencia de cola/pending → prohibida sin ADR (FIND-113 §restart).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: crash mid-pass + restart real (Fjall) verde — DB íntegra, cola efímera, re-encolado desde L0, sin doble procesamiento (dedup skip), reclaim in-process tras lease — + suites scoped + fmt/clippy verdes + review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `test(wire):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a — test-only (`test:` → patch en release-plz; sin cambio de contrato público) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius: `run_pass`/`LocalStateBackend`/`read_session_records`/`run_l1_dedup`) + `codebase-memory-mcp_check_index_coverage` (paths clave: `no_recorded_issue` ✅) + grep puntual
- `cargo nextest` scoped por crate (`-p vanta-memory`, focused `--test scheduler_crash`) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file editado

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `test-driven-development` + `systematic-debugging` (pinned) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` (SDP phase=BUILD) + rol: `rust-write-tests` (tests de crash/restart deterministas), `documentation-skill` (task file bajo `docs/`). `security-and-hardening` excluida con justificación (ver Fase SECURITY); `performance-optimization` excluida (test-only, sin hot path — Regla 9 N/A).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo. Test-only, corre en tempdir local, sin red, sin input externo, sin secrets (los fakes son deterministas, sin API keys), sin deps nuevas. Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — no aplica: test-only; el pass corre 1 vez por intervalo fuera de hot paths; sin serialización/search/ingestión tocadas. Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — Test crash mid-pass + restart (main)

- **Archivos:** `vanta-memory/tests/scheduler_crash.rs` (nuevo)
- **Acción:** escribir el test principal `crash_mid_pass_then_restart_keeps_db_intact_and_reprocesses_from_l0` + helpers (config Fjall, reopen con retry, capture_turn, l1_task, fakes `PanicOnExtraction`/`RecoveryRunner`): Phase A crash determinista (crash-a completa, crash-b reclamada y paniquea mid-flight) → evidencia mid-pass (queue 0/0, pending 1, L1 crash-a 1, L1 crash-b 0) → drop sin close; Phase B reopen → L0/L1 byte-idénticos + cola nueva vacía + re-enqueue desde L0 → RecoveryRunner → crash-b completa, crash-a sin duplicar (skip) + segunda reconstrucción no-op.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test scheduler_crash --build-jobs 2` → GREEN + **RED-equivalente (mutación):** cambiar temporalmente el judge de `"skip"` a `"store"` → el assert anti-duplicado DEBE fallar → revertir → GREEN (evidencia de discriminación del test)
- **Evidencia:** ✅ GREEN `--test scheduler_crash` **1/1** (1.8-2.3s) · ✅ **RED-equivalente (mutación ejecutada):** `"skip"`→`"store"` en `RecoveryRunner` → `cargo nextest ... --test scheduler_crash` **FAILED** en `assertion left == right failed: re-delivery must not duplicate processed work (skip honored)` (`left: 2, right: 1`) → revertido → GREEN 1/1. El test discrimina "doble procesamiento corrupto" real (sin la mutación pasaría cualquier cosa).
- **Estado:** ✅ COMPLETED

### Step 2 — Test reclaim in-process del claim huérfano

- **Archivos:** `vanta-memory/tests/scheduler_crash.rs`
- **Acción:** escribir `panicked_pass_claim_is_reclaimed_after_lease_expiry`: crash hook → claim queda en `pending` → pass siguiente con lease vigente NO lo roba → `FakeClock::advance(60_001)` (lease `WorkerConfig::default().lock_ttl_ms` = 60_000) → el pass lo reclama, lo procesa y no pierde datos.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test scheduler_crash --build-jobs 2` → 2/2 GREEN
- **Evidencia:** ✅ `--test scheduler_crash` **2/2** (1.7s): `crash_mid_pass_then_restart_keeps_db_intact_and_reprocesses_from_l0` + `panicked_pass_claim_is_reclaimed_after_lease_expiry`. El reclaim asertó `reclaimed.processed == 1` con `worker.processed == 0` (el pass siguiente NO roba el claim con lease vigente y SÍ lo recupera tras `advance(60_001)`), y 1 record L1 sin pérdida.
- **Estado:** ✅ COMPLETED

### Step 3 — VERIFY: suites scoped + fmt + clippy

- **Archivos:** —
- **Acción:** correr el gate del contrato: `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` (suite completa default), `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` (no-regresión con feature), `cargo check -p vanta-memory` (default), `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets -- -D warnings` (+ `--features http-server`). Si algo falla → retry ladder.
- **Verify:** los comandos verdes (evidencia cruda en §Step 3)
- **Evidencia:** ✅ **6 gates verdes (ejecutados por shell, salida capturada):** (1) `cargo fmt --check` exit 0 (tras `cargo fmt -p vanta-memory` — solo mi archivo difería, verificado con `git status`); (2) `cargo clippy -p vanta-memory --all-targets -- -D warnings` exit 0; (3) `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings` exit 0; (4) `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` → **670 run: 670 passed, 2 skipped** (668+2 nuevos); (5) `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` → **678 run: 678 passed, 2 skipped** (676+2 nuevos); (6) `cargo check -p vanta-memory` (default) exit 0. ⚠️ **Anomalía de tool:** `campaign_verify_cmd` devuelve `exitCode:-1` sin stdout/stderr en 0.3s (spawn roto en este entorno; `spawnError:false`) → la verificación mecánica se ejecutó por shell con salida cruda (arriba); registrado en Notas + recitation.
- **Estado:** ✅ COMPLETED

### Step 4 — CIERRE: OCR + Review P2-01 + commit local + campaign

- **Archivos:** `docs/dev/tasks/WIRE-18.md` (§Review + RESULTADO §7)
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory; revisar por Rule Group — Critical/High bloquean; Medium → FIND) · clasificar tier HARD-02: paths `vanta-memory/tests/**` + `docs/dev/tasks/**` → **Fast** (ningún path matchea los globs adversariales) → verify fast mecánico + **veredicto registrado**; fork `vanta-review` (agente distinto, fresh context) con contrato + diff + evidencia para el veredicto P2-01 · gates docs (`check-links`/`check-docs`/`gen-index --check`) · commit **LOCAL** `test(wire):` con pathspec de archivos propios · `campaign_update_task_state(completed, taskId:"49")` con recitation + payload `review` · `skill progreso`.
- **Verify:** veredicto registrado en §Review + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Estado:** ⬜ PENDING

## Dependencias

- **WIRE-15 ✅** (`1d5e1697` + `9c881047`): `run_pass`/`MemoryScheduler`/`spawn_memory_scheduler` — releído fresco desde HEAD; el test ejercita `run_pass` (la unidad que el loop ejecuta en `spawn_blocking`).
- **WIRE-16 ✅** (`679b3545`): wiring del wrapper + e2e restart graceful — releído fresco; este test agrega el path **crash** (no-graceful) que WIRE-16 no cubre.
- **ADR-0054 T5** (gate de cierre de la cadena WIRE-14→16). **Habilita:** cierre adversarial de la cadena.
- **Coordinación:** releído fresco desde HEAD `7c5ff1b6`; cambios mínimos (1 test nuevo) + pathspec; race de staging multi-sesión → `git commit -- <paths>` SIEMPRE; conflicto real → BLOQUEO.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — P2-01, contexto fresco (no participó de la implementación); sesión reviewer `ses_ef3236a90ffeBmiBYAni4QGHmX`; review **pre-commit** (changeset sin commitear); tier **Fast** (HARD-02: `vanta-memory/tests/**` + `docs/dev/tasks/**` + `docs/dev/Backlog.md`; ningún path matchea globs adversariales) — review adversarial completa igual (el plan Task 49 exige P2-01). Veredicto: ✅ **APPROVE** (0 Critical / 0 High; 1 Medium de cierre + 2 Low + 2 NIT).
- **Enfoque:** contrato Task 49 punto por punto + RBI (vacuidad del test, determinismo, honestidad semántica de la cola efímera, simulación de crash hook vs subproceso) + tier HARD-02 + zero-producción + fidelidad de evidencia del task file + checklist anti-hábitos tóxicos §12.
- **Cómo se probó:** corridas propias del reviewer (no auto-reporte) — focused `--test scheduler_crash` **2/2 ×6 corridas verdes** (2.6-3.6s) · `nextest list` 670/678 corroborado · `cargo fmt --check` + clippy default/http-server exit 0 (suyos) · OCR reproducido (1 Rule Group `**/*.rs` → solo `scheduler_crash.rs`) · `git status`/`git diff` (0 producción, 0 staged) · lectura de paths (`run_pass`, `local_backend`, `pipeline_worker`, `l1_dedup`/`l1_writer`, `l1_reader`, plan Task 49, ADR-0054, precedentes). Caveat honesto: no re-ejecutó la suite completa 670/678 ni la mutación (tamper del árbol revisado) — verificó la mutación por lectura de código (`DedupAction::Skip => Ok(None)` l1_writer.rs:191; judge solo con candidatos l1_dedup.rs:160-168).
- **Hallazgos + disposición:**
  1. [Medium — cierre] FIND del crash del loop completo no registrada (el task file §Spec #6 la decidía "FIND si no se entrega"; el plan L1414 la nombra en stop conditions) → **APLICADO**: fila **FIND-293** creada en `docs/dev/Backlog.md` (🟢 Baja; acción: test de resiliencia del loop / reclaim en tick posterior).
  2. [Low-1] "byte-idénticos" más fuerte que el assert (comparaba solo `.content`; `read_namespace_records` salta payloads undeserializables) → **APLICADO**: comparación **full-record** (`Vec<MemoryRecord>`, `PartialEq`) — un payload corrupto post-restart ahora rompe el assert por count/contenido; re-verificado (focused 2/2 + fmt exit 0).
  3. [Low-2 — coordinación] WIP ajeno incompleto en la enumeración (Backlog.md con FIND-292 de WIRE-17, que commiteó mid-review) → **APLICADO**: releído fresco (`git status`: solo master plan + `opencode.jsonc` ajenos), commit con pathspec estricto + `git diff --cached` verificado.
  4. [NIT-1] contador stale "% completado 0%" → **APLICADO**: contadores actualizados al cierre.
  5. [NIT-2] prefijo de commit `test(wire):` (orquestador) vs `test(memory):` (plan DoD) → **RESUELTO**: se sigue la instrucción directa del orquestador `test(wire):` con constancia en §Notas (precedente WIRE-15 NIT-3).
  6. [Optional] ventana "L1 escrito pero claim no completado" no forzada → **DISPENSA**: cubierta indirectamente por la re-entrega + dedup skip; mismo eje que FIND-293.
- **Veredicto:** ✅ **APPROVE** — contrato verificado con corridas propias; 0 Critical/High; Medium/Lows/NITs aplicados o dispensados con motivo; fixes post-review re-verificados (focused 2/2 + fmt); changeset listo para commit local con pathspec (sin push).

## Notas

- **Crash determinista (incógnita resuelta):** el hook es el panic del runner (`PanicOnExtraction(marker)`) — el unwind mata el pass mid-flight en un punto exacto y reproducible (pre-mortem 1 del plan). No hay timing ni sleeps.
- **Drop sin close = shutdown simulado:** la fase A no llama `close()`; el reopen recupera vía WAL/SST (precedente `durability_recovery.rs:50-58`). Honestidad: in-process no se puede matar el proceso de forma fiel sin subproceso flaky — el hook es el mecanismo sancionado por el plan.
- **Cola efímera (semántica aceptada):** FIND-113 :219 ("pérdida acotada a un pass pendiente no corrido, regenerable") + :189 ("tasks re-enqueue from persisted sessions"); el test demuestra el path de reconstrucción desde L0 — no se inventa mecanismo (WIRE-16 §Spec #12).
- **Low-1 del review APLICADO:** la integridad post-restart compara `Vec<MemoryRecord>` full (`PartialEq`), no solo `.content` — un payload corrupto que `read_namespace_records` saltaría silenciosamente rompe el assert por count/contenido.
- **NIT-2 del review (prefijo de commit):** instrucción directa del orquestador = `test(wire):`; el DoD del plan decía `test(memory):`. Se sigue la instrucción directa manteniendo `WIRE-18` en el subject (precedente WIRE-15 §Notas).
- **FIND-293** (crash del loop completo) registrada en `docs/dev/Backlog.md` — Medium de cierre del review P2-01.
- **Anomalía de tool:** `campaign_verify_cmd` (MCP) devuelve `exitCode:-1` sin stdout/stderr en 0.3s (spawn roto en este entorno; `spawnError:false`) — la verificación mecánica corrió por shell con salida cruda (Steps 1-3) y el reviewer la reprodujo por shell. Para el orquestador: revisar el spawn de `campaign_verify_cmd` (no es falla de código; no cuenta como retry).
- **NOTICED BUT NOT TOUCHING:** crash del loop completo (proceso) → FIND-293; persistencia de cola/pending → prohibida sin ADR (FIND-113 §restart).
- **Coordinación:** WIP ajeno (`opencode.jsonc`, master plan) intacto; commit con pathspec (verificado `git diff --cached` pre-commit).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — resueltas en DISCOVERY (§Spec #1-#6) |
| Pendientes de ejecución (downhill) | **0** steps (1-4 ejecutados — commit + campaign al cierre) |
| % completado | 100% (steps 1-4 ejecutados y verificados; hash + campaign al cierre) |

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 95cdb2ce (test local, sin push; + commit docs de cierre)
ARCHIVOS: vanta-memory/tests/scheduler_crash.rs (nuevo), docs/dev/tasks/WIRE-18.md (nuevo), docs/dev/Backlog.md (FIND-293)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:no disparado (plan Task 49 ya clasificado; test-only) · D:no disparado (cero símbolos públicos nuevos — test-only; contrato F0 sanciona) · V:no disparado (verde al primer intento; única anomalía de tool `campaign_verify_cmd` → shell) · C:no disparado (sin colaterales bloqueantes; FIND-293 = hallazgo de review registrado, no colateral)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base auto) · test-driven-development, systematic-debugging (pinned) · source-driven-development, doubt-driven-development, incremental-implementation (SDP v3 BUILD) · rust-write-tests, documentation-skill (rol/cierre)
```
