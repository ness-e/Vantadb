---
title: "FIND-521: la misma IQL devuelve filas distintas segun la estimacion de selectividad"
kind: task
status: completed
description: "El planner elegia entre ANN top-k -> post-filtro y scan -> filtro -> refine segun la selectividad estimada; el ANN post-filtrado se muere de hambre y anadir filas no relacionadas cambiaba el resultado. Ahora el orden lo decide la forma de la query, nunca la estimacion."
---

# FIND-521: el resultado de una query deja de depender del plan

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-521`)
- **Origen:** auditoria de la gramatica IQL del 2026-10-21 (`docs/dev/reviews/audit-grammar-iql-20261021.md`, F-05) — **Dep:** `FIND-313`
- **Esfuerzo:** 1-2 semanas (la fila completa) — **ejecutado:** ~1 h para el contrato de determinismo
- **Creado / cerrado:** 2026-11-27
- **Estado:** COMPLETED (contrato de determinismo) + deuda declarada del espacio de `min` con metrica no coseno

## Premisa (medida)

`src/planner.rs` elegia el orden del brazo vectorial por una estimacion:

```rust
} else if joint_selectivity < HIGH_SELECTIVITY_THRESHOLD && !sorted_filters.is_empty() {
    // scan -> filtro -> PhysicalVectorRefine (exhaustivo, coseno)
} else {
    // PhysicalVectorSearch (ANN top-k) -> PhysicalFilter  (post-filtro)
}
```

`joint_selectivity` sale de la poblacion de las estadisticas de cardinalidad, y
`PhysicalVectorSearch` trunca a `vector_candidate_window(LIMIT)` **antes** de filtrar. Una fila
que matchea y rankea por debajo de una que no matchea **nunca llega al filtro**: el resultado
pasa a depender de cuantas filas no relacionadas haya en el store, no de los datos de la query.

Censo antes del cambio (mismo filtro que el cierre):

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 \
  -E 'test(search) or test(hybrid) or test(rrf) or test(iql) or test(query)'  ->  439 tests
```

RED (`f521_selectivity_estimate_does_not_change_the_rows`): la MISMA query (`type = "doc" AND
x ~ 0.5 LIMIT 4`) sobre las MISMAS 3 filas que matchean, con y sin 30 filas no relacionadas que
rankean mas alto en el ANN:

```
left: []            right: [1, 2, 3]
```

Con las filas extra el plan tomaba el orden ANN y el post-filtro se comia la pagina entera.

## Contrato

1. **El plan decide COMO, nunca QUE.** La misma query sobre las mismas filas devuelve las
   mismas filas, sin importar la estimacion de selectividad (que se mueve con la poblacion).
2. **Una sola semantica por forma de query:** con filtro relacional ⇒ exhaustivo
   (scan → filtro → refine coseno); sin filtro ⇒ ANN.

## Cambios

| Archivo | Cambio |
|---|---|
| `src/planner.rs` | El orden del brazo vectorial ya no consulta la estimacion: `else if !sorted_filters.is_empty()` (antes `joint_selectivity < HIGH_SELECTIVITY_THRESHOLD && ...`). Se elimino la acumulacion de `joint_selectivity` (quedaba sin uso) y se documento el techo + via de upgrade. El doc de `HIGH_SELECTIVITY_THRESHOLD` aclara que hoy solo lo consume el `FilterStrategy` del SDK. |

## Tests

- `f521_selectivity_estimate_does_not_change_the_rows` — RED arriba; GREEN: `[1, 2, 3]` en los
  dos casos (conjuntos ordenados).
- No-regresion del area: `f563_intersection_holds_on_the_selective_path` (sigue con el orden
  exhaustivo), `f563_two_vector_predicates_intersect`, `f313_*`, los `rrf_*` y el resto de la
  poblacion del filtro.

## Verificacion

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  -E 'test(search) or test(hybrid) or test(rrf) or test(iql) or test(query)' --no-tests fail
-> Starting 441 tests ... Summary [17.958s] 441 tests run: 441 passed, 2582 skipped
cargo fmt --check -p vantadb                  -> exit 0
cargo clippy -p vantadb --all-targets --no-default-features \
  --features "cli,fjall,mmap2fs2roaring" -- -D warnings  -> limpio
```

## Deuda declarada (delta de la fila, escrita para no cerrar a medias)

- **Costo:** las queries vectoriales **con** filtro relacional pierden el atajo ANN (escanean la
  entidad). Es el precio de que el resultado no dependa del plan. Via de upgrade anotada en el
  codigo (`ponytail:`): ANN filtrado cuando el indice lleve bitsets de etiqueta por nodo, para
  que los predicados viajen en `query_mask` en vez de post-filtrarse. **Sin medir** (Regla 9):
  el cambio no se vende como mejora de performance y el benchmark no se corrio en este lote.
- **Espacio de score de `min`:** el hallazgo pedia ademas documentar el espacio de `min` con
  metrica no coseno (Euclidean devuelve scores `<= 0`, asi que `min = 0.88` no matchea nada en el
  path ANN). Hoy queda: **con filtro** el predicado se evalua en coseno (refine); **sin filtro**
  en la metrica configurada del indice (coseno por default). Con el indice default las dos
  coinciden; con un indice Euclidean el path ANN conserva el desajuste. **No se puede pinchar
  con test hoy**: `Config` no expone la metrica del indice, no hay seam para construir uno no
  coseno. Queda declarado, no arreglado.
