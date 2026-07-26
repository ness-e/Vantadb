# GraphRAG API

> **GraphRAG** is a formal pipeline: seed → expand → retrieve → generate context.

## Rust

```rust
use vantadb::graphrag::pipeline::GraphRagPipeline;

let pipeline = GraphRagPipeline::new();
let result = pipeline.search(&db, "documents", Some("query"), None)?;
println!("{}", result.context_text);
```

## Python

```python
client = vantadb_python.Client()
result = client.graphrag_search("namespace", query="my question")
print(result.context_text)
```

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| seed_k | 10 | Top-K vector search seeds |
| expansion_hops | 2 | BFS depth from seeds |
| max_expansion_nodes | 100 | Max nodes expanded |
| retrieval_top_k | 20 | Final top-K results |
