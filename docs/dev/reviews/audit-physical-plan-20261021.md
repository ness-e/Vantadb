---
title: "Auditoria profunda de physical_plan, planner y executor - 2026-10-21"
kind: review
status: active
description: "Auditoria de solo lectura del nucleo de ejecucion de queries: physical_plan (10 operadores), planner (CBO) y executor. 20 hallazgos (4 criticos, 5 altos, 7 medios, 4 bajos): la proyeccion anula el filtro temporal, el path vectorial no scopea por entidad, el CBO puede eliminar un predicado y la reserva del governor se fuga si la query falla. Cierra 2 de los 12 modulos de FIND-503."
tags: [vantadb, audit, reviews, physical-plan, planner, executor, iql]
---

# Auditoría de `src/physical_plan/` + `src/planner.rs` + `src/executor.rs`

> **Origen:** `FIND-503` (12 módulos del core sin auditar). Informe: auditoría de solo lectura del
> 2026-10-21, sobre `develop`.
> **Alcance real:** `src/physical_plan/` (10 ficheros), `src/planner.rs`, `src/executor.rs`, y su
> contrato con `src/query.rs`, `src/cost_estimator.rs`, `src/governor.rs`,
> `src/sdk/search/**`, `src/storage/engine/**`, `src/index/search/**`.
> **Sin `cargo`** (había otro agente compilando): **todos** los hallazgos son source-level. Lo que
> necesita ejecución está marcado como *sospechado*.
> **Cubre 2 de los 12** módulos de `FIND-503` (los 10 restantes: `graphrag`, `agentic`, `shred`,
> `skills`, `tui`, `metrics`, `serialization`, `utils`, `hardware`, `backends`, `bin`).

## 1. Inventario: los operadores del plan y quién los produce

Un único productor: `src/planner.rs:165 optimize_and_compile` ← `src/executor.rs:625`. El plan se
compila **por query** ⇒ no hay estado de plan compartido entre consultas.

| Operador | Fichero | Lo pide | Origen lógico |
|---|---|---|---|
| `PhysicalScan` | `scan.rs:13` | `planner.rs:351,376,404` | `LogicalOperator::Scan` |
| `PhysicalFilter` | `filter.rs:12` | `planner.rs:332,367,378,396,407` | `FilterRelational` |
| `PhysicalTextFilter` | `filter.rs:112` | `planner.rs:353,428` | `TextFilter` (`~` sin `min`) |
| `PhysicalVectorSearch` | `vector.rs:69` | `planner.rs:345,390` | `VectorSearch` (`~` con `min`) |
| `PhysicalVectorRefine` | `vector.rs:166` | `planner.rs:383` | idem (rama CBO refine) |
| `PhysicalNestedLoopJoin` | `join.rs:25` | `planner.rs:324` | `Join` |
| `PhysicalSubqueryFilter` | `join.rs:148` | `planner.rs:416` | `SubqueryFilter` |
| `PhysicalSort` | `sort.rs:12` | `planner.rs:437` | `Sort` (`RANK BY`) |
| `PhysicalProject` | `project.rs:12` | `planner.rs:445` | `Project` (`FETCH`/`SELECT`) |
| `PhysicalLimit` | `project.rs:53` | `planner.rs:452` | `Limit` |
| `PhysicalOffset` | `offset.rs:15` | registro `"offset"` ← `planner.rs:465` | `Offset` |
| `PhysicalAggregate` | `aggregate.rs:27` | registro `"aggregate"` ← `planner.rs:465` | `Aggregate` |
| `PhysicalDedup` | `dedup.rs:25` | registro `"dedup"` ← `planner.rs:465` | `Dedup` — **sin productor IQL** (`query.rs:507`) |
| `PhysicalRrfFusion` | `planner.rs:95` | `planner.rs:365` | `PROFILE hybrid` |

Cadena reconstruida por el planner: `Scan → filtros → (vector) → text → Sort → Project → Limit →
extensiones(registry)`.

## 2. Hallazgos

### 2.1 Críticos

#### QP-01 · La proyección (`FETCH`) destruye los metadatos de los post-filtros ⇒ `AS OF` deja de filtrar

- `src/physical_plan/project.rs:31-42` — `PhysicalProject::next` **elimina** todo campo no
  proyectado (`node.relational = projected`).
