# vantadb-node

> **Status: experimental (0.x)** — API sujeta a cambios sin deprecación hasta 1.0.
> Instalación desde source; publicación en npm en preparación (ver checklist en
> `docs/dev/tasks/TS-12.md` — el `npm publish` real lo ejecuta un humano).

VantaDB native Node.js bindings via [napi-rs](https://napi.rs) — persistent embedded memory & vector search, with graph traversal and hybrid search. The native backend to [`vantadb-ts`](../vantadb-ts) (WASM).

> **Full reference:** [`docs/api/NODE_SDK.md`](../docs/api/NODE_SDK.md) is the
> canonical source (quickstart, native-vs-WASM matrix, per-runtime examples,
> full API incl. lifecycle/maintenance/advanced search). This README is a
> minimal pointer — details live there.

> **Estado 2026-09-09:** paquete pre-npm (instalación desde source; `npm pack`
> incluye el prebuild `*.node` verificado). Ver
> `docs/dev/reviews/archive/research-vantadb-node-20260825.md` para el plan de distribución
> y `docs/dev/tasks/TS-12.md` para la checklist del publish humano.

## Instalación (source)

```bash
git clone <repo> && cd vantadb-node
npm install          # @napi-rs/cli + vitest
npm run build        # napi build --platform --release (requiere Rust)
```

Requisitos: Node ≥ 18 (este paquete) · Rust stable. Compat: `vantadb-ts` exige Node ≥ 22.19 (`vantadb-ts/package.json:engines`).

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
