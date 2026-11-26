---
title: "FIND-758: el purgado de shred se traga el fallo de borrado"
kind: task
status: completed
description: "`delete.rs:54` hacía `let _ =` sobre el purgado de la fila shred: un borrado que podía fallar sin reportarlo. El delete no debe fallar (el tombstone del WAL es el punto de commit) pero sí reportar: `tracing::warn!` con node_id y error + contador de purgas fallidas. Test con backend que falla en la partición InternalMetadata."
id: FIND-758
---

# FIND-758: el purgado de shred se traga el fallo de borrado

## Metadata

- **Plan file:** [2026-11-26-batch-gates-and-findings.md](../plans/2026-11-26-batch-gates-and-findings.md) — carril R
- **Origen:** `FIND-152` (`f9ec61f9e`) — al clasificar los `unwrap` de `shred` apareció lo contrario de lo que el ticket buscaba: los 92 `unwrap` de `shred`/`wasm` eran **test-only** y el único defecto real era este `let _ =`, que ningún recuento de `unwrap` podía ver.
- **Esfuerzo:** 🟢 1 h
- **Prioridad:** 🟡 Media
- **Tipo:** bug-fix (observabilidad de un fallo silenciado)
- **Carril:** R
- **Creado:** 2026-11-26

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `StorageEngine::delete` (`src/storage/engine/delete.rs:54`) es el **único** call site de `ShreddedRowStore::delete` en producción (`rg 'ShreddedRowStore::delete\b'` → 3 hits, todos `#[cfg(test)]` en `src/shred/mod.rs:558,566,637`) |
| Callees | `ShreddedRowStore::delete` → `StorageBackend::delete(InternalMetadata, "shred::{node_id}")`. No hay otra capa |
| Implicaciones | **Cero cambio de semántica**: `delete()` sigue devolviendo el resultado del delete del backend del nodo; el purgado del shred sigue siendo best-effort. Lo único nuevo es que el fallo deja rastro (`warn!` + contador). `delete_batch` y `purge_permanent` **no se tocan**: purgan el shred vía `write_batch(...)?` atómico, así que un fallo ahí **falla el delete** (no hay split brain: el batch no aplica nada). Son dos políticas opuestas —batch falla, individual reporta— para el mismo borrado lógico, y es deliberado: el path individual ya commiteó el tombstone |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos o en la extensión necesaria:**
  - `src/storage/engine/delete.rs` (457 L, **completo**): `delete()` L19-66 (el `let _ =` en L54), `delete_batch()` L179-290 y `purge_permanent()` L296-317 (ambos purgan el shred vía `delete_op` **dentro** de un `write_batch(...)?`, ya reportado); módulo de test `physical_erasure` L350-457.
  - `src/shred/mod.rs` (49 KB, extensión necesaria): imports L70-73, `ShreddedRowStore` L261, `put`/`put_op` L335-364, `get` L408-462, `delete` L468-473, `delete_op` L478-483, módulo de tests L488+ (`unwrap` permitido por `#[allow(...)]`, `FIND-152`).
  - `src/backend.rs`: `BackendWriteOp` L112-129, `BackendCapabilities` L170-177, `Scannable` L190-220, `StorageBackend` L258-336 (`delete`/`write_batch`/`capabilities`), wrapper de test `DefaultGetManyWrapper` L944-987 (patrón de delegación a seguir).
  - `src/storage/engine/mod.rs`: `StorageEngine` L329-487 (`backend: Arc<dyn StorageBackend>` **`pub(crate)`** L331 — permite la inyección en test).
  - `src/storage/engine/tests/mod.rs`: helpers `in_memory_engine()` L38-46, `sample_node()` L72-76.
  - `src/storage/engine/tests/ops.rs` (40 L iniciales): cabecera de imports y patrón de test del área.
  - `src/attestation.rs`: `PURGE_SURFACES` (`shred` es una de las 9 superficies) — la certificación de purga es *a posteriori*, sólo corre si alguien la pide.
  - `.opencode/rules/durability.md` (completo, 34 L): scope `storage/engine/`, `shred/`.