- `src/executor.rs:257-264` → `filter_valid_at(nodes, as_of_ms)` se aplica **post-plan**, sobre
  nodos ya proyectados.
- `src/executor.rs:330-335` — `node_is_valid_at` lee `__vanta_valid_at_ms` /
  `__vanta_created_at_ms`; si no están, `let Some(valid_at) = valid_at else { return true; }`
  ⇒ **sin metadatos = siempre válido**.

`__vanta_valid_at_ms` y `__vanta_invalid_at_ms` no están en la lista de proyección ⇒ desaparecen ⇒
el predicado temporal se vuelve `true` para todas las filas. `FROM Doc AS OF 1500 FETCH (name)`
devuelve nodos **no válidos** en t=1500: el filtro valid-time (SCH-03 / ADR-046 §D3) queda
**anulado** sin error ni warning.

Mismo mecanismo, segundo caso: `src/executor.rs:280-303 filter_low_confidence_summaries` (solo en
`execute_query`) depende de `type` y `confidence_score`; con `FETCH (name)` el campo `type`
desaparece ⇒ el filtro de confianza de `SemanticSummary` deja de filtrar.

**Fix mínimo:** mover `filter_valid_at` / `filter_low_confidence_summaries` **dentro** del plan
(por encima del `Project`), o que `PhysicalProject` preserve una allowlist de metadata del
sistema (`__vanta_*`, `type`, `_owner_role`, `confidence_score`).

#### QP-02 · El path vectorial por defecto **no aplica el scope de entidad**

- `src/planner.rs:388-401` (rama `else`): `PhysicalVectorSearch::new(storage, query_text,
  min_score)`. **La variable `entity` se descarta**: no se construye ningún `PhysicalScan`.
- `src/physical_plan/vector.rs:88-98` — `PhysicalVectorSearch` no tiene campo de entidad.
- `src/physical_plan/vector.rs:124-137` — `index.search_nearest(&vec, None, None, &ALL_BITSET, 5, …)`
  → top-5 sobre **todo** el índice, con máscara `ALL_BITSET`.
- Contraste: `src/planner.rs:375-387` (rama refine) sí hace `PhysicalScan::new(storage, entity)`.

`FROM Doc WHERE content ~ "x", min = 0.5` devuelve los 5 vecinos más cercanos del **store entero**,
no de `Doc`. La query "equivalente" con un filtro relacional extra entra en la rama refine y **sí**
scopea ⇒ **añadir un filtro redundante cambia el conjunto de filas**.

**Fix mínimo:** `PhysicalVectorSearch` acepta `entity: Option<String>` y filtra por
`type`/namespace tras el `get_many` (misma lógica que `scan.rs:73-92`), o componer un
`PhysicalFilter` de entidad sobre el brazo vectorial.

#### QP-03 · El CBO puede **eliminar un predicado del `WHERE`** en silencio

- `src/planner.rs:287-305` — el CBO estima la selectividad de cada filtro y **descarta** los que
  dan `sel >= 1.0` (`continue`), eliminándolos del plan.
- `src/cost_estimator.rs:100-170` — `selectivity()`:
  - `:102` `total_nodes = self.storage.hnsw.load().node_count()` ← población del **índice vectorial**.
  - `:139` `freq` viene de `cardinality_stats` ← población del **KV store** (todos los nodos).
  - `:143-147` `freq / total_nodes as f32` — **sin `clamp`** ⇒ puede devolver > 1.0.
  - `:166` campo ausente de las stats + `Neq` ⇒ `1.0` ⇒ **predicado eliminado**.
  - `:165` campo ausente + `Eq` ⇒ **`0.0`** (no 0.5) ⇒ "no matchea nada" en lugar de "desconocido".

Numerador y denominador son poblaciones distintas y divergen de forma real:
`src/storage/engine/insert.rs:31-37` (`batch_should_insert_hnsw`: `InsertMode::Rebuild` ⇒ `false`,
`Auto` con batch ≥ umbral ⇒ `false`) escribe en el KV **sin** entrar al HNSW ⇒ `freq > total_nodes`
⇒ `sel > 1.0` ⇒ **el predicado desaparece del plan**. Las stats son además un snapshot de init
(`src/storage/engine/stats.rs:220-254`) que salta los registros que no deserializan (`:226-228`) y
pierden campos cuando salta el cap global (`MAX_CARDINALITY_PAIRS = 10_000`, `src/config.rs:30`).

