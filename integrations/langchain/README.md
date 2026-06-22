# VantaDB × LangChain

LangChain `VectorStore` adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

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

## Development

```bash
pip install -e .
```
