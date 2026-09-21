# MEM-43 — Wire context engine → pipeline worker

**Plan:** docs/plans/2026-08-22-vanta-final-cierre.md — Task 1
**Estado:** ✅ COMPLETED
**Contrato:** cargo check/nextest/fmt/clippy -p vanta-memory exit 0; tests D19: worker ejecuta assemble_with_recall como fase post-L3 (compresión historial + MMD + recall budget compartido); e2e extendido demuestra compresión activa en el pass completo.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-memory/src/services/pipeline_worker.rs` (374L) — target de edición
- `vanta-memory/src/context_engine/engine.rs` (476L, secciones 25-270) — `assemble`, `assemble_with_recall`, `IntegratedContext`
- `vanta-memory/src/context_engine/types.rs` (118L) — ChatMessage/ChatRole/CompactionReport/ContextError
- `vanta-memory/src/offload/state_manager.rs` (197L) — cursor MEM-20
- `vanta-memory/tests/e2e_flow.rs` (384L) — patrón E2eRunner, fixtures, test MEM-37
- `vanta-memory/src/core/conversation/l0_recorder.rs` (parcial) — L0Role {User, Assistant}, read_messages
- `vanta-memory/src/context_engine/mmd.rs` (parcial) — load_active/save_active, TaskMemory

**Referencias hacia dentro (entrantes):**
- `MemoryTaskHandler::new`: llamado por tests/e2e_flow.rs (run_full_pass, run_l1_l2) y tests/pipeline_manager.rs — NO cambiar firma de `new()`; usar builder `with_context_config`.
- `PipelineWorker`: 1 caller en utils/pipeline_factory.rs — no se toca.
- `IntegratedContext`: 2 callers internos (mod.rs, engine.rs) — agregar derives serde es aditivo, no rompe.

**Referencias hacia fuera (salientes que consumirá el wiring):**
- `assemble_with_recall(msgs, budget, est, protected_prefix, cfg, active_mmd, prepend, append, cursor_tool_call_id)`
- `load_active(db, session) -> Result<Option<TaskMemory>, ContextError>`
- `perform_auto_recall(db, AutoRecallParams) -> Result<Option<RecallResult>, _>`
- `OffloadStateManager::last_offloaded_tool_call_id(session)`
- `L0Recorder::read_messages(session)`

**Veredicto:** cambio contenido a vanta-memory. Wiring productivo = MemoryTaskHandler gana fase `run_context_assembly` post-L3, gated por `context_compression_enabled` (default true). El worker es UN caller más de la API existente — cero cambios en algoritmos de compresión. Persistencia del resultado bajo namespace `context/<session>` key `__assembled` (patrón mmd.rs). NO tocar wal/vector/storage ni core vantadb.

## Steps

### Step 1 ✅ — Serde derives en IntegratedContext + ContextAssemblyConfig + fase run_context_assembly
- [x] `engine.rs`: agregar `Serialize, Deserialize` a `IntegratedContext`
- [x] `pipeline_worker.rs`: `ContextAssemblyConfig { enabled: bool (true), budget_tokens: u64 (8192) }` + campo en handler vía builder `with_context_config`
- [x] `run_context_assembly(session_id)`: L0→ChatMessage, MMD activo, recall (query = último mensaje user), cursor MEM-20, assemble_with_recall, persistir JSON en `context/<session>/__assembled`
- [x] `handle()` TaskKind::L3: run_l3 → si enabled, run_context_assembly
- [x] Reader público `load_assembled_context(db, session)`

### Step 2 ✅ — Tests D19 (tests/e2e_flow.rs)
- [x] Test A: pass completo con historial largo → registro `context/__assembled` existe, CompactionMode != None, MMD injectado, recall injectado, total ≤ budget, orden L0→L1→L2→L3→compress→recall assertionado (`d19_worker_assembles_context_post_l3_with_compression_active`, budget 800 → Aggressive 30→3 msgs)
- [x] Test B: flag disabled → no hay registro tras el pass completo (`d19_disabled_flag_skips_the_post_l3_phase`)

### Step 3 ✅ — Verify mecánico
- [x] cargo check -p vanta-memory — exit 0
- [x] cargo nextest run -p vanta-memory — 455/455 passed (453 base + 2 D19)
- [x] cargo fmt --check — exit 0 (tras cargo fmt)
- [x] cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings — exit 0

## Notas
- Sin deps nuevas. Sin unwrap/expect en código nuevo. Errores #[non_exhaustive] intactos.
- NO commitear (regla del orquestador esta sesión).
- Aprendizaje D19: los bloques de recall son whole-or-skip contra el headroom post-compresión — asertar el append de persona acopla el test a la aritmética del TokenEstimator; se aserte el flag `recall_injected` + el bloque dinámico con budget que fuerza compresión profunda (Aggressive 30→3).
- E0463 "can't find crate for rand" en Windows fue transitorio (flake de file-lock/AV): retry simple lo resolvió dos veces.
