---
title: VantaDB HTTP API
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-08-22
aliases: []
---

# VantaDB HTTP API

> REST interface for the VantaDB HTTP server (optional, enabled via `vanta-cli server --http` or the `server` Cargo feature).
>
> **Specification rule:** [`openapi.yaml`](./openapi.yaml) is the formal, machine-readable
> specification (OpenAPI 3.1) and the single source of truth for request/response schemas.
> This document is the **narrative guide** — it explains how to use the API and shows
> worked examples. When this page and `openapi.yaml` disagree, fix one of them in the same
> PR that changes a route: do not let them drift (parity is enforced by
> `scripts/check_openapi_parity.mjs`).

## Base URL

```
http://<host>:<port>
```

Default: `http://127.0.0.1:8080`

## Authentication

Endpoints under `/api/` require a Bearer token if `api_key` is configured:

```
Authorization: Bearer <VANTADB_API_KEY>
```

Without an API key (dev mode), requests pass through unauthenticated.

## Response conventions

- **Success envelope:** write endpoints return `{ "success": true, ... }`; errors return
  `{ "success": false, "error": "<message>" }` with an optional `"hint"`.
- **Metadata values are typed.** Record metadata values are tagged enums:
  `{"topic": {"String": "intro"}}`. Plain JSON scalars (`{"topic": "intro"}`) are rejected
  with a deserialization error listing the valid variants: `String`, `Int`, `Float`,
  `Bool`, `DateTime`, `ListString`, `ListInt`, `ListFloat`, `ListBool`, `ListDateTime`,
  `Null`.
- **Namespaces may contain `/`** (e.g. `agent/main`) — URL-encode path segments as
  `agent%2Fmain`.
- **Rate limiting:** configurable via `rate_limit_rpm` (default 600 req/min). Exceeding it
  returns HTTP 429 with a `Retry-After` header.
- **CORS:** off by default; enable via `VANTADB_ALLOWED_ORIGINS`.

## Quickstart (verified transcript)

The following session was executed against a real server (`vanta-cli server --http --port
18099 --db <tmp-dir>`); outputs below are verbatim:

```bash
# Liveness
curl http://127.0.0.1:18099/health
# → {"success":true,"data":"OK"}

# Put a record
curl -X POST http://127.0.0.1:18099/api/v2/records \
  -H "Content-Type: application/json" \
  -d '{"namespace":"agent/main","key":"note-1","payload":"VantaDB is a vector-native knowledge graph","metadata":{"topic":{"String":"intro"}},"vector":[0.1,0.2,0.3,0.4],"ttl_ms":null}'
# → {"namespace":"agent/main","key":"note-1",...,"version":1,"node_id":"258969631918792983342456585065684593810","vector":[0.1,0.2,0.3,0.4],...}

# Get it back
curl http://127.0.0.1:18099/api/v2/records/agent%2Fmain/note-1

# List records in a namespace
curl "http://127.0.0.1:18099/api/v2/list?namespace=agent/main"
# → {"records":[...],"next_cursor":null}

# Hybrid search (text/BM25) — index state is ensured at server startup
# (no manual rebuild needed on a fresh database)
curl -X POST http://127.0.0.1:18099/api/v2/search \
  -H "Content-Type: application/json" \
  -d '{"namespace":"agent/main","query_vector":[],"filters":{},"text_query":"vector-native","top_k":10,"distance_metric":"Cosine","explain":false}'
# → {"records":[{"record":{...},"score":0.57536423,"explanation":null}],"next_cursor":null}

# IQL query (keywords are UPPERCASE)
curl -X POST http://127.0.0.1:18099/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query":"INSERT NODE#7 TYPE note {title: \"hello\"} VECTOR [0.5, 0.5]"}'
# → {"success":true,"data":"Mutated 1 nodes: Node 7 inserted.","node_id":7}
curl -X POST http://127.0.0.1:18099/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query":"FROM note"}'
# → {"success":true,"data":"Read 1 nodes.","nodes":[{"id":7,...}]}
```

## System

### `GET /health`

Liveness check. Unauthenticated.

```bash
curl http://127.0.0.1:8080/health
```

