---
name: vantadb-mcp
description: VantaDB Model Context Protocol (MCP) server integration for persistent AI agent memory. Use when an AI agent (OpenCode, Claude, Cursor, etc.) needs to work with VantaDB as a memory backend through MCP for: (1) Storing and retrieving persistent memory records, (2) Performing hybrid vector and text search, (3) Managing namespace-scoped memory isolation, (4) Accessing operational metrics, schema information, and collection statistics, (5) Executing IQL graph queries and rehydrating archived nodes, (6) Working with AI frameworks like CrewAI, Mem0, AutoGen, Haystack, LangGraph, Semantic Kernel, or DSPy that require persistent memory storage.
---

# VantaDB MCP Integration

VantaDB provides a complete MCP (Model Context Protocol) server implementation for persistent memory storage with hybrid vector and text search capabilities. The MCP server exposes **79 tools** (49 core + 6 `skill_*` + 8 `code_*` + 6 `wiki_*` + 1 `context_assemble` + 3 `scene_*` + 6 `thread_*`), 2 resources, and 4 prompt templates over stdio JSON-RPC 2.0.

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

## Available MCP Tools (79)

The full contract for all **79 tools** lives in
[references/api-reference.md](references/api-reference.md) § "MCP Tools" — the single source of truth. The sections below document the 49 core tools in detail; the other 30 are summarized here.

| Group | Count | Tools | Precondition |
|-------|-------|-------|--------------|
| Core (Memory/Search/Collections/Graph/IQL/GDS/Recovery) | 49 | documented below | none beyond an open DB |
| Review-agent Skills (`skill_*`) | 6 | `skill_list`, `skill_view`, `skill_create`, `skill_update`, `skill_patch`, `skill_files_write` | `owner_agent` caller identity; writes need `expected_version` |
| Code Intelligence (`code_*`) | 8 | `code_search`, `code_explore`, `code_callers`, `code_callees`, `code_impact`, `code_node`, `code_status`, `code_files`* | graph nodes/edges ingested first; query-only |
| Wiki Knowledge (`wiki_*`) | 6 | `wiki_search`, `wiki_read`, `wiki_list`, `wiki_graph`, `wiki_ingest`, `wiki_ingest_status` | wiki lifecycle in `ready` state |
| Context Engine (`context_assemble`) | 1 | `context_assemble` | read-only; session recall needs prior memory capture into the session |
| Scenes API (`scene_*`) | 3 | `scene_read`, `scene_list`, `scene_query` | read-only; scenes need prior scene capture into the session store |
| Threads (`thread_*`) | 6 | `thread_create`, `thread_send`, `thread_get`, `thread_list`, `thread_delete`, `thread_purge_expired` | conversation history CRUD; ids are u128 strings; delete is permanent |

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

**memory_versions** (MOD-10) - List every retained version of a memory record, ascending (v1..vN)
- Parameters: `namespace`, `key` (both required)
- Returns: JSON array of memory records; empty `[]` if the key does not exist or has no history. Expired versions are included as historical data until purged. Version snapshots deliberately drop the supersession fields (`superseded_by`/`superseded_at_ms` are always `null`) — read the live record via `memory_get` to see supersession.

**memory_supersede** (MOD-10) - Mark an existing record as superseded by another existing record (durable soft-dead, recoverable)
- Parameters: `namespace`, `old_key`, `new_key` (all required)
- Returns: `{"superseded": true}`
- Errors (self-correctable error content) if either key is missing, `old_key == new_key`, or the old record is already superseded (idempotency guard). Superseded records can be hidden from search/list with `exclude_superseded`.

### Search Operations

**search_memory** - Hybrid vector and text search in a namespace
- Parameters: `namespace` (required), `query_vector` (optional array), `text_query` (optional string), `top_k` (default: 10), `distance_metric` (`cosine` | `euclidean`, default: `cosine`, **per-request** — has an observable effect on ranking and scores; no server-side global setting), `explain` (optional boolean), `filters` (optional object)
- `filters` accepts flat values `{"field": value}` **or** explicit equality `{"field": {"$eq": value}}` (both fold to the same equality semantics). Range operators (`$gt`/`$gte`/`$lt`/`$lte`/`$neq`) are NOT supported in `search_memory` — the search request has no operator slot; pass them to `memory_list` instead (returns a clear error pointing there).
- Returns: A **JSON array** of search hits. Each hit is an object with `record` (the memory record), `score` (fused relevance score), and — only when `explain: true` — an `explanation` object with the per-hit scoring breakdown: `identity` (`"namespace\0key"`), `score`, `snippet`, `matched_tokens`, `matched_phrases`, `bm25_terms`, `rrf_text_rank`, `rrf_vector_rank`.
- ⚠️ Explain shape (T15): the response is a **flat hit array**; there is **no top-level `route` or `fusion_report`** key on `search_memory`. Those fields belong to the dedicated core/Python `explain_memory_search()` method (returns `{route, hits, fusion_report}`; `fusion_report` is currently always `null`). Do not assert `route`/`fusion_report` on `search_memory` output.

