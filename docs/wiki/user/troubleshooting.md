---
title: "Troubleshooting"
kind: howto
status: active
description: "Problemas frecuentes con causa verificable: locks, memoria, búsqueda lenta, índices y transportes."
---

# Troubleshooting

El runbook canónico es [docs/user/troubleshooting.md](../../user/troubleshooting.md); aquí los casos con causa verificada en código/docs.

## Lock de base de datos

`[HECHO]` Síntoma: segundo proceso/instancia no abre la misma ruta — single-writer por `.vanta.lock`. En MCP multi-instancia usar el fallback HTTP (ADR-0036). En configs MCP nunca usar `~` en `--db` (llega literal; MCP.md).

## Memoria alta

`[HECHO]` Controles reales: `memory_limit`, `rss_threshold` (rechazo `Error::ResourceLimit`, variantes en [`src/error.rs:137`](../../../src/error.rs#L137)), `mmap_hnsw`, `rebuild_index()`, `compact_wal()`, `purge_expired()` (`src/config.rs:748-788`; [memory-budget](../modules/memory-budget.md)). Diagnosticar con `operational_metrics()`.

## Resultados inesperados de búsqueda

`[INFERENCIA]` Antes de asumir bug: verificar campo leído (`score` ↑ vs `distance` ↓ según operación), `top_k` clampleado a `MAX_K` (observable), expiración TTL, y en WASM `nan_sanitization_count`/`metadata_drop_count` en `operational_metrics()`. Repair: `audit_text_index`/`repair_text_index`/`rebuild_index`.

## Transporte

`[HECHO]` `vantadb-node` está publicado en npm a 0.9.0 (verificado 2026-11-06) aunque conserva la clasificación EXPERIMENTAL declarada — si la instalación falla, revisar `engines`/plataforma antes que asumir ausencia del paquete. La capa cognitiva no existe en TS/Node/WASM (declarado, no bug). `query` IQL está ausente en la superficie Node (exclusión declarada en BINDINGS_NAMESPACES).

## Fuentes

[troubleshooting](../../user/troubleshooting.md) · [ERROR_HANDLING](../../api/ERROR_HANDLING.md) · [MCP](../../api/MCP.md) · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md)