**Fix mínimo:** (i) `sel.clamp(0.0, 1.0)`; (ii) usar el **mismo** denominador que el numerador;
(iii) **no** eliminar predicados por heurística: sólo cuando el predicado sea estructuralmente
redundante.

#### QP-04 · La reserva de memoria del governor no se libera si la ejecución falla ⇒ **DoS permanente**

- `src/executor.rs:607-610` — `governor.request_allocation(estimated_mem_cost)?` reserva en el
  **global** `ALLOCATED_BYTES` (`src/governor.rs:12`).
- `src/executor.rs:628` `physical_op.open()?` · `:630` `physical_op.next()?` · `:651`
  `physical_op.close()?` · `:652` `governor.free_allocation(...)` **sólo en el camino feliz**.

Cualquier `?` anterior retorna **sin liberar**. Cada query fallida deja su reserva cobrada para
siempre. Con `AVG_NODE_BYTES = 1024` (`src/cost_estimator.rs:28`) y 100k nodos, cada fuga ≈ 100 MB
⇒ **~20 queries fallidas** llevan `ALLOCATED_BYTES` por encima de 2 GiB ⇒
`src/executor.rs:247-254 check_memory_pressure` empieza a fallar con `Error::ResourceLimit` ⇒
**toda** query posterior se rechaza, incluidas las de escritura ⇒ el motor queda inservible hasta
reiniciar.

Interactúa con `FIND-413`/`FIND-490`: con el proveedor de embeddings caído *toda* query vectorial
falla ⇒ cada una fuga su reserva ⇒ "provider caído" se convierte en "motor muerto" tras ~20
intentos.

**Fix mínimo:** guard RAII que libere en cualquier salida (`Drop` → `free_allocation`), o
envolver open/next/close en una closure y liberar fuera.

### 2.2 Altos

#### QP-05 · Un `WHERE` con campo **cualificado** (`p.name`) no matchea nunca

`src/parser/lexer.rs:30-36` acepta `.` en un ident ⇒ `p.name` es ident válido;
`src/parser/grammar.rs:67-96` conserva el prefijo; `src/physical_plan/filter.rs:65` hace
`node.relational.get(field)` **sin** `strip_alias` ⇒ `get("p.name")` ⇒ `None` ⇒ `Eq` ⇒ `false` ⇒
fila descartada. **Inconsistencia interna:** `join.rs:210` (subquery) y `join.rs:80-81` (JOIN) **sí**
hacen `strip_alias`; el filtro relacional es el único que no. Efecto colateral: `project.rs:35`
hace `remove("p.name")` ⇒ la fila sale **con cero campos**.

**El test que dice cubrirlo no lo cubre:** `tests/logic/joins.rs:145` (comentario: `WHERE p.name =
"Alice"`) construye el AST a mano con `Condition::Relational("name", …)` (línea 161), sin
cualificar.

**Fix mínimo:** `strip_alias` en `PhysicalFilter` (o normalizar el campo en el planner).

#### QP-06 · `PhysicalSubqueryFilter` deja pasar **todas** las filas cuando el subquery no devuelve filas

`src/physical_plan/join.rs:199-208`: `None => { return self.child.next(); }` — el filtro desaparece.
En SQL, `x = (SELECT …)` con subquery vacío ⇒ `FALSE` para todas las filas ⇒ **0 resultados**.

**Fix mínimo:** `None => return Ok(None)`.

#### QP-07 · El escalar del subquery es el **primer campo alfabético**, no el campo proyectado

`src/physical_plan/join.rs:190-192`: `n.relational.values().next().cloned()` — `relational` es un
`BTreeMap` ⇒ clave alfabéticamente primera. Con `SELECT price, name FROM Item sq` el escalar es
`name` (String) frente a un `Int` ⇒ `compare_field_values` (`join.rs:233-269`) cae en `_ => false`
⇒ **0 filas**. El test existente (`tests/logic/joins.rs:467-468`) proyecta **un solo** campo ⇒ pasa
por casualidad.

**Fix mínimo:** tomar el campo de la proyección del subplan, no `values().next()`.

#### QP-08 · La cláusula `ROLE` prunea todo salvo que el registro lleve una clave `_owner_role` **no documentada**

