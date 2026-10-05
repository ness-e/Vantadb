---
title: "TASK WIRE-15: Servicio scheduler en vanta-memory (run_pass + loop con shutdown)"
kind: task
description: "T2 de la cadena WIRE-14→15→16 (ADR-0054): pass pull-based (`run_pass` = timers + worker + reclaim) + loop helper feature-gated `http-server` con shutdown graceful (espejo MemoryTtlSweeper, Drop best-effort por handoff WIRE-14) e impl `BackgroundService`; runner por pass vía factory (patrón FIND-112); sin runner → skip observable (tareas quedan encoladas, P4)"
---

# TASK WIRE-15: Servicio scheduler en vanta-memory (run_pass + loop con shutdown)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 46, F1 — Distribución; bloque F0-expandido L1317-1344)
- **Fuente:** plan Task 46 (L1317-1344) + ADR-0054 T2 (`docs/dev/architecture/adr/ADR-0054-scheduler-host-vantadb-server.md:178`) + handoff WIRE-14 (`docs/dev/tasks/WIRE-14.md:185`)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1328):** 5d → entregar `run_pass` + test; FIND del loop helper si no cierra
- **Prioridad:** 🟠
- **Tipo:** Rust (`vanta-memory`; `services/scheduler.rs` nuevo + feature `http-server`)
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `46` en el campaign server)
- **Incógnitas (uphill):** 0 — la única abierta ("forma exacta del loop helper / inyectabilidad del runner") quedó resuelta en DISCOVERY por evidencia (§Spec #2/#3: factory genérica `Fn() -> Option<R>` por pass — sin ella el path de procesamiento del loop es intesteable y WIRE-16 no puede inyectar un fake en su e2e; es el mismo shape de "runner por pass" del contrato).
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `46`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Símbolos nuevos (aditivos — sin callers preexistentes):** `services::scheduler::{run_pass, PassStats, MemoryScheduler, spawn_memory_scheduler}` (consumidores: tests WIRE-15 + WIRE-16 futuro). **Callers de lo existente que NO cambia:** `run_bridge_pass`/`HttpCaptureBridge` (tests conversation_hook), `PipelineWorker::{run_once, reclaim_stale}` (conversation_hook + tests), `TimerScanner::run_once` (tests pipeline_manager), `MemoryTaskHandler` (e2e/tests) — todos intactos. |
| Callees | `TimerScanner::run_once` + `PipelineWorker::{run_once, reclaim_stale}` + `MemoryTaskHandler` + `build_ingest_runner` (FIND-112) + `vantadb::cli_server::BackgroundService` (WIRE-14). **Dependencia nueva:** `tokio` opcional (rt/time/sync/macros) activada SOLO por `http-server` — el default pull-based del crate queda tokio-free (pre-mortem 1 del plan). |
| Implicaciones | **API pública aditiva** de `vanta-memory`: `run_pass` always-on (sin tokio); loop helper feature-gated `http-server` (precedente `conversation_hook`). Sin breaking: `LocalStateBackend`/`TimerScanner`/`PipelineWorker`/`run_bridge_pass` sin cambios, sin wire/persistencia, sin migración. Regla 8: el backend es un `Mutex` único; el pass corre síncrono dentro de `spawn_blocking` (sin guard a través de `.await`; lock order intacto). Tests existentes re-corridos; ningún test toca los símbolos nuevos. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `7b2d30d5`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/services/mod.rs` (:1-11 completo — pipeline_worker + conversation_hook; el módulo `scheduler` entra acá).
  - `vanta-memory/Cargo.toml` (:1-80 completo — features; `http-server = ["vantadb/server"]:72`; sin tokio hoy).
  - `vanta-memory/src/services/pipeline_worker.rs` (:1-85 configs, :90-209 telemetría/`PipelineWorker`, :232-267 `run_once`/`reclaim_stale`, :272-335 `run_task`, :340-459 `MemoryTaskHandler` + L1, :460-579 L1 batch/Dream/L2, :600-679 L2/L3, :795-854 dispatch `handle`).
  - `vanta-memory/src/services/conversation_hook.rs` (:1-107 completo — `HttpCaptureBridge` + `run_bridge_pass`; precedente directo del pass).
  - `vanta-memory/src/utils/timer_scanner.rs` (:1-38 completo — `run_once(handler: impl FnMut(&TimerEntry)) -> usize`).
  - `vanta-memory/src/utils/local_backend.rs` (:1-100 + :228-347 — Mutex único, `claim_task`/`claim_stale_tasks`/`requeue_task`, clock inyectado; ceiling multi-proceso documentado `:236-237`).
  - `vanta-memory/src/utils/pipeline_manager.rs` (:1-147 — `l1_idle_member`, `l1_task` L1 priority 1, warm-up) + `stateful_pipeline_manager.rs` (:1-130).
  - `vanta-memory/src/utils/managed_timer.rs` (:1-60 — `Clock`/`SystemClock`/`FakeClock`).
  - `vanta-memory/src/utils/pipeline_factory.rs` (:1-53 completo — trío backend+manager+worker; sin pass).
  - `vanta-memory/src/ingest/runner_config.rs` (:1-70 + :114-258 — `IngestRunnerCfg` + `build_ingest_runner -> Option<ConcreteRunner>`; `ConcreteRunner::None` degrada at-run, `None` del builder = kill-switch futuro).
  - `src/gc.rs` (:90-210 — `MemoryTtlSweeper`: watch + JoinHandle + `shutdown()` + `Drop` send/abort + `impl BackgroundService` + `spawn_memory_ttl_sweeper`; **espejo exacto**).
  - `src/server/state.rs` (:60-179 — trait `BackgroundService: Send` (`shutdown(self: Box<Self>) -> Pin<Box<dyn Future<Output=()>+Send>>`) + `ServerHooks`; doc de parada graceful).
  - `src/server/mod.rs` (:1-59 — re-exports `BackgroundService`/`ServerHooks`/`run_with_hooks` vía `crate::server`; `cli_server` = `pub use crate::server::*`).
  - `src/server/bootstrap.rs` (:280-419 — `run`/`run_with_hooks`, spawn sweeper `:375-388`, join `:399-402`, `shutdown_background_services` `:408-412`; early-return de `serve_http_or_tls` NO joinea → Drop obligatorio).
  - Docs/specs: ADR-0054 (completo), plan Task 45/46/47 (L1289-1373), `docs/dev/tasks/WIRE-14.md` (completo — handoff + Low-1/Low-2), `docs/dev/tasks/FIND-112.md` (completo — patrón runner), `docs/dev/tasks/FIND-113-spec.md` (:55-174 — semántica pull-based, sin daemon), `.opencode/rules/{concurrency-async,api-contract}.md`, `.opencode/references/{clean-code-clean-architecture.md (Apéndice V),definition-of-done.md}`, `docs/api/VANTA_MEMORY.md` (wiring status — se mantiene verdadero: docs = WIRE-17).
- **Archivos referenciados hacia dentro (imports/deps):** `crate::utils::{local_backend::LocalStateBackend, managed_timer::{Clock, SystemClock}, timer_scanner::TimerScanner}` (pipeline_worker.rs:42-44, timer_scanner.rs:10-11), `crate::services::pipeline_worker::{MemoryTaskHandler, PipelineWorker, RunStats, WorkerConfig}` (conversation_hook.rs:26), `crate::core::record::{L1DedupConfig, L1ExtractorConfig}` (pipeline_worker.rs:35-38), `crate::ingest::runner_config::build_ingest_runner` (runner_config.rs:110), `vantadb::cli_server::BackgroundService` (gc.rs:122 vía `crate::server::state`), `tokio::sync::watch`/`tokio::task::{spawn, spawn_blocking, JoinHandle}`/`tokio::time::interval` (gc.rs:102-103,164-167 — patrón).
- **Referencias entrantes (grep HEAD):** `services::scheduler` = **0 hits** (símbolos nuevos, verificado 2026-10-05). `run_bridge_pass` = 1 caller de test (`tests/conversation_hook.rs:16,85,107,117`); `PipelineWorker` = callers `conversation_hook.rs` + `pipeline_factory.rs:47` + tests; `TimerScanner` = re-export `utils/mod.rs:30` + tests; feature `http-server` = consumidor único `conversation_hook` (Cargo.toml:72) — ningún crate externo la activa aún (desktop/python default-features=false; proxy/mcp solo `llm-driver`); WIRE-16 la activará vía `vantadb-server`. `BackgroundService` = impl única `MemoryTtlSweeper` (gc.rs:122) + tests bootstrap.
- **Veredicto impacto:** **BAJO (aditivo puro)** — 3 archivos tocados (1 nuevo `scheduler.rs`, `services/mod.rs` +1 línea, `Cargo.toml` +dep opcional/+feature) + 2 archivos de test nuevos. 0 firmas cambiadas, 0 campos de struct existente, 0 wire, 0 migración. Pre-mortems mitigados: (1) Tokio/feature → loop gateado bajo `http-server` (default intacto, verificable con `cargo check -p vanta-memory` sin features); (2) doble dueño de cola → regla writer del ADR documentada en rustdoc (un solo loop por proceso; WIRE-16 la ejecuta); (3) secrets del runner → WIRE-15 NO lee env/secrets: la factory la provee el host (WIRE-16, R-5).

## Contrato

"`run_pass` (timers + worker + reclaim, pull-based, sin Tokio) + loop helper con shutdown graceful (espejo `MemoryTtlSweeper`), feature-gated `http-server` (plan L1326): (a) **`run_pass(queue, db, runner) -> PassStats`** — `TimerScanner::run_once` (miembros `l1_idle:<session>` → encola `TaskKind::L1` priority 1; miembros desconocidos → skip con debug) + `PipelineWorker::run_once` + `reclaim_stale` (lease = `WorkerConfig::default().lock_ttl_ms`, limit = `batch_size`), todo reusando `MemoryTaskHandler` con `L1ExtractorConfig/L1DedupConfig::default()` y cadencia L3 = const 50 (espejo e2e full pass); (b) **`spawn_memory_scheduler(queue, db, interval, runner_factory)`** con factory `Fn() -> Option<R>` llamada por pass (patrón FIND-112) — `None` → skip observable (debug), tareas quedan encoladas, nunca bloquea (P4); (c) **`MemoryScheduler`** = watch + JoinHandle con `shutdown()` graceful (join), `Drop` best-effort (send + abort — handoff WIRE-14 Low-1: el early-return de `serve_http_or_tls` no invoca shutdown) e `impl vantadb::cli_server::BackgroundService` (WIRE-16 lo empuja a `ServerHooks::background_services`); (d) **tests verdes** feature-gated: `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` (pass procesa, timers disparan, reclaim, shutdown joinea, sin runner degrada) + `cargo check -p vanta-memory` (default, sin Tokio) + `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings`."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato del plan (Task 46, Gate Result ✅ DO) fija: `run_pass` (timers + worker + reclaim) + loop helper espejo `MemoryTtlSweeper` + feature-gated + runner por pass (FIND-112) + sin runner → skip observable. La forma exacta del helper y la inyectabilidad del runner se resolvieron por evidencia (ADR-0054 T2 + precedente conversation_hook + espejo gc.rs + requisito de testabilidad TDD); los símbolos nuevos SON el mecanismo sancionado. Worker sin `question` (question-gates §Routing): default recomendado aplicado y documentado; el orquestador puede ajustar la superficie en review. Precedente de campaña idéntico: WIRE-14 ("Gate D: pre-respondido por el plan F0").

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Ubicación del loop helper | A) **`vanta-memory` feature-gated** (pro: ADR T2 lo asigna a este crate; reuso directo de worker/timers; contra: introduce Tokio opcional) / B) solo host (`vantadb-server`) (contra: contradice plan L1322/L1326 y duplicaría el pass en cada embedder) | ✅ **A** — decidido-por-evidencia: plan L1326 "loop helper ... feature-gated" + ADR T2 fila 178 (verificación `cargo test -p vanta-memory --features http-server`) |
| 2 | Inyección del runner en el loop | A) **factory genérica `Fn() -> Option<R>` por pass** (pro: implementa "runner por pass" + permite fake en tests y en el e2e de WIRE-16; contra: 1 type param extra) / B) `Option<IngestRunnerCfg>` + `build_ingest_runner` fijo (pro: menos genéricos; contra: **el path de procesamiento del loop queda intesteable** — `ConcreteRunner` no inyectable, sin red/llm-driver en CI; WIRE-16 tampoco puede fakes su e2e) / C) runner construido una vez (contra: contradice "runner por pass") | ✅ **A** — decidido-por-evidencia: contrato "runner por pass (patrón FIND-112)" + TDD (loop procesando debe ser testeable) + precedente `run_bridge_pass` (runner inyectado por el caller) |
| 3 | Semántica "sin runner" | A) **skip observable (debug) + tareas quedan encoladas** (pro: P4 — nada se pierde, nada bloquea; contra: nada se procesa) / B) correr igual (contra: sin engine LLM cada L1 falla → retry ×3 → dead-letter: quema tareas) | ✅ **A** — decidido-por-evidencia: plan L1326 "sin runner → degrada P4 (skip observable, nunca bloquea)"; el runner degradado (`ConcreteRunner::None`) sigue su path P4 existente al correr (FIND-112 runner_config.rs:107-109), pero "sin runner" = factory `None` → skip |
| 4 | Feature gating | A) **`http-server` + `dep:tokio`** (pro: default build tokio-free intacto — pre-mortem 1; contra: 1 feature más gorda) / B) Tokio no-opcional (contra: grava a desktop/python/mcp que no lo usan) | ✅ **A** — decidido-por-evidencia: pre-mortem 1 del plan + precedente `conversation_hook` (mismo feature) |
| 5 | Features de Tokio | A) **`rt, time, sync, macros`** (pro: mínimo para spawn/spawn_blocking/interval/watch/`#[tokio::test]`; contra: —) / B) reusar las del core (contra: features son por-crate, no se heredan) | ✅ **A** — decidido-por-evidencia: usos concretos (gc.rs:164-167 espejo) + tests async |
| 6 | Nombres | A) **`run_pass` / `PassStats` / `MemoryScheduler` / `spawn_memory_scheduler`** (pro: plan dice `run_pass`; espejo `MemoryTtlSweeper`/`spawn_memory_ttl_sweeper`; sin stuttering) / B) `scheduler_run_once`/`SchedulerHandle` (contra: diverge del plan) | ✅ **A** — decidido-por-evidencia: plan L1326 + gc.rs:101,159 |
| 7 | Dispatch de timers | A) **`l1_idle:<session>` → `TaskKind::L1` priority 1** (pro: único miembro productivo hoy (`pipeline_manager.rs:79`); precedente exacto de test (`tests/pipeline_manager.rs:215-223`); contra: —) / B) inventar `l2:`/`dream:` (contra: sin productor — YAGNI) | ✅ **A** — decidido-por-evidencia: grep HEAD `set_timer` productivo = solo `l1_idle_member`; miembros desconocidos → debug skip (forward-compatible) |
| 8 | Cadencia L3 | A) **const 50** (pro: espejo del full pass e2e (`tests/e2e_flow.rs:150`); YAGNI; contra: no configurable) / B) parámetro en `run_pass` (contra: superficie sin consumidor) | ✅ **A** — decidido-por-evidencia: `evaluate_persona_trigger(input, 50)` e2e; hosts con otra cadencia arman su propio handler (patrón documentado en conversation_hook.rs:90-92) |
| 9 | Punto de parada | A) **`impl BackgroundService` → join post-run loop (WIRE-14)** (pro: seam existente; el host solo lo empuja; contra: —) / B) shutdown propio del host (contra: duplicaría el mecanismo) | ✅ **A** — decidido-por-evidencia: WIRE-14 handoff (`WIRE-14.md:185`) + plan T3 (WIRE-16 empuja a `ServerHooks`) |
| 10 | Drop | A) **best-effort send + abort** (pro: cubre el early-return de `serve_http_or_tls` que NO joinea — handoff Low-1; espejo `gc.rs:135-145`; contra: —) / B) solo `shutdown()` (contra: servicio colgado en early-exit — prohibido por handoff) | ✅ **A** — decidido-por-evidencia: handoff WIRE-14 Low-1 + review P2-01 WIRE-14 |
| 11 | Config del worker | A) **`WorkerConfig::default()` + configs default del handler** (pro: cero config nueva; WIRE-16 tunable vía env si un slice futuro lo exige — FIND-113-spec:229; contra: —) / B) struct de config nueva (contra: YAGNI) | ✅ **A** — decidido-por-evidencia: `conversation_hook.rs:98` + e2e defaults |
| 12 | Layout de tests | A) **2 files: `tests/scheduler.rs` (pass, ungated) + `tests/scheduler_loop.rs` (loop, `#![cfg(feature = "http-server")]`)** (pro: `run_pass` cubierto también en CI default; contra: —) / B) todo gated (contra: `run_pass` sin cobertura en default) | ✅ **A** — decidido-por-evidencia: `run_pass` es always-on; el loop es el único que exige el feature |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Nada existente cambia:** `run_bridge_pass`/`HttpCaptureBridge`, `PipelineWorker`, `TimerScanner`, `LocalStateBackend`, `MemoryTaskHandler` sin ediciones; sus tests intactos.
  2. **Default build tokio-free:** `cargo check -p vanta-memory` (sin features) verde; Tokio entra SOLO con `http-server`.
  3. **P4 (nunca bloquea, nunca pierde):** sin runner → skip observable y tareas quedan encoladas; errores del pass → warn + siguiente tick (nunca panic/hang).
  4. **Parada graceful:** `shutdown()` joinea (watch + await); `Drop` best-effort (send + abort) — ningún task sobrevive al handle en el path de early-exit (handoff WIRE-14 Low-1); el servicio se joinea por `BackgroundService` (espejo sweeper).
  5. **Single-writer (ADR-0054):** un solo loop por DB/proceso (writer); documentado en rustdoc — WIRE-16 lo cablea en el writer.
  6. **Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción**; sin dependencias nuevas fuera de `tokio` opcional; `src/wal.rs`, `src/vector/`, `src/storage/` **no se tocan**.
  7. **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  8. **WIRE-16 depende de la superficie:** `run_pass`/`PassStats`/`MemoryScheduler`/`spawn_memory_scheduler` quedan públicos, documentados y estables; el factory shape `Fn() -> Option<R>` es el contrato para el wiring.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` · `cargo check -p vanta-memory` (default) · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings`.
