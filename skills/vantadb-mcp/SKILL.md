---
name: vantadb-mcp
description: VantaDB Model Context Protocol (MCP) server integration for persistent AI agent memory. Use when an AI agent (OpenCode, Claude, Cursor, etc.) needs to work with VantaDB as a memory backend through MCP for: (1) Storing and retrieving persistent memory records, (2) Performing hybrid vector and text search, (3) Managing namespace-scoped memory isolation, (4) Accessing operational metrics, schema information, and collection statistics, (5) Executing IQL graph queries and rehydrating archived nodes, (6) Working with AI frameworks like CrewAI, Mem0, AutoGen, Haystack, LangGraph, Semantic Kernel, or DSPy that require persistent memory storage.
---

# VantaDB MCP Integration

VantaDB provides a complete MCP (Model Context Protocol) server implementation for persistent memory storage with hybrid vector and text search capabilities. The MCP server exposes **57 tools** (36 core + 6 `skill_*` + 8 `code_*` + 6 `wiki_*` + 1 `context_assemble`), 2 resources, and 4 prompt templates over stdio JSON-RPC 2.0.

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

## Available MCP Tools (57)

The full contract for all **57 tools** lives in
[references/api-reference.md](references/api-reference.md) § "MCP Tools" — the single source of truth. The sections below document the 36 core tools in detail; the other 21 are summarized here.

| Group | Count | Tools | Precondition |
|-------|-------|-------|--------------|
| Core (Memory/Search/Collections/Graph/IQL/GDS/Recovery) | 36 | documented below | none beyond an open DB |
| Review-agent Skills (`skill_*`) | 6 | `skill_list`, `skill_view`, `skill_create`, `skill_update`, `skill_patch`, `skill_files_write` | `owner_agent` caller identity; writes need `expected_version` |
| Code Intelligence (`code_*`) | 8 | `code_search`, `code_explore`, `code_callers`, `code_callees`, `code_impact`, `code_node`, `code_status`, `code_files`* | graph nodes/edges ingested first; query-only |
| Wiki Knowledge (`wiki_*`) | 6 | `wiki_search`, `wiki_read`, `wiki_list`, `wiki_graph`, `wiki_ingest`, `wiki_ingest_status` | wiki lifecycle in `ready` state |
| Context Engine (`context_assemble`) | 1 | `context_assemble` | read-only; session recall needs prior memory capture into the session |

\* `code_files` is a documented not-supported stub: the built-in GraphRAG has no file-per-node concept.

### Memory CRUD Operations

**memory_put** - Insert or update a memory record
- Parameters: `namespace`, `key`, `payload` (required); `vector` (optional array of numbers), `sparse_vector` (optional object mapping dimension id → weight, e.g. `{"0": 0.5}`), `metadata` (optional object), `expires_at_ms` (optional absolute Unix-ms timestamp — the record expires at that time)
- Returns: The created/updated memory record

**memory_put_batch** (MCP-19) - Store multiple memory records in one batch call
- Parameters: `inputs` (required array; each entry: `namespace`, `key`, `payload` required + same optional fields as `memory_put`)
- All-or-nothing: an invalid input fails the whole call before any write. Duplicate keys are UPSERTs (version bumps). Vector dims must match the live index.
- Returns: JSON array of the created/updated records

**memory_get** - Retrieve a memory record
- Parameters: `namespace`, `key`
- Returns: Memory record or error if not found

**memory_delete** - Delete a memory record
- Parameters: `namespace`, `key`
- Returns: Success status (`{"deleted": true|false}`)

**memory_delete_by_filter** (MCP-18) - Batch-delete every record in a namespace whose metadata matches filters
- Parameters: `namespace`, `filters` (required object; same shape as `memory_list` filters — flat values or `$eq`/`$neq`/`$gt`/`$gte`/`$lt`/`$lte` operators, AND semantics)
- Guard rail: at least one filter item required (prevents accidental full-namespace deletion).
- Returns: `{"deleted_count": N}`

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

**graph_page_rank** (MCP-21) - PageRank over the subgraph reachable from the roots (GDS)
- Parameters: `roots` (required array of u128 decimal strings); `max_iterations` (default 100), `damping_factor` (default 0.85), `tolerance` (default 1e-6)
- Returns: `{"scores": {"<node_id>": rank}}` — ranks sum to ≈ 1.0; node ids are decimal strings
- Note: GDS is **not** reachable via IQL (`RELATE`/`FROM` only create/read edges) — these tools are the correct path.

**graph_degree_centrality** (MCP-21) - In/out degree counts for every node reachable from the roots
- Parameters: `roots` (required array of u128 decimal strings)
- Returns: `{"degrees": {"<node_id>": {"in": <count>, "out": <count>}}}`

**graph_traverse** (MCP-22) - Multi-hop BFS/DFS traversal from one or more start nodes
- Parameters: `start` (required array of u128 decimal strings), `mode` (`bfs` | `dfs`, required), `max_depth` (required); `direction` (`forward` | `reverse` | `both`, default forward); optional `filter` object `{labels: [<u32>], time_range: [from_ms, to_ms]}` restricts traversal to edges matching the label ids and/or the inclusive creation-time window
- Returns: `{"visited": ["<id>", ...], "count": N}` in traversal order

