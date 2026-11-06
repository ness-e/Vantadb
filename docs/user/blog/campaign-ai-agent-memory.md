---
title: AI Agent Memory — Durable Memory for Local Agents
kind: howto
status: active
description: Why agent memory should be a durable embedded database, not a re-ingest of full context on every turn.
tags: [vantadb, ai-agents, memory]
---

# AI Agent Memory — Durable Memory for Local Agents

*By the VantaDB Team*

An agent that re-reads its entire history on every turn is paying the same
cost twice: once in tokens, once in latency. The "SQLite of agent memory"
idea behind VantaDB is simple — treat memory as durable local
infrastructure: write each fact once, read back only what the current turn
needs, and let the database worry about surviving restarts.

This post walks the real workflow the SDK exposes today: observe → put →
flush → reopen → hybrid recall. No marketing metrics — everything shown is
an SDK operation. The hybrid example requires an application-provided
`embed(text)` function; it is not a built-in VantaDB API.

## The workflow

```python
from vantadb import Client

db = Client("./agent-memory")

# 1. Observe: the agent writes a fact once.
db.put(
    "support-agent",
    "decision-2026-11-06",
    "Invoices export must retry with exponential backoff (max 3).",
    metadata={"kind": "decision", "area": "billing"},
    vector=embed("Invoices export must retry with exponential backoff"),
)
db.flush()
db.close()

# 2. Reopen: read the previously flushed database.
db = Client("./agent-memory")

# 3. Recall: hybrid search — BM25 for exact terms, HNSW for meaning.
hits = db.search(
    "support-agent",
    query_vector=embed("invoice export failures"),
    text_query="invoice export",
    top_k=5,
)
db.close()
```

Three properties do the work:

- **Namespaces as scopes.** One namespace per agent, project, or user
  separates records without separate databases. Namespace names are not
  credentials or an authorization boundary.
- **Hybrid retrieval.** Exact strings (`decision-2026-11-06`, error codes)
  come back through BM25; paraphrases come back through the vector index;
  RRF fuses both in one query plan.
- **WAL durability.** Checksummed logging and recovery support persistence.
  Acknowledgment and disk synchronization depend on the configured sync
  mode; CRC32C alone does not prove crash immunity. See
  [WAL integrity](../../api/WAL_INTEGRITY.md).

## What this replaces

The alternatives agents grow out of:

- **Full-context rewriting** — re-embedding and re-ingesting the whole
  transcript each turn. Cost grows linearly with history.
- **In-memory indexes** — FAISS-style graphs serialized by hand on exit;
  a crash between writes loses them.
- **A remote vector service** — network hop, API key, and a third party
  holding the agent's interaction history for a workload that fits in a
  file next to the application.

## Limits, stated plainly

- VantaDB stores and retrieves — it does not extract facts for you. An
  LLM extraction pipeline (like Mem0's) is an application-layer choice
  that sits on top; see `examples/migrate_from_mem0.py` for the adapter
  surface.
- `vector=` is your embedding. The engine does not pick a model; the
  local ONNX exports under `embeddings/models/` or an external encoder
  both work — keep query and document vectors in the same model space.
- Throughput and recall numbers we publish live in
  `docs/user/operations/BENCHMARKS.md` with their harness commands —
  reproduce them before quoting them.

```bash
pip install vantadb-py
```

For the IDE-agent variant of this pattern, read
[VantaDB as Persistent Memory for Claude Code (MCP)](./claude_code_mcp_memory.md);
for a fully local stack with local models, read
[Local Agent Memory with Ollama + VantaDB](./ollama_vantadb_local_memory.md).
