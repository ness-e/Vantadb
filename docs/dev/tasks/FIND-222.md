---
title: "FIND-222: el prefetch resucita el payload viejo en el cache volátil"
kind: task
status: in-progress
description: "El prefetch materializa un nodo y lo inserta después de que el write path haya vaciado la entry: la propia invalidación crea la vacante. Fix: contador de invalidaciones (bump + remove atómicos) y el prefetch re-valida bajo el lock. Determinista, no probabilístico."
---

# FIND-222: el prefetch resucita el payload viejo en el cache volátil

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-222`, línea 204)
- **Origen:** review P2-01 de `FIND-190` (O1) · **Ver:** `FIND-190` (invalida la entry en el write path), `FIND-809` (`purge_lock`), `FIND-826` (lección de test de carrera no determinista)
- **Esfuerzo:** 2-3 h · **Prioridad:** 🟡 Media (**la única de las tres Media que devuelve datos incorrectos en silencio**)
- **Creado / cerrado:** 2026-11-28
- **Estado:** ✅ COMPLETED

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `src/storage/engine/cache.rs` (campo + 3 métodos) · `src/storage/engine/get.rs` (el guard + hook de test) · `src/storage/engine/delete.rs` (2 sitios) · `src/storage/engine/insert.rs` (3 sitios) · `src/storage/engine/txn.rs` (3 sitios) · `src/storage/engine/tests/find222_prefetch_race.rs` (nuevo) · `src/storage/engine/tests/mod.rs` · este task file · `docs/dev/Backlog.md` (fila `FIND-222`) · `docs/dev/avance/activo/core-engine.md` |
| Inmutable | `src/storage/engine/maintenance.rs` (prohibido: la evicción no sirve datos incorrectos, ver §Alcance) · `src/storage/archive.rs` · `src/attestation.rs` · `src/text_index.rs` · `src/sdk/**` · `src/planner.rs` · `vanta-memory/**` · bindings · `.github/**` · `dev-tools/**` · `.opencode/**` · `docs/index.md` |
| Fuera de alcance | `maintenance.rs:403` (evicción/consolidación): un prefetch que re-cachea el mismo payload no devuelve datos incorrectos; **no** se toca (prohibido) y se declara como deuda `ponytail:` |

## Premisa (medida, no re-derivada)

`src/storage/engine/get.rs:625-636` — el prefetch materializa y luego inserta:

```rust
for warm_id in to_fetch {
    if let Ok(Some(node)) = self.get(warm_id) {        // T1: materializa el payload VIEJO
        let mut guard = self.cache.volatile.write();   // T2: toma el lock
        if let Entry::Vacant(e) = guard.entry(warm_id) {
            e.insert(node);                            // RESUCITA lo borrado
        }
    }
}
```

La pieza que hace el bug posible es la propia invalidación que lo corrige:

```rust
// src/storage/engine/delete.rs:163 (apply_delete_inner)
self.cache.volatile.write().remove(&id);   // crea la vacante
```

**Interleaving que devuelve el dato viejo (antes del fix):**

```
P  T0  prefetch arranca (to_fetch = [2])
P  T1  get(2) → payload VIEJO            (el backend todavía lo tiene)
W  T1.5 volatile.remove(2)               (delete / overwrite Cold / commit txn)
W  T1.6 backend.delete(2) | KV put nuevo
P  T2  volatile.write(); entry(2) == Vacant   ← precisamente porque W acaba de limpiar
P  T3  e.insert(payload VIEJO)                ← resurrección
```

`get(2)` sirve después el payload borrado (o el pre-overwrite) desde el cache, indefinidamente.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/get.rs` (parcial dirigido: `get`, `prefetch_related`, `PrefetchGuard`, `get_many`, `materialize_uncached`/`RowGap`), `src/storage/engine/delete.rs` (464 L, completo), `src/storage/engine/cache.rs` (135 L, completo), `src/storage/engine/insert.rs` (`apply_insert`, `cache_batch_hot_nodes`, rollback de batch), `src/storage/engine/txn.rs` (`commit_transaction` apply loop + `apply_insert_with_txn` + `insert_in_txn`/`delete_in_txn`), `src/storage/engine/maintenance.rs:320-408` (lectura para decidir si entra), `src/storage/engine/tests/mod.rs` + `tests/engine.rs:770-900` (infra de test existente), `tests/certified_delete.rs` (baseline 17).
- **Referencias hacia dentro:** `prefetch_related` ← `get()` (`get.rs:583`); `apply_delete_inner` ← `delete()` (`delete.rs:44`), `commit_transaction` (`txn.rs:390`); `apply_insert` ← `insert()`; `cache_batch_hot_nodes`/rollback ← `batch_insert_with_opts`; `CacheLayer::new` ← `init.rs:…` (un solo constructor).
- **Referencias entrantes de lo editado:** todos los sitios que tocan `cache.volatile` están inventariados (49 hits): 8 son de escritura del write path, el resto lecturas y tests. El único sitio que **inserta después de leer** es `prefetch_related` — el resto inserta el payload que acaba de commitear el propio writer.
- **Veredicto impacto:** **medio**. Se añade un campo + 3 métodos `pub(crate)` en `CacheLayer`; ningún camino público cambia de firma. Los escritores cambian `volatile.write().remove(id)` por `cache.invalidate(id)` (mismo efecto + bump). Riesgo real: ordenar mal el bump reabre la ventana (por eso el §Orden, y por eso el Prove-It).

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿`purge_lock` basta? | (a) reusarlo · (b) contador | **NO basta** — ver §`purge_lock` medido. `purge_lock` es un lock **de SDK** (`Embedded`), `prefetch_related`/`delete` son del `StorageEngine` y `delete()` del engine **nunca** lo toma (`mod.rs:380-382`: "engine paths never acquire it"). Tomarlo en `get()` sería (1) un lock de escritura en el hot path de lectura y (2) deadlock por recursión: `get → prefetch → get` (`parking_lot::RwLock::read` puede colgarse si un writer espera entre las dos tomas). |
| 2 | Granularidad del contador | (a) epoch por nodo · (b) contador global · (c) epoch por entrada | (b). Un `AtomicU64` en `CacheLayer`: 5 líneas contra un mecanismo nuevo. Conservador a propósito: con deletes concurrentes el prefetch **se salta**; un prefetch que no ocurre cuesta rendimiento, nunca corrección. El contador global solo se mide si algún bench demuestra que el falso negativo duele — **no se propone sin número**. |
| 3 | ¿Dónde se bumpa? | (a) solo en `delete` (lo que dice el ticket original) · (b) en **todos** los sitios que invalidan la entry | (b): la fila añade "mismo agujero en Hot/txn/delete". Sitios: `delete.rs:163` + `:290` (delete y delete_batch), `insert.rs:384` + `:717` + `:1133` (Cold overwrite, batch, rollback), `txn.rs:376` + `:415` (+ invalidación post-commit del insert Cold). |
| 4 | Orden bump/remove | (a) bump fuera del lock, antes del remove · (b) bump **dentro** del mismo critical section que el remove · (c) bump después del remove | (b) **y** colocado tras la mutación durable. (a) deja agujero: P0 puede leer la generación nueva *después* del bump y *antes* del remove, materializar el payload viejo e insertarlo con la generación "al día". (c) no cancela nada. Ver §Orden. |
| 5 | ¿Qué hace el prefetch al detectar cambio? | (a) reintentar · (b) saltarse el insert | (b). Es un optimizador: saltarse un prefetch es un coste de rendimiento, no de corrección. |

## Orden elegido (justificado con el interleaving)

Regla que se implementa: **la invalidación (`bump` + `remove`, en el mismo critical section del `volatile.write()`) es el último paso del writer, después de la mutación durable que hace ilegible el payload viejo** (tombstone del vstore en `apply_delete_inner:152-159`, put del KV nuevo en `apply_insert`/`apply_insert_with_txn`).

Con esa colocación, para cada prefetch:

| Interleaving | Resultado |
|---|---|
| T0 antes de la invalidación, T2 después | `load() != g0` → **skip** ✔ |
| T0 antes, T2 antes de la invalidación | inserta; la invalidación (posterior) **borra esa entry** ✔ |
| T0 después de la invalidación | materializa ya el estado nuevo (o `None`, si la fila está tombstoneada) ✔ |

Ningún orden deja una entry vieja viva. Si el bump fuese *antes* del remove pero fuera del lock (opción a), el primer caso se rompe: P0 lee `g0+1` en la ventana `[bump, remove]`, y su insert pasa la comparación.

## Steps

1. ✅ RED determinista: hook de test entre T1 y T2 + 5 casos (delete, delete_batch, overwrite, txn insert, txn delete). Con el bump desactivado los 5 fallan — ver §Prove-It.
2. ✅ GREEN: `CacheLayer` (campo + `invalidate`/`note_invalidation`/`invalidation_generation`), guard en `prefetch_related`, 8 call sites.
3. ✅ Prove-It: bump desactivado en un solo punto (`note_invalidation`) → 5/5 rojos con el payload resucitado en el cache.
4. ✅ Verificación completa (12 hilos, `vanta-memory`, nextest audit) + cierre.

## Interleaving, antes y después

**Antes** (el bug, medido en el Prove-It):

```
P  T1  get(2) → payload VIEJO
W  T1.5 volatile.write().remove(2)          ← la invalidación crea la vacante
W  T1.6 backend.delete(2) | KV put nuevo
P  T2  volatile.write(); entry(2) == Vacant ← porque W acaba de limpiar
P  T3  e.insert(payload VIEJO)              ← resurrección; get(2) sirve "old"
```

**Después:**

```
P  T0  g0 = invalidation_generation.load()          (antes de materializar)
P  T1  get(2) → payload VIEJO
W  T1.5 volatile.write() { bump; remove(2) }        (un solo critical section)
P  T2  volatile.write(); load() == g0 ? → NO (g0+1) → continue → no inserta
```

Casos límite, y por qué el orden elegido los cierra:

| Orden | Qué pasa |
|---|---|
| T0 antes de la invalidación, T2 después | mismatch → skip ✔ |
| T0 antes, T2 antes de la invalidación | inserta; la invalidación posterior **borra esa entry** ✔ |
| T0 después de la invalidación | materializa el estado nuevo, o `None` (fila tombstoneada: `materialize_uncached` → `Gap(Tombstoned)`) ✔ |
| bump **fuera** del lock, antes del remove (alternativa descartada) | **agujero**: P0 lee `g0+1` en `[bump, remove]`, materializa el payload viejo e inserta con la generación "al día" ✗ |

## `purge_lock`, medido (no asumido)

`rg -n purge_lock src/` → 35 hits. Los relevantes:

- `mod.rs:374-382` — `sdk_purge_lock`: *"Leaf lock: taken only by SDK code (`sdk::api::memory`); engine paths never acquire it"*.
- `sdk/builder.rs:107,225` — el `Embedded` clona el `Arc`; `from_engine` lo comparte (FIND-651).
- `sdk/api/graph.rs:150,190,242` — `add_edge`/`remove_edge`/`ensure_edge` (FIND-809).

**Conclusión medida:** `purge_lock` **no** cubre este camino. (1) `prefetch_related` y `delete` son métodos del `StorageEngine` y el `delete()` del engine **no lo toma nunca** (solo lo toman los wrappers SDK: `memory.rs:711,1152,1542,…`), así que las dos mitades de la carrera no se encuentran en él. (2) Hacer que `get()` lo tomase sería un lock de **escritura** en el hot path de lectura y, con `get → prefetch → get`, un deadlock documentado de `parking_lot::RwLock` (la toma de `read` recursiva se cuelga si un writer espera entre las dos). → hizo falta el contador.

## Tests (deterministas, no probabilísticos)

`src/storage/engine/tests/find222_prefetch_race.rs` — el hook `set_prefetch_interleave` (`get.rs`, `#[cfg(test)]`, `thread_local`) corre el writer **inline** entre T1 y T2: sin sleeps, sin segundo hilo, sin azar (lección de `FIND-826`). Cada caso es un sitio de invalidación distinto:

| Test | Sitio de invalidación |
|---|---|
| `delete_mid_flight_does_not_resurrect_the_node` | `delete.rs::apply_delete_inner` |
| `batch_delete_mid_flight_does_not_resurrect_the_node` | `delete.rs::delete_batch` (Phase 5) |
| `overwrite_mid_flight_does_not_cache_the_old_payload` | `insert.rs::apply_insert` (el "Hot" de la fila: FIND-190) |
| `txn_insert_commit_mid_flight_does_not_cache_the_old_payload` | `txn.rs::commit_transaction` → `apply_insert_with_txn` |
| `txn_delete_commit_mid_flight_does_not_resurrect_the_node` | `txn.rs::commit_transaction` → `apply_delete_inner` |

## Prove-It (el rojo, pegado)

Bump desactivado en `CacheLayer::note_invalidation` (un solo punto) → los 5 caen:

```
test ...::txn_insert_commit_mid_flight_does_not_cache_the_old_payload ... FAILED
test ...::overwrite_mid_flight_does_not_cache_the_old_payload ... FAILED
test ...::delete_mid_flight_does_not_resurrect_the_node ... FAILED
test ...::batch_delete_mid_flight_does_not_resurrect_the_node ... FAILED
test ...::txn_delete_commit_mid_flight_does_not_resurrect_the_node ... FAILED

---- overwrite_mid_flight_does_not_cache_the_old_payload ----
assertion `left == right` failed: the pre-overwrite payload must not survive in the cache
  left: Some("old")        ← el payload resucitado, servido desde el cache
 right: Some("new")
```

Restaurado el bump → 5/5 verdes.

## Verificación

| Comando | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✅ 0 (medido; el `--check` inicial marcó **mi** fichero y se corrigió con `cargo fmt --all`, que solo tocó mis 6 ficheros) |
| `cargo clippy -p vantadb --all-targets -- -D warnings` | ✅ exit 0 |
| `RUST_TEST_THREADS=12 cargo test -p vantadb --lib -- find222_prefetch_race` | ✅ 5/5 (población del filtro medida antes: 5) |
| `RUST_TEST_THREADS=12 cargo test -p vantadb --lib -- storage::engine` | ✅ **443/443** (barrido del área a 12 hilos) |
| `RUST_TEST_THREADS=12 cargo test -p vantadb --test certified_delete` | ✅ **17/17** |
| `cargo test -p vanta-memory -j 2` | ✅ exit 0 (el crate que consume) |
| `cargo nextest run -p vantadb --profile audit --build-jobs 2` | ✅ **3045/3045**, 0 fallos |
| `nextest … -E 'test(find222_prefetch_race)' --no-tests fail` | ✅ 5 run / 5 passed |
| `node scripts/docs/gen-index.mjs --write` + `--check` | ✅ 0 stale (947 documentos) |
| `node scripts/docs/check-docs.mjs` · `check-links.mjs` | ✅ exit 0 |

## Alcance declarado (no tocado)

- `maintenance.rs:403` (evicción/consolidación): un prefetch que re-cachea el mismo payload **no devuelve datos incorrectos**; además el fichero está fuera de zona. `NOTICED BUT NOT TOUCHING` → si algún día la evicción se mide, ahí falta el mismo `note_invalidation` (coste: re-cachear un nodo recién consolidado).
- `txn.rs` no estaba en la lista de zona, pero la fila nombra el camino `txn` y es el único sitio donde vive (3 líneas). Declarado.

