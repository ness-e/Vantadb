---
title: "FIND-799: el purge ya distingue Err de Ok(None) — premisa refutada"
kind: task
status: completed
description: "El camino ya propaga el `Err`: `NOT-02` (2026-11-03) eliminó el `ok().flatten()` que la fila cita, y el test que la fila pide no puede picar sin una costura de lectura. Fila retirada."
---

# FIND-799: el purge ya distingue `Err` de `Ok(None)`

## Metadata
- **Fuente:** `docs/dev/Backlog.md` → `Alta 2026-11-27` (fila `FIND-799`)
- **Origen:** `FIND-607` (partida 2026-11-27) · **Ver:** `FIND-607`, `FIND-798`
- **Esfuerzo:** 1-2h · **Prioridad:** Alta · **Tipo:** bug (claim de pérdida de datos)
- **Creado:** 2026-11-27 · **Estado:** ✅ COMPLETED (premisa refutada — sin cambio de producto)

## Veredicto

**La premisa es falsa en `HEAD`: un `Err` ya no se traga y no hay pérdida de
datos por esa vía.** La fila cita `src/agentic/thread.rs:247-255` como
`get(...).ok().flatten()`. En esas líneas hoy vive el bloque de TTL de `create`
(`:246-253` + `insert` en `:255`), y el `ok().flatten()` del purge fue
**eliminado el 2026-11-03** por `NOT-02` (`647f4f27c`), **24 días antes** de que
la fila se escribiera. El comentario que lo declara está en el propio código
(`thread.rs:362`):

