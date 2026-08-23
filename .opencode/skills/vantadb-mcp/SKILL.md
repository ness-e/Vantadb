---
name: vantadb-mcp
description: VantaDB Model Context Protocol (MCP) server integration for persistent AI agent memory. Use when an AI agent (OpenCode, Claude, Cursor, etc.) needs to work with VantaDB as a memory backend through MCP for: (1) Storing and retrieving persistent memory records, (2) Performing hybrid vector and text search, (3) Managing namespace-scoped memory isolation, (4) Accessing operational metrics, schema information, and collection statistics, (5) Executing IQL graph queries and rehydrating archived nodes, (6) Working with AI frameworks like CrewAI, Mem0, AutoGen, Haystack, LangGraph, Semantic Kernel, or DSPy that require persistent memory storage.
---

# VantaDB MCP Integration

VantaDB provides a complete MCP (Model Context Protocol) server implementation for persistent memory storage with hybrid vector and text search capabilities. The MCP server exposes **33 tools** (15 core + 6 `skill_*` + 8 `code_*` + 4 `wiki_*`), 2 resources, and 4 prompt templates over stdio JSON-RPC 2.0.

## Quick Start

### Installation

Run the setup script to install VantaDB:

```bash
bash scripts/setup-vantadb.sh
```

This installs VantaDB. Configuration is handled entirely through environment
variables (`VANTADB_*` / `VANTA_*`) and CLI flags — **there is no config
file** (see [references/configuration.md](references/configuration.md)).

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
- `assets/opencode-config.json` - OpenCode configuration

### OpenCode

The user's primary editor is OpenCode. Add to your `opencode.json` (project root or `~/.config/opencode/opencode.json`). Use an **absolute path** for `--db` — OpenCode does not expand `~` when spawning the command directly:

```json
{
  "mcp": {
    "vantadb": {
      "type": "local",
      "command": ["vanta-cli", "server", "--mcp", "--db", "C:/Users/<you>/.vantadb"],
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

The script spawns one server process and drives the MCP handshake
(`initialize` → `tools/list` → `resources/list` → `prompts/list`), exiting 0 if
every request returns a valid result. The server binary is resolved from (in
order): `argv[1]` or the `VANTADB_MCP_BIN` environment variable (explicit
path), `vanta-cli` on PATH, `target/debug|release/vanta-cli.exe`, then
`vantadb-server` on PATH / `target/debug|release/vantadb-server.exe`.

```bash
# Explicit binary (recommended — no PATH dependency):
python scripts/test-mcp.py C:/Users/me/.cargo/bin/vanta-cli.exe
# Or via env var:
VANTADB_MCP_BIN=C:/Users/me/.cargo/bin/vanta-cli.exe python scripts/test-mcp.py
```

### Namespace Management

Namespaces are created **implicitly** — there is no dedicated namespace
creation script or tool. `memory_put` with a new `namespace` value creates it on
first write; list what exists with `collection_list` (or `memory_list_namespaces`):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "collection_list",
    "arguments": {}
  }
}
```

## Available MCP Tools (33)

The full contract for all **33 tools** lives in
[references/api-reference.md](references/api-reference.md) § "MCP Tools" — the single source of truth. The sections below document the 15 core tools in detail; the other 18 are summarized here.

| Group | Count | Tools | Precondition |
|-------|-------|-------|--------------|
| Core (Memory/Search/Collections/Graph/IQL) | 15 | documented below | none beyond an open DB |
| Review-agent Skills (`skill_*`) | 6 | `skill_list`, `skill_view`, `skill_create`, `skill_update`, `skill_patch`, `skill_files_write` | `owner_agent` caller identity; writes need `expected_version` |
| Code Intelligence (`code_*`) | 8 | `code_search`, `code_explore`, `code_callers`, `code_callees`, `code_impact`, `code_node`, `code_status`, `code_files`* | graph nodes/edges ingested first; query-only |
| Wiki Knowledge (`wiki_*`) | 4 | `wiki_search`, `wiki_read`, `wiki_list`, `wiki_graph` | wiki lifecycle in `ready` state |

\* `code_files` is a documented not-supported stub: the built-in GraphRAG has no file-per-node concept.

### Memory CRUD Operations

**memory_put** - Insert or update a memory record
- Parameters: `namespace`, `key`, `payload` (required); `vector` (optional array of numbers), `sparse_vector` (optional object mapping dimension id → weight, e.g. `{"0": 0.5}`), `metadata` (optional object), `expires_at_ms` (optional absolute Unix-ms timestamp — the record expires at that time)
- Returns: The created/updated memory record

**memory_get** - Retrieve a memory record
- Parameters: `namespace`, `key`
- Returns: Memory record or error if not found

**memory_delete** - Delete a memory record
- Parameters: `namespace`, `key`
- Returns: Success status (`{"deleted": true|false}`)