```json
{ "success": true, "data": "OK" }
```

### `GET /metrics`

Prometheus/OpenMetrics text format (`text/plain; version=0.0.4`), for scraping by
Prometheus or any OpenMetrics-compatible collector.

```text
# HELP vantadb_http_requests_total Total HTTP requests
# TYPE vantadb_http_requests_total counter
vantadb_http_requests_total{method="GET",route="/health",status="200"} 42
```

### `GET /api/v2/health`

Same liveness probe as `/health`, served behind auth middleware.

### `GET /api/v2/metrics`

Engine metrics as JSON for the web console: operational snapshot
(`VantaOperationalMetrics` — HNSW node count, WAL replay stats, memory breakdown,
query/import counters) plus per-namespace collection counts.

```json
{
  "metrics": {
    "startup_ms": 772,
    "wal_replay_ms": 0,
    "wal_records_replayed": 0,
    "hnsw_nodes_count": 3,
    "hnsw_logical_bytes": 268439356,
    "process_rss_bytes": 24449024,
    "records_imported": 0
  },
  "namespaces": {
    "agent/main": { "count": 1, "expiring_soon": 0, "expired": 0 }
  }
}
```

### `GET /api/v2/audit`

Query the audit event log written when auditing is enabled. Query parameter: `limit`.
Returns an array of audit event objects; HTTP 409 if the audit log is not configured.

## Query

### `POST /api/v2/query`

Execute an IQL (Interactive Query Language) statement against the database.

**Request body:** `{ "query": "<IQL statement>" }`

**IQL keywords are case-sensitive uppercase.** Verified grammar summary:

| Operation | Syntax |
|-----------|--------|
| Read | `FROM <entity> [WHERE <field> ~ "<text>" \| <field> = <value> \| <field> ~ "<text>", min = <score>] [RANK BY <field> [DESC]] [FETCH <field>,...] [PROFILE keyword\|vector\|hybrid]` |
| Insert | `INSERT NODE#<id> TYPE <type> {<key>: <value>,...} [VECTOR [<f32>,...]]` |
| Update | `UPDATE NODE#<id> SET <key>=<value>,...` or `UPDATE NODE#<id> VECTOR [...]` |
| Delete | `DELETE NODE#<id>` |
| Relate | `RELATE <source> -> <target> AS <label> [WEIGHT <f32>]` |

**Read response:**

```json
{
  "success": true,
  "data": "Read 1 nodes.",
  "node_id": null,
  "nodes": [
    {
      "id": 7,
      "semantic_cluster": 0,
      "relational": {
        "title": { "String": "hello" },
        "type": { "String": "note" }
      },
      "hits": 1,
      "confidence_score": 0.5
    }
  ]
}
```

**Write response:**

```json
{
  "success": true,
  "data": "Mutated 1 nodes: Node 7 inserted.",
  "node_id": 7,
  "nodes": null
}
```

Parse failures return `{"success": false, "data": "Execution Error: IQL parse error ..."}`.

## Records

### `POST /api/v2/records`

Create or overwrite a record (upsert by namespace+key). Body mirrors the SDK
`VantaMemoryInput`: `namespace`, `key`, `payload`, `metadata`, `vector` (nullable),
`sparse_vector` (nullable term-weight map), `ttl_ms` (nullable; null = never expires).

```json
{
  "namespace": "agent/main",
  "key": "note-1",
  "payload": "VantaDB is a vector-native knowledge graph",
  "metadata": { "topic": { "String": "intro" } },
  "vector": [0.1, 0.2, 0.3, 0.4],
  "ttl_ms": null
}
```

Response is the stored record wire shape:

```json
{
  "namespace": "agent/main",
  "key": "note-1",
  "payload": "VantaDB is a vector-native knowledge graph",
  "metadata": { "topic": { "String": "intro" } },
  "created_at_ms": 1787432535128,
  "updated_at_ms": 1787432535128,
  "version": 1,
  "node_id": "258969631918792983342456585065684593810",
  "vector": [0.1, 0.2, 0.3, 0.4],
  "sparse_vector": null,
  "expires_at_ms": null,
  "superseded_by": null,
  "superseded_at_ms": null
}
```

