---
name: vantadb-mcp
description: VantaDB Model Context Protocol (MCP) server integration for persistent AI agent memory. Use when an AI agent (OpenCode, Claude, Cursor, etc.) needs to work with VantaDB as a memory backend through MCP for: (1) Storing and retrieving persistent memory records, (2) Performing hybrid vector and text search, (3) Managing namespace-scoped memory isolation, (4) Accessing operational metrics, schema information, and collection statistics, (5) Executing IQL graph queries and rehydrating archived nodes, (6) Working with AI frameworks like CrewAI, Mem0, AutoGen, Haystack, LangGraph, Semantic Kernel, or DSPy that require persistent memory storage.
---

# VantaDB MCP Integration

VantaDB provides a complete MCP (Model Context Protocol) server implementation for persistent memory storage with hybrid vector and text search capabilities. The MCP server exposes 15 tools, 2 resources, and 4 prompt templates over stdio JSON-RPC 2.0.

## Quick Start

### Installation

Run the setup script to install VantaDB:

```bash
bash scripts/setup-vantadb.sh
```

This installs VantaDB and creates default configuration.

### Starting the MCP Server

The VantaDB MCP server runs as a stdio JSON-RPC server. Use the CLI wrapper (canonical — it spawns the server with the database path):

```bash
vanta-cli server --mcp --db ~/.vantadb
```

Or run the server binary directly. `vantadb-server` has no `--db`/`--path` flags; the storage path comes from the `VANTADB_STORAGE_PATH` environment variable:

```bash
VANTADB_STORAGE_PATH=~/.vantadb vantadb-server --mcp
```

> Note: `vanta-server` is not a real binary and `--path` is not a valid flag on any VantaDB binary. Use `vanta-cli server --mcp --db <path>` or `vantadb-server --mcp` with `VANTADB_STORAGE_PATH`.

### MCP Client Configuration

Configure your MCP client to connect to VantaDB:

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

**Pre-configured templates available in assets/:**
- `assets/claude-desktop-config.json` - Claude Desktop configuration
- `assets/cursor-config.json` - Cursor workspace configuration
- `assets/config-template.json` - VantaDB configuration template

### OpenCode

The user's primary editor is OpenCode. Add to your `opencode.json` (project root or `~/.config/opencode/opencode.json`):

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

### Testing

Test the MCP server:

```bash
python scripts/test-mcp.py
```

### Namespace Management

Create namespaces for isolation:

```bash
python scripts/create-namespace.py create agent/session-001
python scripts/create-namespace.py list
```

## Available MCP Tools (15)

### Memory CRUD Operations

**memory_put** - Insert or update a memory record
- Parameters: `namespace`, `key`, `payload` (required); `vector` (optional array of numbers), `metadata` (optional object)
- Returns: The created/updated memory record

**memory_get** - Retrieve a memory record
- Parameters: `namespace`, `key`
- Returns: Memory record or error if not found

**memory_delete** - Delete a memory record
- Parameters: `namespace`, `key`
- Returns: Success status (`{"deleted": true|false}`)

**memory_list** - List memory records with pagination
- Parameters: `namespace`, `limit` (default: 100), `cursor` (optional number), `filters` (optional object)
- Returns: `{"records": [...], "next_cursor": ...}`

**memory_list_namespaces** - List all namespaces
- Parameters: None
- Returns: List of namespace names

### Search Operations

**search_memory** - Hybrid vector and text search in a namespace
- Parameters: `namespace` (required), `query_vector` (optional array), `text_query` (optional string), `top_k` (default: 10), `distance_metric` (`cosine` | `euclidean`, default: `cosine`), `explain` (optional boolean), `filters` (optional object)
- Returns: Search hits with scores and optional explanations

**search_semantic** - Raw HNSW vector search
- Parameters: `vector` (F32 query vector, required), `k` (default: 5)
- Returns: Nearest neighbors with distances

### Graph Operations

**query_iql** - Execute an IQL (Interactive Query Language) statement. Allows reading structures and inserting/mutating Nodes providing semantic context. **LISP is not supported; statements must be IQL.**
- Parameters: `query`
- Returns: Query results or execution status (read nodes, write result, or stale-context rehydration hint)

**get_node_neighbors** - Inspect node relationships
- Parameters: `node_id` (decimal string; u128 ids exceed JSON number precision)
- Returns: Node and its neighbors

**inject_context** - Inject context into a thread
- Parameters: `content`, `thread_id`
- Returns: Context anchoring status

**read_axioms** - Read system axioms
- Parameters: None
- Returns: Active Devil's Advocate Axioms (Iron Axioms)

