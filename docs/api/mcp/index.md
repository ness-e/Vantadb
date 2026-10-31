---
title: "Model Context Protocol (MCP) Server"
kind: index
status: active
description: "MCP server reference for VantaDB: tools, profiles, Brain cognitive profile, and AI IDE integration"
tags: [mcp, api, brain-profile, tools]
---

# Model Context Protocol (MCP) Server

VantaDB provides a native Model Context Protocol (MCP) server implementation (`vantadb-mcp`) allowing AI assistants in IDEs (Claude Code, Cursor, OpenCode) and autonomous agents to persist, recall, and manage long-term cognitive memory across sessions.

---

## 1. Profiles Overview

To avoid overwhelming an agent's context window with dozens of granular database primitives, `vantadb-mcp` organizes tools into structured profiles:

| Profile | Target Environment | Tool Count | Description |
|---|---|:---:|---|
| **`McpProfile::Brain`** | AI IDEs / Coding Agents | 12 + Dispatch | High-density cognitive memory profile with L0→L3 lifecycle and meta-dispatcher. |
| **`McpProfile::Minimal`** | Constrained Prompts | 4 | Basic storage, search, and recall primitives. |
| **`McpProfile::Full`** | DB Administration | 24+ | Complete engine introspection, raw queries, and index maintenance. |

---

## 2. `McpProfile::Brain` Tool Surface

The Brain profile delivers high cognitive leverage with minimal token footprint:

### Memory Retrieval & Ingestion
- `vanta_remember`: Ingests conversation turns, notes, or structured facts into working memory (L0/L1).
- `vanta_recall`: High-speed hybrid semantic retrieval (BM25 + HNSW) tailored for agent context injection.
- `vanta_search`: Advanced multi-filter search supporting metadata predicates and namespace isolation.
- `vanta_forget`: Unified forgetting API supporting `soft`, `hard`, and `certified` deletion modes.

### Graph & Relationship Traversal
- `vanta_graph_query`: Traverses entity relationship graphs to uncover associative memories.
- `vanta_nodes_related`: Finds interconnected knowledge nodes within N degrees of separation.

### Cognitive Abstraction (L0→L3)
- `vanta_scene_synthesize`: Consolidates discrete turns and events into high-level episodic scenes (L2).
- `vanta_persona_reflect`: Extracts persistent persona traits, rules, and behavioral adaptations (L3).
- `vanta_export`: Exports memory scopes in JSONL interchange format.

### Governance & Meta-Dispatch
- `vanta_scopes_list`: Lists available memory partitions and authorization boundaries.
- `vanta_session_checkpoint`: Creates an atomic persistence checkpoint for current agent state.
- `vanta_stats`: Reports engine memory usage, resident vectors, and cache health.
- `vanta_dispatch`: Unified meta-tool router accepting dynamic sub-action payloads to conserve context tokens.

---

## 3. Configuration in AI IDEs

### Claude Code (`claude.json` / `settings.json`)

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--profile", "brain", "--db", "./.vanta/memory.db"]
    }
  }
}
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--profile", "brain"]
    }
  }
}
```
