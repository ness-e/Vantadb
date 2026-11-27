---
title: "FIND-773: `add_edge` perdía aristas en silencio — el read-modify-write de la lista no estaba serializado"
kind: task
status: completed
description: "`add_edge` leía el nodo, añadía la arista y lo reescribía sin serializar: 8 escritores sobre un mismo nodo dejaban 164 de 800 aristas (636 perdidas, en silencio). Un guard de escritura sobre la `purge_lock` del engine —el idioma que ya usan `insert_one_internal` y `put_batch_inner`— cubre get+insert de ambas mitades. Medido antes/después, techo y hueco declarados."
---

# FIND-773: `add_edge` hacía get-modify-insert sin lock

## Metadata

- **Fuente:** `docs/dev/Backlog.md` §`Alta 2026-11-26 - add_edge es get-modify-insert sin lock` (fila retirada al cerrar)
- **Origen:** `FIND-756` (contrato de `ADR-0066`, deuda declarada en su §"Deuda que este contrato deja abierta", ítem 3)
- **Esfuerzo declarado / real:** 🟡 1-2d / ~2h
- **Prioridad:** 🟠 Alta
- **Creado:** 2026-11-26 · **Cerrado:** 2026-11-27
- **Archivos tocados:** `src/sdk/api/graph.rs` (fix) · `tests/graph_edge_concurrency.rs` (nuevo) · este fichero · `docs/dev/Backlog.md` (sólo mi fila) · `docs/dev/avance/activo/core-engine.md`

## Impacto mapeado (Regla 0)

- **Leído completo:** `src/sdk/api/graph.rs` (351 líneas, antes del cambio) — los cinco mutadores del fichero (`insert_node`, `delete_node`, `add_edge`, `remove_edge`, `ensure_edge`), `src/sdk/builder.rs:13-120` (la struct `Embedded`, `purge_lock` y su doc), `src/sdk/api/memory.rs` (los read-modify-write del módulo hermano: `:588-685` `resolve_existing_for_write{,_locked}`, `:1131-1316` `insert_one_internal`, `:1486-1705` `put_batch_inner`, `:2440-2537` `supersede`), `src/storage/engine/insert.rs:156-230` (`insert` y su `insert_lock`), `.opencode/rules/concurrency-async.md`, `.opencode/rules/core-engine.md`, `tests/concurrency_primitives.rs` y `tests/memory_multi_writer.rs` (idioma de tests concurrentes del repo).
- **Referencias hacia dentro de `add_edge`:** el SDK público (este crate), `vantadb-python` (`src/lib.rs:2398`), `vantadb-node` (`src/lib.rs:662`), `vantadb-wasm` (`src/lib.rs:1943`), `vantadb-mcp` (`src/code_index.rs:295,314`), `vanta-memory` (`core/ontology/mod.rs:296`, la entrada de `ADR-0066`), `benchmarks/graphrag_bench.rs`, `examples/rust/graphrag.rs`, y 6 ficheros de test del repo. **Ninguna firma ni semántica cambia**, así que ninguno de esos consumidores se ve afectado.
- **Referencias hacia fuera:** `engine.intern_label`, `engine.get`, `engine.insert`, `self.purge_lock`, `Error::NodeNotFound`, `now_ms()`. Sólo se usa lo que ya estaba importado/visible; `purge_lock` es `pub(crate)` en el mismo crate.
- **Veredicto:** un guard de lock dentro de `add_edge`, sin tocar la firma ni el orden de las operaciones. Sin dependencias nuevas (prohibido en esta tarea). Riesgo de deadlock evaluado explícitamente (abajo, §Mecanismo).

## La demostración, antes del fix (RED)

`tests/graph_edge_concurrency.rs`, sobre `HEAD` (`ac4072a3e`, `graph.rs` sin el guard):

```
$ cargo nextest run --profile audit -p vantadb --build-jobs 2 --test graph_edge_concurrency --no-fail-fast

thread 'concurrent_add_edge_loses_no_edges_on_one_node' panicked at tests\graph_edge_concurrency.rs:76:5:
  assertion `left == right` failed: lost 636 of 800 edges on the shared node:
  the read-modify-write of the edge list is not serialized (FIND-773)
    left: 164
   right: 800

thread 'add_edge_serializes_across_handles_over_one_engine' panicked at tests\graph_edge_concurrency.rs:121:5:
  assertion `left == right` failed: two handles over one engine lost 640 of 800
  edges: the serialization must be engine-owned, not per-wrapper (FIND-773)
    left: 160
   right: 800

error: test run failed
```

