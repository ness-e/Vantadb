---
title: Local Agent Memory with Ollama + VantaDB
kind: howto
status: active
description: Fully local agent memory — Ollama produces embeddings on localhost, VantaDB stores them in-process.
tags: [ollama, local-llm, memory]
---

# Local Agent Memory with Ollama + VantaDB

*By the VantaDB Team*

Ollama made the model local. Your agent's memory should be local too. If you
run `llama3.2:3b` for chat and `nomic-embed-text` for embeddings, the only
thing still reaching for the cloud is usually the vector database — a bill
and a network dependency for data that never needed to leave your machine.

This post shows the other half of the local stack: Ollama produces vectors
on `localhost`, VantaDB keeps them **in-process** — no database container,
no compose file, no server to start.

---

## 1. The stack: one local model service, one embedded engine

Provision the local models first; `ollama pull` may access the model registry.
After provisioning, the example's embedding requests use the local service:

```bash
# Your own local Ollama service (install once, runs on localhost:11434)
ollama pull llama3.2:3b
ollama pull nomic-embed-text
```

```bash
pip install vantadb-py
```

VantaDB is an **embedded** engine — `Client(path)` opens a database inside
your Python process. There is no `vantadb-server` to run and nothing to
containerize. Ollama is the only service in this stack, and it stays on
your machine.

## 2. Embeddings from the local Ollama API

A minimal `embed()` over the local HTTP endpoint — no extra dependencies:

```python
import json
import urllib.request

def embed(text: str) -> list[float]:
    req = urllib.request.Request(
        "http://localhost:11434/api/embed",
        data=json.dumps({"model": "nomic-embed-text", "input": text}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as res:
        return json.loads(res.read())["embeddings"][0]
```

This is an explicit local-network call to *your own* Ollama instance —
optional, replaceable by any embedding function that returns a list of
floats (including the local ONNX exports under `embeddings/models/`).
The request and response follow the
[official Ollama embedding API](https://github.com/ollama/ollama/blob/main/docs/api.md);
`/api/embed` supersedes `/api/embeddings`. Keep the same model for writes and
queries rather than mixing vectors from different encoders.

## 3. Memory as put() and search()

Each agent gets a namespace in the embedded store. Observations go in with
payload, metadata, and the Ollama-produced vector:

```python
from vantadb import Client

db = Client("./agent-memory")
db.put(
    namespace="support-agent",
    key="obs-0042",
    payload="Customer reports timeout on invoice export; retry succeeded.",
    vector=embed("Customer reports timeout on invoice export"),
    metadata={"kind": "tool_log", "session": "s-118"},
)
```

Recall is hybrid search — BM25 catches the exact error string, HNSW catches
the paraphrase, RRF fuses both:

```python
hits = db.search(
    namespace="support-agent",
    text_query="invoice export timeout",
    query_vector=embed("invoice export timeout"),
    top_k=5,
)
```

One database covers what used to be several systems: semantic recall,
session state (`memory.get()` by key), structured metadata filters, and
WAL-backed persistence (CRC32C-checksummed log; see
`docs/api/WAL_INTEGRITY.md`). Recovery guarantees depend on sync configuration
and failure conditions; this example does not establish crash immunity.

---

## 4. Why not just keep vectors in memory?

Because agents crash, laptops sleep, and demos happen on planes. An
memory-only index needs an application-managed persistence strategy; losing
an index does not necessarily require re-embedding if vectors are preserved.
VantaDB persists canonical records in its storage backend and
rebuilds the derived HNSW/BM25 indexes from them — memory becomes
infrastructure instead of a warm-up cost.

The throughput and recall numbers we stand behind live in
`docs/user/operations/BENCHMARKS.md`, with the harness commands to
reproduce them on your hardware before quoting them.

---

## Conclusion

Local models solved half the privacy and cost problem. Local memory solves
the other half: `ollama pull` for the brains, `pip install vantadb-py` for
the memory. Ollama remains a separate local model service; VantaDB runs inside
the Python application process.

```bash
pip install vantadb-py
```

Star the [VantaDB repository](https://github.com/ness-e/Vantadb) and share
your agent setup. To give the same persistent memory to your IDE agent,
read [VantaDB as Persistent Memory for Claude Code (MCP)](./claude_code_mcp_memory.md).
