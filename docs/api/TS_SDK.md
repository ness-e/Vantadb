---
title: TypeScript SDK Documentation
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-07-22
aliases: []
---

# TypeScript SDK Documentation

> **Stability:** the documented TypeScript SDK API is covered by the [Versioning & Stability Policy](VERSIONING.md).

## Installation

```bash
npm install vantadb
```

> **Note:** Requires Node.js 18+, Bun, or Deno. The package bundles pre-compiled WASM — no native build step needed. Also works in browsers via ESM.

## Quick Start

```ts
import { VantaDB } from "vantadb";

// In-memory database (default)
const db = VantaDB.create();

// Store a record
db.put({
  namespace: "agent/main",
  key: "memory-1",
  payload: "The user prefers dark mode in all applications.",
  metadata: { theme: { type: "String", value: "dark" } },
  vector: [0.1, 0.2, 0.3],
});

// Hybrid search
const hits = db.search({
  namespace: "agent/main",
  query_vector: [0.1, 0.2, 0.3],
  text_query: "display mode",
  top_k: 10,
});

console.log(hits[0].record.payload);

// Generate a highlighted snippet
const snippet = db.generateSnippet(
  hits[0].record.payload,
  "display mode",
  true
);
console.log(snippet);

db.close();
```

## API Reference

### Connecting

#### `VantaDB.create(config?)`

```ts
static create(config?: VantaConfig): VantaDB
```

Create a new in-memory instance. Accepts an optional `VantaConfig` object. To use persistent storage, call `connect()` or `open()` instead.

**WASM note:** In WASM mode, `storage_path` in `VantaConfig` is ignored by `create()` (CODE-089) — the default factory opens an in-memory WASM engine. For persistent storage, use `connect()` / `open()` (Node on-disk), or in browsers `connect_persistent()` (OPFS) / `connect_idb()` (IndexedDB) / `connect_worker()`. A console warning is emitted when `storage_path` is supplied to `create()` but no persistent backend is selected.

#### `VantaDB.connect(path?)`

```ts
static connect(path?: string): VantaDB
```

If `path` is provided and not `":memory:"`, opens a persistent on-disk database. If `path` is empty, omitted, or `":memory:"`, opens an in-memory engine.

#### `VantaDB.open(path)`

```ts
static open(path: string): VantaDB
```

Always opens a persistent database at the given filesystem path. Prefer `connect()` for portability.

#### `connect_idb(path)`

```ts
static connect_idb(path: string): Promise<VantaDB>
```

Open a VantaDB instance with **IndexedDB-backed persistence** for browser environments. This is the recommended persistence backend when OPFS (the Origin Private File System) is unavailable, or when you need cross-tab coordination via `BroadcastChannel`.

**Availability:** `connect_idb` is exposed on the raw WASM constructor from `vantadb-wasm`. It is **not** re-exported by the high-level `vantadb` wrapper — access it directly via the WASM import.

**Parameters:**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | `string` | — | Storage path / database identifier used as the IndexedDB object-store key prefix |

**Returns:** `Promise<VantaDB>` — a fully loaded VantaDB instance with all previously saved records restored into memory.

**How it works:**
1. An inline JavaScript bridge (registered at wasm-bindgen time) connects to the `"VantaDB"` IndexedDB database with a `"state"` object store
2. On open, `load_idb()` reads the `db_state.json` key from IndexedDB and imports all stored records into memory
3. All mutations (`put`, `delete`, etc.) run in-memory — call `save_idb()` explicitly to persist changes to IndexedDB
4. Uses `web-locks` (`navigator.locks.request("vantadb-write")`) when available for multi-tab write coordination

**Full example:**

```ts
import { VantaDB as WasmVantaDB } from "vantadb-wasm";

// Open or create an IndexedDB-backed database
const db = await WasmVantaDB.connect_idb("./my_brain");

// Insert records (in-memory — not yet persisted)
db.put({
  namespace: "notes",
  key: "note-1",
  payload: "IndexedDB persists VantaDB state in the browser.",
  metadata: { source: { type: "String", value: "docs" } },
  vector: [0.1, 0.2, 0.3],
});

db.put({
  namespace: "notes",
  key: "note-2",
  payload: "Use save_idb() to write in-memory state to IndexedDB.",
  metadata: { source: { type: "String", value: "docs" } },
  vector: [0.4, 0.5, 0.6],
});

// Persist to IndexedDB
await db.save_idb();

// Hybrid search
const hits = db.search({
  namespace: "notes",
  query_vector: [0.1, 0.2, 0.3],
  text_query: "IndexedDB",
  top_k: 10,
});

console.log(hits[0].record.payload);

// Delete persisted state from IndexedDB
await db.delete_idb();

db.close();
```

