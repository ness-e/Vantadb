---
title: "Módulo: retrieval (`sdk::search`)"
kind: concept
status: active
description: "Pipeline de recuperación híbrida: vector, lexical, fusión RRF, MMR, multi-namespace, explain y snippets — estructura y fuentes."
---

# Módulo: retrieval (`sdk::search`)

Responsabilidad: la rama `src/sdk/search/` que convierte un `MemorySearchRequest` en hits ordenados.

## Estructura

`[HECHO]` Archivos: `mod.rs` (`search` en `:73`), `vector.rs` (ANN), `lexical.rs` (BM25), `hybrid.rs`, `fusion.rs` (RRF + `EntityBoostProvenance`, `get` en `:362`), `mmr.rs` (diversidad `MmrConfig`), `multi.rs` (cross-namespace), `page.rs` (`MemorySearchPage`), `explain.rs` (`SearchExplanation`), `snippet.rs`, `phrase.rs`, `sparse.rs`, `text_index.rs` (puente al índice textual), `audit.rs`, `debug.rs`, `debug_ops.rs`, `tests.rs`.

## Contratos

`[HECHO]` `search(MemorySearchRequest) -> Vec<MemorySearchHit>` con `score` mayor-es-mejor ([`src/sdk/search/mod.rs:73`](../../../src/sdk/search/mod.rs#L73); convención BINDINGS_NAMESPACES). `[HECHO]` La fusión RRF usa constante k=60 ([`RRF_K = 60.0` en `src/search_profile.rs:20`](../../../src/search_profile.rs#L20); decisión ADR-0006) y el perfil neutro vive en el mismo módulo (`src/lib.rs:143`). `[HECHO]` `MemorySearchPage` es el objeto página que produce la señal de abstinencia (solo viaja en HTTP/MCP — BINDINGS_NAMESPACES §SCH-07).

`[HECHO]` Parámetros del request verificados en la matriz v2: `query_vector`, `text_query`, `top_k`, `filters`, `distance_metric`, `method`, `explain`, `exclude_superseded`, `as_of_ms`, `valid_window`, `include_quarantined`, `min_confidence` (BINDINGS_NAMESPACES §SCH-07/W1). `[INFERENCIA]` `filter_ops` y `search_profile` están marcados como no expuestos en bindings (limitación core declarada) — en Rust sí son campos del request.

## Límites

Límite: orden estable, manejo de no-finitos y top-k exacto bajo filtros/expiración no se afirman aquí — la semántica vive en `src/sdk/search/{hybrid,fusion}.rs` y el contrato documentado es [scores.md](../../api/scores.md).

## Fuentes

`src/sdk/search/` · `src/search_profile.rs` · [scores](../../api/scores.md) · [GRAPH_RAG](../../api/GRAPH_RAG.md) · ADR-0006/0047 · [hybrid-search](../user/hybrid-search.md)
