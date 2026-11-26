# vantadb-node

> **Status: active** (core-promise until 1.0 — one of the three active language
> connectors: native Node.js; see the [root README](../README.md#installation)).

VantaDB native Node.js bindings via [napi-rs](https://napi.rs) — persistent embedded memory & vector search, with graph traversal and hybrid search. The native backend to [`vantadb-ts`](../vantadb-ts) (WASM).

> **Full reference:** [`docs/api/NODE_SDK.md`](../docs/api/NODE_SDK.md) is the
> canonical source (quickstart, native-vs-WASM matrix, per-runtime examples,
> full API incl. lifecycle/maintenance/advanced search). This README is a
> minimal pointer — details live there.
>
> **Estado 2026-10-20:** publicado en npm (`vantadb-node@0.9.0`, verificado live
> contra el registry). La instalación desde source sigue siendo válida para
> trabajar contra el árbol actual. Ver
> Notion › Reviews 2026 › `research-vantadb-node-20260825.md` (https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd) para el plan de
> distribución y `docs/dev/avance/activo/ci-cd.md` §TS-12 para la checklist
> del publish humano.

## Instalación

```bash
npm install vantadb-node
```

O desde el código fuente:

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

const hits = await db.search({
  namespace: "agent/main",
  query_vector: [...],
  text_query: "typescript",
});
console.log(hits[0]?.record.payload, hits[0]?.score);   // search() → MemorySearchHit[]

// IQL (mismo contrato que WASM/Python)
const result = await db.query('FROM agent/main WHERE text ~ "typescript", min = 0.7');

// Grafo dirigido con traversal filtrada
await db.addEdge("1", "2", "depends_on");
const dag = await db.graphIsDag(["1"]);

await db.close();   // drain async: espera a las ops in-flight, flush, cierra
```

## API (`VantaDb` — todos async)

| Área | Métodos |
|------|---------|
| Ciclo de vida | `connect(path, {read_only?, memory_limit?, reserve_bytes?})` · `flush()` · `close()` (drain async) · `reopen(path?)` |
| Memoria | `put(record)` · `putBatch(records)` · `get(ns, key)` · `delete(ns, key)` · `forget(ns, key, mode?)` (`soft`/`hard`/`certified`) · `list(ns, {filters?, limit?, cursor?})` · `listNamespaces()` |
| Búsqueda | `search(request, abort?)` (vector+filters+text, hybrid) · `explainSearch(request)` · `similarToKey` · `searchWithMethod` · `searchMulti` |
| IQL | `query(iql, abort?)` → `{Read\|Write\|StaleContext}` (FIND-268) |
| Grafo | `insertNode` · `getNode` · `deleteNode(id, reason)` · `addEdge` · `removeEdge` · `graphBfs/Dfs/TopologicalSort/IsDag/FilteredTraversal/Degree` |
| GraphRAG | `graphragSearch(ns, query?, queryVector?, abort?)` |
| Cognitiva (`vanta-memory`) | `memoryRecall` · `contextAssemble` · `memoryConsolidate` · `memoryReinforce` |
| Mantenimiento | `vacuum` · `rebuildIndex` · `compactLayout` · `compactWal` · `purgeExpired` · `deleteByFilter` · `count` · `versions` · `getVersion` · `supersede` |
| Portabilidad | `checkpointSave(path)` · `checkpointLoad(path, quarantine?)` |
| Runtime | `capabilities()` · `AbortHandle` (cancelación cooperativa — detach, no DOM `AbortSignal`) |

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
