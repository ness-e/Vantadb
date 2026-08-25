# VantaDB TypeScript SDK

> WASM-powered embedded vector & graph memory for JavaScript runtimes.

```ts
import { VantaDB } from "vantadb";

const db = VantaDB.create();

await db.put({ namespace: "docs", key: "intro", payload: "VantaDB is a vector database", vector: [0.1, 0.2, ...] });
const results = await db.search({ namespace: "docs", query_vector: [0.15, 0.25, ...], top_k: 5 });
```

## Features

- **Vector search** — cosine/euclidean HNSW search
- **Hybrid search** — vector + BM25 text fusion (RRF)
- **Graph queries** — BFS, DFS, topological sort
- **Persistence** — export/import JSONL
- **TTL** — auto-expiring records
- **Works everywhere** — Node.js, Bun, Deno, browsers

## Install

```bash
npm install vantadb
```

## Quick Start

```ts
import { VantaDB } from "vantadb";

// In-memory (default)
const db = VantaDB.create();

// Or persistent:
// const db = VantaDB.open("./vanta_data");

// Store
await db.put({
  namespace: "memories",
  key: "greeting",
  payload: "Hello, world!",
  metadata: { lang: "en" },
  vector: [0.1, 0.2, 0.3],
});

// Search
const hits = await db.search({
  namespace: "memories",
  query_vector: [0.1, 0.2, 0.3],
  top_k: 10,
});

console.log(hits[0].record.payload); // "Hello, world!"

db.close();
```

## Real Embeddings

