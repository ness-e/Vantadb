---
title: VantaDB Model Context Protocol (MCP) Server
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-08-05
aliases: []
---

# VantaDB Model Context Protocol (MCP) Server

## Overview

VantaDB provides a complete [[mcp|Model Context Protocol (MCP)]] server implementation that enables AI agents to interact with VantaDB through a standardized interface. The MCP server exposes tools, resources, and prompts for seamless integration with AI assistants and agents.

## Features

### Tools

The MCP server exposes the following tools for memory operations:

#### Memory CRUD Operations

- **`memory_put`** - Insert or update a memory record in a namespace
  - Parameters: `namespace`, `key`, `payload`, `vector` (optional), `metadata` (optional)
  - Returns: The created/updated memory record

- **`memory_get`** - Retrieve a memory record by namespace and key
  - Parameters: `namespace`, `key`
  - Returns: The memory record or error if not found

- **`memory_delete`** - Delete a memory record
  - Parameters: `namespace`, `key`
  - Returns: Success status

- **`memory_list`** - List memory records in a namespace with pagination
  - Parameters: `namespace`, `limit` (default: 100), `cursor` (optional), `filters` (optional)
  - `filters` accepts BOTH formats (AUD-048 — unified with the CLI channel): flat values `{"field": value}` (implicit `$eq`) **or** operator objects `{"field": {"$gt": value}}` (`$eq`, `$neq`, `$gt`, `$gte`, `$lt`, `$lte`)
  - Returns: Page of records with next cursor

- **`memory_list_namespaces`** - List all available namespaces
  - Parameters: None
  - Returns: List of namespace names

#### Search Operations

- **`search_memory`** - [[hybrid-search|Hybrid]] vector and text search
  - Parameters: `namespace`, `query_vector` (optional), `text_query` (optional), `top_k`, `distance_metric`, `explain`, `filters`, `search_profile` (optional)
  - `filters` accepts flat values `{"field": value}` **or** explicit equality `{"field": {"$eq": value}}` (identical equality semantics). Range operators (`$gt`/`$gte`/`$lt`/`$lte`/`$neq`) are NOT supported in search — the search request has no operator slot; use `memory_list` for those (AUD-048).
  - `search_profile` (MEM-02): optional object `{"mode": "keyword"|"vector"|"hybrid", "rrf_k": <int>, "candidate_k": <int>}` mirroring the native API and the IQL `PROFILE` clause. `mode` forces the retrieval channel; `rrf_k`/`candidate_k` tune RRF fusion (defaults = core constants). Bounds: `rrf_k` in `1..=max_rrf_k` (default 100), `candidate_k` in `1..=max_candidate_k` (default 10 000); out-of-bounds or unknown `mode` values return an explicit error.
  - Returns: Search hits with scores and optional explanations

- **`search_semantic`** - Raw [[hnsw|HNSW]] vector search
  - Parameters: `vector`, `k`
  - Returns: Nearest neighbors with `distance` (**lower is more similar**; for cosine, `distance = 1 − cosine_similarity`, so an identical vector reports `0.0`), plus `id` and `node`

#### Graph Operations

- **`query_iql`** - Execute an IQL (Interactive Query Language) statement (reads + mutations). LISP is not supported.
  - Parameters: `query`
  - Returns: Query results or execution status

- **`get_node_neighbors`** - Inspect node neighbors
  - Parameters: `node_id`
  - Returns: Node and its neighbors

- **`inject_context`** - Inject context into a thread
  - Parameters: `content`, `thread_id`
  - `thread_id` must be a numeric id (integer). A string (e.g. `"200"`) is rejected with a clear invalid-params error; the message distinguishes a missing field from a wrong type (AUD-050).
  - Returns: Context anchoring status

- **`read_axioms`** - Read system axioms
  - Parameters: None
  - Returns: Active Devil's Advocate Axioms

- **`rehydrate`** - Recover shadow-archived nodes that belonged to a summary node from TombstoneStorage
  - Parameters: `summary_id` (u128 as string)
  - Returns: `recovered_count`, `summary_id`, `rehydration_complete`

#### Collection Operations

