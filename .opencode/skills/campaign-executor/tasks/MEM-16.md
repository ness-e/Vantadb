# MEM-16: F4 Orquestación timers+locks (estado local, reloj fake)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 19)
- **Fuente:** plan file Task 19 (MEM-16)
- **Esfuerzo:** 🔴
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-20
- **Estado:** ✅ COMPLETED (verify 4/4 gates exit 0)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Principios 1-7 + D19 + Task 19, task file `MEM-15.md` (112, plantilla + deuda checkpoint), TDAM `MC/utils/managed-timer.ts` (138 completo), `MC/core/state/types.ts` (167 completo), `MC/core/state/local-backend.ts` (318 completo), `MC/utils/checkpoint.ts` (L40-179: Checkpoint/RunnerSessionState/PipelineSessionState + índice de métodos), skims estructurales de `pipeline-manager.ts` (1218), `stateful-pipeline-manager.ts` (500), `services/pipeline-worker.ts` (843); crate: `l0_recorder.rs` (record_turn/read_messages/cursor/sanitize), `scene_extractor.rs` (extract_scenes_with_llm + SceneMemoryInput{id,content,created_at}), `persona_trigger.rs` (completo — evaluate_persona_trigger pura), `persona_generator.rs` (generate_persona/get_persona/has_persona_body), `l1_extractor.rs` (extract_l1_memories + L1ExtractorConfig), `l1_dedup.rs` (run_l1_dedup/batch_dedup), `l1_reader.rs` (read_session_records pub), `utils/mod.rs`, `services/mod.rs`, `core/mod.rs`, `Cargo.toml` (sin deps nuevas; dev-dep tempfile)
- **Referencias hacia dentro:** nuevos módulos consumen `core::conversation::{now_ms, sanitize_component, sanitize_key}` (pub(crate)), `core::abstractions::LlmRunner`, `core::record::{extract_l1_memories, run_l1_dedup, read_session_records}`, `core::scene::extract_scenes_with_llm`, `core::persona::{evaluate_persona_trigger, generate_persona, get_persona, has_persona_body}` — todo consumo, cero duplicación
- **Referencias entrantes:** ninguna hoy — MEM-17..19/37 consumirán la orquestación; wirings aditivos en `utils/mod.rs`, `services/mod.rs`, `core/mod.rs` (solo `pub mod`)
- **Veredicto impacto:** bajo — 9 archivos nuevos + 2 mod de directorio (`core/state/mod.rs`, ya existe `utils/mod.rs` y `services/mod.rs` con placeholder) + 3 wirings aditivos; cero archivos del core `vantadb`; API pública existente intacta

## Contrato

"`cargo check -p vanta-memory` pasa; tests dedicados de pipeline manager (D19) pasan (`cargo nextest run -p vanta-memory`); `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Simplificaciones documentadas (ponytail — NO port literal de ~5000 líneas TDAM)

| TDAM | Port Rust | Por qué |
|---|---|---|
| `ManagedTimer` sobre setTimeout (fire automático) | `ManagedTimer<C: Clock>` pull-based: deadline+callback, firing vía `poll()`/TimerScanner contra reloj inyectable | Sin threads ni tokio (Regla ponytail); firing determinista con FakeClock = tests sin sleeps |
| `IStateBackend` trait (Local + Redis) | Struct concreto `LocalStateBackend` (Mutex<Inner>) | Un solo backend → sin trait (YAGNI); Redis prohibido por Principio 7; un trait se extrae cuando exista el segundo backend |
| `CheckpointManager` archivo JSON + fileLocks + distributed lock opcional | `CheckpointManager` sobre store VantaDB (Principio 2): namespace `pipeline_checkpoint`, record JSON único; atomicidad por read-modify-write single-record | Persistencia = VantaDB siempre; sin locks distribuidos (proceso único) |
| `pipeline-manager.ts` (1218) + `stateful-pipeline-manager.ts` (500) | `MemoryPipelineManager` (mapas propios + ManagedTimers + warmup) y `StatefulPipelineManager` (estado en LocalStateBackend vía capture_atomic + persister a checkpoint) | Misma superficie pública mínima: notify_conversation / flush_session / mark_l1_complete; GC de sesiones y recovery de pendientes diferidos (deuda) |
| `pipeline-factory.ts` (1231) | `PipelineFactory`: build(config, clock) → {backend, manager, worker} | Solo el wiring que los tests ejercitan |
| `pipeline-worker.ts` (843) | `PipelineWorker`: consume cola priorizada, lock por sesión, handler trait `TaskHandler`, retry→dead-letter; `MemoryTaskHandler<R>` orquesta L0→L1(dedup)→L2(escenas)→L3(trigger+persona) con módulos existentes | Sin métricas/prometheus ni pending-recovery multi-worker (single-process); fases cableadas a extract_l1_memories/run_l1_dedup/extract_scenes_with_llm/evaluate_persona_trigger/generate_persona |

