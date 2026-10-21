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
- **last-synced:** 2026-10-21T12:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 6 steps de ejecución restantes

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
| `lastAction` | (se actualiza en cada step) |
| `result` | ⬜ PENDING |
| `nextAction` | Step 1 (ver `## Steps`) |
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
security-and-hardening (cuando el hallazgo es de integridad de datos).

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 6 |
| % completado | 0% |

## Steps

1. **Verificar la evidencia** de las dos filas (sin editar): la rama vectorial de
   `planner.rs` y el `continue` que descarta el filtro con `sel >= 1.0`.
2. **FIND-554 primero** (es el mas grave y el mas chico): (a) `sel.clamp(0.0, 1.0)`; (b) el
   **mismo** denominador que el numerador (el total de `cardinality_stats`, no
   `hnsw.node_count()`); (c) "campo ausente en las stats" = **desconocido** (0.5), no 0.0 ni
   1.0; (d) **no** eliminar predicados por heuristica: solo si son estructuralmente redundantes.
   Test primero: un `WHERE` con un filtro presente en los datos cuyo `sel` estimado da >= 1.0.
3. **FIND-553**: `PhysicalVectorSearch` acepta `entity: Option<String>` y filtra por tipo
   (reutilizando la logica de `scan.rs:73-92`), o componer un `PhysicalFilter` de entidad
   sobre el brazo vectorial. Test primero: `FROM Doc WHERE content ~ "x", min = 0.5` con un
   nodo de otro tipo en el store.
4. **FIND-560** (el mismo conmutador, `sel = 0.0` cuando no hay stats): queda resuelto por el
   paso 2(c); verificalo.
5. **Medir**: el recall antes/despues (el numero va al informe del task file).
6. **Verificar**: fmt, clippy, la suite del lib completa, y los tests nuevos en verde.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