`src/parser/grammar.rs:221` → `Query.owner_role` → `src/query.rs:628` → `LogicalPlan.enforce_role`;
`src/executor.rs:634-646` descarta todo nodo sin `relational["_owner_role"]` (salvo
`required_role == "admin"`). **`grep _owner_role` en `src/`: ningún escritor** — solo un benchmark
(`benches/high_density.rs:113`); los nodos de memoria lo reciben si el usuario lo pone a mano en
`metadata` (`src/sdk/serialization/mod.rs:621-623`). `docs/api/IQL.md:108` documenta `ROLE` como
"RBAC owner role filter" sin mencionar la clave.

`FROM Doc ROLE "user"` sobre nodos sin `_owner_role` ⇒ **0 filas, sin error**. No es un bypass
(fail-closed) pero es un resultado vacío silencioso. **Agravante:** `PhysicalProject` elimina
`_owner_role` ⇒ con proyección, `ROLE` **siempre** devuelve 0 filas.

**Fix mínimo:** documentar `_owner_role` y tratar "nodo sin `_owner_role`" como *pasa* o como
*error*; y preservarlo en la proyección (ver QP-01).

#### QP-09 · El conmutador entre los dos paths vectoriales depende de una estimación que puede ser `0.0`

`src/planner.rs:296-305` — `joint_selectivity` = producto de selectividades **estimadas**;
`src/planner.rs:373` — `joint_selectivity < 0.1` ⇒ refine vs HNSW;
`src/cost_estimator.rs:165-168` — campo **ausente** de las stats: `Eq` ⇒ **0.0**, `Neq` ⇒ 1.0,
rango ⇒ 0.5.

`WHERE content ~ "x", min = 0.5 AND status = "active"`: si `status` está en las stats ⇒ `sel = 0.3`
⇒ path HNSW (k=5, métrica del índice, **sin scope** — QP-02); si **no** está (cap de 100
valores/campo `stats.rs:234`, cap global 10 000, o campo evictado) ⇒ `sel = 0.0` ⇒ path refine
(fuerza bruta, coseno, **con scope**). El mismo dataset produce dos conjuntos de filas distintos
según un artefacto de estadísticas interno que puede cambiar al reiniciar, al evictar un campo o
al insertar un valor.

**Fix mínimo:** no decidir la forma del plan por una heurística de cardinalidad ausente; tratar
"sin stats" como `unknown` (0.5).

### 2.3 Medios

| # | Hallazgo | Evidencia |
|---|---|---|
| **QP-10** | **Materialización sin cap** (viola `memory-budget.md` R-3/R-4): `PhysicalScan::prefetched` = N × ~9 KB (campos + vector 1536d) **más** la partición completa del backend en RAM; `Sort`, `Aggregate` (grupos sin cap), `Join` (lado derecho completo), `RrfFusion` y `Dedup` (`HashSet` sin cap) hacen lo mismo. 100k nodos ≈ **0.9 GB** por query; 500k ≈ 4.5 GB | `scan.rs:37-61`, `sort.rs:44-46`, `aggregate.rs:139-190`, `join.rs:67-71`, `planner.rs:118-130`, `dedup.rs:31` |
| **QP-11** | La estimación del governor **subestima el pico ~9×**: `AVG_NODE_BYTES = 1024` vs el nodo real (6–9 KB) ⇒ el guard de 2 GiB no dispara donde debe | `cost_estimator.rs:28,206` |
| **QP-12** | Sólo la **última** condición vectorial del `WHERE` sobrevive al plan (`vector_search = Some(...)` sobrescribe; `text_matches.push(...)` sí acumula) ⇒ `WHERE a ~ "x", min = 0.5 AND b ~ "y", min = 0.6` pierde la condición sobre `a` | `planner.rs:239-245` vs `:246-248` |
| **QP-13** | `text_contains_query` **descarta los términos sueltos** cuando la query contiene alguna frase entrecomillada (`plan.phrases` no vacío ⇒ los `terms` no se comprueban) ⇒ `WHERE content ~ '"frase exacta" suelto'` ignora "suelto" | `src/text_index.rs:508-520` |
| **QP-14** | El plan no puede expresar los filtros del contrato del SDK (`as_of_ms`, `valid_window`, `min_confidence`, `include_quarantined`, `exclude_superseded`) ⇒ `db.query("SELECT * FROM ns")` devuelve registros **en cuarentena, supersedidos o fuera de la ventana** que `db.list`/`db.search` sí ocultan. Mismo dato, dos respuestas según la API | `sdk/search/page.rs:302-374` vs `executor.rs:256-276` |
| **QP-15** | `check_memory_pressure()` del executor es un **no-op** que no mide RSS (compara un contador contra un literal de 2 GiB, no `Config::memory_limit`); el guard real de RSS está en la capa de storage | `executor.rs:247-254`, `governor.rs:44-68`, `storage/engine/stats.rs:195-199` |
| **QP-16** | **N+1 de I/O**: `PhysicalScan` llama `storage.is_deleted(node.id)` por **cada** nodo ⇒ un `backend.get(Tombstones, key)` por fila (100k filas ⇒ 100k lookups) | `scan.rs:69`, `storage/engine/delete.rs:316-322` |

