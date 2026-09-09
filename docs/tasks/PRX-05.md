# PRX-05 — Model discovery + endpoints auxiliares

> **Plan:** docs/plans/2026-09-09-backlog.md (Task 4, Wave1)
> **Estado:** ⏳ IN PROGRESS
> **Branch:** develop
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (+ progreso/ponytail base; frontend-ui-engineering excluida: sin `web/`)

## Contrato

`cargo test -p vanta-proxy` 0 failed + `GET /v1/models` + `count_tokens` + beta headers ✅ + `cargo clippy -p vanta-proxy -- -D warnings` 0

## Spec (gate mecánico spec-first)

| # | Decisión | Opción elegida + evidencia |
|---|----------|---------------------------|
| 1 | Fuente de la lista de modelos | **Derivar de config** (`[upstream] models = [...]` en TOML → `UpstreamConfig.models: Vec<String>`, serde default `[]`). Evidencia: pre-mortem del plan ("hardcodeada diverge → derivar de config"); grep confirma que hoy no existe ningún `models` en `vanta-proxy/` |
| 2 | Shape `GET /v1/models` | **OpenAI List Models**: `{"object":"list","data":[{"id","object":"model","created","owned_by"}]}`. Fuente: `platform.openai.com/docs/api-reference/models/list` (picker Claude Code con `CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1` consume este shape) |
| 3 | Rutas con prefijo agente | Registrar **ambas**: `GET /v1/models` y `GET /{agent}/{spaceId}/v1/models` (mismo handler). Precedente: `chat/completions` y `messages` usan prefijo; `responses` usa plana. Discovery llama a la plana; clientes gateway al prefijo |
| 4 | `count_tokens` | **Estimación local, sin upstream**: `POST /v1/messages/count_tokens` y `POST /{agent}/{spaceId}/v1/messages/count_tokens` responden `{"input_tokens": N}` con `N = ceil(chars/4)` sobre textos del body, mín 1. Sin nueva dependencia (ponytail: no tiktoken). Fuente shape: `docs.anthropic.com` (Count Tokens API). Local = funciona offline en tests, nunca 404 (litellm#13252) |
| 5 | Beta headers | `anthropic-beta` + `anthropic-version` ya pasan por `filter_headers` (solo hop-by-hop se strippean). **Sin cambio de forward**; test de passthrough verbatim + idempotencia PRX-04 (re-inject estable con beta headers presentes). Fuente: `docs.anthropic.com` (beta headers) + `forward.rs:16-32` |
| 6 | Auth en endpoints nuevos | **Sí, `x-vanta-user-key` obligatoria** (D34: sin modo abierto). `GET /v1/models` y `count_tokens` autentican igual que el resto del wire |
| 7 | `count_tokens` no toca memoria | No session/inject/capture: respuesta local directa tras auth (como `mem:` commands y `health`) |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `server.rs` (567L), `forward.rs` (154L), `config.rs` (208L), `error.rs` (46L), `inject.rs` (531L), `handlers/{mod,openai,anthropic,responses}.rs`, `lib.rs`, `tests/proxy_wire.rs` (502L), `Cargo.toml`, `config.toml`
- **Referencias hacia dentro (lo nuevo referencia):** `handlers::aux` → `config::ProxyConfig/UpstreamConfig`, `server::AppState`, `error::ProxyError`; `router()` suma 4 rutas; `UpstreamConfig` suma campo `models`
- **Referencias entrantes (quién usa lo tocado):** `router()` ← `main.rs` + 4 suites tests (`proxy_wire`, `pipeline`, `prx01_wiring`, `tool_loop`); `UpstreamConfig` ← `forward.rs`, `server.rs`, tests. Campo serde-`default` → structs literales existentes compilan sin cambio
- **Veredicto:** ADITIVO puro (1 archivo nuevo `handlers/aux.rs` + 4 líneas router + 1 campo config + ejemplo en `config.toml`). Cero cambios en `forward.rs`/`inject.rs`. Riesgo bajo; regresión cubierta por suite existente (125+118 tests PRX-04/08)

## Steps

- [x] **S1:** `UpstreamConfig.models` + `GET /v1/models` (plana + prefijo) + tests unitarios/integración
- [x] **S2:** `POST count_tokens` (plana + prefijo) + estimador local + tests
- [x] **S3:** test beta-headers passthrough + idempotencia PRX-04 + verify full + commit + memoria + cierre

## Estado

✅ COMPLETO — 134 tests 0 failed (104 lib + 4 aux unit + 5 prx05_aux + resto suites) + clippy 0 + fmt 0.
Doubt (TDD-RED como disproof + tests 401/shape/passthrough): sin hallazgos accionables; cross-model skipped (contexto pipeline no-interactivo).
NOTICED BUT NOT TOUCHING: suites existentes construían `UpstreamConfig` literal (5 sitios, fix 1-línea aplicado por necesidad de compilación — no refactor).

## Context Save Point

(nada pendiente — tarea completa)
