# MEM-50 — Wire WriteBack::track al request path (H1 crítico)

Plan: `docs/plans/2026-08-22-vanta-ultima-milla.md` — Task 1. Estado: ⏳ IN PROGRESS.
Ruta: vanta-worker. Appetite ½d. Cynefin: 🟦 obvio.

## Contrato

`cargo check -p vanta-proxy --all-targets` pasa; tests D19: request completado →
track() encola el turno L0 → visible en pending queue → flush lo persiste;
fallo de enqueue NO rompe el forward. Suite proxy 52/52 se mantiene verde.

## Impacto mapeado (Regla 0)

**Archivos leídos completos (codegraph verbatim):**
- `vanta-proxy/src/server.rs` — `process()`/`process_inner()`: funnel único de los
  3 handlers (openai/anthropic/responses llaman todos a `state.process`). Ya tiene
  `self.writeback: Arc<WriteBack>` y `self.memory: Arc<VantaEmbedded>` en AppState.
- `vanta-proxy/src/writeback.rs` — API completa existe: `track(label, L0Job)`
  (spawn + with_l0_retry 3×500ms→1s→2s; en fallo final → `enqueue` a pending +
  persist), `pending_count()`, `flush(deadline)`. **No falta API** — solo wiring.
- `vanta-proxy/src/mem_command.rs` — `extract_text(content)` (privada): string o
  array `{type:"text"}` → String. Reutilizable para extracción user-text mínima.
- `vanta-proxy/src/handlers/{openai,anthropic,responses}.rs` — thin wrappers que
  delegan a `process()`; **no requieren edición** (el wiring en el funnel cubre los 3).
- `tests/pipeline.rs` — patrón de setup: `seeded_engine()` InMemory +
  `AppState::from_engine` + mock upstream axum + `post_chat`.

**Referencias entrantes:** server.rs router → handlers; handlers → process().
**Referencias salientes:** process → forwarder/inject/session/auth/writeback.
**Veredicto:** cambio aditivo localizado en vanta-proxy. Nuevo módulo `capture.rs`
+ ~10 líneas en `server.rs::process`. Cero cambios en core (`vantadb`) ni APIs
públicas existentes. Riesgo bajo.

## D47 / diseño

Un solo camino para writes L0: tras forward exitoso (2xx) con session key,
extraer user-text del body ORIGINAL y llamar `writeback.track("turn:<session>", job)`.
El job hace `VantaEmbedded::put(namespace="proxy-turns", key="{ms}-{seq}", payload=JSON{session,protocol,space,model,text})`.
Fire-and-forget post-respuesta: fallo de put/enqueue jamás afecta la respuesta.

## Steps

1. ✅ `mem_command::extract_text` → `pub(crate)` (reuso, no duplico).
2. ✅ Nuevo `src/capture.rs`: `last_user_text`, `turn_job`, `list_turns`, const
   `TURNS_NAMESPACE`; tests unitarios extracción + D19 mecánica
   (fallo→pending→flush persiste). Registrado en lib.rs.
3. ✅ Wiring en `server.rs::process`: clone barato de Bytes, si 2xx →
   `capture_turn(...)` (método privado).
4. ✅ Test integración `g_completed_request_tracks_l0_turn` en tests/pipeline.rs:
   request con session header → 200 → turno persistido legible; sin session
   header → sin turno.
5. ✅ Verify full: check --all-targets ✅ · nextest -p vanta-proxy 57/57 ✅ ·
   fmt --check ✅ · clippy -p vanta-proxy --all-targets --no-deps -D warnings
   exit 0 ✅ (7 warnings pre-existentes de core `vantadb`, fuera del crate).
   Sin commit (regla del lead).

## Notas

- MCP WIP-blocked al abrir (GOV-B2/GOV-D2 de otras sesiones): no se pudo marcar
  in-progress vía campaign_update_task_state; se cierra igual al terminar.
- Extracción user-text es mínima inline (pre-mortem): sólo shape `messages`;
  refinamiento completo = Task 8 (MEM-57).