**Reproducibilidad:** el escenario es `8` hilos × `100` aristas distintas sobre un nodo compartido (`HUB = 1`), sincronizados con un `Barrier` en el arranque. No hace falta inyectar `yield_now()` ni sleep: la ventana entre `engine.get(source)` y `engine.insert(&source)` abarca un `get` + un `insert` sobre el destino, y con 800 operaciones se cruza siempre. Tres corridas sobre `HEAD` dieron `164`, `156`, `156` aristas de 800 — **rojo siempre**, no por timing afortunado. El test es un fallo de aserción de conteo, no de tiempo: no puede pasar "por suerte".

## El mecanismo elegido, y por qué encaja con el fichero

```rust
// src/sdk/api/graph.rs · add_edge, tras `let engine = self.engine_handle()?;`
let _write_guard = self.purge_lock.write();
```

Una línea de mecanismo. Cubre el `get` → `push` → `insert` del **origen** y el del **destino** bajo el mismo guard: lo que garantiza es que **no hay interleaving** entre dos escritores, no que las dos mitades sean atómicas (ver §Deuda: `add_edge` escribe el origen antes de validar el destino).

| Opción | Veredicto |
|---|---|
| **`purge_lock.write()` por handle** (elegida) | Es **el idioma que ya existe**: `insert_one_internal:1152`, `put_batch_inner:1542`, `delete_inner:1908`, `put_record_exact:2187`, `purge_expired` — todos los read-modify-write del SDK sobre este handle toman el mismo guard. `Arc<RwLock<()>>` **propiedad del engine** (`builder.rs:48-52`), compartido por todo `Embedded` construido con `from_engine` (`FIND-651`), así que también cubre dos wrappers PyO3/WASM/MCP sobre la misma base. Sin dependencia nueva, sin campo nuevo, sin firma nueva. |
| Lock por nodo (`DashMap<u128, Mutex>`) | **Descartada**: una arista escribe **dos** nodos. Dos `add_edge` opuestos (`A→B` y `B→A`) tomando locks en orden de llegada se bloquean mutuamente: hace falta orden total (`min/max`) y un mapa de locks con ciclo de vida — patrón nuevo en un fichero que ya tiene uno, y superficie de deadlock nueva. |
| CAS con reintento | **Descartada**: no existe primitiva. `StorageEngine::insert` es incondicional (`insert.rs:156`, sin parámetro de versión esperada); añadir compare-and-swap es `src/storage/**`, propiedad de Arch/Engine y fuera de la lista de escritura de esta tarea. |
| Serializar por `writer` / por pase | **Descartada**: no hay un "writer" en el SDK. `add_edge` es API pública que llaman Python/Node/WASM/MCP y `vanta-memory`; serializar "el pase" no protege a dos llamadores cualesquiera, que es exactamente el caso que el test ejerce. |

**Orden de locks (chequeo de deadlock, no asumido):** el único orden que aparece en el código es `purge_lock` → `insert_lock` del engine (dentro de `engine.insert`), que se toma y **se suelta** dentro de la llamada. Ningún camino toma `insert_lock` y luego `purge_lock`, así que no hay ciclo. Es el mismo orden que `insert_one_internal` ya usa desde 0.9.0.

## Techo declarado (`ponytail:`)

En el propio código:

> `ponytail: one handle-wide write lock instead of a per-node lock — an edge touches TWO nodes, so per-node locks need a total order or they deadlock on A→B racing B→A. Ceiling: edge writes now serialize against the other SDK read-modify-writes on this handle. Escape hatch: a striped lock keyed by the ordered (min(src,dst), max(src,dst)) pair.`

**Medición de contención** (build `dev` con `debuginfo`, máquina compartida — los absolutos no valen, los ratios sí). Probe temporal de `4 000` `add_edge` (`tests/zz_find773_probe.rs`, borrado tras medir; el código está reproducido abajo):

