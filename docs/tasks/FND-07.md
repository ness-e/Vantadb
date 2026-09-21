# FND-07 — Regla de observabilidad real (prometheus) + probe endpoint

> **Wave:** P20a — Reglas de ingeniería · **Prio:** 🔴 · **Estado:** ✅ COMPLETED
> **Backlog:** `docs/Backlog.md:489` · **Plan:** wave-r2-r7-fnd

## Objetivo

Cerrar el gap entre `prometheus` declarado en `Cargo.toml` y lo consultable:
el server debe exponer `/metrics` con latencia de queries real (no placeholders).
Un dev de Show HN va a probeer `/metrics`.

## Alcance

- `vantadb-server/` (wrapper — el server real vive en `src/cli_server.rs` del core)
- `src/metrics/`
- `Cargo.toml` (solo si feature/dep falta — **no hace falta**, ya existen)
- `.opencode/rules/server-mcp.md`
- **NO:** benches/, src/index/, git commit, Backlog.md, plan files, AGENTS.md

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/cli_server.rs` (1444L) — router, handlers, middleware, tests
- `src/metrics/core/registry.rs` (1168L) — METRICS_REGISTRY + ~35 métricas
- `src/metrics/core/mod.rs` (916L) — record_* + export_metrics_text
- `src/metrics/mod.rs` — `pub use core::*`
- `vantadb-server/src/{main,server,lib}.rs` + `Cargo.toml` — re-export de `cli_server`
- `Cargo.toml` (root) — feature `prometheus = ["dep:prometheus"]` ya existe
- `.opencode/rules/server-mcp.md` (22L) — R-1, R-2 vigentes

**Referencias hacia dentro de lo que voy a tocar:**
- `src/cli_server.rs` → re-exportado por `vantadb-server/src/server.rs:1-4` (app, run, etc.)
- `metrics::QUERY_LATENCY` / `metrics::record_query_latency` (nueva) → usados desde `cli_server.rs` (nuevo call site); QUERY_LATENCY ya es `pub` via `metrics::core::*`
- `metrics::record_http_request` → ya llamado por `request_metrics_middleware` (línea 483)
- `.opencode/rules/server-mcp.md` → indexado por `.opencode/AGENTS.md` (lazy-loading tabla)

**Referencias entrantes (callers de lo que cambio):**
- `execute_query` → únicamente ruteado por `app_with_cors` (línea 146); sin callers externos
- `record_query_latency` (nueva) → nuevo call site en `execute_query`; patrón idéntico a `record_hybrid_query`
- `export_metrics_text` → `metrics_endpoint` (línea 458) + test existente

**Veredicto de impacto:** bajo. Cambios aditivos (nueva fn record + observación en hot path + regla doc). No rompe contrato público: `vantadb-server/src/server.rs` re-exporta los mismos símbolos. Sin feature `prometheus` la fn nueva es no-op (macro cfg-guarded), igual que `record_hybrid_query`.

## Análisis (gap)

| Item | Estado |
|---|---|
| Endpoint `/metrics` | ✅ existe (`src/cli_server.rs:147`, handler 457) |
| Datos reales vs placeholder | ✅ `export_metrics_text()` usa `METRICS_REGISTRY.gather()` + TextEncoder |
| Histograma de latencia de queries (`vanta_query_latency_ms`) | ❌ **registrado pero jamás observado** en path HTTP |
| Contador queries HTTP | ✅ `vanta_http_requests_total{route="/api/v2/query"}` (middleware) |
| Gauges colección/memoria | ✅ `record_memory_breakdown` desde engine init/maintenance + sdk/api |
| Regla observabilidad en server-mcp.md | ❌ falta |

## Steps

- [x] S1 DISCOVERY + task file (este archivo) — ✅ hecho en esta iteración
- [x] S2 ACT: `src/metrics/core/mod.rs` — agregar `record_query_latency(duration_ms)` (observa QUERY_LATENCY) + test
- [x] S3 ACT: `src/cli_server.rs` — `execute_query` mide y registra latencia de query
- [x] S4 ACT: `.opencode/rules/server-mcp.md` — R-3 regla must de métricas reales
- [x] S5 VERIFY: `cargo check -p vantadb-server` + `--features prometheus` + test endpoint /metrics (cfg prometheus) + test record_query_latency
- [x] S6 CIERRE: bloque RESULTADO

## Contrato (verify mecánico)

1. `cargo check -p vantadb-server` pasa (default, sin prometheus)
2. `cargo check -p vantadb-server --features prometheus` pasa (feature on)
3. Endpoint `/metrics` existe (grep route) y emite ≥3 métricas reales (histograma de latencia incluido) — test cfg(prometheus) verifica body contiene `vanta_query_latency_ms`, `vanta_http_requests_total`, `vanta_http_request_duration_ms`
4. Regla en `.opencode/rules/server-mcp.md`
5. Feature flag no rompe `--no-default-features` (record_query_latency es no-op sin feature)