## Invariantes de dominio (handoff - MUST)

1. Reloj INYECTABLE en todo timer/lock/expiry — `Clock` trait + `SystemClock` + `FakeClock` (tests deterministas, cero sleeps).
2. Sin tokio/threads: timers pull-based (scanner/poll); worker `run_once` síncrono.
3. Locks con TTL + owner; renew solo del owner; release solo del owner; expiry limpiado perezosamente contra el clock.
4. Checkpoint persiste SOLO vía store VantaDB (Principio 2), namespace sanitizado ≤128 bytes.
5. `evaluate_persona_trigger` sigue siendo función pura — el checkpoint manager provee los contadores (paga deuda MEM-15).
6. LLM opcional (Principio 4): fallo de runner en handler → tarea a retry/dead-letter, NUNCA pérdida de datos L0.
7. Sanitización namespace `[A-Za-z0-9._/-]` ≤128, keys ≤512 sin NUL (reuso sanitize_component/sanitize_key).
8. Sin unwrap/expect en producción; errores tipados (thiserror); sin deps nuevas.
9. Cola priorizada determinista: priority asc, luego created_at asc (D19-testable).

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM (managed-timer, state/types, local-backend completos; checkpoint/pipeline-managers/worker estructural) + APIs del crate
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — core/state/types.rs + utils/{managed_timer,local_backend,timer_scanner}.rs + wiring parcial
- [x] `core/state/mod.rs` + `types.rs`: PipelineSessionState, TaskKind, TaskPayload, TimerEntry, CaptureAtomicParams/Result
- [x] `utils/managed_timer.rs`: Clock/SystemClock/FakeClock + ManagedTimer (schedule/schedule_at/try_advance_to/cancel/flush/poll)
- [x] `utils/local_backend.rs`: LocalStateBackend (buffers/states/timers/cola priorizada/locks/capture_atomic)
- [x] `utils/timer_scanner.rs`: TimerScanner (take_expired_timers + dispatch)
- [x] Wiring `core/mod.rs` + `utils/mod.rs`
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — utils/checkpoint.rs + pipeline_manager.rs + stateful_pipeline_manager.rs + pipeline_factory.rs
- [x] `checkpoint.rs`: Checkpoint/RunnerSessionState + CheckpointManager sobre store (mark_persona_generated, set/clear_persona_request, increment_scenes_processed, merge_pipeline_states)
- [x] `pipeline_manager.rs`: MemoryPipelineManager<C> (warmup doubling, idle timer, threshold→enqueue)
- [x] `stateful_pipeline_manager.rs`: StatefulPipelineManager<C> (backend-backed + persister)
- [x] `pipeline_factory.rs`: PipelineConfig + build(config, clock)
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — services/pipeline_worker.rs + wiring final
- [x] `pipeline_worker.rs`: TaskHandler trait, PipelineWorker (consume/lock/retry/dead-letter), MemoryTaskHandler<R> (L0→L1→L2→L3)
- [x] Wiring `services/mod.rs`
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 5 — Tests D19 + verify completo + cierre
- [x] `tests/pipeline_manager.rs`: fake clock timers, warmup, capture_atomic, locks TTL, cola priorizada, checkpoint roundtrip→trigger, worker retry/dead-letter, e2e handler con fake runner
- [x] Verify: cargo check + nextest + fmt --check + clippy -D warnings exit 0
- [x] CIERRE: campaign_update_task_state taskId=19 completed + recitation; bloque RESULTADO §7
- **Gate:** verify todo exit 0

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Diferidos documentados (no deuda introducida): GC de sesiones stale del manager, pending-recovery multi-worker del TDAM worker, claimStaleTasks — single-process no los necesita hoy; se agregan cuando exista segundo backend/proceso.