- **Referencias hacia dentro:** `ShreddedRowStore::delete` sólo lo llama `StorageEngine::delete`. `ShreddedRowStore::get` no tiene call sites de producción todavía (el consumidor del fast-path es deuda declarada en `FIND-604`), así que la fila huérfana hoy sólo la ve la atestación.
- **Referencias hacia fuera:** `ShreddedRowStore` → `BackendPartition::InternalMetadata` (`src/backend.rs`); el `warn!` usa `tracing` (ya dependencia del crate, `Cargo.toml:134`); el contador usa `std::sync::atomic` (sin dependencia nueva).
- **Veredicto de impacto:** **bajo**. Un call site, cero cambios de firma pública (`purge_failure_count` es `pub(crate)`), cero dependencias nuevas, cero cambios de formato on-disk. El riesgo real es el pre-mortem: (a) convertir el `let _ =` en `?` — **prohibido**, cambiaría la semántica del borrado; (b) reportar con un `debug!` que nadie ve — se usa `warn!`; (c) un test que no fuerce el fallo del shred — el backend de test cuenta los intentos y el test **afirma** que hubo exactamente uno.

## Diagnóstico

`src/storage/engine/delete.rs:54`:

```rust
let _ = crate::shred::ShreddedRowStore::delete(id, &*self.backend);
```

El comentario del sitio (L49-53) razona bien: *"the WAL tombstone is the commit point, so a shred purge failure must never fail the delete"*. Correcto y **se conserva**. Pero *no fallar* ≠ *no reportar*: el `Result` se descarta y, si el borrado de la fila `shred::{node_id}` falla, no queda rastro **en el momento del fallo**. La atestación de purga (`src/attestation.rs`, superficie `shred`) puede volver a sondear después — pero sólo si alguien corre la certificación. Consecuencia: la fila sobrevive al nodo y nadie sabe cuántas hay ni desde cuándo.

Contraste con los vecinos que **sí** reportan: `delete_batch` (L276) y `purge_permanent` (L315) meten `ShreddedRowStore::delete_op(id)` dentro de un `write_batch(...)?` atómico — si falla, el `?` propaga y el llamador lo ve. El path de `delete()` individual es el único que silencia.

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Discovery: único call site, vecinos que sí reportan, partición de la fila, contador disponible en la zona | ✅ |
| 2 | Task file + Impacto mapeado (Regla 0) antes de la primera edición | ✅ |
| 3 | RED: test con backend que falla en `InternalMetadata` — `delete()` Ok **y** contador sube, con el fallo forzado y contado | ✅ |
| 4 | GREEN: `ShreddedRowStore::delete_reporting` (`warn!` + contador) y `delete.rs` sin `let _ =` | ✅ |
| 5 | VERIFY: nextest del área + clippy + fmt + docs + OCR | ✅ |
| 6 | Review P2-01 (tier Adversarial: `src/storage/**`) + commit | ✅ |

## Review (GATE — agente distinto, P2-01)

- **Tier:** **Adversarial** — el diff toca `src/storage/**` (y `src/shred/**`). Reviewer distinto del autor.
- **Reviewer:** `vanta-review`, contexto fresco (sesión `ses_de6dfb604ffepJIbhbcTmM1NoF`), con instrucción explícita de verificar el contrato contra el código, no contra el resumen.
- **Veredicto:** ✅ **approve** — **0 Critical, 0 High, 0 Medium abiertos**; 3 Low (`L1` nivel del `warn!` sin afirmar, `L2` divergencia de *outcome* no nombrada, `L3` el delete transaccional no purga) y 4 Nits.
- **Premisa verificada por el reviewer contra HEAD:** `git show HEAD:src/storage/engine/delete.rs` → `:54 let _ = crate::shred::ShreddedRowStore::delete(...)`, y **0** `warn!`/contador en ese fichero. El defecto no está manufacturado.
- **El gate sirvió:** el reviewer levantó un **Medium y lo cerró dentro de la misma review** — con la primera versión del test (`count > before`) un mutante que contara **intentos** en vez de **fallos** pasaba verde. Verificado empíricamente: contra la versión actual ese mutante falla en `ops.rs` con `a successful shred purge moved the failure counter`. La aserción quedó en delta exacto (`before + 1`) + "un purgado sano no la mueve".
- **Aplicado del review:**
  - **N1** — borrado el scaffolding de depuración que sobrevivió a una iteración (`ptr_before`/`ptr_after`, `probe` sin aserción, dos `eprintln!`); además era inútil (comparaba dos `Arc` distintos, nunca pueden coincidir).
  - **N2** — el bloque (c) dejó de depender de un `engine.delete()` de un id no insertado: se reescribió **hermético**, llamando `delete_reporting` directamente contra el backend que falla y contra el sano. Más estricto y sin acoplamiento.
  - **N3** — líneas del `avance` corregidas (`delete_batch :276→:283`, `purge_permanent :315→:322`: el propio diff las corrió).
  - **N4** — añadida la razón de que el purgado viva **dentro** de `insert_lock` (sacarlo reabriría una carrera con un re-insert del mismo id que ya escribió su fila nueva).
  - **L2** — nombrada la divergencia de *outcome* en `## Blast Radius` (el batch **falla**, el individual **reporta**: el batch es atómico y no hay split brain).
