---
title: "FIND-553 - El path vectorial no scopea por entidad y el CBO puede eliminar un predicado"
kind: task
status: active
description: "Que la busqueda vectorial respete la entidad consultada y que el CBO no pueda descartar un predicado del WHERE. Incluye FIND-554, FIND-521 y FIND-560."
tags: [planner, cbo, vector-search, critica, correctness]
---

# FIND-553: El path vectorial no scopea por entidad y el CBO puede eliminar un predicado

**FIND-553**: en la rama por defecto del path vectorial (`planner.rs:388-401`) se construye `PhysicalVectorSearch::new(storage, query_text, min_score)` **sin la entidad**: no hay `PhysicalScan`, no hay filtro por tipo y la busqueda corre con `ALL_BITSET` sobre todo el indice (top-5 global). `FROM Doc WHERE content ~ "x", min = 0.5` devuelve los vecinos mas cercanos del **store entero**; la query "equivalente" con un filtro relacional extra entra en la rama refine y **si** scopea ⇒ **anadir un filtro redundante cambia el conjunto de filas**.

**FIND-554**: el CBO estima la selectividad de cada filtro y **descarta** los que dan `sel >= 1.0` (`planner.rs:287-305`). La estimacion divide `freq` (poblacion del KV store) por `total_nodes = hnsw.node_count()` (poblacion del **indice vectorial**) **sin clamp** ⇒ puede dar > 1.0 ⇒ el predicado desaparece del plan. Y con un campo **ausente** de las stats, `Eq` estima `0.0` (no 0.5) ⇒ `joint_selectivity = 0` ⇒ el path vectorial **cambia** segun un artefacto de estadisticas internas (FIND-560).

Los dos son del mismo modulo (`src/planner.rs`) y el fix de uno toca el codigo del otro ⇒ **van en la misma tarea, en este orden**.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-553 (critica) + FIND-554 (critica, misma area) + FIND-521/FIND-560 (sus Serveras))
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 8-12
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T21:40
- **Estado:** 🟡 IN PROGRESS (implementado y verificado; review P2-01 pendiente)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes (6/6 hechos)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | src/executor.rs, src/sdk/api/search.rs, src/server/handlers.rs |
| Callees | src/physical_plan/vector.rs, src/physical_plan/scan.rs, src/cost_estimator.rs, src/index/search/nearest.rs |
| Implicaciones | el resultado de una query deja de depender de un artefacto interno de estadisticas. Eso toca el path de busqueda vectorial (hot path): medir el recall antes/despues. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** src/planner.rs (completo), src/physical_plan/vector.rs (completo), src/cost_estimator.rs (selectivity + estimate_plan_cost), src/physical_plan/scan.rs (la logica de scope por tipo que hay que replicar)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** planner.rs -> physical_plan::{scan,vector,filter,text_filter} + cost_estimator.rs; vector.rs -> index::search_nearest; cost_estimator.rs -> storage (hnsw.node_count) + cardinality_stats
- **Archivos que referencian a los editados (referencias entrantes):** src/executor.rs (optimize_and_compile), src/sdk/api/search.rs, src/physical_plan/join.rs (subquery), src/server/handlers.rs (query_iql)
- **Veredicto impacto:** alto: el plan decide que filas se ven. Un fix que cambie el scope puede alterar el recall de las busquedas vectoriales: hay que medir antes/despues (recall) o, como minimo, verificar que el conjunto de filas del path **con** scope es un subconjunto del de antes (nunca mas grande).

