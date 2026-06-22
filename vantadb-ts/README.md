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
  metadata: { lang: { type: "String", value: "en" } },
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