- **`collection_stats`** - Returns statistics for a namespace/collection
  - Parameters: `namespace`
  - Returns: `total_records`, `total_bytes`, `has_vector_index`, `vector_count`, `created_at`

- **`collection_list`** - Lists all collections with metadata
  - Parameters: None
  - Returns: Array of `{name, record_count, has_vector_index, created_at}`

- **`collection_delete`** - Deletes an entire namespace/collection and all its records
  - Parameters: `namespace`, `confirm` (must be `"yes"`)
  - Returns: Deletion status

#### Skill Operations (MEM-07)

Versioned agent skills over the core `SkillStore`. All six tools take
`owner_agent` as caller identity (the embedded server has no HTTP auth layer):
a skill owned by a different agent responds exactly like a missing skill
(not found — no existence leak). Writes (`skill_update`, `skill_patch`,
`skill_files_write`) require `expected_version` (optimistic lock).

- **`skill_list`** - List skills owned by an agent
  - Parameters: `owner_agent` (required scope), `name_prefix` (optional), `limit` (default: 50), `offset`
  - Returns: `{items: [{skill_id, version, name, description}], total}`

- **`skill_view`** - Read a skill (head or a specific version) including its files
  - Parameters: `skill_id`, `owner_agent`, `version` (optional, defaults to head)
  - Returns: `{skill_id, version, name, description, content, files: [{path, content, encoding, mime_type, is_executable, size_bytes}]}`

- **`skill_create`** - Create a skill (version 1). Idempotent for the same owner, name and content; a different content under an existing name is a conflict.
  - Parameters: `name`, `owner_agent`, `content`, `description` (optional), `metadata` (optional), `ttl_secs` (optional)
  - Returns: `{ok, skill_id, version, idempotent}`

- **`skill_update`** - Replace a skill's head content, appending a new version
  - Parameters: `skill_id`, `owner_agent`, `expected_version`, `content`, `description` (optional; omitted keeps the current one)
  - Returns: `{ok, version, idempotent}`

- **`skill_patch`** - Substring replacement in a skill's content (TDAM-compatible)
  - Parameters: `skill_id`, `owner_agent`, `expected_version`, `old_string`, `new_string`, `replace_all` (required when `old_string` occurs more than once)
  - Returns: `{ok, version, idempotent}`

- **`skill_files_write`** - Write a resource file into a skill (stored in the skill's metadata manifest, versioned with the head)
  - Parameters: `skill_id`, `owner_agent`, `expected_version`, `path` (relative; no absolute paths, no `..` segments, no null bytes), `content`, `encoding` (`utf-8` default | `base64`), `mime_type` (optional), `is_executable` (optional)
  - Limits (configurable via `McpConfig`): 5 MB per resource, 50 MB total per skill (content + all files)
  - Returns: `{ok, version, idempotent}`

### Resources

The MCP server exposes the following resources:

- **`metrics://`** - Operational metrics
  - Memory usage, [[hnsw|HNSW]] statistics, storage information

- **`schema://`** - Database schema information
  - [[hnsw|HNSW]] configuration, text index version

- **`memory://{namespace}/{key}`** - Individual memory records
  - Access specific memory records by URI

- **`namespace://{namespace}`** - Namespace content
  - List records in a namespace

### Prompts

The MCP server provides the following prompt templates:

- **`search_memory`** - Optimized prompt for memory search
- **`analyze_namespace`** - Analyze namespace content and structure
- **`summarize_context`** - Generate context summaries
- **`query_builder`** - Build IQL queries

## Usage

### Starting the MCP Server

The MCP server runs as a stdio JSON-RPC server via the CLI:

```bash
# Using the VantaDB CLI with MCP mode
vanta-cli server --mcp --db ./vanta_data

# Or from source
cargo run --bin vanta-cli -- server --mcp --db ./vanta_data
```

### Per-IDE Setup

Configure your MCP client to connect to VantaDB:

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "/path/to/vantadb"]
    }
  }
}
```

Below are IDE-specific instructions.

### Cursor

1. Open Cursor Settings → **Features** → **MCP Servers**
2. Click **Add new MCP server**
3. Fill in:
   - **Name:** `vantadb`
   - **Type:** `command`
   - **Command:** `vanta-cli server --mcp --db ~/.vantadb`
4. Click **Save**

The MCP server will start automatically when Cursor needs it. If `vanta-cli` is not in PATH, use the full path (e.g., `~/.cargo/bin/vanta-cli`).

### Claude Code

Add to your project's `.claude/settings.json` (or `~/.claude/settings.json` for global):

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "/path/to/vantadb"]
    }
  }
}
```

