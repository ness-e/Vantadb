---
title: Node.js Native SDK Documentation
type: api
status: active
tags: [vantadb, api, node]
last_reviewed: 2026-09-07
aliases: []
---

# Node.js Native SDK Documentation

> **Stability:** the documented Node.js SDK API is covered by the [Versioning & Stability Policy](VERSIONING.md).

`vantadb-node` is the **native** Node.js binding for VantaDB, built with
[napi-rs](https://napi.rs). It is an additional backend to the WASM build
([`vantadb`](TS_SDK.md)): the native `.node` module gives Node.js real
filesystem persistence (fjall/WAL/fsync) that WASM cannot provide, plus
multi-threaded execution via Tokio's blocking pool.

## Native vs WASM — which binding to use

|  | `vantadb-node` (native, this doc) | `vantadb-ts` (WASM) |
|---|---|---|
| Node.js ≥ 18 (backend, CLIs, agents) | ✅ **recommended** — real filesystem persistence (fjall/WAL/fsync) | ✅ works (in-memory in Node; no OPFS outside browsers) |
| Bun / Deno | ⚠️ loads via Node compat — prefer WASM there | ✅ preferred path |
| Browser / edge runtimes | ❌ native `.node` cannot load | ✅ only path |
| Persistence | ✅ disk (fjall) + `":memory:"` opt-in | ⚠️ OPFS/IDB in browsers, memory-only in Node |
| Performance numbers | harness only (`npm run bench`, PERF-BENCH-01) — claims published by release lead | citable bench in [BENCHMARKS.md §15](../user/operations/BENCHMARKS.md#15-jswasm-bench-vantadb-ts-ts-09--insert--search-p50p95p99) (TS-09) |

> **Fairness caveat (by design):** native runs persistent fjall storage
> (fsync included); WASM in Node is in-memory only. Native pays persistence
> costs WASM never sees — compare harnesses, not headlines.

## Installation

```bash
npm install vantadb-node
```

> **Note:** requires Node.js 18+. Pre-built binaries are shipped for
> Windows (x64), Linux (x64/arm64), and macOS (x64/arm64). Building from
> source requires a Rust toolchain with the napi CLI:
> `npm run build` (uses `napi build`).

## Quick Start

```js
import { VantaDb } from "vantadb-node";

// Persistent database on disk (":memory:" gives a non-persistent engine).
const db = await VantaDb.connect("./vanta_data");

await db.put({
  namespace: "agent/main",
  key: "memory-1",
  payload: "The user prefers dark mode in all applications.",
  vector: [0.1, 0.4, 0.8],
});

// Hybrid search (vector + text, RRF fusion)
const hits = await db.search({
  namespace: "agent/main",
  text_query: "display mode",
  query_vector: [0.1, 0.4, 0.8],
  top_k: 5,
});
console.log(hits[0]?.record.payload);

// Explain the search plan without executing it
const plan = await db.explainSearch({
  namespace: "agent/main",
  text_query: "display mode",
  query_vector: [0.1, 0.4, 0.8],
});

await db.close(); // drains in-flight ops, flushes, then closes
```

## Runtimes

All methods are **async** and return Promises — the engine runs on Tokio's
blocking pool, so the JS event loop is never blocked.

| Runtime | Support | Notes |
|---------|---------|-------|
| Node.js ≥ 18 | ✅ | ESM (`import`) and CJS (`require`) |
| Bun | ⚠️ | Loads via Node compat (`bun add` + `import`); no prebuilt Bun target — prefer the WASM SDK (`vantadb`) for Bun-first projects |
| Deno | ⚠️ | Requires `--allow-read --allow-write --allow-ffi` and Node compat; prefer the WASM SDK (`vantadb`) for Deno-first projects |

### CommonJS

```js
const { VantaDb } = require("vantadb-node");

async function main() {
  const db = await VantaDb.connect(":memory:");
  await db.put({ namespace: "ns", key: "k", payload: "hello" });
  console.log(await db.get("ns", "k"));
  await db.close();
}
main();
```

### TypeScript

```ts
import { VantaDb, type MemoryInput, type SearchRequest } from "vantadb-node";

const db = await VantaDb.connect("./data");
const record: MemoryInput = { namespace: "ts", key: "1", payload: "typed", vector: [1, 0] };
await db.put(record);
const req: SearchRequest = { namespace: "ts", query_vector: [1, 0], top_k: 3 };
const hits = await db.search(req);
await db.close();
```

### Bun (via Node compat)

```ts
// bun add vantadb-node
import { VantaDb } from "vantadb-node";

const db = await VantaDb.connect(":memory:");
await db.put({ namespace: "ns", key: "k", payload: "hello from bun" });
console.log((await db.get("ns", "k"))?.payload);
await db.close();
```

> Bun-first project? Prefer the WASM SDK (`vantadb`, see
> [TS_SDK.md](TS_SDK.md)) — no prebuilt Bun target is shipped.

### Deno (via Node compat, permissions required)

```ts
// deno add npm:vantadb-node
// deno run --allow-read --allow-write --allow-ffi main.ts
import { VantaDb } from "npm:vantadb-node";

const db = await VantaDb.connect(":memory:");
await db.put({ namespace: "ns", key: "k", payload: "hello from deno" });
console.log((await db.get("ns", "k"))?.payload);
await db.close();
```

> Deno-first project? Prefer the WASM SDK (`vantadb`, see
> [TS_SDK.md](TS_SDK.md)).

## API Reference

Full typed surface: `index.d.ts` (auto-generated at build time by napi-rs — see [`vantadb-node/`](../../vantadb-node/) when built). Summary:

| Method | Description |
|--------|-------------|
| `VantaDb.connect(path, options?)` | Open/create a database. `path` = directory (persistent) or `":memory:"`. `options: { read_only?, memory_limit? }` |
| `flush()` | Flush the WAL and memory-mapped files to disk |
| `close()` | Drain in-flight ops, flush, close. New ops rejected with `database is closing` |
| `put(record)` | Insert/update one memory record → `MemoryRecord` |
| `putBatch(records)` | Insert/update many records atomically → `MemoryRecord[]` |
| `get(namespace, key)` | Fetch a record, or `null` |
| `delete(namespace, key)` | Delete a record → `boolean` (true if deleted) |
| `list(namespace, options?)` | Paginate records with metadata filters → `{ records, next_cursor? }` |
| `listNamespaces()` | All namespaces holding records → `string[]` |
| `search(request)` | Hybrid/vector search → `MemorySearchHit[]` |
| `explainSearch(request)` | Search plan + per-hit scoring breakdown, without executing |
| `capabilities()` | Stable runtime capabilities (sync) |
| `insertNode(input)` | Insert/update a graph node (`id` as decimal string) |
| `getNode(id)` | Fetch a graph node, or `null` |
| `deleteNode(id, reason)` | Delete a graph node (reason recorded for auditing) |
| `addEdge(sourceId, targetId, label, weight?, createdAtMs?)` | Add a directed edge (+ reverse edge) |
| `removeEdge(sourceId, targetId, label)` | Remove all edges with label between two nodes |
| `graphBfs(roots, maxDepth, direction)` | Breadth-first traversal (`Forward`/`Reverse`/`Both`) |
| `graphDfs(roots, maxDepth, direction)` | Depth-first traversal |
| `graphTopologicalSort(roots)` | Topological order; errors on cycles |
| `graphIsDag(roots)` | Whether the reachable subgraph is a DAG |
| `graphFilteredTraversal(roots, maxDepth, direction, filter?)` | BFS with `{ labels?, time_range? }` edge filter |
| `graphDegree(roots)` | Degree centrality entries `{ id, in_degree, out_degree }` |
| `versions(ns, key)` | Every retained version of a record, ascending (v1..vN) |
| `getVersion(ns, key, version)` | One historical version, or `null` |
| `supersede(ns, oldKey, newKey)` | Mark `oldKey` superseded by `newKey` (ADR-028) |
| `vacuum()` | Purge HNSW tombstones → `VacuumReport` |
| `rebuildIndex()` | Rebuild vector/derived/text/scalar indexes → `RebuildReport` |
| `compactLayout()` | Compact vector store file → estimated bytes reclaimed (`bigint`) |
| `compactWal()` | Flush, archive WAL, start fresh |
| `purgeExpired()` | Delete TTL-expired records → count purged (`bigint`) |
| `deleteByFilter(ns, filter)` | Delete by metadata filter (≥1 item) → count deleted (`bigint`) |
| `count(ns, filter?)` | Count records, optionally filtered (`bigint`; `null` = all) |
| `similarToKey(ns, key, topK)` | Records similar to an existing record's vector (source excluded) |
| `searchWithMethod(req, method?)` | `search()` with explicit backend (`Hnsw`/`Ivf`/`Flat`/`DiskAnn`/`Scann`, `null` = auto) |
| `searchMulti(namespaces, req)` | Search several namespaces at once, merged by descending score |

> **⚠️ `bigint` runtime truth (FIND-BND12-01):** `count()`, `purgeExpired()`,
> `compactLayout()`, and `deleteByFilter()` return **`bigint`** at runtime
> (napi-rs maps Rust `u64` → BigInt — verified by `tests/api.test.ts`,
> BND-12). `index.d.ts` still declares `Promise<number>` for these — the
> declaration is stale; **trust the runtime**. Compare with `0n`, or wrap
> with `Number(...)` when you need a number:
>
> ```js
> const n = await db.count("docs", null);
> console.log(n === 0n ? "empty" : `${n} records`); // bigint comparison
> const purged = Number(await db.purgeExpired());   // safe for small counts
> ```

### Memory lifecycle

```js
const db = await VantaDb.connect(":memory:");
const record = await db.put({
  namespace: "lifecycle",
  key: "draft",
  payload: "version one",
  metadata: { status: { String: "draft" } },
  ttl_ms: 60_000,            // relative TTL, non-negative integer
});
// record: { namespace, key, payload, metadata, version: 1, node_id: "...", ... }

const same = await db.put({ namespace: "lifecycle", key: "draft", payload: "version two" });
// same.version === 2  (upsert bumps the version)

const found = await db.get("lifecycle", "draft");
const page = await db.list("lifecycle", { limit: 10, filters: { status: { String: "draft" } } });
const removed = await db.delete("lifecycle", "draft"); // true
await db.close();
```

> **Note:** `metadata` uses the tagged `VantaValue` form, e.g.
> `{ tag: { String: "keep" } }`, `{ count: { Int: 3 } }`. `node_id` is a
> decimal **string** — u128 ids exceed `Number.MAX_SAFE_INTEGER`.

### Search

```js
const hits = await db.search({
  namespace: "docs",
  query_vector: [0.9, 0.1],
  text_query: "rust programming", // optional: enables hybrid RRF fusion
  top_k: 10,
  distance_metric: "Cosine",     // or "Euclidean"
  filters: { lang: { String: "en" } },
});
// hit: { record: MemoryRecord, score: number, explanation?: SearchExplanationHit }
```

**Score is relevance, not a distance (WSM-10):** the `score` field is
**higher-is-better** — it is a relevance score (BM25 for text, cosine
similarity ∈ [-1.0, 1.0] for vectors, RRF-fused for hybrid). It is **not**
a raw L2 / cosine distance. This matches the Rust core and the Python SDK.
**It is intentionally different from the TypeScript wrapper `vantadb-ts`**
(which renames the field to `distance` and inverts the semantics —
"lower is more similar", CODE-091).

The full per-transport field map (Rust core / WASM binding / TS wrapper /
Node / Python / HTTP API) lives in
[`WASM_API.md` → "Score vs distance semantics (WSM-10)"](WASM_API.md#score-vs-distance-semantics-wsm-10).
The Node-side rationale (the formula and the range per `distance_metric`)
is documented inline in the Rust source
[`vantadb-node/src/lib.rs` `search()` docstring](../../vantadb-node/src/lib.rs).

`explainSearch(request)` returns `{ route, hits, fusion_report }` where each
hit carries `matched_tokens`, `matched_phrases`, and per-term BM25
contributions (`bm25_terms`), plus RRF ranks when hybrid fusion ran.

### Graph

```js
const db = await VantaDb.connect(":memory:");
await db.insertNode({ id: "42", content: "ada", fields: { name: { String: "Ada" } } });
await db.insertNode({ id: "43", content: "babbage" });
await db.addEdge("42", "43", "mentored");

const node = await db.getNode("42"); // { id: "42", fields: {…}, edges: [{ target: "43", label: "mentored", … }], … }
const visited = await db.graphBfs(["42"], 3, "Forward");
await db.removeEdge("42", "43", "mentored");
await db.deleteNode("42", "cleanup");
await db.close();
```

### Record lifecycle (versions & supersede)

```js
await db.put({ namespace: "lifecycle", key: "v1", payload: "first" });
await db.put({ namespace: "lifecycle", key: "v2", payload: "second" });
await db.supersede("lifecycle", "v1", "v2"); // both keys must exist and differ

const history = await db.versions("lifecycle", "v1"); // v1..vN ascending
const first = await db.getVersion("lifecycle", "v1", 1); // { … version: 1 }
const live = await db.get("lifecycle", "v1"); // live record carries superseded_by: "v2"
```

### Advanced search

```js
// Metadata-filtered count + delete (FilterItem: { field, op, value })
const n = await db.count("docs", [{ field: "lang", op: "Eq", value: { String: "en" } }]);
const deleted = await db.deleteByFilter("docs", [{ field: "lang", op: "Eq", value: { String: "en" } }]);
// op is one of: Eq | Neq | Gt | Lt | Gte | Lte (value uses the tagged VantaValue form)

// "More like this" from an existing record's vector
const similar = await db.similarToKey("docs", "seed-key", 5);

// Pin the dense-vector backend (null/undefined = automatic routing)
const flat = await db.searchWithMethod({ namespace: "docs", query_vector: [1, 0] }, "Flat");

// One call across namespaces (request.namespace is ignored)
const merged = await db.searchMulti(["docs", "notes"], { namespace: "", query_vector: [1, 0], top_k: 5 });
```

### Maintenance

```js
const vacuum = await db.vacuum();
// { scanned_nodes, removed_nodes, reclaimed_bytes, duration_ms, success }

const rebuild = await db.rebuildIndex();
// { scanned_nodes, indexed_vectors, skipped_tombstones, duration_ms, derived_rebuild_ms, index_path, success }

const reclaimed = await db.compactLayout(); // bigint: estimated bytes reclaimed
await db.compactWal();                      // flush + archive WAL + fresh start
const purged = await db.purgeExpired();     // bigint: TTL-expired records removed
```

## Error Handling

Rejected promises carry descriptive messages from the Rust engine, e.g.:

- `missing required field 'key'` — malformed input object
- `'query_vector' exceeds max vector dimension 10000` — FFI dimension cap
- `database is closing` — operation after `close()` began
- `invalid direction 'Sideways': expected 'Forward', 'Reverse', or 'Both'`

Engine failures are prefixed with the stable machine-readable code
(`"{VANTADB_CODE}: {message}"`, ERR-TS-01 — e.g. `VANTADB_NOT_FOUND: …`),
so you can branch on `err.message` without an error class hierarchy:

```js
try {
  await db.getNode("nope");
} catch (err) {
  if (String(err.message).startsWith("VANTADB_NOT_FOUND")) {
    // handle missing node
  }
  throw err;
}
```

Use `.catch()` or try/catch on every await; there is no error class hierarchy.

## Benchmark (PERF-BENCH-01)

A reproducible A/B harness comparing this native binding against the WASM
SDK (`vantadb-ts`) on the same operation mix. It prints numbers only —
performance **claims** are published by the release lead.

```bash
# from vantadb-node/
npm run bench                        # both backends, defaults
npm run bench -- --records 1000 --dim 384 --searches 200
npm run bench -- --backend native    # single backend
```

Prereqs: native binding built (`npm run build`) and `vantadb-ts` built
(`cd ../vantadb-ts && npm run build`). The final `JSON:` line on stdout is
the machine-readable contract.

> **Fairness caveat (by design):** the native backend runs persistent fjall
> storage (fsync included); the WASM backend in Node is in-memory only (no
> OPFS outside browsers). Native pays persistence costs WASM never sees.