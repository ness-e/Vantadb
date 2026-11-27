---
title: "FIND-798: `create` de un thread dejaba un nodo huérfano si fallaba el índice"
kind: task
status: completed
description: "`create` insertaba el nodo antes de escribir el índice: si el índice fallaba, el nodo quedaba huérfano —invisible e inmortal— y el error invitaba a reintentar, duplicando. Orden invertido con compensación: el residuo pasa a ser un id sin nodo, y `list` ahora pagina sobre hilos existentes para que un id colgante no oculte datos."
---

# FIND-798: `create` dejaba un nodo huérfano si fallaba la escritura del índice

## Metadata

- **Fuente:** `docs/dev/Backlog.md` `Alta 2026-11-27` — **la fila ya no está en `HEAD`**: la
  retiró `eac9f948f` (2026-11-27), un commit de otro carril que horneó en su árbol la delegación
  que este carril tenía sin commitear. Verificado que fue la porción exacta (`-| `FIND-798` | 🟠 Alta |
  `create` de un thread deja un **nodo huérfano**…`) y que en ese commit el resto de la fila no se
  tocó; el commit de cierre de esta tarea por eso **no incluye** `Backlog.md` (no queda diff), pero
  el `-ExpectAbsent` se verifica igual.
- **Origen:** `FIND-607` (partida 2026-11-27) — **Ver:** `FIND-607`, `FIND-799`, `FIND-812`
- **Esfuerzo declarado / real:** 1-2h / ~2h
- **Prioridad:** Alta · **Tipo:** bug (duplicación de datos por reintento + ocultamiento en `list`)
- **Creado / cerrado:** 2026-11-27 · **Estado:** ✅ COMPLETED
- **Archivos tocados:** `src/agentic/thread.rs` (fix de `create` + fix de `list` + 4 tests) · este
  fichero · `docs/dev/Backlog.md` (sólo mi fila) · `docs/dev/avance/activo/core-engine.md`

## Impacto mapeado (Regla 0)

- **Leído completo:** `src/agentic/thread.rs` (593 líneas) — `create`, `send_message`, `list`,
  `delete`, `purge_expired_threads`, los helpers de índice (`load_thread_ids`/`save_thread_ids`/
  `add_thread_id_locked`/`remove_thread_id_locked`) y los dos tests existentes del módulo;
  `src/storage/engine/insert.rs:156-200` (la rama de transacción que produce el `InvalidInput`),
  `src/storage/engine/txn.rs:100-128` (`has_active`/`sole_id`), `src/storage/engine/partition.rs:10-25`
  (`put_to_partition` y su contrato de `insert_lock`), `src/storage/engine/ops.rs:285-351`
  (`scan_nodes` y su gate por HNSW), `src/storage/engine/stats.rs:27-38` (`ensure_writable`) y
  `:113-149` (`check_pressure`), `src/storage/engine/delete.rs:19`, `src/gc.rs:28-64`
  (`register_ttl` + `sweep`), `src/server/pagination.rs:33-41` (`page_meta`: `has_more =
  returned > limit`), `src/server/handlers.rs:1300-1315` (`list_threads(limit+1, skip)`),
  `src/sdk/builder.rs` (`create_thread`/`list_threads`/`engine_handle`), `dev-tools/verify.ps1:66`
  (`Get-CoreFeatures`), `.opencode/rules/core-engine.md` (R-3).
- **Referencias hacia dentro:** `src/sdk/builder.rs:362` (`Embedded::create_thread`),
  `src/sdk/builder.rs` (`list_threads`), el servidor HTTP (`src/server/handlers.rs:1300+`) y
  `vantadb-mcp/src/threads.rs`. **Ninguna firma ni semántica pública cambia**: `create` sigue
  devolviendo `Result<u128>` con el mismo conjunto de errores, y `list` sigue devolviendo
  `Result<Vec<MessageThread>>`. Lo que cambia es qué ids cuentan como posición de página.
- **Referencias hacia fuera:** `engine.insert`, `engine.delete` (ya `pub`), `engine.begin_transaction`/
  `abort_transaction`, `self.index_lock`, `add_thread_id_locked`, `remove_thread_id_locked`,
  `generate_id`. Sólo símbolos ya visibles en el crate.
