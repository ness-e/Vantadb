---
title: VantaDB Model Context Protocol (MCP) Server
kind: reference
status: active
description: "MCP server reference — tools, profiles, and the current implementation version (tracked below)"
tags: [vantadb, api]
---

# VantaDB Model Context Protocol (MCP) Server

Current MCP implementation version: 0.9.0

The VantaDB MCP server (`vantadb-mcp`) exposes the database to LLM agents over the Model Context Protocol. Tool definitions live in `vantadb-mcp/src/handlers/tools.rs` (`handle_tools_list`); per-IDE setup lives in the VantaDB MCP Skill.

## Getting Started

VantaDB runs as a **stdio JSON-RPC 2.0 server**: your MCP client spawns the process, talks to it over stdin/stdout, and every memory operation (store, search, graph, wiki, skills) becomes a tool the agent can call. No daemon, no network port.

### Requirements

- The VantaDB CLI (`vanta-cli` ≥ **0.9.0**; **`vanta-cli` ≥ 0.10.0** required for the default `brain` MCP profile) installed and on PATH — see [Embedded CLI](../../README.md#embedded-cli) for one-line installers.
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
| `VANTADB_BUSY` / `DatabaseBusy` when running a local script alongside the IDE | VantaDB holds an exclusive OS lock on `<db_path>/.vanta.lock` (`src/storage/engine/init.rs:262-305`). If your IDE's MCP server owns `--db <path>`, a concurrent writable Python/Node/CLI script on the same path fails with `VANTADB_BUSY`. Use a separate `--db` path for local scripts, pause the MCP server, or call the MCP tools directly. |
| Tools error at call time | Check disk space and permissions on the `--db` directory; use `capabilities` to introspect the engine state. |
| Text search fails with `text_index not found` | Restarting the server reconciles indexes automatically at startup (`ensure_indexes_current`). |

For the full behavioral contract (error channels, response envelope, edge cases), see the VantaDB MCP Skill.

### Writer discovery & re-election (MCP-48)

When a second `vanta-cli server --mcp --db <path>` finds the database locked, it does not fail: it
reads `<db_path>/.vanta.server.json`, validates the writer's PID and `GET /api/v2/health`, and forwards
storage-backed methods over loopback HTTP to that writer (proxy mode). `initialize`, `tools/list`,
`prompts/*` are answered locally, so the client's tool surface never depends on the writer.

If the writer dies, the proxy **re-elects** instead of failing forever:

| Step | Behavior |
|---|---|
| Writer unreachable (connect refused) | The handle is invalidated and the writer is re-resolved; the call is replayed only because it provably never reached the writer. |
| Writer outcome unknown (timeout, 5xx, undecodable body) | The handle is invalidated, but the call is **not** replayed: the writer may already have executed it. The client gets `-32603` with `data.retriable: false` and a hint to verify state first. |
| Writer answered `4xx` | A live writer refusing (auth, rate limit, payload). Passed through with `retriable: false`; no re-election, no extra round-trip. |
| Re-resolve | Up to 3 probe passes with async backoff (50 ms, 150 ms total); a fresh `.vanta.server.json` is enough to reconnect. Concurrent requests single-flight through one election. |
| No writer left | The stale discovery file is removed **only if its PID is provably dead** (a health timeout, a partial read or an unparseable file is never evidence of death), then this process promotes itself to writer by opening the storage. |
| Promotion safety | The promotion takes the same fs2 exclusive lock as any writer. If another writer holds it, the open fails with `DatabaseBusy` and the proxy stays a proxy — two writers on one database are impossible by construction. |
| Promotion limits | It needs the same write config as a first-time writer, including the encryption key; without it (or with a `read_only` config) the session stays proxy-only and answers `-32001`. |
| Still no writer | The call returns `-32001` (`VANTADB_BUSY`) with `data.retriable: true`; nothing retries in a loop. |
| Observable state | `tracing` events (`proxy reconnected to writer`, `proxy promoted itself to writer`) plus a 30 s `MCP proxy metrics` log line with `role`, `writer`, `reconnects`, `resolve_failures`. |

A writer that is merely slow is **not** treated as dead: a live PID keeps its discovery entry, so the
proxy cannot promote over it, and a timeout is reported as an unknown outcome rather than as damage.
Domain errors from a live writer (`success: false`, e.g. `-32009`) pass through without any
re-election attempt.

### Programmatic Tool Calling & Cloudflare Code Mode

- **Anthropic Programmatic Tool Calling (PTC):** Anthropic's PTC specification excludes tools provided via MCP connectors, and `allowed_callers` is a Messages API field rather than part of the MCP tool JSON schema. Consequently, `vantadb-mcp` does **not** emit `allowed_callers`.
- **Cloudflare Code Mode (`@cloudflare/codemode`):** Code Mode derives typed TypeScript wrappers directly from `tools/list` JSON Schemas and works with `vantadb-mcp` out of the box when started with **`VANTADB_MCP_PROFILE=full`** (or `dev` / `memory`). Do **not** use the default `brain` profile with Code Mode: `brain` collapses the extended tool catalog behind the generic `vanta_dispatch` meta-tool, hiding per-tool schemas from Code Mode's type generator.

## Error Handling & JSON-RPC Codes

The VantaDB MCP server speaks [JSON-RPC 2.0](https://www.jsonrpc.org/specification)
over stdio. A **domain/typed** error response carries both a **JSON-RPC transport
code** (the `error.code` field) and a **Vanta canonical code** (in
`error.data.code`) from the 10-element contract in
[`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md). The five **standard JSON-RPC
factories** below are transport-level only: they omit the `data` member
entirely (`vantadb-mcp/src/error.rs:21-27,123-129` — `plain()` sets
`data: Value::Null`, and `to_json()` drops the key when it is null).

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

**90 unique listed tools across profiles (`84` in `full` + `6` `brain`-exclusive in `tools/list`; `96` defined = `55` in `handle_tools_list()` + `41` extended, minus `6` WIRE-02 absorbed `code_*` projections; spec 2025-06-18, every tool carries `annotations` with `title`, `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint` per [MCP Tool Annotations](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) / [blog 2026-03-16](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations)):**

| Family | Count | Source module |
|--------|-------|---------------|
| Core | 55 defined (49 in `full` + 6 `brain`-exclusive in `tools/list`) | `handlers/tools.rs` — defined in `handle_tools_list()` |
| `code_*` | 3 listed (9 defined) | `code.rs` + `code_index.rs` — 6 projections absorbed dispatch-only (WIRE-02) |
| `skill_*` / `bugfix_*` | 9 | `skills.rs` |
| `wiki_*` | 6 | `wiki.rs` |
| `context_assemble` | 1 | `context.rs` |
| `scene_*` | 5 | `scenes.rs` |
| `thread_*` | 6 | `threads.rs` |
| `dream_*` | 5 | `dreams.rs` |

> Annotations are display hints (untrusted, not enforcement): `readOnlyHint` true = no persistent mutation, `destructiveHint` true = may delete/overwrite (13 tools), `idempotentHint` true = retry-safe, `openWorldHint` true = host filesystem (wiki_ingest, bulk_import_file, code_index). Clients that ignore annotations assume pessimistic defaults.
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
| `brain` (**default**) | 16 | Compact cognitive surface (L0-L3 + `query_iql` + meta-tools + bugfix): `memory_recall`, `memory_search`, `memory_put`, `memory_get`, `memory_forget`, `memory_supersede`, `memory_dream`, `query_iql`, `context_assemble`, `thread_send`, `vanta_scene`, `vanta_dream`, `vanta_graph`, `vanta_dispatch`, `bugfix_record`, `bugfix_query` (meta-tool `vanta_dispatch` keeps all 96 defined tools — 90 listed + 6 dispatch-only `code_*` — callable on demand). Requires `vanta-cli >= 0.10.0`. | Autonomous agents, Cursor/Windsurf/Claude Code with strict context budgets (default since PRD-01) |
| `agent` | 39 | Memory CRUD + search + recall + reinforce + IQL + collections + threads + scenes + context engine + wiki read. | Agent clients wanting full un-multiplexed memory/scenes surface (the pre-PRD-01 default) |
| `full` | 84 | All listed tools: memory, graph, collections, maintenance, snapshots, backup, introspection, code intelligence (3 listed: search/explore/index), wiki, skills & bugfix patterns, threads, scenes, dreams, context engine. | Claude Desktop, Claude Code, OpenCode, unrestricted clients (pre-0.8 default; opt-in) |
| `dev` | 38 | Memory CRUD + search + IQL + graph traversal + collections + key maintenance (snapshots, export/import, flush, compact) + axioms. Excludes: code intelligence, wiki, skills, threads, scenes, context engine, bulk import, index audit/repair, vacuum, rebuild_index. | **Cursor** (cap ~40), VS Code extensions, clients with moderate tool caps |
| `memory` | 22 | Core memory CRUD (put/get/delete/list/versions/supersede/reinforce) + certificate verify + search (semantic/memory/with_method/multi) + IQL + collections + capabilities + generate_snippet. | Memory-only agents, minimal clients, testing |

**Usage:**

```bash
# Brain profile (default): compact cognitive surface (L0-L3 + query_iql + meta-tools)
vanta-cli server --mcp --db ~/.vantadb

# Agent profile: memory + threads + scenes + context + wiki read
VANTADB_MCP_PROFILE=agent vanta-cli server --mcp --db ~/.vantadb

# Full profile (all 84 listed tools — the pre-0.8 default; opt-in)
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
- **Strict profile & custom tool allowlist (`FIND-684`):** by default `brain` lists 16 tools while allowing execution of all ~90 known tools. Set `VANTADB_MCP_STRICT_PROFILE=true` (`1`/`true`/`yes`) to restrict `brain` dispatch strictly to its 16 listed tools, or set `VANTADB_MCP_ALLOW_TOOLS` (comma-separated tool names, e.g. `memory_put,memory_get,memory_list,memory_list_namespaces,memory_search,memory_recall,memory_reinforce,capabilities`) to intersect `tools/list` and enforce a hard `tools/call` allowlist before any side effects run.
- **Migration (WIRE-02):** the default changed from `full` to `agent`, and subsequently to `brain` (`vanta-cli >= 0.10.0`). Set `VANTADB_MCP_PROFILE=full` to keep the full listed surface (the pre-0.8 default; 84 tools); the legacy `search_memory`/`collection_list` aliases keep working.
- **Minimum CLI version for `brain` (FIND-626):** the `brain` profile requires `vanta-cli >= 0.10.0`. Running `vanta-cli mcp init --profile brain` probes the resolved `vanta-cli` binary on `PATH` and emits a non-fatal warning on `stderr` when the binary predates `0.10.0` (or cannot be verified within its timeout).

## Core Tools (55 defined: 49 in `full` + 6 `brain`-exclusive in `tools/list`)

### Memory CRUD (13 — 11 in `full`/`memory` + 2 `brain`-listed: `memory_dream`, `memory_forget`)

| Tool | Description |
|------|-------------|
| `memory_put` | Inserts or updates a memory record in a namespace with payload, vector, optional sparse vector, metadata, TTL, `quarantine`, `dedup_policy` (`'off'`, `'exact'`, or `'semantic'`; falls back to `VANTADB_DEDUP_POLICY` across all profiles — FIND-683), and optional provenance fields (`confidence` in `[0, 1]` for asserted records, default `1.0` and forbidden when `'derived'`; `confidence_class`: `'asserted'` default or `'derived'`; `derived_from`: same-namespace parent keys, required non-empty when `'derived'` and forbidden when `'asserted'` — FIND-278). |
| `memory_put_batch` | Stores multiple records in a single **atomic, staged** batch operation (`FIND-640`; each item accepts the same fields as `memory_put`, including per-record `dedup_policy`, `confidence`, `confidence_class`, and `derived_from` — FIND-278 / FIND-683). **Stage A** pre-validates shape, namespace/key, metadata, vector dimensions, **and** domain invariants (`derived_from` parents, cycles, depth limits against `batch_seen ∪ store`) across the entire batch before any write — any validation or domain failure aborts the whole call with zero writes. **Stage B** writes KV + WAL in a single batch-wide framed transaction (`Begin → per-chunk Insert → Prepare → apply → Commit` under one `insert_lock` hold via `batch_insert_with_opts(..., skip_existing_check=false)`), making the whole batch atomic both in-process and across crash recovery (`ADR-0061`). **Stage C** commits netted derived/text/sparse index ops atomically and refreshes the stats cache post-commit. Duplicate keys are upserts; vector dimensions must match the live index. |
| `memory_get` | Retrieves a memory record by namespace and key. |
| `memory_delete` | Deletes a memory record by namespace and key. With `attest:true`, also emits the VER-02 purge certificate (per-surface residue inventory, integrity hash, VER-01 WAL chain reference) under `certificate` — verify it with `memory_verify_certificate`. |
| `memory_verify_certificate` | DIST-16 (VER-02): verifies a stored purge certificate against the live database — counterpart of `memory_delete {attest:true}`. Checks schema + integrity hash, then re-scans the re-checkable surfaces (store, shred, vector index, version history, derived index). Accepts the certificate as a JSON object/string or the `{deleted, certificate}` envelope returned by the attested delete. Valid → `structuredContent {valid:true, verification}` (typed); edited/corrupted certificate or residues reappeared → typed `isError` envelope (ERR-MCP-01). Read-only; idempotent. The certificate is not bound to a database instance — verification matches by namespace/key/node_id against whichever database is open; the VER-01 WAL chain stays `vanta-cli verify` (cited, not duplicated). |
| `memory_delete_by_filter` | Batch-deletes every record in a namespace whose metadata matches the given filters (AND semantics). |
| `memory_list` | Lists memory records in a namespace with optional pagination and metadata filters. SCH-07 query params: `as_of_ms` / `valid_window` (valid time), `include_quarantined`, `min_confidence`. Response is bounded by `byte_budget` (default 40 KB); see [Output budgeting](#output-budgeting-byte_budget-mcp-39) for the truncation semantics. |
| `memory_list_namespaces` | Lists all available namespaces in the database (API-04 canonical name). The legacy `collection_list` alias stays dispatchable but is not listed — it returns rich collection metadata (record_count/has_vector_index/created_at) while this tool returns the bare namespace list. |
| `memory_versions` | Lists every retained version of a memory record, ascending (v1..vN); empty if the key does not exist or has no history. Expired versions are included as historical data until purged. |
| `memory_supersede` | Marks an existing record as superseded by another existing record (durable, recoverable soft-delete). Errors if either key is missing, if old_key equals new_key, or if the old record is already superseded. |
| `memory_reinforce` | MEMG-02: outcome loop — report the outcome of a recalled memory and feed it back into the record's confidence (explicit host signal; the engine never infers it). `outcome`: `used` bumps confidence +0.05 (saturated at 1.0) and stamps `last_validated_at_ms`, at most once per 5-minute window; `corrected` decays −0.10 (floored at 0.0) without stamping; `unused` is neutral (audit-only). Derived records are rejected (their score is computed from parents). Returns `structuredContent {namespace, key, confidence, last_validated_at_ms, outcome}`. Policy: [scores.md §Reinforcement](./scores.md). |
| `memory_dream` | *(`brain` profile listed; callable in `tools/call` via `vanta_dispatch`)* TEC-07/PRD-06: consolidates unconsolidated L1 memories for a namespace/session via dreaming pass (dedupe, contradiction resolution, relative date normalization, and safe auto-promotion of high-confidence patterns). |
| `memory_forget` | *(`brain` profile listed; callable in other profiles via `tools/call`)* Unified semantic/physical forgetting tool (`mode`: `soft` \| `hard` \| `certified`, default `'hard'`; optional `namespace`, default `'default'`) targeting a `key`, metadata `filters`, or semantic `query` (`{text, top_k, min_score}`). In `hard`/`certified` mode, purges matching records across KV, vector, lexical, and graph surfaces (with cryptographic purge certificates in `'certified'` mode). |

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
| `snapshot_create` | Creates a filesystem snapshot under `<data_dir>/snapshots/<name>`; returns the path. |
| `snapshot_restore` | DESTRUCTIVE: schedules snapshot `<name>` for the next writable open (requires `confirm: true`; `name` must be a plain identifier). Restart the server to apply; writes made before restart are discarded. Delete `<storage_root>/.restore_pending` to cancel. The response reports `scheduled: true` and `restored: false`. Data and backend are staged with rollback on I/O errors; this is not a crash-atomic two-tree transaction. If an interruption leaves staging directories, reopening fails and requires manual recovery. Legacy snapshots without either format marker restore data only. |
| `list_snapshots` | Lists physical snapshot names under `<data_dir>/snapshots`. |
| `purge_expired` | Scans all records and physically deletes those whose TTL expiry has passed. |
| `rehydrate` | Recovers shadow-archived nodes that belonged to a summary node from TombstoneStorage. |
| `vacuum` | Purges tombstoned nodes from the HNSW index; returns a report with scanned_nodes, removed_nodes, reclaimed_bytes, duration_ms, and success. |

RocksDB snapshots copy a native checkpoint rather than the live backend root.
The inventory records the backend kind; at the next writable open, a mismatch
with the configured backend fails before staging and keeps the pending marker.
Restore does not migrate backends. Legacy snapshots have no backend kind.
Since v0.10.0 the `snapshot_restore` response reports `restored: false` +
`scheduled: true` (the swap is deferred to the next writable open); earlier
versions performed the swap inline and reported `restored: true`.

Scheduling and consuming pending restores are serialized by a bookkeeping
lock. Contention beyond five seconds returns retriable `DatabaseBusy`; retry
after the in-flight restore or scheduling operation completes.

### Introspection & Utility (6 — 2 in `full`/`memory`/`dev` + 4 `brain`-listed: `vanta_*`)

| Tool | Description |
|------|-------------|
| `capabilities` | Introspects supported engine features: runtime profile, persistence, vector search, IQL queries, read-only mode. Also reports **embeddings** (`embedding`) and the **memory ingest pipeline** (`ingest`: runner in use, last pass, live queue and derived `health`/`drain_path` — see the note below). |
| `generate_snippet` | Generates a text snippet from a payload, optionally highlighting matched query terms. |
| `vanta_scene` | *(`brain` profile exclusive in `tools/list`)* Multiplexed scene navigation and mutation gateway (`action`: `read` \| `list` \| `query` \| `write` \| `edit`). |
| `vanta_dream` | *(`brain` profile exclusive in `tools/list`)* Multiplexed idle memory consolidation and promotion gateway (`action`: `list` \| `load` \| `discard` \| `consolidate` \| `promote`). |
| `vanta_graph` | *(`brain` profile exclusive in `tools/list`)* Multiplexed property-graph analysis and traversal gateway (`action`: `neighbors` \| `traverse` \| `page_rank` \| `degree_centrality` \| `topological_sort` \| `is_dag` \| `remove_edge`). |
| `vanta_dispatch` | *(`brain` profile exclusive in `tools/list`)* Universal MCP meta-tool for the `brain` profile: invokes any of the 96 defined VantaDB tools (90 listed + 6 dispatch-only `code_*`) by `{tool, arguments}` without inflating the `tools/list` schema token budget. |

**The `ingest` block (MEM-72, ADR-0065).** A memory pipeline without an LLM
provider is a legitimate state — the default ingest provider builds no engine and
every extraction is `NotConfigured` — which is exactly why it must not be silent.
`capabilities` therefore carries an additive `ingest` object:

```json
"ingest": {
  "runner": "none",               // unknown | none | ollama | openai
  "degraded": true,               // runner == none: extraction is disabled
  "scheduler_running": true,      // a scheduler loop is alive in this process
  "scheduler_ever_started": true, // a loop ran here at least once (process life)
  "queue": { "depth": 5, "claimed": 0 },  // read WHEN this answer was built
  "last_pass": { "at_ms": 1764100000000, "skipped": true,
                 "processed": 0, "failed": 0, "timers_fired": 0,
                 "queue": { "depth": 5, "claimed": 0 } },  // sampled BY that pass
  "health": "broken",             // unknown | idle | draining | broken
  "drain_path": "none",           // none | loop | on_demand
  "degraded_warnings": 1,
  "notice": "ingest runner is none: 5 queued task(s) will never be extracted
             (the pass is skipped). Set VANTADB_INGEST_PROVIDER=ollama|openai ..."
}
```

`queue` is a **live probe**: it is read in this process at the moment the answer
is built, so a turn enqueued a millisecond ago is already visible. The field is
nullable for callers of the library surface (`ingest::observability::status()`
has no queue to read and returns `null`); on this wire it is always an object,
because the handler always has its process's queue to read. Guard the access
anyway (`ingest.queue?.depth`): a client must not turn "I cannot read a queue"
into `0` by accident.
`health` is what a client should branch on, and it is derived, not guessed:

| `health` | Meaning |
|---|---|
| `unknown` | `queue` is `null`, or this process has no evidence of hosting the pipeline (no runner observed, no pass, no loop) — e.g. a read-only writer, or a writer whose loop is disabled (`VANTADB_SCHEDULER_INTERVAL_MS=0`) before the first pass or enqueue. **Not** "healthy". |
| `idle` | `queue.depth == 0`. A degraded runner with nothing queued is `idle` (with a loud `notice`), not `broken`. |
| `draining` | `queue.depth > 0` and something here can drain it (`drain_path` is `loop` or `on_demand`). |
| `broken` | `queue.depth > 0` and `drain_path: none` — nobody will drain it. |

`drain_path` says who can drain in this process: `loop` (a scheduler loop is
running), `on_demand` (no loop, but a host drives `run_pass` from a request hook
and has run one), `none` (degraded, or a loop that started and stopped, or no
loop and no pass ever observed). A **degraded** episode with backlog logs a
single `warn!` (not one per pass) and `degraded_warnings` counts it — a "loop
started and stopped" with a healthy runner is `broken` but logs nothing, because
the runner itself is not the problem.
`last_pass: null` means no pass ran in this process at all. `last_pass.queue`
is the occupancy **sampled by that pass**, dated by its `at_ms` — history, never
"now". `scheduler_running` goes back to `false` when the loop stops (graceful
shutdown or a dropped handle), so it never claims a loop that is gone.

**Who runs the loop (MCP-43).** The `--mcp` writer hosts the `vanta-memory`
scheduler over the thread capture queue — the same queue `thread_send` fills
(MCP-44), so a captured turn becomes memory with no other host alive. The loop
starts when the process opens the storage as the writer, and again if it is
later promoted into that role by writer re-election (MCP-48); a read-only
writer starts none (it never captures a turn, and reports
`scheduler_running:false`). The loop is stopped and joined **before** the
storage flush on shutdown, and before the writer's discovery file is dropped, so
no pass outlives the engine and no second instance is told "no writer here"
while one is still draining.
Interval: `VANTADB_SCHEDULER_INTERVAL_MS` (default `5000` in `--mcp` — an agent
needs the previous turn recallable on the next one — versus `60000` in `--http`;
`0` disables the loop). A plain **proxy starts no loop**: the queue belongs to
the writer, so a local consumer would drain someone else's queue. It says so in
its startup log (`scheduler_running=false`, reason `proxy mode`), and
`capabilities` still tells the truth there because `tools/call` is forwarded
whole to the writer — the answer comes from the process that owns the queue.

The queue is the scheduler's in-process state (ADR-0054), so `ingest` describes
**one** process: whichever process answered the call. A one-shot `mcp-call`
spawns its own `--mcp` child, and what the caller sees depends on what that child
became: a **writer** answers with *its own* queue and health (`idle` /
`drain_path: loop` when it owns the database and its loop is running), and when
another writer already holds the database the child **proxies the call**, so the
answer is that writer's own queue and health — never a merged view, and two
processes never drain the same queue. Read `ingest` from the process that holds
the backlog when the answer matters (ADR-0065 § D8).


### Backup & Bulk Import (4)

| Tool | Description |
|------|-------------|
| `export` | Exports memory records as JSONL (max 10 MB per call); pair with `import` for backup/restore. |
| `import` | Imports records from JSONL produced by `export`; malformed lines are counted as errors, not fatal. |
| `bulk_import_file` | Bulk-imports from a binary `.vdbdump` file on the host filesystem, **by design** bypassing per-record validation for throughput (MCP-25; do not "normalize" — it would lose the raw-throughput contract). |
| `bulk_import_stream` | Bulk-imports inline NDJSON or raw `.vdbdump` content (max 10 MB); imported entries are raw engine nodes, **by design** not addressable via `memory_get`/`memory_list` (MCP-25). |

## Extended Tool Families (41 defined; 35 listed)

Dispatched via `tools/call`, defined outside `handlers/tools.rs` (9+9+6+5+6+1+5 = 41 defined; WIRE-02 absorbs 6 `code_*` → 35 listed; combined with the 55 Core tools in `handle_tools_list()` = 96 defined total):

### Code Intelligence — `code.rs` + `code_index.rs` (9 defined; 3 listed)

| Tool | Description |
|------|-------------|
| `code_search` | Searches indexed code symbols. |
| `code_explore` | Explores symbols with call paths and blast radius. |
| `code_index` | Indexes a Rust source tree by symbol into a namespace (full profile): writes file-per-node records + `defines` edges; idempotent by content hash. |
| `code_callers` | *(dispatch-only, WIRE-02)* Lists callers of a symbol. |
| `code_callees` | *(dispatch-only, WIRE-02)* Lists callees of a symbol. |
| `code_impact` | *(dispatch-only, WIRE-02)* Impact analysis for a change target. |
| `code_node` | *(dispatch-only, WIRE-02)* Fetches a single code-graph node. |
| `code_files` | *(dispatch-only, WIRE-02)* Lists indexed files — "not supported" stub (no file-per-node concept). |
| `code_status` | *(dispatch-only, WIRE-02)* Index health/status of the code graph. |

> Absorbed names are never listed in any profile; call them through their canonical tool (see §Legacy & absorbed).

### Skills & Bugfix Patterns — `skills.rs` (9)

| Tool | Description |
|------|-------------|
| `skill_list` | Lists installed agent skills. |
| `skill_view` | Views a skill's content. |
| `skill_create` | Creates a new skill. |
| `skill_update` | Updates an existing skill. |
| `skill_patch` | Applies targeted patches to a skill. |
| `skill_files_write` | Writes supporting files for a skill. |
| `skill_extract` | Reviews a transcript and returns reusable-skill candidates WITHOUT writing anything (candidates-only, read-only). No LLM runner is configured in MCP, so non-empty transcripts honestly degrade to `{success:false, candidates:[], error}`; empty transcripts succeed trivially with `[]`. |
| `bugfix_record` | Records a structured bugfix pattern (symptom, root cause, fix summary, files touched, regression test) into the skill store. |
| `bugfix_query` | Queries previously recorded bugfix patterns by keyword/symptom or by touched file path. |

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

### Threads API - `threads.rs` (6)

Conversation-history CRUD over the agentic thread store (`src/agentic/thread.rs`,
reached through the SDK builder; MCP-32). Thread ids are `u128` and travel as
JSON **strings** — the same wire convention every other u128-facing tool uses.
Domain errors (unknown thread) surface as error-content results the caller can
self-correct; malformed params are JSON-RPC `-32602`.

| Tool | Description |
|------|-------------|
| `thread_create` | Creates a conversation thread. Returns `{thread_id}` (u128 string). `ttl_secs` is optional — expired threads are reaped later by `thread_purge_expired`. Write path. |
| `thread_send` | Appends one message to a thread **and feeds the memory pipeline**: `user`/`assistant` turns are captured as L0 records and one L1 extraction task is enqueued. Roles are free-form strings (`user`, `assistant`, …); other roles are stored but not captured. Returns `{ok:true, memory:{captured,queued}}`. Write path (MCP-44). |
| `thread_get` | Reads one thread with its full message history: `{thread:{thread_id,title,messages:[{role,content,timestamp}],created_at,updated_at}}`. Missing thread → error-content "not found". Read-only. |
| `thread_list` | Lists threads with `{limit?, offset?}` pagination. Returns `{threads:[...],count}`. Read-only. |
| `thread_delete` | Deletes a thread and its messages. Returns `{deleted:true}` — permanent for the thread store, no undo (turns already captured remain as memory: see below). Scoped destructive. |
| `thread_purge_expired` | Removes every thread whose TTL has expired. Returns `{purged:n}`. Scoped destructive (TTL-expired threads only). |

### Thread writes feed memory (MCP-44)

Storage is not memory: a turn appended by `thread_send` becomes recallable
memory only if it enters the pipeline. Each `user`/`assistant` turn is

1. **captured into L0** (`l0/<thread_id>`, LLM-free, idempotent — a replay
   filtered by the L0 cursor enqueues nothing), and
2. **enqueued as ONE L1 extraction task** on the process-local queue.

`memory:{captured,queued}` reports what actually happened. `queued:true` means
the task is waiting for the writer's scheduler, not that memory already exists:
one scheduler pass (`run_pass`, hosted by the `--mcp` writer since MCP-43) turns
it into `l1/*` records that `memory_recall` returns on the next turn — no
`memory_put` required.

Two boundaries worth knowing:

- **Process-local queue (ADR-0054).** The queue belongs to the process that owns
  the database. In proxy mode this handler never runs locally: the proxy
  forwards the whole `tools/call` to the writer (MCP-48), so the enqueue happens
  in the process that can drain it. A proxy must never enqueue locally — that
  queue has no drainer. The queue is in memory: queued turns live **for the
  lifetime of that process**. There is no L0→queue backfill, on purpose: a
  correct one needs a durable "consolidated" marker on the L0 path, and without
  it every restart would re-enqueue turns that were already extracted
  (duplicate LLM work). Captured L0 records survive a restart; what a restart
  can lose is the automatic re-extraction of turns captured by a process that
  died before a pass.
- **The `--mcp` writer drains it (MCP-43).** A writer-mode `--mcp` process runs
  the scheduler over this very queue (`VANTADB_SCHEDULER_INTERVAL_MS`, default
  `5000`), so a captured turn is consolidated by the next tick and recalled on
  the next agent turn. A degraded runner stays visible instead of silent: with
  a `none` ingest runner and a non-empty queue, `capabilities.ingest` reports
  `health: "broken"` (with `drain_path: "none"`) and a legible `notice` — see
  [VANTA_MEMORY.md](./VANTA_MEMORY.md) § Operational modules
  (`ingest::observability`). The first enqueued turn also teaches this
  process its runner (the same config→runner mapping `wiki_ingest` uses), so a
  threads-only `--mcp` session is not stuck reporting `unknown`.

One semantic that is easy to misread:

- **Deleting a thread does not delete memory.** `thread_delete` and
  `thread_purge_expired` operate on the thread store (chat history). Turns
  already captured stay in `l0/<thread_id>` and, once extracted, in
  `l1/<thread_id>`. A purge cascade into memory is not implemented.

### OpenCode capture plugin (MCP-47)

The OpenCode capture plugin
(`.opencode/plugins/vanta-memory-capture.ts`, outside this repo — read it in the
harness checkout) is the client-side half of this contract: it posts the turn so
nothing has to call `memory_put` by hand, and it assembles recalled memory into
the model context under the recall-first rule.

| Hook (OpenCode v2) | What it does |
|---|---|
| `session.idle` event | Reads the closed exchange with `ctx.session.context()` and posts the `user` + `assistant` turns (text parts only) to `thread_send` on a stable per-session thread (`vanta-opencode-idle-<sessionID>`, resolved from plugin storage, else `thread_list`, else `thread_create`; re-resolved if a send fails). Deduped by the assistant message id (falling back to the user id, then to the turn text). |
| `ctx.session.hook("context")` | Calls `memory_recall` with the **verbatim** user turn (`scope: "agent"`, `top_k: 5`) once per distinct turn, and injects `prepend_context` **verbatim** only when there are hits. |
| `ctx.session.hook("context")` | Publishes `capabilities.ingest` once per session, **reading** the core's `health` verdict (ADR-0065 § D3) instead of recomputing it. `broken` emits one legible line saying the pipeline captures but does not consolidate, naming `drain_path` and separating three causes (a runner that was never configured, a loop that started and died, a loop that never started). `unknown` emits one line stating that there is **no reading** — not a verdict either way — and `idle`/`draining` are silent. A payload with no `health` field (a writer older than ADR-0065) is never guessed from `runner` + `queue.depth`: the semaphore stays silent and says why once per process, because that predicate is blind to the dead-loop case. |

Contract details that are easy to get wrong:

- **Empty recall injects nothing.** `recalled: []` is an explicit "no hits"; the
  plugin then adds no system text at all. Fabricated filler is worse than a gap,
  because the model cannot tell it apart from real memory.
- **Fail-open.** Every path is guarded, and the `vanta-cli mcp-call` transport is
  bounded by a timeout (12 s for reads, 15 s for writes; override with
  `VANTADB_MEMORY_TIMEOUT_MS` / `VANTADB_MEMORY_WRITE_TIMEOUT_MS`). Two
  consecutive read failures turn recall off for the rest of the session: a dead
  or slow library costs one timeout, not one per turn.
- **The budgets are measured, not guessed.** Each `mcp-call` spawns a fresh
  server (storage init + hardware profile + the query-embedding ONNX session), so
  on the reference machine `capabilities` takes 1.6–2.5 s and `memory_recall`
  7.0–8.5 s. A 5 s read budget made recall time out on every turn — a silently
  dead feature. The write path runs on `session.idle` where nobody waits; the
  read path runs inside a model request where everybody does.
- **Turn text never travels through argv.** Both the turn (`thread_send`) and the
  recall query (`memory_recall`) are passed on stdin through `vanta-cli mcp-call`'s
  `{{placeholder}}` resolution, because argv is readable by other processes.
  Credential-shaped strings (private keys — including an unterminated paste —,
  `ghp_`/`sk-`/`AKIA` tokens, JWTs, `Bearer …`, and `password:`/`token:`/`secret:`
  assignments, quoted or bare) are redacted before posting, and tool
  calls/results are never posted at all.
- **One slow send does not fork the history.** A failed `thread_send` drops the
  cached thread id (so the next turn re-resolves or recreates it) instead of
  re-posting into a brand-new thread — a new `thread_id` is a new server-side
  `session_id`, which L0 cannot dedupe.
- **One caveat on the semaphore.** `capabilities.ingest` describes *the process
  that answers it*, and each `mcp-call` spawns a fresh `--mcp` child. A child that
  owns the database answers with its own empty queue as `idle`; a child that finds
  another writer holding it proxies the call, so the answer is that writer's own
  queue and health. `health: "unknown"` therefore never describes someone else's
  backlog — it describes a process that cannot evidence hosting one (read-only
  writer, loop disabled before the first pass). The transport decides which
  process answers; the wording no longer decides what is assumed.
- **What it does not promise.** `thread_send` *enqueues*; it does not extract.
  Until an in-process scheduler exists (`MCP-43`), a queued turn waits in L0 and
  the queue does not drain. The notice says exactly that, and it does **not**
  present `VANTADB_INGEST_PROVIDER` as the fix: setting a runner clears
  `degraded` and silences the warning while the queue still drains nowhere.

Configuration is environment-only (no secrets in files):

```sh
VANTADB_DB_PATH=/path/to/memory          # required; without it the plugin is an inert no-op
VANTADB_CLI=/path/to/vanta-cli           # optional; default: <repo>/target/{debug,release}, then PATH
VANTADB_REPO=/path/to/repo               # optional; locates the debug/release binary
VANTADB_MEMORY_TIMEOUT_MS=12000          # optional; per read call (recall, capabilities)
VANTADB_MEMORY_WRITE_TIMEOUT_MS=15000    # optional; per write call (thread_send, thread_create)
```

Mechanical gate: `node .opencode/plugins/vanta-memory-capture.selftest.mjs`
(28 tests over a mocked transport, including the branches that must never
invert — empty recall injects nothing, hits inject the block, a credential never
reaches the wire, a dead loop with queued work warns, `health: unknown` raises no
broken alarm, `queue: null` is never read as depth 0, and the semaphore speaks
once per session). The mutation that restores the old
`runner === "none" && depth > 0` predicate as the decision turns the dead-loop
case red; that run is recorded in the `FIND-749` task file.

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
`context_assemble`, `inject_context`) run under four per-request rules:

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

**3. Trust classes (opt-in; MGR-04).** Set `VANTADB_MCP_TAINTED_NAMESPACES` to
a comma-separated list of namespace prefixes to classify them as **tainted**:
their content is never injected by default (skipped + recorded as a `denied`
audit event). Orthogonal to the ACL — both compose with AND (to inject, a
source must pass the ACL *and* be trusted). `VANTADB_MCP_INCLUDE_TAINTED=1`
(also `true`/`yes`) opts back in for review workflows (the ACL still applies).
Empty (default) = every namespace trusted — current behavior. Classifying a
namespace is an explicit operator act (restart with the env set): nothing is
promoted or demoted automatically by time or content.

**4. Injection audit (opt-in).** Set `VANTADB_MCP_AUDIT_LOG` to a file path to
record one metadata-only event per injected memory and per denial (ACL or
trust; append-only JSONL, rotated 10 MiB × 5):

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

Tool coverage on this page is **partially** enforced mechanically by `scripts/validate-docs-coverage.ps1`. That gate scans only the `handle_tools_list()` block in `vantadb-mcp/src/handlers/tools.rs` (all **55** core tools: 49 in `full` + 6 `brain`-exclusive in `tools/list`) and checks that each name appears on this page — it does **not** cover the 41 extended tools (35 listed + 6 absorbed `code_*`) in §Extended Tool Families, so those are maintained by hand. Last sync: **2026-11-20** (`T44 / FIND-531` + `FIND-640`: reconciled real counts — `55` core in `handle_tools_list()` + `41` extended = `96` defined / `90` unique listed across profiles; `Brain` = **16** listed including `memory_dream`; updated `memory_put_batch` contract to staged Stage A/B/C post-`FIND-640`). Prior — **2026-11-04** (Fase 2 bugfix MCP exposure: `bugfix_record` and `bugfix_query` exposed in `tools/list` under `Brain` (→16 with `memory_dream`) and `Full` (82→84 listed / 94→96 total defined); `skills.rs` 7→9 tools). Prior — **2026-10-05** (MEMG-09: `code_index` — MGR-22 slice v0: chunker por símbolo Rust + records file-per-node + edges `defines`, idempotente por content-hash; 81→82 listed / 87→88 defined; code intelligence 2→3 listed / 8→9 defined; profiles full 81→82 · openWorldHint true 2→3. Prior — 2026-10-05 (MEMG-02: `memory_reinforce` — outcome loop → refuerzo de confianza post-recall (`used`/`corrected`/`unused`; +0.05 saturado / −0.10 con piso, ventana de 5 min, stamp success-only, derived rechazado), 80→81 listed / 86→87 defined; Core 48→49, Memory CRUD 10→11; profiles agent 38→39 · full 80→81 · dev 37→38 · memory 21→22). Prior — 2026-10-04 (DIST-16: `memory_verify_certificate` — VER-02 certificate verify via the SDK `verify_purge_certificate` (CLI parity), 79→80 listed / 85→86 defined; Core 47→48, Memory CRUD 9→10; profiles agent 37→38 · full 79→80 · dev 36→37 · memory 20→21; WAL chain VER-01 cited, not duplicated → FIND-266). Prior — 2026-09-29 (SCH-07/F3.5: v2 query params — `as_of_ms`, `valid_window`, `include_quarantined`, `min_confidence` — on `memory_search`/`search_with_method`/`search_multi`/`memory_list`, plus the selective-abstention section; tool count unchanged at 79 listed). Prior — 2026-09-27 (WIRE-02: 85→79 listed — 6 redundant `code_*` projections absorbed dispatch-only; default profile `full`→`agent`; `tools/call` profile enforcement live. Prior — 2026-09-25, API-04: 87→85 — canonicalized `search_memory`/`collection_list` out of the listing; prompts renamed `recall_search`; `thread_id` is now a u128 decimal string).

## Registry manifest

VantaDB MCP is published to the [Official MCP Registry](https://registry.modelcontextprotocol.io/)
under the namespace `io.github.ness-e/vantadb`. The canonical descriptor lives at
the repo root: [`/server.json`](../../server.json) (schema version `2025-12-11`).

| Field | Value |
|-------|-------|
| `name` | `io.github.ness-e/vantadb` |
| `version` | tracks `[workspace.package].version` (`0.9.0` as of this doc) |
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