| Escenario | Sin guard (`HEAD`) | Con guard |
|---|---|---|
| 1 hilo, `4 000` aristas | `2 743` ops/s (`365 µs/op`) | `2 575` ops/s (`388 µs/op`) |
| (segunda corrida, máquina cargada) | `901` ops/s (`1 110 µs/op`) | `2 632` ops/s (`380 µs/op`) |
| **8 hilos, mismo nodo, `4 000` aristas** | `4 080`–`17 417` ops/s **descartando ~80 % de las aristas** | **`2 232` ops/s** |

Lectura honesta de esos números:

1. **No contendido:** el spread intra-configuración (`365`–`1 110 µs/op en HEAD`) es mayor que la diferencia entre configuraciones, así que el guard es **indistinguible del ruido** frente a un `add_edge` que cuesta cientos de µs (el `engine.insert` domina).
2. **8 escritores sobre el mismo nodo:** `2 232` ops/s agregadas contra `2 632/s` de un solo hilo ⇒ el lock hace que `8` escritores del **mismo** nodo se comporten como uno. Es el peor caso para un lock de handle y es justo el que el linaje de `ADR-0066` produce (cada pase escribe aristas de escena/persona/L1 sobre un puñado de nodos calientes).
3. La cifra agregada mayor de `HEAD` **no es throughput, es pérdida**: si 8 de cada 10 escrituras se descartan, "va más rápido" por arista escrita es falso. Un `add_edge` que pierde la arista no es más rápido, es un no-op.

**No medido / declarado:** no se midió el efecto sobre un `bench` de release con `canonical_p99` (Regla 9). Este cambio es de correctitud, no de performance, y el delta medido está dentro del ruido del instrumento; si alguien quiere un número de release, el escape hatch del lock por par ordenado es la respuesta antes de aceptar contención.

## Hueco declarado, **no** cerrado

`remove_edge` y `ensure_edge` tienen el mismo read-modify-write y **no** reciben el guard:

- Se llaman **desde dentro** del guard en `src/sdk/api/memory.rs:1259,1262` (`insert_one_internal`), `:1697,1700` (`put_batch_inner`), `:2243,2246,2253,2258` (`put_record_exact`) y `:2515,2520,2525` (`supersede`, que sólo tiene `supersede_lock`).
- Un `RwLock` de `parking_lot` **no es reentrante**: tomar `purge_lock.write()` dentro de una sección que ya lo tiene es deadlock (o panic con deadlock detection), así que no se puede añadir la línea sin tocar esos call sites.
- Cerrarlo bien exige el split que el módulo ya usa (`resolve_existing_for_write` / `resolve_existing_for_write_locked`): `remove_edge_locked`/`ensure_edge_locked` internos, y el guard sólo en la entrada pública. Eso toca `src/sdk/api/memory.rs`, **fuera de la lista de escritura de esta tarea**.
- **Hay exactamente dos llamadores que hoy no están bajo `purge_lock`:**
  - `supersede` (`src/sdk/api/memory.rs:2515-2529`, con sólo `supersede_lock`).
  - `bulk_import_stream` (`src/sdk/api/memory.rs:3259`): su último guard está en `:2798` (`purge_expired`), y la llamada es hermana de `engine.insert(&node)` (`:3257`), fuera de cualquier sección protegida. *(Corregido en review: la primera versión de este fichero decía "el único llamador que no está bajo el lock" — falso, lo levantó `vanta-review` con la línea exacta.)*
- Por eso los otros ocho call sites sí están cubiertos por el fix aunque no lo pidan. Y por eso el arreglo de esos dos **no** necesita el split locked/unlocked: al no estar bajo el guard, pueden tomar `purge_lock.write()` directamente, igual que `add_edge`. El split sólo hace falta para los ocho que ya lo tienen.
- Riesgo residual concreto: un `supersede` o un `bulk_import_stream` concurrente con un `add_edge` sobre el mismo nodo puede perder una de las dos aristas.
- El bug del ticket —`add_edge` público, y el camino que `ADR-0066` usa vía `ensure_lineage_edge` → `db.add_edge`, `vanta-memory/src/core/ontology/mod.rs:296`— **sí** queda cerrado.

## Contrato mecánico: población del filtro (FIND-794)

El `-E` del contrato **no se ejecuta sin medir la población** (un filtro que matchea 0 sale exit 0 y se lee verde):

