# PRX-02 — Fallback multi-upstream + retries

> **Plan:** docs/plans/2026-09-10-code.md (Task 1) · **Campaign:** 2c3d4e5f-6a7b-8c9d-0e1f-2a3b4c5d6e01
> **Estado:** ✅ COMPLETO · **Ruta:** vanta-worker · **Rama:** develop
> **Appetite:** max 3d · **Contrato:** `cargo test -p vanta-proxy` 0 failed + failover 429/5xx→siguiente upstream ✅ + backoff exponencial ✅ + clippy 0
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design (+doubt-driven omitted: no auth/crypto; +frontend-ui-engineering omitted: no web/)
> **Wave:** Wave0 con MEM-69 y DESKTOP-40-slice2 (disjuntos; paralelo MAX 3)
> **Stop conditions:** 2 fallas mismo-error → Gate V; appetite >3d → shippear retries + DEFER health.
> **Deuda/WIP ajeno (NO tocar ni stagear):** M .opencode, M desktop/src-tauri/Cargo.lock, M opencode.jsonc, ?? Investigacion-plan.md, ?? docs/plans/2026-09-10-code.md

## Spec (gate mecánico spec-first — tabla de decisiones)

| # | Decisión | Opción elegida | Evidencia / por qué |
|---|----------|----------------|---------------------|
| 1 | Dónde vive la lista | `ProxyConfig.upstreams: Vec<UpstreamConfig>` + `#[serde(default)]`; `upstream` legacy intacto como fallback | grep: `ProxyConfig{...}` literales exhaustivos en 7 tests; campo nuevo con default = TOML viejo parsea igual (pre-mortem compat). `upstreams_resolved()` → `upstreams` si no-vacío, si no `vec![upstream.clone()]` |
| 2 | Qué es retryable | 429 o 5xx en respuesta; errores de transporte (timeout/unreachable) | Precedente crate: `UpstreamHealth::observe` (rate_limit.rs:194-199) ya clasifica 429/5xx como failure y <400 como éxito; `note_upstream` (server.rs:358) cuenta transporte como 503. Reuso misma partición |
| 3 | Retries de writes no-idempotentes | Failover aplica a TODOS los forwards del proxy (solo existen POSTs de inferencia) | El proxy solo forwardea llamadas de inferencia LLM (server.rs `forward_raw` POST + tool-loop POST): recompute side-effect-free desde el cliente, práctica estándar gateway (Portkey/Bifrost). Sin writes con estado cliente → pre-mortem "solo idempotentes" satisfecho por construcción |
| 4 | Backoff | Exponencial puro `min(base*2^attempt, max)`, base 100ms / max 2s, SIN jitter | Determinista → testeable con asserts exactos. Jitter = no-determinismo en tests; se añade si el tráfico real muestra thundering herd (`ponytail:`) |
| 5 | Health pasivo anti-flap | Contador por-upstream: ≥3 fallos consecutivos → skip; 1 éxito resetea; si TODOS skipeados → fail-open (intentar todos) | Histéresis sin timers ni tasks background (3 para entrar / 1 éxito para salir, espejo de `DEGRADED_ENTER_FAILURES=3` existente). Sin reloj = sin flapping por ventanas de tiempo |
| 6 | Superficie nueva | `forward()` single-upstream INTACTO (primitiva); nuevo `forward_with_failover()`; `Forwarder::new` usa timeout del primario | Blast radius mínimo: 2 call sites en server.rs cambian de `forward` a `forward_with_failover`. Sin romper firma existente |
| 7 | Sleep async | `tokio::time::sleep` → añadir feature `time` a tokio en vanta-proxy/Cargo.toml | tokio actual sin `time` (Cargo.toml:11). Feature, no crate nueva. Tests usan base 0-1ms → suite rápida |

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `vanta-proxy/src/config.rs` (235L), `vanta-proxy/src/forward.rs` (154L), `vanta-proxy/src/error.rs` (46L), `vanta-proxy/src/lib.rs`, `vanta-proxy/Cargo.toml`, `server.rs:340-459` (forward_raw + tool-loop), `rate_limit.rs:155-250` (UpstreamHealth), `tests/prx01_wiring.rs:1-100` (patrón E2E).
**Referencias hacia dentro (lo que toco usa):** `UpstreamConfig` (config.rs:152), `ProxyError::{UpstreamTimeout,UpstreamUnreachable,Forward}` (error.rs), `UpstreamHealth::observe` (rate_limit.rs:194), `AppState::forward_raw` + tool-loop (server.rs:373,409), `Forwarder::new` timeout (forward.rs:78).
**Referencias entrantes (quién usa lo que toco):** `config.upstream` en `main.rs:26-27`, `server.rs:98,104,382,434`, `handlers/auxiliary.rs:21,56` (`models_response(&UpstreamConfig)` — NO tocado); literales exhaustivos `UpstreamConfig{...}`/`ProxyConfig{...}` en tests (pipeline, proxy_wire, prx01, prx05, prx09, tool_loop, prx12) — se actualizan mecánicamente al añadir campo.
**Veredicto:** impacto ACOTADO — 2 src + 1 manifest + literales test mismo crate + 1 test nuevo. Fuera de radio: `auxiliary.rs` (lee `models`, intacto), `main.rs` (lee `upstream.url`, intacto vía fallback), vector/engine/storage (prohibidos, no tocados). Cobertura índice: `no_recorded_issue` ambos paths (best-effort; fuente leída directo = ground truth).

