---
title: VantaDB Model Context Protocol (MCP) Server
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-08-23
aliases: []
---

# VantaDB Model Context Protocol (MCP) Server

The VantaDB MCP server (`vantadb-mcp`) exposes the database to LLM agents over the Model Context Protocol. Tool definitions live in `vantadb-mcp/src/handlers/tools.rs` (`handle_tools_list`); per-IDE setup lives in the VantaDB MCP Skill.

## Getting Started

VantaDB runs as a **stdio JSON-RPC 2.0 server**: your MCP client spawns the process, talks to it over stdin/stdout, and every memory operation (store, search, graph, wiki, skills) becomes a tool the agent can call. No daemon, no network port.

### Requirements

- The VantaDB CLI (`vanta-cli` ≥ **0.5.0**) installed and on PATH — see [Embedded CLI](../../README.md#embedded-cli) for one-line installers.
- A writable directory for the database (e.g. `~/.vantadb`).
- An MCP-capable client: Claude Desktop, Claude Code, Cursor, OpenCode, or any client speaking MCP over stdio.

### Starting the server

The canonical launcher is the CLI wrapper:

```bash
vanta-cli server --mcp --db ~/.vantadb
```

(Advanced: run `vantadb-server --mcp` directly with `VANTADB_STORAGE_PATH=~/.vantadb`.)

### Client configuration

All three clients use the same server command; only the config file differs. Use an **absolute path** for `--db` if your client does not expand `~`.

**Claude Desktop** — `%APPDATA%\Claude\claude_desktop_config.json` (Windows) / `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

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

**Cursor** — `~/.cursor/mcp.json` (global) or `.cursor/mcp.json` (project):

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

**Claude Code** — project-level `.mcp.json`, or one-shot via CLI:

```bash
claude mcp add vantadb -- vanta-cli server --mcp --db ~/.vantadb
```

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

### First test

Verify the handshake (`initialize` → `tools/list`) works before touching your editor config:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | vanta-cli server --mcp --db ~/.vantadb
```

You should see a JSON response listing the available tools. Namespaces are created implicitly on first write (`memory_put` with a new namespace); list existing ones with `collection_list`.

### Troubleshooting

| Symptom | Fix |
|---------|-----|
| Client shows no tools | Run the First test above; if it fails, check that `vanta-cli` is on PATH (`vanta-cli --version`). |
| `Server closed stdout` / immediate exit | The database path must be writable and not locked by another VantaDB process. |
| Tools error at call time | Check disk space and permissions on the `--db` directory; use `capabilities` to introspect the engine state. |
| Text search fails with `text_index not found` | Restarting the server reconciles indexes automatically at startup (`ensure_indexes_current`). |

For the full behavioral contract (error channels, response envelope, edge cases), see the VantaDB MCP Skill.

## Tool Families

**76 tools in 7 families (spec 2025-06-18, every tool carries `annotations` with `title`, `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint` per [MCP Tool Annotations](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) / [blog 2026-03-16](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations)):**

| Family | Count | Source module |
|--------|-------|---------------|
| Core | 46 | `handlers/tools.rs` — listed in `tools/list` |
| `code_*` | 8 | `code.rs` |
| `skill_*` | 6 | `skills.rs` |
| `wiki_*` | 6 | `wiki.rs` |
| `context_assemble` | 1 | `context.rs` |
| `scene_*` | 3 | `scenes.rs` |
| `thread_*` | 6 | `threads.rs` |

> Annotations are display hints (untrusted, not enforcement): `readOnlyHint` true = no persistent mutation, `destructiveHint` true = may delete/overwrite (11 tools), `idempotentHint` true = retry-safe, `openWorldHint` true = host filesystem (wiki_ingest, bulk_import_file only). Clients that ignore annotations assume pessimistic defaults.

## Tool Surface Profiles (MCP-37)

The VantaDB MCP server exposes a **tool surface profile** via the `VANTADB_MCP_PROFILE` environment variable. This allows clients with tool caps (e.g., Cursor ~40 tools) to select a subset that fits their limits while preserving full functionality for unrestricted clients.

| Profile | Tool Count | Description | Recommended For |
|---------|------------|-------------|-----------------|
| `full` (default) | 76 | All tools: memory, graph, collections, maintenance, snapshots, backup, introspection, code intelligence, wiki, skills, threads, scenes, context engine. | Claude Desktop, Claude Code, OpenCode, unrestricted clients |
| `dev` | ~35 | Memory CRUD + search + IQL + graph traversal + collections + key maintenance (snapshots, export/import, flush, compact) + axioms. Excludes: code intelligence, wiki, skills, threads, scenes, context engine, bulk import, index audit/repair, vacuum, rebuild_index. | **Cursor** (cap ~40), VS Code extensions, clients with moderate tool caps |
| `memory` | ~18 | Core memory CRUD (put/get/delete/list/versions/supersede) + search (semantic/memory/with_method/multi) + IQL + collections + capabilities + generate_snippet. | Memory-only agents, minimal clients, testing |

**Usage:**

```bash
# Full profile (default)
vanta-cli server --mcp --db ~/.vantadb

# Dev profile (recommended for Cursor)
VANTADB_MCP_PROFILE=dev vanta-cli server --mcp --db ~/.vantadb

# Memory-only profile
VANTADB_MCP_PROFILE=memory vanta-cli server --mcp --db ~/.vantadb
```

**Client Configuration (Cursor):**

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "~/.vantadb"],
      "env": { "VANTADB_MCP_PROFILE": "dev" }
    }
  }
}
```

**Behavior:**
- The profile is read once at server startup from `VANTADB_MCP_PROFILE`.
- `tools/list` returns only the tools allowed by the selected profile.
- `tools/call` for a non-listed tool returns `method_not_found` with a clear error: `Tool not found: <name> (not in profile <profile>)`.
- Profile `full` preserves backward compatibility — existing clients see all 76 tools by default.

## Core Tools (46)

### Memory CRUD (9)

| Tool | Description |
|------|-------------|
| `memory_put` | Inserts or updates a memory record in a namespace with payload, vector, optional sparse vector, metadata, and TTL. |
| `memory_put_batch` | Stores multiple records in a single all-or-nothing batch; duplicate keys are upserts, vector dimensions must match the live index. |
| `memory_get` | Retrieves a memory record by namespace and key. |
| `memory_delete` | Deletes a memory record by namespace and key. |
| `memory_delete_by_filter` | Batch-deletes every record in a namespace whose metadata matches the given filters (AND semantics). |
| `memory_list` | Lists memory records in a namespace with optional pagination and metadata filters. Response is bounded by `byte_budget` (default 40 KB); see [Output budgeting](#output-budgeting-byte_budget-mcp-39) for the truncation semantics. |
| `memory_list_namespaces` | Lists all available namespaces in the database. |
| `memory_versions` | Lists every retained version of a memory record, ascending (v1..vN); empty if the key does not exist or has no history. Expired versions are included as historical data until purged. |
| `memory_supersede` | Marks an existing record as superseded by another existing record (durable, recoverable soft-delete). Errors if either key is missing, if old_key equals new_key, or if the old record is already superseded. |

### Search & Query (7)

| Tool | Description |
|------|-------------|
| `search_memory` | Hybrid memory search in a namespace: text/vector/hybrid modes, filters, distance metric, RRF tuning, and explain output. |
| `search_semantic` | Raw semantic vector search directly in the HNSW index. |
| `search_with_method` | Memory search with an explicit dense-index backend override (`method`: hnsw \| ivf \| flat \| diskann \| scann); omit to keep automatic routing. Same parameters as `search_memory`. |
| `search_multi` | Run one search request across multiple namespaces and merge results (sorted by score, capped at `top_k` globally). Response is bounded by `byte_budget` (default 40 KB); see [Output budgeting](#output-budgeting-byte_budget-mcp-39). |
| `query_iql` | Executes an IQL statement against typed graph nodes and memory namespaces (each namespace is queryable as a table named by its sanitized form: `/` and `-` → `_`, leading digit/`.` gets a `_` prefix). LISP not supported. |
| `memory_search` | MEM-59: Semantic alias of `search_memory` with the canonical agent-friendly name (mem0/Letta parity). Same wire shape and engine path as `search_memory`; both tools share the same dispatch so behavior cannot diverge. |
| `memory_recall` | MEM-59: High-level recall mirroring vanta-memory's auto-recall hook (MEM-18) over the public MCP surface. Runs keyword/embedding/hybrid search over L1 records visible under the given scope (session/agent/team), ranks with D38 dual-pool + RRF logic, and returns structured hits plus prepended context block. Read-only; idempotent; does not require a session_key. |

### Graph (7)

| Tool | Description |
|------|-------------|
| `get_node_neighbors` | Inspects neighbors or lineage of a node. |
| `graph_page_rank` | Computes PageRank over the subgraph reachable from the given root nodes. |
| `graph_degree_centrality` | Incoming/outgoing edge counts for every node reachable from the given roots. |
| `graph_traverse` | Multi-hop BFS/DFS traversal from start nodes with optional edge-label and temporal filters. |
| `graph_topological_sort` | Topological sort of the subgraph reachable from the given roots; errors on cycles. |
| `graph_is_dag` | Returns whether the subgraph reachable from the given roots is a directed acyclic graph. |
| `remove_edge` | Removes all edges between two nodes with the given label (both directions). Node ids are u128 decimal strings (JSON numbers lose precision above 2^53). |

### Context & Axioms (4)

| Tool | Description |
|------|-------------|
| `inject_context` | Injects external state or context connected to a specific thread for subsequent consolidation. |
| `read_axioms` | Returns the active Devil's Advocate Axioms (Iron Axioms) in the database. |
| `write_axiom` | Registers or updates an agent axiom (invariant rule) in the reserved `_axioms` namespace; returns `{id, name, description}`. |
| `delete_axiom` | Removes an agent axiom by name from the `_axioms` namespace; returns `{deleted}`. |

### Collections (3)

| Tool | Description |
|------|-------------|
| `collection_list` | Lists all collections with record count, vector index status, and creation time. |
| `collection_stats` | Statistics for one namespace/collection: count, byte size, index info, creation time. |
| `collection_delete` | Deletes an entire namespace/collection and all its records (requires `confirm: "yes"`). |

### Maintenance, Indexes & Snapshots (12)

| Tool | Description |
|------|-------------|
| `flush` | Flushes WAL and memory-mapped files to disk as a manual durability checkpoint. |
| `compact_wal` | Archives the current WAL file and starts a fresh one to reclaim WAL space. |
| `compact_layout` | Compacts the vector store file grouping nodes in BFS order from the HNSW entry point; returns bytes reclaimed. |
| `rebuild_index` | Rebuilds the HNSW vector index, derived indexes, and text index from scratch (recovery primitive). |
| `audit_text_index` | Read-only integrity audit of the derived persistent text index vs canonical records; `deep=true` verifies postings/tf/stats. |
| `repair_text_index` | Repairs the text index by rebuilding it from canonical storage (use when `audit_text_index` reports drift). |
| `snapshot_create` | Creates a filesystem snapshot of the storage directory under `<data_dir>/snapshots/<name>`; returns the snapshot path. |
| `snapshot_restore` | DESTRUCTIVE: replaces the live database directory with the contents of snapshot `<name>` (requires `confirm: true`; `name` must be a plain identifier). Current data is staged aside with rollback-on-failure; snapshots survive the swap. The running engine must be restarted to serve restored state. |
| `list_snapshots` | Lists physical snapshot names under `<data_dir>/snapshots`. |
| `purge_expired` | Scans all records and physically deletes those whose TTL expiry has passed. |
| `rehydrate` | Recovers shadow-archived nodes that belonged to a summary node from TombstoneStorage. |
| `vacuum` | Purges tombstoned nodes from the HNSW index; returns a report with scanned_nodes, removed_nodes, reclaimed_bytes, duration_ms, and success. |

### Introspection & Utility (2)

| Tool | Description |
|------|-------------|
| `capabilities` | Introspects supported engine features: runtime profile, persistence, vector search, IQL queries, read-only mode. |
| `generate_snippet` | Generates a text snippet from a payload, optionally highlighting matched query terms. |

### Backup & Bulk Import (4)

| Tool | Description |
|------|-------------|
| `export` | Exports memory records as JSONL (max 10 MB per call); pair with `import` for backup/restore. |
| `import` | Imports records from JSONL produced by `export`; malformed lines are counted as errors, not fatal. |
| `bulk_import_file` | Bulk-imports from a binary `.vdbdump` file on the host filesystem, bypassing per-record validation for throughput. |
| `bulk_import_stream` | Bulk-imports inline NDJSON or raw `.vdbdump` content (max 10 MB); imported entries are raw engine nodes. |

## Extended Tool Families (30)

Dispatched via `tools/call`, defined outside `handlers/tools.rs` (8+6+6+3+6+1 = 30):

### Code Intelligence — `code.rs` (8)

| Tool | Description |
|------|-------------|
| `code_search` | Searches indexed code symbols. |
| `code_explore` | Explores symbols with call paths and blast radius. |
| `code_callers` | Lists callers of a symbol. |
| `code_callees` | Lists callees of a symbol. |
| `code_impact` | Impact analysis for a change target. |
| `code_node` | Fetches a single code-graph node. |
| `code_files` | Lists indexed files. |
| `code_status` | Index health/status of the code graph. |

### Skills Management — `skills.rs` (6)

| Tool | Description |
|------|-------------|
| `skill_list` | Lists installed agent skills. |
| `skill_view` | Views a skill's content. |
| `skill_create` | Creates a new skill. |
| `skill_update` | Updates an existing skill. |
| `skill_patch` | Applies targeted patches to a skill. |
| `skill_files_write` | Writes supporting files for a skill. |

### Wiki — `wiki.rs` (6)

| Tool | Description |
|------|-------------|
| `wiki_ingest` | Ingests documents into the wiki knowledge base. |
| `wiki_ingest_status` | Reports status of a wiki ingest run. |
| `wiki_read` | Reads a wiki page/node. |
| `wiki_search` | Searches the wiki corpus. |
| `wiki_graph` | Queries the wiki knowledge graph. |
| `wiki_list` | Lists wiki pages/nodes. |

### Context Engine — `context.rs` (1)

| Tool | Description |
|------|-------------|
| `context_assemble` | Assembles a context window under a token budget with the vanta-memory context engine (MCP-31): compacts the provided chat history and injects session recall (relevant L1 memories, persona, scene navigation). Returns `{messages, report, mmd_injected, recall_injected}`. Read-only. |

### Scenes API - `scenes.rs` (3)

Read-only wrappers over the vanta-memory gateway scene handlers (`vanta_memory::gateway`) — structured scene navigation for external agents. Domain errors surface as error-content messages; `scene_query` ranks by keyword overlap only.

| Tool | Description |
|------|-------------|
| `scene_read` | Reads one live scene block by name from a session's scene store. Returns `{scene:{scene_name, meta{created,updated,summary,heat}, content}}`. Missing or soft-deleted scenes answer "not found". Read-only. |
| `scene_list` | Lists the scene index of a session (heat descending, soft-deleted excluded). Returns `{scenes:[{filename,summary,heat,created,updated}]}` where `filename` is the id for `scene_read`. Read-only. |
| `scene_query` | Keyword search over live scene blocks: ranks scenes by term overlap between the keyword and summary+content, ties by heat. Returns `{hits:[{scene_name,summary,heat,updated,score}]}`; load hits via `scene_read`. Read-only. |

## Output budgeting (`byte_budget`, MCP-39)

Large list / search responses can exceed the per-message cap of popular MCP clients
(Claude Code 25k tokens ~ 100 KB, OpenCode 2000 lines / 50 KB). Without budgeting
the JSON gets truncated silently — the client does not know that data is missing.
MCP-39 wraps the affected tools in a byte-budget guard so truncation is explicit
and clients can react.

### Knob

| Env var | Default | Floor | Ceiling | Description |
|---------|---------|-------|---------|-------------|
| `VANTADB_MCP_BYTE_BUDGET` | `40 * 1024` (40 KB) | `1 KB` | `1 MB` | Target response envelope size. Set lower for tight clients; do not exceed the chosen client's cap. |

Read at server startup via `McpConfig::from_storage`; clamped to `[min_byte_budget, max_byte_budget]`. Changing the value requires a server restart.

### Truncation semantics

| Tool | Shape on the wire | Truncation policy |
|------|-------------------|-------------------|
| `memory_list` | `content[0].text` is a JSON object `{records, next_cursor, byte_count, truncated}` | Trailing `records` entries are popped until the envelope fits `byte_budget`. `next_cursor` is preserved; `truncated: true` advertises the trim. If the array is fully popped, the `records` key is dropped (consumers should treat absent `records` as "hard-truncated"). |
| `search_multi` | `content[0].text` stays the raw hits array (back-compat); `structuredContent` carries `{hits, byte_count, truncated}` | Trailing `hits` are popped in both the text and the structuredContent copy. `truncated: true` flags the trim. `search_memory` / `search_semantic` / `search_with_method` are NOT budgeted in this release — their `top_k` cap is the documented upper bound; tracked as debt for a follow-up. |

### When to react

| `truncated` | Action |
|------------|--------|
| `false` | Full response delivered; `byte_count` reports the envelope size. |
| `true` (object shape: `memory_list`) | Refine the filter / lower `limit` / narrow the `cursor` window. |
| `true` (array shape: `search_multi`) | Narrow `namespaces` to fewer entries, or lower `top_k`. |

### Example (`memory_list`, default budget)

```json
{
  "records": [ { "key": "doc-1", "payload": "..." }, ... ],
  "next_cursor": 200,
  "byte_count": 38921,
  "truncated": false
}
```

After oversize trimming, `truncated` flips to `true` and the last items are dropped from `records`. `next_cursor` remains the next-page marker so the consumer can keep paging.

## Parity

Tool coverage on this page is enforced mechanically by `scripts/validate-docs-coverage.ps1` against `handle_tools_list()` in `vantadb-mcp/src/handlers/tools.rs`. Last sync: **2026-08-23**.

## Registry manifest

VantaDB MCP is published to the [Official MCP Registry](https://registry.modelcontextprotocol.io/)
under the namespace `io.github.ness-e/vantadb`. The canonical descriptor lives at
the repo root: [`/server.json`](../../server.json) (schema version `2025-12-11`).

| Field | Value |
|-------|-------|
| `name` | `io.github.ness-e/vantadb` |
| `version` | tracks `[workspace.package].version` (`0.5.0` as of this doc) |
| `transport` | stdio (no remote service; no published `packages[]` — install via `cargo install --git`) |
| `repository` | [`ness-e/Vantadb`](https://github.com/ness-e/Vantadb) |

For submission state, glama/smithery aggregator status, and the
`server.json` regeneration procedure (per release), see
[`/docs/operations/MCP_REGISTRY.md`](../operations/MCP_REGISTRY.md).

> Pre-mortem: the registry submission PR is a **manual** step (not a CI gate).
> Until it's approved, the `server.json` lives in-repo as a discoverable descriptor;
> the binary itself is the source of truth. See
> [`MCP_REGISTRY.md#submission-state`](../operations/MCP_REGISTRY.md#submission-state)
> for the current PR/approval state.
