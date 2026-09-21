# MEM-55 — conversation/add dispara extracción L1 (H6)

**Plan:** docs/plans/2026-08-22-vanta-ultima-milla.md · Task 7 · Estado inicial: ⬜ PENDING
**Contrato:** tests D19: POST /conversation/add → thread guardado → tarea de extracción encolada (worker MEM-16 o spawn) → memories aparecen en l1/<session>; fallo de extracción NO falla el HTTP response (P4).

## Impacto mapeado (Regla 0)

**Archivos leídos completos (secciones relevantes):**
- `src/cli_server.rs:100-260` (ServerState struct :109-128, router :212-259), `:2569-2640` (ConversationAddRequest + conversation_add handler), `:2860-2980` (patrón de tests: ServerState literal con BackendKind::InMemory, spawn_app helper).
- `vanta-memory/src/services/pipeline_worker.rs:1-565` (TaskHandler :39, PipelineWorker::run_once :126, MemoryTaskHandler :195-220, run_l1_inner :251, handle dispatch :498-513).
- `vanta-memory/src/core/state/types.rs:34-66` (TaskKind::L1, TaskPayload campos).
- `vanta-memory/src/core/conversation/l0_recorder.rs:133-212` (record_turn firma, cursor filter, L0Message/L0Capture).
- `vanta-memory/src/utils/local_backend.rs:36-66,180-218` (LocalStateBackend::new/enqueue_task/consume_task/queue_depth; id generado por enqueue).
- `vanta-memory/src/core/record/l1_reader.rs` (l1_namespace, read_session_records — asserts de l1/<session>).
- `vanta-memory/tests/e2e_flow.rs:1-130` (fake runner scripted por task_id, open_db InMemory, fixture task()).

**Referencias hacia dentro:** `conversation_add` registrado en router (:255); ServerState construido en **17 sitios literales** (grep `ServerState {`): src/cli_server.rs ×7 (uno productivo :1762, resto tests), src/cli_server_auth_tests.rs ×1, vantadb-server/tests/{server×4, helpers×1, e2e×1, benchmarks×1}.

**Referencias entrantes clave:** dirección de deps `vanta-memory → vantadb` (Cargo prohíbe ciclo) ⇒ el trigger NO puede vivir en core llamando a vanta-memory; el gancho debe ser una trait aditiva en core (pre-mortem del plan) y la implementación vive en vanta-memory.

**Veredicto de impacto:** agregar campo `conversation_trigger: Option<Arc<dyn ConversationTrigger>>` a ServerState rompe los 17 literales ⇒ editar todos con `None` (mecánico). Handler dispara el trigger best-effort tras guardar. En vanta-memory: módulo puente feature-gated `http-server = ["vantadb/server"]` (off por default, build lean preservado). Sin deps nuevas.

## Steps

### Step 1 — Core: trait + campo + disparo best-effort ✅
- [ ] Trait `ConversationTrigger` (object-safe, `trigger(&self, thread_id: u128, role, content) -> Result<(), String>`) en cli_server.rs junto a ServerState.
- [ ] Campo `conversation_trigger` en ServerState + `None` en los 17 literales.
- [ ] conversation_add: tras `Ok(id)`, llamar trigger; `Err` → tracing::warn y respuesta 201 igual (P4).
- [ ] Tests core: trigger grabado con thread_id correcto + contenido; trigger que falla → 201 igual.

### Step 2 — vanta-memory: puente HTTP→pipeline ✅
- [ ] Feature `http-server = ["vantadb/server"]` en Cargo.toml.
- [ ] `src/services/conversation_hook.rs`: `HttpCaptureBridge<C: Clock>` implementa la trait core → captura L0 (`record_turn`, session = thread_id decimal) + encola `TaskKind::L1` en `LocalStateBackend` compartido.
- [ ] `run_bridge_pass(queue, db, runner)` — driver del worker MEM-16 (MemoryTaskHandler con configs default). Sin runner configurado el host simplemente no lo llama: mensajes quedan seguros en `l0/` y tareas pendientes (fallback P4 documentado).
- [ ] Registro cfg-gated en services/mod.rs.

### Step 3 — Tests D19 (vanta-memory/tests/conversation_hook.rs, cfg http-server) ✅
- [ ] Happy path: trigger → tarea encolada (queue_depth) → run_bridge_pass con runner scripted → memories en `l1/<session>` (read_session_records).
- [ ] Fallo extracción: runner falla → stats.failed, l0 intacto, HTTP-contract lo cubre el test core (201); retry sano recupera.
- [ ] Rol inválido → Err best-effort, nada encolado.

### Step 4 — Verify mecánico + cierre ✅
- [ ] `cargo check -p vantadb --features server --all-targets`
- [ ] `cargo test -p vantadb --features server --lib`
- [ ] `cargo test -p vanta-memory --features http-server` y `cargo test -p vanta-memory` (default intacto)
- [ ] `cargo fmt -p vantadb -- --check` · `cargo fmt -p vanta-memory -- --check`
- [ ] `cargo clippy -p vantadb --all-targets --no-deps -- -D warnings` (+ `-p vanta-memory`)
- [ ] campaign_update_task_state completed + recitation §3. Sin commit (regla de esta invocación).

## Context Save Point
(nada aún — tarea sin intentos previos)

