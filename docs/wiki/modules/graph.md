---
title: "Módulo: graph (`graph`, `graphrag`, `gds`)"
kind: concept
status: active
description: "Grafo de propiedades, traversals, GraphRAG y algoritmos GDS: superficie pública y fuentes."
---

# Módulo: graph (`graph`, `graphrag`, `gds`)

Responsabilidad: grafo de propiedades nativo del motor — nodos/aristas, traversals y el pipeline GraphRAG.

## Estructura y entradas

`[HECHO]` `src/graph.rs` (`src/lib.rs:115`) expone [`GraphTraverser`](../../../src/graph.rs#L18) con `TraversalDirection` ([`:29`](../../../src/graph.rs#L29)) y traversals (`bfs_traverse` en `:66`, `dfs_traverse` en `:239`, `topological_sort` en `:263`); `src/edge_index.rs` (`pub(crate)`, L102) indexa aristas; `src/graphrag/` = `seed` → `expand` → `retrieve` → `context` orquestado por [`GraphRagPipeline`](../../../src/graphrag/pipeline.rs#L7) (`search` en `:106`); `src/gds.rs` (L113) algoritmos tipo graph-data-science; `src/accumulator.rs` aporta `GraphAccumulator` (`src/lib.rs:66`).

`[HECHO]` Superficie por binding (matriz W1, BINDINGS_NAMESPACES): `insert_node`/`get_node`/`delete_node`, `add_edge`/`remove_edge`, `graph_bfs`/`graph_dfs`/`graph_topological_sort`/`graph_is_dag`/`graph_degree`/`graph_filtered_traversal`, `graphrag_search` (seed→expand→retrieve→context, DIST-15). Python añade `graph_page_rank`, `graph_degree_centrality`, `graph_bfs_filtered`; solo Python carece de `remove_edge` — tabla de diferencias declarada.

`[HECHO]` El wire shape GraphRAG con ids u128 como strings decimales está fijado por test (`tests/graphrag_test.rs`) y verificado WASM↔Node (BINDINGS_NAMESPACES §GraphRAG evidence).

## Tests existentes

`[HECHO]` `tests/core/graph.rs`, `tests/core/temporal_edges.rs`, `tests/graphrag_test.rs` (`Cargo.toml [[test]]`).

## Límites

`[INFERENCIA]` PageRank/centralidad no están en bindings TS/WASM/Node (solo Python); el diseño GraphRAG vive en `docs/dev/graphrag/README.md`.

## Fuentes

`src/graph.rs` · `src/graphrag/` · `src/gds.rs` · `src/edge_index.rs` · `src/accumulator.rs` · [GRAPH_RAG](../../api/GRAPH_RAG.md) · [graphrag README](../../dev/graphrag/README.md) · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md)