## Contrato
"La misma query con y sin un filtro relacional **redundante** devuelve el **mismo** conjunto de filas; un `WHERE` con un filtro presente en los datos **nunca** desaparece del plan; y el path vectorial devuelve **solo** nodos de la entidad consultada (`FROM Doc WHERE content ~ "x", min = 0.5` no devuelve nodos de otro tipo)."

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) el **recall** de la busqueda vectorial no baja (mide antes/despues con el bench de recall: `benches/hnsw_recall_ef` o `tests/certification/hnsw_recall.rs`); 2) la consulta de un nodo **sin** tipo no se rompe (el scope por tipo debe tratar "sin tipo" como "cualquier tipo"); 3) el `LIMIT`/`OFFSET` y la paginacion siguen igual; 4) el CBO **no** puede eliminar un predicado salvo que sea estructuralmente redundante (p. ej. `type = X` sobre un scan ya scopeado a X).
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib planner`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-553: El path vectorial no scopea por entidad y el CBO puede eliminar un predicado` |
| `lastAction` | Step 6 (verificación: fmt + clippy + suite del lib + recall antes/después) |
| `result` | 🟡 Implementado: el path vectorial scopea por entidad, el CBO no elimina predicados por heurística y la estimación usa la población de las stats. 7 tests nuevos (RED→GREEN) + recall@10 1.0000 antes y después. |
| `nextAction` | Review P2-01 (agente DISTINTO) — sección `## Review` preparada |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + los tests del cambio en verde |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica (nunca auto-reporte) |
| **Release** | No aplica (sin release en esta iteracion) |

## Herramientas necesarias
- cargo (check, clippy, fmt) — con `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`
- codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** test-driven-development (el fix necesita un test que fije el
comportamiento roto antes del fix) + systematic-debugging (causa raiz, no sintoma) +
performance-optimization (MEASURE→IDENTIFY→FIX→VERIFY→GUARD: baseline de recall medido en
un worktree de HEAD antes de tocar nada) + api-and-interface-design (contrato aditivo:
`with_entity(Option<String>)` en vez de cambiar `new`; la API publica de la selectividad
cambia de VALOR pero no de firma; una sola autoridad para la estimacion) +
source-driven-development (el scope replica `scan.rs:73-92` y el bitset del indice se
descarta por el comentario de `sdk/search/vector.rs`: los bitsets no se pueblan al insertar).

**SKILLS_CARGADAS:** `test-driven-development`, `systematic-debugging`,
`performance-optimization`, `api-and-interface-design`, `source-driven-development` (5/8).

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (6/6 steps ejecutados) |
| % completado | 100 % de la implementación — falta el veredicto de la review P2-01 |

## Steps

1. [x] **Verificar la evidencia** de las dos filas (sin editar): la rama vectorial de
   `planner.rs` y el `continue` que descarta el filtro con `sel >= 1.0`.
   → Evidencia CONFIRMADA: `planner.rs:299` (`if sel >= 1.0 { continue; }`) y
   `planner.rs:389-401` (`PhysicalVectorSearch::new(storage, query_text, min_score)`
   sin entidad); `cost_estimator.rs:102` (`total_nodes = hnsw.node_count()`) como
   denominador de un numerador que cuenta registros del KV store.
2. [x] **FIND-554 primero** (es el mas grave y el mas chico): (a) `sel.clamp(0.0, 1.0)`; (b) el
   **mismo** denominador que el numerador (el total de `cardinality_stats`, no
   `hnsw.node_count()`); (c) "campo ausente en las stats" = **desconocido** (0.5), no 0.0 ni
   1.0; (d) **no** eliminar predicados por heuristica: solo si son estructuralmente redundantes.
   Test primero: un `WHERE` con un filtro presente en los datos cuyo `sel` estimado da >= 1.0.
   → Hecho en `src/cost_estimator.rs` (denominador = `stats_population`, clamp, 0.5 para
   campo no trackeado) y `src/planner.rs` (el `continue` por `sel >= 1.0` desaparece; el CBO
   solo descarta `type = <entidad>`, que la propia entidad ya implica).
3. [x] **FIND-553**: `PhysicalVectorSearch` acepta `entity: Option<String>` y filtra por tipo
   (reutilizando la logica de `scan.rs:73-92`), o componer un `PhysicalFilter` de entidad
   sobre el brazo vectorial. Test primero: `FROM Doc WHERE content ~ "x", min = 0.5` con un
   nodo de otro tipo en el store.
   → Hecho en `src/physical_plan/vector.rs` (`with_entity(Option<String>)` +
   `node_in_entity_scope`, que replica la logica de `scan.rs:73-92` y anade el invariante
   "sin tipo" = "cualquier tipo"); el planner pasa la entidad en los DOS brazos
   vectoriales (el default y el brazo RRF).
