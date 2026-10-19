# VantaDB × Ollama

Ollama embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Superseded — not part of the PyPI release (FIND-273, owner decision
> 2026-10-06).** The `vantadb-ollama` PyPI name and `vantadb_ollama` module
> belong to the canonical Rust provider
> [`providers/ollama`](../../providers/ollama/README.md); this Python adapter
> stays in-repo as a source-only reference and is **not** published by the
> `adapters-v*` lane.

```bash
# Source-only, from a repo checkout
cd integrations/ollama && pip install .
```

### From PyPI (the provider, after its first release)

```bash
pip install vantadb-ollama   # installs the Rust provider (providers-v* lane)
```

## Quickstart

```python
from vantadb_ollama import VantaDBOllama

store = VantaDBOllama(
    model="nomic-embed-text",
    db_path="./my_data",
)

store.add_texts(["VantaDB is an embedded vector database."])
results = store.similarity_search("vector database")
for doc in results:
    print(doc.page_content)
```

## API

- `add_texts(texts, metadatas=None, ids=None)` — embed and store texts
- `similarity_search(query, k=4)` — search by query text
- `delete(ids)` — delete by IDs

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box,
  on top of Ollama's local embeddings — fully offline.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```
