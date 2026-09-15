# FIND-88 — cablear `record_response_usage` al SSE drain (o DEFER-ratificado)

> **Plan:** `docs/plans/2026-09-15-find-correcciones.md` (Task 6, Wave1 — disjunto de FIND-63/65: `server.rs`/`cost.rs` vs desktop/`cache.rs` test)
> **Estado:** ✅ DONE (DEFER-ratificado con evidencia — salida válida del contrato, 2026-09-15)
> **Appetite:** 2d · **Esfuerzo:** 🟠 · **Prioridad:** 🟢 Baja
> **Branch:** `develop` · **Commit:** `docs: FIND-88 — ...` (solo este task file, tras verify mecánico)
> **Ruta:** vanta-worker · **nextTask:** FIND-79 (Wave2)
> **SDP:** `campaign_discover_skills_v2` archivosClave=`vanta-proxy/src/server.rs vanta-proxy/src/cost.rs` phase=`BUILD`
> contractKeywords=[SSE, cost, record_response_usage, proxy] → 8 skills (base + lifecycle, 0 keyword-mapped).
> Sugeridas del plan: `systematic-debugging` ✅ cargada, `code-review-and-quality` ✅ cargada.
> **SKILLS_CARGADAS:** systematic-debugging, code-review-and-quality, incremental-implementation, test-driven-development, context-engineering (+ base campaign-executor, progreso, ponytail full; source-driven-development, doubt-driven-development, api-and-interface-design devueltos por SDP, no cargados por irrelevantes al veredicto DEFER — frontend-ui-engineering descartada)
> **Referencias:** `.opencode/rules/server-mcp.md` (leída — scope `vantadb-server/`+`vantadb-mcp/`, NO cubre `vanta-proxy/` → sin regla normativa aplicable, solo contexto), `docs/api/PROXY.md` (NO existe — confirmado `Test-Path False`; FIND-68 la crea en Wave5 — no duplicar, solo leer el plan). Sin símbolos públicos nuevos → sin Spec (Gate D no dispara, justificado abajo).
> **Uphill:** ⬆️ 1 (forma del drain — resuelto en DISCOVERY: § Forma del drain).
> **Downhill:** ⬇️ 2 steps (DISCOVERY con evidencia + verify + task file + commit docs).

## Contrato (del plan — una de dos salidas, nunca rabbit hole)