**memory_list** - List memory records with pagination
- Parameters: `namespace`, `limit` (default: 100), `cursor` (optional number), `filters` (optional object)
- `filters` accepts BOTH formats (AUD-048 — unified semantics with the CLI channel): flat values `{"field": value}` (implicit `$eq`) **or** operator objects `{"field": {"$gt": value}}` (`$eq`, `$neq`, `$gt`, `$gte`, `$lt`, `$lte`). Operators route through the core's `filter_ops` slot.
- Returns: `{"records": [...], "next_cursor": ...}`

**memory_list_namespaces** - List all namespaces
- Parameters: None
- Returns: List of namespace names

### Search Operations

**search_memory** - Hybrid vector and text search in a namespace
- Parameters: `namespace` (required), `query_vector` (optional array), `text_query` (optional string), `top_k` (default: 10), `distance_metric` (`cosine` | `euclidean`, default: `cosine`, **per-request** — has an observable effect on ranking and scores; no server-side global setting), `explain` (optional boolean), `filters` (optional object)
- `filters` accepts flat values `{"field": value}` **or** explicit equality `{"field": {"$eq": value}}` (both fold to the same equality semantics). Range operators (`$gt`/`$gte`/`$lt`/`$lte`/`$neq`) are NOT supported in `search_memory` — the search request has no operator slot; pass them to `memory_list` instead (returns a clear error pointing there).
- Returns: A **JSON array** of search hits. Each hit is an object with `record` (the memory record), `score` (fused relevance score), and — only when `explain: true` — an `explanation` object with the per-hit scoring breakdown: `identity` (`"namespace\0key"`), `score`, `snippet`, `matched_tokens`, `matched_phrases`, `bm25_terms`, `rrf_text_rank`, `rrf_vector_rank`.
- ⚠️ Explain shape (T15): the response is a **flat hit array**; there is **no top-level `route` or `fusion_report`** key on `search_memory`. Those fields belong to the dedicated core/Python `explain_memory_search()` method (returns `{route, hits, fusion_report}`; `fusion_report` is currently always `null`). Do not assert `route`/`fusion_report` on `search_memory` output.

**search_semantic** - Raw HNSW vector search
- Parameters: `vector` (F32 query vector, required), `k` (required in the input schema; optional at runtime — when omitted it defaults to 5)
- Returns: Nearest neighbors with distances. `distance` is a **real distance, not a similarity**: lower is more similar, and hits are returned in ascending distance order. Under the cosine metric `distance = 1 − cosine_similarity` — an identical vector reports `0.0`, an orthogonal vector reports `1.0`.

### Graph Operations

**query_iql** - Execute an IQL (Interactive Query Language) statement. Allows reading structures and inserting/mutating Nodes providing semantic context. **LISP is not supported; statements must be IQL.**
- Parameters: `query`
- Returns: Query results or execution status (read nodes, write result, or stale-context rehydration hint)
- **Scope (MCP-27):** IQL operates over **typed graph nodes only** (`TYPE`). Memory records written via `memory_put` are NOT exposed as IQL tables — `SELECT * FROM <namespace>` returns `[]` without error because those records live as internal nodes with reserved `__vanta_*` fields and no `type` field. Use `memory_list` / `memory_get` / `search_memory` for memory records. To make data queryable via IQL from the agent channel, insert graph nodes directly: `INSERT NODE#<id> TYPE <Type> { ... }` then `SELECT * FROM <Type>`.

#### IQL Syntax

IQL is **not** Cypher and **not** LISP. The following statements are supported
(verified against `vantadb-mcp/tests/mcp_tests.rs` and the parser test suite):

| Statement | Syntax | Example |
|-----------|--------|---------|
| Insert node | `INSERT NODE#<id> TYPE <Type> { <field>: "<value>" }` (optional `VECTOR [<f32>, ...]`) | `INSERT NODE#999 TYPE TestNode { tier: "Cold" }` |
| Insert node with vector | `INSERT NODE#<id> TYPE <Type> { ... } VECTOR [<f32>, ...]` | `INSERT NODE#42 TYPE Item { name: "test", price: 99 } VECTOR [0.1, -0.4, 0.9]` |
| Read node | `FROM NODE#<id>` | `FROM NODE#999` |
| Update node | `UPDATE NODE#<id> SET <field> = "<value>"` (uses `SET` + `=`, **not** `{ }`) | `UPDATE NODE#101 SET nombre = "Eros Dev", activo = true` |
| Delete node | `DELETE NODE#<id>` | `DELETE NODE#5` |
| Relate nodes (edge) | `RELATE NODE#<a>--"<label>"-->NODE#<b>` (optional `WEIGHT <f64>`) | `RELATE NODE#1 --"amigo"--> NODE#2 WEIGHT 0.95` |
| Insert message into thread | `INSERT MESSAGE SYSTEM|USER|ASSISTANT "<text>" TO THREAD#<id>` | `INSERT MESSAGE SYSTEM "hello world" TO THREAD#200` |

