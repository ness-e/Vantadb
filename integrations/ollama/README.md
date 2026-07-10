# VantaDB × Ollama

Ollama embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-ollama
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

## Development

```bash
pip install -e .
```
