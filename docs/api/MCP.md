---
title: VantaDB Model Context Protocol (MCP) Server
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-07-10
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
  - Returns: Page of records with next cursor

- **`memory_list_namespaces`** - List all available namespaces
  - Parameters: None
  - Returns: List of namespace names

#### Search Operations

- **`search_memory`** - [[hybrid-search|Hybrid]] vector and text search
  - Parameters: `namespace`, `query_vector` (optional), `text_query` (optional), `top_k`, `distance_metric`, `explain`, `filters`
  - Returns: Search hits with scores and optional explanations

- **`search_semantic`** - Raw [[hnsw|HNSW]] vector search
  - Parameters: `vector`, `k`
  - Returns: Nearest neighbors with distances

#### Graph Operations

- **`query_lisp`** - Execute VantaLISP code
  - Parameters: `query`
  - Returns: Query results or execution status

- **`get_node_neighbors`** - Inspect node neighbors
  - Parameters: `node_id`
  - Returns: Node and its neighbors

- **`inject_context`** - Inject context into a thread
  - Parameters: `content`, `thread_id`
  - Returns: Context anchoring status

- **`read_axioms`** - Read system axioms
  - Parameters: None
  - Returns: Active Devil's Advocate Axioms

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

- **First time?** Install the CLI: `cargo install vantadb-cli` or download the binary from [releases](https://github.com/ness-e/Vantadb/releases).
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
      "metadata": {
        "type": "preference",
        "priority": 2
      }
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

Current MCP implementation version: 0.1.5

Protocol version: 2024-11-05

## Support

For issues and questions:
- GitHub Issues: https://github.com/ness-e/Vantadb/issues
- Documentation: https://vantadb.dev