```
$ cargo nextest list --profile audit -p vantadb --build-jobs 2 \
    -E 'test(graph) or test(edge) or test(lineage) or test(add_edge)'
POPULATION: 117
vantadb backend::tests::test_backend_trait_get_many_edge_cases
vantadb edge_index::tests::test_edge_index_remove_all_for_node
vantadb graph::tests::test_bfs_chain_traversal
... (55 binarios)
vantadb::graph_edge_concurrency add_edge_serializes_across_handles_over_one_engine
vantadb::graph_edge_concurrency concurrent_add_edge_loses_no_edges_on_one_node
```

**117 > 0 y > 10**: no hizo falta ampliar el filtro. Y los dos tests nuevos están **dentro** del filtro (ambos nombres contienen `add_edge`), así que el contrato cubre el fix y no sólo lo que ya existía.

## Verificación

| Comprobación | Comando | Resultado |
|---|---|---|
| El bug (RED) | `--test graph_edge_concurrency` sobre `HEAD` | **ROJO**: `164/800`, `160/800` (636 y 640 perdidas) |
| El fix (GREEN) | idem con el guard | **800/800** ✅, 3 corridas consecutivas |
| El fix es bootstrap-through (dos handles) | `add_edge_serializes_across_handles_over_one_engine` | **800/800** ✅ (falla sin el `Arc` compartido) |
| La mitad reverse está cubierta | mutante: `engine.insert(&target)` → `let _ = &target; Ok(())` | **ROJO**: `leaf nodes whose reverse half is missing or duplicated` ✅ (restaurado, `numstat` `29 0`) |
| Contrato (población medida antes) | `-E 'test(graph) or test(edge) or test(lineage) or test(add_edge)'` | **117/117** ✅ (117 listados) |
| Formato | `cargo fmt --check` | exit 0 ✅ |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | limpio, 5m47s ✅ |
| Sin dependencias nuevas | `git diff Cargo.toml Cargo.lock` | vacío ✅ |

**No verificado, declarado:** el probe de throughput fue un fichero temporal (`tests/zz_find773_probe.rs`) y **no** está en el commit; sus números son de build `dev`, no de release (ver §Techo). Tampoco se ejecutó la suite completa `--workspace` (sólo el filtro del contrato + clippy de workspace), por presupuesto de disco — carril único, 21,37 GB libres.

## Review

**P2-01 · `vanta-review`, contexto fresco, una ronda (2026-11-27).** El revisor no participó en el cambio.

**Veredicto de la ronda 1: 🔴 `changes-required`** — el **mecanismo** aprueba y está verificado; el **registro de cierre** y una aserción del test, no. 0 Critical, 3 Medium, 2 Low. Los cinco se aplicaron (abajo).

**Lo que el revisor confirmó por su cuenta, no por mi resumen:**

- **La premisa (H-07), re-medida por él:** revirtió `src/sdk/api/graph.rs` al blob exacto de `HEAD` (`git hash-object` == `rev-parse HEAD:...` == `f79084a2c`), tocó el mtime (la trampa de rebuild de Windows) y corrió los tests nuevos: `lost 640 of 800 edges on the shared node` y `two handles over one engine lost 634 of 800 edges`. Restauró el fix (SHA256 `F409D613…` idéntico al de antes del experimento), `git diff --numstat` = `29 0`, verde otra vez. **La fila no era un fantasma.**
- **Contrato:** `list` → 117 líneas de test; `run` → `117/117` en 7,18 s; los dos tests nuevos dentro del filtro. Sin trampa `FIND-794`.
- **Deadlock, verificado y no asumido:** la lista completa de llamadores de `add_edge` en el crate es `src/sdk/api.rs` (módulo `#[cfg(test)]`) más `UnifiedNode::add_edge` (otro tipo, sin locks) — ningún llamador bajo el guard, así que no hay reentrada. `sdk_purge_lock` sólo se **crea** (`init.rs:246`) y se **declara** (`engine/mod.rs:382`): **nunca se adquiere dentro de `src/storage`**, así que el orden inverso `insert_lock → purge_lock` no puede existir.
- **`engine.insert` sin CAS:** cierto, `insert.rs:156` es incondicional; el `insert_lock` se toma en `:208` y está documentado como no reentrante (`:703`, `:721`, `:919`).
- **`fmt --all --check` y `clippy --workspace --all-targets --all-features -- -D warnings`: exit 0**, re-ejecutados.
- **Retiro de la fila del Backlog:** `0 1`, sin mojibake (verificado por bytes, no por consola).