**search_semantic** - Raw HNSW vector search
- Parameters: `vector` (F32 query vector, required), `k` (required in the input schema; optional at runtime — when omitted it defaults to 5; **clamped to `config.max_top_k`**, the same cap `search_memory` uses)
- Returns: Nearest neighbors with distances. `distance` is a **real distance, not a similarity**: lower is more similar, and hits are returned in ascending distance order. Under the cosine metric `distance = 1 − cosine_similarity` — an identical vector reports `0.0`, an orthogonal vector reports `1.0`.

**search_with_method** (MCP-24) - Hybrid memory search with an explicit dense-index backend override
- Parameters: same as `search_memory` plus `method` (optional string enum: `hnsw` | `ivf` | `flat` | `diskann` | `scann`). Omit `method` to keep automatic engine routing.
- Returns: Same flat hit array as `search_memory` (`{record, score, explanation?}`), but the dense-vector channel is forced through the selected backend. Use it to compare backends or pin a specific index.

**search_multi** (MCP-24) - Run one search request across multiple namespaces and merge the results
- Parameters: `namespaces` (required array of strings, must not be empty), plus the same request fields as `search_memory` (`query_vector`, `text_query`, `top_k`, `distance_metric`, `explain`, `filters`, `search_profile`). `top_k` caps the **merged** result globally.
- Returns: A flat hit array merged across all namespaces, sorted by descending score. Namespaces that fail validation are skipped; engine errors are surfaced.

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

**remove_edge** (MOD-10) - Remove all edges between two nodes with the given label (both directions)
- Parameters: `source_id`, `target_id` (u128 decimal strings, required), `label` (required)
- Returns: `{"removed": true}`
- Node ids are u128 decimal strings (JSON numbers lose precision above 2^53). Edges created by `RELATE` in IQL or `add_edge` are removed symmetrically; a missing node comes back as self-correctable error content.

**inject_context** - Inject context into a thread
- Parameters: `content`, `thread_id`
- Returns: Context anchoring status

**read_axioms** - Read system axioms
- Parameters: None
- Returns: Active Devil's Advocate Axioms (Iron Axioms)

**write_axiom** (MCP-33) - Register or update an agent axiom (an invariant rule)
- Parameters: `name` (unique key in the `_axioms` namespace, required), `description` (required)
- Returns: `{"id", "name", "description"}` — `id` is auto-assigned above the Iron Axioms (> 4)
- Upsert by name. Axioms live in the reserved `_axioms` namespace; the built-in Iron Axioms are read-only and never affected.

**delete_axiom** (MCP-33) - Remove an agent axiom by name
- Parameters: `name`
- Returns: `{"deleted": <bool>}` — `false` if the axiom does not exist
- The built-in Iron Axioms cannot be deleted.

**rehydrate** - Recover shadow-archived nodes that belonged to a summary node from TombstoneStorage
- Parameters: `summary_id` (u128 as string)
- Returns: `{"recovered_count", "summary_id", "rehydration_complete"}`

### Collection Operations

**collection_stats** - Returns statistics for a namespace/collection
- Parameters: `namespace`
- Returns: `{"total_records", "total_bytes", "has_vector_index", "vector_count", "created_at"}`
- Note (MOD-11 H6): `total_bytes` is a deliberate **estimate** — payload UTF-8 length plus a Debug-format length of each metadata value. It excludes vectors, sparse vectors, index overhead and serialization framing, so treat it as an order-of-magnitude figure, not the on-disk footprint.

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

**vacuum** (MOD-10) - Purge tombstoned nodes from the HNSW index
- Parameters: None
- Returns: `{scanned_nodes, removed_nodes, reclaimed_bytes, duration_ms, success}` (a `VacuumReport`-shaped object)

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

