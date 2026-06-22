# VantaDB × LlamaIndex

LlamaIndex `BasePydanticVectorStore` adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-llamaindex
```

## Quickstart

```python
from llama_index.core import VectorStoreIndex, SimpleDirectoryReader
from llama_index.embeddings.openai import OpenAIEmbedding
from vantadb_llamaindex import VantaDBVectorStore

# Create vector store
vector_store = VantaDBVectorStore(
    db_path="./my_data",
    namespace="docs",
)

# Create index from documents
documents = SimpleDirectoryReader("./data").load_data()
embed_model = OpenAIEmbedding(model="text-embedding-3-small")

index = VectorStoreIndex.from_documents(
    documents,
    embed_model=embed_model,
    vector_store=vector_store,
)

# Query
query_engine = index.as_query_engine()
response = query_engine.query("What is VantaDB?")
print(response)
```

## API

- `add(nodes)` — add pre-embedded nodes to the store
- `delete(ref_doc_id)` — delete all nodes from a document
- `query(query)` — vector + hybrid search
- `get_nodes(node_ids)` — retrieve nodes by ID
- `clear()` — remove all records

## Development

```bash
pip install -e .
```