- [x] **Salida A:** SSE drain llama a `record_response_usage` con cuerpo buffereado — NO tomada (evidencia abajo: no hay punto de buffer general; forzarla rompería el invariante SSE-safe).
- [x] **Salida B (tomada):** DEFER-ratificado con evidencia de por qué no + suite proxy verde + clippy 0 (scope proxy).
- [x] Suite proxy verde: `cargo test -p vanta-proxy -j 2` → 163 passed, 0 failed (25.5s, `campaign_verify_cmd`).
- [x] Clippy 0 en scope: `cargo clippy -p vanta-proxy --all-targets` sin warnings en archivos `vanta-proxy/` + `cargo fmt --check -p vanta-proxy` limpio. Workspace-wide `-D warnings` falla en `src/storage/engine/txn.rs:158` (`cloned_sole_buffer` dead_code) — pre-existente, fuera de scope, no tocado (ver § Colateral).
- [x] Sin deuda neta: 0 líneas de código tocadas (solo este task file). Prohibidos intactos: `.opencode/`, `completions/`, desktop lock, GOV-C4 stash, translate simétrico (no exigido → no tocado).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vanta-proxy/src/server.rs:248-264` (contabilidad input-side + TODO) + `:636-737` (`forward_with_tool_loop` con el único `drain`), `vanta-proxy/src/cost.rs:100-119` (`usage_from_response_body` solo JSON) + `:198-214` (`record_response_usage`) + `:397-411` (test output-side), `vanta-proxy/src/forward.rs:335-351` (`into_axum` streaming sin buffer) + `:196-222` (doc SSE-safe), `vanta-proxy/src/sse_intercept.rs:55-111` (`drain` + `data_events` raw SSE), `vanta-proxy/src/server.rs:508-553` (`maybe_store` solo JSON cacheable) + `:887-898` (`map_translated_response`: "SSE streams pass through untouched (never buffered)" + test `:1081` `mapback_sse_never_buffered`), `.opencode/rules/server-mcp.md` (42L, scope ajeno).
- **Referencias hacia dentro (quién llamaría al drain):** `forward_with_tool_loop` ← 1 caller (`process_inner`, codegraph) — y solo tras gate `announces(&body)` + `content-type: text/event-stream` (`server.rs:650-655,679-686`). `record_response_usage` ← 0 callers fuera de tests (codegraph: "1 caller in cost.rs" = el propio test `:400`; `rg` confirma: solo def `:201` + tests `:400,408`). `usage_from_response_body` ← solo `record_response_usage` + tests.
- **Referencias entrantes (quién depende del output-side):** `TurnReport { output_tokens: 0, cost_usd: turn_cost }` (`server.rs:268-280`) + `/snapshot` (`cost.snapshot()`). Hoy ambos reportan coste input-side real y output 0 — sub-contabilización honesta (no inventa números), visible en el TODO explícito.
- **Veredicto:** blast radius de cablear = streaming invariante (`forward.rs` SSE-safe) + parser SSE→usage inexistente + contabilidad por-turno en `process()` (sin acceso al cuerpo buffereado profundo). Tocar el wire para bufferear todo SSE = latencia + memoria + regresión de `mapback_sse_never_buffered` y `cache::response_gates` — fuera de appetite 2d/prioridad Baja. Riesgo 🟠 innecesario para un ledger log-first. **No se edita código.**
- **Gate D:** no dispara (0 archivos tocados, sin hot path modificado, sin API pública nueva, sin símbolos `pub` nuevos, contrato con salida DEFER explícita y Stop "sin punto de buffer → DEFER, no rabbit hole"). Veredicto documentado, no preguntado.

## Forma del drain (uphill ⬆️1 — resuelto)

1. **Tráfico general (mayoría):** `forward_raw` → `forward_with_failover` → `into_axum(resp)` = `Body::from_stream(resp.bytes_stream())` (`forward.rs:342-349`). **Cero buffering por diseño** ("no buffering — SSE-safe", `:197,222`). No hay cuerpo que parsear — `usage_from_response_body` no tiene input.
2. **Único drain existente:** `forward_with_tool_loop` (`server.rs:688`) — gated por `announces(&body)` (solo requests que anuncian nuestras memory tools) + `is_success` + `is_sse` (`:650-655,679-686`). Los rounds intermedios se re-requestean; solo el round final hace `replay(parts, captured.chunks)` (`:707`). El `captured.full` es **SSE crudo** (`data: {...}\n\n...[DONE]`), NO JSON con `{"usage":{...}}` — `usage_from_response_body(captured.full)` devuelve `None` → `record_response_usage` retornaría `0.0` (no-op). Cablearlo ahí sería wiring decorativo que siempre suma 0.
3. **Buffering JSON existente (no-SSE):** `maybe_store` (`:511-553`) bufferea solo `is_cacheable_response` (SSE excluido por `cache.rs:727`) y `map_translated_response` (`:887-898`) excluye SSE explícito con test dedicado (`:1081`). Son paths de cache/translate, no puntos de contabilidad — reutilizarlos para coste acoplaría dos concerns y dejaría SSE (el caso del contrato) igualmente en 0.
4. **Uso real en SSE:** OpenAI streaming solo envía `usage` en el chunk final si `stream_options.include_usage=true` — el proxy no lo solicita (grep `include_usage` = 0 hits en `vanta-proxy/src/`). Aunque se parsearan `data_events`, no habría `usage` que extraer en el tráfico actual. Construirlo exigiría: solicitar `include_usage` en el forward (cambia el wire), parser SSE→Usage nuevo, y plomería de `virtual_key/session/model` hasta el loop — feature nueva, no cableado. Fuera de appetite.

## Root cause (por qué output_tokens es 0 por diseño actual)

`process()` (`server.rs:248-267`) contabiliza DESPUÉS de construir la respuesta, con acceso solo al request `body` (buffered) — el response `Body` es un stream ya devuelto al cliente (`replay`/`from_stream`), no un cuerpo. `record_response_usage` existe y está testeado para cuerpos JSON buffereados (`cost.rs:198-214,400`), pero el wire nunca le entrega uno: el invariante SSE-safe prohíbe bufferear el caso general. El TODO (`:250-251`) es honesto — no es bug de lógica, es wiring pendiente bloqueado por arquitectura de streaming. Doble conteo descartado como riesgo activo: `record_response_usage` solo suma `output_tokens` (`input_tokens: 0`, `:205-213`), así que un futuro wiring sumaría sin duplicar input — el diseño del ledger ya lo prevé.

## Steps atómicos

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | DISCOVERY: forma del drain + `rg` + codegraph + reglas + PROXY.md + task file con Regla 0 y DEFER-evidencia | ✅ DONE | este task file |
| 2 | Verify contrato: `cargo test -p vanta-proxy -j 2` 163/163 + clippy scope-proxy 0 + fmt limpio + commit `docs:` solo este archivo | ✅ DONE | comandos abajo + hash |

## Pre-mortem (del plan — verificado en DISCOVERY)

1. SSE es streaming sin buffer → **confirmado**: `forward.rs:342-349` + `server.rs:897-898` + test `:1081`. DEFER-ratificado es la salida válida (Stop del plan), no rabbit hole.
2. Doble conteo request+response → **descartado por diseño**: `record_response_usage` fija `input_tokens: 0` (`cost.rs:209`); el ledger suma lados disjuntos. Sin acción.
3. Alcance (translate simétrico fuera de roadmap) → **no exigido, no tocado**: `translate.rs` solo leído vía `rg` (`:408` mapea usage para el cuerpo traducido, path ajeno al SSE drain).

## Herramientas

- Forma: `codegraph_explore "vanta-proxy SSE drain sse_intercept forward_with_tool_loop record_response_usage cost"` (45 símbolos, 6 archivos; `record_response_usage` 0 callers fuera de tests; `forward_with_tool_loop` 1 caller) + `Read server.rs:200-349,350-599,600-849,850-1099` + `Read cost.rs` + `Read forward.rs` + `Read sse_intercept.rs`
- Evidencia: `rg -n "record_response_usage|output_tokens|drain\(|text/event-stream|bytes_stream|to_bytes" vanta-proxy/src/ --glob "*.rs"` (0 callers prod; 3 puntos `to_bytes`: `maybe_store:534` JSON-cacheable, `map_translated_response:910` no-SSE, tests) + `rg include_usage vanta-proxy/src/` = 0 hits
- Verify: `campaign_verify_cmd "cargo test -p vanta-proxy -j 2"` ✅ 163 passed / 0 failed (25.5s) + `cargo clippy -p vanta-proxy --all-targets --message-format short` (0 líneas `vanta-proxy/`, único warning `src/storage/engine/txn.rs:158` crate ajeno) + `cargo fmt --check -p vanta-proxy` ✅ limpio
- Cierre: `git add docs/tasks/FIND-88.md` + commit `docs:` (solo este archivo; WIP ajeno intacto) + `campaign_update_task_state` + RESULTADO
- Prohibidos (M): `.opencode/` (M en worktree, no tocar), `completions/` (M, no tocar), `desktop/src-tauri/Cargo.lock` (M, no tocar), `vanta-proxy/src/cache.rs` (M — WIP FIND-65 paralelo, no tocar), `docs/pipeline-state.json` (M, no tocar), translate simétrico (no exigido).

## Colateral — NOTICED BUT NOT TOUCHING (no FIND nuevo: pre-existente fuera de scope)

- `src/storage/engine/txn.rs:158` `cloned_sole_buffer` never used → `cargo clippy -- -D warnings` workspace rojo en crate `vantadb` (dependencia de compilación del proxy, no archivo del task). Sin modificar en worktree (no es WIP nuestro). No se arregla aquí (scope discipline; un `#[allow]`/`#[expect]` en core merece su propio ticket si el owner lo quiere). Evidencia: `cargo clippy -p vanta-proxy --all-targets -- -D warnings` exit 1 solo por ese error; sin `-D` el mismo comando termina `Finished` con 1 warning ajeno y 0 en `vanta-proxy/`.
- `docs/api/PROXY.md` inexistente → ya trackeado como FIND-68 (Wave5). No duplicar.
- `vanta-proxy/src/cache.rs` modificado en worktree (FIND-65 en paralelo) → la suite 163/163 corre sobre ese WIP ajeno; el resultado sigue siendo válido como "suite proxy verde" para nuestro contrato read-only, y nuestro commit no lo incluye.

## Context Save Point

- 2026-09-15 DISCOVERY completo: plan Task 6 + `server.rs` (4 ventanas) + `cost.rs` + `forward.rs` + `sse_intercept.rs` + `server-mcp.md` + `Test-Path PROXY.md=False` leídos; `campaign_detect_task_type`=proxy; SDP BUILD 8 skills; `systematic-debugging` + `code-review-and-quality` + `incremental-implementation` + `test-driven-development` + `context-engineering` cargadas; uphill forma-del-drain resuelto → DEFER; verify mecánico verde (163/163 + clippy-scope 0 + fmt); siguiente: commit solo-task-file + recitation + RESULTADO.