**snapshot_create** (MCP-34a) - Creates a physical filesystem snapshot under `<data_dir>/snapshots/<name>` (instant O(1) hard-link image on Unix, copy fallback on Windows)
- Parameters: `name` (required — a plain identifier, no path separators)
- Returns: `{path: "<snap_dir>", created_at: "<Instant debug>"}` — built manually because `FsSnapshot` does not derive `Serialize`
- Trust boundary: the name becomes a subdirectory, so path traversal (`/`, `\`, `.`, `..`) is rejected
- Note: `snapshot_restore` is a core feature and NOT yet implemented — this tool is create-only. Logical backup/restore lives in `export`/`import`.

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
| **JSON-RPC error** | `"error": {"code": <int>, "message": "<str>", "data": {…}?}` at top level (no `result`) | Malformed requests, invalid params, unknown methods/tools, domain `VantaError` from resource reads | `rehydrate` with a non-numeric `summary_id` → `-32602 Invalid params` |
| **isError content** | `result.isError: true` + `content[0].text` | Tool ran but hit a domain error | `get_node_neighbors` with a non-existent node → `"Node not found"`; `memory_get` missing key → `"Record not found"` |

Common JSON-RPC codes: `-32600` invalid request, `-32601` method not found, `-32602` invalid params, `-32603` internal error. Domain `VantaError` failures (ERR-MCP-01) additionally surface on the isError channel as a JSON string in `content[0].text` — parse it: `{"code": -320xx, "message": "...", "data": {"code": "VANTADB_*", "retriable": bool, "hint": "..."}}`. Retry decisions should branch on `data.retriable`. The `-320xx` numeric codes and mapping table live in `docs/api/MCP.md`. Hardcoded not-found replies (`"Record not found"`, `"Node not found"`) remain plain strings.

## Available MCP Resources

The server lists 2 static resources in `resources/list`; 2 additional dynamic URIs are servable via `resources/read` (not listed):

- **metrics://** - Operational metrics (memory usage, HNSW statistics, storage information) — listed
- **schema://** - Database schema information (HNSW configuration, text index version) — listed
- **memory://{namespace}/{key}** - Individual memory records by URI — servable, not listed
- **namespace://{namespace}** - Namespace content listing — servable, not listed. Returns the **first page** (`default_list_limit`, 100 records) with a `next_cursor`. The URI has no cursor parameter, so paginate large namespaces via the `memory_list` tool (which accepts `cursor` and clamps at `max_list_limit`).

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
- **F9 — `read_axioms`:** returns the 4 hardcoded Iron Axioms (ids 1-4, each `{id, name, description}`) plus any agent axioms stored in the reserved `_axioms` namespace (ids > 4), merged and sorted by id. The Iron Axioms are always present — `write_axiom`/`delete_axiom` never modify them.
- **F12 — `write_axiom`/`delete_axiom`:** agent axioms are records in the reserved `_axioms` namespace (`key` = name, `payload` = JSON `{id, name, description}`). Writing an axiom adds it to `read_axioms` output without touching the Iron Axioms; deleting one removes only the agent axiom.
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

### Threat Model — LLM06 (Excessive Agency)

The MCP server is an LLM-facing surface: every tool an agent can call is a
capability the agent may exercise, so prompt injection is the primary threat
(OWASP LLM Top 10 **LLM06**). The trust boundary is the **host** running the
server — stdio, local, single-user. Threat model for the host-file tools:

- **`bulk_import_file(path)`** and **`wiki_ingest(root)`** accept **arbitrary
  host filesystem paths**. A prompt-injected agent can make the server read
  local files (any `.vdbdump`, any markdown tree) and ingest them into the
  database, where the agent can read them back via `wiki_read` / `memory_list`.
- This is an **accepted risk for the local single-user stdio deployment** —
  the operator who started the server already has full access to the same
  filesystem. **Not safe to expose** to a multi-tenant / remote client without
  an allowlist or root-cap on `path`/`root` and per-tenant isolation.
- Destructive/irreversible tools (`collection_delete`, `memory_delete_by_filter`,
  `purge_expired`, `compact_wal`, `compact_layout`, `vacuum`, `rebuild_index`,
  `thread_delete`, `delete_axiom`) are **ungated**: any agent that can reach
  the server can call them. Use namespace isolation and editor-level access
  control to bound what a compromised agent can destroy.
- **`k` in `search_semantic` is clamped to `config.max_top_k`** (same cap as
  `search_memory`) so a crafted query cannot materialize the whole HNSW graph
  into memory (bounded consumption, LLM10).
- **Timeout caveat (H5):** `request_timeout` drops the *client* response on
  expiry, but tokio cannot cancel the in-flight `spawn_blocking` engine work —
  it keeps running and holds its concurrency permit until done. Hung
  operations can saturate the pool; accepted for the local server.

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