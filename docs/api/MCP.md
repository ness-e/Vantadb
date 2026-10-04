---
title: VantaDB Model Context Protocol (MCP) Server
kind: reference
status: active
description: "MCP server reference — tools, profiles, and the current implementation version (tracked below)"
tags: [vantadb, api]
---

# VantaDB Model Context Protocol (MCP) Server

Current MCP implementation version: 0.8.0

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

All three clients use the same server command; only the config file differs. **Never use `~` in these JSON configs**: MCP clients spawn the server process directly, without a shell, so `~/.vantadb` arrives literally and the open fails (`Path::new` receives the verbatim string — verified `src/cli_handlers/server.rs`). Always use an **absolute path** for `--db` (shown as `C:/Users/<you>/.vantadb`; on macOS e.g. `/Users/<you>/.vantadb`).

**Claude Desktop** — `%APPDATA%\Claude\claude_desktop_config.json` (Windows) / `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "C:/Users/<you>/.vantadb"]
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
      "args": ["server", "--mcp", "--db", "C:/Users/<you>/.vantadb"]
    }
  }
}
```

**Claude Code** — project-level `.mcp.json`, or one-shot via CLI:

```bash
claude mcp add vantadb -- vanta-cli server --mcp --db C:/Users/<you>/.vantadb
```

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "C:/Users/<you>/.vantadb"]
    }
  }
}
```

### First test

Verify the handshake (`initialize` → `tools/list`) works before touching your editor config:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | vanta-cli server --mcp --db ~/.vantadb
```

You should see a JSON response listing the available tools. Namespaces are created implicitly on first write (`memory_put` with a new namespace); list existing ones with `memory_list_namespaces`.

### Troubleshooting

| Symptom | Fix |
|---------|-----|
| Client shows no tools | Run the First test above; if it fails, check that `vanta-cli` is on PATH (`vanta-cli --version`). |
| `Server closed stdout` / immediate exit | The database path must be writable and not locked by another VantaDB process. |
| Tools error at call time | Check disk space and permissions on the `--db` directory; use `capabilities` to introspect the engine state. |
| Text search fails with `text_index not found` | Restarting the server reconciles indexes automatically at startup (`ensure_indexes_current`). |

For the full behavioral contract (error channels, response envelope, edge cases), see the VantaDB MCP Skill.

## Error Handling & JSON-RPC Codes

