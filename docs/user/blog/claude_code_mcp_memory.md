---
title: "VantaDB as Persistent Memory for Claude Code (MCP)"
version: 0.5.0
slug: claude-code-mcp-memory
date: 2026-06-26
author: "VantaDB Team"
tags: ["mcp", "claude-code", "ai-agents", "memory", "tutorial", "ide"]
description: "Wire VantaDB into Claude Code over MCP stdio so your coding agent remembers project decisions, past fixes, and architecture context across sessions."
tag: Tutorial
readTime: "7 min"
canonical: https://vantadb.vercel.app/blog/claude-code-mcp-memory
draft: true
---

# VantaDB as Persistent Memory for Claude Code (MCP)

*By the VantaDB Team*

Coding agents are amnesiacs. Every new Claude Code session starts from zero: re-read the architecture, re-discover the decision you documented three weeks ago, re-fix the bug with the same root cause. The context window is short-term memory. What it needs is a long-term one — project-scoped, searchable, and crash-safe.

VantaDB's MCP server (`vantadb-mcp`) is that memory. It runs as a stdio process your client spawns — no daemon, no port — and every memory operation becomes a tool the agent can call.

---

## 1. Starting the server

The canonical launcher is the CLI wrapper (see `docs/api/MCP.md` for the full reference, currently at MCP implementation version 0.5.0):

```bash
vanta-cli server --mcp --db ~/.vantadb
```

Requirements: the `vanta-cli` binary (0.5.0) on PATH and a writable directory for the database. The server speaks JSON-RPC 2.0 over stdin/stdout; tool definitions live in `vantadb-mcp/src/handlers/tools.rs`.

---

## 2. Registering it with your client

All MCP clients use the same server command; only the config file differs. Add this block to your client's MCP configuration (per-client paths are documented in `docs/api/MCP.md`):

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "~/.vantadb"]
    }
  }
}
```

Use an absolute path for `--db` if your client does not expand `~`. Restart the client and the memory tools appear alongside the built-in ones.

---

## 3. The memory workflow: namespaces per project

The pattern that works: one namespace per project, keyed records for decisions, hybrid search for recall.

- **Store:** after a significant decision, fix, or architectural discovery, have the agent store it — payload plus structured metadata (`kind: adr`, `area: auth`, `session: <id>`) plus a vector when semantic recall matters.
- **Recall:** at session start, the agent searches the project namespace for the current task area. BM25 finds the exact symbol names; HNSW finds the conceptually related notes; RRF fuses them.
- **Survive:** everything is WAL-persisted, so memory outlives the session, the reboot, and the laptop upgrade.

This is the same surface the Python SDK exposes (`put()`, `get()`, `search()`), now callable as agent tools. The agent stops re-deriving what the project already decided.

---

## Conclusion

Short-term memory is the context window. Long-term memory is a database with an MCP front door: one server command, one config block, one namespace per project.

```bash
vanta-cli server --mcp --db ~/.vantadb
```

Star the [VantaDB repository](https://github.com/ness-e/Vantadb) and join the Discord to share your memory setup. For the fully-offline variant of this pattern with local models, read [Local Agent Memory with Ollama + VantaDB](/blog/ollama-vantadb-local-memory).