## Steps atómicos (~100 líneas c/u)

- [x] **Step 1 — Config multi-upstream:** `upstreams: Vec<UpstreamConfig>` + `upstreams_resolved()` + test compat TOML (viejo sin `upstreams` + nuevo con 2) + actualizar literales exhaustivos en tests. Verify: `cargo test -p vanta-proxy config` ✅ (7 tests config verdes 2026-09-10)
- [x] **Step 2 — Primitivas retry (TDD RED→GREEN):** `is_retryable_status(u16)->bool` + `backoff_delay(attempt, base_ms, max_ms)->Duration` puros en forward.rs + unit tests valores exactos (100,200,400,…,cap 2000; 429/500/503 true, 200/400/404 false). Verify: `cargo test -p vanta-proxy forward` ✅ (5 tests forward verdes)
- [x] **Step 3 — Failover + wiring:** `UpstreamHealthSet`/contador por índice + `forward_with_failover()` (ordena config, skip-if-unhealthy, backoff entre intentos, fail-open) + server.rs 2 call sites + feature `time` tokio. Verify: `cargo check -p vanta-proxy` + tests existentes ✅ (sin regresión: 118 unit + suites wire/prx01/prx05/prx09/prx12/tool_loop verdes)
- [x] **Step 4 — E2E + cierre:** `tests/prx02_failover.rs` (mock A=429 siempre + B=200 → proxy 200 de B; A=500→B; backoff acotado; todo-caído→último status) + `cargo test -p vanta-proxy` 0 failed + clippy 0 + fmt + commit solo propios + RESULTADO. ✅ 2026-09-10: 118 unit + 43 integración (prx02 4/4) 0 failed; clippy 0 (1 fix: field_reassign_with_default en test config); fmt package OK. ⛔ COMMIT BLOQUEADO: pre-commit hook corre `cargo fmt --all --check` workspace-wide y falla SOLO en `vanta-memory/tests/l1_batch.rs` (WIP ajeno MEM-69, unstaged) — mis archivos staged y verificados limpios; NO uso --no-verify sin orden explícita (Regla 1); NO toco archivo ajeno. Desbloqueo: (A) MEM-69 corre `cargo fmt` y avisa → reintentar commit; (B) usuario autoriza `--no-verify` con evidencia de verify scoped verde.

## Verify contrato

`cargo test -p vanta-proxy` (0 failed) · `cargo clippy -p vanta-proxy --all-targets -- -D warnings` (0) · `cargo fmt --check` · `campaign_verify_cmd` si funciona (bug exit -1 conocido → bash directa).
