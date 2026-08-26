# vantadb-node

VantaDB native Node.js bindings via [napi-rs](https://napi.rs) — persistent embedded memory & vector search, with graph traversal and hybrid search. The native backend to [`vantadb-ts`](../vantadb-ts) (WASM).

> **Estado 2026-08-25:** paquete pre-npm (instalación desde source). Ver
> `docs/reviews/research-vantadb-node-20260825.md` para el plan de distribución.

## Instalación (source)

```bash
git clone <repo> && cd vantadb-node
npm install          # @napi-rs/cli + vitest
npm run build        # napi build --platform --release (requiere Rust)
```

Requisitos: Node ≥ 18 · Rust stable.

## Uso

```ts
import { VantaDb } from "vantadb-node";

const db = await VantaDb.connect("./data");   // o ":memory:"
await db.put({ namespace: "agent/main", key: "pref-1", payload: "usa TypeScript" });

const hit = await db.search({
  namespace: "agent/main",
  query_vector: [...],
  text_query: "typescript",
});
console.log(hit.hits[0].record.payload, hit.hits[0].score);

// Grafo dirigido con traversal filtrada
await db.addEdge("1", "2", "depends_on");
const dag = await db.graphIsDag(["1"]);

await db.close();   // drena operaciones in-flight antes de flush
```

## API (`VantaDb` — todos async)

| Área | Métodos |
|------|---------|
| Ciclo de vida | `connect(path, {read_only?, memory_limit?})` · `flush()` · `close()` |
| Memoria | `put(record)` · `putBatch(records)` · `get(ns, key)` · `delete(ns, key)` · `list(ns, {filters?, limit?, cursor?})` · `listNamespaces()` |
| Búsqueda | `search(request)` (vector+filters+text, hybrid) · `explainSearch(request)` |
| Grafo | `insertNode` · `getNode` · `deleteNode(id, reason)` · `addEdge` · `removeEdge` · `graphBfs/Dfs/TopologicalSort/IsDag/FilteredTraversal/Degree` |
| Runtime | `capabilities()` |

Los ids de grafo viajan como **decimal strings** (u128 > Number.MAX_SAFE_INTEGER).

## Native vs WASM — cuándo usar cada uno

| | `vantadb-node` (nativo) | `vantadb-ts` (WASM) |
|---|---|---|
| Node.js / Bun (backend, CLIs, agentes) | ✅ **recomendado** | ✅ funciona |
| Browser / edge runtime | ❌ | ✅ único camino |
| Performance | nativa (benchmark A/B pendiente — H-09) | buena, con overhead wasm |

## Tests

```bash
npm test            # vitest run
```

## Licencia

Apache-2.0
