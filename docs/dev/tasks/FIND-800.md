---
title: "FIND-800: el TTL de los threads decía «auto-expiry» y no lo ejecutaba nadie"
kind: task
status: completed
description: "El `GcWorker` estaba muerto en producción (`Embedded` pasa `gc = None`) y el TTL de los hilos sólo se evalúa en el purge explícito. Salida (b): no se enciende el GC —su índice vive en memoria, no sobrevive a un reinicio y su barrido deja ids colgantes— y el doc pasa a decir la verdad, con el TTL declarado como ventana de inactividad que `send_message` re-arma."
---

# FIND-800: el TTL de los threads decía «auto-expiry» y no lo ejecutaba nadie

## Metadata

- **Fuente:** `docs/dev/Backlog.md` `Media 2026-11-27` — `FIND-800` (fila retirada en el cierre de esta tarea).
- **Origen:** `FIND-607` (partida 2026-11-27) — **Ver:** `FIND-607`, `FIND-799`, `FIND-798`
- **Esfuerzo declarado / real:** 2-4h / ~2h
- **Prioridad:** Media · **Tipo:** doc/contrato (promesa de API sin mecanismo) + decisión de diseño
- **Creado / cerrado:** 2026-11-27 · **Estado:** ✅ COMPLETED
- **Archivos tocados:** `src/agentic/thread.rs` (doc del módulo, `ThreadStore`, `create`, `send_message`,
  `purge_expired_threads`) · `src/agentic/mod.rs` (doc del módulo: la página de aterrizaje de
  `ThreadStore`, 3 líneas) · `src/gc.rs` (doc del módulo y de `register_ttl`/`sweep`) ·
  `src/sdk/builder.rs` (una línea de doc de `create_thread`; **ningún cambio de código**) ·
  `tests/message_thread_test.rs` (2 tests) · `docs/api/EMBEDDED_SDK.md` (§Threads: cabecera + 3 celdas
  de tabla) · este fichero · `docs/dev/Backlog.md` (sólo mi fila — la retiró un commit ajeno, así que
  este commit **no** la incluye: se verificó con `-ExpectAbsent`) ·
  `docs/dev/avance/activo/core-engine.md`

## Impacto mapeado (Regla 0)

- **Leído completo:** `src/agentic/thread.rs` (837 líneas: `create`, `send_message`, `get`, `list`,
  `delete`, `purge_expired_threads`, `is_expired`, los helpers de índice, `Clock`/`SystemClock`/
  `ManualClock` y los 6 tests del módulo) · `src/gc.rs` (548 líneas: `GcWorker` + `sweep` +
  `register_ttl` + `purge_ttl_for_deleted`, `MemoryTtlSweeper` y `spawn_memory_ttl_sweeper` con sus
  tests) · `tests/message_thread_test.rs` (239 líneas, incluye `test_thread_ttl_expiry` con
  `ManualClock`) · `src/sdk/builder.rs:340-414` (`thread_store`, `create_thread`, `send_message`,
  `purge_expired_threads`) · `src/sdk/api/memory.rs:2789-2845` (`Embedded::purge_expired`, el *otro*
  TTL: registros de memoria, indexado por `scalar_lookup_int_le`) · `src/agentic/mod.rs` (exports) ·
  `vantadb-mcp/src/threads.rs:20-60,120-160,200-240,410-425` (tools `thread_create`,
  `thread_purge_expired`) · `docs/api/EMBEDDED_SDK.md:505-515` (tabla de threads) ·
  `.opencode/rules/api-contract.md` (R-1, R-5), `.opencode/rules/durability.md` (scope `gc.rs`),
  `docs/dev/tasks/FIND-798.md` (convención de task file y el fix de `list` que tolera ids colgantes).
