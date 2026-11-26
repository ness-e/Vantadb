---
title: VantaDB as Persistent Memory for Claude Code (MCP)
kind: howto
status: active
description: Configure the VantaDB MCP launcher and use project-scoped memory with explicit operational limits
tags: [mcp, claude-code, memory]
---

# VantaDB as Persistent Memory for Claude Code (MCP)

*By the VantaDB Team*

Coding agents benefit from project-scoped memory that can be retrieved across
sessions. VantaDB supplies persistent storage and search; the agent and host
still decide which information to store and retrieve.

VantaDB's MCP server (the `vantadb-mcp` crate) exposes a profile-dependent tool
surface through a stdio process your client spawns.

---

## 1. Starting the server

The launcher is the CLI (see `docs/api/MCP.md` for the full reference):

```bash
vanta-cli server --mcp --db C:/absolute/path/to/db
```

Requirements: a `vanta-cli` build with MCP support on PATH and a writable
database directory. Replace the example database path with an absolute path
appropriate to your operating system. Select the profile explicitly with
`VANTADB_MCP_PROFILE`; the configuration below uses `brain`. The server speaks
JSON-RPC 2.0 over stdin/stdout. See [the MCP reference](../../api/MCP.md) for the
tool names and profile behavior.

---

## 2. Registering it with your client

All MCP clients use the same server command; only the config file differs. Add this block to your client's MCP configuration (per-client paths are documented in `docs/api/MCP.md`):

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "C:/absolute/path/to/db"],
      "env": {"VANTADB_MCP_PROFILE": "brain"}
    }
  }
}
```

Restart the client and the memory tools appear alongside the built-in ones.

---

## 3. The memory workflow: namespaces per project

The pattern that works: one namespace per project, keyed records for decisions, hybrid search for recall.

- **Store:** after a significant decision, fix, or architectural discovery, have the agent store it — payload plus structured metadata (`kind: adr`, `area: auth`, `session: <id>`) plus a vector when semantic recall matters.
- **Recall:** at session start, the agent searches the project namespace for the current task area. BM25 finds the exact symbol names; HNSW finds the conceptually related notes; RRF fuses them.
- **Persist:** use the configured durability controls and keep database backups.
  Recovery, upgrades, and migration to another machine are separate concerns;
  checksummed WAL does not guarantee survival of every failure.

The concepts correspond to Python `put()`, `memory.get()`, and `search()`, but MCP
tool names and argument shapes differ. Namespace selection is organizational
scoping, not authorization; apply host and filesystem access controls as needed.

---

## Conclusion

Short-term memory is the context window. Long-term memory is a database with an MCP front door: one server command, one config block, one namespace per project.

```bash
vanta-cli server --mcp --db C:/absolute/path/to/db
```

Star the [VantaDB repository](https://github.com/ness-e/Vantadb) and join the Discord to share your memory setup. For the fully-offline variant of this pattern with local models, read [Local Agent Memory with Ollama + VantaDB](./ollama_vantadb_local_memory.md).
