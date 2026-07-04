---
title: "Migrating from ChromaDB to VantaDB"
status: draft
tags: [vantadb, tutorial, guide, migration, chromadb]
last_reviewed: 2026-07-03
aliases: []
---

# Migrating from ChromaDB to VantaDB

If you're using ChromaDB today, switching to VantaDB unlocks **graph edges**, **MCP protocol support**, **WASM browser runtime**, and **hybrid search** — while keeping your existing vector workflow. This tutorial shows the exact API mappings and provides a migration script you can run on your existing ChromaDB data.

## Side-by-side API comparison

| Operation | ChromaDB | VantaDB |
|-----------|----------|---------|
| Connect | `chromadb.PersistentClient(path)` | `vantadb.connect(path)` |
| Create/get collection | `client.get_or_create_collection(name)` | `db.space(name)` |
| Insert documents | `collection.add(ids, documents, metadatas)` | `space.put({...node...})` |
| Semantic search | `collection.query(query_texts)` | `space.similar_to(query)` |
| Get by ID | `collection.get(ids)` | `space.get(id)` |
| Delete | `collection.delete(ids)` | `space.delete(id)` |
| List all | `collection.get()` | `space.list()` |
| Hybrid search | Not built-in | `space.search(query, mode="hybrid")` |

## 1. Setup comparison

**ChromaDB:**

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma_data")
collection = client.get_or_create_collection(
    name="my_docs",
    metadata={"hnsw:space": "cosine"},
)
```

**VantaDB:**

```python
import vantadb

db = vantadb.connect("./vantadb_data")
space = db.space("my_docs")
# HNSW with cosine is the default — nothing extra to configure.
```

## 2. Inserting documents

**ChromaDB:**

```python
collection.add(
    ids=["doc1", "doc2"],
    documents=["VantaDB is an embedded vector database.", "It supports Python, TS, and Rust."],
    metadatas=[
        {"source": "docs", "page": 1},
        {"source": "docs", "page": 2},
    ],
)
```

**VantaDB:**

```python
space.put({
    "id": "doc1",
    "content": "VantaDB is an embedded vector database.",
    "source": "docs",
    "page": 1,
    "embedding_field": "content",
})
space.put({
    "id": "doc2",
    "content": "It supports Python, TS, and Rust.",
    "source": "docs",
    "page": 2,
    "embedding_field": "content",
})
```

Key differences:
- ChromaDB separates `ids`, `documents`, and `metadatas` into parallel arrays.
- VantaDB uses a single **node** object — everything (content + metadata) lives together.
- In VantaDB, you hint which field to embed with `embedding_field`.

## 3. Querying

**ChromaDB:**

```python
results = collection.query(
    query_texts=["embedded database"],
    n_results=5,
    where={"source": "docs"},
)
```

**VantaDB:**

```python
results = space.similar_to(
    "embedded database",
    top_k=5,
    filter={"source": "docs"},
)
```

VantaDB returns objects with attribute access (`r.score`, `r.content`, `r.source`) instead of ChromaDB's dict-of-lists format.

## 4. Full migration script

This script exports all documents from a ChromaDB collection and imports them into VantaDB:

```python
#!/usr/bin/env python3
"""
Migration script: ChromaDB → VantaDB

Usage:
    python migrate_chroma_to_vantadb.py <chroma_path> <collection_name> <vantadb_path>
"""

import sys
import json
import chromadb
import vantadb
from datetime import datetime


def export_chromadb_collection(chroma_path: str, collection_name: str) -> dict:
    """Read all documents from a ChromaDB collection."""
    client = chromadb.PersistentClient(path=chroma_path)
    collection = client.get_collection(name=collection_name)

    data = collection.get(include=["documents", "metadatas", "embeddings"])

    print(f"Exported {len(data['ids'])} documents from ChromaDB '{collection_name}'")
    return data


