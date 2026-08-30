# Bindings Namespace Map

> **Status:** canonical contract for SDKB campaign (`docs/plans/2026-08-22-vantadb-bindings-sdk.md`).
> **Decisions:** D42 (sub-clients TS/Python only — zero WASM/Rust changes), D43 (v1 groups already-exposed methods only; vanta-memory pipeline is core-only and deferred), D45 (additive → minor bump).
> **Rule:** every public method of every SDK maps to **exactly one** domain. `system` is the catch-all for orphan operations (capabilities, import/export, metrics, lifecycle).
>
> **Sub-client docs:** TypeScript examples in [`vantadb-ts/README.md` → Domain Sub-clients](../../vantadb-ts/README.md#domain-sub-clients) · Python examples in [`PYTHON_SDK.md` → Domain Sub-clients](./PYTHON_SDK.md#domain-sub-clients).

## Domain Taxonomy

| Domain | Covers |
|---|---|
| `memory` | Records in a namespace: put/get/list/search/supersede/TTL purge, text snippets over payloads |
| `graph` | Node/edge CRUD and traversals (BFS, DFS, topological sort, DAG check, PageRank, degree) |
| `conversation` | Reserved — L0–L3 context-engine pipeline lives in crate `vanta-memory`, **not exposed via bindings today** (D43) |
| `skills` | Reserved — no binding surface exists today (D43) |
| `wiki` | Summary/archive lifecycle over nodes (recover archived nodes). Full wiki features are core-only (D43) |
| `system` | Catch-all: constructors/lifecycle, capabilities/hardware profile, metrics, IQL query engine, index maintenance, compaction, import/export |

## SDK Surface Differences (verified 2026-08-22 via grep)

| Capability | WASM | TS | Python |
|---|---|---|---|
| `supersede(namespace, old_key, new_key)` | ✅ | ❌ | ✅ |
| `count(namespace, filter?)` | ✅ | ❌ | ✅ |
| `similar_to_key(namespace, key, top_k)` | ✅ | ❌ | ✅ |
| `remove_edge(source_id, target_id, label)` | ✅ | ❌ | ✅ |
| `search_multi(namespaces, request)` | ✅ | ✅ | ❌ |
| `sparse_vector` on `put()` / `put_batch()` | ✅ | ✅ | ✅ |
| `exclude_superseded` on `search()` | ✅ | ✅ | ✅ |
| `exclude_superseded` on `list()` | ✅ (WSM-06) | ✅ | ❌ |
| `filter_ops` on `search()` | ❌ (core limitation: flat `filters` only) | ❌ | ❌ |
| `search_profile` on `search()` | ❌ (advanced, internal `None`) | ❌ | ❌ |
| Node CRUD by explicit id (`insert_node`/`get_node`/`delete_node`) | ✅ | ✅ | ⚠️ via `insert`/`get`/`delete` (`id: u128`) |
| `graph_page_rank` / `graph_degree_centrality` | ❌ (has `graph_degree`) | ❌ (has `graphDegree`) | ✅ both |
| `delete_by_filter` / `search_vector` / `audit_text_index_deep` / `export_namespace_filtered` / `import_records` | ✅ | ✅ | ⚠️ `delete_by_filter` ✅; `search_vector` / `audit_text_index_deep` / `export_namespace_filtered` / `import_records` ❌ |
| `bulk_import` / `bulk_import_bytes` | ✅ | ❌ | ✅ |
| `put_batch_raw` / `search_batch` / `search_batch_requests` / `hardware_profile` | ❌ | ❌ | ✅ |
| `recover_archived_nodes(summary_id)` | ❌ | ❌ | ✅ (wiki) |
| Hybrid search request shape | `search(request)` | `search(SearchRequest)` | `search(vector)` pure ANN + separate `explain_memory_search` |

⚠️ **Naming hazard:** `get`/`delete` are memory-record ops (namespace+key) in WASM/TS but **node-level ops (`id: u128`, graph domain)** in Python. Sub-client design must not blindly mirror names across SDKs.

## WASM (`vantadb-wasm/src/lib.rs`) — 47 pub fns

| Method | Domain | Notes |
|---|---|---|
| `new` | system | constructor |
| `open` | system | constructor |
| `close` | system | lifecycle |
| `put` | memory | supports `sparse_vector` (WSM-06) |
| `put_batch` | memory | supports `sparse_vector` (WSM-06) |
| `get` | memory | namespace+key |
| `delete` | memory | namespace+key |
| `delete_by_filter` | memory | full operator `filter_ops` |
| `list` | memory | supports `exclude_superseded` (WSM-06) |
| `list_namespaces` | memory | |
| `search` | memory | hybrid request; supports `exclude_superseded` |
| `search_vector` | memory | pure ANN |
| `search_multi` | memory | cross-namespace hybrid search |
| `similar_to_key` | memory | vector search from existing key |
| `count` | memory | optional operator filter |
| `supersede` | memory | mark record as superseded |
| `explain_memory_search` | memory | explain plan |
| `generate_snippet` | memory | text highlight over payload |
| `purge_expired` | memory | TTL housekeeping |
| `insert_node` | graph | |
| `get_node` | graph | |
| `delete_node` | graph | |
| `add_edge` | graph | |
| `remove_edge` | graph | |
| `graph_bfs` | graph | |
| `graph_dfs` | graph | |
| `graph_topological_sort` | graph | |
| `graph_is_dag` | graph | |
| `graph_filtered_traversal` | graph | |
| `graph_degree` | graph | |
| `capabilities` | system | |
| `operational_metrics` | system | |
| `query` | system | IQL engine |
| `flush` | system | durability flush |
| `compact_wal` | system | WAL maintenance |
| `compact_layout` | system | storage layout |
| `rebuild_index` | system | index maintenance |
| `reindex_hnsw_from_text` | system | index maintenance |
| `repair_text_index` | system | index maintenance |
| `audit_text_index` | system | index diagnostics |
| `audit_text_index_deep` | system | index diagnostics |
| `export_all` | system | portability |
| `export_namespace` | system | portability |
| `export_namespace_filtered` | system | portability |
| `import_file` | system | portability |
| `import_records` | system | portability |
| `bulk_import` | system | portability |
| `bulk_import_bytes` | system | portability |

**Totals:** memory 15 · graph 11 · system 21 = 47 ✔

## TypeScript (`vantadb-ts/src/vantadb.ts`) — 38 public methods

(`native.ts` implements the sync subset: `capabilities`, `close`, `delete`, `flush`, `get`, `list`, `listNamespaces`, `put`, `putBatch`, `search`.)

| Method | Domain | Exposed today | Notes |
|---|---|---|---|
| `put` | memory | ✅ | delegates to wasm `put` |
| `putBatch` | memory | ✅ | |
| `get` | memory | ✅ | namespace+key |
| `delete` | memory | ✅ | namespace+key |
| `deleteByFilter` | memory | ✅ | |
| `list` | memory | ✅ | |
| `listNamespaces` | memory | ✅ | |
| `search` | memory | ✅ | hybrid request |
| `searchVector` | memory | ✅ | pure ANN |
| `explainSearch` | memory | ✅ | wraps wasm `explain_memory_search` |
| `generateSnippet` | memory | ✅ | |
| `purgeExpired` | memory | ✅ | TTL housekeeping |
| `insertNode` | graph | ✅ | |
| `getNode` | graph | ✅ | |
| `deleteNode` | graph | ✅ | |
| `addEdge` | graph | ✅ | |
| `graphBfs` | graph | ✅ | |
| `graphDfs` | graph | ✅ | |
| `graphTopologicalSort` | graph | ✅ | |
| `graphIsDag` | graph | ✅ | |
| `graphFilteredTraversal` | graph | ✅ | |
| `graphDegree` | graph | ✅ | |
| `close` | system | ✅ | lifecycle |
| `capabilities` | system | ✅ | |
| `operationalMetrics` | system | ✅ | |
| `query` | system | ✅ | IQL |
| `flush` | system | ✅ | |
| `compactWal` | system | ✅ | |
| `compactLayout` | system | ✅ | |
| `rebuildIndex` | system | ✅ | |
| `reindexHnswFromText` | system | ✅ | |
| `repairTextIndex` | system | ✅ | |
| `auditTextIndex` | system | ✅ | |
| `auditTextIndexDeep` | system | ✅ | |
| `exportAll` | system | ✅ | |
| `exportNamespace` | system | ✅ | |
| `importRecords` | system | ✅ | |
| `importFile` | system | ✅ | |

**Totals:** memory 12 · graph 10 · system 16 = 38 ✔

**Not exposed in TS (wasm-only or Python-only), deferred per D43/D42:** `supersede` (Python-only), `graph_page_rank`/`graph_degree_centrality` (Python-only), `bulk_import`/`bulk_import_bytes` (wasm/Python-only), `hardware_profile` (Python-only), `recover_archived_nodes` (Python-only). Do NOT add wrappers in SDKB-02 — v1 is grouping only.

## Python (`vantadb-python/src/lib.rs`) — 44 pyclass methods (+ module-level `connect()`)

| Method | Domain | Exposed today | Notes |
|---|---|---|---|
| `new` | system | ✅ | constructor |
| `connect` *(module fn)* | system | ✅ | alias of `new` |
| `insert` | graph | ✅ | node insert by explicit id |
| `get` | graph | ✅ | **node get by `id: u128`** (≠ TS semantics) |
| `delete` | graph | ✅ | **node delete by id** (≠ TS semantics) |
| `add_edge` | graph | ✅ | |
| `graph_bfs` | graph | ✅ | |
| `graph_dfs` | graph | ✅ | |
| `graph_topological_sort` | graph | ✅ | |
| `graph_is_dag` | graph | ✅ | |
| `graph_page_rank` | graph | ✅ | Python-only |
| `graph_degree_centrality` | graph | ✅ | Python-only (= wasm `graph_degree`) |
| `put` | memory | ✅ | |
| `put_batch` | memory | ✅ | |
| `put_batch_raw` | memory | ✅ | Python-only |
| `get_memory` | memory | ✅ | |
| `delete_memory` | memory | ✅ | |
| `delete_by_filter` | memory | ✅ | operator filter_ops (flat → `$eq`, or `{"$op": value}`) |
| `count` | memory | ✅ | optional operator filter |
| `similar_to_key` | memory | ✅ | vector search from an existing key |
| `list_memory` | memory | ✅ | |
| `search_memory` | memory | ✅ | hybrid |
| `search` | memory | ✅ | pure vector ANN |
| `search_batch` | memory | ✅ | Python-only |
| `search_batch_requests` | memory | ✅ | Python-only |
| `explain_memory_search` | memory | ✅ | |
| `supersede` | memory | ✅ | **Python-only** |
| `generate_snippet` | memory | ✅ | |
| `purge_expired` | memory | ✅ | TTL |
| `list_namespaces` | memory | ✅ | |
| `capabilities` | system | ✅ | |
| `hardware_profile` | system | ✅ | Python-only |
| `operational_metrics` | system | ✅ | |
| `query` | system | ✅ | IQL |
| `flush` | system | ✅ | |
| `compact_wal` | system | ✅ | |
| `compact_layout` | system | ✅ | |
| `rebuild_index` | system | ✅ | |
| `reindex_hnsw_from_text` | system | ✅ | |
| `repair_text_index` | system | ✅ | |
| `audit_text_index` | system | ✅ | deep variant absent in Python |
| `export_namespace` | system | ✅ | |
| `export_all` | system | ✅ | |
| `import_file` | system | ✅ | |
| `bulk_import` | system | ✅ | |
| `bulk_import_bytes` | system | ✅ | |
| `recover_archived_nodes` | wiki | ✅ | summary-node shadow archive recovery |
| `close` | system | ✅ | lifecycle |

**Totals:** memory 18 · graph 10 · wiki 1 · system 18 = 47 pyclass methods ✔ (+ module-level `connect()` → system, 48 total surface)

> **Naming hazard reminder:** Python `insert` is classified as graph (node-level). The name collides with memory-record insertion semantics in other ecosystems — sub-client tests must use the real signatures above.

**Not exposed in Python (wasm/TS-only), deferred per D42:** `search_vector`, `audit_text_index_deep`, `export_namespace_filtered`, `import_records`.

## Core-Only Capabilities (D43 — deferred, NOT part of this campaign)

| Capability | Lives in | Would land under |
|---|---|---|
| Memory pipeline L0–L3 (persona, compression, recall assembly) | crate `vanta-memory` | `conversation` |
| Context engine / narrative assembly | crate `vanta-memory` | `conversation` |
| Wiki pages / TDAM chunker / wiki seeds | core (`vantadb::wiki`) | `wiki` |
| Skill stores | future | `skills` |

Exposing any of these requires new Rust bindings (out of scope per D42). Tracked as post-campaign work.

## Sub-Client Design v1

### TypeScript (SDKB-02)

Lazy getters on `VantaDB` returning frozen delegate objects. Arrow functions capture `this` (no bind drift):

```ts
class VantaDB {
  // ...flat methods unchanged...

  private _memory?: Readonly<MemoryClient>;
  get memory(): Readonly<MemoryClient> {
    return (this._memory ??= Object.freeze({
      put: (input: MemoryInput) => this.put(input),
      search: (req: SearchRequest) => this.search(req),
      // ...one arrow per mapped method...
    }));
  }
  get graph(): Readonly<GraphClient> { /* same pattern */ }
  get wiki(): Readonly<WikiClient> { /* empty in TS v1 */ }
  get system(): Readonly<SystemClient> { /* same pattern */ }
}
```

- **Delegation only** — zero new logic (D43); stop condition from plan applies.
- `conversation`/`skills` getters are omitted in v1 (no methods exist; D43).
- Types reuse existing `types.ts`; no duplicates.
- `db.memory.x(...) === db.x(...)` result/firma identity is the test contract.

### Python (SDKB-03 — recommendation, final call at its DISCOVERY)

PyO3 nested properties are friction-heavy. Recommended (ponytail-simplest that satisfies `db.memory.*` tests):

1. Define lightweight `#[pyclass]` delegate structs (`MemoryClient`, `GraphClient`, `SystemClient`, `WikiClient`) holding `db: Py<VantaDB>`; each `#[pymethod]` forwards to the flat method.
2. Expose them as read-only attributes via `#[getter]` on `VantaDB`, constructing each delegate once lazily.
3. Fallback (plan stop-condition): if getter wiring fights PyO3, ship helper functions instead and update this map.

### Cross-SDK rule

Sub-clients group by **domain**, never by mirrored method name. Where semantics diverge (`get`/`delete`, `search`), each SDK's sub-client exposes its own real surface — this document is the arbiter.