### 2.4 Bajos

| # | Hallazgo | Evidencia |
|---|---|---|
| **QP-17** | `evaluate_condition`: "campo ausente" (`Neq` ⇒ **true**), "campo Null" (`Neq` ⇒ **false**) e "incompatible" (`_ => false`) se contradicen; en SQL los tres son NULL ⇒ la fila se descarta. El test `physical_plan/mod.rs:224-232` **fija** el comportamiento ausente+`Neq` ⇒ true | `filter.rs:96-105` |
| **QP-18** | `WITH TEMPERATURE` no tiene efecto observable: `apply_temperature_limits` sólo toca `Traverse.max_depth` y `Traverse` **se ignora** en el planner | `governor.rs:76-87`, `planner.rs:258-262` |
| **QP-19** | La fila agregada sintética usa `id = 0` y sale por `db.query()` como un `NodeRecord` con id 0 (0 es el centinela de auto-asignación) | `aggregate.rs:179`, `sdk/api/graph.rs:317-322` |
| **QP-20** | `self.storage.vector_store[0]` sin guarda (panic latente) — **sospechado**, no alcanzable en el flujo normal | `vector.rs:127` |

## 3. Invariantes del executor

**Se respetan:** el `WHERE` relacional se aplica al child (`planner.rs:331-409`, salvo que el CBO
lo elimine — QP-03) · `LIMIT`/`OFFSET` = skip-then-take (`query.rs:557-569`, `offset.rs:42-50`) ·
`AS OF` se aplica (`executor.rs:257-264`, **pero se anula con `FETCH`** — QP-01) · `ROLE` se aplica
(fail-closed; vacía la query salvo `_owner_role` — QP-08) · veto de `Conflict#`
(`executor.rs:613-622`) · el error de embedding llega al usuario (`vector.rs:119-122`).

**NO se respetan:** `AS OF` con `FETCH` → no-op (QP-01) · el filtro de confianza de
`SemanticSummary` se aplica en `execute_query` pero no en `execute_select` ⇒ `FROM` y `SELECT`
divergen sobre el mismo dato · `LIMIT` antes de `AS OF` ⇒ una página puede quedar corta
(documentado en `IQL.md:244-247`) · `include_quarantined`, `exclude_superseded`,
`min_confidence`, `valid_window` no existen en el plan (QP-14) · `_owner_role` se pierde con
`FETCH` · los campos cualificados en `WHERE`/`FETCH` (QP-05).

## 4. Dónde se incumple el budget de memoria

`.opencode/rules/memory-budget.md` R-3 (back-pressure antes del OOM) y R-4 (cap declarado en toda
estructura residente dependiente del dataset).

| Estructura | Fichero:línea | Cap | Escala |
|---|---|---|---|
| `PhysicalScan::prefetched` | `scan.rs:19,44,60` | ❌ | **N × ~9 KB** + la partición completa del backend |
| `PhysicalSort::nodes` | `sort.rs:20,44-46` | ❌ | N × ~9 KB |
| `PhysicalAggregate::groups` | `aggregate.rs:139-140` | ❌ | cardinalidad de grupos |
| `PhysicalNestedLoopJoin::right_nodes` | `join.rs:35,68-71` | ❌ | \|RHS\| × ~9 KB |
| `PhysicalRrfFusion::fused` | `planner.rs:98,118-138` | ❌ | (brazos) × ~9 KB |
| `PhysicalDedup::seen` | `dedup.rs:31` | ❌ | valores distintos del campo |
| `PhysicalVectorRefine` | `vector.rs:166` | ✔ streaming | — |
| `PhysicalVectorSearch::prefetched` | `vector.rs:79` | ✔ ≤ k=5 | — |

