# PRX-11 — Traducción Anthropic↔OpenAI fiel (slice 1: messages→OpenAI + responses + SSE mínimo)

> Plan: `docs/plans/2026-09-10-code.md` Task 19 · Wave6 SOLO (sin paralelo same-crate) · Ruta: vanta-worker
> Estado: ⏳ IN PROGRESS (slice 1) · Appetite: max 5d · Rama: `develop`
> SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (+systematic-debugging si RED falla)
> Estado: ✅ COMPLETE slice 1 (2026-09-10: RED E0432 → GREEN 9/9 → suite 0 failed → clippy/fmt 0 → commit solo-propio)
> Slice 2 (2026-09-10): 🟡 LIB-COMPLETE + WIRING-DEFERRED — thinking fidelity fina + `TranslateConfig`/`should_translate` (contract-first) en `translate.rs`, 16/16 tests, suite 0 failed, clippy/fmt 0. Hook `server.rs` + campo `ProxyConfig.translate` DEFERIDOS: `config.rs`/`cache.rs` bajo edición activa del wave-mate (PRX-09-embeddings, verificado `git status`: `M vanta-proxy/src/cache.rs`, `M docs/tasks/PRX-09.md`) — escribir `config.rs` violaría el write-scope (hook + translate.rs + tests propios) → BLOQUEO reportado, no FAILED.

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

---

## Slice 2 (2026-09-10) — wiring opt-in + thinking fidelity fina

> SDP: test-driven-development, api-and-interface-design, systematic-debugging, incremental-implementation, security-and-hardening (+ base: campaign-executor, source-driven-development, context-engineering, doubt-driven-development) · `campaign_discover_skills_v2` BUILD 8 skills, lifecycle only, sin keyword-mapped.
> Fuentes protocolo: https://platform.claude.com/docs/en/build-with-claude/extended-thinking (thinking blocks `{type,thinking,signature?}`, request config `{type:enabled|adaptive|disabled, budget_tokens?}`) + https://platform.openai.com/docs (chat completions `reasoning_content` — variante `reasoning` cubierta defensivamente, fail-open).

### Spec slice 2 (aditiva sobre slice 1)

| # | Decisión | Opciones | Evidencia / fuente | Elegido |
|---|----------|----------|-------------------|---------|
| S2-1 | Alcance ejecutable sin `config.rs` | A) lib + contract-first / B) tocar `config.rs` igual | `git status` verifica wave-mate editando `cache.rs`+`PRX-09.md`; write-scope = hook + translate.rs + tests propios; firma compartida → STOP+BLOQUEO | A: lib completa + gate puro; hook+`ProxyConfig.translate` DEFER con diff exacto abajo |
| S2-2 | Thinking request por variante | A) catch-all `_` / B) arms explícitos | Docs confirman `thinking{thinking,signature?}` + `redacted_thinking{data}` como variantes distintas | B: `Some("thinking")\|Some("redacted_thinking") => {}` explícito, drop documentado (sin contraparte OpenAI) |
| S2-3 | Reasoning response por variante | A) solo `reasoning_content` / B) + `reasoning` str/obj + signature | Gateways OpenAI-compat emiten `reasoning` además de `reasoning_content`; `signature` = preservación multi-turno (opaca, nunca inventada) | B: `extract_reasoning` → `(text, signature)`; desconocido → `(None,None)` fail-open |
| S2-4 | Request `thinking` config top-level | A) copiar / B) no copiar | Solo campos compartidos se copian (Spec slice 1); config `{type,budget_tokens}` es envelope Anthropic-only | B: no se copia (lock con test caracterización) |
| S2-5 | Gate opt-in | A) bool suelto / B) `TranslateConfig{enabled}` + `should_translate` | Precedente `GuardrailConfig`/`RedactConfig`: struct serde-defaulted + check puro; `TranslateSource` espeja `inject::Protocol` sin dependencia (hoja intacta) | B: Default off; `Anthropic→true` solo si enabled; OpenAI-cliente → verbatim (DEFER simétrico) |
| S2-6 | Seguridad thinking/PII | — | thinking puede traer PII; hook diferido corre post-redact (5a) → lo dropeado nunca sale | Documentado en código + task; sin gate nuevo (usa PRX-07 existente) |

### Steps slice 2

