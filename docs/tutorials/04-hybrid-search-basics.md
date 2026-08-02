---
title: "Hybrid Search with VantaDB"
status: active
tags: [vantadb, tutorial, guide, search, hybrid, bm25]
last_reviewed: 2026-08-02
aliases: []
---

# Hybrid Search with VantaDB

Vector search finds *meaning*; keyword search finds *exact terms*. Most real queries need both. VantaDB fuses HNSW vector search with BM25 lexical search in a single call, so you get semantic recall *and* precise term matching without running two systems.

In this tutorial you'll learn the four search modes of `search_memory()`:

1. Vector-only search (semantic)
2. Keyword search (BM25 via `text_query`)
3. Hybrid search (vector + BM25 fusion)
4. Filtered search (metadata equality filters)

## Setup

```bash
pip install vantadb-py
```

We'll use deterministic embeddings so the example runs offline:

```python
from vantadb_py import VantaDB

# Use a real embedding model in production; this is a demo stand-in.
def embed(text: str, dim: int = 8) -> list[float]:
    import hashlib
    out = []
    for i in range(dim):
        h = hashlib.sha256(f"{text}:{i}".encode()).digest()
        out.append(int.from_bytes(h[:4], "big") / 2**32)
    return out

db = VantaDB(":memory:", backend="memory")

docs = [
    ("d1", "VantaDB is an embedded vector database written in Rust."),
    ("d2", "Hybrid search combines BM25 keyword scoring with HNSW vector search."),
    ("d3", "The Python SDK ships migration tools for ChromaDB and LanceDB."),
    ("d4", "Graph edges let you build knowledge graphs over your records."),
]

for key, text in docs:
    db.put("docs", key, text, vector=embed(text))

print(f"Indexed {len(docs)} documents")
```

`VantaDB(":memory:", backend="memory")` creates a throwaway in-memory database — handy for experiments. Use a path like `VantaDB("app.db")` for persistence.

## 1. Vector-only search (semantic)

Pass a query vector and no `text_query`:

```python
hits = db.search_memory("docs", embed("database written in rust"), top_k=2)
for h in hits:
    print(f"  {h.key}: {h.payload[:60]}  (score={h.score:.3f})")
```

The top hit should be `d1` — the query never contains the literal words *"embedded"* or *"database"*, yet vector search retrieves it by meaning.

## 2. Keyword search (BM25)

Pass `text_query` and an **empty vector** to search lexically:

```python
hits = db.search_memory("docs", [], text_query="migration tools", top_k=2)
for h in hits:
    print(f"  {h.key}: {h.payload[:60]}  (score={h.score:.3f})")
```

Expected top hit: `d3`. BM25 indexes the `payload` field, so exact terms like *"migration"* and *"tools"* match strongly even when the vectors are unrelated.

## 3. Hybrid search (vector + BM25 fusion)

Pass both a query vector **and** `text_query` — VantaDB fuses the two scores:

```python
hits = db.search_memory(
    "docs",
    embed("vector database with keyword search"),
    text_query="hybrid BM25 keyword vector",
    top_k=3,
)
for h in hits:
    print(f"  {h.key}: {h.payload[:60]}  (score={h.score:.3f})")
```

Now a document that is semantically relevant **and** shares exact terms (like `d2`) ranks above documents that only match one signal. This is the recommended mode for most production queries — use it by default and drop to vector-only when you don't have a keyword component.

## 4. Filtered search (metadata equality)

Attach metadata at write time and filter before ranking:

```python
from vantadb_py import VantaDB

db.put("docs", "d5", "The WASM build runs fully in the browser.",
       metadata={"topic": "wasm", "lang": "rust"}, vector=embed("wasm in browser"))
db.put("docs", "d6", "The TypeScript SDK wraps the native engine.",
       metadata={"topic": "sdk", "lang": "typescript"}, vector=embed("typescript sdk"))

hits = db.search_memory(
    "docs",
    embed("browser runtime"),
    filters={"topic": "wasm"},
    top_k=5,
)
for h in hits:
    print(f"  {h.key}: {h.payload[:60]}  (score={h.score:.3f})")
```

Expected top hit: `d5`. Filters narrow the candidate set **before** vector comparison, which is faster and more accurate than filtering results afterwards.

> **Supported filter values:** the Python SDK matches metadata values with equality semantics (strings, ints, floats, bools). Range operators (`$gt`, `$gte`, `$lt`, `$lte`) are available on the Rust SDK.

## 5. Tuning

### `distance_metric`

Choose how vector distance is computed:

```python
hits = db.search_memory("docs", embed("vector database"),
                        top_k=3, distance_metric="cosine")     # default
hits = db.search_memory("docs", embed("vector database"),
                        top_k=3, distance_metric="euclidean")  # L2 distance
```

### `top_k`

Controls how many hits are returned. Note that filtering happens first, so a heavily filtered query may return fewer than `top_k`.

### `explain_memory_search`

Inspect *why* a hit ranked where it did with the dedicated explain method:

```python
explanation = db.explain_memory_search(
    "docs",
    embed("vector database"),
    text_query="vector database",
    top_k=2,
)
print(explanation)   # search route, fusion weights, per-hit vector/lexical breakdown
```

It returns a dict describing the search route, the fusion parameters, and each hit's vector and lexical contributions — useful when debugging unexpected rankings.

## When to use which mode

| Query type | Mode | Example |
|------------|------|---------|
| Paraphrased, natural language | Vector-only | "what should I use instead of SQLite" |
| Exact names, codes, product terms | Keyword (`text_query`) | "vantadb-py 0.5.0 put signature" |
| General retrieval | Hybrid (both) | production default |
| Narrow scope | Any + `filters` | "only docs about wasm" |

## Next steps

- **Reranking:** for high-precision retrieval, retrieve with a wide `top_k` and rerank with a cross-encoder (`vantadb-litellm`)
- **Snippets:** `db.generate_snippet(payload, text_query, with_highlighting=True)` produces highlighted excerpts from BM25 matches
- **Export:** `db.export_namespace("dump.jsonl", namespace="docs")` dumps a namespace for offline analysis

---

**Key takeaway:** one method — `search_memory()` — covers semantic, keyword, hybrid, and filtered search. Add `text_query` to any vector search to get BM25 fusion, and use `filters` to scope results before ranking.
