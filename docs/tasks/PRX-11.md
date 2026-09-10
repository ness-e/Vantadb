# PRX-11 — Traducción Anthropic↔OpenAI fiel (slice 1: messages→OpenAI + responses + SSE mínimo)

> Plan: `docs/plans/2026-09-10-code.md` Task 19 · Wave6 SOLO (sin paralelo same-crate) · Ruta: vanta-worker
> Estado: ⏳ IN PROGRESS (slice 1) · Appetite: max 5d · Rama: `develop`
> SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (+systematic-debugging si RED falla)
> Estado: ✅ COMPLETE slice 1 (2026-09-10: RED E0432 → GREEN 9/9 → suite 0 failed → clippy/fmt 0 → commit solo-propio)

## Contrato

`/v1/messages`↔OpenAI + SSE streaming + tool_use/result incremental + thinking blocks + sanitización + max_tokens guard + beta headers + fixtures PRX-12 ✅ + clippy 0

**Slice 1 (esta invocación):** librería pura `translate.rs` (sin wiring al pipeline — el wire sigue verbatim por defecto). Slices 2+ (wiring opt-in por ruta/upstream, SSE incremental full, thinking fidelity fina) → DEFER con números si appetite >5d.

## Spec (gate mecánico spec-first — decisiones por evidencia)

| # | Decisión | Opciones | Evidencia / fuente | Elegido |
|---|----------|----------|-------------------|---------|
| 1 | Dirección slice 1 | A) messages→OpenAI primero / B) bidireccional total | Pre-mortem plan: scope total excede; fixtures PRX-12 prueban ambas direcciones pero el gap real es Anthropic→OpenAI (verbatim hoy) | A + inversa non-streaming mínima (misma lib, costo marginal) |
| 2 | `max_tokens` guard | A) default+clamp / B) rechazar si falta | Fixture claude req trae `max_tokens:1024`; Anthropic lo exige, OpenAI no. Rechazar rompería compat | A: default 1024, clamp [1,128000] |
| 3 | `system` (string vs array) | A) ambos / B) solo string | `inject.rs` ya maneja ambos (string + array con `type:text`); fixture usa string | A: string→system message; array→concat text blocks (ignora no-text) |
| 4 | `thinking` blocks | A) strip al traducir a OpenAI / B) serializar como texto | OpenAI no tiene thinking; fidelidad total exige variante por fixture (pre-mortem). Slice 1: no inventar semántica | A: request→drop (con `dropped_thinking:true` en meta solo en tests, no en wire); response `reasoning_content`→`thinking` block |
| 5 | `tool_use`/`tool_result` | A) mapear a tool_calls/tool + B) texto plano | `sse_intercept.rs` ya acumula tool_calls por índice (OpenAI) y por bloque (Anthropic); PRX-12 fixtures usan `Read` tool ambos lados | A: `tool_use{id,name,input}`↔`tool_calls[{id,function{name,arguments}}]`; `tool_result`↔role `tool` |
| 6 | SSE | A) chunk mappers puros por evento / B) stream stateful | Pre-mortem: SSE a medias → tests por chunk; `sse_intercept::data_events` ya parsea eventos | A: fns puras `anthropic_event_to_openai` / `openai_chunk_to_anthropic` + test por chunk |
| 7 | `beta headers` | A) helper passthrough allowlist / B) ignorar | Contrato exige beta headers; el forwarder reenvía headers (salvo hop-by-hop) | A: `pass_through_beta_headers` allowlist `anthropic-beta, anthropic-version, x-api-key` (nunca секреты en logs) |
| 8 | Sanitización | A) strip nulls + clamp + drop vacíos / B) validar estricto | Pipeline es fail-open (server.rs); estricto rompería verbatim | A: recursivo null-strip, clamp max_tokens, drop messages sin content |
| 9 | Wiring | A) lib pura sin hook / B) hook en pipeline ya | Regla 0: tocar pipeline = blast radius 900L server.rs + D34/cost/routing intactos en juego | A: slice 1 sin wiring; `pub mod translate` solo |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vanta-proxy/src/lib.rs` (32L — mod list), `vanta-proxy/src/handlers/{anthropic,openai,mod}.rs` (thin wrappers → `state.process`), `vanta-proxy/src/inject.rs` (Protocol enum + shapes ambos lados), `vanta-proxy/src/error.rs` (109L — ProxyError non_exhaustive), `vanta-proxy/src/forward.rs` (1-120 — verbatim + failover), `vanta-proxy/src/sse_intercept.rs` (1-120 — data_events + Accumulated), `vanta-proxy/Cargo.toml` (sin nuevas deps), `vanta-proxy/tests/{prx12_compat.rs,fixtures/*}` (shapes reales).
- **Referencias hacia dentro (lo que el slice usa):** `serde_json::Value` únicamente. Nada de `AppState`, `AuthDb`, `Forwarder`. Cero imports del pipeline.
- **Referencias entrantes (quién usa lo nuevo):** nadie en slice 1 (módulo hoja). `lib.rs` declara `pub mod translate`. Tests `tests/prx11_translate.rs` + unit tests inline.
- **Veredicto:** ADITIVO puro — 2 archivos nuevos + 1 línea en `lib.rs`. Blast radius 0 sobre wire/pipeline. Rollback = revert commit. `unwrap/expect` solo en tests.

## Steps (~100 líneas cada uno)

- [x] **Step 0 — DISCOVERY:** tipo proxy, skills, blast radius, fixtures PRX-12, Spec table, este file. Sin código.
- [x] **Step 1 — RED:** `tests/prx11_translate.rs` (request/response/SSE/sanitize/guard/beta) que falla con E0432 (`translate.rs` inexistente). Verificar falla por razón correcta. ✅ E0432 confirmado.
- [x] **Step 2 — GREEN lib:** `src/translate.rs` request↔response + `pub mod translate` en `lib.rs`. Mínimo para pasar. ✅ 9/9.
- [x] **Step 3 — GREEN SSE+beta:** chunk mappers + beta helper + thinking mapping. Tests verdes. ✅ (mismo commit; 1 fix clippy `if_same_then_else` + `cargo fmt`).
- [x] **Step 4 — VERIFY+COMMIT:** `cargo test -p vanta-proxy` 0 failed + clippy 0 + fmt + commit solo-propio en develop. ✅ suite (lib 151+2 + 15 suites) 0 failed, clippy 0, fmt 0.

## Notas

- `ponytail:` fns puras sobre `Value`, sin tipos nuevos — structs tipados si el wiring (slice 2) lo exige.
- Gate D: feature-add con `pub fn` nuevos → Spec table arriba + slice aditivo sin wire = proceder sin `question` (registrado en GATES_EVALUADOS).
- Deuda ajena NO tocada: `M .opencode, M opencode.jsonc, ?? Investigacion-plan.md`, resto `git status`.