- **Referencias hacia dentro (quién depende de lo que documento):**
  `Embedded::create_thread`/`send_message`/`purge_expired_threads` (`src/sdk/builder.rs`),
  el servidor HTTP (`src/server/handlers.rs:1227-1260`, `POST/GET /api/v2/threads`), la tool MCP
  `thread_create`/`thread_purge_expired` (`vantadb-mcp/src/threads.rs`), `docs/api/EMBEDDED_SDK.md`.
  **Ninguna firma, tipo ni semántica pública cambia**: el diff es de comentarios `///` y `//!` más
  tests. No hay cambio de comportamiento que un consumidor pueda observar.
- **Referencias hacia fuera:** ninguna nueva. Los tests usan `ThreadStore::with_clock` +
  `ManualClock`, ya `pub` (`src/agentic/mod.rs:14`).
- **Veredicto:** cambio de documentación dentro de dos módulos (`gc.rs` y `agentic/thread.rs`) más
  tests de caracterización. Riesgo de deadlock: **cero** (no se toca ningún lock ni firma). Riesgo de
  regresión funcional: **cero** (ninguna rama de ejecución cambia). El riesgo real es de *contrato*:
  si el doc y el código divergen, el módulo vuelve a prometer un GC que nadie instancia.

## La premisa, verificada antes de tocar

| Afirmación de la fila | Verificación en `HEAD` (`cce353f10`) |
|---|---|
| `Embedded` siempre pasa `gc = None` | ✅ `src/sdk/builder.rs:374` (`create_thread`) y `:382` (`send_message`) — eran `:371`/`:379` antes de la corrección de doc de esta tarea |
| `send_message` desliza el `expires_at` | ✅ `src/agentic/thread.rs:375-384`: `new_expires_at = now + ttl_secs` y `insert` del nodo |
| `GcWorker` no se instancia en producción | ✅ **cero** en `src/` fuera de `gc.rs`: los `GcWorker::new` son 13 en `src/gc.rs` (bajo `#[cfg(test)]`) y 1 en `tests/storage/gc.rs` |
| El doc dice «auto-expiry» | ✅ `src/sdk/builder.rs:361` («Pass `ttl_secs` for auto-expiry»), `docs/api/EMBEDDED_SDK.md:509`, `vantadb-mcp/src/threads.rs:32`. El tool MCP ya acota («purged later via `thread_purge_expired`»); el SDK no. |
| Hay un mecanismo que sí ejecuta el TTL | ✅ `Embedded::purge_expired_threads` (`builder.rs:413`) → `ThreadStore::purge_expired_threads` (`thread.rs:449`), expuesto como tool MCP `thread_purge_expired`. **Nadie lo llama solo.** |

## La decisión: salida (b) — no se enciende el GC, el doc dice la verdad

El contrato admitía dos salidas honestas. Se elige **(b)** por cinco razones verificables en el
código, no por comodidad:

1. **El índice del GC no es durable.** `GcWorker` guarda las expiraciones en un `BTreeMap` en memoria
   (`src/gc.rs:30`). El deadline real ya vive **en el nodo** (`_expires_at`, `thread.rs:278`), que sí
   sobrevive a un reinicio. Encender el GC crearía una **segunda fuente de verdad** que nace vacía en
   cada arranque: tras un reinicio, ningún hilo con TTL registrado expiraría hasta que alguien llame
   al purge explícito igualmente. La propiedad no se vuelve durable por instanciarlo.
2. **El GC no compartiría estado sin un lock nuevo en el camino de escritura.** `Embedded::create_thread`
   construye un `ThreadStore` **por llamada** (`builder.rs:351-357`, `:365-367`), así que el worker
   tendría que ser un `Mutex<GcWorker>` en `Embedded` tomado en cada `create`/`send_message` (el lock
   cubriría sólo el `push` al `BTreeMap`, pero se toma en toda escritura de hilo) para resolver un
   deadline que ya está persistido en el nodo.