- **Veredicto:** dos cambios locales (`create` y `list`), sin tocar firmas ni dependencias. Riesgo
  de deadlock evaluado: el orden `index_lock → insert_lock` es el mismo que ya existía en `create`,
  `delete` y `purge`, y nada que corra bajo `insert_lock` toma `sdk_thread_index_lock` (el único
  consumidor del índice es `thread.rs`, más la construcción en `builder.rs` y `engine/init.rs`).

## La premisa, verificada antes de tocar

`src/agentic/thread.rs:255-256` en `HEAD`:

```rust
self.engine.insert(&node)?;            // :255 — el nodo queda persistido
self.add_thread_id_locked(thread_id)?; // :256 — el índice puede fallar aquí
```

`add_thread_id_locked` (`:448-455`) hace `load_thread_ids()?` → `save_thread_ids(&ids)?`; ambos
pueden fallar. El `_guard` de `:225` serializa llamadores concurrentes pero **no revierte** el
`insert`.

Consecuencias leídas en el propio módulo:

- `list` (`:321-332`) camina **el índice**, no el backend ⇒ el huérfano es invisible.
- `purge_expired_threads` (`:344-377`) también camina el índice ⇒ el huérfano no entra ni en el
  conteo ni en el rebuild, así que es inmortal aunque tenga TTL.
- El error dice "create falló" y el reintento llama a `generate_id()` ⇒ id nuevo ⇒ segundo nodo.

## Steps

1. ✅ RED — test que falla en `HEAD`.
2. ✅ GREEN — orden invertido + compensación.
3. ✅ No-regresión — camino feliz (un nodo, una entrada de índice).
4. ✅ Review P2-01 → `changes-required` (4 findings aplicados, ver §Review).
5. ✅ Verify — `list`/`run` con el mismo filtro, fmt, clippy, doc gates.
6. ✅ Cierre — Backlog (mi fila), `avance/activo/core-engine.md`, commit aislado.

## RED — el test falla en `HEAD` por la razón correcta

```
thread 'agentic::thread::tests::create_failing_at_the_index_leaves_no_orphan_node' panicked at src\agentic\thread.rs:540:9:
assertion `left == right` failed: a create that failed at the index must persist zero nodes (found 1)
  left: 1
 right: 0
```

`create` devolvió `Err` (el `assert!(...is_err())` anterior pasó) y aun así quedó **1 nodo** en el
backend. Ese nodo es el huérfano: `list` no lo lista, `purge` no lo ve, y el reintento iba a sumar
el segundo.

## Cómo se inyecta el fallo (y por qué así)

El módulo **no** tenía costura de tests. La crate sí tiene una (`fail` crate tras el feature
`failpoints`, 4 `fail_point!` en `src/utils/fs.rs`, `src/storage/vfile.rs`,
`src/crypto/registry.rs`), pero `failpoints` **no** está en `Get-CoreFeatures`
(`dev-tools/verify.ps1:66`), así que un test gateado ahí no correría en el gate local y se
podriría. El failpoint que sí existe para el `insert` (`storage_insert_fail`) es además **global al
proceso**: armarlo haría fallar los `insert` de los tests que corren en paralelo en el mismo
binario (Regla 2).

Se usan **precondiciones reales**, una por rama:

| Rama a ejercitar | Precondición | Mecanismo |
|---|---|---|
| falla el **índice** (el bug de la fila) | blob corrupto en `InternalMetadata/_thread_ids` | `load_thread_ids` falla al deserializar dentro de `add_thread_id_locked` |
| falla el **insert** (la compensación) | **dos transacciones abiertas** | `insert` devuelve `InvalidInput` (`insert.rs:164-178`, rama `sole_id() == None`) mientras `put_to_partition` no conoce transacciones y escribe el índice igual |

Ambas son deterministas, por instancia, y no requieren costura nueva ni cambio de firma. La segunda
es exactamente el orden "índice escrito, insert falla" que la compensación existe para revertir.

El conteo se toma con `engine.scan_nodes()` — la vista **del store**, no la del índice —, así que ve
el huérfano que `list` no ve. Contar con `list` habría dejado el test en verde sobre `HEAD`.
Techo: `scan_nodes` está gateado por HNSW (`ops.rs:329-332` salta la fila sin `storage_offset`), así
que vale para este huérfano (que se inserta completo) pero no es un oráculo general de "backend
vacío".

