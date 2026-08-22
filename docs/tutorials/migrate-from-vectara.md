---
title: "Migrating from Vectara to VantaDB"
status: active
tags: [vantadb, tutorial, guide, migration, vectara]
last_reviewed: 2026-08-04
aliases: []
---

# Migrating from Vectara to VantaDB

> **Canonical reference.**

Vectara shut down its self-service RAG-as-a-Service tier in 2026 and repositioned as an enterprise agent platform. If you were on the self-service tier, you need a new home for your corpora. VantaDB is a drop-in local-first replacement: **embedded** (no hosted API), **open source** (Apache 2.0), **free** (no per-index or per-query pricing), and **BYO-embeddings** (no lock-in to Vectara's Boomerang model).

This tutorial walks you through the full migration: exporting your Vectara corpora, converting them to VantaDB's import format, importing them, and re-embedding so semantic search keeps working.

## Side-by-side concept mapping

| Vectara | VantaDB |
|---------|---------|
| Corpus | Namespace (lazy — created on first `put()`) |
| Document (`id`, `metadata`) | Memory record (`key`, `metadata`) |
| Document part (`parts[].text`) | Payload (the record's text content) |
| Document part metadata | Merged into the record `metadata` dict |
| Semantic query (Boomerang) | `db.search_memory(ns, query_vector, ...)` with your own embeddings |
| Keyword / lexical search | `db.search_memory(ns, ..., text_query="...")` (BM25) |
| Hybrid search (RRF / MMR) | `search_memory` with both `query_vector` and `text_query` (RRF fusion) |
| Query filters (`metadata_filter`) | `filters={...}` |
| API key / OAuth client credentials | None — embedded, no server, no auth |
| Hosted API (`https://api.vectara.io/v2`) | Local library — no network required |

## Prerequisites

- A Vectara API key (or client ID + client secret) with access to the corpora you want to migrate
- Python 3.11+ and `pip install vantadb-py`
- An embedding model you can call from Python (OpenAI, Ollama, LiteLLM — anything) to re-embed your text. Vectara does **not** expose its Boomerang embeddings, so you cannot preserve the original vectors.

## 1. Export your corpus from Vectara

The script below authenticates with Vectara's OAuth2 client-credentials flow, pages through all documents in a corpus, fetches each document's full content, and writes one JSON line per document part to `corpus-export.jsonl`.

```python
# vanta-skip: requires VECTARA_CLIENT_ID/SECRET/CORPUS_KEY credentials and network access
"""Export a Vectara corpus to corpus-export.jsonl (Vectara API v2)."""
import json
import os
import requests

CLIENT_ID = os.environ["VECTARA_CLIENT_ID"]
CLIENT_SECRET = os.environ["VECTARA_CLIENT_SECRET"]
CORPUS_KEY = os.environ["VECTARA_CORPUS_KEY"]

BASE = "https://api.vectara.io/v2"
TOKEN_URL = "https://vectara.io/oauth2/token"


def get_token() -> str:
    resp = requests.post(
        TOKEN_URL,
        data={"grant_type": "client_credentials", "client_id": CLIENT_ID, "client_secret": CLIENT_SECRET},
        timeout=30,
    )
    resp.raise_for_status()
    return resp.json()["access_token"]


def get_document(token: str, doc_id: str) -> dict:
    resp = requests.get(
        f"{BASE}/corpora/{CORPUS_KEY}/documents/{doc_id}",
        headers={"Authorization": f"Bearer {token}"},
        timeout=60,
    )
    resp.raise_for_status()
    return resp.json()


def main() -> None:
    token = get_token()
    headers = {"Authorization": f"Bearer {token}"}
    seen = 0
    page_key = None

    with open("corpus-export.jsonl", "w", encoding="utf-8") as out:
        while True:
            params = {"limit": 100}
            if page_key:
                params["page_key"] = page_key
            resp = requests.get(
                f"{BASE}/corpora/{CORPUS_KEY}/documents",
                headers=headers,
                params=params,
                timeout=60,
            )
            resp.raise_for_status()
            body = resp.json()

            for doc in body.get("documents", []):
                full = get_document(token, doc["id"])
                for i, part in enumerate(full.get("parts", [])):
                    out.write(json.dumps({
                        "document_id": full["id"],
                        "part_index": i,
                        "text": part.get("text", ""),
                        "metadata": {
                            **(full.get("metadata") or {}),
                            **(part.get("metadata") or {}),
                        },
                    }) + "\n")
                    seen += 1

            page_key = (body.get("metadata") or {}).get("page_key")
            if not page_key:
                break

    print(f"Exported {seen} parts from corpus '{CORPUS_KEY}'")


if __name__ == "__main__":
    main()
```

Run it:

```bash
export VECTARA_CLIENT_ID=... VECTARA_CLIENT_SECRET=... VECTARA_CORPUS_KEY=...
python export_vectara.py
```

> **Note:** the Vectara API returns document content in `parts[].text` (tables, if enabled, live in `parts[].table`). This script keeps only text parts; convert table cells yourself if you depend on them.

## 2. Convert to VantaDB's import format

VantaDB's `import_file()` reads the same JSONL format its own exports use — one record per line with a `schema_version` header. The conversion step maps each exported part to a record:

| Export field | VantaDB field |
|--------------|---------------|
| (fixed) | `schema_version: 1` |
| `document_id` + part index | `key` (`"{document_id}#{part_index}"`) |
| corpus key | `namespace` |
| `text` | `payload` |
| `metadata` | `metadata` (scalar values only) |
| your embedding model's output | `vector` (optional, see below) |

```python
# vanta-skip: reads corpus-export.jsonl produced by the step 1 export (needs real Vectara data)
"""Convert corpus-export.jsonl (Vectara) to vantadb-import.jsonl (VantaDB)."""
import json

EXPORT_SCHEMA_VERSION = 1
NAMESPACE = "vectara_docs"  # or use your corpus key


def embed(text: str) -> list[float] | None:
    # Plug in your embedding model (OpenAI, Ollama, LiteLLM, ...).
    # Return None to skip vectors — you can re-embed after import instead.
    return None


def main() -> None:
    with open("corpus-export.jsonl", encoding="utf-8") as src, \
         open("vantadb-import.jsonl", "w", encoding="utf-8") as dst:
        for line in src:
            part = json.loads(line)
            vec = embed(part["text"])
            record = {
                "schema_version": EXPORT_SCHEMA_VERSION,
                "namespace": NAMESPACE,
                "key": f"{part['document_id']}#{part['part_index']}",
                "payload": part["text"],
                "metadata": {k: v for k, v in part["metadata"].items() if isinstance(v, (str, int, float, bool))},
                "vector": vec,
                "sparse_vector": None,
                "created_at_ms": 0,
                "updated_at_ms": 0,
                "version": 1,
                "expires_at_ms": None,
            }
            dst.write(json.dumps(record) + "\n")

    print("Wrote vantadb-import.jsonl")


if __name__ == "__main__":
    main()
```

**About embeddings:** Vectara doesn't let you extract Boomerang vectors, so there are two paths:

1. **Embed during conversion** (recommended): implement `embed()` with your own model and write the vector into each line. Semantic search works immediately after import.
2. **Embed after import**: leave `vector` as `None`, then run `db.reindex_hnsw_from_text()` once the embedding provider is configured in VantaDB (see the [Embedding Providers](05-embedding-integrations.md) tutorial).

## 3. Import into VantaDB

```python
# vanta-skip: imports vantadb-import.jsonl produced by the step 2 conversion
from vantadb_py import VantaDB

db = VantaDB("./vantadb_data")
report = db.import_file("vantadb-import.jsonl")
print(report)
# {'inserted': ..., 'updated': ..., 'skipped': ..., 'errors': ..., 'duration_ms': ...}
```

If you skipped vectors in step 2, rebuild the HNSW index from the payload text now:

```python
db.reindex_hnsw_from_text("vectara_docs", page_size=500)
report = db.rebuild_index()
print(f"Rebuilt {report['indexed_vectors']} vectors")
```

Flush so the import is durable:

```python
db.flush()
```

## 4. Verify the migration

```python
# vanta-skip: depends on records imported in step 3 and a user-defined my_embedding()
# Spot-check a record by its Vectara document id
record = db.get_memory("vectara_docs", "my-vectara-doc-id#0")
print(record.key, record.payload[:80], record.metadata)

# Semantic search must return relevant hits
results = db.search_memory(
    "vectara_docs",
    query_vector=my_embedding("your query"),
    top_k=5,
)
for hit in results:
    print(hit.key, round(hit.score, 4), hit.payload[:60])

# Total count sanity check
total = len(db.list_memory("vectara_docs", limit=1000))
print(f"records in namespace: {total}")
```

Compare the top hits for a few queries you used on Vectara. Results won't be identical (different embeddings, different chunk boundaries) but should be equally relevant.

## 5. What you gain after migrating

| Feature | Vectara (self-service) | VantaDB |
|---------|------------------------|---------|
| Hosted API | ✅ | ❌ (embedded, local) |
| Self-host / air-gapped / offline | ❌ | ✅ |
| Open source | ❌ | ✅ (Apache 2.0) |
| Free / no metered pricing | ❌ | ✅ |
| BYO embeddings | ❌ (Boomerang lock-in) | ✅ |
| Hybrid search (BM25 + vector) | ✅ | ✅ |
| **Graph edges (knowledge graph)** | ❌ | ✅ |
| **MCP protocol support** | ❌ | ✅ |
| **WASM browser runtime** | ❌ | ✅ |
| Python / TypeScript / Rust SDKs | ✅ (Python) | ✅ (all three) |

Now that your data is in VantaDB, the graph engine, MCP protocol, and browser runtime are available with zero extra setup — see the [ChromaDB migration tutorial](03-migrating-from-chromadb.md) for the same post-migration features (`add_edge()`, `graph_bfs()`, MCP config, WASM).

## Known limitations

- **No vector preservation.** Boomerang embeddings are not exportable. You must re-embed with your own model; exact result parity is not expected.
- **Summarizers and rerankers don't carry over.** Vectara's Mockingbird summarizer and reranking are server-side. In VantaDB, post-process results yourself (e.g. `generate_snippet()` for highlighting) or call your own LLM.
- **Chunking is client-side.** Vectara's pipeline chunked documents server-side. Here, each document part exported from Vectara becomes one record — chunk long parts yourself before importing if you want finer granularity.
- **Metadata is scalar-only.** VantaDB metadata holds `str`/`int`/`float`/`bool` (and homogeneous lists); nested structures are dropped by the conversion script.
- **VantaDB is embedded only.** There is no VantaDB server to connect to remotely (the optional HTTP server is for localhost tooling).

## Rollback plan

The migration is read-only against Vectara: `corpus-export.jsonl` is a plain file and your Vectara account stays untouched. To roll back, stop using `./vantadb_data` and keep your Vectara data until you're confident — no data is ever deleted from the source.
