# VantaDB × DSPy

DSPy Retriever adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

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

## Development

```bash
pip install -e .
```