Un `SELECT * FROM ns` sobre N nodos ≈ **N × 9 KB** de pico (más el escaneo del backend). Con 100k
nodos ⇒ **~0.9 GB por query**; con 500k ⇒ **~4.5 GB** — por encima del límite de 2 GiB del
governor, que además subestima el pico ~9× (QP-11). Combinado con QP-04, el margen es menor aún.

## 5. Lo que no se pudo verificar

1. **Nada se ejecutó** (0 invocaciones de `cargo`): todos los hallazgos son source-level. QP-01,
   QP-02, QP-04, QP-05 y QP-06 merecen un test dirigido.
2. **QP-03**: confirmado por lectura que el clamp no existe y que las poblaciones pueden divergir
   (`InsertMode::Rebuild`); *sospechado* que `sel > 1.0` sea alcanzable en el flujo por defecto. La
   divergencia `Neq` → 1.0 con campo ausente **sí** es alcanzable por lectura.
3. **QP-09**: confirmado por lectura (`cost_estimator.rs:165` → 0.0); no verificado con un dataset
   real donde un campo esté y otro no.
4. **QP-20**: sospechado.
5. No se auditó en profundidad `src/parser/**` (fuera de alcance), `src/index/**` (más allá de
   `search_nearest` y `distance/mapper.rs`) ni `src/sdk/search/**` (sólo el contrato de filtros).

## 6. Cobertura de tests

**Sin ningún test:** `PhysicalNestedLoopJoin`, `PhysicalSubqueryFilter`, `PhysicalTextFilter`
(ninguno de los tres tiene `mod tests`), el flag `FETCH` × `AS OF`, el flag `ROLE`, y el JOIN con
`WHERE` cualificado **vía el parser** (el test que lo simula construye el AST a mano sin
cualificar, `tests/logic/joins.rs:161`).

**Tests que fijan un comportamiento que no es el contrato:**
`src/physical_plan/mod.rs:224-232` ("campo ausente + `!=` ⇒ la fila pasa", contradice SQL) y
`src/planner.rs:630-679` (`optimize_and_compile_eliminates_identity_filter` **blinda** el CBO que
elimina filtros con `sel >= 1.0` — es decir, el test legitima QP-03).

## 7. Estado de los hallazgos de la campaña del 2026-10-20

| ID | Estado |
|---|---|
| `FIND-313` (k=5 hardcoded) | ✅ CONFIRMADO (`vector.rs:128`) |
| `FIND-521` (dos paths vectoriales) | ✅ CONFIRMADO y **AGRAVADO**: el path HNSW no scopea (QP-02), el conmutador depende de una estimación que puede ser 0.0 (QP-09) y el CBO puede eliminar el predicado (QP-03) |
| `FIND-527` (`candidate_k` ignorado) | ✅ CONFIRMADO (`planner.rs:274-277`) |
| `FIND-520` (`~` degrada a léxico) | ✅ CONFIRMADO + QP-13 (la rama de frases descarta los términos sueltos) |
| `FIND-519` / `FIND-518` (trailing / `WHERE` mal formado) | ✅ CONFIRMADOS, no corregidos; QP-03 agrava: un `WHERE` con dos condiciones puede dejar una sola y el CBO eliminarla |
| `FIND-413` (error de embedding) | ✅ CONFIRMADO; QP-04 lo convierte en "motor inservible" tras ~20 queries |
| `FIND-490` (el fallo del brazo vectorial tumba la híbrida) | ✅ CONFIRMADO (`planner.rs:344-349`) |
| `FIND-522` (`min = nan`/`inf`) | ✅ CONFIRMADO (efecto) |
| `FIND-316` (agregaciones avanzadas) | Vigente; `PhysicalAggregate` sin cap de grupos (QP-10) |
| `FIND-531` (doc vs código) | Vigente; se refuerza con QP-14 y QP-08 |
| `FIND-503` (12 módulos sin auditar) | 🟡 Parcialmente cerrada: este informe cubre 2 de los 12 |
