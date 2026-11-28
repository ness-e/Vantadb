---
title: "EGO-15: max_degree_per_node y min_edge_weight expuestos en los tres bindings"
kind: task
status: completed
description: "Los setters existian y el cap ya era el default del pipeline (128); faltaba la superficie. graphrag_search acepta ahora max_degree_per_node y min_edge_weight en Py/Node/WASM, opt-in: sin argumentos la llamada es byte-identica."
id: EGO-15
---

# EGO-15: `max_degree_per_node` / `min_edge_weight` en los bindings

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `EGO-15`)
- **Origen:** Ego `VDB-REQ-07` (2026-10-10) · **Ver:** `FIND-300`
- **Esfuerzo:** 🟡 2-3 d declarados; el delta real de esta fila (la superficie) es de horas
- **Estado:** ✅ COMPLETED (la superficie de los tres bindings; el resto de la fila sigue abierto, ver §Deuda)

## Premisa (medida, no re-derivada)

| Qué | Medición |
|---|---|
| El cap existe y **ya es el default** | `DEFAULT_MAX_DEGREE_PER_NODE = 128` en `src/graphrag/expand.rs:13`, aplicado en `pipeline.rs:71` |
| Los setters existen y son **API pública** | `with_max_degree_per_node` (`pipeline.rs:96`), `with_min_edge_weight` (`pipeline.rs:101`); `tests/api/public-api.txt:3151,3153` |
| El camino con pipeline propio ya es público | `Embedded::graphrag_search_with_pipeline` (`src/sdk/builder.rs:312`, `public-api.txt:6187`) |
| Ningún binding los exponía | `rg -c 'max_degree_per_node' vantadb-python/src vantadb-node/src vantadb-wasm/src` → **0** en los tres |

Setters que ya existen + superficie inexistente = no hacía falta core nuevo, sólo forwardear.

## Contrato entregado

`graphrag_search` en los tres bindings acepta dos parámetros nuevos, **mismo nombre y semántica**
que el core, **opt-in**:

| Binding | Firma |
|---|---|
| Python | `graphrag_search(namespace, query=None, query_vector=None, max_degree_per_node=None, min_edge_weight=None)` |
| Node | `graphragSearch(namespace, query?, queryVector?, abort?, maxDegreePerNode?, minEdgeWeight?)` |
| WASM | `graphrag_search(namespace, query, query_vector, max_degree_per_node, min_edge_weight)` |

- `max_degree_per_node`: `Option<usize>` (`None` ⇒ 128 del core; `0` ⇒ sólo semillas).
- `min_edge_weight`: `Option<f32>` (`None` ⇒ sin filtro, comportamiento pre-EGO-15).
- **Byte-idéntico por defecto:** con ambos `None` el binding llama a `graphrag_search` (el camino
  viejo, tal cual); sólo construye `GraphRagPipeline` cuando se pasa al menos uno. No hay cambio de
  comportamiento para quien no usa los knobs.
- Los nuevos parámetros de Node van **al final** de la firma (`abort` conserva su posición): las
  llamadas posicionales existentes siguen compilando.

## Tests (uno por binding, efecto medido)

El fixture es el mismo en los tres: **una** semilla en el namespace consultado y **otro** registro en
un namespace distinto, unidos por una arista de peso 1.0. El segundo nunca es semilla, así que sólo
puede entrar en la expansión por la arista — justo lo que los knobs gobiernan. `nodes_expanded` es
**2** por defecto y **1** con cap 0 o con peso mínimo 2.0.

| Binding | Test | Corre |
|---|---|---|
| Python | `vantadb-python/tests/test_graphrag.py::test_graphrag_search_honours_{max_degree_per_node,min_edge_weight}` | pytest |
| Node (Rust) | `vantadb-node/src/lib.rs::tests::graphrag_search_forwards_the_pipeline_knobs` | `cargo test -p vantadb-node` (efecto real, sin rebuild del `.node`) |
| Node (JS) | `vantadb-node/tests/graphrag.test.ts` (describe EGO-15) | vitest, tras `npm run build` |
| WASM | `vantadb-wasm/tests/wasm_tests.rs::test_graphrag_search_honours_the_pipeline_knobs` | `wasm-pack test --chrome` (browser-only por diseño del archivo) |

## Verificación

| Comando | Resultado |
|---|---|
| `pytest vantadb-python/tests/test_graphrag.py -q` (before `maturin develop`) | ❌ **RED medido**: `TypeError: Client.graphrag_search() got an unexpected keyword argument 'min_edge_weight'` |
| `maturin develop` + `pytest vantadb-python/tests/test_graphrag.py -q` | ✅ **6 passed** (2 nuevos con el efecto medido) |
| `cargo test --manifest-path vantadb-node/Cargo.toml --lib graphrag` | ✅ **2 passed** — `graphrag_search_forwards_the_pipeline_knobs` + el test de abort (actualizado a la aridad nueva) |
| `cargo check -p vantadb-wasm --target wasm32-unknown-unknown --tests` | ✅ compila lib + suite |
| `cargo fmt --check` en las 4 crates | ✅ |
| `scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |

**No ejecutado, declarado:** el `wasm_bindgen_test` es browser-only
(`wasm_bindgen_test_configure!(run_in_browser)` en ese archivo) ⇒ corre con
`wasm-pack test --chrome`, no en este runner. El vitest del binding Node necesita
`npm run build` (rebuild release del `.node`) ⇒ queda para la lane de CI; el efecto
real sí está ejecutado por el test Rust del mismo crate (llama al mismo método).

## Deuda declarada

- **`vantadb-ts` no reenvía los knobs.** Su fachada (`vantadb-ts/src/vantadb.ts:837`,
  `native.ts:653`) sigue llamando con 3/4 argumentos, así que desde el paquete `vantadb-ts` las
  nuevas opciones no son alcanzables — el binding que las expone es el de abajo (WASM/Node). Fuera
  de la zona declarada de esta fila (los tres bindings del contrato). Son ~3 líneas por fachada.
- **`max_total_nodes`** sigue sin existir (alcance original de la fila; 0 hits repo-wide).
- **El sustrato sembrado sólo con `insert_node`/`add_edge` no produce semillas**: `find_seeds` sale
  de `embedded.search` sobre el índice de memoria (`src/graphrag/seed.rs:31`), así que un grafo
  sembrado a mano devuelve 0 nodos. Sigue abierto (causa raíz del `PARCIAL` original); esta fila
  **no** lo toca.
