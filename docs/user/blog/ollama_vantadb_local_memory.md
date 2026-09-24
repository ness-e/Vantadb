---
title: "Local Agent Memory with Ollama + VantaDB"
version: 0.5.0
slug: ollama-vantadb-local-memory
date: 2026-06-19
author: "VantaDB Team"
tags: ["ollama", "local-llm", "ai-agents", "memory", "tutorial", "docker"]
description: "Give local agents running on Ollama a memory that survives restarts: embeddings from nomic-embed-text, recall from VantaDB hybrid search, zero cloud dependencies."
tag: Tutorial
readTime: "8 min"
canonical: https://vantadb.vercel.app/blog/ollama-vantadb-local-memory
draft: true
---

# Local Agent Memory with Ollama + VantaDB

*By the VantaDB Team*

Ollama made the model local. Your agent's memory should be local too. If you run `llama3.2:3b` for chat and `nomic-embed-text` for embeddings, the only thing still reaching for the cloud is usually the vector database — a bill and a network dependency for data that never needed to leave your machine.

This post shows the other half of the local stack: Ollama produces the vectors, VantaDB keeps them. One `docker-compose.yml` runs both services; one Python script wires agent memory with `put()` and hybrid `search()`.

---

## 1. The stack: two containers, zero cloud

The demo composition ships in the repo root (`docker-compose.yml`): VantaDB serving on `8080` next to `ollama/ollama:0.33.2` on the default `11434`. CPU-only by default, ~4 GB RAM for the reference models. AnythingLLM is deliberately excluded — it does not list VantaDB as a supported vector backend, so there is no glue to write and none is faked here.

```bash
ollama pull llama3.2:3b
ollama pull nomic-embed-text
docker compose up -d
```

Embeddings come from `nomic-embed-text` over the local Ollama API. Nothing is sent anywhere else, ever.

---

## 2. Memory as put() and search()

Each agent gets a namespace in a local VantaDB store. Observations go in with payload, metadata, and the Ollama-produced vector:

```python
from vantadb import VantaDB

db = VantaDB(path="./agent-memory")
db.put(
    namespace="support-agent",
    key="obs-0042",
    payload="Customer reports timeout on invoice export; retry succeeded.",
    vector=embed("Customer reports timeout on invoice export"),
    metadata={"kind": "tool_log", "session": "s-118"},
)
```

Recall is hybrid search — BM25 catches the exact error string, HNSW catches the paraphrase, RRF fuses both:

```python
hits = db.search(
    namespace="support-agent",
    text="invoice export timeout",
    vector=embed("invoice export timeout"),
    top_k=5,
)
```

One database covers what used to be four systems: semantic recall, session state (`get()` by key), structured metadata filters, and crash-safe persistence via the CRC32C-checksummed WAL. Kill the process mid-session and the memories are still there on restart.

---

## 3. Why not just keep vectors in memory?

Because agents crash, laptops sleep, and demos happen on planes. An in-memory index forgets everything on exit and re-embeds the whole corpus on start. VantaDB persists the canonical records in its LSM store and rebuilds the derived HNSW/BM25 indexes from them — memory becomes infrastructure instead of a warm-up cost.

The numbers we stand behind for this pattern (batch throughput, competitive recall) live in `docs/user/operations/BENCHMARKS.md`. Reproduce them on your hardware before quoting them — that file documents the exact commands.

---

## Conclusion

Local models solved half the privacy and cost problem. Local memory solves the other half: `ollama pull` for the brains, `pip install vantadb-py` for the memory, one compose file to run both.

```bash
pip install vantadb-py
```

Star the [VantaDB repository](https://github.com/ness-e/Vantadb) and join the Discord to share your agent setup. To give the same persistent memory to your IDE agent instead, read [VantaDB as Persistent Memory for Claude Code (MCP)](/blog/claude-code-mcp-memory).