def import_into_vantadb(vantadb_path: str, chroma_data: dict):
    """Write all documents into VantaDB, preserving metadata."""
    db = vantadb.connect(vantadb_path)
    space = db.space("documents")

    ids = chroma_data["ids"]
    documents = chroma_data["documents"]
    metadatas = chroma_data.get("metadatas", [{}] * len(ids))
    embeddings = chroma_data.get("embeddings")

    batch_size = 100
    total = len(ids)

    for start in range(0, total, batch_size):
        end = min(start + batch_size, total)
        batch_ids = ids[start:end]
        batch_docs = documents[start:end]
        batch_meta = metadatas[start:end]
        batch_embs = embeddings[start:end] if embeddings else None

        for i, doc_id in enumerate(batch_ids):
            node = {
                "id": doc_id,
                "content": batch_docs[i] or "",
                "embedding_field": "content",
                "source_collection": "chroma_migrated",
                "migrated_at": datetime.utcnow().isoformat(),
            }

            # Preserve all original metadata
            meta = batch_meta[i] if batch_meta else {}
            if isinstance(meta, dict):
                for k, v in meta.items():
                    if k != "id":
                        # Flatten ChromaDB list metadata
                        node[k] = v if not isinstance(v, list) else json.dumps(v)

            # Preserve pre-computed embeddings (bypass auto-embedding)
            if batch_embs and batch_embs[i] is not None:
                node["embedding"] = batch_embs[i]
                # Remove embedding_field so VantaDB doesn't re-embed
                node.pop("embedding_field", None)

            space.put(node)

        print(f"  Migrated {end}/{total} documents...")

    print(f"\n✓ Migration complete: {total} documents → {vantadb_path}")


def verify_migration(vantadb_path: str, sample_query: str = "test"):
    """Run a test query to verify the migration worked."""
    db = vantadb.connect(vantadb_path)
    space = db.space("documents")

    count = len(space.list())
    print(f"\nVerification: {count} documents in VantaDB")

    results = space.similar_to(sample_query, top_k=3)
    print(f"Sample query '{sample_query}' returned {len(results)} results")
    for r in results:
        print(f"  [{r.score:.3f}] {r.content[:80]}")

    return count > 0


if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python migrate_chroma_to_vantadb.py <chroma_path> <collection_name> <vantadb_path>")
        sys.exit(1)

    chroma_path = sys.argv[1]
    collection_name = sys.argv[2]
    vantadb_path = sys.argv[3]

    data = export_chromadb_collection(chroma_path, collection_name)
    import_into_vantadb(vantadb_path, data)
    verify_migration(vantadb_path)
```

Run it:

```bash
python migrate_chroma_to_vantadb.py ./chroma_data my_collection ./vantadb_data
```

## 5. Feature comparison: what you gain

| Feature | ChromaDB | VantaDB |
|---------|----------|---------|
| Vector search (HNSW) | ✅ | ✅ |
| Metadata filtering | ✅ | ✅ |
| **Hybrid search (vector + BM25)** | ❌ | ✅ |
| **Graph edges (knowledge graph)** | ❌ | ✅ |
| **MCP protocol support** | ❌ | ✅ |
| **WASM browser runtime** | ❌ | ✅ |
| Python SDK | ✅ | ✅ |
| TypeScript SDK | ❌ | ✅ |
| Rust SDK | ❌ | ✅ |
| Embedded (no server) | ✅ | ✅ |
| Auto-embedding | ❌ (manual) | ✅ (plug-in) |
| Single-node data model | ❌ (parallel arrays) | ✅ (rich objects) |

## 6. Post-migration checklist

After migrating, you can immediately start using VantaDB-specific features:

### Add graph edges between documents

```python
space.edge("doc1", "doc2", relation="related")
space.edge("doc1", "doc3", relation="supersedes")
# Later: traverse from any document
neighbors = space.neighbors("doc1", relation="related")
```

### Enable hybrid search

```python
results = space.search("your query", mode="hybrid", alpha=0.5)
```

### Expose via MCP

```python
# In your MCP server config:
# {
#   "vantadb": {
#     "path": "./vantadb_data"
#   }
# }
# Now any MCP-compatible host (Claude Desktop, etc.) can query your data.
```

### Run in the browser

```python
# With VantaDB WASM build, the same code works in a browser:
# import vantadb from '@vantadb/wasm'
```

## Summary

| Task | ChromaDB equivalent | VantaDB equivalent |
|------|-------------------|--------------------|
| Connect | `PersistentClient(path)` | `vantadb.connect(path)` |
| Collection | `get_or_create_collection(name)` | `db.space(name)` |
| Insert | `collection.add(ids, docs, metas)` | `space.put({id, content, ...meta})` |
| Query | `collection.query(query_texts)` | `space.similar_to(query)` |
| Delete | `collection.delete(ids)` | `space.delete(id)` |
| Filter | `where={...}` | `filter={...}` |

Migration takes ~5 minutes and you keep all your existing data and embeddings. From there, the graph engine, MCP protocol, WASM runtime, and hybrid search are available with zero additional setup.

---

**Key takeaway:** VantaDB is a drop-in upgrade from ChromaDB — same mental model, richer feature set, and your migration script runs in under 60 lines of Python.