> **Alcance de la refutación (lección de la review P2-01):** esto cierra el
> camino del **`Err`**, que es lo que la fila pedía. **No** cierra el síntoma
> entero: `materialize_uncached` mapea un header/vector corrupto o un **torn
> write** a `Ok(None)` (`get.rs:329-348`; `:257` documenta *"`None` = torn
> write"*), y en el rebuild ese `Ok(None)` borra el id del índice mientras la
> fila del backend sigue existiendo — mismo resultado ("invisible y no
> purgable") con una causa distinta. Ver **Riesgo residual** al final.

```rust
// Rebuild the index — keep only non-expired threads that still exist
// NOT-02: propagate real storage errors instead of silencing them with ok().flatten().
```

Evidencia mecánica:

- `git log -S 'ok().flatten()' -- src/agentic/thread.rs` → sólo `647f4f27c` (lo
  quita) y los commits de restauración masiva del repo.
- `git show 647f4f27c -- src/agentic/thread.rs` → reemplaza el
  `.filter(|id| self.engine.get(*id).ok().flatten()...)` por un `for` con
  `if let Some(node) = self.engine.get(id)?`.
- `rg 'ok\(\)\.flatten\(\)' src/agentic/thread.rs` → **una** coincidencia, y es
  la del comentario. `rg '\.get\([^)]*\)\.ok\(\)' src/agentic/thread.rs` → **0**.

## Demostración de que el invariante se sostiene

`purge_expired_threads` tiene exactamente dos sitios que traducen el resultado
de `get` a pertenencia al índice, y **ambos propagan**:

```rust
// thread.rs:352-359 — loop 1 (detecta y borra vencidos)
for id in &ids {
    if let Some(node) = self.engine.get(*id)? {      // Err => return, índice intacto
        if self.is_expired(&node, now) {
            self.engine.delete(*id, "ttl_expired")?;
            purged += 1;
        }
    }
}

// thread.rs:363-374 — loop 2 (reconstruye el índice)
if purged > 0 {
    let all_ids = self.load_thread_ids()?;
    let mut remaining = Vec::with_capacity(all_ids.len());
    for id in all_ids {
        if let Some(node) = self.engine.get(id)? {   // Err => return, sin save_thread_ids
            if !self.is_expired(&node, now) {
                remaining.push(id);
            }
        }
    }
    self.save_thread_ids(&remaining)?;
}
```

El índice **sólo** se reescribe si todo `get` devolvió `Ok`. Un `Err` en
cualquiera de los dos loops sale antes de `save_thread_ids`, así que el id
sobrevive. No hay otro camino que escriba el índice en esta función.

`Err` es alcanzable de verdad: `StorageEngine::get` propaga por
`fetch_backend_metadata` (`src/storage/engine/get.rs:240-249`, el
`deserialize_node_payload(...)?`) y por `read_header_at` /
`vstore_segment_reader` (`get.rs:257-283`). El camino de datos corruptos, en
cambio, **no** produce `Err`: `materialize_uncached` mapea header/vector corrupto
a `Ok(None)` (`get.rs:330-347`) — por eso el bug descrito necesitaba un `Err`
real, y por eso `NOT-02` importaba.

## Por qué el test que pide la fila no puede picar (ni antes ni ahora)

La fila exige un test que inyecte el fallo y compruebe que el id sigue.
**Con la estructura actual — y con la de antes del fix — ese test no puede
discriminar**, por una razón que no depende de dónde esté el `?`:

> Para que el loop 2 se ejecute hace falta que el loop 1 haya leído **todos** los
> ids sin error. Con un almacenamiento determinista, el loop 2 vuelve a leer los
> mismos ids y obtiene el mismo resultado. Un fallo determinista que alcance el
> loop 2 **necesariamente** aborta el loop 1 primero.

El bug pre-`NOT-02` sólo era alcanzable con un fallo **transitorio** (la primera
lectura pasa, la segunda no). **La razón de que hoy no haya test discriminante
no es el tipo concreto** — corregido en review: el crate ya trae failpoints de
primera clase (`Cargo.toml:141,164`: `fail = "0.5"` con `features=["failpoints"]`
bajo la feature `failpoints`; `src/lib.rs:221-235` re-exporta `FailScenario`,
`cfg_failpoint` y `remove_failpoint`, con uso real en `src/crypto/registry.rs:846`
y `src/edge_index.rs:21`). Un `fail::fail_point!` que contara lecturas **sí**
discriminaría la ventana transitoria. El motivo real es que **no hay failpoint en
el camino de lectura**, y añadirlo es `src/storage/engine/get.rs` — fuera del
alcance de esta tarea (y del presupuesto de una fila de 1-2 h para un defecto ya
cerrado).

Un test que inyecte un `Err` permanente y afirme "el id sigue en el índice"
**pasa igual con el código viejo y con el nuevo** (en ambos, el loop 1 aborta
antes de reconstruir). Sería un test verde-por-la-razón-equivocada: exactamente
el pre-mortem (d) de la tarea y la clase de `FIND-794`. **No se agrega.**

## Costura mínima, si alguna vez se quiere el guard

No se implementa (YAGNI: el defecto está cerrado). Pero si se quiere un test
discriminante de la ventana transitoria, **la costura más barata es un
failpoint, no un lector inyectable** (corregido en review): el crate ya paga la
dependencia `fail` bajo `--features failpoints` y `src/lib.rs:221-235` expone
`cfg_failpoint`/`remove_failpoint`/`FailScenario` para tests. Un
`fail::fail_point!("thread_store_read")` en el `get` del purge, configurado con
`cfg_failpoint("thread_store_read", "2*off->return")`, falla la **segunda**
lectura y discrimina.

Alternativa (más cara, y sólo si se quiere evitar `src/storage/**`): un lector
inyectable junto al `clock`.

```rust
// en ThreadStore, junto a `clock`
read_node: Arc<dyn Fn(&StorageEngine, u128) -> Result<Option<UnifiedNode>> + Send + Sync>,
// default: Arc::new(|e, id| e.get(id))
```

Con eso un test cuenta las lecturas y falla en la **segunda** — que es la única
forma de alcanzar el loop 2 con un `Err`. Coste: un campo + constructor estilo
`with_clock`. Ninguna de las dos se paga hoy porque no hay bug que guardar.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/agentic/thread.rs` (593 L) ·
  `src/storage/engine/get.rs` (1-350) · `src/storage/ops.rs`
  (`deserialize_node_payload`, 41-56) · `src/backend.rs` (1-160) ·
  `src/gc.rs` (`sweep`, 38-78) · `src/sdk/builder.rs` (395-424) ·
  `vantadb-mcp/src/threads.rs` (405-439) · `tests/message_thread_test.rs` (239 L).
- **Referencias hacia dentro de `purge_expired_threads`:** `Embedded::purge_expired_threads`
  (`src/sdk/builder.rs:410`) y el tool MCP `thread_purge_tool`
  (`vantadb-mcp/src/threads.rs:418`) — ambos wrappers finos, sin lógica propia.
- **Referencias salientes:** `StorageEngine::get`, `StorageEngine::delete`,
  `load_thread_ids`, `save_thread_ids`, `is_expired`.
- **Veredicto de impacto:** nulo. No hay cambio de código; el diff es la fila del
  Backlog y este task file. `gc.rs::sweep` ya distingue
  `Err(Error::NodeNotFound)` (quita) de cualquier otro `Err` (loguea y reintenta,
  `gc.rs:61-67`) - la misma clase, ya correcta.

## Verificación

- **Población del filtro, medida primero** (`FIND-794`): `cargo nextest list
  --profile audit -p vantadb --build-jobs 2 -E 'test(purge) or test(thread) or
  test(agentic) or test(ttl)'` → **40 tests** (>0).
- **Corrida con el mismo filtro** → **40/40 ✅**.
- `cargo fmt --check` ✅ · `cargo clippy` no aplica (sin cambio de Rust).
- `node scripts/docs/check-links.mjs` + `check-docs.mjs` ✅.

## No tocado, a propósito

- **`purged > 0` como única puerta del rebuild:** un id cuyo nodo ya no existe
  (`Ok(None)`) sólo sale del índice si **otro** hilo venció en la misma pasada.
  Deja ids fantasma permanentes (coste: una lectura por `list`). No es pérdida de
  datos — el nodo no existe — y la fila no lo pedía. Fuera de alcance.
- **`src/storage/**`, `src/shard/**`, `src/shred/**`:** prohibidos por la tarea.
  El `ok().flatten()` que queda en `src/storage/engine/delete.rs:38` es
  `pre_delete_node` para auditoría, no un índice de pertenencia.
- **`FIND-798` y `FIND-800`:** sus campos `Ver:` citan `FIND-799`. Quedan con la
  referencia colgada; el alcance de escritura del Backlog era "solo mi fila".
- **Contador del Backlog (`:17`):** "294 filas en tablas" quedó en 293. Es un
  contador de sweep manual y el drift es el normal; no se toca.

## Riesgo residual (fuera de esta fila, declarado)

El algoritmo tiene dos criterios y **sólo uno está nombrado**: `Err` (ahora
propagado — correcto) y `Ok(None)` (leído como "nodo ausente"). El segundo
miente cuando el nodo **sí existe** pero no se puede materializar: header/vector
corrupto o **torn write**, que `materialize_uncached` colapsa a `Ok(None)`
(`get.rs:329-348`; `:257` documenta *"`None` = torn write"*). En el rebuild
(`thread.rs:367-373`) ese `Ok(None)` saca el id del índice mientras la fila del
backend sigue viva: el hilo queda invisible y no purgable — el **mismo síntoma**
de la fila, con otra causa.

El contrato de `FIND-799` lo daba por bueno ("en `Ok(None)` se purga como hoy"),
así que corregirlo es **una fila aparte**, con este contrato: *el rebuild no
puede borrar un id sin prueba positiva de ausencia; "ilegible" no es "ausente"*.
Coste: `StorageEngine` tiene que exponer ambos estados sin colapsarlos
(`exists_with_status` o un `Result` que no mapee "ilegible" a `None`).

## Review

- **Revisor:** `vanta-review` (contexto fresco, P2-01).
- **Premisa (H-07):** refutada, **re-ejecutada por el revisor** de forma
  independiente: `rg 'ok\(\)\.flatten\(\)' src/agentic/thread.rs` → 1 hit
  (`:362`, el comentario); `rg '\.get\([^)]*\)\.ok\(\)'` → 0; ambos loops de
  `purge_expired_threads` (`:353`, `:367`) propagan y `save_thread_ids` sólo se
  alcanza en `:373`; `:247-255` es hoy el bloque de TTL de `create`.
- **Enfoque:** aprobado — "no se implementó un test que no puede fallar"; el
  argumento del loop 2 se confirmó contra la estructura y es más fuerte de lo
  que hacía falta (el loop 2 re-lee `all_ids` bajo el mismo `index_lock`, así
  que la lista es idéntica).
- **Cómo se probó:** población del filtro medida por el revisor (40 tests, 5
  binarios), coincidente con el auto-reporte; no corrió los 40 (el diff no toca
  Rust, la corrida no es falsable). `check-links` / `check-docs` re-corridos:
  exit 0.
- **Veredicto:** **approve** del análisis. Requeridos **M1** (registrar este
  gate) y **M2** (commit anti-clobber con `-Paths` explícitos + verificación
  post-commit) — ambos aplicados. **M3** (corregir la fila padre en
  `docs/dev/plans/`) **no se aplica: `docs/dev/plans/**` está PROHIBIDO por esta
  tarea**; queda declarado en el RESULTADO. Opcional **O1** (riesgo del
  `Ok(None)`) incorporado en § Riesgo residual; **O2** (failpoints, no el tipo
  concreto) corregido en § Por qué el test no puede picar.
