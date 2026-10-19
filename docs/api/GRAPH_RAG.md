---
title: GraphRAG API
kind: reference
status: active
description: GraphRAG (seed → expand → retrieve → context) is exposed by the Rust SDK and all four bindings — Python, WASM, TypeScript and Node — with one canonical wire shape
tags: [vantadb, api, graphrag, retrieval]
---

# GraphRAG API

> **GraphRAG** is a formal pipeline: seed → expand → retrieve → generate context.
>
> **Binding availability: Rust + Python + WASM + TypeScript + Node** (DIST-15,
> 2026-10-04). The canonical method name is `graphrag_search` (Rust / Python /
> WASM JS) and `graphragSearch` (TypeScript / Node), and every surface returns
> the same wire shape described in [Wire shape](#wire-shape) below.
>
> **Naming (ADR-0047 anti-stutter):** `Embedded` is canonical (legacy
> `VantaEmbedded` alias removed in 0.6.0, AST-010).

## Rust

GraphRAG runs through the embedded SDK handle (`Embedded`). The default
pipeline configuration (`seed_k=10`, `expansion_hops=2`, `max_expansion_nodes=100`,
`retrieval_top_k=20`) is available as a convenience method:

```rust
use vantadb::Embedded;

let path = std::env::temp_dir().join(format!("vantadb-graphrag-{}", std::process::id()));
let db = Embedded::open(&path).expect("open database");

// Default pipeline configuration:
let result = db
    .graphrag_search("documents", Some("query"), None)
    .expect("graphrag search");
println!("{}", result.context_text);

db.close().expect("close database");
let _ = std::fs::remove_dir_all(&path);
```

For custom settings, construct `GraphRagPipeline` directly (all fields are public):

```rust
use vantadb::Embedded;
use vantadb::graphrag::pipeline::GraphRagPipeline;

let path = std::env::temp_dir().join(format!("vantadb-graphrag-{}", std::process::id()));
let db = Embedded::open(&path).expect("open database");

let pipeline = GraphRagPipeline {
    seed_k: 20,
    expansion_hops: 3,
    max_expansion_nodes: 200,
    retrieval_top_k: 30,
};
let result = pipeline
    .search(&db, "documents", Some("query"), None)
    .expect("graphrag search");
println!("{}", result.context_text);

db.close().expect("close database");
let _ = std::fs::remove_dir_all(&path);
```

`search` takes the embedded handle, the namespace to search, and an optional
text query plus an optional query vector (either may be `None`). The pipeline
itself is unchanged across bindings — the bindings call
`Embedded::graphrag_search` with the default configuration.

## Wire shape

Every binding returns the same object — snake_case, one casing per payload —
pinned by `tests/graphrag_test.rs::graphrag_result_serializes_with_u128_ids_as_decimal_strings`:

```json
{
  "nodes": [
    { "id": "123", "content": "vector database for agents", "score": 0.6, "hop_distance": 0 }
  ],
  "edges": [
    { "source": "123", "target": "456", "label": "uses" }
  ],
  "context_text": "## Relevant Nodes\n- Node 123 (score: 0.6000, hop_distance: 0)\n  vector database for agents",
  "stats": {
    "seeds_found": 1,
    "nodes_expanded": 2,
    "total_candidates": 3,
    "expansion_hops_used": 1
  }
}
```

- `id` / `source` / `target` are u128 ids: **decimal strings** on the JSON
  transports (WASM / TypeScript / Node) and **native ints** in Python (API-01
  convention — u128 exceeds `Number.MAX_SAFE_INTEGER`).
- `score` is the combined ranking
  (`0.6·seed + 0.3·hop_boost + 0.1·degree_factor`) — higher is better.
- `context_text` is the LLM-ready block (`## Relevant Nodes` plus `## Graph
  Relationships`); it is empty when no seeds are found.
- At least one of `query` / `query_vector` should be provided; both may be
  combined (hybrid seeds).

## Python

```python
from vantadb import Client

db = Client(":memory:", backend="memory")
rec_a = db.put("docs", "a", "vector database for agents", vector=[0.1, 0.2, 0.3])
rec_b = db.put("docs", "b", "graph expansion uses edges", vector=[0.2, 0.3, 0.4])
db.add_edge(rec_a.node_id, rec_b.node_id, "uses")

result = db.graphrag_search("docs", query="vector database")
print(result["context_text"])  # dict: nodes / edges / context_text / stats
```

`graphrag_search(namespace, query=None, query_vector=None) -> dict` — GIL
released (pure Rust compute); `query_vector` accepts a list, a NumPy array, or
a `Vector`. A query vector above `MAX_VEC_DIM` (10 000) raises
`ValidationError`. Smoke: `vantadb-python/tests/test_graphrag.py`.

## WASM / TypeScript

```ts
// WASM binding (vantadb-wasm) — snake_case method name:
const result = db.graphrag_search("docs", "vector database");

// TypeScript wrapper (vantadb-ts) — camelCase:
const result2 = db.graphragSearch("docs", "vector database");

// Native backend (vantadb/native → vantadb-node):
const result3 = await nativeDb.graphragSearch("docs", "vector database");
```

`graphragSearch(namespace, query?, queryVector?)` returns `GraphRagResult`
(see `vantadb-ts/src/types.ts`). The WASM and Node surfaces emit ids as decimal
strings. Smoke: `vantadb-ts/src/__tests__/graphrag.test.ts` (includes a
WASM↔Node parity assertion on `context_text` and node ids).

## Node

```js
const db = await VantaDb.connect(":memory:");
const recA = await db.put({ namespace: "docs", key: "a", payload: "vector database for agents", vector: [0.1, 0.2, 0.3] });
const recB = await db.put({ namespace: "docs", key: "b", payload: "graph expansion uses edges", vector: [0.2, 0.3, 0.4] });
await db.addEdge(String(recA.node_id), String(recB.node_id), "uses");

const result = await db.graphragSearch("docs", "vector database", undefined);
console.log(result.context_text);
```

`graphragSearch(namespace, query?, queryVector?)` runs on a blocking thread
(`spawn_blocking`); an oversized query vector is rejected at the boundary.
Smoke: `vantadb-node/tests/graphrag.test.ts`.

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| seed_k | 10 | Top-K vector search seeds |
| expansion_hops | 2 | BFS depth from seeds |
| max_expansion_nodes | 100 | Max nodes expanded |
| retrieval_top_k | 20 | Final top-K results |

Custom configurations are Rust-only (`GraphRagPipeline`); the bindings expose
the default pipeline. Cross-binding parity matrix row:
[`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md#w1-parity-matrix-api-02--method--signature-across-the-4-bindings).