**graph_topological_sort** (MCP-22) - Topological order of the subgraph reachable from the roots
- Parameters: `roots` (required array of u128 decimal strings)
- Returns: `{"order": ["<id>", ...]}`; a cycle comes back as self-correctable error content

**graph_is_dag** (MCP-22) - Whether the subgraph reachable from the roots is a directed acyclic graph
- Parameters: `roots` (required array of u128 decimal strings)
- Returns: `{"is_dag": true|false}`
- Design note (MCP-22): graph accumulators (`graph_create_accumulator`/`_add`/`_get`/`_snapshot`) are intentionally NOT exposed — they are in-process parallelism primitives holding no engine state, so an MCP lifecycle tool would need server-side session state for zero agent value.

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

### Maintenance Operations

**purge_expired** - Scans all memory records and physically deletes those whose TTL expiry has passed
- Parameters: None
- Returns: `{"purged": <count>}`
- Note: pairs with the `expires_at_ms` TTL of `memory_put` — an agent that uses TTL should call this periodically to reclaim space.

**compact_wal** - Flushes, archives the current WAL file, and starts a fresh one to reclaim WAL space
- Parameters: None
- Returns: `{"compacted_wal": true}`

**flush** - Flushes the WAL and memory-mapped files to disk (manual durability checkpoint)
- Parameters: None
- Returns: `{"flushed": true}`

**compact_layout** - Compacts the vector store file grouping nodes in BFS order from the HNSW entry point
- Parameters: None
- Returns: `{"bytes_reclaimed": <count>}`

### Index Recovery / Introspection (MCP-20/MCP-26)

**rebuild_index** (MCP-20) - Rebuilds the HNSW vector index, derived indexes, and text index from scratch (recovery primitive for a corrupted index)
- Parameters: None
- Returns: `{scanned_nodes, indexed_vectors, skipped_tombstones, duration_ms, derived_rebuild_ms, index_path, success}`

**audit_text_index** (MCP-20) - Read-only integrity audit of the derived persistent text index (BM25 postings/stats vs canonical memory records)
- Parameters: `namespace` (optional filter; omit to audit all), `deep` (optional boolean — value-level verification of posting positions, term frequencies and stats)
- Returns: audit report; `passed=true` + `status="ok"` mean no drift, `status="repair_recommended"` means run `repair_text_index`

**repair_text_index** (MCP-20) - Repairs the derived text index by rebuilding it from canonical storage
- Parameters: None
- Returns: `{record_count, posting_entries, doc_stats_entries, term_stats_entries, namespace_stats_entries, duration_ms, success}`

**capabilities** (MCP-26) - Introspects the engine's supported features so the agent can discover what the connected database supports
- Parameters: None
- Returns: `{runtime_profile, persistence, vector_search, iql_queries, read_only}`

**generate_snippet** (MCP-26) - Generates a text snippet from a payload around matched query terms
- Parameters: `payload`, `text_query` (required); `with_highlighting` (optional boolean, default false — wraps matched terms in `<strong>` markers)
- Returns: `{snippet: "..."}`, or `{snippet: null}` when the query yields no terms (e.g. empty/whitespace query)

**list_snapshots** (MCP-26) - Lists existing physical snapshot names under `<data_dir>/snapshots` (sorted)
- Parameters: None
- Returns: `{snapshots: ["<name>", ...]}`
- Note: logical backup/restore lives in `export`/`import`; snapshots are physical Fjall copies

### Backup / Restore (MCP-17)

**export** - Exports memory records as JSONL (one JSON object per line, schema_version 1)
- Parameters: `namespace` (optional — omit to export ALL namespaces)
- Returns: the raw JSONL as text content (`content[0].text` is NOT wrapped in JSON)
- Note: capped at 10 MB per call; larger datasets use the CLI/SDK file export (`export_namespace(path, ...)`). Pair with `import` for backup/restore.

**import** - Imports records from a JSONL string as produced by `export`
- Parameters: `content` (JSONL string, max 10 MB per call)
- Returns: `VantaImportReport` `{inserted, updated, skipped, errors, duration_ms}` — empty lines are skipped and malformed lines count as `errors` instead of failing the call

### Bulk Import (MCP-25)

**bulk_import_file** - Bulk-imports from a binary `.vdbdump` file on the host filesystem
- Parameters: `path` (host path)
- Returns: `BulkImportReport` `{total_records, batches_committed, duration_ms}`
- Note: bypasses per-record validation for raw throughput; missing file returns clear error text

**bulk_import_stream** - Bulk-imports from inline content
- Parameters: `content` (NDJSON — one `VantaMemoryInput` per line — or raw `.vdbdump` payload starting with the `VDBJSON` magic; max 10 MB per call)
- Returns: same `BulkImportReport`
- Note: writes the reserved `__vanta_*` record fields (MCP-28), so imported records ARE addressable via `memory_get`/`memory_list`/`memory_delete` like `put()` records

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

For per-IDE setup (Cursor, Claude Code, Windsurf, OpenCode, Cline, VS Code), see [docs/api/MCP.md](../../docs/api/MCP.md) (a stub). The source of truth for the MCP contract is this skill — [references/api-reference.md](references/api-reference.md) § "MCP Tools (45)".

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