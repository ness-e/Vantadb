---
title: "VantaDB Architecture & Cognitive Memory Engine"
kind: concept
status: active
description: "Architectural overview of VantaDB as the local-first cognitive memory engine for AI agents"
tags: [architecture, memory, engine, mcp, wal]
---

# VantaDB Architecture & Cognitive Memory Engine

VantaDB is engineered from the ground up as the "SQLite for agents": an in-process, zero-dependency cognitive memory engine designed to provide AI agents with durable, governed long-term memory.

---

## 1. Core Architectural Pillars

VantaDB rejects client-server operational overhead in favor of embedded simplicity, pairing storage durability with cognitive pipeline stages.

```
+---------------------------------------------------------------+
|                       AI Agent Applications                   |
|       (Claude Code / Cursor / LangGraph / Python / TS SDKs)   |
+-------------------------------+-------------------------------+
                                |
                   Unified Embedded API / MCP
                                |
+-------------------------------v-------------------------------+
|            vanta-memory: Cognitive Pipeline (L0 -> L3)        |
|  - L0 Raw Capture     - L1 Entity Dedup & Extraction          |
|  - L2 Episodic Scenes - L3 Persona Synthesis & Distillation   |
+-------------------------------+-------------------------------+
                                |
+-------------------------------v-------------------------------+
|                      VantaDB Core Engine                      |
|  - Hybrid Retrieval: HNSW Vector ANN + BM25 Lexical + RRF     |
|  - Durability Layer: Write-Ahead Log (WAL) + CRC32 Checksums  |
|  - Lifecycle Governance: Unified forget() [Soft/Hard/Cert]    |
|  - Storage Engine: Segmented VFile Stores + In-Memory Backend |
+---------------------------------------------------------------+
```

### 1.1 In-Process Embedded Engine
Just as SQLite eliminated the need for standalone database servers for relational data, VantaDB runs directly inside the agent application process or as a lightweight companion sidecar. There are no external databases to configure, no Docker containers to maintain, and no network latency on the critical memory path.

### 1.2 Write-Ahead Log (WAL) & Crash Safety
Every mutation (`put`, `update`, `forget`) is appended to a checksummed Write-Ahead Log before being committed to memory-mapped segment files (`VFile`). Upon unexpected process termination, the engine recovers consistent state by replaying uncommitted WAL entries.

### 1.3 Native Hybrid Retrieval
Keyword matching and dense vector similarity solve complementary problems:
- **Vector search (HNSW)** retrieves semantically related concepts without exact keyword overlap.
- **Lexical search (BM25)** matches precise identifiers, technical terms, and code symbols.
- **Reciprocal Rank Fusion (RRF)** combines candidate lists using reciprocal ranking, eliminating the need to normalize disparate score scales.

---

## 2. Cognitive Memory Tiers (`vanta-memory` L0→L3)

Agent memory is structured into four distinct cognitive abstraction tiers:

1. **L0 (Working Memory & Turns)**: Exact dialog messages, timestamps, and tool interaction turns captured during active agent sessions.
2. **L1 (Extracted Entities & Facts)**: Deduplicated facts, preferences, and verified assertions extracted from session turns.
3. **L2 (Episodic Scenes)**: Chronologically grouped task scenes summarizing objectives, attempts, outcomes, and failure modes.
4. **L3 (Distilled Persona & Knowledge)**: High-level enduring knowledge, core identity constraints, and consolidated insights periodically refreshed during idle cycles.

---

## 3. Unified Deletion & Forget Semantics

Managing memory requires forgetting obsolete or sensitive data. The `forget()` interface supports three distinct operational guarantees:

- **Soft Forget**: Inserts a tombstone, preserving audit history while excluding the item from retrieval.
- **Hard Forget**: Purges physical records and immediately rebuilds associated HNSW and BM25 index pointers.
- **Certified Forget**: Zeroizes bytes on disk and generates a cryptographically signed `PurgeCertificate` (VER-02) verifying absence of data remnants.

---

## 4. MCP Integration (`McpProfile::Brain`)

VantaDB exposes an optimized Model Context Protocol (MCP) server profile designed specifically for AI IDEs (Claude Code, Cursor, OpenCode). The `McpProfile::Brain` surface limits exposed tools to 12 focused cognitive primitives plus a meta-dispatch router (`vanta_dispatch`), preserving LLM prompt tokens while enabling full memory autonomy. For hands-on setup instructions, see the [Getting Started Guide](../guide/getting-started.md).
