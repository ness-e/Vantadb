---
title: "ADR-036: MCP-35 HTTP Fallback Discovery for Multi-Instance MCP on Same Database"
type: adr
status: proposed
tags: [vantadb, architecture, adr, mcp, multi-instance, discovery, proxy]
created: 2026-09-02
last_reviewed: 2026-09-02
---

# ADR-036: MCP-35 HTTP Fallback Discovery for Multi-Instance MCP on Same Database

## Context

**Problem:** Currently, `vanta-cli server --mcp --db <path>` spawns `vantadb-server --mcp` which opens the `StorageEngine` directly. The engine uses an exclusive file lock (`.vanta.lock`) to enforce single-writer semantics. When a second MCP instance tries to open the same database, it fails with `VantaError::DatabaseBusy` and exits with code 1. This prevents multiple OpenCode sessions (or any MCP clients) from sharing the same database concurrently.

**Incident:** 2026-08-25 — Two OpenCode sessions on the same DB; the second session had no tools available because it couldn't acquire the lock.

**Requirements (from MCP-35 backlog):**
1. First instance writes discovery file `.vanta.server.json` in the data directory with `{pid, http_port}` and opens HTTP listener
2. Subsequent instances detect "Database busy", read discovery file, verify PID is alive via `/health`, and start in **proxy mode** (expose same MCP tools but resolve calls via HTTP `/api/v2/*` against lock-owning server)
3. If discovery file points to dead PID → cleanup lock and open embedded normally
4. Contract: 2+ simultaneous OpenCode sessions share memory; owner crash doesn't corrupt; tools parity 1:1 with embedded mode

## Decision

### Discovery File Format

**Location:** `<data_dir>/.vanta.server.json` (where `<data_dir>` = `<db_path>/data`)

**Schema:**
```json
{
  "version": 1,
  "pid": 12345,
  "http_port": 8080,
  "started_at_ms": 1725234567890,
  "mode": "owner"
}
```

Fields:
- `version`: Schema version (starts at 1)
- `pid`: Process ID of the lock-owning server
- `http_port`: HTTP port the owner server is listening on
- `started_at_ms`: Unix milliseconds when owner started (for stale detection)
- `mode`: `"owner"` | `"proxy"` — indicates this instance's role

### Lock Detection & Fallback Flow

```
Instance starts (vantadb-server --mcp)
    │
    ▼
Try StorageEngine::open_with_config()
    │
    ├─► Success → First instance (OWNER)
    │     │
    │     ├─► Pick free HTTP port (config.port or ephemeral)
    │     ├─► Write discovery file atomically (temp + rename)
    │     ├─► Start HTTP server on picked port (reuse existing /api/v2/* routes)
    │     ├─► Run MCP stdio server (existing run_stdio_server)
    │     └─► On shutdown: delete discovery file, flush, exit
    │
    └─► Err(VantaError::DatabaseBusy) → Subsequent instance (PROXY CANDIDATE)
          │
          ├─► Read discovery file
          │     │
          │     ├─► File missing/corrupt → Retry open (race window) or fail
          │     ├─► PID not alive (kill 0 / /health fails) → 
          │     │     Delete stale lock file (.vanta.lock), delete discovery file
          │     │     Retry StorageEngine::open_with_config() as OWNER
          │     │
          │     └─► PID alive & /health OK → PROXY MODE
          │           │
          │           ├─► Write discovery file with mode="proxy" (for visibility)
          │           ├─► Start HTTP client pointing to owner's http_port
          │           ├─► Run MCP stdio server with proxy dispatcher
          │           └─► On shutdown: delete proxy discovery entry, exit
          │
          └─► Health check: GET http://127.0.0.1:{http_port}/health
                - 200 + status="healthy" → owner confirmed alive
                - timeout/connection refused/non-200 → owner dead
```

### HTTP Proxy Mode for MCP Tools

**Architecture:** The proxy instance runs the same `run_stdio_server` loop but with a **proxy dispatcher** instead of the embedded `handle_tools_call`.

**Proxy Dispatcher:**
- Receives `tools/call` JSON-RPC requests over stdio
- Translates to HTTP POST `http://127.0.0.1:{owner_port}/api/v2/<endpoint>`
- Maps MCP tool names to REST endpoints (see mapping table below)
- Forwards auth token (Bearer) from MCP client if present
- Returns HTTP response as MCP tool result

**Tool → REST Endpoint Mapping:**