The examples above use toy vectors. Generate real ones with your own client —
local [Ollama](https://ollama.com) or the OpenAI API — and pass them to
`put()` / `search()`:

```ts
const res = await fetch("http://localhost:11434/api/embed", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ model: "nomic-embed-text", input: "Hello, world!" }),
});
const [vec] = (await res.json()).embeddings;

await db.put({ namespace: "memories", key: "greeting", payload: "Hello, world!", vector: vec });
const hits = await db.search({ namespace: "memories", query_vector: vec, top_k: 10 });
console.log(hits[0].record.payload); // "Hello, world!"
```

Use one embedding model per namespace — stored and query vectors must share
the same dimensionality. Full walkthrough:
[QUICKSTART → Real Embeddings](../docs/QUICKSTART.md#4-real-embeddings-optional).

## API

### Lifecycle

| Method | Description |
|--------|-------------|
| `VantaDB.create(config?)` | Create in-memory or configured instance |
| `VantaDB.open(path)` | Open persistent store from disk |
| `.close()` | Free WASM resources |

### CRUD

| Method | Description |
|--------|-------------|
| `.put(input)` | Store a memory record |
| `.putBatch(inputs)` | Batch store |
| `.get(namespace, key)` | Retrieve by key |
| `.delete(namespace, key)` | Remove by key |
| `.deleteByFilter(namespace, filter)` | Batch delete matching an AND-combined filter; returns count (rejects empty filter) |
| `.list(namespace, options?)` | List with pagination |
| `.listNamespaces()` | List all namespaces |

### Search

| Method | Description |
|--------|-------------|
| `.search(request)` | Hybrid vector + text search |
| `.searchVector(vector, topK)` | Pure vector search |
| `.explainSearch(request)` | Search with score breakdown |

### Graph

| Method | Description |
|--------|-------------|
| `.insertNode(id, content?, vector?, fields?)` | Create a graph node |
| `.getNode(id)` | Get node with edges |
| `.deleteNode(id, reason?)` | Remove node |
| `.addEdge(source, target, label?, weight?)` | Create edge |
| `.graphBfs(roots, maxDepth?)` | BFS traversal |
| `.graphDfs(roots, maxDepth?)` | DFS traversal |
| `.graphTopologicalSort(roots)` | Topological sort |
| `.graphIsDag(roots)` | Check if DAG |

### Maintenance

| Method | Description |
|--------|-------------|
| `.flush()` | Flush WAL to storage |
| `.compactWal()` | Compact WAL |
| `.purgeExpired()` | Remove TTL-expired records |
| `.rebuildIndex()` | Rebuild ANN index |
| `.compactLayout()` | Compact storage layout |
| `.operationalMetrics()` | Get runtime metrics |
| `.capabilities()` | Get build capabilities |

### Export / Import

| Method | Description |
|--------|-------------|
| `.exportNamespace(path, namespace)` | Export a namespace to JSONL |
| `.exportAll(path)` | Export all namespaces to JSONL |
| `.importRecords(records)` | Import records from an array |
| `.importFile(path)` | Import records from a JSONL file |

### Text Index

| Method | Description |
|--------|-------------|
| `.auditTextIndex(namespace?)` | Audit text index integrity |
| `.auditTextIndexDeep(namespace?)` | Deep structural text index audit |
| `.repairTextIndex()` | Repair text index from canonical storage |

### Utilities

| Method | Description |
|--------|-------------|
| `.query(iqlQuery)` | Execute IQL query |
| `.generateSnippet(payload, query, withHighlighting?)` | Generate highlighted text snippet |

## Cross-SDK Search Parity

VantaDB exposes the same search capabilities across bindings, but **the `search()`
name carries different semantics per SDK**. Read this before porting code between
TypeScript and Python. The canonical method→domain map lives in
[`docs/api/BINDINGS_NAMESPACES.md`](../docs/api/BINDINGS_NAMESPACES.md).

| Capability | TypeScript SDK | Python SDK |
|---|---|---|
| `search()` meaning | **Hybrid** search (vector + text) → returns `SearchHit[]` | **Pure vector ANN** (K-NN) → returns `(node_id, distance)` |
| Hybrid (vector + text) | `search({ namespace, query_vector, text_query })` | `search_memory(namespace, query_vector, text_query=...)` |
| Pure vector ANN | `searchVector(vector, topK?)` | `search(vector, top_k=10)` |
| Namespace scoping | `search({ namespace })` | `search_memory(namespace=...)` (`search()` is global over nodes) |
| Filters | `search({ filters })` | `search_memory(filters=...)` |
| `top_k` | `search({ top_k })` / `searchVector(v, topK)` | `search(top_k=)` / `search_memory(top_k=)` |
| `distance_metric` | `search({ distance_metric: "Cosine"/"Euclidean" })` | `search_memory(distance_metric="cosine"/"euclidean")` |
| `text_query` | `search({ text_query })` | `search_memory(text_query=...)` |
| Explain | `search({ explain })` + `explainSearch()` | `search_memory(explain=True)` + `explain_memory_search()` |
| Batch search | — | `search_batch(vectors)` / `search_batch_requests(requests)` — **Python-only** |
| Hybrid method / profile override | — | `search_memory(method=...)` — **Python-only** |

> **Porting hazard:** `search()` in TypeScript and `search()` in Python do **different
> things**. To get pure vector ANN in TypeScript use `searchVector()`; to get hybrid
> search in Python use `search_memory()`.

## Domain Sub-clients

Every flat method is also reachable through a **domain sub-client**: `db.memory.*`, `db.graph.*`, `db.wiki.*`, `db.system.*`. Sub-clients are pure organizational sugar over the flat API — each call forwards verbatim to the flat method of the same behavior.

> **Backward-compat guarantee:** the flat API is unchanged. `db.memory.put(x)` and `db.put(x)` are the same call; existing code keeps working as-is. Canonical method→domain map: [`docs/api/BINDINGS_NAMESPACES.md`](../docs/api/BINDINGS_NAMESPACES.md).

```ts
// memory — namespace+key records, search, TTL
await db.memory.put({ namespace: "docs", key: "intro", payload: "...", vector: [0.1, 0.2] });
const hits = await db.memory.search({ namespace: "docs", query_vector: [0.15, 0.25], top_k: 5 });
await db.memory.purgeExpired();

// graph — node/edge CRUD + traversals (traversals use short names)
const node = await db.graph.getNode(42);
const reachable = await db.graph.bfs([42], 3);
const order = await db.graph.topologicalSort([1, 2, 3]);
if (await db.graph.isDag([1, 2])) { /* safe to topologically sort */ }

// wiki — empty in v1: wiki features are core-only (not exposed via WASM yet)
Object.keys(db.wiki); // []

// system — lifecycle, metrics, IQL, maintenance, import/export
console.log(await db.system.capabilities());
const result = await db.system.query("(match (node :content \"rust\") (return node))");
await db.system.flush();
```

Notes:

- Sub-clients are lazy, frozen (`Readonly`) singletons — accessing `db.graph` twice returns the same object.
- `conversation` / `skills` sub-clients do not exist yet; their capabilities live in the core crate only.
- Python exposes the equivalent grouping via `db.memory`, `db.graph`, `db.system`, `db.wiki` — see [PYTHON_SDK.md → Domain Sub-clients](../docs/api/PYTHON_SDK.md).

## Runtimes

| Runtime | Status |
|---------|--------|
| Node.js 18+ | ✅ |
| Bun | ✅ |
| Deno | ✅ |
| Browser | ✅ (ESM) |

## Examples

See the [`examples/`](./examples) directory:

- [Vercel AI SDK](./examples/vercel-ai) — streaming chat with VantaDB memory
- [LangChain](./examples/langchain) — LangChain vector store integration
- [LlamaIndex](./examples/llamaindex) — LlamaIndex document store

## License

Apache 2.0