## Recitation (canónico)

- **activeGoal:** MEM-16: F4 Orquestación timers+locks (estado local, reloj fake)
- **lastAction:** Implementada la capa de orquestación MEM-16: trait Clock inyectable (SystemClock + FakeClock determinista) en managed_timer.rs con ManagedTimer pull-based (schedule/schedule_at/try_advance_to downward-only/cancel/flush/poll, guard destroyed); LocalStateBackend (Mutex<Inner>: buffers, session states, timers, cola priorizada por priority+created_at estilo TDAM, locks TTL owner-scoped, capture_atomic crítico); TimerScanner pull-based; CheckpointManager sobre store VantaDB (namespace pipeline_checkpoint sanitizado; mark_persona_generated/set-clear_persona_request/increment_scenes_processed/add_memories_extracted/merge_pipeline_states/persona_trigger_input — PAGA deuda MEM-15); MemoryPipelineManager (warmup 1→2→4→cap, idle timer, threshold→enqueue) + StatefulPipelineManager (backend-backed + persister a checkpoint); PipelineFactory mínimo; PipelineWorker (consume priorizado, lock por sesión, retry attempts→dead-letter, release SIEMPRE antes de actuar) + MemoryTaskHandler<R: LlmRunner> que orquesta L0(read_messages)→L1(extract_l1_segments+run_l1_dedup+contadores checkpoint)→L2(read_session_records→extract_scenes_with_llm→increment_scenes_processed)→L3(persona_trigger_input→evaluate_persona_trigger→generate_persona→mark_persona_generated). Extensión aditiva en l1_extractor.rs: extract_l1_segments devuelve las memorias para el dedup sin romper extract_l1_memories. 21 tests nuevos D19 (6 unit ManagedTimer + 15 integration tests/pipeline_manager.rs), todo con FakeClock, cero sleeps, cero threads/tokio.
- **result:** OK — 4/4 gates exit 0: cargo check ✅, nextest 240/240 (219 previos + 21 nuevos) ✅, fmt --check ✅, clippy -D warnings ✅ (7 warnings unsafe pre-existentes del core vantadb, fuera de scope)
- **nextAction:** Ninguna para MEM-16. Siguiente tarea del plan: Task 20 (MEM-17 — skill extractor); el lead commitea (`feat:` MEM-16)
- **contract:** cargo check -p vanta-memory exit 0; cargo nextest run -p vanta-memory 240/240 passed; cargo fmt --check exit 0; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings exit 0
- **invariantes:** reloj inyectable en todo deadline; sin tokio/threads (timers pull-based); locks TTL+owner; checkpoint SOLO vía store VantaDB; evaluate_persona_trigger sigue pura; LLM opcional (fallo→retry/dead-letter, nunca pérdida L0); sin unwrap/expect producción; sin deps nuevas
- **deuda:** GC de sesiones stale y pending-recovery multi-worker del TDAM diferidos (single-process no los necesita); glue L1-e2e con formato real del parser L1 cubierto por tests existentes de l1_extractor
- **nextTask:** Task 20 (MEM-17 — F4 Skill extractor)
