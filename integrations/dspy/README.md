# VantaDB × DSPy

DSPy Retriever adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Published on PyPI at 0.9.0** (verified live 2026-10-20). The PyPI
> install pulls the published release; to run against the current tree use
> the repo-checkout path.

```bash
# From a repo checkout (current tree)
cd integrations/dspy && pip install .
```

### Install from PyPI

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

- `VantaDBRetriever(embedding=None, *, db_path=..., namespace=..., k=...)` — DSPy-compatible retriever
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
