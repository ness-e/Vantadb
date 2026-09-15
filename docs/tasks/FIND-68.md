# FIND-68 — `docs/api/PROXY.md` + config/env documentados (docs-only)

> **Campaign:** 6ab26f3f-cf16-4416-9255-c18cca0bcaf0 · **Wave:** Wave5 · **Ruta:** vanta-docs
> **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 · **Estado:** ⏳ IN PROGRESS
> **Branch:** develop · **Commit:** `docs: FIND-68`
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design + documentation-and-adrs, writing-guidelines, spec-driven-development (contractKeywords: proxy-docs, api-reference, config-env-docs, rust-doc-sync)

## Contrato

- `docs/api/PROXY.md` nuevo: 8 endpoints lógicos + 8 features opt-in + defaults + env documentados.
- Índice de docs (`docs/master-index.md`, tabla API Reference) enlaza PROXY.md.
- Regla 11: cada número/claim con fuente (ruta:línea).
- Scope: endpoints + features + config. NO tutorial, NO código Rust (0 líneas), NO ejemplo (config.toml ya existe).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-proxy/src/server.rs:741-772` (router) + `:774-840` (health/snapshot/session_advance) — 10 route registrations = 8 endpoints lógicos.
- `vanta-proxy/src/config.rs` (399 líneas, entero) — `ProxyConfig` 14 secciones, defaults de server/upstream/auth/mem_command/writeback/cache/report/cost.
- `vanta-proxy/config.toml` (19 líneas, entero) — solo `[server]` + `[upstream]` reales (D31).
- `vanta-proxy/src/routing.rs:56-95`, `redact.rs:1-63`, `context.rs:1-84`, `guardrails.rs:1-21`, `translate.rs:15-50` — defaults de las 5 features con config propia.
- `vanta-proxy/src/cache.rs:592-599` (env embed), `cost.rs:27-84` (PriceTable), `main.rs:18-22` (VANTA_PROXY_CONFIG).
- `docs/api/MCP.md:1-120` (modelo de formato: frontmatter + Getting Started + tablas) y `docs/master-index.md:65-86` (tabla API Reference donde enlazar).

**Referencias hacia dentro (qué cita el doc):** solo lectura como evidencia — server.rs, config.rs, config.toml, routing/redact/context/guardrails/translate/cache/cost/main.rs.

**Referencias entrantes:** `docs/master-index.md` (nuevo enlace en tabla API Reference). Ningún código referencia PROXY.md (doc nueva, sin blast radius de código).

**Veredicto:** docs-only puro. 2 archivos escritos (PROXY.md nuevo + 1 línea en master-index) + este task file. 0 líneas de Rust. Prohibidos intactos: `.opencode/`, `completions/`, tauri lock, FIND-67 (QUICKSTART), FIND-81 (vantadb-server/).

## Steps

- [x] Step 1 — DISCOVERY + PROXY.md + enlace master-index + task file
- [x] Step 2 — Verify mecánico + commit `docs:` + memoria + cierre

## Verify (2026-09-15, bash directa — `campaign_verify_cmd` con bug exit -1 conocido)

- `git diff --check` limpio (solo warning CRLF pre-existente en `completions/`, no tocado).
- `Test-Path docs/api/PROXY.md` = True.
- `rg -c "\.route\(" vanta-proxy/src/server.rs` = 10 (8 lógicos — coincide con el doc).
- `git status`: solo 3 archivos propios (PROXY.md + master-index.md + FIND-68.md); WIP ajeno intacto.
- 0 líneas Rust (docs-only).

## Spec (tabla de decisiones — doc nueva, sin Gate D: 0 símbolos públicos nuevos, solo documenta existentes)

| # | Decisión | Opción elegida | Evidencia |
|---|----------|----------------|-----------|
| 1 | Conteo "8 endpoints" | 8 lógicos (10 route registrations: models y count_tokens tienen forma plain + `{agent}/{spaceId}`) | `server.rs:758-770` |
| 2 | Las "8 features opt-in" | mem_command, cache, report, routing, redact, context, guardrails, translate (las 8 con `enabled: false` / endpoint vacío por defecto; cost tracking está ON por defecto → sección propia, no opt-in) | `config.rs:28-55` + defaults en cada módulo |
| 3 | Env vars documentadas | `VANTA_PROXY_CONFIG`, `VANTA_EMBED_BASE_URL`, `VANTA_EMBED_MODEL` (únicas `std::env::var` en vanta-proxy/src) + `RUST_LOG` vía `EnvFilter::try_from_default_env` (tracing, no config propia) | grep `std::env::var` = 3 hits + `main.rs:12-15` |
| 4 | Sin tutorial ni ejemplo nuevo | config.toml real ya existe y se cita; el doc referencia, no duplica | `vanta-proxy/config.toml:1-19` |
| 5 | Precios USD en tabla | 4 modelos + `__default__` fallback, con nota de stale (el propio código advierte) | `cost.rs:50-74` + `:32-38` |

## Checklist anti-drift (pre-mortem: el doc diverge del código al mes)

- [ ] Al añadir/quitar un `.route(` en `server.rs::router` → actualizar tabla Endpoints.
- [ ] Al añadir un campo a `ProxyConfig` (`config.rs:17-56`) → actualizar tabla Features/defaults.
- [ ] Al cambiar un `Default` en routing/redact/context/guardrails/translate/cache/cost → actualizar tabla Defaults.
- [ ] Al añadir `std::env::var` en `vanta-proxy/src/` → actualizar tabla Env.
- [ ] Al cambiar precios en `cost.rs::PriceTable::default` → actualizar tabla de precios.
- [ ] Verificación mecánica sugerida: `rg -c "\.route\(" vanta-proxy/src/server.rs` == filas de la tabla Endpoints.

## Fuentes (Regla 11 — toda afirmación del doc cita una de estas)

- server.rs:741-772 (router), :774-840 (handlers), config.rs:9-12/97-103/124-134/175-204/215-278, config.toml:1-19, routing.rs:81-95, redact.rs:17-20/54-63, context.rs:25-33/74-84, guardrails.rs:13-21/30-48, translate.rs:15-23/29-50, cache.rs:25-26/592-599, cost.rs:27-84/86-90, main.rs:11-22/40-50.

## Context Save Point

Step 1 hecho (PROXY.md + enlace + task file). Falta Step 2: `git diff --check`, Test-Path PROXY.md, recontar `.route(`, commit `docs:` solo 3 archivos propios, `campaign_memory_write` lesson, `campaign_update_task_state` completed, RESULTADO §7.
