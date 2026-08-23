# MEM-51 — H2/O2 Interceptor de stream con loop agéntico de memory-tools

## Plan
- **Task 2 del plan P33** (`docs/plans/2026-08-22-vanta-ultima-milla.md`) · 🔴 · appetite max 3d
- **Contrato:** cargo check/test/fmt/clippy `-p vanta-proxy --all-targets` exit 0; tests D19 (a)-(e) con upstream mockeado que scripta tool_use.

## Impacto mapeado (Regla 0)
**Archivos leídos completos:**
- `vanta-proxy/src/server.rs` — `AppState`, `process`/`process_inner` (auth→rate→mem-cmd→session→inject→forward_raw), `capture_turn`, `forward_raw`.
- `vanta-proxy/src/forward.rs` — `Forwarder::forward` devuelve axum `Response<Body>` streaming; hop-by-hop strip; timeout cliente 600s.
- `vanta-proxy/src/inject.rs` — `Protocol`, `TOOL_SPECS` (`vanta_memory_capture`, `vanta_memory_search`), `merge_tools`, `inject_into`.
- `vanta-proxy/src/capture.rs` — `turn_job(memory, session, protocol, space, model, text) -> L0Job`, `TURNS_NAMESPACE="proxy-turns"`, `list_turns` (tests).
- `vanta-proxy/src/writeback.rs` — `track(label, job)` fire-and-forget con retry 3x.
- `vanta-proxy/src/session.rs` — `session_key_from_headers` (aliases x-conversation-id/x-session-id/x-claude-code-session-id).
- `vanta-proxy/src/config.rs` — `ProxyConfig` completo (sin flag nuevo para el loop: gate = presencia de nuestras tools en el body post-inject).
- `vanta-proxy/tests/proxy_wire.rs` — patrón mock upstream (axum Router en 127.0.0.1:0) + seeded_engine + post_json.
- `vanta-memory/src/core/hooks/auto_recall.rs` — `perform_auto_recall(db, AutoRecallParams, Option<&EmbedFn>)`; `RecallResult.prepend_context` trae el bloque `<relevant-memories>`.

**Referencias entrantes:** handlers/{openai,anthropic,responses}.rs llaman `state.process(...)`; router en server.rs. Los módulos nuevos no tienen callers previos.
**Referencias salientes (nuevos módulos):** `sse_intercept.rs` y `memory_tools.rs` se declaran en `lib.rs`; usados SOLO por `server.rs`.
**Veredicto de impacto:** crate-local (`vanta-proxy`). Core intocable. Riesgo principal: romper passthrough streaming existente → test (c) byte-identical + test (g) existente como gates permanentes.

## Steps

### ✅ S1 — `sse_intercept.rs`: buffering + parseo SSE (OpenAI + Anthropic) + unit tests
- `StreamCapture { chunks, full }`, `drain(Body)`, `replay(parts, chunks)`, `data_events(&[u8]) -> Vec<Value>`, `openai_message` / `anthropic_message` que reconstruyen el mensaje assistant completo.
- Unit tests in-module: CRLF, `[DONE]`, basura tolerada; acumulación OpenAI (fragments arguments) y Anthropic (input_json_delta); drain preserva chunks.

### ✅ S2 — `memory_tools.rs`: gate, extracción, ejecución (D47 capture / recall search), síntesis tool_result
- `announces(body) -> bool`; `extract()` filtra llamadas memory-tool del mensaje acumulado.
- Capture → `writeback.track(capture::turn_job(...))` fire-and-forget (mismo camino D47). Search → `perform_auto_recall` síncrono → bloque `<relevant-memories>` o "No relevant memories found."
- `append_exchange(protocol, req, assistant, results)` — shape estándar OpenAI (role:tool) y Anthropic (tool_result en mensaje user).

### ✅ S3 — Integración `server.rs`: `forward_with_tool_loop` (cap duro D48 = 3 iteraciones)
- Gate cero-overhead: protocolo OpenAI/Anthropic + body con nuestras tools + respuesta SSE exit. Resto → forward_raw verbatim.
- Loop: forward → buffer → ¿tool_use nuestro? no → replay bytes verbatim; sí → ejecutar → re-request. Cap 3 ejecuciones (máx 4 forwards) → streamear último response verbatim.
- Responses protocol → passthrough (techo documentado).

### ✅ S4 — Tests D19 (a)-(e) en `tests/tool_loop.rs`
- (a) openai capture loop (2 forwards + persistencia proxy-turns verificada por polling); (b) anthropic search loop con recall síncrono (shape tool_result estándar); (c) sin session/sin tools → byte-identical; (d) cap = exactamente 4 forwards, 4to response replayado verbatim; (e) streaming final intacto (content-type SSE + bytes completos en orden).

### ✅ S5 — Verify mecánico completo + cierre
- `cargo check -p vanta-proxy --all-targets` ✅ 0 · `cargo test -p vanta-proxy` ✅ 73/73 (53 lib + 5 pipeline + 10 wire + 5 tool_loop) · `cargo fmt -p vanta-proxy --check` ✅ · `cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings` ✅ 0.
- NOTA BND-06: `cargo nextest -p` roto (GOV-C1) → verificación con `cargo test` equivalente aceptada.
- Sin commit (regla explícita de la invocación).

## Context Save Point
Tarea COMPLETA sin commit. Cambios en worktree: vanta-proxy/{Cargo.toml, src/lib.rs, src/server.rs} modificados; src/{memory_tools,sse_intercept}.rs y tests/tool_loop.rs nuevos; task file nuevo. El lead commitea al cerrar la wave.
