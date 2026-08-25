# Server & MCP — Reglas

> **Scope:** `vantadb-server/` (`src/server.rs`, `main.rs`, `lib.rs`), `vantadb-mcp/` (`src/lib.rs` — MCP stdio)
> **No tocar aquí:** bindings Python (`python-bindings.md`), JS/WASM (`js-ecosystem.md`), API pública del core (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-011, INV-003

## Reglas

### R-1: serverInfo/versiones coherentes entre doc y código

- **Must:** mantener `serverInfo.name`/`version` de `vantadb-mcp` (`src/metadata.rs`) y los recursos/tools documentados (MCP.md) sincronizados con la implementación real.
- **Must not:** documentar tools que no existen (`query` vs `query_lisp`) ni versiones de protocolo imaginarias.
- **Por qué:** la auditoría INV-011 detectó drift entre MCP.md y el código (tool `query` inexistente, `schema://` fantasma, versión 0.1.5 vs protocolo real 2024-11-05).

### R-2: Concurrencia de handlers vía semáforo + `spawn_blocking`

- **Must:** limitar concurrencia con `tokio::sync::Semaphore` (`acquire_owned()` en `vantadb-mcp`) y ejecutar el trabajo del motor en `tokio::task::spawn_blocking`.
- **Must not:** bloquear el event loop del servidor con trabajo síncrono del motor ni inventar límites con mutex síncronos.
- **Por qué:** ver `concurrency-async.md` R-3 y R-7 (INV-003) — los handlers MCP son el mismo patrón que el HTTP server.

### R-3: Observabilidad real en `/metrics` — métricas alimentadas, no placeholders

- **Must:** todo endpoint público expone métricas REALES vía `GET /metrics` (registry `src/metrics/core/registry.rs` + `export_metrics_text`), alimentadas desde el hot path del server: `vanta_query_latency_ms` (histograma p50/p95/p99 vía `metrics::record_query_latency` en `execute_query`), `vanta_http_request_duration_ms`/`vanta_http_requests_total` (middleware), `vanta_records_imported`/`vanta_records_exported` (ingestión).
- **Must not:** registrar métricas que nadie observa, ni exponer contadores/histogramas vacíos, ni devolver texto placeholder/`String::new()` cuando el feature `prometheus` está activo.
- **Por qué:** FND-07 — un dev de Show HN va a probeer `/metrics`; un histograma de latencia registrado pero jamás observado (como `QUERY_LATENCY` antes de FND-07) hace que el probe devuelva cero datos reales y el endpoint parezca decorativo.

<!-- Referencias cruzadas: → ver concurrency-async.md, api-contract.md, release-ci.md -->