| MCP Tool | REST Endpoint | Method |
|----------|---------------|--------|
| `memory_put` | `/api/v2/records` | POST |
| `memory_put_batch` | `/api/v2/records/batch` | POST |
| `memory_get` | `/api/v2/records/{ns}/{key}` | GET |
| `memory_delete` | `/api/v2/records/{ns}/{key}` | DELETE |
| `memory_delete_by_filter` | `/api/v2/records` | DELETE (with query) |
| `memory_list` | `/api/v2/list` | GET |
| `memory_list_namespaces` | `/api/v2/list?namespace=` (all) | GET |
| `memory_versions` | `/api/v2/records/{ns}/{key}/versions` | GET |
| `memory_supersede` | (via IQL) `/api/v2/query` | POST |
| `search_memory` / `memory_search` | `/api/v2/search` | POST |
| `search_semantic` | `/api/v2/search` (vector only) | POST |
| `search_with_method` | `/api/v2/search` (with method) | POST |
| `search_multi` | `/api/v2/search` (multi-ns) | POST |
| `query_iql` | `/api/v2/query` | POST |
| `get_node_neighbors` | `/api/v2/graph/v2/bfs` (depth 1) | POST |
| `graph_page_rank` | `/api/v2/graph/pagerank` | POST |
| `graph_degree_centrality` | `/api/v2/graph/degree` | POST |
| `graph_traverse` | `/api/v2/graph/v2/bfs|dfs` | POST |
| `graph_topological_sort` | `/api/v2/graph/v2/toposort` | POST |
| `graph_is_dag` | `/api/v2/graph/v2/is-dag` | POST |
| `remove_edge` | (via IQL) `/api/v2/query` | POST |
| `inject_context` | `/api/v2/threads/{id}` (send) | POST |
| `read_axioms` | (via IQL) `/api/v2/query` | POST |
| `write_axiom` | (via IQL) `/api/v2/query` | POST |
| `delete_axiom` | (via IQL) `/api/v2/query` | POST |
| `collection_stats` | `/api/v2/metrics` + namespace filter | GET |
| `collection_list` | `/api/v2/list?namespace=` (all) | GET |
| `collection_delete` | (via IQL) `/api/v2/query` | POST |
| `rehydrate` | (via IQL) `/api/v2/query` | POST |
| `purge_expired` | `/api/v2/maintenance/purge` | POST |
| `compact_wal` | `/api/v2/maintenance/compact` | POST |
| `flush` | `/api/v2/maintenance/flush` | POST |
| `compact_layout` | (via IQL) `/api/v2/query` | POST |
| `vacuum` | (via IQL) `/api/v2/query` | POST |
| `rebuild_index` | `/api/v2/maintenance/rebuild-index` | POST |
| `audit_text_index` | (via IQL) `/api/v2/query` | POST |
| `repair_text_index` | (via IQL) `/api/v2/query` | POST |
| `capabilities` | `/api/v2/health` (extended) | GET |
| `generate_snippet` | (local, no DB access) | — |
| `list_snapshots` | `/api/v2/snapshots` | GET |
| `snapshot_create` | `/api/v2/snapshots/{name}` | POST |
| `snapshot_restore` | (via IQL) `/api/v2/query` | POST |
| `export` | `/api/v2/export` | POST |
| `import` | `/api/v2/import` | POST |
| `bulk_import_file` | (local filesystem) | — |
| `bulk_import_stream` | `/api/v2/import` (streaming) | POST |
| `embed_texts` | (local ONNX) | — |

**Tools requiring local execution (no HTTP proxy):**
- `generate_snippet` — pure text processing
- `embed_texts` — local ONNX embeddings (if available)
- `bulk_import_file` — host filesystem access
- `capabilities` — can be served from proxy's cached config

**Auth Token Propagation:** The MCP client may send an auth token via the MCP protocol (not standard). The proxy forwards `Authorization: Bearer <token>` header to owner's HTTP endpoints. If no token, proxy uses its own configured token (from env).

### Owner Crash Detection & Cleanup

**Detection:** Proxy instances periodically (every 10s) check owner health via `/health`. If 3 consecutive failures → assume owner dead.

**Cleanup Protocol:**
1. Proxy detects owner dead
2. Proxy acquires exclusive lock on `.vanta.lock` (using `fs2::FileExt::try_lock_exclusive`)
3. If lock acquired → proxy becomes new owner:
   - Deletes stale discovery file
   - Writes new discovery file with its PID/port
   - Starts HTTP server
   - Transitions from proxy → owner mode (hot swap dispatcher)