## GREEN — el orden elegido y por qué

```rust
self.add_thread_id_locked(thread_id)?;
if let Err(e) = self.engine.insert(&node) {
    if let Err(cleanup) = self.remove_thread_id_locked(thread_id) {
        tracing::warn!(thread_id, error = %cleanup, "…");
    }
    return Err(e);
}
if let (Some(gc), Some(expires_at)) = (gc, ttl_expires_at) {
    gc.register_ttl(thread_id, expires_at);
}
```

**Por qué invertir en vez de compensar el `insert` con un `delete`.** Las dos inconsistencias no
cuestan lo mismo:

| Residuo de un fallo | ¿`list` lo renderiza? | ¿Se recicla solo? | Efecto sobre el reintento |
|---|---|---|---|
| nodo sin id en el índice (orden viejo) | **no** | **no** | id nuevo ⇒ **duplica** |
| id en el índice sin nodo (orden nuevo) | **no** | **no** (ver abajo) | ninguno: no duplica |

El orden nuevo pone el residuo en la dirección barata: el peor caso es un id desperdiciado, no un
nodo duplicado con datos. La compensación elimina además ese id en el caso común.

**Corrección tras la review (R1b):** un id colgante **no** lo recicla el purge. El rebuild de
`purge_expired_threads` sólo corre `if purged > 0` (`:392`), y un id sin nodo hace `get → Ok(None)`,
que no incrementa `purged`. Sólo desaparece si *otro* hilo expira en el mismo barrido. El comentario
del código y este documento dicen eso ahora; la versión anterior afirmaba lo contrario.

**Por qué compensar.** Sin compensación, cada `create` fallido deja un id en el JSON del índice
hasta que ocurra una expiración ajena. La compensación lo limpia ya. Si la compensación también
falla (`save_thread_ids` toma el mismo `insert_lock` temporizado en el que el `insert` pudo haber
expirado), se conserva el id colgante —la dirección barata— y se devuelve el error **original** del
`insert`; el fallo del cleanup se registra con `tracing::warn!` en vez de descartarse en silencio
(norma de `FIND-758`, `delete.rs:54-61`).

**Por qué `register_ttl` se movió detrás del `insert`.** Registrarlo antes dejaba al GC con una
expiración para un id que nunca tuvo nodo, y eso se materializa en un `Delete` escrito en el WAL en
el siguiente barrido. Ahora sólo se registra un TTL de un thread que existe.

**Lo que NO se hizo:** ignorar el error del índice y devolver `Ok` — eso cambiaría un huérfano por
un hilo invisible, que es el síntoma de `FIND-799`/`FIND-812`. `create` sigue fallando cuando el
índice falla.

## El fix de `list` (hallazgo de la review, R1)

El orden invertido hace que el residuo barato sea un id sin nodo, y eso sólo es barato si `list` no
lo cuenta. **No era el caso:** `list` hacía `skip(offset).take(limit)` sobre los ids **crudos** y
filtraba después, así que un id colgante gastaba un slot de página.

La cadena, ya en código: `src/server/handlers.rs:1306-1310` pide `list_threads(limit+1, skip)` y
`src/server/pagination.rs:38` calcula `has_more = returned > limit`. Con ids colgantes al frente,
`returned` se queda corto ⇒ `has_more: false` y **sin `next_cursor`** ⇒ el cliente no puede avanzar
mientras los hilos existen y se leen por id. Eso es el "huérfano por invisible" que el contrato de
esta tarea prohíbe, alcanzado desde `list`.

No era residuo teórico del reorder: `GcWorker::sweep` (`src/gc.rs:57`) llama `StorageEngine::delete`
directo, sin pasar por `ThreadStore::delete`, así que deja ids colgantes desde antes.

Fix: resolver primero, paginar después. Sobre datos sanos el resultado es idéntico; el costo es a lo
sumo `offset + limit` `get`s en el caso normal (antes eran `limit`, con slots desperdiciados).

## No-regresión del camino feliz

