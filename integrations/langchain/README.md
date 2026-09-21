# VantaDB × LangChain

LangChain `VectorStore` adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Not on PyPI yet.** The `vantadb-langchain` package builds and passes
> `twine check` locally; it will go live with the first `adapters-v*` tag
> release. Until then, install from source.

```bash
# Today, from a repo checkout
cd integrations/langchain && pip install .
```

### Install from PyPI (after first release)

```bash
pip install vantadb-langchain
```

## Quickstart

```python
from langchain_openai import OpenAIEmbeddings
from vantadb_langchain import VantaDBVectorStore

embedding = OpenAIEmbeddings(model="text-embedding-3-small")

store = VantaDBVectorStore(
    embedding=embedding,
    db_path="./my_data",
    namespace="docs",
)

# Add documents
store.add_texts(
    ["VantaDB is an embedded vector database written in Rust.",
     "It supports hybrid search across vectors and text."],
    metadatas=[{"source": "docs"}, {"source": "docs"}],
)

# Search
results = store.similarity_search("vector database", k=5)
for doc in results:
    print(doc.page_content, doc.metadata)
```

## API

- `similarity_search(query, k=4)` — search by text
- `similarity_search_by_vector(embedding, k=4)` — search by raw vector
- `similarity_search_with_score(query, k=4)` — search with cosine distance
- `add_texts(texts, metadatas=None, ids=None)` — add documents
- `delete(ids=...)` — delete by key
- `from_texts(texts, embedding, metadatas=None, ids=None)` — create + populate store

## LangGraph (INTG-01)

Persistent [LangGraph](https://docs.langchain.com/oss/python/langgraph/stores)
adapters — same embedded database, no server. Requires
`langgraph-checkpoint>=2,<5` (pinned major: upstream API is unstable).

```python
from vantadb_langchain import VantaDBCheckpointer, VantaDBStore

# Short-term memory: per-thread checkpoints (put/get/list/writes/delete)
checkpointer = VantaDBCheckpointer(db_path="./my_data")
graph = builder.compile(checkpointer=checkpointer)
graph.invoke(inputs, {"configurable": {"thread_id": "user-1"}})

# Long-term memory: hierarchical KV store, namespaces are tuples
store = VantaDBStore(db_path="./my_data", embeddings=embedding)
await store.aput(("users", "123", "prefs"), "theme", {"mode": "dark"})
item = await store.aget(("users", "123", "prefs"), "theme")
results = await store.asearch(("users", "123"), query="dark mode")
```

Notes:

- Namespace tuples map to VantaDB namespaces joined with `/`
  (parts must be non-empty strings without `/`).
- `search(filter=...)` matches scalar value fields exactly; nested
  dicts/lists are payload-only.
- Semantic `query` needs `embeddings=`; without it raises `ValueError`.
- TTL is not supported (`supports_ttl = False`); `index=` is accepted
  and ignored.

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box,
  where LangChain's `InMemoryVectorStore` covers only small single-process
  sessions.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```
