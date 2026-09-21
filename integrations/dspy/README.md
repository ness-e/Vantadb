# VantaDB × DSPy

DSPy Retriever adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Not on PyPI yet.** The `vantadb-dspy` package builds and passes
> `twine check` locally; it will go live with the first `adapters-v*` tag
> release. Until then, install from source.

```bash
# Today, from a repo checkout
cd integrations/dspy && pip install .
```

### Install from PyPI (after first release)

```bash
pip install vantadb-dspy
```

## Quickstart

```python
import dspy
from vantadb_dspy import VantaDBRetriever

retriever = VantaDBRetriever(
    db_path="./my_data",
    namespace="docs",
    k=5,
)

# Use in DSPy program
class RAG(dspy.Module):
    def __init__(self):
        self.retrieve = retriever

    def forward(self, question):
        context = self.retrieve(question)
        return context
```

## API

- `VantaDBRetriever(db_path, namespace, k)` — DSPy-compatible retriever
- `forward(query)` — search and return passages

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box;
  DSPy has no built-in memory, so VantaDB adds retrievable context across
  program runs.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```
