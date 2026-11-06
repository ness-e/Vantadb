---
title: "Búsqueda híbrida (vector + BM25 + RRF)"
kind: howto
status: active
description: "Cómo funciona la búsqueda híbrida: entradas, fusión RRF, score vs distance, filtros y parámetros verificados."
---

# Búsqueda híbrida (vector + BM25 + RRF)

## Modelo

`[HECHO]` Tres señales: ANN vectorial (HNSW por defecto; override `method`: `ivf`/`scann`/`hnsw`/`flat`), BM25 textual y fusión RRF k=60 ([`RRF_K = 60.0`](../../../src/search_profile.rs#L20)). La entrada es `MemorySearchRequest`; la salida `MemorySearchHit` con `score` mayor-es-mejor ([`src/sdk/search/mod.rs:73`](../../../src/sdk/search/mod.rs#L73)).

## Parámetros verificados

`[HECHO]` Por request: `query_vector`, `text_query`, `top_k`, `filters` (igualdad sobre metadata), `distance_metric` (`cosine`/`euclidean`), `method`, `explain`, `exclude_superseded`, `as_of_ms`, `valid_window`, `include_quarantined`, `min_confidence` (BINDINGS_NAMESPACES §W1/SCH-07). `[HECHO]` `top_k` se clampa a `MAX_K` por `clamp_top_k` (`vantadb-ffi-core`); `[INFERENCIA]` la señal del clamp depende del canal de logging de cada binding (en Python es `tracing::warn!`, visible solo si hay subscriber instalado) — no es una observabilidad universal garantizada.

## Reglas de lectura

`[HECHO]` `score` (híbrida) ↑ mejor; `distance` (ANN crudo `search_vector`) ↓ mejor — igual en los 6 transportes ([scores.md](../../api/scores.md)). `[HECHO]` `similar_to_key` busca desde una clave existente; `search_multi` cruza namespaces; `explain_memory_search`/`explainSearch` devuelve contribuciones BM25/fusión. `[INFERENCIA]` MMR (`MmrConfig`) existe en el SDK Rust; su exposición por binding varía — verificar en BINDINGS_NAMESPACES antes de asumirlo.

## Rendimiento

`[HECHO]` `search_batch*` solo existe en Python. Cualquier cifra de latencia/recall/speedup pertenece a [BENCHMARKS](../../user/operations/BENCHMARKS.md) con su entorno declarado — no extrapolar ni reimprimir aquí.

## Fuentes

`src/sdk/search/` · [scores](../../api/scores.md) · [EMBEDDINGS](../../api/EMBEDDINGS.md) · [BENCHMARKS](../../user/operations/BENCHMARKS.md) · [tutorial híbrido](../../user/tutorials/04-hybrid-search-basics.md)