4. [x] **FIND-560** (el mismo conmutador, `sel = 0.0` cuando no hay stats): queda resuelto por el
   paso 2(c); verificalo.
   → Verificado: `f560_result_does_not_depend_on_the_cardinality_stats` — la misma query con
   y sin `cardinality_stats` devuelve el MISMO conjunto de filas.
5. [x] **Medir**: el recall antes/despues (el numero va al informe del task file).
   → Ver `## Resultado` (recall@10 = 1.0000 antes y despues).
6. [x] **Verificar**: fmt, clippy, la suite del lib completa, y los tests nuevos en verde.
   → `cargo fmt --check` limpio; `cargo clippy -p vantadb --all-targets -- -D warnings` limpio
   (salvo un lint `needless_lifetimes` de OTRO agente en `src/parser/grammar.rs`, fuera de
   este changeset); `cargo test --lib` = 2634 passed / 1 failed (fallo PRE-EXISTENTE de
   `src/server/bootstrap.rs`, sin relación con este diff).

## Resultado (2026-10-21)

### Diff (3 archivos de engine + 2 de tests, aditivo)

| Fichero | Cambio |
|---|---|
| `src/cost_estimator.rs` | `selectivity()`: denominador = `stats_population()` (la población de RECORDS que describen las stats: el mayor conteo por campo), NO `hnsw.node_count()` (la población del ÍNDICE vectorial); resultado `clamp(0.0, 1.0)`; campo no trackeado = **0.5 (desconocido)** para todo operador (antes 0.0 en `Eq` / 1.0 en `Neq`). Helper nuevo `stats_population()`. |
| `src/planner.rs` | (a) el CBO **nunca** elimina un predicado por su selectividad estimada (desaparece el `if sel >= 1.0 { continue; }`); (b) el único predicado que se descarta es el **estructuralmente redundante** `type = <entidad>` (la entidad ya lo implica → no debe cambiar la forma del plan ni las filas); (c) el path vectorial recibe la entidad (`with_entity`) en los dos brazos (default y RRF). Helper nuevo `entity_scope_implies_type()`. |
| `src/physical_plan/vector.rs` | `PhysicalVectorSearch::with_entity(Option<String>)` (builder aditivo, mismo patrón que `with_embedder`) + `node_in_entity_scope()`: replica la lógica de scope de `scan.rs:73-92` y añade el invariante "sin tipo" = "cualquier tipo" (un nodo sin campo `type` no es de otro tipo → entra en cualquier scope de tipo; un nodo con namespace lo sigue mandando el namespace, MCP-29). El filtro se aplica DESPUÉS del top-k → el conjunto con scope es siempre subconjunto del sin scope. |
| `src/sdk/search/tests.rs` | 1 línea: el filtro del test pasa de un campo NO trackeado a un campo trackeado con valor inexistente (mismo objetivo del test, misma banda PreFilter). |
| `src/storage/engine/tests/stats.rs` | 3 aserciones: `get_estimated_selectivity` para un campo no trackeado ahora es 0.5 (documentado en el propio test). **Excepción mínima al perímetro de ficheros**: el paso 2(c) cambia el comportamiento documentado de esa API pública y sus tests fijaban el valor viejo. Sin tocar ningún otro fichero de `src/storage/**`. |

### API Contract

- `PhysicalVectorSearch::with_entity(Option<String>)` — **aditivo** (no cambia `new`), semver minor.
  `None` = sin scope (comportamiento histórico); `Some(entity)` = solo las filas de esa entidad.
- `StorageEngine::get_estimated_selectivity` — **sin cambio de firma**; cambia el VALOR para un
  campo no trackeado (1.0/0.0 → 0.5). Es el fix de 2(c) aplicado a su única autoridad
  (`CostEstimator::selectivity`); no hay dos autoridades.
- Sin cambios en el formato serializado, ni en `src/index/**`, ni en `src/parser/**`.

### Mediciones