The VantaDB MCP server speaks [JSON-RPC 2.0](https://www.jsonrpc.org/specification)
over stdio. Every error response carries both a **JSON-RPC transport code**
(the `error.code` field) and a **Vanta canonical code** (in `error.data.code`)
from the 10-element contract in [`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md).

> **Canonical reference:** [`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md) §6
> — full code table, LLM retry guidance, and the upcoming `From<Error>
> for McpError` impl from `ERR-MCP-01`.

### JSON-RPC standard factories (5)

Defined in [`vantadb-mcp/src/error.rs`](../../vantadb-mcp/src/error.rs) as the
`McpError` type. These cover transport-level concerns (malformed request,
unknown method, invalid params, internal failure):

| JSON-RPC code | `McpError` factory | Meaning |
|---------------|--------------------|---------|
| `-32700` | `parse_error` | Invalid JSON received by the server |
| `-32600` | `invalid_request` | JSON sent is not a valid Request object |
| `-32601` | `method_not_found` | Method does not exist or is unavailable |
| `-32602` | `invalid_params` | Invalid method parameter(s) |
| `-32603` | `internal_error` | Internal JSON-RPC error |

### Vanta custom `-320xx` codes

Implemented in `ERR-MCP-01` as `impl From<Error> for McpError`
(`vantadb-mcp/src/error.rs`, from the Rust core `vantadb::Error`). The mapping
is driven by the canonical `Error::code()` string — never by re-matching
variants — so the table below is a projection of the core §1.1 codes onto the
JSON-RPC range:

| JSON-RPC code | `Error::code()` source | Canonical `data.code` | LLM retry? |
|---------------|------------------------------|------------------------|:----------:|
| `-32001` | `VANTADB_BUSY` (`DatabaseBusy`, `NotInitialized`) | `VANTADB_BUSY` | ✅/❌ per `data.retriable` |
| `-32002` | `VANTADB_CORRUPT` (`WALVersionMismatch`, `IncompatibleFormat`, `SchemaError`, `SerializationError`, `RestoreError`, `BackupError`) | `VANTADB_CORRUPT` | ❌ |
| `-32003` | *reserved* — see note below | - | - |
| `-32004` | `VANTADB_NOT_FOUND` (`NodeNotFound`, `NotFound`) | `VANTADB_NOT_FOUND` | ❌ |
| `-32005` | (auth layer — writer-proxy Bearer rejection in `server.rs`) | - | ❌ |
| `-32006` | (rate limit layer — not yet emitted) | - | ⚠️ backoff |
| `-32007` | `VANTADB_RESOURCE_LIMIT` (`ResourceLimit`, overflow variants) | `VANTADB_RESOURCE_LIMIT` | ⚠️ backoff |
| `-32008` | `VANTADB_TIMEOUT` (`Timeout`) | `VANTADB_TIMEOUT` | ✅ |
| `-32009` | `VANTADB_VALIDATION_ERROR` **and** `VANTADB_INVALID_ARGUMENT` | `VANTADB_VALIDATION_ERROR` / `VANTADB_INVALID_ARGUMENT` | ❌ |
| `-32603` | unmapped codes (`VANTADB_IO_ERROR`, `VANTADB_WASM_ERROR`, `VANTADB_CLOSED`) | as emitted | ⚠️ retry once |

> **Note on `-32003` and `retriable`:** the core `code()` folds the conflict
> variants (`ExecutionConflict`, `NodeIdCollision`, `CycleDetected`) into
> `VANTADB_VALIDATION_ERROR`, so they surface as `-32009`; `-32003` is held
> in reserve for a future distinct conflict code. Because the mapping is
> code-driven, per-variant nuance is carried by `data.retriable` (e.g.
> `DatabaseBusy` retriable ✅, `NotInitialized` ❌) and `data.hint` — clients
> should branch on `data`, not infer retry from the numeric code alone.

On the tool-result (`isError`) channel the same envelope is serialized into
`content[0].text` as a JSON string (`{"code":…,"message":…,"data":{…}}`), so
domain failures expose the same fields as JSON-RPC errors. The legacy
`"Put Error: …"`-style prefixes were removed in `ERR-MCP-01`.

### Response envelope

Every JSON-RPC error response from VantaDB MCP carries the canonical code in
`data.code` (the prefixed `VANTADB_*` value returned by `Error::code()`;
`data.hint` is omitted when the error has no recovery hint), plus `retriable`
so LLM agents can branch on a stable identifier without parsing message text:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "validation failed on 'namespace': must not be empty",
    "data": {
      "code": "VANTADB_VALIDATION_ERROR",
      "retriable": false
    }
  }
}
```

### Retry guidance for LLM clients

| Situation | Vanta code | Recommended action |
|-----------|-------------|-------------------|
| Validation failed (bad input) | `VALIDATION_ERROR` | Fix the input, do not retry |
| Database busy / locked | `BUSY` | Wait briefly, retry |
| Operation timed out | `TIMEOUT` | Retry (or increase timeout) |
| Resource limit exceeded | `RESOURCE_LIMIT` | Retry with backoff, monitor memory |
| Corrupt / version mismatch | `CORRUPT` | Surface to user, do not retry |
| Not found | `NOT_FOUND` | Adjust query, do not retry |
| Internal JSON-RPC error | (none) | Retry once, then escalate |

## Tool Families

**80 listed tools in 8 families (86 defined − 6 WIRE-02 absorbed `code_*` projections; spec 2025-06-18, every tool carries `annotations` with `title`, `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint` per [MCP Tool Annotations](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) / [blog 2026-03-16](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations)):**

| Family | Count | Source module |
|--------|-------|---------------|
| Core | 48 | `handlers/tools.rs` — listed in `tools/list` |
| `code_*` | 2 listed (8 defined) | `code.rs` — 6 projections absorbed dispatch-only (WIRE-02) |
| `skill_*` | 7 | `skills.rs` |
| `wiki_*` | 6 | `wiki.rs` |
| `context_assemble` | 1 | `context.rs` |
| `scene_*` | 5 | `scenes.rs` |
| `thread_*` | 6 | `threads.rs` |
| `dream_*` | 5 | `dreams.rs` |

> Annotations are display hints (untrusted, not enforcement): `readOnlyHint` true = no persistent mutation, `destructiveHint` true = may delete/overwrite (13 tools), `idempotentHint` true = retry-safe, `openWorldHint` true = host filesystem (wiki_ingest, bulk_import_file only). Clients that ignore annotations assume pessimistic defaults.
> Dependent ops (e.g. `put` then `search` over the new record) go in sequential invocations, not one multi-call batch: batches may reorder (smoke Fase 2 note).

## Legacy & absorbed names (dispatch-only)

Gate P (API-STD-15) canonicalized the tool surface to **one listed name per tool**, and WIRE-02 (2026-09-27) absorbed the redundant `code_*` projections into their canonical tool. These names stay *dispatchable* (`tools/call` resolves them) but are **not** returned by `tools/list` in any profile:

| Legacy (unlisted) | Canonical (listed) | Behavior |
|-------------------|--------------------|----------|
| `search_memory` | `memory_search` | Same shared dispatch (identical wire shape). |
| `collection_list` | `memory_list_namespaces` | Keeps its richer per-namespace metadata (`record_count`, `has_vector_index`, `created_at`) for legacy callers. |
| `code_callers` | `code_explore` | `code_explore` already splits outgoing (callees) / incoming (callers) neighbors. |
| `code_callees` | `code_explore` | Same depth-1 BFS, reverse edge direction. |
| `code_impact` | `code_explore` | Reachability projection (multi-hop via `graph_traverse`). |
| `code_node` | `code_explore` | `code_explore` returns the node record plus its neighborhood. |
| `code_status` | `capabilities` | Same `operational_metrics()` snapshot. |
| `code_files` | `code_search` | Documented "not supported" stub (the built-in graphrag has no file-per-node concept). |

The legacy prompt `search_memory` (see §Prompts) follows the same rule via `prompts/get`.

## Tool Surface Profiles (MCP-37 / WIRE-02)

The VantaDB MCP server exposes a **tool surface profile** via the `VANTADB_MCP_PROFILE` environment variable. This allows clients with tool caps (e.g., Cursor ~40 tools) to select a subset that fits their limits while preserving full functionality for unrestricted clients.

| Profile | Tool Count | Description | Recommended For |
|---------|------------|-------------|-----------------|
| `agent` (**default**) | 38 | Memory CRUD + search + recall + IQL + collections + threads + scenes + context engine + wiki read. | Agent clients out of the box (the old default `full` cost ≈ 23K tokens of tool schemas per session). |
| `full` | 80 | All listed tools: memory, graph, collections, maintenance, snapshots, backup, introspection, code intelligence (2 listed primitives), wiki, skills, threads, scenes, dreams, context engine. | Claude Desktop, Claude Code, OpenCode, unrestricted clients (pre-0.8 default; opt-in) |
| `dev` | 37 | Memory CRUD + search + IQL + graph traversal + collections + key maintenance (snapshots, export/import, flush, compact) + axioms. Excludes: code intelligence, wiki, skills, threads, scenes, context engine, bulk import, index audit/repair, vacuum, rebuild_index. | **Cursor** (cap ~40), VS Code extensions, clients with moderate tool caps |
| `memory` | 21 | Core memory CRUD (put/get/delete/list/versions/supersede) + certificate verify + search (semantic/memory/with_method/multi) + IQL + collections + capabilities + generate_snippet. | Memory-only agents, minimal clients, testing |

**Usage:**

```bash
# Agent profile (default): memory + threads + scenes + context + wiki read
vanta-cli server --mcp --db ~/.vantadb

