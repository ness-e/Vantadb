# PRX-13 — Optimización contexto en tránsito

- **Estado:** ✅ COMPLETE (2026-09-10: context.rs + wiring + prx13 14/14, suite 0 failed, clippy/fmt 0)
- **Plan:** `docs/dev/plans/2026-09-10-code.md` Task 16 · **Campaign:** 2c3d4e5f-6a7b-8c9d-0e1f-2a3b4c5d6e01
- **Archivos clave:** pipeline pre-forward (`server.rs::process_inner`), `vanta-proxy/src/config.rs`
- **Contrato:** `cargo test -p vanta-proxy` 0 failed + test resize/modos ✅ + clippy 0
- **Ruta:** vanta-worker · **Workflow:** feature-add (spec → implement → verify → review → accept → close)
- **SDP:** test-driven-development, incremental-implementation, context-engineering, source-driven-development, api-and-interface-design (+ base campaign-executor/progreso; `doubt-driven-development`/`frontend-ui-engineering` descartadas: sin stakes de seguridad ni `web/`)
- **Gate D:** no-disparado — owner aprobó plan 19 DO (Gate P 2026-09-10) + blast radius ~5 archivos (<10) + símbolos `pub` solo en módulo nuevo aditivo (precedente PRX-03/06/07)

## Spec

| Decisión | Opción elegida | Por qué |
|---|---|---|
| Dónde engancha | Paso entre 5a (redact) y 5b (cache) en `process_inner` | Lo que se cachea/forwarda son bytes finales post-trim (consistente con redact); paths verbatim (sidequery, sin sesión) bypassan igual |
| Default | `enabled = false` (proxy transparente) | Invariante repo (cache/routing/cost-enforce/redact) |
| Modos (`ContextMode`) | `Performance` (default, safe) / `Balanced` / `Economy`, global `mode` + `modes_by_key: HashMap<String, ContextMode>` (por virtual key) | Contrato plan: "performance/balanced/economy por key"; pre-mortem: default es el modo safe |
| Estimador tokens | Reúso `crate::handlers::auxiliary::estimate_tokens` (chars/4 sobre strings JSON) | Ladder rung 2: ya existe en el crate, sin dep nueva |
| Resize | Si estimado > `max_input_tokens`: dropear turns más viejos, conservar system + últimos 2; si no alcanza, passthrough (fail-open) | Nunca romper el request por optimizar; stop condition fixtures |
| Guard tools/imágenes (pre-mortem) | Performance: body con tools o imágenes → passthrough intacto. Balanced: trima solo texto, conserva imágenes/tools. Economy: además dropea bloques imagen (placeholder `[image removed: context budget]`) y trunca `tool` results largos | Resize degrada tool_calls con imagen → safe default |
| Budget default | `max_input_tokens = 8000` | Conservador; configurable por TOML |
| Cap body | `max_scan_bytes` default 2 MiB; mayor → passthrough + warn (igual que redact) | DoS-bound; requests LLM ya acotados |
| Salida | `ApplyOutcome::Pass(Vec<u8>)` siempre (nunca bloquea ni 4xx) | Optimización ≠ gate; sin variante nueva en `ProxyError` |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `config.rs` (390L, `ProxyConfig` + `#[serde(default)]`), `server.rs:140-469` (`process_inner` pasos 1-5c + `AppState` campos), `redact.rs` (427L, patrón `Config`+compilado+`apply`+hook 5a), `error.rs` (93L, sin cambio necesario), `lib.rs` (30L, registro `pub mod`), `inject.rs:1-120` (`Protocol`, shapes `messages`/`tools`), `cost.rs:88-94` + `handlers/auxiliary.rs:35-47` (estimador existente), `tests/prx07_redact.rs` (patrón test + wire setup), `tests/prx12_compat.rs:1-130` (fixtures verbatim, `ProxyConfig` literal).
- **Referencias hacia dentro (context.rs usará):** `std`, `serde::Deserialize`, `serde_json::Value`, `bytes::Bytes`, `crate::inject::Protocol`, `crate::handlers::auxiliary::estimate_tokens`.
- **Referencias entrantes (lo que toco):** `server.rs::process_inner` (+1 hook entre 5a y 5b) + `AppState` (+1 campo `optimizer: Arc<ContextOptimizer>`) + `from_engine` (construcción); `config.rs::ProxyConfig` (+1 campo `context`) ← `main.rs`, `proxy_wire.rs`; `lib.rs` (+1 `pub mod context`); ~16 literales `ProxyConfig {` en 11 archivos de `tests/` (+1 campo `context: Default::default()` — colateral mecánico requerido, mismo crate).
- **Veredicto:** aditivo puro; NO toca `wal.rs`/`vector/`/`storage/`, ni failover PRX-02, ni cost, ni routing, ni redact, ni verbatim paths. TOML legacy compat vía `#[serde(default)]`. Sin dep nueva → sin gate `cargo deny`. Riesgo: fixtures PRX-12 (stop condition → default disabled = byte-identical + suite verde lo prueba).

## Steps

- [x] **S1 RED:** `tests/prx13_context.rs` — resize/modos (drop-oldest, guards tools/imágenes ×3 modos, per-key override, disabled transparente, non-JSON passthrough) → FAIL (E0432 módulo inexistente)
- [x] **S2 GREEN core:** `src/context.rs` (`ContextConfig`+`ContextMode`+`ContextOptimizer::apply`, trim con `keep_images` en Balanced) + `pub mod context` → 12/12 ✅
- [x] **S3 wiring:** `ContextConfig` en `ProxyConfig` + campo `optimizer` en `AppState`/`from_engine` + hook entre 5a y 5b + 16 literales tests + 2 wire tests → suite 0 failed (incl. prx12 6/6 verdes)
- [x] **S4 VERIFY+CLOSE:** `cargo test -p vanta-proxy` 0 failed + clippy 0 + fmt 0 → review 3 ejes → commit solo-propio → plan inline

## Context Save Point

- 2026-09-10: DISCOVERY done; task file creado; IN PROGRESS. Siguiente: S1 RED.
- 2026-09-10: COMPLETE. S1 RED (E0432) → S2 GREEN (context.rs + 12/12; fix trim `keep_images` en Balanced + fix asserts per-key) → S3 wiring (config+AppState+hook 5a/5b+16 literales+2 wire) → S4 verify (suite 0 failed: lib 148 + 13 suites incl. prx13 14/14 y prx12 6/6; clippy 0 tras fix `expect` en lib + 12 let-else en tests; fmt 0). Review 3 ejes OK. Commit solo-propio en develop.
