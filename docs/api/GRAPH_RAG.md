# GraphRAG API

> **GraphRAG** is a formal pipeline: seed → expand → retrieve → generate context.
>
> **Binding availability: Rust only.** GraphRAG is implemented in the core SDK
> (`src/graphrag/`, `VantaEmbedded::graphrag_search`) and is **not** exposed by
> any binding yet — there is no `graphrag_search` method on the Python, WASM,
> TypeScript, or Node bindings.

## Rust

GraphRAG runs through the embedded SDK handle (`VantaEmbedded`). The default
pipeline configuration (`seed_k=10`, `expansion_hops=2`, `max_expansion_nodes=100`,
`retrieval_top_k=20`) is available as a convenience method:

```rust
use vantadb::VantaEmbedded;

let path = std::env::temp_dir().join(format!("vantadb-graphrag-{}", std::process::id()));
let db = VantaEmbedded::open(&path).expect("open database");

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
use vantadb::VantaEmbedded;
use vantadb::graphrag::pipeline::GraphRagPipeline;

let path = std::env::temp_dir().join(format!("vantadb-graphrag-{}", std::process::id()));
let db = VantaEmbedded::open(&path).expect("open database");

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
text query plus an optional query vector (either may be `None`).

## Python

> **Not implemented.** The Python bindings (`vantadb_py`) do **not** expose
> GraphRAG — `VantaDB` has no `graphrag_search` method (tracked as pending:
> `examples/python/graphrag_pipeline.py`). Use the Rust SDK for GraphRAG today.

The Python entrypoint that *does* exist is the `VantaDB` class:

```python
from vantadb_py import VantaDB

db = VantaDB(":memory:", backend="memory")
db.put("agent/main", "task-1", "organize the backlog", vector=[1.0, 0.0, 0.0])
hits = db.search_memory("agent/main", [0.9, 0.1, 0.0], top_k=5)
print(hits[0].payload)
```

Until GraphRAG lands in Python, the closest available primitives are the graph
traversal methods on `VantaDB` (`graph_bfs`, `graph_dfs`, `graph_page_rank`,
`graph_degree_centrality`).

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| seed_k | 10 | Top-K vector search seeds |
| expansion_hops | 2 | BFS depth from seeds |
| max_expansion_nodes | 100 | Max nodes expanded |
| retrieval_top_k | 20 | Final top-K results |
