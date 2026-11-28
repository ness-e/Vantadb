---
title: "FIND-809: remove_edge y ensure_edge serializan con el split *_locked"
kind: task
status: completed
description: "El camino de borrado de aristas conservaba el read-modify-write sin guard de FIND-773 (525 de 800 borrados perdidos). Los wrappers publicos toman el purge_lock y los 10 call sites internos usan las variantes *_locked."
id: FIND-809
---

# FIND-809: `remove_edge` / `ensure_edge` sin serializar

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-809`)
- **Origen:** cierre de `FIND-773` (declarado como hueco no cerrado) · **Ver:** `FIND-773`, `FIND-756`, `FIND-798`
- **Esfuerzo:** 🟠 2-4 h · **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED (local; review P2-01 pendiente — ver §Review)

## Premisa (re-medida en HEAD)

`FIND-773` puso `purge_lock.write()` en `add_edge`; `remove_edge` y `ensure_edge` quedaron fuera
porque sus llamadores ya están dentro del guard (una segunda adquisición se bloquearía contra sí
misma: `parking_lot::RwLock` no es reentrante). Resultado: el camino de **borrado** conservaba el
read-modify-write de la lista entera de aristas.

Medición propia sobre `HEAD` con el test de 8×100 (8 hilos borrando cada uno sus 100 aristas de un
nodo con 800):

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --test graph_edge_concurrency -E 'test(concurrent_remove_edge)'
→ panicked: 525 of 800 edges survived the removal: the read-modify-write of
  remove_edge is not serialized (FIND-809)
→ Summary [0.176s] 1 test run: 0 passed, 1 failed
```

525 supervivientes = borrados perdidos (el `insert` de un hilo pisa la lista que otro ya había
recortado), no "aristas que no se borraron".

## Contrato entregado: split `*_locked` (§8 de `concurrency-async.md`)

| Capa | Método | Guard |
|---|---|---|
| Público | `Embedded::remove_edge` | toma `purge_lock.write()` y delega |
| Interno | `Embedded::remove_edge_locked` (`pub(crate)`) | **sin** guard — caller ya lo tiene |
| Público (`pub(crate)`) | `Embedded::ensure_edge` | toma `purge_lock.write()` y delega |
| Interno | `Embedded::ensure_edge_locked` (`pub(crate)`) | **sin** guard |

Call sites (14 en total, censo con `rg`):

- **Dentro del guard → `_locked`** (10): `put_one` (4), `put_batch_inner` (2),
  `put_record_exact` (4) — todos bajo `insert_one_internal`/`put_batch_inner`/`put_record_exact`,
  que ya toman `purge_lock.write()`.
- **Fuera del guard → wrapper público** (4): `supersede` (3) y `bulk_import_stream` (1). Se
  cierran solos, sin tocar esas líneas: el wrapper toma el guard.
- `record_graph_op` vive en la variante `_locked`, para que el camino interno también cuente.

`add_edge` se queda como está (ya estaba guardado por `FIND-773`); solo se corrigió su comentario,
que afirmaba que `remove_edge`/`ensure_edge` no estaban cubiertos.

### Orden de locks (por qué no hay deadlock nuevo)

`supersede` sostiene `supersede_lock` y ahora alcanza `purge_lock.write()` por `ensure_edge`.
`forget(Soft)` toma `supersede_lock` → `purge_lock.write()`. **El orden es el mismo**
(`supersede_lock` → `purge_lock`). Ningún camino del crate toma `purge_lock` y después
`supersede_lock`, y ningún `_locked` vuelve a entrar al wrapper.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/api/graph.rs`, `src/sdk/api/memory.rs` (§los 14 call
  sites y sus guards), `tests/graph_edge_concurrency.rs`, `.opencode/rules/concurrency-async.md`
  §8, `src/sdk/builder.rs` (§`purge_lock`/`supersede_lock` compartidos entre wrappers).
- **Referencias entrantes:** PyO3/WASM/Node/MCP llaman `remove_edge` (wrapper, sin guard propio);
  `vantadb-mcp/src/code_index.rs` idem. Firma pública sin cambios.
- **Veredicto:** confinado a `src/sdk/api/graph.rs` + 10 líneas de call sites en `memory.rs`.
  Coste: los dos caminos de arista ahora serializan contra los demás read-modify-write del handle
  (ya contendían en el `insert_lock` del motor).

## Steps

1. ✅ Test RED: `concurrent_remove_edge_removes_every_edge` (8 hilos × 100 borrados → 0
   restantes, y 0 mitades reverse supervivientes). Falla en HEAD con 525 de 800.
2. ✅ Split `remove_edge{,_locked}` / `ensure_edge{,_locked}` en `graph.rs`.
3. ✅ 10 call sites internos de `memory.rs` migrados a `_locked`; los 4 externos intactos.
4. ✅ Comentario de `add_edge` corregido (afirmaba lo contrario).

## Contrato de verificación

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --test graph_edge_concurrency       # 3/3 (los 2 de FIND-773 + el nuevo)
cargo nextest run … --test memory_graph_lineage --test graphrag_test
                                      # 27/27 (supersede/linaje bajo el guard nuevo)
```

**Mutante:** el propio `HEAD` es el mutante (sin guard) — pone el test en ROJO con 525/800. Un
mutante equivalente (borrar la línea `let _write_guard = self.purge_lock.write();` de
`remove_edge`) reproduce el mismo rojo.

**Invariante:** `_locked` no debe llamar nunca al wrapper público; el wrapper no debe llamarse
desde dentro de un guard (deadlock, no error).

## Review

- **P2-01 ronda 1 (`vanta-review`, contexto fresco, agente distinto): APPROVE.** Sin
  CRITICAL/HIGH. Evidencia del review: `cargo test --test graph_edge_concurrency` → 3 passed
  en `bf5434231`; y **el test no es vacuo**: con el wrapper revertido volvió a fallar con
  `585 of 800 edges survived`.
- `DEADLOCK_SCAN: clean` — `supersede_lock` siempre antes de `purge_lock`, ningún camino
  `purge_lock → supersede_lock`, e `insert_lock` (dentro de `compact_wal`) nunca espera
  `purge_lock`.
- `REENTRANCY_SCAN: clean` — los únicos wrappers públicos que quedan dentro del crate son
  `supersede` (3) y `bulk_import_stream` (1), ninguno con el guard tomado; todos los sitios que
  lo toman usan `*_locked`.
- `LOW` (aplicado): se perdió `#[tracing::instrument]` en `ensure_edge` durante el split —
  restituido.
- **No verificado:** el crate `vantadb-python` (su feature set no se compila en este carril).
  `rg` no encontró call sites, pero no se compiló: declarado.
- **Auto-revisión mecánica:** `dev-tools/verify.ps1` → 9 PASS / 1 SKIPPED (`cargo-clippy`, rojo
  de otro carril: `FIND-490`).

## Deuda declarada

- **`ensure_edge` no tiene test de concurrencia propio:** comparte el mismo read-modify-write y
  el mismo guard, y su ruta (linaje derivado) sí está cubierta por `memory_graph_lineage`; el
  test de 8×100 se escribió para el camino de **borrado**, que era el que perdía datos de forma
  observable. Añadir el gemelo de `ensure_edge` es trabajo de 10 minutos si se quiere el pin
  explícito.
- **Contención:** un solo lock de handle para las aristas (no por nodo). Mismo techo y misma
  salida que declaró `FIND-773`: stripes por `(min(src,dst), max(src,dst))` si el throughput de
  escritura lo pide.