### `DELETE /api/v2/records?namespace=<ns>&filter=<json>`

Delete all records in a namespace whose metadata matches the JSON-encoded filter passed
as query parameter.

```bash
curl -X DELETE "http://127.0.0.1:8080/api/v2/records?namespace=agent/main&filter=%7B%22op%22%3A%22Eq%22%7D"
```

### `POST /api/v2/records/batch`

Upsert multiple records in one call. Body is an array of the same record objects used by
`POST /api/v2/records`; the response is an array of stored records.

### `GET /api/v2/records/{ns}/{key}`

Fetch a single record by namespace and key (URL-encode `/` inside the namespace as
`%2F`). Returns the same wire shape as the PUT response; missing records return
`{"error":"record not found: <key>","success":false}` with HTTP 404 semantics.

### `DELETE /api/v2/records/{ns}/{key}`

Soft-delete (tombstone) a record.

```json
{ "deleted": true }
```

A subsequent GET returns `record not found`.

### `GET /api/v2/records/{ns}/{key}/versions?limit=<n>`

Version history of a record. Returns an array of version entries.

### `GET /api/v2/list?namespace=<ns>&limit=100&cursor=<cursor>&filter_ops=<json>`

Paginated listing of records in a namespace.

```json
{
  "records": [ { "...record wire shape..." : "" } ],
  "next_cursor": null
}
```

### `POST /api/v2/export`

Export records to a JSONL file on the server. Body: `path` (required), optional
`namespace` and `filter` (array of AND-combined metadata filter objects).

### `POST /api/v2/import`

Import records inline (`records`) or from a server-side file (`path`). With a file, set
`format` to `jsonl` (default) or `bulk` for `.vdbdump` bulk files. `records` and `path`
are mutually exclusive.

## Search

### `POST /api/v2/search`

Vector / sparse / BM25 hybrid similarity search over a namespace. Wire format mirrors the
SDK's `VantaMemorySearchRequest` plus offset pagination (`cursor` = zero-based offset,
`limit` = page size, defaults to `top_k`). An empty `query_vector` skips dense search;
`text_query` drives BM25 lexical scoring. `distance_metric` is one of `Cosine`,
`Euclidean`, `Dot`; `explain: true` adds a `VantaSearchExplanation` per result.

> Text search works on fresh databases out of the box: the server ensures index state
> at startup (MOD-12). `POST /api/v2/maintenance/rebuild-index` remains available for
> explicit rebuilds of existing data.

**Request:**

```json
{
  "namespace": "agent/main",
  "query_vector": [],
  "filters": {},
  "text_query": "vector-native",
  "top_k": 10,
  "distance_metric": "Cosine",
  "explain": false
}
```

**Response:**

```json
{
  "records": [
    {
      "record": { "...record wire shape..." : "" },
      "score": 0.57536423,
      "explanation": null
    }
  ],
  "next_cursor": null
}
```

### `GET /api/v2/autocomplete?prefix=<prefix>`

IQL completion suggestions for a prefix.

```bash
curl "http://127.0.0.1:8080/api/v2/autocomplete?prefix=FR"
```

```json
["FROM"]
```

## Graph