4. If lock not acquired → another instance won; continue as proxy to new owner

**Owner Graceful Shutdown:**
- On SIGINT/SIGTERM: owner deletes discovery file, flushes storage, releases lock, exits
- Proxy instances detect missing discovery file → retry lock acquisition → one becomes new owner

### Port Selection

- Owner: Uses `config.port` (default 8080) or `port` CLI arg. If port in use, bind to port 0 (OS assigns) and record actual port in discovery file.
- Proxy: Does not bind HTTP server (unless promoted). Connects to owner's recorded port.

## Consequences

### Pros
- **Zero-downtime multi-session:** Multiple OpenCode sessions share same DB seamlessly
- **Tools parity 1:1:** Proxy exposes identical MCP tool surface; calls route to same REST endpoints
- **No data corruption:** Single-writer lock preserved; only one process holds `.vanta.lock` at a time
- **Graceful failover:** Owner crash → proxy promotes automatically within ~30s
- **Reuses existing HTTP surface:** `/api/v2/*` endpoints already cover all MCP tools
- **Minimal new code:** Proxy dispatcher ~200 lines; discovery logic ~100 lines

### Cons
- **Added latency:** Proxy mode adds one HTTP hop per tool call (~1-5ms local)
- **Complexity:** Two code paths (owner vs proxy) for MCP dispatch
- **Port management:** Ephemeral port allocation on conflict; discovery file must be atomic
- **Split-brain risk:** If health check flakes, two owners could theoretically coexist (mitigated by lock file as source of truth)

### Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Discovery file race on startup | Atomic write (temp + rename); retry open on conflict |
| Stale discovery file after crash | PID + /health verification; lock file is source of truth |
| Proxy doesn't see owner shutdown | Periodic health checks (10s); 3-strikes rule |
| Auth token mismatch | Proxy forwards client token; owner validates per existing RBAC |
| Tool parity gaps | Exhaustive mapping table; integration test verifies all 76 tools |

## Implementation Phases

### Phase 1: Discovery & Lock (vanta-arch DISCOVERY)
- Discovery file schema & atomic write
- Lock detection in `StorageEngine::open_with_config` error path
- Health check endpoint already exists (`/health`)
- Port selection logic

### Phase 2: Proxy Dispatcher (vanta-worker)
- HTTP client in `vantadb-mcp` crate
- Tool → REST endpoint mapping
- Proxy `handle_tools_call_proxy` dispatcher
- MCP server mode flag (owner vs proxy)

### Phase 3: Failover & Cleanup
- Periodic health check in proxy
- Lock acquisition on owner death
- Hot-swap proxy → owner transition
- Graceful shutdown cleanup

### Phase 4: Integration Tests
- 2+ simultaneous `vanta-cli server --mcp --db <same>` 
- Tools parity verification (all 76 tools)
- Owner kill → proxy promotion test
- `cargo check -p vantadb --tests` passes

## Alternatives Considered

### Alternative 1: Shared Memory / Unix Socket
- Use shared memory or Unix domain socket for IPC between instances
- **Rejected:** Windows compatibility; adds platform-specific complexity; HTTP already works cross-platform

### Alternative 2: Read-Only Replicas
- Subsequent instances open DB in read-only mode
- **Rejected:** MCP tools include writes (`memory_put`, `delete`, etc.); read-only doesn't satisfy tools parity

### Alternative 3: External Lock Service (etcd/Consul)
- Use distributed lock service
- **Rejected:** Overkill for local embedded DB; adds operational dependency

### Alternative 4: Single Process, Multiple Threads
- One `vantadb-server` handles multiple MCP stdio connections
- **Rejected:** MCP stdio protocol assumes one client per process; clients (OpenCode) spawn separate processes

## Verification

- [ ] ADR reviewed by vanta-lead
- [ ] Discovery file atomic write tested under concurrent startup
- [ ] Proxy dispatcher covers all 76 MCP tools (mapping table complete)
- [ ] Owner crash simulation test passes (kill -9 owner, proxy promotes)
- [ ] `cargo check -p vantadb --tests` exit 0
- [ ] 2× `vanta-cli server --mcp --db <same>` simultaneous → both respond to `tools/list`

## Related ADRs
- ADR-026: Vanta Studio Fase 3 REST Dashboard (established `/api/v2/*` surface)
- ADR-020: Storage Backend Default (Fjall with file locking)
- ADR-014: PITR (point-in-time recovery context for crash safety)