**IndexedDB vs OPFS:**

| Feature | IndexedDB (`connect_idb`) | OPFS (`connect_persistent` / `connect_worker`) |
|---------|--------------------------|------------------------------------------------|
| Browser support | Universal (all modern browsers) | Requires Chromium-based browsers (Chrome, Edge, Opera) |
| Cross-tab sync | ✅ `BroadcastChannel("vantadb-sync")` — sends `data-changed` notifications. Use `IdbStorage.subscribe()` to listen | ❌ None — two tabs writing to the same OPFS storage **silently corrupt** each other's data |
| Write coordination | ✅ `navigator.locks.request("vantadb-write")` (Web Locks API) when available | ❌ No locking |
| Write atomicity | ✅ IDB transactions are atomic per `put` | ❌ `createWritable` + `write` + `close` is not atomic — crash between write and close leaves indeterminate state |
| Storage limits | Larger quota (~50% of disk), shared with other origin data | Same origin quota, shared with OPFS |
| Incremental append | ❌ Not supported — full-dump only | ✅ `OpfsFile::append()` with `{keepExistingData: true}` |

**When to use `connect_idb`:**
- You need cross-tab consistency (multiple tabs writing to the same VantaDB)
- You need broader browser support (Safari, Firefox, mobile browsers)
- OPFS is unavailable or you prefer a simpler persistence model
- You want write atomicity guarantees via IDB transactions

**When to use OPFS (`connect_persistent` / `connect_worker`):**
- Single-tab Chromium-based applications
- You need incremental append capability
- You are working with very large datasets where full-dump overhead matters

**Related methods:**

| Method | Description |
|--------|-------------|
| `save_idb()` | Persist all in-memory records to IndexedDB. Serializes all records to `db_state.json` in the IDB object store |
| `load_idb()` | Restore all records from IndexedDB into memory (called automatically on `connect_idb`) |
| `delete_idb()` | Delete all persisted state from IndexedDB (`db_state.json` key) |

**Multi-tab notes:**
- The `BroadcastChannel` is named `"vantadb-sync"`. On every write or delete, a `{type: "data-changed", key}` message is broadcast to all other tabs
- Use `IdbStorage.subscribe(callback)` to listen for cross-tab changes. The callback receives the changed key as a `JsValue`
- Web Locks API (`navigator.locks`) provides best-effort write coordination, but last-write-wins semantics apply — there is no merge or conflict detection

#### `close()`

```ts
close(): void
```

Close the database and release underlying WASM engine resources. After `close()`, all public methods throw an error. This is safer than relying on WASM GC/finalization to prevent use-after-free.

### Memory API (Namespace-Scoped)

#### `put()`

```ts
put(input: {
  namespace: string;
  key: string;
  payload: string;
  metadata?: Record<string, VantaValue>;
  vector?: number[];
  ttl_ms?: number;
}): MemoryRecord
```

Insert or update a memory record. The record is upserted — if a record with the same `(namespace, key)` already exists, it is replaced and the `version` counter is incremented.

#### `putBatch()`

```ts
putBatch(inputs: Array<{
  namespace: string;
  key: string;
  payload: string;
  metadata?: Record<string, VantaValue>;
  vector?: number[];
  ttl_ms?: number;
}>): MemoryRecord[]
```

Insert or update multiple records in parallel. Up to 5x faster than sequential `put()` calls.

#### `get()`

```ts
get(namespace: string, key: string): MemoryRecord | null
```

Retrieve a record by namespace and key. Returns `null` if not found or if the record has expired.

#### `delete()`

```ts
delete(namespace: string, key: string): boolean
```

Delete a record. Returns `true` if the record existed and was deleted, `false` if it did not exist.

#### `list()`

```ts
list(namespace: string, options?: ListOptions): MemoryListPage
```

List records in a namespace with optional metadata filters, limit, and cursor pagination.

