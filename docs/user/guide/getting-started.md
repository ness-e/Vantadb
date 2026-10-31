---
title: "Getting Started with VantaDB"
kind: howto
status: active
description: "Quick start guide for embedding VantaDB as the local-first cognitive memory engine for AI agents"
tags: [getting-started, python, quickstart, memory]
---

# Getting Started with VantaDB

VantaDB is the embedded, local-first cognitive memory engine for AI agents ("SQLite for agents"). It provides durable, crash-safe storage via Write-Ahead Logging (WAL), native hybrid search (BM25 + HNSW + RRF), and a multi-tiered memory lifecycle (`vanta-memory` L0→L3) with unified forgetting semantics.

This guide walks you through installing VantaDB, storing memories, searching across semantic and lexical indices, and managing memory retention.

---

## 1. Installation

Install the native Python bindings via pip:

```bash
pip install vantadb-py
```

For Rust projects, add `vantadb` to your `Cargo.toml`:

```toml
[dependencies]
vantadb = "0.9"
```

---

## 2. Basic Setup and Memory Operations

Initialize an embedded instance and start storing agent memories:

```python
import vantadb

# Open a persistent local database
client = vantadb.Client.open("./agent_memory.vanta")

# Store a memory record with text, optional vector, and structured metadata
client.memory.put(
    key="pref_user_theme",
    text="User prefers dark monochrome editor themes and concise responses",
    metadata={
        "category": "preferences",
        "scope": "session_global",
        "confidence": 0.95,
    }
)
```

---

## 3. Hybrid Search & Recall

Query memories using hybrid search fusing semantic similarity (HNSW) and full-text keyword matches (BM25) via Reciprocal Rank Fusion (RRF):

```python
# Search across stored memories
results = client.search(
    query="theme preference",
    limit=5
)

for hit in results:
    print(f"[{hit.score:.3f}] {hit.key}: {hit.text}")
```

---

## 4. Unified Memory Forgetting (`forget`)

VantaDB supports three distinct lifecycle forgetting modes via the unified `forget()` API:

- **Soft**: Writes a tombstone marker, instantly hiding the record from search queries while retaining history for audit trails.
- **Hard**: Removes the record from disk and unlinks it immediately from HNSW and BM25 indices.
- **Certified**: Performs physical zeroization and generates a cryptographically verifiable deletion receipt (`PurgeCertificate`).

```python
# Soft deletion (reversible tombstone)
client.memory.forget("pref_user_theme", mode="soft")

# Certified deletion with cryptographic receipt
receipt = client.memory.forget("secret_api_token", mode="certified")
print(f"Purge certificate hash: {receipt['certificate_hash']}")
```

---

## 5. Next Steps

- Explore [Architecture](../concepts/architecture.md) for engine internals.
- Connect your AI IDE via [MCP Server Index](../../api/mcp/index.md).