All graph endpoints share a generic traversal request body rooted at numeric node ids:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v2/graph/bfs` | Breadth-first traversal |
| `POST` | `/api/v2/graph/dfs` | Depth-first traversal |
| `POST` | `/api/v2/graph/degree` | In/out degree map per node |
| `POST` | `/api/v2/graph/centrality` | Centrality scores per node |
| `POST` | `/api/v2/graph/pagerank` | PageRank scores per node |
| `POST` | `/api/v2/graph/v2/bfs` | BFS (v2 engine, WEB-02 extended SDK surface) |
| `POST` | `/api/v2/graph/v2/dfs` | DFS (v2 engine) |
| `POST` | `/api/v2/graph/v2/degree` | Degree (v2 engine) |

**Request (verified):**

```json
{ "roots": [7], "max_depth": 2 }
```

> `roots` are numeric node IDs and `max_depth` is required in practice; string roots or a
> missing `max_depth` return a 400 deserialization error.

**Traversal response (bfs/dfs):** array of visited nodes in visit order.

```json
[7]
```

**Degree response:** map of node id to `[in_degree, out_degree]`.

```json
{ "7": [0, 0] }
```

Centrality/PageRank return a score object keyed by node id.

## Maintenance

### `POST /api/v2/maintenance/purge`

Removes all expired (TTL elapsed) records.

```json
{ "purged": 0 }
```

### `POST /api/v2/maintenance/compact`

Compacts underlying storage layers. Returns a result acknowledgement object.

### `POST /api/v2/maintenance/flush`

Forces pending WAL/memory writes down to durable storage. Returns a result object.

### `POST /api/v2/maintenance/rebuild-index`

Rebuilds secondary indexes (text/HNSW) for the database.

```json
{
  "scanned_nodes": 1,
  "indexed_vectors": 1,
  "skipped_tombstones": 0,
  "duration_ms": 0,
  "derived_rebuild_ms": 0,
  "index_path": "<db>/data/vector_index.bin",
  "success": true
}
```

### `GET /api/v2/snapshots`

Lists snapshot names available on this database.

```json
["demo-snap"]
```

### `POST /api/v2/snapshots/{name}`

Creates a named snapshot of the current database state.

```json
{
  "name": "demo-snap",
  "path": "<db>/data/snapshots/demo-snap"
}
```

## Threads

Conversation threads store role-tagged messages as auto-embedded nodes.

### `GET /api/v2/threads?limit=<n>`

Lists threads.

```json
[
  {
    "thread_id": "310279622029206533993990662647183162021",
    "title": "demo thread",
    "messages": [],
    "created_at": 1787432771,
    "updated_at": 1787432771,
    "metadata": {}
  }
]
```

### `POST /api/v2/threads`

Creates a thread. Body: `{ "title": "<human-readable title>" }`.

```json
{ "thread_id": "310279622029206533993990662647183162021" }
```

### `GET /api/v2/threads/{id}`

Fetches a thread with its messages.

```json
{
  "thread_id": "310279622029206533993990662647183162021",
  "title": "demo thread",
  "messages": [
    {
      "role": "user",
      "content": "hello thread",
      "timestamp": 1787432790463,
      "metadata": {}
    }
  ],
  "created_at": 1787432771,
  "updated_at": 1787432790,
  "metadata": {}
}
```

### `POST /api/v2/threads/{id}`

Appends a role-tagged message (auto-embedded). Body requires `role` and `content`.

```json
{ "sent": true }
```

### `DELETE /api/v2/threads/{id}`

Deletes the thread and its messages.

## Web Console (stable)

### `GET /dashboard` and `GET /dashboard/{path}`

Web console entry point and static asset fallback for Vanta Studio. Requires starting the
server with `--dashboard-dir <dir>`; otherwise `/dashboard` responds 404 with a hint.

> Promoted from experimental to stable 2026-08-25: covered by e2e tests and served as the
> Vanta Studio admin surface (ADR-026/ADR-027).

## Experimental endpoints

> ⚠️ **Experimental** - these routes are marked `x-experimental: true` in
> `openapi.yaml`. They are unstable and may change without notice.

### `POST /conversation/add`

Legacy conversational ingestion endpoint: auto-selects or creates a thread and appends a
turn. Body includes optional `thread_id` (omitted creates/reuses the default thread).

### `GET /skill/listing?limit=<n>`

Lists skill-like records (capped at 200 items, default limit 50).

### `POST /api/v2/skills`

Creates a versioned skill (version 1). Body: `owner_agent`, `name`, `content`,
optional `description`, `metadata`, `ttl_secs`. Idempotent when the same
`(owner_agent, name)` + content already exists (`idempotent: true` in the
response). Duplicate name with different content → 409.

### `PUT /api/v2/skills/{skill_id}?owner_agent=<a>&expected_version=<n>`

Replaces description and content, appending a new immutable version. A stale
`expected_version` returns 409 (optimistic lock). A foreign `owner_agent`
returns the same 404 as a missing skill (anti-enumeration).

### `PATCH /api/v2/skills/{skill_id}?owner_agent=<a>&expected_version=<n>`

Partial update — only provided fields (`description`, `content`, `metadata`)
change. Same optimistic-lock and ownership semantics as PUT.

### `DELETE /api/v2/skills/{skill_id}?owner_agent=<a>&expected_version=<n>`

Removes every version plus the head index row.

## Starting the Server

```bash
# HTTP server only
vanta-cli server --http --port 8080 --host 127.0.0.1 --db ./vanta_data