```ts
interface ListOptions {
  filters?: Record<string, VantaValue>;  // equality filter on metadata
  limit?: number;                         // default: 100
  cursor?: number;                        // from previous page's next_cursor
}

interface MemoryListPage {
  records: MemoryRecord[];
  next_cursor?: number;
}
```

#### `listNamespaces()`

```ts
listNamespaces(): string[]
```

List all namespaces in the database.

### Search

#### `search()`

```ts
search(request: SearchRequest): SearchHit[]
```

Hybrid search combining vector similarity and BM25 text search with RRF fusion.

```ts
interface SearchRequest {
  namespace: string;
  query_vector: number[];
  filters?: Record<string, VantaValue>;
  text_query?: string;          // BM25 lexical search term
  top_k?: number;               // default: 10
  distance_metric?: "Cosine" | "Euclidean";  // default: "Cosine"
  explain?: boolean;            // include score breakdown
}
```

**Distance vs Score (CODE-091):** The `distance` field in `SearchHit` is a **L2 or cosine distance**, not a similarity score. Lower values indicate higher similarity. This differs from the Rust and Python SDKs which expose a `score` field where higher is better.

**Cross-binding map:** see [`WASM_API.md` → "Score vs distance semantics (WSM-10)"](WASM_API.md#score-vs-distance-semantics-wsm-10) for the full per-transport field map (Rust core / WASM binding / TS wrapper / Node / Python / HTTP). That section is the single source of truth for "which field carries which convention across which transport"; this subsection is the TS-side rationale only.

**Cross-SDK convention (TS-03):** the asymmetry between the TS SDK and the other bindings is intentional and pinned in CI. Each transport exposes a different field so consumers should pick the row that matches their SDK:

| SDK binding | Field on hit | Convention | Range |
|-------------|--------------|------------|-------|
| `vantadb-ts` (this SDK) | `SearchHit.distance` | **lower is more similar** (raw L2 / cosine distance) | `[0.0, +∞)` for cosine; `[0.0, +∞)` for Euclidean |
| `vantadb` (Rust core) | `VantaMemorySearchHit.score` | higher is better (cosine `1.0 - distance`; Euclidean `-distance²` then sqrt) | `[-1.0, 1.0]` cosine; `(-∞, 0.0]` Euclidean |
| `vantadb-python` | `hit.score` | higher is better | `[-1.0, 1.0]` cosine |
| `vantadb-node` | `{node_id, score}` | higher is better | `[-1.0, 1.0]` cosine |
| HTTP API (`POST /api/v2/search`) | `score` | higher is better | `[-1.0, 1.0]` cosine |

The score semantics are pinned by `src/sdk/serialization/vector_types.rs::tests` (TS-03 integration block: `score_roundtrips_through_serde_json`, `cosine_score_range_matches_documented_contract`, `euclidean_score_supports_negative_values`, `cosine_sim_f32_zero_norm_returns_finite_zero`, `euclidean_squared_distance_never_negative_under_fp_rounding`). A future change to the core formula will fail CI before reaching `develop`.

When porting TS code to another SDK, invert the comparison: `hits.sort((a, b) => a.distance - b.distance)` becomes `hits.sort((a, b) => b.score - a.score)`.

#### `searchVector()`

```ts
searchVector(vector: number[], topK?: number): { node_id: string; distance: number }[]
```

Pure HNSW vector search against the low-level node graph. Returns distance-ranked results where lower distance = more similar.

**WASM note:** `searchVector()` is the TypeScript wrapper name. It delegates to `search_vector()` on the WASM binding (`vantadb-wasm`), which in turn calls the Rust core `VantaDB::search_vector`.

#### `explainSearch()`

```ts
explainSearch(request: SearchRequest): any
```

Returns a detailed breakdown of how a search arrives at its results, including the planner route (hybrid / text-only / vector-only), per-hit score breakdown, and RRF fusion report.

### Graph API (Low-Level Node)

#### `insertNode()`

```ts
insertNode(
  id: number | bigint,
  content?: string,
  vector?: number[],
  fields?: Record<string, VantaValue>
): void
```

Insert a graph node with optional content, vector, and metadata fields.

**BigInt note (CODE-090):** For IDs > 2^53, pass a `bigint` — JavaScript Numbers lose integer precision above 2^53. Passing a non-safe-integer `number` throws.

#### `getNode()`

```ts
getNode(id: number): NodeRecord | null
```

Retrieve a node by numeric ID. Returns `null` if not found or tombstoned.

#### `deleteNode()`

```ts
deleteNode(id: number, reason?: string): void
```

Delete a node with an auditable reason. The node is tombstoned, not immediately removed from storage.

#### `addEdge()`

```ts
addEdge(source: number, target: number, label?: string, weight?: number): void
```

Add a directed edge from source to target with an optional label and weight.

#### `graphBfs()`

```ts
graphBfs(roots: number[], maxDepth?: number): bigint[]
```

Breadth-first traversal from one or more root nodes. Returns visited node IDs
in BFS order as `bigint` (because the underlying VantaDB graph uses `u128` node
IDs, which exceed `Number.MAX_SAFE_INTEGER`).

#### `graphDfs()`

```ts
graphDfs(roots: number[], maxDepth?: number): bigint[]
```

Depth-first traversal from one or more root nodes. Returns `bigint[]` of node IDs
in DFS order.

#### `graphTopologicalSort()`

```ts
graphTopologicalSort(roots: number[]): bigint[]
```

Topological sort of the subgraph reachable from the given roots. Returns
`bigint[]` in topological order.

#### `graphIsDag()`

```ts
graphIsDag(roots: number[]): boolean
```

Check whether the subgraph reachable from the given roots is a Directed Acyclic Graph.

### Maintenance

| Method | Description |
|--------|-------------|
| `flush()` | Flush WAL and HNSW to storage for durability |
| `compactWal()` | Archive and compact the WAL |
| `purgeExpired()` | Remove all TTL-expired records. Returns `bigint` count |
| `rebuildIndex()` | Rebuild ANN (HNSW), derived, and text indexes |
| `compactLayout()` | BFS-order physical compaction of vector store. Returns `bigint` |

### Export / Import

| Method | Description |
|--------|-------------|
| `exportNamespace(path, namespace)` | Export a namespace as JSONL |
| `exportAll(path)` | Export all namespaces as JSONL |
| `importRecords(records)` | Import records from a `MemoryRecord[]` array |
| `importFile(path)` | Import records from a JSONL file |

### Text Index Diagnostics

| Method | Description |
|--------|-------------|
| `auditTextIndex(namespace?)` | Structural audit of the text index for a namespace (or all) |
| `auditTextIndexDeep(namespace?)` | Deep structural audit — decodes TF, positions, DF, doc lengths |
| `repairTextIndex()` | Rebuild text index from canonical storage |

### Observability

| Method | Description |
|--------|-------------|
| `operationalMetrics()` | Snapshot of runtime metrics. Returns `OperationalMetrics` |
| `capabilities()` | Stable runtime capabilities. Returns `Capabilities` |
| `generateSnippet(payload, query, withHighlighting?)` | Highlighted text snippet |

### IQL Queries

```ts
query(query: string): QueryResult
```

Execute an IQL query string against the graph. Returns `QueryResult` which can be a `Read` (nodes), `Write` (affected count), or `StaleContext`.

## Types

### `VantaValue`

```ts
type VantaValue =
  | { type: "String"; value: string }
  | { type: "Int"; value: number }
  | { type: "Float"; value: number }
  | { type: "Bool"; value: boolean }
  | { type: "Null" }
  | { type: "ListString"; value: string[] }
  | { type: "ListInt"; value: number[] }
  | { type: "ListFloat"; value: number[] }
  | { type: "ListBool"; value: boolean[] };
```

### `MemoryRecord`

```ts
interface MemoryRecord {
  namespace: string;
  key: string;
  payload: string;
  metadata: Record<string, VantaValue>;
  created_at_ms: string;       // u64 as string
  updated_at_ms: string;       // u64 as string
  version: string;             // u64 as string
  node_id: string;             // u64 as string
  vector?: number[];
  expires_at_ms?: string;      // u64 as string, present if TTL was set
}
```

Numeric timestamp fields are serialized as strings to preserve u64 precision over JSON serialization.

### `SearchHit`

```ts
interface SearchHit {
  record: MemoryRecord;
  distance: number;       // L2/cosine distance — lower is more similar
  explanation?: SearchExplanationHit;
}
```

### `OperationalMetrics`

`OperationalMetrics` contains 37+ fields grouped by subsystem:

- **Startup:** `startup_ms`, `wal_replay_ms`, `wal_records_replayed`
- **ANN rebuild:** `ann_rebuild_ms`, `ann_rebuild_scanned_nodes`
- **Derived index:** `derived_rebuild_ms`, `derived_prefix_scans`, `derived_full_scan_fallbacks`
- **Text index:** `text_index_rebuild_ms`, `text_postings_written`, `text_index_repairs`, `text_lexical_queries`, `text_lexical_query_ms`, `text_candidates_scored`, `text_consistency_audits`, `text_consistency_audit_failures`
- **Hybrid:** `hybrid_query_ms`, `hybrid_candidates_fused`, `planner_hybrid_queries`, `planner_text_only_queries`, `planner_vector_only_queries`
- **Export/Import:** `records_exported`, `records_imported`, `import_errors`
- **Memory:** `process_rss_bytes`, `process_virtual_bytes`, `hnsw_nodes_count`, `hnsw_logical_bytes`, `mmap_resident_bytes`, `volatile_cache_entries`, `volatile_cache_cap_bytes`
- **Jemalloc:** `jemalloc_allocated_bytes`, `jemalloc_active_bytes`, `jemalloc_metadata_bytes`, `jemalloc_resident_bytes`, `jemalloc_mapped_bytes`, `jemalloc_retained_bytes`

All numeric fields are `string` (u64 serialized over WASM boundary).

### `Capabilities`

```ts
interface Capabilities {
  runtime_profile: string;   // "Enterprise" | "Performance" | "LowResource"
  persistence: boolean;
  vector_search: boolean;
  iql_queries: boolean;
  read_only: boolean;
}
```

### `VantaConfig`

```ts
interface VantaConfig {
  storage_path?: string;   // ignored in WASM (CODE-089)
  read_only?: boolean;
  rss_threshold?: number;
  memory_limit?: number;
}
```

## Error Handling

Every error thrown by the SDK is an instance of the `VantaError` class (see
[`vantadb-ts/src/errors.ts`](../../vantadb-ts/src/errors.ts)). All errors carry
a stable `code` from the 10-element `ERROR_CODES` contract — **branch on
`code`, never on `message` text**.

> **Canonical reference:** [`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md)
> documents the full code table, `is_retriable()` / `recovery_hint()`
> semantics, MCP `-320xx` mapping, and the upcoming `VANTADB_`-prefixed codes
> from `ERR-CORE-01`.

### `ERROR_CODES` (10 — contract surface)

```ts
import { ERROR_CODES, VantaError, wrapWasmError } from "vantadb";

const codes = ERROR_CODES;
// {
//   CLOSED: "VANTADB_CLOSED",
//   WASM_ERROR: "VANTADB_WASM_ERROR",
//   VALIDATION_ERROR: "VANTADB_VALIDATION_ERROR",
//   NOT_FOUND: "VANTADB_NOT_FOUND",
//   INVALID_ARGUMENT: "VANTADB_INVALID_ARGUMENT",
//   CORRUPT: "VANTADB_CORRUPT",
//   RESOURCE_LIMIT: "VANTADB_RESOURCE_LIMIT",
//   TIMEOUT: "VANTADB_TIMEOUT",
//   BUSY: "VANTADB_BUSY",
//   IO_ERROR: "VANTADB_IO_ERROR",
// }
```

| Code (wire value) | Meaning | Source Rust variant(s) | Retriable |
|------|---------|-------------------------|:---------:|
| `VANTADB_VALIDATION_ERROR` | Input failed validation | `DimensionMismatch`, `DuplicateNode`, `ValidationError`, `InvalidInput`, `IqlParseError`, `UnsupportedOperation`, `NoVectorForKey`, `NodeIdCollision`, `CycleDetected`, `ExecutionConflict` | ❌ |
| `VANTADB_NOT_FOUND` | Requested entity does not exist | `NodeNotFound`, `NotFound` | ❌ |
| `VANTADB_TIMEOUT` | Operation exceeded its time budget | `Timeout` | ✅ |
| `VANTADB_BUSY` | Resource locked or not initialized | `DatabaseBusy`, `NotInitialized` | ✅ |
| `VANTADB_RESOURCE_LIMIT` | Memory / disk / backpressure limit exceeded | `ResourceLimit` | ✅ |
| `VANTADB_CORRUPT` | Persisted data is corrupt or incompatible format | `WALVersionMismatch`, `IncompatibleFormat`, `SerializationError`, `SchemaError`, `RestoreError`, `BackupError` | ❌ |
| `VANTADB_INVALID_ARGUMENT` | Caller passed a malformed argument | `IqlError` | ❌ |
| `VANTADB_IO_ERROR` | Filesystem or backend I/O failure | `IoError`, `WalError`, `BackendError`, `CliError`, `SearchError`, `RuntimeError` | ✅ |
| `VANTADB_WASM_ERROR` | Generic WASM-binding fallback | `Generic` (only when no `code` is attached) | ❌ |
| `VANTADB_CLOSED` | Operation on a closed database handle | (lifecycle, not in `VantaError`) | ❌ |

> **Resolved (`ERR-TS-01`):** the `VANTADB_` prefix from `VantaError::code()`
> is now live on the TS/WASM/Node wire — the values above are the contract.
> TS keeps unprefixed *keys* (`ERROR_CODES.BUSY === "VANTADB_BUSY"`).
> BREAKING for code that compared `err.code` against the unprefixed strings.

### `VantaError` class shape

```ts
import { VantaError } from "vantadb";

export class VantaError extends Error {
  readonly code: string;        // one of the 10 VANTADB_* codes above
  readonly details?: unknown;   // structured payload (Rust variant fields)
  readonly timestamp: Date;
  // readonly cause?: unknown;  // ES2022 ErrorOptions — set by wrapWasmError/wrapNativeError

  toJSON(): {
    name: string;
    code: string;
    message: string;
    details?: unknown;
    timestamp: string;          // ISO-8601
  };
}
```

### `wrapWasmError` — boundary classification

The WASM binding preserves the structured `code` (via `vantadb-wasm`'s
`to_js_err`). When that is missing (older pkg builds), `wrapWasmError` falls
back to `classifyWasmError`, which uses message-prefix regex mirroring the
`Display` strings in `src/error.rs`.

```ts
import { VantaError, wrapWasmError } from "vantadb";

try {
  db.put({ namespace: "ns", key: "k", payload: "hello" });
} catch (err) {
  const vantaErr = wrapWasmError(err, "db.put");
  switch (vantaErr.code) {
    case "VANTADB_VALIDATION_ERROR":
      console.warn("validation failed:", vantaErr.details);
      break;
    case "VANTADB_BUSY":                 // is_retriable
      await sleep(100);
      return retry();
    case "VANTADB_NOT_FOUND":
      throw new Error("resource missing");
    default:
      throw vantaErr;
  }
}
```

### Cause chain (TS 4.4+)

> Since `ERR-TS-01`, `wrapWasmError`/`wrapNativeError` set
> `VantaError.cause` to the original thrown value (ES2022 `ErrorOptions`),
> while `details` keeps its legacy shape (`{name, stack}` or
> `{original}`) for backward compatibility:

```ts
try {
  await db.put(record);
} catch (err) {
  if (err instanceof VantaError && err.cause instanceof Error) {
    console.error("root cause:", err.cause.message);
  }
}
```

### Lifecycle errors (closed handle)

Calling any method after `close()` throws a `VantaError` with `code: "VANTADB_CLOSED"`.
This is safer than relying on WASM GC/finalization to prevent use-after-free:

```ts
db.close();
try {
  db.get("ns", "k");
} catch (err) {
  if (err instanceof VantaError && err.code === "VANTADB_CLOSED") {
    console.warn("db was closed");
  }
}
```

## WASM vs Node.js Differences

| Feature | WASM (browser / Bun / Deno) | Node.js with `connect()` |
|---------|-----------------------------|---------------------------|
| Persistence | In-memory by default (`storage_path` ignored, CODE-089). Use `connect_idb()` for IndexedDB persistence in browsers, or `connect_persistent()` / `connect_worker()` for OPFS in Chromium | On-disk at given path |
| Threading | Single-threaded | Multi-threaded (Tokio) |
| File I/O | Limited (export/import works via JS APIs) | Full filesystem access |
| Memory | WebAssembly heap (limited) | Native heap |

## Runtimes

| Runtime | Status |
|---------|--------|
| Node.js 18+ | ✅ |
| Bun | ✅ |
| Deno | ✅ |
| Browser (ESM) | ✅ |

## Data Types (Subpath Import)

```ts
import type { VantaConfig, SearchHit, OperationalMetrics } from "vantadb/types";
```