Notes:
- **LISP is not supported** — `query_iql` rejects LISP-style expressions.
- `UPDATE` uses `SET <field> = <value>`, not the `{ field: value }` object syntax of `INSERT`.
- `LINK` is not a valid clause; see [Behavior Notes](#behavior-notes).
- See the IQL parser (`src/parser/mod.rs`) and `vantadb-mcp/tests/mcp_tests.rs` for the full grammar.

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

## Response Envelope

Every `tools/call` response wraps its payload in the MCP content envelope — the
"Returns:" descriptions above show the JSON **inside** `content[0].text`, not the
top-level response. The wire shape is:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      { "type": "text", "text": "<json payload as a string>" }
    ]
  }
}
```

- `content[0].text` is a JSON **string**; parse it to get the actual payload (records, hits, stats).
- On tool-level failure, the result also carries `"isError": true`:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "isError": true,
    "content": [
      { "type": "text", "text": "Record not found" }
    ]
  }
}
```

Extraction pattern (Python):

```python
res = r["result"]
if res.get("isError"):
    raise RuntimeError(res["content"][0]["text"])
payload = json.loads(res["content"][0]["text"])
```

## Error Channels

Two distinct error channels exist — know which one you are handling:

| Channel | Shape | When it happens | Example |
|---------|-------|-----------------|---------|
| **JSON-RPC error** | `"error": {"code": <int>, "message": "<str>"}` at top level (no `result`) | Malformed requests, invalid params, unknown methods/tools | `rehydrate` with a non-numeric `summary_id` → `-32602 Invalid params` |
| **isError content** | `result.isError: true` + `content[0].text` | Tool ran but hit a domain error | `get_node_neighbors` with a non-existent node → `"Node not found"`; `memory_get` missing key → `"Record not found"` |

Common JSON-RPC codes: `-32600` invalid request, `-32601` method not found, `-32602` invalid params, `-32603` internal error.

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

## Behavior Notes

Verified edge-case behaviors (2026-08-17 battery; see the
Rust test suite for sources). These are **documented behavior**, not bugs:

- **F4 — `memory_get` not-found:** returns `isError` content `"Record not found"` (not a JSON-RPC error).
- **F5 — `memory_list` cursor:** the cursor is a numeric **offset**. Use the `next_cursor` from a page as the `cursor` of the next call to continue pagination; it is not an opaque token.
- **F7 — IQL parser edges:**
  - `LINK` **does not exist** — a query like `INSERT NODE#102 TYPE X { ... } LINK NODE#101` succeeds silently: the node is inserted but **no edge is created** (trailing garbage is accepted and dropped by the parser).
  - Trailing garbage on any statement is silently ignored.
  - `FROM NODE#a,b` (multi-id) returns only the **first** id.
  - `FROM` a deleted node returns `[]` (empty list, no error).
- **F8 — `get_node_neighbors`:** reports only **outgoing** edges whose target node is alive; dangling edges to missing/tombstoned targets are omitted.
- **F9 — `read_axioms`:** returns 4 objects, each `{id, name, description}` (hardcoded Iron Axioms).
- **F10 — `rehydrate`:** only recovers nodes that were shadow-archived by a prior summary consolidation. A single-pass session that never created a summary returns `recovered_count: 0` — the archived-node condition is not reachable with MCP tools alone.
- **F11 — `memory_put` response fields:** the returned record includes extra fields beyond `namespace`/`key`/`payload`/`metadata` — `version`, `node_id`, `created_at_ms`, `updated_at_ms`, `expires_at_ms` (see `VantaMemoryRecord` in `references/api-reference.md`).

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

Text and hybrid search are functional out of the box: the MCP server reconciles
the text index at startup (`ensure_indexes_current`), so no manual index build
is required on a fresh database.

Minimal example — hybrid query (vector + text) against a namespace:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "search_memory",
    "arguments": {
      "namespace": "agent/session-1",
      "text_query": "concise technical answers",
      "query_vector": [0.8, 0.1, 0.5],
      "top_k": 5
    }
  }
}
```

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

For per-IDE setup (Cursor, Claude Code, Windsurf, OpenCode, Cline, VS Code), see [docs/api/MCP.md](../../docs/api/MCP.md) (a stub). The source of truth for the MCP contract is this skill — [references/api-reference.md](references/api-reference.md) § "MCP Tools (33)".

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
- HNSW parameters (`m`, `ef_construction`, `ef_search`, …) are **not** exposed as environment variables — they are set programmatically via `HnswConfig` when constructing the engine (see [references/api-reference.md](references/api-reference.md))
- Implement periodic cleanup of old memories

## Security

- Use namespace isolation for different contexts
- Read-only mode is **not** an environment variable — set it programmatically via `VantaConfig::with_read_only(true)` in the embedded SDK (see [references/configuration.md](references/configuration.md))
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

For general documentation, see [docs/api/MCP.md](../../docs/api/MCP.md) — a stub pointing back to this skill as the source of truth.