- **Deuda pendiente:** ninguna del servicio. (El wiring productivo = WIRE-16; docs = WIRE-17; la cadencia L3 configurable y los timers L2/Dream quedan sin productor por diseño — YAGNI, se agregan con su productor.)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin clones en hot path (el pass corre 1 vez por intervalo; el loop clona Arc/handles O(1)), sin abstracciones especulativas (1 fn de pass + 1 struct de loop + 1 factory param). El cambio **elimina** la deuda de "loop helper inexistente" (bloqueante de ADR-0054 T2 y de WIRE-16) y agrega los tests del lifecycle (pass + loop + skip + seam). 1 dep nueva **opcional** (tokio, ya en el lockfile del workspace vía core/server — cero peso nuevo en el árbol). `NOTICED BUT NOT TOUCHING`: `run_bridge_pass` no se refactoriza sobre `run_pass` (tiene semántica distinta — `usize::MAX` persona; WIRE-16 decidirá si converge); `docs/api/**` → WIRE-17.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: `run_pass` (timers + worker + reclaim) + loop helper feature-gated con shutdown/Drop/`BackgroundService` + factory por pass + skip sin runner + tests (pass procesa, timers disparan, reclaim, shutdown joinea, sin runner degrada, seam) + suite scoped `--features http-server` + `cargo check -p vanta-memory` (sin features) + fmt/clippy verdes |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius: `PipelineWorker`/`TimerScanner`/`run_bridge_pass`/`MemoryTtlSweeper`) + `codebase-memory-mcp_check_index_coverage` (10 paths, `no_recorded_issue` ✅) + grep puntual
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file editado

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (SDP phase=BUILD) + rol: `rust-write-tests` (tests del lifecycle pass/loop), `api-and-interface-design` (superficie pública nueva), `documentation-skill` (task file bajo `docs/`). `security-and-hardening` excluida (sin trust boundary nuevo: el loop no lee input externo ni secrets — la factory la provee el host; WIRE-16 ejecuta R-5). `performance-optimization` excluida (no hot path — pass periódico O(queue), `spawn_blocking` por diseño; Regla 9 N/A).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo. El loop no acepta input de red/usuario ni lee env/secrets (la factory es del host; WIRE-16 ejecuta R-5); sin auth/FFI/deps de red nuevas (tokio es runtime, ya presente en el workspace). Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — no aplica: pass periódico O(queue) fuera de hot paths; trabajo del motor en `spawn_blocking` (Regla 1 concurrency-async); sin serialización/search/ingestión tocadas. Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — RED: tests del pass y del loop (2 files nuevos)

