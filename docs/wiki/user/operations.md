---
title: "Operación y mantenimiento"
kind: howto
status: active
description: "Tareas operativas verificadas: métricas, mantenimiento de índices/WAL, TTL/GC, observabilidad y configuración."
---

# Operación y mantenimiento

## Métricas y salud

`[HECHO]` `operational_metrics()` en los 4 bindings (RSS, nodos HNSW, bytes lógicos, páginas mmap residentes); HTTP `GET /metrics` y `GET /api/v2/metrics`; `capabilities()` reporta `runtime_profile`, `persistence`, `vector_search`, `iql_queries`, `read_only` (BINDINGS_NAMESPACES §parity). Prometheus tras feature `prometheus` (`Cargo.toml:195`); stack en [OBSERVABILITY](../../user/operations/OBSERVABILITY.md) y [GRAFANA_SETUP](../../user/operations/GRAFANA_SETUP.md).

## Mantenimiento

`[HECHO]` `flush`, `compact_wal`, `compact_layout`, `rebuild_index`, `reindex_hnsw_from_text`, `repair_text_index`, `audit_text_index[_deep]` en bindings; HTTP `POST /api/v2/maintenance/{flushes,compactions,index-rebuilds}` + `DELETE .../expired-records` ([HTTP_API](../../api/HTTP_API.md)). `[HECHO]` TTL: `ttl_ms` por registro + `purge_expired()`; GC en [GC_TTL](../../user/operations/GC_TTL.md).

## Configuración y despliegue

`[HECHO]` `Config` gobierna límites de memoria, eviction, mmap, cuota por namespace y embeddings ([`src/config.rs:719`](../../../src/config.rs#L719)); referencia operativa en [CONFIGURATION](../../user/operations/CONFIGURATION.md), [PERFORMANCE_TUNING](../../user/operations/PERFORMANCE_TUNING.md), [DEPLOYMENT_GUIDE](../../user/operations/DEPLOYMENT_GUIDE.md), [hardening](../../user/operations/hardening.md) y [SECURITY](../../user/operations/SECURITY.md).

## Fuentes

`src/config.rs` · [HTTP_API](../../api/HTTP_API.md) · `docs/user/operations/` (índice [master-index](../../user/operations/master-index.md))
