# VantaDB × OpenAI

OpenAI embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-openai
```

## Quickstart

```python
from vantadb_openai import VantaDBOpenAI

store = VantaDBOpenAI(
    api_key="sk-...",
    model="text-embedding-3-small",
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