`create_leaves_one_node_and_one_index_entry`: un `create` correcto deja exactamente 1 nodo en el
store y exactamente 1 entrada en el índice, con el mismo id. Los dos tests preexistentes del módulo
(`concurrent_send_message_keeps_both_messages`,
`concurrent_create_thread_keeps_both_threads_in_the_index`) siguen verdes, igual que
`message_thread_test` (3 tests de integración cargo/lectura/TTL).

## Verificación

- `cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(thread) or test(agentic) or test(create)'`
  → **35** tests antes · **39** después (4 nuevos).
- `cargo nextest run` con **el mismo** filtro + `--no-tests fail` → **39/39 passed**.
- `cargo fmt --check` → ✅
- `cargo clippy -p vantadb --all-targets -- -D warnings` → ✅
- `node scripts/docs/check-links.mjs` → ✅ (0 broken) · `node scripts/docs/check-docs.mjs` → ✅

## Techo declarado

- El `save_thread_ids` del camino de índice **no** se ejercita: el test inyecta el fallo en el
  `load`. Ambos cuelgan del mismo `?` del mismo helper y abortan antes del `insert`, así que
  comparten la garantía, pero la ruta `save` queda sin cubrir.
- El fallo **simultáneo** de `insert` y `save_thread_ids` (el caso en que el `warn` se dispara) no
  tiene test: requiere hacer fallar el mismo `insert_lock` dos veces, y no hay inyección por
  instancia para eso.
- `scan_nodes` está gateado por HNSW (ver arriba): es oráculo suficiente aquí, no general.
- `list` hace `get` por id existente hasta llenar la página; con un índice lleno de ids colgantes el
  barrido es O(índice) en el peor caso. Aceptable: el residuo es raro y el `break` corta en cuanto
  la página se llena.

## NOTICED BUT NOT TOUCHING

- `purge_expired_threads` sólo reconstruye el índice `if purged > 0` (`:392`). Hacer el rebuild
  incondicional reclaimaría ids colgantes de todos los orígenes (incluidos los del barrido de GC),
  pero agrega una escritura de índice en cada barrido: fuera de alcance, y `list` ya no depende de
  ese reciclado.
- `GcWorker::sweep` (`gc.rs:57`) borra nodos sin tocar el índice: sigue siendo un productor de ids
  colgantes. `list` ahora lo tolera; el saneamiento del índice queda para una fila propia.
- `add_edge`/`remove_edge` son de `FIND-773`/`FIND-809`: no tocados.

## Review

**P2-01, ronda 1 — `vanta-review` en contexto fresco: `changes-required`.** Cuatro findings
Required/Optional aplicados:

| # | Finding | Aplicado |
|---|---|---|
| R1 | `list` gastaba un slot de página por id colgante y podía devolver página vacía con `has_more: false` (ocultaba datos, el fallo que el contrato prohíbe) | `list` resuelve antes de paginar + test `list_pages_over_threads_not_over_dangling_index_ids` |
| R1b | El comentario afirmaba que "el rebuild del purge lo descarta", y con `if purged > 0` no ocurre | Comentario del código, frontmatter y tabla corregidos |
| R2 | La compensación no tenía test, y la justificación ("único punto de inyección = failpoint global") era falsa | Test `create_failing_at_the_insert_rolls_back_the_index_entry` con dos transacciones abiertas |
| O1 | `let _ =` sobre el cleanup: silenciaba el fallo en la única traza que vería un operador | `tracing::warn!` |
| O2 | `register_ttl` antes de las escrituras dejaba un TTL (y un `Delete` de WAL) para un id sin nodo | Movido detrás del `insert` exitoso |
| N1/N2 | `scan_nodes` no es un scan crudo del backend; `load`/`save` no comparten ruta ejercitada | Doc comments corregidos con el techo explícito |

Verificado por el reviewer en su delta: `cargo test --lib agentic::thread::tests::create` → 2
passed. Tras aplicar los findings, el módulo entero corre **6/6** y el filtro **39/39**.

**Lo que el reviewer confirmó como correcto:** invertir el orden es la decisión adecuada (el residuo
barato es estrictamente menor que el caro), el `?` conserva la semántica pública, la inyección por
precondición real en vez de costura nueva es la elección correcta, y no hay inversión de locks entre
`index_lock` e `insert_lock`.