- **Anotado como deuda, no aplicado:** `L1` (el helper barato para afirmar el `WARN` vive privado en `src/executor.rs`, fuera de la lista de escritura de esta tarea) y `L3` (fila `FIND-779` propuesta). Ver `## Deuda declarada`.
- **Verificación del reviewer:** el comando exacto del contrato **111/111, dos veces** (antes y después de la edición del test que llegó a mitad de review); `cargo fmt --check` exit 0; `check-docs`/`check-links` exit 0; reprodujo el `LNK2019` con `CARGO_INCREMENTAL=1` y relinkeó limpio con `0` (confirma que es del entorno, no del cambio). Declara **no** haber re-corrido clippy y **no** haber re-ejecutado M1/M2 (los da por deducción cerrada: hay un solo `fetch_add` en el crate).

## Medición

| Hecho | Antes | Después |
|---|---|---|
| `let _ =` sobre un borrado real en el path de `delete()` | **1** (`delete.rs:54`) | **0** |
| Rastro del fallo en el momento en que ocurre | ninguno (ni log, ni contador) | `warn!` con `node_id` + error + `purge_failures_total` |
| Call sites de `ShreddedRowStore::delete` | 4 (3 test-only + 1 productivo silenciado) | 3 test-only; el productivo pasa por `delete_reporting` |
| Test que fuerza el fallo | **no existía** | `shred_purge_failure_is_reported_but_does_not_fail_the_delete` |

**Falsabilidad (mutantes, corridos):**

- **M1 — el bug restaurado** (`let _ = crate::shred::ShreddedRowStore::delete(id, &*self.backend)`): el test **falla** →
  `the shredded-row purge failure left no trace (FIND-758): counter did not move`.
- **M2 — el purgado eliminado**: el test falla antes, en `assert_eq!(shred_delete_attempts, 1)`; el wrapper sólo falla en `BackendPartition::InternalMetadata` y cuenta los intentos, así que un "delete devolvió Ok" sin purgado no puede pasar en verde (pre-mortem c).