3. **El barrido produce ids colgantes por diseño.** `GcWorker::sweep` llama `StorageEngine::delete`
   directo (`src/gc.rs:85`), sin pasar por `ThreadStore::delete`, así que **no toca el índice**. El id
   queda en `_thread_ids`, y el único que lo recicla es el rebuild de **otro** `purge_expired_threads`
   que sí borre algo: ese rebuild sólo corre `if purged > 0` (`thread.rs:468`) y un id sin nodo no
   incrementa `purged` (ver §NOTICED #4). `list` ya lo tolera (`cce353f10`), pero el blob del índice
   —que se re-serializa entero en cada `create`— acumula basura que sólo se limpia de rebote. Es
   exactamente el productor de ids colgantes que el pre-mortem señalaba.
4. **El reloj del barrido no es inyectable.** `sweep` lee `web_time::SystemTime` (`gc.rs:67`), no el
   `Clock` del store. El test de TTL existente lo dice explícitamente: «`GcWorker::sweep` is
   intentionally not used: it reads wall time» (`tests/message_thread_test.rs:170`). Encenderlo
   pondría en producción un camino que ningún test puede controlar; el mecanismo actual
   (`purge_expired_threads` con `ManualClock`) sí es testeable al segundo.
5. **El mecanismo honesto ya existe y es mejor.** `purge_expired_threads` camina el índice, lee el
   nodo, evalúa `_expires_at` y **reconstruye el índice** en el mismo paso: es durable, index-driven,
   respeta el `Clock` inyectado y está expuesto (`Embedded` + tool MCP). Lo que faltaba no era un
   motor, era **decir que ése es el motor**.

### Coste medido / descartado

La salida (a) exigía medir el coste de encenderlo; al no encenderse, el coste de producción es
**0** (ningún tick, ningún escaneo de fondo, ningún lock nuevo). El coste que **sí** queda es el del
purge explícito, que ya existía y lo paga quien lo llama: `purge_expired_threads` hace **un `get` por
id del índice** y, **sólo si algo expiró**, un **segundo pase con otro `get` por superviviente** más
una reescritura completa del blob del índice — hasta **2N lecturas + 1 escritura** de índice
(O(hilos), constante 2). No se añade periodicidad — ver §NOTICED BUT NOT TOUCHING (#2), que es donde
esa decisión debe tomarse, con medición.

### Alternativas descartadas (y por qué)

- **Expiración lazy en lectura** (`get`/`list` devuelven `None` si `_expires_at < now`, ~3 líneas):
  daría «expirado = invisible» sin timer y sin lock nuevo, pero (i) deja el nodo **inmortal e
  invisible** — la clase exacta de `FIND-798`/`FIND-799` — y (ii) mataría el rescate por
  `send_message`, porque el escritor ya no vería el nodo. Descartada.
- **Encender el `GcWorker`** (salida (a)): ver las cinco razones de arriba.
- **Tope absoluto además del de inactividad**: requiere un campo nuevo (`_hard_expires_at`) y un
  cambio de contrato; fuera de alcance (fila aparte si el owner lo quiere).

## Los tests de la propiedad

**Marco honesto:** esta tarea no añade lógica, así que **no hay RED posible** — el defecto es una
promesa de doc sin mecanismo, y un test no puede fallar por un comentario. Los dos tests que se
añaden son **de caracterización**: fijan en ejecutable exactamente lo que el doc pasa a afirmar, y
fallan si alguien cambia el comportamiento documentado. Se declara como tal en vez de disfrazarlo de
TDD (la alternativa —forzar un RED artificial— habría sido teatro).

1. `expired_thread_is_readable_until_the_explicit_purge_runs` — un hilo con `ttl_secs = 60`:
   - a los **60 s exactos** el purge devuelve **0** (el borde es `_expires_at < now`, estricto,
     `thread.rs:486-488`) y el hilo sigue leyéndose;
   - a los **61 s** el hilo **sigue leyéndose** (`get` + `list`) porque *nada* evalúa el TTL en
     lectura — el doc ahora lo dice;
   - tras `purge_expired_threads()` devuelve **1**, el hilo desaparece de `get` y de `list`, y el
     índice queda vacío.
2. `send_message_re_arms_the_ttl_so_an_active_thread_never_expires` — el «deslizamiento» que la fila
   denunciaba, declarado como semántica:
   - `ttl = 60`, mensaje a los 50 s, y a los 100 s (40 s después del deadline original) el purge
     devuelve **0** y el hilo vive: el mensaje lo re-armó;
   - 61 s después del último mensaje, el purge lo borra. Es una ventana de **inactividad**, no un TTL
     absoluto, y está acotada: sin actividad, expira.

## Steps

1. ✅ Contexto cargado y premisa re-verificada contra `HEAD` (tabla arriba).
2. ✅ Decisión (b) justificada contra el código (`gc.rs`, `builder.rs`, `thread.rs`).
3. ✅ Tests de la propiedad (`tests/message_thread_test.rs`).
4. ✅ Doc corregido en `thread.rs` + `gc.rs` (+ la línea de `builder.rs`).
5. ✅ Verificación mecánica (población del filtro + `run` + fmt + clippy).
6. ✅ Review P2-01 (`vanta-review`, contexto fresco) → ronda 1 `changes-required` (5 Required, todos de
   doc/expediente, **cero de código**; 5 Optional/Nit); los 5 Required y los Optional aplicados
   (ver §Review).
7. ✅ Cierre — Backlog (mi fila), `avance/activo/core-engine.md`, commit aislado con `-ExpectAbsent`.

## El cambio de doc, línea por línea

| Dónde | Decía | Dice |
|---|---|---|
| `//!` de `src/agentic/thread.rs` | «Threads can opt into TTL-based expiry via `GcWorker`» | El deadline vive en el nodo (`_expires_at`); **lo evalúa sólo `purge_expired_threads`**, que nadie programa solo; `GcWorker` no se instancia en producción |
| doc de `ThreadStore` | «Callers that want TTL-based expiry pass a `GcWorker`» | El TTL se evalúa en el purge explícito; el `GcWorker` es un acelerador opcional que ningún camino de producción provee |
| doc de `create` | «The thread auto-expires after this many seconds» | Ventana de **inactividad** desde `created_at`; se re-arma con cada `send_message`; no hay borrado automático |
| doc de `send_message` | «The thread's TTL is refreshed by re-registering with the original duration» | **Es inactividad por diseño**: un hilo con actividad nunca caduca; sin actividad, el purge lo borra. Y el `gc` es no-op si nadie lo pasa |
| doc de `purge_expired_threads` | «Returns the number of threads purged» | **Éste es el mecanismo**: O(hilos), borde `< now`, reconstruye el índice sólo si borró algo |
| `//!` de `src/gc.rs` | «evicts expired entries from the StorageEngine on each sweep» | Aclara que **ningún camino de producción lo instancia** y que el TTL de hilos va por `purge_expired_threads`; `sweep` deja el id en el índice |
| doc de `register_ttl` | «Registers a node to be automatically expired» | Registra en un índice **en memoria** que no sobrevive al proceso |
| doc de `sweep` | (nota de que lo usan callers) | Nota el efecto colateral sobre `_thread_ids` y remite al purge |
| doc de `Embedded::create_thread` (`builder.rs`) | «Pass `ttl_secs` for auto-expiry» | «Pass `ttl_secs` for a TTL window … of **inactivity** … evaluated by `purge_expired_threads`: there is no background expiry» |

`docs/api/EMBEDDED_SDK.md` §Threads también se corrigió (decía «auto-expiry» en la fila de
`create_thread` y describía el TTL como si fuera automático): una celda de tabla y el párrafo de
cabecera, sin tocar código. `vantadb-mcp/src/threads.rs:32` dice «auto-expiry» y **no se puede
tocar** (ruta prohibida): ver §NOTICED BUT NOT TOUCHING (#1).

## Verificación

- **Contrato mecánico — población:** `cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(gc) or test(thread) or test(ttl) or test(expire)'`
  → **45** tests, de los cuales 6 son de `agentic::thread` (módulo unitario), 13 de `gc`, 3 de
  `message_thread_test` (las 2 nuevas + `test_thread_ttl_expiry`), 4 de `quarantine_containment` +
  `write_receipts`, y el resto de TTL de memorias/skills/serialización/versiones.
- **Contrato mecánico — corrida:** `cargo nextest run` con **el mismo** filtro + `--no-tests fail`
  → **45/45 passed**, 2969 skipped (2,7 s). Las dos nuevas incluidas.
- `cargo fmt --check` → ✅
- `cargo clippy -p vantadb --lib --test message_thread_test -- -D warnings` → ✅
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → ✅ (2m34s)
- `node scripts/docs/check-links.mjs` → ✅ (0 broken markdown) · `node scripts/docs/check-docs.mjs`
  → ✅ (gating all clear).
- **Nota de la ventana compartida:** durante la medición el árbol pasó por **dos roturas ajenas**
  (`tests/certification/glove_dimension_scale.rs` sin `HnswConfig::auto_tune`; `src/storage/vfile_crypto.rs`
  con `use std::path::Path` gateado a `encryption` pero la función sin gatear). Ambas las resolvieron
  sus carriles; los números de arriba son la **corrida final con el árbol sano**. `--all-features`
  **no** sirve como sustituto en este equipo: activa `rocksdb` y el link rompe por `LNK2038`
  (`RuntimeLibrary` MD vs MT) en artefactos cacheados de otro build — no es un fallo del diff.

## Techo declarado

- **No hay expiración automática.** Tras esta tarea el TTL de hilos se evalúa **cuando alguien llama**
  `purge_expired_threads` (tool MCP `thread_purge_expired` o `Embedded::purge_expired_threads`). Un
  despliegue que nunca lo llame acumula hilos vencidos. Es el estado honesto, no el ideal: la
  periodicidad es una decisión de coste que necesita medición (§NOTICED #2).
- **El TTL es de inactividad, no absoluto.** Un hilo con un mensaje cada 59 s con `ttl = 60` nunca
  caduca. Es la semántica que el código ya tenía; ahora está escrita. Si se quisiera un tope absoluto
  haría falta un campo nuevo (`_hard_expires_at`) y un cambio de contrato — no se hizo.
- **El borde es estricto:** `_expires_at < now`, así que en el segundo exacto del deadline el hilo
  todavía vive. Fijado por test.
- **`ttl_secs = Some(0)` no es «sin TTL»:** sella `_expires_at = now`, expira al segundo siguiente y
  **no se puede rescatar** (el re-armado exige `_ttl_secs > 0`). Documentado en el doc de `create`.
- Los tests nuevos usan `ThreadStore::with_clock` (nivel store), no `Embedded`: el `Embedded` usa
  `SystemClock` (`builder.rs:352`) y no hay costura para inyectar reloj por el handle. La ruta
  `Embedded::purge_expired_threads` → `ThreadStore::purge_expired_threads` es una delegación de una
  línea (`builder.rs:413-417`), sin lógica propia que cubrir.

## NOTICED BUT NOT TOUCHING

1. **`vantadb-mcp/src/threads.rs:32` sigue diciendo «auto-expiry».** Es la única superficie que queda
   con el término: `vantadb-mcp/**` estaba prohibido en esta tarea. Riesgo bajo y acotado por el
   propio texto del tool («purged later via `thread_purge_expired`»), así que no promete borrado
   automático, sólo usa la palabra. **Fila de docs pendiente** (`FIND-*`): el resto de superficies ya
   dice la verdad. No se creó la fila desde acá porque `.opencode/**` (donde vive el routing de
   hallazgos) está prohibido y otro carril está tocando `Backlog.md`.
2. **El barrido del servidor no purga hilos.** `spawn_memory_ttl_sweeper` (`src/gc.rs:187`) corre
   `db.purge_expired()` cada `ttl_sweep_interval_ms`, y `purge_expired` (`src/sdk/api/memory.rs:2791`)
   sólo barre **registros de memoria** (por `expires_at_ms` + índice escalar). Enganchar
   `purge_expired_threads()` ahí daría expiración automática en despliegues con `feature = "server"`
   a cambio de **un `get` por hilo en cada tick** — sin índice escalar sobre `_expires_at` (segundos),
   sería O(hilos)/tick, que es justo el escaneo O(n) que el pre-mortem prohíbe adoptar sin medir. No
   se hizo: requiere medir a 1k/10k/100k hilos primero (fila propia, con el número).
3. **Carrera purge ↔ `send_message` (no reproducida, no arreglada).** `purge_expired_threads` toma
   `index_lock`; `send_message` toma `message_lock` (`thread.rs:354`). Un `send_message` que haya
   hecho el `get` **antes** del delete del purge y el `insert` **después** re-crea el nodo (y el purge
   ya lo contó como purgado); si el rebuild del índice ya pasó por ese id, el nodo queda **sin id en
   el índice** (hilo invisible e inmortal: la clase de `FIND-798`, desde el otro lado). Es una ventana
   estrecha y **no se pudo reproducir de forma determinista**, así que no se toca: **lo que falta es
   una costura de test** — `ThreadStore` sostiene un `&StorageEngine` concreto, no hay punto de
   inyección entre el `delete` del purge y el rebuild, ni forma de suspender `send_message` entre `get`
   e `insert`; y un fix sin test de reproducción no se verifica. El arreglo probable es tomar
   `message_lock` durante el purge (orden `index_lock → message_lock`, **sin inversión**: `create` y
   `delete` sólo toman `index_lock`, `send_message` sólo `message_lock`, así que no hay ciclo), pero
   eso es otra tarea con su costura.
4. **`purge_expired_threads` reconstruye el índice sólo `if purged > 0`** (`thread.rs:468`): ids
   colgantes de cualquier origen se acumulan hasta que *otro* hilo expire en el mismo barrido. Ya
   declarado en `FIND-798` §NOTICED; sigue igual.
5. **El parámetro `gc` sigue en la firma pública** de `ThreadStore::create`/`send_message` y ahora su
   doc dice que producción nunca lo pasa. Retirarlo es un cambio de API pública (semver) y no aporta
   al cierre de esta tarea: fila aparte si se quiere limpiar.
6. **`gc.rs` usa `expiry <= now` y el purge de hilos `< now`**: los dos mecanismos discrepan en el
   segundo exacto de un deadline. Irrelevante mientras el GC no se instancie; anotado en el doc de
   `sweep` para que nadie «armonice» uno solo.

## Review

**P2-01, ronda 1 — `vanta-review`, contexto fresco: `changes-required`** (5 Required, todos de
doc/expediente, **cero de código**; 5 Optional/Nits). Todos aplicados:

| # | Finding | Aplicado |
|---|---|---|
| R1 | El task file declaraba un `approve` inexistente: paso 6 ✅ `approve` contra §Review vacía, y `status: in-progress` contra "✅ COMPLETED" | Estado real escrito (ronda 1 = `changes-required`), `status: completed`, esta tabla |
| R2 | `NOTICED #1` afirmaba que `docs/api/EMBEDDED_SDK.md:509` seguía con «auto-expiry» y que `docs/api/**` estaba fuera del write set: **ambas falsas** (el diff lo edita), y el fichero faltaba en §Archivos tocados | NOTICED #1 reescrito para citar **sólo** `vantadb-mcp/src/threads.rs:32`; `docs/api/EMBEDDED_SDK.md` y `src/agentic/mod.rs` añadidos a §Archivos tocados |
| R3 | `gc.rs:9-10` decía «only in **this file's** tests»: falso, `tests/storage/gc.rs:33` también construye un `GcWorker` | «nothing outside the tests (`src/gc.rs` **and** `tests/storage/gc.rs`) constructs one» |
| R4 | El coste del purge estaba subestimado: con `purged > 0` hay un **segundo** pase de `get` por superviviente + rewrite del blob | Corregido en `thread.rs` (doc de `purge_expired_threads`), en §Coste y en el avance: «hasta 2N lecturas + 1 escritura» |
| R5 | `src/agentic/mod.rs:1,5` — la página de aterrizaje de `ThreadStore` — seguía prometiendo «TTL-based garbage collection via `GcWorker`» | Los 3 primeros párrafos reescritos; ya no nombra el GC como mecanismo |
| O1 | Alternativa *lazy on read* no evaluada | §"Alternativas descartadas": descartada con las dos razones (nodo inmortal e invisible; mata el rescate por `send_message`) |
| O2 | Edge `ttl_secs = 0`: expira al segundo siguiente y **no se puede rescatar** | Añadido al doc de `create` y al §Techo |
| O3 | El parámetro `gc` sigue en la firma pública y ahora está documentado como no usado en producción | §NOTICED #5: retirarlo es breaking (semver), fila aparte |
| O4 | El techo (e) atribuía el bloqueo a la Regla 8; el orden `index_lock → message_lock` es demostrablemente seguro | Reformulado: lo que falta es una **costura de test**; la seguridad del orden queda afirmada con la evidencia (`create`/`delete` sólo `index_lock`, `send_message` sólo `message_lock`) |
| O5 | `gc.rs` usa `expiry <= now` y el purge `< now` | Nota en el doc de `sweep` para que nadie «armonice» uno solo |
| N1-N5 | Citas desplazadas (`gc.rs:67` → `:67`, `gc.rs:29` → `:30`, `gc.rs:79` → `:85`, refs `thread.rs` en los tests), «14 hits» → 13 + 1, y «nada lo recicla» contradiciendo NOTICED #4 | Todos corregidos por barrido |

**Lo que el reviewer confirmó como correcto:** que el diff de `src/` es **sólo comentarios** (leyó
cada línea añadida); que la salida (b) es la decisión adecuada y 4 de sus 5 razones son literalmente
verificables en el código; que los dos tests **fallan en ambas direcciones** (no son teatro de TDD);
y que declarar los techos —incluida la carrera no arreglada y la fila de MCP en ruta prohibida— es el
estándar. Verificó por su cuenta la premisa con `rg "GcWorker::new"` (13 en `src/gc.rs`, 1 en
`tests/storage/gc.rs`, **0** en producción) y corrió los dos tests nuevos: **2/2 PASS**.

**P2-01, ronda 2 — `vanta-review`, contexto fresco: `approve`** (commit `16619fe0c`). 0 Critical,
0 Required. Verificó el delta contra el commit, no contra este documento: leyó las 8 rutas
completas y confirmó que **todas** las líneas añadidas en `src/` son comentarios; re-corrió
`cargo nextest run --profile audit -p vantadb -E 'test(gc) or test(thread) or test(ttl) or test(expire)'
--no-tests fail` → **45/45, 2969 skipped** (coincide al dígito); `cargo fmt --check` → 0;
`cargo doc -p vantadb --no-deps --lib` → 0 warnings en los 4 ficheros del commit (los 2 avisos que
salen son de `src/wal.rs:2201/2203`, ajeno) ⇒ **los intra-doc links nuevos resuelven**;
`check-docs` → GATING all clear; `check-links` → 0 broken; `git show 16619fe0c:docs/dev/Backlog.md`
→ sin fila propia (la retiró otro carril); `rg -c "^### FIND-800"` en el avance → **1**, sin
duplicado. **No pudo re-verificar `clippy`** porque el árbol estaba roto por terceros en el minuto
del intento (`src/migration.rs:170,188`, `src/cli_handlers/backup.rs:469`, ninguno del commit) —
declarado en vez de escrito como ✅; el mismo lib compiló y pasó 45/45 cinco minutos antes. Residual
aplicado tras el veredicto (no bloqueaba): `src/agentic/mod.rs:1` pasa de «lazily-evaluated TTL» a
**«explicitly-purged TTL»** (el término «lazy» era el de la alternativa **descartada**), se corrige
la concordancia en el doc de `send_message`, y se realinean tres citas (`gc.rs:67`, `gc.rs:187`,
`thread.rs:375-384`).