- **Archivos:** `vanta-memory/tests/scheduler.rs` (nuevo, ungated), `vanta-memory/tests/scheduler_loop.rs` (nuevo, `#![cfg(feature = "http-server")]`)
- **Acción:** RED — escribir los tests que fallan por símbolos ausentes:
  - `tests/scheduler.rs` (pass, FakeClock determinista): `pass_processes_queued_l1_task_and_writes_memories`; `pass_dispatches_expired_idle_timer_into_l1_task` (timer `l1_idle:` → L1, dispara 1 vez y se consume); `pass_skips_unknown_timer_members_without_enqueuing`; `pass_reclaims_stale_task_after_lease_expiry` (claim con lease 1ms → avanzar clock → reclaim procesa).
  - `tests/scheduler_loop.rs` (loop, SystemClock + interval corto, first-tick inmediato): `scheduler_loop_processes_queued_tasks_via_runner_factory` (+ shutdown con timeout); `scheduler_loop_without_runner_skips_pass_and_keeps_tasks_queued` (factory contadora → `None`; queue intacta, db vacía); `scheduler_shutdown_joins_loop_and_stops_processing` (contador de passes estable post-shutdown); `scheduler_shutdowns_through_background_service_seam` (`Box<dyn BackgroundService>`).
  - Fake runner local (patrón `ScriptedRunner` de `tests/conversation_hook.rs`); helper `wait_until` con deadline (sin sleeps sueltos para asserts positivos).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` → RED correcto (compile error E0432/E0433 por `services::scheduler` ausente, no assertion)
- **Evidencia:** ✅ RED verificado (exit 101): `error[E0432]: unresolved import vanta_memory::services::scheduler` (`tests/scheduler.rs:14`) — log `$env:TEMP\wire15-red.log`. El target `scheduler_loop` también falló en la corrida de consola (12 errores: `E0432` del módulo + `E0433` de `tokio` no vinculado — dep aún ausente); el log citado solo captura el target `scheduler` (nextest corta en el primer target rojo) — [Low-1 del review, disposición: redacción ajustada]. Falla por la razón correcta (símbolos/dep ausentes, no assertion).
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: módulo scheduler + feature/dep (Cargo.toml + mod.rs + scheduler.rs)

- **Archivos:** `vanta-memory/Cargo.toml` (dep `tokio` opcional + feature `http-server` ampliada), `vanta-memory/src/services/mod.rs` (+`pub mod scheduler;`), `vanta-memory/src/services/scheduler.rs` (nuevo)
- **Acción:** GREEN — `Cargo.toml`: `tokio = { version = "1", features = ["rt", "time", "sync", "macros"], optional = true }` + `http-server = ["vantadb/server", "dep:tokio"]` (comentario actualizado). `mod.rs`: registrar `scheduler` con doc. `scheduler.rs`: (a) `PassStats` + `run_pass` (timer dispatch `l1_idle:`→L1 + `PipelineWorker::run_once` + `reclaim_stale` con `WorkerConfig::default()`, handler con configs default y const persona 50); (b) feature-gated: `MemoryScheduler` (watch + JoinHandle + `shutdown()` + `Drop` send/abort + `impl BackgroundService` con UFCS) + `spawn_memory_scheduler(queue, db, interval, runner_factory)` (interval `MissedTickBehavior::Skip`, first-tick inmediato; factory por pass en `spawn_blocking`; `None` → debug skip; stats → info/debug; join error → warn). Rustdoc completo (contrato, single-writer, Drop best-effort, skip P4).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` → GREEN (tests nuevos + suite existente del crate con feature) + `cargo check -p vanta-memory` (default, sin features: no-regresión sin Tokio)
- **Evidencia:** ✅ GREEN: focused `--test scheduler` 4/4 + `--test scheduler_loop` 4/4 (tras anotar los closures `|| -> Option<ScriptedRunner>` — E0283 de inferencia en el factory `None`) · ✅ suite scoped `--features http-server`: **676 tests run: 676 passed, 2 skipped** · ⚠️→✅ `cargo check -p vanta-memory` (default) detectó el `#[cfg(feature = "http-server")]` faltante en `spawn_memory_scheduler` (10 errores E0422/E0425/E0433) → corregido → exit 0 (gate del contrato funcionando: default build sin Tokio).
- **Estado:** ✅ COMPLETED