## Verify (contrato mecánico)

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb -E 'test(shred_purge_failure_is_reported) or test(shred) or test(delete)'` | ✅ **111/111** |
| `cargo nextest run --profile audit -p vantadb -E 'test(storage)' --build-jobs 2` | ✅ **513/513** (1 SLOW pre-existente: `put_batch_multi_chunk_atomic_commit_visible`, 79,7 s) |
| `cargo clippy -p vantadb --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --check` | ✅ 0 diffs |
| `node scripts/docs/check-links.mjs` | ✅ exit 0 (1 wikilink historial, pre-existente y presupuestado) |
| `node scripts/docs/check-docs.mjs` | ✅ exit 0 (2 huérfanos pre-existentes: `FIND-764`, `FIND-768` — no son míos) |
| `pwsh dev-tools/ocr-review.ps1` + `ocr delegate rule` | ✅ 2 reviewables míos (`src/shred/mod.rs`, `src/storage/engine/delete.rs`) revisados contra el Rule Group `project/**` (durability) |

**Nota de entorno:** el primer rebuild tras el mutante M1 falló el link con `LNK2019 ... 428 externos sin resolver` (artefactos incrementales corruptos por presión de disco, ~10 GB libres). `dev-tools/target-cleanup.ps1 -AutoSafe` liberó 2,06 GB y `CARGO_INCREMENTAL=0` relinkeó limpio. No es del cambio.

## Incidente de árbol compartido (2026-11-26) — la fila resucitó una vez

El review P2-01 lo cazó con `git show HEAD:docs/dev/Backlog.md`: el commit `a187d034d` (`docs(backlog): FIND-768 - restaurar la fila FIND-757…`) **bakeó una copia stale** de `Backlog.md` y devolvió la fila `FIND-758` a HEAD — el modo de fallo exacto de `FIND-753` (una lane commitea su copia del fichero y revierte el trabajo de otra). No es un fallo del cambio: es el coste de que N carriles compartan un checkout y un `Backlog.md`.

Mitigación aplicada: se re-aplicó el hunk y el diff contra HEAD quedó en **1 insertion / 2 deletions** (retitular el header de la sección + borrar la fila), sin tocar `FIND-757`/`FIND-768` (su trabajo ya estaba en HEAD). El `-ExpectAbsent 'docs/dev/Backlog.md::| `FIND-758` |'` del helper es el gate que detecta una segunda resurrección **por contenido**, no por exit code: es la primera cosa a mirar si este cierre se ve raro.



## Pre-mortem (del ticket)

- **(a) Convertir el `let _ =` en `?`** → descartado: haría fallar el borrado por un dato derivado y contradiría el comentario del sitio. El contrato es *reportar sin cambiar el resultado*.
- **(b) Reportar con `debug!`** → descartado: es exactamente el fallo que el ticket describe (un rastro que nadie ve). Se usa `warn!` con `node_id` y el error.
- **(c) Test que no fuerce el fallo** → cerrado por construcción: el backend de test falla **sólo** en la partición `InternalMetadata` (la de la fila shred), delega todo lo demás al backend real, y el test afirma que hubo exactamente un intento de borrado en esa partición. Si el purgado desapareciera, el test falla por el contador de intentos, no por el contador de fallos.

## Deuda declarada

- El contador es de **proceso** (`static AtomicU64`), no por-`StorageEngine`: dos motores en el mismo proceso comparten cuenta. Es lo que pide el contrato ("un contador si hay uno disponible en la zona") y el canal de operador seguirá siendo el `warn!`; un contador por-handle exigiría un campo nuevo en `StorageEngine` y su hilo por todos los constructores (`init.rs`, `db.rs`, tests), fuera del presupuesto de 1 h. `ponytail:` aplicado en el sitio.
- **El *nivel* del `warn!` no lo afirma ningún test** (review L1). El test afirma el contador (dos veces: falla suma, éxito no), y `purge_failure_count` tiene **cero consumidores externos**; el único canal del operador es el `warn!`. Un refactor `warn!` → `debug!` dejaría CI verde y al operador ciego — el pre-mortem (b) del ticket, a nivel de test. **No se cerró** porque la costura barata es el helper `capture_warnings` de `src/executor.rs:1325-1358`, que es **privado de ese módulo de tests** y `src/executor.rs` está fuera de la lista de escritura de esta tarea; duplicarlo serían ~35 líneas de plomería de subscriber. Cerrarlo cuando el helper se promueva a `#[cfg(test)]` compartido (candidato a fila propia si se decide).
- **El delete transaccional nunca intenta el purgado del shred** (review L3, pre-existente y fuera de scope): `rg -c 'shred' src/storage/engine/txn.rs` → **0**; la rama `BufferedWrite::Delete` de `commit_transaction` (`txn.rs:381-389`) hace `stamp_deleted_in_backend` + `apply_delete_inner` sin tocar `shred::{id}`, y nada lo reclama después (`purge_permanent`/`delete_batch` no tienen callers de producción). Consecuencia para esta señal: **contador en 0 ≠ sin residuo** — cubre sólo el path no-transaccional. Fila propuesta abajo (el owner del Backlog la registra: `docs/dev/Backlog.md` está fuera de mi escritura salvo mi propia fila).

### Fila propuesta para `docs/dev/Backlog.md`

```
| `FIND-779` | 🟡 Media | **El delete transaccional nunca purga la fila shred, y el contador nuevo no puede ver ese residuo** | `src/storage/engine/txn.rs:381-389` (`BufferedWrite::Delete`) · `src/shred/mod.rs` (`delete_reporting`) | 🟢 2-3h | 🟡 Media | 🟡 Pendiente (2026-11-26) | `FIND-758` dejó observable el fallo del purgado en el path **individual** (`delete()` reporta sin fallar). El path **transaccional** ni siquiera lo intenta: `rg -c 'shred' src/storage/engine/txn.rs` → **0**, y `commit_transaction` se queda en `stamp_deleted_in_backend` + `apply_delete_inner`. Nada lo reclama después: `purge_permanent` y `delete_batch` no tienen callers de producción (sólo tests) y el camino de reparación (`maintenance.rs:797`) limpia el stamp `deleted_by_txn` sin emitir op de shred. Consecuencia: una fila `shred::{id}` de un nodo borrado en transacción sobrevive para siempre y **el contador de `FIND-758` no la ve** (sólo cuenta intentos fallidos, y ahí no hay intento). Un operador con `purge_failures_total = 0` leería "sin residuo" y estaría equivocado. Contrato: el commit transaccional emite `ShreddedRowStore::delete_op(id)` en su batch (misma atomicidad que `delete_batch`) o, si se decide no hacerlo, la señal se documenta como parcial en `attestation.rs` y en el doc del contador; test que borre en transacción y verifique que `shred::{id}` no sobrevive. | Origen: `FIND-758` (2026-11-26, review L3) | Ver: `FIND-758`, `VER-02` |
```

