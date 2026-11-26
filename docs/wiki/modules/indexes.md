---
title: "Módulo: indexes (`index`, `vector`, `text_index`, `tokenizer`)"
kind: concept
status: active
description: "Índices vectoriales (HNSW/IVF/ScaNN/DiskANN/flat), índice de texto BM25 y métricas de distancia: estructura, entradas y fuentes."
---

# Módulo: indexes (`index`, `vector`, `text_index`, `tokenizer`)

Responsabilidad: estructuras de búsqueda aproximada/exacta y su persistencia.

## Estructura

`[HECHO]` `src/index/` contiene: `graph/` (HNSW — `core.rs`, `prefetch.rs`, `types.rs`), `ivf.rs`, `scann.rs`, `diskann.rs`, `flat.rs`, `neighbor_index.rs`, `auto_tune.rs`, `refresh.rs`, `core.rs`, `distance/` (`kernels.rs`, `metrics.rs`, `mapper.rs`), `search/` (capas BFS/nearest/neighbors/pool), `port_impl.rs`. `src/index_port.rs` es la hoja neutral rompe-ciclos (precedente citado por `vantadb-ffi-core`). `[HECHO]` `VECTOR_INDEX_VERSION` se re-exporta (`src/lib.rs:195`); parámetros HNSW decididos en ADR-0005.

`[HECHO]` `src/vector/` añade `quantization.rs`, `transform.rs`, `governor.rs`; sparse vectors son tipo nativo `SparseVector` (ADR-0011/0019/0058; persistencia ADR-0032). `[HECHO]` `src/text_index.rs` (`pub(crate)`; spec pública [`TextTokenizerSpec`](../../../src/text_index.rs#L44) / [`TextIndexSpec`](../../../src/text_index.rs#L72) re-exportada, `src/lib.rs:219`) implementa el índice invertido BM25; `src/tokenizer.rs` tras `advanced-tokenizer` usa tantivy stemmer+stopwords (`Cargo.toml:145`,`:171`); diseño en [TEXT_INDEX_DESIGN](../../dev/architecture/TEXT_INDEX_DESIGN.md) y [ADVANCED_TOKENIZER](../../dev/architecture/ADVANCED_TOKENIZER.md). `[HECHO]` `src/scalar_index.rs` (`pub(crate)`) indexa campos escalares para filtros.

## Entradas públicas

`[HECHO]` OverRide de método de índice por búsqueda (`method`: `"ivf"|"scann"|"hnsw"|"flat"`) en bindings; mantenimiento `rebuild_index`/`reindex_hnsw_from_text`/`repair_text_index`/`audit_text_index[_deep]` ([BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md)). `[HECHO]` SIMD en kernels de distancia (AVX2/NEON) reportado por `hardware`/`hardware_profile`; prefetch mmap tras `prefetch_mode` (`src/config.rs:768-773`).

## Límites

`[INFERENCIA]` Recall/latencia concretas no se afirman aquí: la fuente de métricas es `docs/user/operations/BENCHMARKS.md` con entorno declarado. `[HECHO]` La imagen HNSW es una ruta de escritura cifrable (`src/lib.rs:56-59`).

## Fuentes

`src/index/` · `src/vector/` · `src/text_index.rs` · `src/tokenizer.rs` · `src/scalar_index.rs` · [BENCHMARKS](../../user/operations/BENCHMARKS.md) · ADR-0005/0011/0019/0032/0058