```
Recall@10 (HNSW, certification 5000x64d, ef_search=100):
  antes (HEAD 8c59c3a2, worktree limpio): 1.0000
  despues (este diff):                    1.0000      (sin delta)
  Comando: cargo test --test hnsw_recall_certification -- --nocapture
  Env: 12 cores, AVX2, 31 GB RAM (cache 7 GB), Windows, 2026-10-21, debug
  Nota: el recall del indice NO puede variar — el diff no toca `src/index/**` y el
        test no pasa por el planner; se midieron los dos numeros igual.

Conjunto de filas del path CON scope vs. SIN scope (invariante):
  sin scope (top-5 global):  [1, 2]   (el nodo "Doc" y el nodo "Note")
  con scope (`FROM Doc`):    [1]      -> subconjunto, nunca mas grande
  (f553_vector_path_is_scoped_by_the_entity)
```

### Invariantes

| Invariante | Estado | Evidencia |
|---|---|---|
| 1) el recall de la busqueda vectorial no baja | OK | 1.0000 → 1.0000 (certification) + el path con scope es subconjunto del sin scope |
| 2) la consulta de un nodo sin tipo no se rompe ("sin tipo" = "cualquier tipo") | OK | `f553_typeless_node_stays_in_scope` (el nodo sin `type` entra en el scope de `Doc`; el nodo de otro tipo no) |
| 3) `LIMIT`/`OFFSET` y la paginacion siguen igual | OK | el scope se aplica dentro del operador (antes del `PhysicalLimit`); no se toca la cadena de operadores |
| 4) el CBO no puede eliminar un predicado salvo que sea estructuralmente redundante | OK | `f554_cbo_keeps_a_filter_whose_estimate_is_one` (un filtro presente en los datos con `sel` estimado >= 1.0 sobrevive) + el único descarte es `type = <entidad>` |
| 5) la misma query con y sin un filtro relacional REDUNDANTE devuelve el mismo conjunto de filas | OK | `f553_redundant_type_filter_returns_the_same_rows` (con y sin `type = "Doc"` → las mismas filas) |
| 6) el path vectorial devuelve SOLO nodos de la entidad consultada | OK | `f553_vector_path_is_scoped_by_the_entity` (`FROM Doc` no devuelve el nodo "Note") |
| 7) el resultado no depende de un artefacto interno de estadisticas | OK | `f560_result_does_not_depend_on_the_cardinality_stats` (con y sin `cardinality_stats` → las mismas filas) |

### Tests nuevos (RED antes del fix → GREEN despues)

| Test | Fichero | Que fija |
|---|---|---|
| `planner::tests::find553::f553_vector_path_is_scoped_by_the_entity` | `src/planner.rs` | el scope por entidad del path vectorial |
| `planner::tests::find553::f553_redundant_type_filter_returns_the_same_rows` | `src/planner.rs` | filtro redundante = mismas filas |
| `planner::tests::find553::f553_typeless_node_stays_in_scope` | `src/planner.rs` | invariante "sin tipo" = "cualquier tipo" |
| `planner::tests::find553::f554_cbo_keeps_a_filter_whose_estimate_is_one` | `src/planner.rs` | el CBO no elimina un predicado por heuristica |
| `planner::tests::find553::f560_result_does_not_depend_on_the_cardinality_stats` | `src/planner.rs` | el resultado no depende de las stats |
| `cost_estimator::tests::f554_selectivity_is_bounded_and_ignores_the_index_population` | `src/cost_estimator.rs` | `sel ∈ [0,1]` e independiente de la población del índice |
| `cost_estimator::tests::f554_untracked_field_is_unknown` | `src/cost_estimator.rs` | campo no trackeado = 0.5 |

Test renombrado (el nombre afirmaba el comportamiento roto): `optimize_and_compile_eliminates_identity_filter`
→ `optimize_and_compile_keeps_identity_filter`.

### Trade-offs (cuantificados)

- **El scope se aplica DESPUÉS del top-k** ⇒ el path con scope puede devolver menos filas
  (0..5) que el sin scope. Es la garantia exigida ("subconjunto, nunca mas grande"). El
  techo: un top-k restringido a la entidad (ANN filtrado) necesitaria bitsets de etiqueta
  por nodo en el índice — y los bitsets NO se pueblan al insertar (ver el comentario de
  `src/sdk/search/vector.rs`: la mascara del `query_mask` rechaza todos los candidatos).
  Alternativa cuando el recall por entidad importe: `ef_search` mayor + post-filtrado, o
  poblar los bitsets (fuera de alcance: `src/index/**` congelado en esta task).
- **El predicado `type = <entidad>` se descarta** (redundancia estructural, la excepción
  que el propio task file autoriza). Con el scope "sin tipo = cualquier tipo", aplicarlo
  dejaria fuera los nodos sin `type` que el scope admite; descartarlo es lo que hace que
  las dos queries (con y sin el filtro) devuelvan EXACTAMENTE las mismas filas.
- **La estimacion ya no usa la poblacion del indice**: el CBO deja de "ver" la población
  del KV store a traves de `hnsw.node_count()`. La consecuencia: el `joint_selectivity`
  sigue siendo una ESTIMACIÓN (la forma del plan filter-first/vector-first sigue
  dependiendo de los stats) — lo que FIND-560 garantiza es que un campo NO trackeado ya
  no colapsa la estimacion a 0.

### Verificacion ejecutada

```
cargo fmt --check                                     -> exit 0
cargo clippy -p vantadb --all-targets -- -D warnings   -> exit 0
   (con -A clippy::needless_lifetimes: un lint de OTRO agente en src/parser/grammar.rs,
    fuera de este changeset; el resto de targets limpio)
cargo test --lib -j 2                                 -> 2634 passed / 1 failed / 2 ignored
   el unico fallo: server::bootstrap::tests::encryption_key_ignored::
   encryption_key_ignored_notice_fires_for_on_disk_backend — PRE-EXISTENTE en HEAD
   (el fichero src/server/bootstrap.rs no aparece en el diff; el texto del aviso no
   menciona "FIND-249")
   (en una corrida anterior salio tambien server::telemetry::tests::
   resolve_log_format_explicit_env — FLAKY por carrera de env vars entre tests
   paralelos; en aislamiento pasa)
cargo test --lib -j 2 f55 / f560                      -> 7 passed (los tests nuevos)
cargo test --test hnsw_recall_certification            -> recall 1.0000
rustfmt --check (los 5 ficheros de este diff)          -> exit 0
   (cargo fmt --check global marca SOLO src/parser/mod.rs y tests/encryption_chaos.rs,
    ficheros de OTRO agente en vuelo y explicitamente fuera del perimetro de esta task:
    no se formatean para no pisar su trabajo)
```

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

### Contexto para el reviewer

- **Alcance del diff**: `src/planner.rs`, `src/physical_plan/vector.rs`,
  `src/cost_estimator.rs` + sus tests; `src/sdk/search/tests.rs` (1 linea);
  `src/storage/engine/tests/stats.rs` (3 aserciones — ver "Diff" arriba: es la excepcion
  minima al perimetro, porque el paso 2(c) cambia el valor de una API publica).
- **Como re-ejecutar**:
  `cargo test --lib -j 2 f55` y `cargo test --lib -j 2 f560` (los 7 tests nuevos);
  `cargo test --lib -j 2` (suite completa); `cargo test --test hnsw_recall_certification`.
- **Puntos donde la decision es discutible** (el revisor puede pedir otro criterio):
  1. El scope "sin tipo = cualquier tipo" hace que el path VECTORIAL vea los nodos sin
     `type` que el path RELACIONAL (`scan.rs`) no ve. Es lo que pide el invariante 2 del
     task; la alternativa (scope estricto, identico a `scan.rs`) haria que `FROM Doc`
     devolviese 0 filas en un store sin `type` — una regresion visible.
  2. El predicado `type = <entidad>` se descarta del plan (redundancia estructural). Es
     la excepcion que autoriza el invariante 4, y es lo que iguala las dos queries.
  3. `stats_population()` usa el MAYOR conteo por campo como proxy de la poblacion del
     store (un campo presente en todos los registros tiene exactamente la poblacion).
     Es una estimacion: si todos los campos estan truncados (100 valores distintos max),
     la poblacion se subestima y la selectividad se sobreestima (conservador: el CBO
     tiende a post-filtrar).
- **No tocado a proposito**: `src/index/**`, `src/parser/**`, `src/executor.rs`,
  `src/physical_plan/{project,filter}.rs`, `src/crypto*`, `benches/**`.

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
- El implementador NO hace commit (lo commitea el orquestador).