**rehydrate** - Recover shadow-archived nodes that belonged to a summary node from TombstoneStorage
- Parameters: `summary_id` (u128 as string)
- Returns: `{"recovered_count", "summary_id", "rehydration_complete"}`

### Collection Operations

**collection_stats** - Returns statistics for a namespace/collection
- Parameters: `namespace`
- Returns: `{"total_records", "total_bytes", "has_vector_index", "vector_count", "created_at"}`

**collection_list** - Lists all collections with metadata
- Parameters: None
- Returns: Array of `{"name", "record_count", "has_vector_index", "created_at"}`

**collection_delete** - Deletes an entire namespace/collection and all its records
- Parameters: `namespace`, `confirm` (must be `"yes"`)
- Returns: `{"deleted", "records_removed"}`

## Available MCP Resources

The server lists 2 static resources in `resources/list`; 2 additional dynamic URIs are servable via `resources/read` (not listed):

- **metrics://** - Operational metrics (memory usage, HNSW statistics, storage information) — listed
- **schema://** - Database schema information (HNSW configuration, text index version) — listed
- **memory://{namespace}/{key}** - Individual memory records by URI — servable, not listed
- **namespace://{namespace}** - Namespace content listing — servable, not listed

## Available MCP Prompts

- **search_memory** - Optimized prompt for memory search
- **analyze_namespace** - Analyze namespace content and structure
- **summarize_context** - Generate context summaries
- **query_builder** - Build IQL queries

## Namespace Isolation

Use namespaces to isolate memory by context:

- Per-agent namespaces: `agent/{agent-id}`
- Per-session namespaces: `session/{session-id}`
- Per-project namespaces: `project/{project-name}`
- Global namespace: `global`

## Metadata Filtering

Use metadata to organize and filter memories:

```json
{
  "type": "preference",
  "category": "user",
  "priority": "high"
}
```

Filter during search or list operations to retrieve specific subsets.

## Hybrid Search

VantaDB supports both vector and text search:

- **Vector search**: Use `query_vector` parameter for semantic similarity
- **Text search**: Use `text_query` parameter for BM25 lexical search
- **Hybrid search**: Provide both for combined ranking

## AI Framework Integrations

VantaDB provides Python SDK integrations for popular AI frameworks:

- **CrewAI**: See [examples/python/crewai_memory.py](../../examples/python/crewai_memory.py)
- **Mem0**: See [examples/python/mem0_integration.py](../../examples/python/mem0_integration.py)
- **AutoGen**: See [examples/python/autogen_memory.py](../../examples/python/autogen_memory.py)
- **Haystack**: See [examples/python/haystack_documentstore.py](../../examples/python/haystack_documentstore.py)
- **LangGraph**: See [examples/python/langgraph_checkpoint.py](../../examples/python/langgraph_checkpoint.py)
- **Semantic Kernel**: See [examples/python/semantic_kernel_memory.py](../../examples/python/semantic_kernel_memory.py)
- **DSPy**: See [examples/python/dspy_retriever.py](../../examples/python/dspy_retriever.py)

## Editor Integration

For per-IDE setup (Cursor, Claude Code, Windsurf, OpenCode, Cline, VS Code), see [docs/api/MCP.md](../../docs/api/MCP.md). This is the source of truth for the MCP contract; this skill stays consistent with it.

Supported editors:
- Cursor
- VS Code (with MCP-compatible extension)
- OpenCode
- OpenClaw
- Devin
- Antigravity

## Performance Optimization

- Configure memory limits via environment variables (see [references/configuration.md](references/configuration.md))
- Use namespace isolation to limit scope
- Adjust HNSW parameters for memory efficiency
- Implement periodic cleanup of old memories

## Security

- Use namespace isolation for different contexts
- Consider read-only mode for production deployments
- Implement access control at the editor level
- Audit memory access logs

## Troubleshooting

**Connection issues**: Verify the VantaDB server is installed and running (`vanta-cli server --mcp --db <path>`)
**Permission errors**: Ensure database path is writable
**Memory issues**: Configure appropriate memory limits

## Detailed Reference

For comprehensive documentation, see the reference files:

- **[references/mcp-protocol.md](references/mcp-protocol.md)** - Complete MCP protocol specification
- **[references/api-reference.md](references/api-reference.md)** - Full VantaDB API reference (Python and Rust)
- **[references/configuration.md](references/configuration.md)** - Advanced configuration guide (environment variables)

These files provide in-depth technical details for:
- MCP protocol methods and error handling
- Complete API methods and data structures
- HNSW parameter tuning
- Performance optimization
- Security configuration

For general documentation, see [docs/api/MCP.md](../../docs/api/MCP.md).