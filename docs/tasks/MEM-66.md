# MEM-66: claimStaleTasks (recuperación multi-worker)

## Metadata
- **Plan file:** docs/plans/2026-09-08-backlog.md (Task 9, Wave1)
- **Creado:** 2026-09-09
- **Estado:** ⏳ IN PROGRESS
- **Type:** Rust core (feature-add port TDAM, lógica nueva pequeña)
- **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design (+ progreso, ponytail full base). `frontend-ui-engineering` descartada (no hay web/ en el slice).
- **Appetite:** max 1d · **Esfuerzo:** 🟡 1d · **Wave:** Wave1 (disjunto BND-08/PRX-08)

## Gate D (question-gates.md)
- **Disparado SÍ:** la solución agrega símbolos `pub` nuevos (`claim_task`, `complete_task`, `renew_task_lease`, `claim_stale_tasks`, `pending_count`, `with_owner`, `reclaim_stale`).
- **Resolución:** PRE-APROBADO — el plan DO nombra `claimStaleTasks` como deliverable, Gate P owner aprobó 2026-09-08, y la invocación ordena ejecutar este contrato exacto. Sin question-tool en este runner; blast radius = 3 archivos, sin hot path, sin cambio de protocolo. No se divide: cabe en 2 slices ≤1d.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vanta-memory/src/services/pipeline_worker.rs` (708L), `vanta-memory/src/utils/local_backend.rs` (364L), `vanta-memory/src/core/state/types.rs` (TaskPayload/TaskKind), `vanta-memory/tests/pipeline_manager.rs` (451L).
- **Referencias hacia dentro (lo que toco usa):** `Mutex<Inner>` único, `Clock::now_ms` (FakeClock-friendly), `TaskPayload{id,kind,session_id,priority,created_at_ms,attempts}`, `enqueue_task` (reasigna id `t_{created}_{seq}`), `acquire_lock/release_lock` (session lock TTL), `RunStats/TaskHandler`.
- **Referencias entrantes (quién usa lo que toco):** `PipelineWorker` ← `conversation_hook.rs`, `utils/pipeline_factory.rs`; `run_once` ← `conversation_hook.rs` (3 callers); `consume_task` ← `pipeline_worker.rs:216` + test `pipeline_manager.rs:220`; `snapshot()` ← test `:152` (solo lee `.timers`); `BackendSnapshot` ← `utils/mod.rs` re-export (constructor solo en `local_backend.rs:334`).
- **Ausencia confirmada:** `claimStaleTasks`/`claim_stale` no existe en código (solo mención doc `pipeline_worker.rs:12-13` + `pipeline_manager.rs:9`). `detect_changes` inbound depth 3 desde develop: 0 símbolos impactados (solo `.opencode`/`opencode.jsonc`/plan tocados en worktree — AJENOS, no tocar).
- **Veredicto:** impacto CONTENIDO — 2 fuentes + 1 test. `run_once` cambia a flujo claim (observable idéntico). Sin nesting de locks (un solo `Mutex`, guards por método). Regla 8 NO dispara (sin dashmap/parking_lot/Tokio/multi-índice).

## Contrato
`cargo test -p vanta-memory` 0 failed + test nuevo worker-muerto→reclaim ✅ + `cargo clippy -p vanta-memory -- -D warnings` 0

## Spec (decisiones — port de diseño TDAM existente, no greenfield)
| # | Decisión | Opción elegida | Por qué |
|---|----------|----------------|---------|
| 1 | Dónde vive el lease | `pending: HashMap<task_id, (TaskPayload, owner, expire_at_ms)>` en `Inner`, tras el MISMO `Mutex` | atomicidad gratis in-process; sin Redis (Principio 7) |
| 2 | Doble-reclaim (pre-mortem 1) | `claim_stale_tasks` reasigna owner+lease bajo el mutex en UNA sección crítica + `complete_task`/`renew` con check de owner (fencing); comentario del techo multi-proceso (CAS futuro) | imposible doble-reclaim in-process; test lo prueba con worker-C |
| 3 | TTL vs heartbeat (pre-mortem 2) | SEPARADOS: `renew_task_lease` = heartbeat del claim (task lease) vs `acquire/renew_lock` = session lock TTL | semánticas distintas, nombres distintos, doc en cada fn |
| 4 | Reclaim → queue o directo | `claim_stale_tasks` devuelve `Vec<TaskPayload>` re-asignados (siguen pending del nuevo owner); `PipelineWorker::reclaim_stale` los procesa inline con el MISMO settle que `run_once` | sin requeue fantasma; sin pérdida (pending hasta complete) |
| 5 | Owner duplicado | `PipelineWorker::with_owner` builder (el `new` usa `worker-{pid}` — 2 workers mismo proceso colisionarían) | tests multi-worker deterministas |
| 6 | `run_once` a flujo claim | `claim_task` → session lock → handle → `settle` (complete / requeue-atómico / dead-letter). `consume_task` se conserva (tests lo usan directo) | pending se puebla de verdad; si no, reclaim sería dead code |
| 7 | Requeue con claim | `requeue_task(&task, owner)`: complete old-id + enqueue (id nuevo) atómico bajo un lock; `false` si no-owner (sin mutación) | evita pending huérfano por el re-id de `enqueue_task` |
| 8 | Diagnóstico | `pending_count()` + campo `pending` en `BackendSnapshot` (constructor único local; tests solo leen `.timers`) | observable sin romper API |

## Steps
### Step 1: Backend claim/pending + lease atómico (local_backend.rs + test)
- **Archivos:** `vanta-memory/src/utils/local_backend.rs`, `vanta-memory/tests/pipeline_manager.rs`
- **Acción:** RED: test `stale_pending_task_reclaimed_once_by_new_owner` (claim A → muere → avanza clock → B reclaim 1 + C reclaim 0 + heartbeat `renew_task_lease` extiende + `complete_task` wrong-owner false). GREEN: `pending` map + `claim_task`/`complete_task`/`renew_task_lease`/`requeue_task`/`claim_stale_tasks`/`pending_count` + `snapshot.pending`.
- **Verify:** `cargo test -p vanta-memory --test pipeline_manager` 0 failed + `cargo clippy -p vanta-memory -- -D warnings` 0
- **Estado:** ✅ COMPLETE (2026-09-09: claim_task/complete/renew/requeue/claim_stale/pending_count + snapshot.pending + destroy.clear; test Step 1 pasa 1/1)

### Step 2: Worker reclaim + e2e worker-muerto→reclaim (pipeline_worker.rs + test)
- **Archivos:** `vanta-memory/src/services/pipeline_worker.rs`, `vanta-memory/tests/pipeline_manager.rs`
- **Acción:** `with_owner` + `run_once` a flujo claim/settle + `reclaim_stale(handler, lease_ttl, limit)` (session-lock por tarea, mismo settle). Test contrato: worker-A claim+muere → clock avanza → worker-B `reclaim_stale` procesa ✅ (`processed==1`), queue vacía, segundo reclaim 0.
- **Verify (contrato full):** `cargo test -p vanta-memory` 0 failed + `cargo clippy -p vanta-memory -- -D warnings` 0 + `cargo fmt --check` scoped
- **Estado:** ✅ COMPLETE (2026-09-09: with_owner + run_once claim/settle + reclaim_stale; test contrato worker-muerto→reclaim pasa; suite full 0 failed; clippy 0; fmt limpio)

## Dependencias
- Ninguna (Wave1 disjunta). Desbloquea: nada en este plan.

## Notas
- Deuda/WIP ajeno: `git status` M `opencode.jsonc` + M `.opencode` — NO tocar; commit SOLO paths propios.
- Ponytail: O(n) scan de `pending` en reclaim con comentario techo (pending es diminuto: ≤ queue depth); sin abstracción trait backend (un solo backend, YAGNI).
- Trust: fuente/tipos del proyecto = trusted; sin APIs externas.

## Context Save Point
- **Fecha:** 2026-09-09
- **Branch:** develop (verificar en cierre)
- **Decisiones:** spec tabla arriba (lease bajo mismo Mutex; heartbeat separado; reclaim inline con settle compartido)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** ninguna (una tarea por invocación; handoff al orquestador)
