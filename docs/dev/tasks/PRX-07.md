# PRX-07 — PII/secret redaction en egress

- **Estado:** ✅ COMPLETE (2026-09-10: redact.rs + wiring + prx07 12/12, suite 0 failed, clippy 0, fmt OK)
- **Plan:** `docs/dev/plans/2026-09-10-code.md` Task 10 · **Campaign:** 2c3d4e5f-6a7b-8c9d-0e1f-2a3b4c5d6e01
- **Archivos clave:** `vanta-proxy/src/redact.rs` (nuevo), pipeline pre-forward (`server.rs::process_inner`)
- **Contrato:** `cargo test -p vanta-proxy` 0 failed + test AWS keys/tokens/emails/regex ✅ + modos block/mask/log + clippy 0
- **Ruta:** vanta-worker · **Workflow:** feature-add (spec → implement → verify → review → accept → close)
- **SDP:** test-driven-development, incremental-implementation, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, security-and-hardening (+ base campaign-executor/progreso; `frontend-ui-engineering` descartada: sin `web/`)
- **Gate D:** no-disparado — owner aprobó plan 19 DO (Gate P 2026-09-10) + blast radius 4 archivos (<10) + símbolos `pub` solo en módulo nuevo aditivo (precedente PRX-03/PRX-06)

## Spec

| Decisión | Opción elegida | Por qué |
|---|---|---|
| Dónde engancha | Paso 5a en `process_inner`: post-`inject_into`, pre-cache (5b) | Lo que egresa es bytes finales post-inyección; máscara consistente con cache |
| Paths verbatim (`forward_raw`: sidequery, sin sesión) | NO toca redacción | Contrato "forward verbatim" + D29-solo-sesiones; documentado |
| Default | `enabled = false` (proxy transparente) | Invariante repo (cache/routing/cost-enforce); modo implícito si enabled = `mask` (pre-mortem: mask default vs FPs) |
| Detectores built-in | Scanners lineales hand-rolled, SIN regex: `AKIA[0-9A-Z]{16}`, `aws_secret`-adyacente 40ch, `sk-/ghp-/xox[bpas]-/Bearer` por prefijo, email heurístico `local@domain.tld` | ReDoS imposible por construcción (pre-mortem); sin dependencia nueva para defaults |
| Patrones custom (`patterns: Vec<String>`) | Vía crate `regex` (nueva dep, MIT/Apache-2.0 deny-limpia) con `size_limit(1<<20)` + `dfa_size_limit` acotado | Contrato exige "regex ✅" configurable; riesgo ReDoS acotado + doc "patrones lineales" |
| Cap de escaneo | `max_scan_bytes` default 2 MiB; body mayor → fail-open (sin hallazgos + warn) | DoS-bound (STRIDE-D); requests LLM ya acotados por servidor |
| Block response | `ProxyError::RedactionBlocked { kinds }` → 422 `redaction_blocked`, SOLO kinds, nunca valores | security-and-hardening: nunca exponer secretos en errores |
| Mask tokens | `[REDACTED_AWS_KEY]`, `[REDACTED_SECRET]`, `[REDACTED_TOKEN]`, `[REDACTED_EMAIL]`, `[REDACTED_CUSTOM]` | Sin eco de secreto en wire/logs |
| Log mode | `tracing::warn!` con kinds+counts, nunca valores | Nunca loguear sensitive data |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `lib.rs` (29L, registro `pub mod`), `error.rs` (75L, `ProxyError` + `IntoResponse`), `Cargo.toml` (33L, sin `regex`), `server.rs:240-399` (`process_inner` pasos 1-5c), `config.rs:1-120` (`ProxyConfig` + `#[serde(default)]`), `inject.rs::inject_into` (vía codegraph, 197-297), `tests/prx06_routing.rs:1-80` (patrón test: TOML inline + asserts).
- **Referencias hacia dentro (redact.rs usará):** `std`, `regex::RegexSet`-like compilado, `serde::Deserialize`, `crate::error::ProxyError`.
- **Referencias entrantes (lo que toco):** `server.rs::process_inner` (1 hook 5a) ← `forward_with_tool_loop`; `config.rs::ProxyConfig` (+1 campo `redact`) ← `main.rs`, `proxy_wire.rs`; `error.rs::ProxyError` (+1 variante) ← `auth.rs`, `server.rs`; `lib.rs` (+1 `pub mod redact`).
- **Veredicto:** aditivo puro; NO toca `wal.rs`/`vector/`/`storage/`, ni failover PRX-02, ni cost PRX-03, ni routing PRX-06, ni verbatim paths. TOML legacy compat vía `#[serde(default)]`. Riesgo: nueva dep `regex` → gate `cargo deny` en verify.

## Steps

- [x] **S1 RED:** `tests/prx07_redact.rs` — AWS key/email/token/custom-regex × modos block/mask/log → FAIL (módulo inexistente)
- [x] **S2 GREEN core:** `src/redact.rs` (tipos+detectores+`scan`/`apply`) + `pub mod redact` → lib tests ✅
- [x] **S3 wiring:** `regex` dep + `RedactConfig` en `ProxyConfig` + `ProxyError::RedactionBlocked` + hook 5a en `server.rs` → suite 0 failed
- [x] **S4 VERIFY+CLOSE:** `cargo test -p vanta-proxy` + clippy + fmt (+ deny por dep nueva) → review 3 ejes → commit solo-propio → plan inline

## Context Save Point

- 2026-09-10: DISCOVERY done; task file creado; IN PROGRESS. Siguiente: S1 RED.
- 2026-09-10: COMPLETE. S1 RED (E0432) → S2 GREEN (redact.rs ~330L, prx07 10/10) → S3 wiring (config+error+hook 5a+literals, wire tests 12/12) → S4 verify (suite 0 failed, clippy 0, fmt OK; deny advisories FAIL pre-existente RSA/jsonwebtoken+lru, licenses ok, sin regex). Review 3 ejes OK. Commit solo-propio en develop.
