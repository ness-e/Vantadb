---
title: "Paridad entre transportes"
kind: reference
status: active
description: "Qué se promete igual entre Rust, Python, WASM, TS, Node, HTTP y MCP: matrices verificadas, runner de conformidad y divergencias declaradas."
---

# Paridad entre transportes

Audiencia: quien escribe código multi-binding o audita promesas cross-SDK. La autoridad es [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md); esta página resume lo verificado y enlaza.

## Convenciones alineadas

`[HECHO]` Relevance field: `score` (mayor-es-mejor) en híbrida/memoria en los 6 transportes; `distance` (menor-es-mejor) reservado al ANN crudo — tabla por transporte verificada (W1/API-02; naming canónico ADR-0047). `[HECHO]` `u128` ids: decimal string en JSON/Node/WASM, `bigint[]` en traversals TS, `int` nativo en Python. `[HECHO]` Casing por payload (Gate P): snake_case interno Rust/Python, camelCase público TS/JS y payloads JSON objetivo, kebab-case CLI y nombres de tools MCP.

## Matriz de capacidad (resumen normativo)

`[HECHO]` De la tabla W1 + diferencias declaradas: `graph_page_rank`/`graph_degree_centrality`, `put_batch_raw`, `search_batch*`, `hardware_profile` son **Python-only**; `bulk_import*` falta en TS; `remove_edge` y `sparse_vector` faltan en Python; `filter_ops`/`search_profile` no cruzan a bindings (limitación core declarada); Node no expone IQL `query` ni ANN crudo; la capa cognitiva `memory_capture`/`memory_recall` existe solo en Python (DIST-02); `recover_archived_nodes` (wiki) está en Python/WASM/TS.

`[HECHO]` Señal de abstinencia (`abstained`): solo HTTP `SearchPageV2` y el envelope MCP — los cuatro bindings devuelven arrays sin página; es una divergencia **declarada** con tracking (SCH-07).

## Conformidad mecánica (DIST-17)

`[HECHO]` Un escenario canónico (`tests/parity/scenario.json`: put×3, hybrid search, add_edge×2 + BFS, IQL) está diseñado para correr en Python, WASM y Node produciendo `target/bindings-parity/<binding>.json`; el comparador `dev-tools/parity-compare.mjs` exige hash idéntico por paso (normalización: ids decimales, hits ordenados score-desc/id-asc con 6 decimales, timestamps excluidos). El gate de CI es el job `parity` de [`.github/workflows/ci-fleet.yml`](../../../.github/workflows/ci-fleet.yml#L191-L286) (líneas 191-286; comparador en `:279`); su resultado no se ejecutó en esta sesión. `[HECHO]` La rama IQL cubre solo Python+WASM por la exclusión declarada de la superficie Node.

## Regla para añadir superficie

`[HECHO]` Una capacidad W1 nueva debe aterrizar en los cuatro bindings con fila en la matriz y evidencia same-PR (BINDINGS_NAMESPACES §W1). Criterios de promoción de sub-clientes: superficie SDK estable, compila a wasm32, wasm-safe (sin `std::fs`/`SystemTime::now`/`reqwest::blocking`), solo-aditivo, matriz+docs en el mismo PR.

## Fuentes

[BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md) · [SEARCH_PARITY](../../api/SEARCH_PARITY.md) · [scores](../../api/scores.md) · [`tests/parity/scenario.json`](../../../tests/parity/scenario.json) · [`dev-tools/parity-compare.mjs`](../../../dev-tools/parity-compare.mjs) · [`vantadb-ffi-core/src/lib.rs`](../../../vantadb-ffi-core/src/lib.rs)