### Step 3 — VERIFY: suite scoped + fmt + clippy

- **Archivos:** —
- **Acción:** correr el gate del contrato: `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` (suite completa del crate con feature), `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` (default, no-regresión), `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings`. Si algo falla → retry ladder.
- **Verify:** los 4 comandos verdes (evidencia cruda en §Step 3)
- **Evidencia:** ✅ `cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2` → **676 run: 676 passed, 2 skipped** (skips pre-existentes) · ✅ `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` (default) → **668 run: 668 passed, 2 skipped** · ✅ `cargo fmt --check` exit 0 (tras `cargo fmt -p vanta-memory` — solo mis 2 test files diferían; verificado con `cargo fmt --check` workspace: únicos diffs) · ✅ `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings` exit 0 · ✅ `cargo clippy -p vanta-memory --all-targets -- -D warnings` (default) exit 0 · ✅ focused re-run post-fmt 8/8.
- **Estado:** ✅ COMPLETED

### Step 4 — CIERRE: OCR + Review P2-01 + commit local + campaign

- **Archivos:** `docs/dev/tasks/WIRE-15.md` (§Review + RESULTADO §7)
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory; revisar por Rule Group — Critical/High bloquean; Medium → FIND) · clasificar tier HARD-02: paths `vanta-memory/src/services/**` + `vanta-memory/Cargo.toml` + tests + `docs/dev/tasks/**` → **Fast** (ninguno matchea los globs adversariales) → verify fast mecánico + **veredicto registrado**; fork `vanta-review` (agente distinto, fresh context) con contrato + diff + evidencia para el veredicto P2-01 · gates docs (`check-links`/`check-docs`/`gen-index --check`) · commit **LOCAL** `feat(memory):` con pathspec de archivos propios · `campaign_update_task_state(completed, taskId:"46")` con recitation + payload `review` · `skill progreso`.
- **Verify:** veredicto registrado en §Review + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** ✅ OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json` → spec 2 Rule Groups: 1 `vanta-memory/Cargo.toml` (manifest hygiene) / 2 los 4 `.rs` (ownership, error handling, unsafe, concurrencia, API); pasada cognitiva: 0 Critical / 0 High / 0 Medium — sin `unwrap`/`unsafe`/lock-a-traverso-de-await/bloqueo en async; WIP ajeno fuera del alcance) · ✅ review P2-01 `vanta-review` **APPROVE** (sesión `ses_ef3993124ffeeYg54pY1Q21uYC`; 0C/0H/0M; Lows/NITs aplicados o dispuestos) · ✅ gates docs (`check-links` exit 0 · `check-docs` exit 0 · `gen-index --check` exit 0 tras `--write` de `docs/index.md`+`llms.txt` por el task file nuevo) · ✅ tier **Fast** (HARD-02) + review adversarial completa pedida por el orquestador · ✅ commit **LOCAL** `feat(wire):` con pathspec (pendiente de hash al cierre).
- **Estado:** ⏳ IN PROGRESS (commit local en curso)

## Dependencias

- **WIRE-14 ✅** (`b5d294d2` + `4448788b`): seam `ServerHooks`/`BackgroundService`/`run_with_hooks` + facades `vantadb::cli_server` — releído fresco desde HEAD; handoff Low-1 (Drop best-effort) incorporado al diseño.
- **Habilita:** WIRE-16 (Task 47, wiring del wrapper `vantadb-server` — consume `spawn_memory_scheduler` + `BackgroundService`), WIRE-17 (docs, post WIRE-16). ADR-0054 T2.
- **Coordinación:** releído fresco desde HEAD `7b2d30d5`; cambios mínimos + pathspec; race de staging multi-sesión → `git commit -- <paths>` SIEMPRE; conflicto real → BLOQUEO.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (HARD-02: `vanta-memory/src/services/**` + `vanta-memory/Cargo.toml` + tests + `docs/dev/tasks/**` + `docs/index.md`/`llms.txt` generados + `Cargo.lock`; ningún path matchea globs adversariales). Sesión reviewer: `ses_ef3993124ffeeYg54pY1Q21uYC`. Review **pre-commit** (changeset sin commitear), adversarial completa pedida por el orquestador. Veredicto: ✅ **APPROVE** (0 Critical / 0 High / 0 Medium).
- **Enfoque:** five-axis + RBI sobre el diff completo; contrato punto por punto (7/7) con comandos propios del reviewer (no auto-reporte); seam/mirror (Drop best-effort vs early-exit de `serve_http_or_tls`; abort no cancela spawn_blocking en vuelo — best-effort aceptado y documentado); factory por pass vs cacheada; skip P4; concurrencia (sin lock síncrono cruzando await); flake risk (wait_until bounded + dos snapshots post-shutdown); WIP ajeno; OCR rule groups verificados independientemente.
- **Cómo se probó:** corridas propias del reviewer — scoped `--features http-server` **676 passed / 2 skipped** · default **668 passed / 2 skipped** · `cargo check -p vanta-memory` exit 0 · `cargo clippy -p vanta-memory --all-targets --features http-server -- -D warnings` exit 0 (+ default 0) · `cargo fmt --check` exit 0 · focused 8/8 · `cargo doc --all-features` 0 warnings del scheduler · RED log verificado contra el árbol (E0432 real) · `git status --porcelain` sin staging · `cargo tree` (tokio transitivo pre-existente por reqwest — NIT-2).
- **Hallazgos + disposición:**
  1. [Low-1] Log RED citado solo contiene E0432 (nextest corta en el primer target; el E0433 de `scheduler_loop` quedó en consola sin artefacto) → **APLICADO (redacción)**: Step 1 ajustado con la captura exacta y la nota del gap.
  2. [Low-2] Doc de `timers_fired` decía "dispatched" pero incluye miembros desconocidos consumidos sin despacho → **APLICADO**: doc corregido (`scheduler.rs`).
  3. [Low-3] Sin assert de factory per-pass (>1 invocación; un fake cacheado pasaría) → **APLICADO**: el test de shutdown espera `passes >= 2` (interval 20ms, deadline 10s, sin flake).
  4. [NIT-1] Link intra-doc `[scheduler::spawn_memory_scheduler]` no resuelve en build default (1 warning rustdoc) → **APLICADO**: backticks en `services/mod.rs`.
  5. [NIT-2] "tokio-free" impreciso (tokio ya transitivo por reqwest en test builds) → **APLICADO**: comentario de `Cargo.toml` reformulado ("el pass no agrega uso de Tokio al default").
  6. [NIT-3] Prefijo de commit inconsistente prompt (`feat(wire):`) vs plan DoD (`feat(memory):`) → **RESUELTO**: se sigue la instrucción directa del orquestador `feat(wire):` (nota en §Notas).
- **Veredicto:** ✅ **APPROVE** — contrato + DoD Task + gates verdes; 0 Critical/High/Medium; fixes aplicados y re-verificados (suites re-corridas post-fix: 676/676 + 668/668, fmt/clippy verdes); changeset listo para commit local con pathspec (sin push).
- **Checklist anti-hábitos tóxicos** (§12 — verificado por el revisor):
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron (único gap = evidencia RED parcial, corregida).
  - [x] No saltarse la clarificación por "ya sé qué quiere" (Gate D pre-respondido por plan/ADR con evidencia).
  - [x] No declarar done sin verificar contra los acceptance criteria (Step 4 seguía PENDING al momento del review).
  - [x] No ignorar fallos ni reportar "todo OK" (fallo real de cfg en Step 2 documentado + corregido).
  - [x] No hacer un solo intento de búsqueda y darlo por saturado (impacto mapeado exhaustivo con path:línea).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad (P4 correcto; sin trust boundary nuevo).
  - [x] No gastar presupuesto infinito; paradas explícitas (verificación scoped por crate).
  - [x] Verificar cobertura SDP (v3): skills cargadas proporcionales al dominio (sin `pinned` faltantes).

## Notas

- **Forma del loop helper (incógnita resuelta):** factory genérica `Fn() -> Option<R>` por pass (§Spec #2) — la alternativa `Option<IngestRunnerCfg>` dejaba el path de procesamiento intesteable (el enum `ConcreteRunner` no es inyectable) y bloqueaba el fake del e2e de WIRE-16. Producción: `move || build_ingest_runner(&cfg)`; sin runner: `|| None` → skip.
- **Semántica de parada (handoff WIRE-14):** `shutdown()` graceful (watch + join) vía `BackgroundService` post-run loop; `Drop` best-effort (send + abort) cubre el early-return de `serve_http_or_tls` (espejo `MemoryTtlSweeper`; documentado en rustdoc).
- **Single-writer (ADR-0054):** un loop por DB/proceso (writer); el no-writer no lo ejecuta. WIRE-16 lo cablea; documentado en el rustdoc del módulo.
- **Skip observable:** factory `None` → `tracing::debug!` por tick, tareas quedan encoladas (P4). El runner degradado (`ConcreteRunner::None`) NO se skipea: corre y toma su path P4 existente (FIND-112 runner_config.rs:107-109).
- **NOTICED BUT NOT TOUCHING:** `run_bridge_pass` (semántica distinta: `usize::MAX` persona) no converge a `run_pass`; timers L2/Dream sin productor (YAGNI); `docs/api/**` → WIRE-17.
- **Commit scope (NIT-3 del review):** instrucción directa del orquestador = `feat(wire):`; el DoD del plan decía `feat(memory):`. Se sigue la instrucción directa (scope=wire) manteniendo `WIRE-15` en el subject; ambos parsean igual para release-plz (minor).
- **Coordinación:** WIP ajeno (`opencode.jsonc`, master plan) intacto; commit con pathspec.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — forma del loop/inyección resuelta en DISCOVERY (§Spec #2/#3) |
| Pendientes de ejecución (downhill) | **0** steps (1-4 ejecutados — commit + campaign al cierre) |
| % completado | 100% (steps 1-4 ejecutados; hash + campaign al cierre) |

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: <pendiente — commit local en curso; se actualiza en el commit docs de cierre>
ARCHIVOS: vanta-memory/src/services/scheduler.rs (nuevo), vanta-memory/src/services/mod.rs, vanta-memory/Cargo.toml, Cargo.lock (+tokio), vanta-memory/tests/scheduler.rs (nuevo), vanta-memory/tests/scheduler_loop.rs (nuevo), docs/dev/tasks/WIRE-15.md (nuevo), docs/index.md + llms.txt (gen-index)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:si→resuelto-por-evidencia V:no C:no | P:no disparado (plan Task 46 ya clasificado; sin ambigüedad de producto) · D:disparado por símbolos públicos nuevos → pre-respondido por el plan F0 (contrato L1326 sanciona run_pass + loop helper + factory) y resuelto por evidencia (§Spec #2/#3: factory genérica = testabilidad TDD + fake del e2e WIRE-16); worker sin `question` (question-gates §Routing) · V:no disparado (verde al primer intento tras 2 correcciones de compilación diagnósticas: E0283 de inferencia del factory `None` y cfg faltante detectado por el gate default) · C:no disparado (WIP ajeno no stageado; commit pathspec; sin colaterales)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base auto) · source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering (SDP v3 BUILD) · rust-write-tests, api-and-interface-design, documentation-skill (rol/cierre)
```