- [x] **Step 0 — DISCOVERY:** PRX-11.md + orden hooks (`process_inner`: 1b cost → 1c guardrails → 2 rate → 5a redact → optimizer → 5b cache → `forward_with_tool_loop`) + fixtures PRX-12 + `ProxyConfig` literales explícitos en ~15 tests (blast radius que confirma S2-1). Sin código.
- [x] **Step 1 — RED:** 6 tests nuevos en `tests/prx11_translate.rs` (thinking+signature, redacted, reasoning str/obj, signature, gate matrix) → E0432 `should_translate/TranslateConfig/TranslateSource` ✅ razón correcta.
- [x] **Step 2 — GREEN lib:** `TranslateConfig` + `TranslateSource` + `should_translate` + arms explícitos + `extract_reasoning`/`reasoning_to_thinking` + 1 test caracterización (`thinking` config no se reenvía). ✅ 16/16.
- [x] **Step 3 — VERIFY:** `cargo test -p vanta-proxy --tests -j 2` 0 failed (lib 155 + 15 suites, prx12 6/6 verbatim intacto) + clippy 0 + fmt ✅. (1 lock transitorio OS-32 en 1er run → retry verde; deuda conocida rustc-1.95/-j 2.)
- [x] **Step 4 — WIRING:** ❌ BLOQUEADO (evidencia abajo) → DEFER, sin FAILED. Commit solo-propio.

### Wiring DEFERRED — evidencia y próximo step exacto

**BLOQUEO:** el hook necesita `pub translate: TranslateConfig` en `ProxyConfig` (`config.rs`) + `Arc` en `AppState::from_engine` + inserción en `process_inner` — `config.rs` fuera del write-scope (wave-mate PRX-09-embeddings activo en `cache.rs`/`config.rs`) y el campo rompería ~15 literales `ProxyConfig{...}` explícitos de otros tasks (pipeline, proxy_wire, prx01/02/03/05/06/07/09/12/13, tool_loop). Cambiar firma compartida → STOP per brief. Default-verbatim intacto verificado: prx12 6/6 ✅.

**Diff diferido exacto (próxima invocación, tras clearance de `config.rs`):**
1. `config.rs`: `pub translate: crate::translate::TranslateConfig` en `ProxyConfig` (con doc opt-in, tras `guardrails`) — serde-defaulted, TOML legacy intacto.
2. `config.rs`: los ~15 literales de tests ganan `translate: Default::default(),` (1 línea c/u, mecánico).
3. `server.rs` `from_engine`: `translate: config.translate.clone().into()` (nuevo campo `pub translate: TranslateConfig` en `AppState`).
4. `server.rs` `process_inner`, punto 5d (tras optimizer, antes de 5b-cache): si `should_translate(&self.translate, map(protocol))` + body JSON parseable → `body = anthropic_to_openai(&v)` + protocolo efectivo OpenAI para forward; tras `forward_with_tool_loop`, si se tradujo + respuesta JSON buffered no-SSE → `openai_to_anthropic` + rebuild (SSE → passthrough, DEFER streaming-full; chunk mappers lib listos).
5. Test integración: enabled → upstream recibe shape OpenAI + cliente recibe shape Anthropic; disabled → byte-idéntico (extiende prx12).
6. Re-verify full + commit `feat: PRX-11-slice3 wiring opt-in translate`.

**NOTICED BUT NOT TOUCHING:** `M vanta-proxy/src/cache.rs`, `M docs/tasks/PRX-09.md` (wave-mate); `M desktop/...`, `M docs/tasks/DESKTOP-40-slice3.md`, `?? docs/tasks/DESKTOP-40-slice3.md`, `M .opencode`, `M opencode.jsonc`, `?? Investigacion-plan.md`, `D docs/plans/2026-09-10-code.md` (orquestador).

### Slice 3 ejecutado lead-inline (2026-09-10, subagentes abortados ×3 sin task_id)

- **Diseño ejecutado:** hook 5d en `process_inner` (tras optimizer, antes de 5b-cache) + `ProxyConfig.translate` + wire_path efectivo `/v1/chat/completions` (comparte caché con nativo OpenAI) + `map_translated_response` (SSE passthrough, JSON→Anthropic con content-length fijo, fail-open).
- **Desvío justificado vs diff:** sin campo en `AppState` (se lee de `self.config.translate`; evita romper literales AppState) + `wire_path` → `_wire_path` (target fijo, clippy).
- **Tests nuevos (server.rs mod tests):** gate off/openai/garbage/mapeo (4) + mapback passthrough/SSE/JSON/inválido (4).
- **Verify:** lib 163 + 15 suites 0 failed · clippy all-targets 0 · fmt 0 · hooks pre-commit.
- **Commit:** (este cierre).
