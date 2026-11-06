---
title: Introducing VantaDB
kind: howto
status: active
description: An introduction to VantaDB 0.9.0, its embedded memory workflow, and the limits of its guarantees
tags: [vantadb, local-first, memory]
---

# Introducing VantaDB

*By the VantaDB Team*

VantaDB is an embedded, local-first memory engine for AI agents. The positioning
is "SQLite for AI agents": open a database inside your application process,
store memories, and retrieve them without a database service. This is an analogy
about deployment, not a claim of SQLite file-format or SQL compatibility.

## The local-first thesis

The Rust engine combines BM25 lexical retrieval and HNSW vector retrieval with
native Reciprocal Rank Fusion (RRF). Embedded access needs no HTTP daemon,
database credentials, or per-query service fee. It still has CPU, memory,
storage, and operational costs; crossing a language binding also requires
conversion between host-language and Rust values.

Remote services and other embedded databases remain valid alternatives. Choose
according to deployment, concurrency, recovery, and retrieval requirements,
rather than assuming that one architecture wins every workload.

## Why Rust, why PyO3

Python uses the `vantadb-py` distribution and the canonical `vantadb` import.
The SDK lives under `src/sdk/`, with PyO3 bindings in `vantadb-python/`.
Native Node.js, TypeScript/WASM, and MCP provide additional integration surfaces;
they are not interchangeable packages or identical runtime environments.

For VantaDB 0.9.0, this Python example uses lexical retrieval without an embedding
provider:

```python
from vantadb import Client

db = Client("./agent-memory")
try:
    db.put("demo", "preference", "The user prefers concise answers.")
    db.flush()
    record = db.memory.get("demo", "preference")
    hits = db.search("demo", text_query="concise answers", top_k=5)
    print(record.payload, [hit.key for hit in hits])
finally:
    db.close()
```

## Embeddings and cognitive memory

Local model configurations include `multilingual-e5-small` (384 dimensions) and
`EmbeddingGemma 2` (768 dimensions). Query and document vectors must use the same
model space, not merely matching dimensions. The Python `Client` does not expose
`embed_text`; supply vectors from a provisioned encoder to add semantic retrieval.

The `vanta-memory` cognitive pipeline includes deterministic FastGate admission
and deduplication before LLM work. This does not mean every low-level `Client.put`
runs that pipeline, or that a fixed sub-0.1 ms latency is guaranteed.

## Persistence and deletion have explicit limits

The WAL includes CRC32C checks and recovery mechanisms, but durability depends on
sync configuration and failure conditions. The default plaintext WAL format is
v3; the encrypted variant is v4. See [WAL integrity](../../api/WAL_INTEGRITY.md)
for the mechanisms, rather than inferring universal crash immunity.

The `forget` API's `Certified` mode produces a purge receipt with SHA-256
integrity material and declared limits. It is not independent GDPR certification
or proof that every backup or physical medium has been overwritten. See
[certified deletion](../../api/CERTIFIED_DELETE.md).

Use [the benchmark reference](../operations/BENCHMARKS.md) for measured results
and reproduction commands. No new performance measurements are asserted here.

```bash
pip install vantadb-py
```

Join the community on Discord and star the [VantaDB repository](https://github.com/ness-e/Vantadb). If you want the story behind the engine, read [Why I Built a Local Memory Engine for AI Agents in Rust](./why_i_built.md).
