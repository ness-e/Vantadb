---
title: "Módulo: memory-budget (`governor`, `eviction`, `gc`)"
kind: concept
status: active
description: "Presupuesto de memoria: governor, backpressure por RSS, políticas de evicción ponderada/decaimiento bayesiano y GC — estructura y fuentes."
---

# Módulo: memory-budget (`governor`, `eviction`, `gc`)

Responsabilidad: contener el consumo de memoria del motor — límites, presión, evicción y recolección.

## Estructura

`[HECHO]` `src/governor.rs` público ([`ResourceGovernor`](../../../src/governor.rs#L24), `src/lib.rs:114`), `src/memory_governor.rs` (`pub(crate)`, L125), `src/vector/governor.rs` (presupuesto del índice vectorial), `src/eviction.rs` ([`EvictionPolicy`](../../../src/eviction.rs#L18) con scoring ponderado y [`BayesianDecay`](../../../src/eviction.rs#L93); feature `bayesian_decay`, `Cargo.toml:183`), `src/gc.rs` (L112 — recolección/TTL housekeeping).

## Contratos

`[HECHO]` Controles en `Config`: `memory_limit` (bytes), `rss_threshold` (fracción 0.0–1.0 que dispara rechazo `Error::ResourceLimit` en escrituras; 0.0 desactiva), `eviction_weight_{hits,confidence,importance,recency}` (defaults 1.0/2.0/3.0/1.0) y `eviction_ratio` (0.20) (`src/config.rs:748-788`). `[HECHO]` La observabilidad llega por `operational_metrics()` — `process_rss_bytes`, `hnsw_nodes_count`, `hnsw_logical_bytes`, `mmap_resident_pages` — en los cuatro bindings (BINDINGS_NAMESPACES) y `GET /api/v2/metrics` en HTTP.

`[HECHO]` TTL: registros con `ttl_ms` expiran y se excluyen de búsqueda; `purge_expired()` los elimina físicamente (superficie en todos los bindings; endpoint `DELETE /api/v2/maintenance/expired-records`). Operación y métricas en [MEMORY_TELEMETRY](../../user/operations/MEMORY_TELEMETRY.md) y [GC_TTL](../../user/operations/GC_TTL.md).

## Tests existentes

`[HECHO]` `tests/memory/backpressure.rs`, `tests/memory/eviction.rs`, `tests/memory/mmap_hnsw.rs` (feature `cli`), `tests/memory_telemetry.rs`, `tests/memory_brutality.rs`, `tests/storage/gc.rs`, bench `memory_budget` (`Cargo.toml`).

## Límites

Límite: no se afirman umbrales efectivos ni la relación estimación lógica vs RSS real — comprobar con `operational_metrics()` en vivo antes de operar.

## Fuentes

`src/{governor,memory_governor,eviction,gc}.rs` · `src/vector/governor.rs` · `src/config.rs:748-788` · [MEMORY_TELEMETRY](../../user/operations/MEMORY_TELEMETRY.md) · [PERFORMANCE_TUNING](../../user/operations/PERFORMANCE_TUNING.md)