# Full profile (all 80 listed tools — the pre-0.8 default; opt-in)
VANTADB_MCP_PROFILE=full vanta-cli server --mcp --db ~/.vantadb

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
      "args": ["server", "--mcp", "--db", "C:/Users/<you>/.vantadb"],
      "env": { "VANTADB_MCP_PROFILE": "dev" }
    }
  }
}
```

**Behavior:**
- The profile is read once at server startup from `VANTADB_MCP_PROFILE` (unknown values fall back to the default).
- `tools/list` returns only the tools allowed by the selected profile.
- `tools/call` for a known tool outside the profile returns `method_not_found` (-32601) with the clear error: `Tool not found: <name> (not in profile <profile>)` (WIRE-02: enforced in code; unknown names keep the plain `Tool not found: <name>` fall-through). Absorbed dispatch-only names (§Legacy & absorbed) stay callable while their canonical tool is listed in the active profile.
- **Migration (WIRE-02):** the default changed from `full` to `agent`. Set `VANTADB_MCP_PROFILE=full` to keep the pre-0.8 80-tool listed surface; the legacy `search_memory`/`collection_list` aliases keep working.

## Core Tools (48)

### Memory CRUD (10)

| Tool | Description |
|------|-------------|
| `memory_put` | Inserts or updates a memory record in a namespace with payload, vector, optional sparse vector, metadata, and TTL. |
| `memory_put_batch` | Stores multiple records in a single all-or-nothing batch; duplicate keys are upserts, vector dimensions must match the live index. |
| `memory_get` | Retrieves a memory record by namespace and key. |
| `memory_delete` | Deletes a memory record by namespace and key. With `attest:true`, also emits the VER-02 purge certificate (per-surface residue inventory, integrity hash, VER-01 WAL chain reference) under `certificate` — verify it with `memory_verify_certificate`. |
| `memory_verify_certificate` | DIST-16 (VER-02): verifies a stored purge certificate against the live database — counterpart of `memory_delete {attest:true}`. Checks schema + integrity hash, then re-scans the re-checkable surfaces (store, shred, vector index, version history, derived index). Accepts the certificate as a JSON object/string or the `{deleted, certificate}` envelope returned by the attested delete. Valid → `structuredContent {valid:true, verification}` (typed); edited/corrupted certificate or residues reappeared → typed `isError` envelope (ERR-MCP-01). Read-only; idempotent. The certificate is not bound to a database instance — verification matches by namespace/key/node_id against whichever database is open; the VER-01 WAL chain stays `vanta-cli verify` (cited, not duplicated). |
| `memory_delete_by_filter` | Batch-deletes every record in a namespace whose metadata matches the given filters (AND semantics). |
| `memory_list` | Lists memory records in a namespace with optional pagination and metadata filters. SCH-07 query params: `as_of_ms` / `valid_window` (valid time), `include_quarantined`, `min_confidence`. Response is bounded by `byte_budget` (default 40 KB); see [Output budgeting](#output-budgeting-byte_budget-mcp-39) for the truncation semantics. |
| `memory_list_namespaces` | Lists all available namespaces in the database (API-04 canonical name). The legacy `collection_list` alias stays dispatchable but is not listed — it returns rich collection metadata (record_count/has_vector_index/created_at) while this tool returns the bare namespace list. |
| `memory_versions` | Lists every retained version of a memory record, ascending (v1..vN); empty if the key does not exist or has no history. Expired versions are included as historical data until purged. |
| `memory_supersede` | Marks an existing record as superseded by another existing record (durable, recoverable soft-delete). Errors if either key is missing, if old_key equals new_key, or if the old record is already superseded. |

### Search & Query (7)

| Tool | Description |
|------|-------------|
| `memory_search` | Hybrid memory search in a namespace (API-04 canonical name, mem0/Letta parity): text/vector/hybrid modes, filters, distance metric, RRF tuning, and explain output. SCH-07 query params: `as_of_ms` / `valid_window` (valid time), `include_quarantined`, `min_confidence`; the response envelope carries the selective-abstention signal (`abstained`, `abstention_reason`) — see [Selective abstention](#selective-abstention-adr-046-d2). The legacy `search_memory` alias stays dispatchable but is not listed. |
| `search_semantic` | Raw semantic vector search directly in the HNSW index. |
| `search_with_method` | Memory search with an explicit dense-index backend override (`method`: hnsw \| ivf \| flat \| diskann \| scann); omit to keep automatic routing. Same parameters as `memory_search` (temporal/quarantine/confidence included) and the same abstention signal. |
| `search_multi` | Run one search request across multiple namespaces and merge results (sorted by score, capped at `top_k` globally). Same request params as `memory_search` (temporal/quarantine/confidence included); the multi-namespace merge has no page-level abstention signal (N/A). Response is bounded by `byte_budget` (default 40 KB); see [Output budgeting](#output-budgeting-byte_budget-mcp-39). |
| `query_iql` | Executes an IQL statement against typed graph nodes and memory namespaces (each namespace is queryable as a table named by its sanitized form: `/` and `-` → `_`, leading digit/`.` gets a `_` prefix). LISP not supported. Param-level failures (empty/NUL/oversize query) are JSON-RPC `invalid_params` (-32602). |
| `memory_recall` | MEM-59: High-level recall mirroring vanta-memory's auto-recall hook (MEM-18) over the public MCP surface. Runs keyword/embedding/hybrid search over L1 records visible under the given scope (session/agent/team), ranks with D38 dual-pool + RRF logic, and returns structured hits plus prepended context block (`{prepend_context, recalled, effective_mode, byte_count, truncated}`; hits carry `source_namespace`/`source_key`). Budgeted and ACL-governed — see [Injection governance](#injection-governance-ver-04). Read-only; idempotent; does not require a session_key. Param-level failures are JSON-RPC `invalid_params`. |
| `embed_texts` | Embeds a batch of texts into dense float vectors with the active provider (local ONNX real; `ollama`/`openai` when configured) and an explicit deterministic fallback. Inputs: `texts` (required, 1–128 items of 1–8000 chars), optional `model` (manifest id override, EMB-17), `cursor` pagination offset. Response always carries `fallback: false` (real vectors) or `fallback: true` + `warning` (deterministic hash, no semantic signal — never silent, Q5). Supports `max_embed_tokens` (25k) / `max_embed_batch_size` (128) budgeting. Read-only; idempotent. Verified EMB-19 (`a5d549af`): `multilingual-e5-small` dim 384 `fallback:false`, `s(par)=0.9158` vs `0.8427/0.8423` gap `0.0732`. See [Embeddings](#embeddings-providers-model-selection-and-dim-gate) below. |

## Embeddings — providers, model selection, and dim gate

Model catalog source of truth: `embeddings/manifest.json` (9 ids, rev pinned). Full table
(model → dim → langs → size → when to use) lives in `embeddings/README.md`
(§ "Cuándo usar cada modelo"); this section is the MCP-surface contract.

| Provider | Activation | Requirement | Verified state (EMB-19 `a5d549af`) |
|----------|------------|-------------|-------------------------------------|
| `local` (ONNX, recommended) | `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<absolute>/embeddings/models/<id>/onnx` + `ORT_DYLIB_PATH=<onnxruntime>=1.27.dll`, or launcher `.\vanta-mcp-local.ps1 -DbPath <db>` | Model on disk (`python embeddings/download.py --only <id>`) + onnxruntime ≥ 1.27 (EMB-11 persistent `%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`; System32 1.17.1 aborts — FIND-100) | REAL: `multilingual-e5-small` dim 384 `fallback:false`, `0.9158` vs `0.8427/0.8423` |
| `ollama` (core default when no env is set, `src/config.rs:956`) | `VANTADB_EMBEDDING_PROVIDER=ollama` (server `localhost:11434`) | Live Ollama server + embedding model pulled (`ollama pull nomic-embed-text`) | DEGRADATION-ANNOUNCED: server v0.33.2 alive but `/api/tags` → `{"models":[]}` → `fallback:true` + `warning` with `404`, exit 0. Real embedding: documented-not-executed (FIND-69 precedent) |
| `openai` | `VANTADB_EMBEDDING_PROVIDER=openai` + `VANTADB_OPENAI_API_KEY=<key>` (session env only, NEVER to disk) + optional `VANTADB_OPENAI_MODEL` | Valid key with credit | DEGRADATION-ANNOUNCED without key (`fallback:true` + `warning="VANTADB_OPENAI_API_KEY must be set"`). Real embedding: documented-not-executed without key (FIND-69 precedent) |

- **Model override (EMB-17):** `embed_texts` accepts `model: "<manifest id>"` and uses THAT
  model (per-model session cache, cap `MAX_CACHED_LOCAL_MODELS=2` with warned eviction).
  Unknown id → `invalid_params` with the valid 9-id list. Known id without files on disk →
  clear error with `python embeddings/download.py --only <id>` + size. `model: null` =
  active provider from env (EMB-13 behavior intact). Only `embed_texts` has a `model`
  slot; puts/search use the active provider (no per-call slot — by design).
- **Auto-embed (EMB-14):** `memory_put`/`put_batch` without `vector` store WITH the active
  provider's vector (`memory_get` shows it, len == dim). A supplied `vector` is respected
  (no re-embed). Provider failure → stored without vector + explicit notice.
- **Same-provider query (EMB-15):** `memory_search`/`memory_recall` with text embed the
  query with the ACTIVE provider. Verified synonyms `keys=["d1","d0","d2"]` + recall
  `hybrid` recovering D0 via felino↔gato (EMB-19 Step 1, same as EMB-15).
- **e5 prefixes (EMB-16):** e5 family uses `query:` (queries) / `passage:` (documents);
  MiniLM and others use none. Measured margin: asymmetric `0.1204` vs symmetric `0.0812`
  (+48% relative; source `docs/dev/tasks/EMB-16.md`).
- **One-dim-per-database (Q4, EMB-18):** one database = one dimension. Writing a vector
  whose dim differs from the stored vectors is REJECTED with expected + got dims and the
  exact regen command. Never silent auto-reindex. Empty base has no gate (first vector
  write defines the dim); text-only puts are never gated. Honest guide: re-embed with the
  original dim OR re-ingest everything with the new model + `rebuild_index` (MCP tool) /
  `reindex_hnsw_from_text(ns, page_size)` (SDK). `rebuild_index` alone does NOT change dims.
- **On disk (EMB-19):** `all-MiniLM-L6-v2`, `multilingual-e5-small` (default, 384d),
  `paraphrase-multilingual-MiniLM-L12-v2` (all 384d). Rest on demand (Q2: size/time warning
  before large downloads).

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
| `inject_context` | Injects external state or context connected to a specific thread for subsequent consolidation. Fail-closed injection budget: content above `byte_budget` (default 40 KB, `VANTADB_MCP_BYTE_BUDGET`) is rejected with a validation error; the response carries `byte_count` and the call is audited (see [Injection governance](#injection-governance-ver-04)). |
| `read_axioms` | Returns the active Devil's Advocate Axioms (Iron Axioms) in the database. |
| `write_axiom` | Registers or updates an agent axiom (invariant rule) in the reserved `_axioms` namespace; returns `{id, name, description}`. |
| `delete_axiom` | Removes an agent axiom by name from the `_axioms` namespace; returns `{deleted}`. |

### Collections (2)

| Tool | Description |
|------|-------------|
| `collection_stats` | Statistics for one namespace/collection: count, byte size, index info, creation time. |
| `collection_delete` | Deletes an entire namespace/collection and all its records (requires `confirm: "yes"`). |

> `collection_list` is a legacy alias: dispatchable but **not listed**. Use `memory_list_namespaces` for the canonical namespace listing (see §Legacy aliases).

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
| `bulk_import_file` | Bulk-imports from a binary `.vdbdump` file on the host filesystem, **by design** bypassing per-record validation for throughput (MCP-25; do not "normalize" — it would lose the raw-throughput contract). |
| `bulk_import_stream` | Bulk-imports inline NDJSON or raw `.vdbdump` content (max 10 MB); imported entries are raw engine nodes, **by design** not addressable via `memory_get`/`memory_list` (MCP-25). |

## Extended Tool Families (38 defined; 32 listed)

Dispatched via `tools/call`, defined outside `handlers/tools.rs` (8+7+6+5+6+1+5 = 38 defined; WIRE-02 absorbs 6 `code_*` → 32 listed):

### Code Intelligence — `code.rs` (8 defined; 2 listed)

| Tool | Description |
|------|-------------|
| `code_search` | Searches indexed code symbols. |
| `code_explore` | Explores symbols with call paths and blast radius. |
| `code_callers` | *(dispatch-only, WIRE-02)* Lists callers of a symbol. |
| `code_callees` | *(dispatch-only, WIRE-02)* Lists callees of a symbol. |
| `code_impact` | *(dispatch-only, WIRE-02)* Impact analysis for a change target. |
| `code_node` | *(dispatch-only, WIRE-02)* Fetches a single code-graph node. |
| `code_files` | *(dispatch-only, WIRE-02)* Lists indexed files — "not supported" stub (no file-per-node concept). |
| `code_status` | *(dispatch-only, WIRE-02)* Index health/status of the code graph. |

> Absorbed names are never listed in any profile; call them through their canonical tool (see §Legacy & absorbed).

### Skills Management — `skills.rs` (7)

| Tool | Description |
|------|-------------|
| `skill_list` | Lists installed agent skills. |
| `skill_view` | Views a skill's content. |
| `skill_create` | Creates a new skill. |
| `skill_update` | Updates an existing skill. |
| `skill_patch` | Applies targeted patches to a skill. |
| `skill_files_write` | Writes supporting files for a skill. |
| `skill_extract` | Reviews a transcript and returns reusable-skill candidates WITHOUT writing anything (candidates-only, read-only). No LLM runner is configured in MCP, so non-empty transcripts honestly degrade to `{success:false, candidates:[], error}`; empty transcripts succeed trivially with `[]`. |

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
| `context_assemble` | Assembles a context window under a token budget with the vanta-memory context engine (MCP-31): compacts the provided chat history and injects session recall (relevant L1 memories, persona, scene navigation). Returns `{messages, report, mmd_injected, recall_injected, byte_count, truncated}`; recall is ACL-governed and audited per the [Injection governance](#injection-governance-ver-04) rules. Read-only. |

### Scenes API - `scenes.rs` (5)

Read wrappers over the vanta-memory gateway scene handlers (`vanta_memory::gateway`) plus the sandboxed write path (`vanta_memory::core::scene::scene_tools`) — structured scene navigation for external agents. Domain errors surface as error-content messages; `scene_query` ranks by keyword overlap only.

| Tool | Description |
|------|-------------|
| `scene_read` | Reads one live scene block by name from a session's scene store. Returns `{scene:{scene_name, meta{created,updated,summary,heat}, content}}`. Missing or soft-deleted scenes answer "not found". Read-only. |
| `scene_list` | Lists the scene index of a session (heat descending, soft-deleted excluded). Returns `{scenes:[{filename,summary,heat,created,updated}]}` where `filename` is the id for `scene_read`. Read-only. |
| `scene_query` | Keyword search over live scene blocks: ranks scenes by term overlap between the keyword and summary+content, ties by heat. Returns `{hits:[{scene_name,summary,heat,updated,score}]}`; load hits via `scene_read`. Read-only. |
| `scene_write` | Creates or fully replaces one scene block (upsert: heat bump, `created` preserved on update). Returns `{scene:{...}}`. Empty/whitespace-only content is rejected as an error-content message. Write path. |
| `scene_edit` | Patches `summary` and/or `content` of an existing scene (at least one field; missing scene answers "not found"). Returns `{scene:{...}}`. Write path. |

### Dreams API - `dreams.rs` (5)

Read wrappers plus a scoped delete over the vanta-memory dream store (`vanta_memory::core::dream` over `dream/<session>/<run_id>`) — reviewable, discardable idle consolidation — plus the LLM-free consolidation pass and the promotion surface (dry-run plan by default; apply only with `dry_run:false`, idempotent and gated). Consolidation never touches `l1/<session>`; promotion is the module's only mutating entry point. Domain errors surface as error-content messages.

| Tool | Description |
|------|-------------|
| `dream_list` | Lists every dream run for a session (metadata only: `run_id`, `started_at_ms`, `ended_at_ms`, `inputs_scanned`, `runner_label`). Load a run via `dream_load`. Read-only. |
| `dream_load` | Loads the full persisted dream run (consolidated view; originals never replaced). Missing runs answer "not found". Read-only. |
| `dream_discard` | Discards one dream run (deletes `dream/<session>/<run_id>`) after review. L1 remains untouched. Idempotent. Scoped destructive (dream namespace only). |
| `dream_consolidate` | Runs one LLM-free consolidation pass (`dedupe + contradictions + date normalization`) and persists the view to `dream/<session>/<run_id>`. Fails as error-content when not idle. Write path. |
| `dream_promote` | Promotes a run into `l1/<session>`: returns the record-level diff `{action: ADD\|UPDATE\|DELETE\|NOOP, key, reason}` (`reason`: `normalize`/`supersede`/`merge`/`dedup`/`new`/`content`/`unchanged`) and applies it only with `dry_run:false` (default `dry_run:true` = preview, nothing mutates). Idempotent (re-apply → all NOOP). Fail-closed quality gate on supersedes; DELETE only touches records the run scanned (post-run additions survive). |

## Prompts (4)

`prompts/list` returns 4 workflow templates. **Prompts are a separate registry from tools** (API-04): no prompt name shadows a tool name, so clients can never confuse a workflow with a callable tool.

| Prompt | Description | Arguments |
|--------|-------------|-----------|
| `recall_search` | Recall-first memory search: `memory_recall`, then hybrid `memory_search` with deterministic temporal ranges. | `namespace` (required), `query` (required), `filters` (optional) |
| `analyze_namespace` | Analyze a namespace for structure AND vigencia: clusters plus TTL, supersession and curation signals. | `namespace` |
| `summarize_context` | Summarize context honouring supersession and TTL: superseded records are history, not current state. | `namespace`, `limit` (optional) |
| `query_builder` | Build IQL queries with honest temporal rules: no server-side time-travel WHERE on memory records. | `operation`, `target`, `conditions` (optional) |

> The legacy prompt name `search_memory` still resolves via `prompts/get` (redirect to `recall_search`) so saved prompts keep working; it is not listed.

## Resources (2 fixed URIs + 2 dynamic schemes)

`resources/list` advertises two fixed URIs; `resources/read` additionally serves two dynamic schemes:

| URI | Kind | Content |
|-----|------|---------|
| `metrics://` | fixed | Current operational metrics (memory usage, HNSW statistics, storage information). |
| `schema://` | fixed | Active HNSW config + text index schema/tokenizer version. |
| `memory://{namespace}/{key}` | dynamic | One memory record as JSON. |
| `namespace://{namespace}` | dynamic | First page of the namespace with `next_cursor` (full pagination via `memory_list`). |

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
| `search_multi` | `content[0].text` stays the raw hits array (back-compat); `structuredContent` carries `{hits, byte_count, truncated}` | Trailing `hits` are popped in both the text and the structuredContent copy. `truncated: true` flags the trim. |
| `memory_search` / `search_with_method` | Same raw-hits text array (back-compat); `structuredContent` carries `{hits, byte_count, truncated, abstained, abstention_reason}` | Same trailing-hits budget policy. The two extra fields are the selective-abstention signal (ADR-0046 §D2, SCH-07) — see [Selective abstention](#selective-abstention-adr-046-d2). |
| `search_semantic` | Same budgeted envelope `{hits, byte_count, truncated}` | Same trailing-hits policy. |

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

> VER-04: within the same byte budget, the **memory-injection** surfaces
> (`memory_recall`, `context_assemble`, `inject_context`) enforce the identical
> cap — see [Injection governance](#injection-governance-ver-04).

## Injection governance (VER-04)

The surfaces that feed memory into a model context (`memory_recall`,
`context_assemble`, `inject_context`) run under three per-request rules:

**1. Budget.** `memory_recall` caps its recalled lines at the source
(`max_chars_per_memory` / `max_total_recall_chars` = `byte_budget`) and then
measures its envelope against `byte_budget` with a 64-byte reserve for the
`byte_count`/`truncated` metadata — the delivered payload stays within the
cap; `truncated: true` flags popped trailing hits and `byte_count` reports
the delivered envelope size. `inject_context` content is **fail-closed**:
content above `byte_budget` is rejected with a validation error (raise
`VANTADB_MCP_BYTE_BUDGET` to allow more) — never silently truncated.
`context_assemble` budgets by its own `token_budget` parameter (the shared
MEM-37 assembly budget, unchanged); its additive `byte_count` reports the
serialized messages size and `truncated` means the history had to be
compacted/dropped to fit that token budget — neither is measured against the
MCP byte budget.

**2. Namespace ACL (opt-in).** Set `VANTADB_MCP_INJECT_NAMESPACES` to a
comma-separated list of namespace prefixes to restrict which namespaces the
recall surfaces may read:

```bash
VANTADB_MCP_INJECT_NAMESPACES="l1/,persona/,scene/" vanta-cli server --mcp --db ~/.vantadb
```

Empty (default) allows everything — current behavior. Matching is
boundary-aware (`l1/sess-1` allows `l1/sess-1/...` but never `l1/sess-12`).
Anything not matched is skipped and recorded as a `denied` audit event —
never silent (the governance lists are bounded to 16 entries; when more
namespaces were involved the last slot degrades to `…overflow`, so the bound
never hides the ACL's existence). A pass where the ACL denies every source
still audits: nothing is injected, but the `denied` events are recorded.

**3. Injection audit (opt-in).** Set `VANTADB_MCP_AUDIT_LOG` to a file path to
record one metadata-only event per injected memory and per ACL denial
(append-only JSONL, rotated 10 MiB × 5):

```json
{"timestamp":"2026-09-29T12:00:00Z","op":"injection","namespace":"l1/mcp","key":"m1","outcome":"ok","reason":"surface=mcp;tool=memory_recall;session=mcp;kind=l1;score=3;budget=40960;acl=allow"}
```

Registered fields: source namespace/key, score, budget, ACL decision, tool and
session. Memory **content never lands in the audit**. Consult with `jq`:

```bash
jq -c 'select(.op=="injection" and .outcome=="denied")' audit.jsonl
jq -c 'select(.op=="injection" and (.reason | contains("tool=memory_recall")))' audit.jsonl
```

Both knobs are read once at startup (`McpConfig::from_storage`, same as
`VANTADB_MCP_PROFILE`/`VANTADB_MCP_BYTE_BUDGET`); changing them requires a
server restart. This JSONL is WORM-ready; cryptographic chaining is out of
scope here — the WAL hash-chain (`vanta-cli verify`, VER-01) is the chained
evidence surface, cited not duplicated.

## Selective abstention (ADR-0046 §D2)

When the server runs with `VANTADB_CONFIDENCE_THRESHOLD=<t>` (or
`Config::confidence_threshold = Some(t)`, `t` finite in `[0, 1]`), search hits
whose record `confidence` is below `t` are dropped and an emptied page carries
an explicit signal instead of a silent `[]`:

| Field (`structuredContent`) | Value |
|-----------------------------|-------|
| `abstained` | `true` when the threshold removed every candidate |
| `abstention_reason` | `no_candidates_above_threshold` — every candidate fell below `t`; `all_quarantined` — every candidate was quarantined and the default-exclude gate removed it |

Surfaces: `memory_search` (and legacy `search_memory`), `search_with_method`,
and the single-namespace HTTP `SearchPageV2` (`abstained` +
`abstention_reason` top-level). `search_multi` merges per-namespace rankings
with no page object — the signal is N/A there. The array-shaped binding APIs
(Python/TS/Node/WASM `search`) return hit arrays and have no page either; the
declared parity note lives in `docs/api/BINDINGS_NAMESPACES.md`.

The per-request `min_confidence` filter is **not** an abstention trigger: it
narrows results and never sets these fields. Default (`None` = OFF) keeps the
historic empty page byte-identical.

## Quarantine (ADR-0046 §D5)

Records written with `memory_put`'s `quarantine: true` flag (T1) enter the
**quarantined** state — distinct from `superseded` and TTL expiry:

- Excluded by default from `memory_search`, `memory_list` and every recall
  path (`memory_recall`, `context_assemble`); `include_quarantined: true` is
  the explicit opt-in to inspect the queue.
- `memory_get` always returns a quarantined record with **visible state**
  (`quarantined_at_ms`, `quarantine_reason`, `quarantined_by`,
  `quarantine_review_due_ms`) — never a silent 404.
- **Sticky:** a re-`put` over a quarantined key preserves the state.
- **Never auto-promoted:** the review deadline is a metric signal only
  (`keep` policy); promotion/rejection are explicit core SDK operations
  (`Embedded::quarantine_promote` / `quarantine_reject`). MCP-level
  quarantine transition tools are a declared follow-up — the current MCP
  surface supports flagging at write time and the opt-in views.

## Parity

Tool coverage on this page is enforced mechanically by `scripts/validate-docs-coverage.ps1` against `handle_tools_list()` in `vantadb-mcp/src/handlers/tools.rs`. Last sync: **2026-10-04** (DIST-16: `memory_verify_certificate` — VER-02 certificate verify via the SDK `verify_purge_certificate` (CLI parity), 79→80 listed / 85→86 defined; Core 47→48, Memory CRUD 9→10; profiles agent 37→38 · full 79→80 · dev 36→37 · memory 20→21; WAL chain VER-01 cited, not duplicated → FIND-266). Prior — 2026-09-29 (SCH-07/F3.5: v2 query params — `as_of_ms`, `valid_window`, `include_quarantined`, `min_confidence` — on `memory_search`/`search_with_method`/`search_multi`/`memory_list`, plus the selective-abstention section; tool count unchanged at 79 listed). Prior — 2026-09-27 (WIRE-02: 85→79 listed — 6 redundant `code_*` projections absorbed dispatch-only; default profile `full`→`agent`; `tools/call` profile enforcement live. Prior — 2026-09-25, API-04: 87→85 — canonicalized `search_memory`/`collection_list` out of the listing; prompts renamed `recall_search`; `thread_id` is now a u128 decimal string).

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
[`/docs/user/operations/MCP_REGISTRY.md`](../user/operations/MCP_REGISTRY.md).

> Pre-mortem: the registry submission PR is a **manual** step (not a CI gate).
> Until it's approved, the `server.json` lives in-repo as a discoverable descriptor;
> the binary itself is the source of truth. See
> [`MCP_REGISTRY.md#submission-state`](../user/operations/MCP_REGISTRY.md#submission-state)
> for the current PR/approval state.
