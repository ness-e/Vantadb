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

**WASM note:** In WASM mode, `storage_path` in `VantaConfig` is ignored (CODE-089) — the WASM backend always uses an in-memory engine. The parameter is accepted without error, but a warning is emitted to the console.

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

Methods throw an `Error` with a descriptive message on failure:

- Calling any method after `close()` throws `"VantaDB instance is closed"`.
- `insertNode()` with a non-safe-integer `number` throws an explicit precision warning.
- WASM-level errors (node not found, dimension mismatch, I/O errors) propagate as `Error` with the Rust error message.

```ts
try {
  db.put({ namespace: "ns", key: "k", payload: "hello" });
} catch (err) {
  console.error("VantaDB error:", err.message);
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