**Hallazgos y qué se hizo:**

- **M1 (aplicado) — la aserción de la mitad reverse no probaba nada.** El comentario decía que la reverse-half estaba comprobada, pero la aserción era `node_exists(leaf_id(0,0))` y los 800 leaves ya existían **antes** de que hubiera aristas: habría pasado aunque se borrase por completo la reescritura del destino (`graph.rs:170-180`). **Ahora:** se leen las aristas de los **800** leaves y cada uno debe tener exactamente **una** `reverse` hacia `HUB`; el helper `node_exists` se eliminó por quedar muerto.
  **Falsabilidad del arreglo de M1 (mutante, medido):** se sustituyó `engine.insert(&target)` por `let _ = &target; Ok(())` — es decir, se borró la mitad reverse — y el test falla con `leaf nodes whose reverse half is missing or duplicated: the target-half rewrite of add_edge is not covered by the serialization (FIND-773)`. Restaurado: `git diff --numstat src/sdk/api/graph.rs` = `29 0`, verde.
- **M2 (aplicado) — "el único llamador sin guard" era falso.** `bulk_import_stream` (`src/sdk/api/memory.rs:3259`) también llama a `remove_edge` sin guard (su último `purge_lock` está en `:2798`); verificado leyendo `:3240-3262`. Corregido en este fichero y en `core-engine.md`, y con una consecuencia útil: esos dos sitios **no** necesitan el split locked/unlocked (no están bajo el guard), así que cerrarlos son dos líneas, no una refactorización.
- **M3 (aplicado) — `status: completed` con el §Review vacío.** Era justo el gate que este fichero exige. Este bloque es el dictamen real; hasta ahora el fichero no tenía derecho a decir `completed`.
- **L1 (aplicado) — referencia colgante** `§Commit del task file` en `core-engine.md`, que no existe. Eliminada.
- **L2 (aplicado) — "las dos mitades o nada" era falso.** El guard garantiza *no interleaving*, no atomicidad: `engine.insert(&source)` (`graph.rs:168`) corre **antes** de validar el destino, así que un `target_id` inexistente deja la arista forward escrita y devuelve `NodeNotFound` sin rollback. Es **pre-existente**, no lo introduce este cambio, y ningún test cubre `add_edge` con destino ausente. Queda declarado en §Deuda; merece fila propia.

**No verificado, declarado:** el revisor no re-midió el probe de throughput (era un fichero temporal, ya borrado) y aceptó §Techo como está — ratios sobre absolutos, sin `canonical_p99` de release.

## Deuda y NOTICED BUT NOT TOUCHING

- **`remove_edge`/`ensure_edge` sin guard** — declarado arriba. Dos llamadores fuera del lock hoy (`supersede`, `bulk_import_stream:3259`) que pueden tomar el guard directamente; los otros ocho necesitan el split locked/unlocked en `memory.rs`.
- **`add_edge` no es atómico entre mitades** (pre-existente, levantado por el review como L2): `engine.insert(&source)` (`graph.rs:168`) ocurre antes de validar el destino; si el destino no existe, la arista forward queda persistida y se devuelve `NodeNotFound` sin rollback. No lo introduce este cambio (el guard sólo elimina el interleaving) y ningún test lo cubre. Candidato a fila propia: `add_edge_with_missing_target_leaves_half_edge`.
- **`supersede` no toma `purge_lock`** (`memory.rs:2449` sólo `supersede_lock`): su RMW sobre el nodo puede perder una arista frente a un `add_edge`. Pre-existe y no es de esta fila.
- **NOTICED BUT NOT TOUCHING:** `src/sdk/api/graph.rs` documenta en su cabecera que el fichero agrupa operaciones "raramente usadas fuera del path de grafo" — el `vacuum`/`pipeline` colocation sigue igual, no se toca. `insert_node` no necesita guard (no lee-modifica-escribe: sustituye el nodo entero desde el input), y `delete_node` tampoco (`engine.delete` es atómico en el engine).
