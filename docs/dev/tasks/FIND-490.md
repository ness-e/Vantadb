---
title: "FIND-490: un fallo del brazo vectorial tumba la query hibrida entera"
kind: task
status: completed
description: "En la fusion RRF del planner, un brazo vectorial que no abre (provider caido, indice ausente) propagaba el error y perdia tambien el brazo lexico. Ahora el brazo fallido se degrada, la query sigue con los demas y la degradacion se publica (WARN con brazo y motivo + contador rrf_arm_failure_stats). Con todos los brazos caidos la query sigue fallando."
---

# FIND-490: la hibrida no pierde los resultados lexicos si cae el brazo vectorial

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-490`)
- **Origen:** auditoria profunda v0.9.0 (2026-10-20) + review adversarial de `FIND-413`/`FIND-414`
- **Esfuerzo:** 2-3 dias (la fila) — **ejecutado:** ~1 h
- **Creado / cerrado:** 2026-11-27
- **Estado:** COMPLETED

## Premisa (medida)

`src/planner.rs` (fusion RRF, WIRE-08) abria cada brazo con `arm.open()?`:

```rust
for arm in &mut self.arms {
    arm.open()?;          // un provider caido aborta la fusion entera
    ...
}
```

Censo antes del cambio (mismo filtro que el cierre):

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 \
  -E 'test(search) or test(hybrid) or test(rrf) or test(iql) or test(query)'  ->  439 tests
```

RED (`f490_vector_arm_failure_degrades_to_the_lexical_arm`, embedder que devuelve `Err`):

```
panicked at src/planner.rs:1734: a dead vector arm must not fail the hybrid query (FIND-490):
Generic(ChainedError { msg: "embedding provider down (FIND-490 test)" })
1 passed, 1 failed        (el segundo test ya pasaba: no habia brazo que sobreviviera)
```

## Contrato

1. **Degradar, no tumbar:** si un brazo falla y **otro sobrevive**, la query sigue con los
   demas y devuelve su respuesta parcial.
2. **Nunca silenciosa:** cada brazo degradado publica **brazo + motivo** — `tracing::warn!`
   con `op="rrf_fusion"`, `arm`, `error`, mas el contador `planner::rrf_arm_failure_stats()`.
   Un RRF con un brazo de menos devuelve peores resultados y el llamador debe poder saberlo.
3. **No mentir con 0 hits:** si **todos** los brazos fallan, la query falla con el error del
   brazo (no devuelve un fusion vacio).

## Cambios

| Archivo | Cambio |
|---|---|
| `src/planner.rs` | `PhysicalRrfFusion` degrada el brazo que no abre (`note_arm_failure`), fusiona el resto y devuelve `Err` solo cuando `failures == arms.len()`. Campo `labels` + `with_arm_labels` para nombrar el brazo en el log; el call site pasa `["vector", "lexical"]`. Contador `RRF_ARM_FAILURES` + `rrf_arm_failure_stats()` / `reset_rrf_arm_failure_stats()` (patron TEC-04 de `sdk/search/vector.rs`). |

## Tests

- `f490_vector_arm_failure_degrades_to_the_lexical_arm` — RED arriba; GREEN: `open()` Ok,
  fila `1` del brazo lexico, contador `= 1`.
- `f490_all_arms_failing_stays_an_error` — dos brazos caidos ⇒ `open()` devuelve el motivo
  del brazo (`arm unavailable`), nunca cero filas.
- `rrf_cbo_fuses_arms_when_profile_is_present` — se le agrego la no-regresion: con los dos
  brazos sanos el contador queda en `0` y la fusion devuelve la misma fila que en `HEAD`.

## Verificacion

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  -E 'test(search) or test(hybrid) or test(rrf) or test(iql) or test(query)' --no-tests fail
-> Starting 441 tests ... Summary [28.180s] 441 tests run: 441 passed, 2580 skipped
cargo fmt --check -p vantadb                  -> exit 0
cargo clippy -p vantadb --all-targets --no-default-features \
  --features "cli,fjall,memmap2,fs2,roaring" -- -D warnings  -> limpio
```

## No tocado / fuera de alcance

- El camino hibrido del SDK (`Embedded::search`, `sdk/search/mod.rs`) tambien propaga el
  error del brazo vectorial, pero ahi la entrada es un vector ya calculado: el unico fallo
  alcanzable es un error de argumento del llamador (dimension, `ERR-028`), que **debe**
  seguir siendo error. No se degrada input invalido.
- `sdk/search/hybrid.rs::hybrid_search` es codigo muerto (`#[allow(dead_code)]`, sin
  llamadores: `sdk/search/mod.rs` reimplementa la hibrida inline). No se borro en esta
  tarea: fuera de alcance. `NOTICED BUT NOT TOUCHING`.
