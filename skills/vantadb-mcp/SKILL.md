---
name: vantadb-mcp
description: VantaDB Model Context Protocol (MCP) server integration for persistent AI memory. Use when Claude needs to work with VantaDB as a memory backend through MCP for: (1) Storing and retrieving persistent memory records, (2) Performing hybrid vector and text search, (3) Managing namespace-scoped memory isolation, (4) Accessing operational metrics and schema information, (5) Working with AI frameworks like CrewAI, Mem0, AutoGen, Haystack, LangGraph, Semantic Kernel, or DSPy that require persistent memory storage.
---

# VantaDB MCP Integration

VantaDB provides a complete MCP (Model Context Protocol) server implementation for persistent memory storage with hybrid vector and text search capabilities.

## Quick Start

### Installation

Run the setup script to install VantaDB:

```bash
bash scripts/setup-vantadb.sh
```

This installs VantaDB and creates default configuration.

### Starting the MCP Server

The VantaDB MCP server runs as a stdio JSON-RPC server:

```bash
vanta-server --mcp --path ~/.vantadb
```

### MCP Client Configuration

Configure your MCP client to connect to VantaDB:

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-server",
      "args": ["--mcp", "--path", "~/.vantadb"]
    }
  }
}
```

**Pre-configured templates available in assets/:**
- `assets/claude-desktop-config.json` - Claude Desktop configuration
- `assets/cursor-config.json` - Cursor workspace configuration
- `assets/config-template.json` - VantaDB configuration template

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

## Available MCP Tools

### Memory CRUD Operations

**memory_put** - Insert or update a memory record
- Parameters: `namespace`, `key`, `payload`, `vector` (optional), `metadata` (optional)
- Returns: The created/updated memory record

**memory_get** - Retrieve a memory record
- Parameters: `namespace`, `key`
- Returns: Memory record or error if not found

**memory_delete** - Delete a memory record
- Parameters: `namespace`, `key`
- Returns: Success status

**memory_list** - List memory records with pagination
- Parameters: `namespace`, `limit` (default: 100), `cursor` (optional), `filters` (optional)
- Returns: Page of records with next cursor

**memory_list_namespaces** - List all namespaces
- Parameters: None
- Returns: List of namespace names

### Search Operations

**search_memory** - Hybrid vector and text search
- Parameters: `namespace`, `query_vector` (optional), `text_query` (optional), `top_k`, `distance_metric`, `explain`, `filters`
- Returns: Search hits with scores and optional explanations

**search_semantic** - Raw HNSW vector search
- Parameters: `vector`, `k`
- Returns: Nearest neighbors with distances

### Graph Operations

**query_lisp** - Execute VantaLISP code
- Parameters: `query`
- Returns: Query results or execution status

**get_node_neighbors** - Inspect node relationships
- Parameters: `node_id`
- Returns: Node and its neighbors

**inject_context** - Inject context into a thread
- Parameters: `content`, `thread_id`
- Returns: Context anchoring status

**read_axioms** - Read system axioms
- Parameters: None
- Returns: Active Devil's Advocate Axioms

## Available MCP Resources

- **metrics://** - Operational metrics (memory usage, HNSW statistics, storage information)
- **schema://** - Database schema information (HNSW configuration, text index version)
- **memory://{namespace}/{key}** - Individual memory records by URI
- **namespace://{namespace}** - Namespace content listing

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

For editor-specific configuration, see [docs/EDITOR_INTEGRATIONS.md](../../docs/EDITOR_INTEGRATIONS.md).

Supported editors:
- Cursor
- VS Code (with MCP-compatible extension)
- OpenCode
- OpenClaw
- Devin
- Antigravity

## Performance Optimization

- Configure memory limits in VantaConfig
- Use namespace isolation to limit scope
- Adjust HNSW parameters for memory efficiency
- Implement periodic cleanup of old memories

## Security

- Use namespace isolation for different contexts
- Consider read-only mode for production deployments
- Implement access control at the editor level
- Audit memory access logs

## Troubleshooting

**Connection issues**: Verify VantaDB server is installed and running
**Permission errors**: Ensure database path is writable
**Memory issues**: Configure appropriate memory limits

## Detailed Reference

For comprehensive documentation, see the reference files:

- **[references/mcp-protocol.md](references/mcp-protocol.md)** - Complete MCP protocol specification
- **[references/api-reference.md](references/api-reference.md)** - Full VantaDB API reference (Python and Rust)
- **[references/configuration.md](references/configuration.md)** - Advanced configuration guide

These files provide in-depth technical details for:
- MCP protocol methods and error handling
- Complete API methods and data structures
- HNSW parameter tuning
- Performance optimization
- Security configuration

For general documentation, see [docs/MCP.md](../../docs/MCP.md).