The server starts automatically when Claude Code needs to use a VantaDB tool.

### Windsurf

1. Open Windsurf Settings → **AI** → **MCP Servers**
2. Click **Add Server**
3. Fill in:
   - **Name:** `vantadb`
   - **Command:** `vanta-cli`
   - **Arguments:** `server --mcp --db ~/.vantadb`
4. Click **Save**

### OpenCode

Add to your `opencode.json` (project root or `~/.config/opencode/opencode.json`):

```json
{
  "mcp": {
    "vantadb": {
      "type": "local",
      "command": ["vanta-cli", "server", "--mcp", "--db", "~/.vantadb"],
      "enabled": true
    }
  }
}
```

### Cline (VS Code)

Configure in VS Code settings (`settings.json`):

```json
{
  "cline.mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "~/.vantadb"]
    }
  }
}
```

### Notes

- **First time?** Install the CLI: `cargo install vanta-cli` or download the binary from [releases](https://github.com/ness-e/Vantadb/releases).
- **Cross-IDE:** VantaDB's MCP server can run simultaneously across multiple IDEs — each connects independently to the same database path.
- **Custom binary path:** If `vanta-cli` is not in PATH, replace with the full path (e.g., `~/.cargo/bin/vanta-cli`).
- **Windows:** Use forward slashes or escaped backslashes for paths (e.g., `C:/Users/me/.vantadb`).

### Example Tool Calls

#### Store a Memory

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "memory_put",
    "arguments": {
      "namespace": "agent/session-1",
      "key": "ctx-001",
      "payload": "User prefers concise technical answers",
      "vector": [0.8, 0.1, 0.5],
      "sparse_vector": { "0": 0.5, "7": 1.25 },
      "metadata": {
        "type": "preference",
        "priority": 2
      },
      "expires_at_ms": 1893456000000
    }
  }
}
```

#### Search Memory

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "search_memory",
    "arguments": {
      "namespace": "agent/session-1",
      "text_query": "technical answers",
      "top_k": 5
    }
  }
}
```

#### Read a Resource

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "resources/read",
  "params": {
    "uri": "metrics://"
  }
}
```

## Architecture

The MCP server implementation:

1. **JSON-RPC 2.0 Protocol** - Standard JSON-RPC over stdio
2. **Async/Sync Bridge** - Tokio async runtime with blocking sync operations
3. **Semaphore Concurrency Control** - Configurable thread limits
4. **Error Handling** - Structured error codes and messages
5. **Type Safety** - Rust type system ensures data integrity

## Integration Examples

## Integration Examples

- **Latency**: Sub-millisecond for in-process operations
- **Throughput**: Configurable via semaphore limits
- **Memory**: Embedded mode with configurable limits
- **Persistence**: Zero-copy MMAP for vector operations

## Security

- **Namespace Isolation** - Separate memory spaces per agent
- **Read-Only Mode** - Optional read-only operation mode
- **Resource Governance** - Configurable memory and thread limits

## Troubleshooting

### Connection Issues

Ensure the VantaDB server is running and the MCP client is configured with the correct path.

### Permission Errors

Check that the database path is writable and that the user has appropriate filesystem permissions.

### Performance Issues

Adjust the `max_blocking_threads` configuration in VantaConfig to optimize for your workload.

## Future Enhancements

- Streaming responses for large result sets
- Batch operations for bulk inserts/deletes
- Advanced metadata querying
- Real-time change notifications
- Resource watching capabilities

## Version

Current MCP implementation version: 0.5.0

Protocol version: 2024-11-05

## Support

For issues and questions:
- GitHub Issues: https://github.com/ness-e/Vantadb/issues
- Documentation: https://vantadb.dev