# MCP and HTTP are mutually exclusive modes. Passing --http --mcp together
# starts MCP only (mcp_mode = mcp && !http); HTTP is not served.
vanta-cli server --mcp --port 8080 --db ./vanta_data

# With TLS
vanta-cli server --http --port 443 --db ./vanta_data
# Requires VANTADB_TLS_CERT and VANTADB_TLS_KEY env vars
# TLS requires the `tls` Cargo feature.
```

Note: the `server` Cargo feature gates the HTTP/MCP wrapper. If your build lacks it,
rebuild with `cargo build --features server`.

### Security guard: refuse-to-start on exposed unauthenticated binds (FIND-07)

The server **refuses to start** when all of the following hold:

- The bind host is non-loopback (anything other than `127.0.0.1`, `localhost`, `::1` — e.g. `0.0.0.0`)
- No API key is configured (`VANTADB_API_KEY` unset)
- No explicit dev override is given

The startup error explains every remediation path:

```text
Refusing to start: non-loopback host without an API key
Fix either way: (1) set VANTADB_API_KEY to enable Bearer auth, or
(2) bind a loopback host (127.0.0.1/localhost/::1), or (3) pass
--allow-insecure to override this check in dev.
```

**Dev override** — `--allow-insecure` bypasses the check for local development.
The server then logs a prominent warning and starts unauthenticated:

```bash
# Explicitly opt in to an exposed, unauthenticated server (dev only)
vanta-cli server --http --host 0.0.0.0 --port 8080 --db ./vanta_data --allow-insecure
```

Loopback binds without a key keep working as before (dev mode). Setting
`VANTADB_API_KEY` makes any host acceptable; `--require-auth` additionally
refuses to start without a key regardless of host.

## Route Summary

| Method | Path | Auth | Domain | Description |
|--------|------|------|--------|-------------|
| `GET` | `/health` | No | System | Liveness check |
| `GET` | `/metrics` | Bearer (if configured) | System | Prometheus metrics (OpenMetrics format) |
| `GET` | `/api/v2/health` | Bearer (if configured) | System | Authenticated health check |
| `GET` | `/api/v2/metrics` | Bearer (if configured) | System | Engine metrics as JSON |
| `GET` | `/api/v2/audit` | Bearer (if configured) | System | Audit event log |
| `POST` | `/api/v2/query` | Bearer (if configured) | Query | Execute IQL query |
| `POST` | `/api/v2/records` | Bearer (if configured) | Records | Put record (upsert) |
| `DELETE` | `/api/v2/records` | Bearer (if configured) | Records | Delete records by metadata filter |
| `POST` | `/api/v2/records/batch` | Bearer (if configured) | Records | Put records batch |
| `GET` | `/api/v2/records/{ns}/{key}` | Bearer (if configured) | Records | Get record |
| `DELETE` | `/api/v2/records/{ns}/{key}` | Bearer (if configured) | Records | Delete record (tombstone) |
| `GET` | `/api/v2/records/{ns}/{key}/versions` | Bearer (if configured) | Records | List record versions |
| `GET` | `/api/v2/list` | Bearer (if configured) | Records | Paginated record list |
| `POST` | `/api/v2/export` | Bearer (if configured) | Records | Export records to JSONL file |
| `POST` | `/api/v2/import` | Bearer (if configured) | Records | Import records (inline/file) |
| `POST` | `/api/v2/search` | Bearer (if configured) | Search | Hybrid search |
| `GET` | `/api/v2/autocomplete` | Bearer (if configured) | Search | IQL autocomplete |
| `POST` | `/api/v2/graph/bfs` | Bearer (if configured) | Graph | Breadth-first traversal |
| `POST` | `/api/v2/graph/dfs` | Bearer (if configured) | Graph | Depth-first traversal |
| `POST` | `/api/v2/graph/degree` | Bearer (if configured) | Graph | Node degree |
| `POST` | `/api/v2/graph/centrality` | Bearer (if configured) | Graph | Centrality scores |
| `POST` | `/api/v2/graph/pagerank` | Bearer (if configured) | Graph | PageRank scores |
| `POST` | `/api/v2/graph/v2/bfs` | Bearer (if configured) | Graph | BFS (v2 engine) |
| `POST` | `/api/v2/graph/v2/dfs` | Bearer (if configured) | Graph | DFS (v2 engine) |
| `POST` | `/api/v2/graph/v2/degree` | Bearer (if configured) | Graph | Degree (v2 engine) |
| `POST` | `/api/v2/maintenance/purge` | Bearer (if configured) | Maintenance | Purge expired records |
| `POST` | `/api/v2/maintenance/compact` | Bearer (if configured) | Maintenance | Compact storage |
| `POST` | `/api/v2/maintenance/flush` | Bearer (if configured) | Maintenance | Flush pending writes |
| `POST` | `/api/v2/maintenance/rebuild-index` | Bearer (if configured) | Maintenance | Rebuild indexes |
| `GET` | `/api/v2/snapshots` | Bearer (if configured) | Maintenance | List snapshots |
| `POST` | `/api/v2/snapshots/{name}` | Bearer (if configured) | Maintenance | Create snapshot |
| `GET` | `/api/v2/threads` | Bearer (if configured) | Threads | List threads |
| `POST` | `/api/v2/threads` | Bearer (if configured) | Threads | Create thread |
| `GET` | `/api/v2/threads/{id}` | Bearer (if configured) | Threads | Get thread |
| `POST` | `/api/v2/threads/{id}` | Bearer (if configured) | Threads | Send message to thread |
| `DELETE` | `/api/v2/threads/{id}` | Bearer (if configured) | Threads | Delete thread |
| `POST` | `/api/v2/skills` | Bearer (if configured) | Skills | Create skill (idempotent by content hash) |
| `PUT` | `/api/v2/skills/{skill_id}` | Bearer (if configured) | Skills | Update skill (optimistic lock) |
| `PATCH` | `/api/v2/skills/{skill_id}` | Bearer (if configured) | Skills | Patch skill fields |
| `DELETE` | `/api/v2/skills/{skill_id}` | Bearer (if configured) | Skills | Delete skill and all versions |
| `GET` | `/dashboard` | Bearer (if configured) | Stable | Web console entry point |
| `GET` | `/dashboard/{path}` | Bearer (if configured) | Stable | Web console static assets |
| `POST` | `/conversation/add` ⚠️ experimental | Bearer (if configured) | Experimental | Legacy conversation turn ingestion |
| `GET` | `/skill/listing` ⚠️ experimental | Bearer (if configured) | Experimental | Skill-like record listing |

## Error responses

All error bodies share the shape `{ "success": false, "error": "<message>",
"hint"?: "<guidance>" }`:

| Status | Meaning |
|--------|---------|
| `400` | Invalid request (parse error, malformed query/body) |
| `401` | Missing or invalid Bearer token |
| `403` | Authenticated but insufficient RBAC permissions |
| `404` | Referenced resource (node, record, namespace, thread) not found |
| `409` | Operation not available (e.g. audit log not configured) |
| `429` | Rate limit exceeded (`Retry-After` header included) |
| `500` | Internal server error |

## CORS

CORS is **off by default**: the server sends no CORS headers unless origins are explicitly
configured. To allow specific origins to call the HTTP API from a browser, set:

- Env var: `VANTADB_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com`
- Config API: `VantaConfig::with_allowed_origins(vec![...])`

When configured, the server mounts a `tower_http::cors::CorsLayer` as the outermost layer
(so CORS preflight `OPTIONS` are answered before authentication), echoing
`Access-Control-Allow-Origin` for the configured origins only — never `AllowOrigin::any()`.

Decision history: recorded 2026-08-05 (TECH-06); feature implemented 2026-08-06
(AUDREP-14) via `VANTADB_ALLOWED_ORIGINS`.
