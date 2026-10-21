---
title: "FIND-343 - Borrado fisico incompleto: el payload sigue legible en el WAL rotado y el journal"
kind: task
status: review
description: "Delimitar y corregir la retencion del payload de un registro borrado: el WAL rotado y el journal de Fjall lo conservan."
tags: [durabilidad, wal, privacidad, critica]
---

# FIND-343: Borrado fisico incompleto: el payload sigue legible en el WAL rotado y en el journal Fjall

Tras un `delete` (logico), el **payload** sigue siendo recuperable: en el **WAL rotado**
(`vanta.wal.<ts>`, los shards rotados por auto-rotate) y en el **journal de Fjall** (el backend).
Es decir: un registro borrado no ha borrado sus bytes, solo ha marcado su clave como borrada. Un atacante
con acceso al disco (o un `strings` sobre los ficheros) recupera el contenido.

El alcance de W6 (cifrado en reposo) **cambia el marco**: con la feature `encryption` activada, el WAL y el
backend estan cifrados ⇒ el payload ya no esta en claro ⇒ el problema pasa a ser (a) el **cifrado en reposo no
esta activo** por defecto (la feature es opt-in) y (b) la **retencion** del material cifrado: rotar el WAL no
borra las DEKs antiguas, y un ciphertext de un registro borrado sigue descifrable con la clave vigente.

Ojo: el cifrado **no** es la remezcla del borrado fisico (un `delete` fisico real debe compactar/reescribir los
segmentos), y ademas el borrado fisico tiene un coste que hay que medir (Regla 9). Esta tarea es de **durabilidad
y privacidad**, y su alcance hay que delimitar antes de codificar.

> **Estado:** 🟡 IMPLEMENTADO Y VERIFICADO (la mitad WAL) - falta la review P2-01 de un agente DISTINTO
> (no se marca COMPLETED sin ella). El contrato **NO** queda cerrado: el journal del LSM (Fjall) sigue
> reteniendo los bytes en claro ⇒ ver `## Alcance real` y `## Deuda que sale de este trabajo`.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-343 (critica))
- **Evidencia de la auditoria:** fila del backlog (EGO-16) + `tests/certified_delete.rs::physical_erasure_no_payload_bytes_after_full_maintenance` (en `#[ignore]` como pin)
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 8-12
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T20:10
- **Estado:** 🟡 EN REVISION (implementado y verificado; falta la review P2-01 de otro agente)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución (6/6 hechos) - queda el gate P2-01

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | src/storage/engine/maintenance.rs (`compact_wal`), src/wal_shipping.rs (lee los segmentos), src/storage/archive.rs |
| Callees | src/wal.rs, src/wal_sharded.rs, src/backends/**, src/storage/archive.rs |
| Implicaciones | el borrado fisico tiene un coste (reescribir segmentos) y un riesgo (perder datos si el crash ocurre a mitad). El cifrado en reposo ya mitiga la lectura en claro, pero no borra el material cifrado. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** src/wal.rs (auto-rotate y retencion de segmentos), src/wal_sharded.rs, src/storage/engine/delete.rs (borrado logico), src/backends/ (el journal de Fjall), src/storage/archive.rs (compactacion)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** wal.rs -> storage::vfile_crypto + crypto (el envelope); backends/ -> journal de Fjall; delete.rs -> backend + wal
- **Archivos que referencian a los editados (referencias entrantes):** src/storage/engine/ops.rs, src/storage/engine/txn.rs, src/wal_shipping.rs (lee los segmentos), src/storage/archive.rs
- **Veredicto impacto:** alto: es el unico hallazgo de privacidad de datos en el backlog. Toca el hot path del WAL y el formato en disco ⇒ cualquier cambio de formato necesita version (WAL v4 ya existe) y el shipper (`FIND-496`) lo tiene que poder leer.
- **Ficheros tocados por este trabajo (y solo estos):** `src/wal.rs`, `src/wal_sharded.rs`, `src/storage/engine/delete.rs` (+ sus tests). Nada de `src/parser/**`, `src/planner.rs`, `src/physical_plan/**`, `src/cost_estimator.rs`, `src/index/**`, `src/crypto*`, `src/storage/engine/init.rs`, `src/config.rs`, `src/lib.rs`, `benches/**`, `tests/encryption_chaos.rs`.

## Contrato
"Tras `delete` + `vacuum`/compactacion, el payload **no** es recuperable ni del WAL rotado ni del journal: un `strings`/busqueda de bytes del payload en el directorio de datos **no** lo encuentra (con la feature `encryption` activada: el contenido esta cifrado y el registro borrado no se puede descifrar sin la DEK vigente)."

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Medición (el entregable que decide el alcance)

Canario: `FIND343-PROBE-payload-debe-desaparecer-7f3a9c1e5b2d4860` (solo existe en el payload del test).
Flujo: `put` → `delete` → `vacuum()` → `pipeline(MergeOnly)` → `compact_wal()` → `flush()` → `close()` → escaneo
recursivo de bytes en todo el directorio de datos.

| Modo | Ficheros donde el payload sigue legible **antes** del fix | **Despues** del fix |
|------|----------------------------------------------------------|---------------------|
| plaintext (features por defecto) | `data/vanta.shard0.wal` (segmento WAL vivo) + `<db>/0.jnl` (journal de Fjall, 64 MiB preasignado) | **solo** `<db>/0.jnl` |
| `--features encryption` + clave | **ninguno** (0 hits en todo el directorio) | **ninguno** (0 hits) |

Hallazgos de la medición (verificados leyendo el código, no supuestos):

1. **El "WAL rotado" del hallazgo no existía como tal.** `ShardedWal::rotate_all()` (`src/wal_sharded.rs`) **no
   rota**: reabre el mismo fichero (`WalWriter::open_with_buffer` con `truncate(false)`) ⇒ el segmento vivo
   acumula **todos** los payloads escritos y nunca se acorta. `compact_wal()` (`maintenance.rs:129`) es por tanto
   un no-op a efectos de bytes.
2. **No hay ninguna politica de retencion** de los segmentos rotados: el auto-rotate (`try_auto_rotate`, 256 MiB)
   renombra `vanta.shardN.wal` → `vanta.shardN.wal.<ts>` y **nadie borra** ese fichero (unicos `remove_file` del
   codigo: el overwrite del archive en `rotate`/`try_auto_rotate`). El shipper (`FIND-496`) los lee, pero no los purga.
3. **El journal del LSM es del backend, no de VantaDB.** Fjall escribe cada `put` en su journal (`0.jnl`,
   `PRE_ALLOCATED_BYTES = 64 MiB`) y solo lo borra cuando el segmento ha sido **rotado** (a los 512 MiB de journal
   escritos: `fjall::Config::max_journaling_size_in_bytes`) **y** el LSM ya ha flusheado el keyspace. El segmento
   **activo** nunca se purga ⇒ los bytes de un registro borrado quedan en el journal indefinidamente.
4. **Con cifrado en reposo el problema desaparece**: el WAL va en v4 (frame AEAD) y los valores del backend son
   sobres AEAD (`EncryptedBackend`, `src/backends/kv_crypto.rs`) ⇒ 0 hits. Lo que queda no es "legible" sino
   "descifrable con la DEK vigente" (crypto-shredding ⇒ fuera del boundary: `src/crypto/**`).

## Invariantes de dominio (handoff — MUST)

| # | Invariante | Estado | Como se verifica |
|---|------------|--------|------------------|
| 1 | **No se puede perder datos**: una compactacion interrumpida por un crash deja la base recuperable (WAL + replay) | ✅ CUMPLIDO | El reescrito es `tmp → fsync → rename → fsync(dir)`: un crash antes del rename deja el segmento original intacto. El segmento cifrado v4 se salta (no se toca). `wal::tests::redact_erased_keeps_the_replayed_state` comprueba que el estado reproducido es identico |
| 2 | El WAL rotado no se puede borrar antes de que su contenido este en un segmento vigente (o en un snapshot) | ✅ CUMPLIDO (por construccion) | Los segmentos rotados **no se borran**: se reescriben *in situ* con el mismo nombre ⇒ el shipper (`*.wal.*`) los sigue encontrando y el recovery (que solo lee el segmento vivo) no cambia |
| 3 | Formato en disco versionado: un cambio necesita subir de version y el lector viejo debe seguir leyendo | ✅ CUMPLIDO (sin cambio de formato) | El frame es el mismo (`[len][payload][crc][prev‖hash]` v3); solo cambian los *bytes* de los registros redigidos y se recalcula la cadena VER-01. `wal::tests::redact_erased_replaces_the_payload_with_a_tombstone` verifica `verify_wal_file` → `Verified` |
| 4 | El shipper (`FIND-496`) no puede romperse | ✅ CUMPLIDO | `discover_segments` (`wal_shipping.rs:262`) busca `name.contains(".wal.") && !name.ends_with(".wal")`: el fichero conserva su nombre ⇒ se sigue descubriendo. Se le manda una historia **sin el payload** (y por tanto valida). El scratch del reescrito **no** lleva `.wal.` a proposito, para que el shipper no lo lea |
| 5 | El borrado fisico no puede degradar el hot path sin medirlo (Regla 9) | ✅ CUMPLIDO (medido) | La pasada corre **solo** en `compact_wal()` (operacion de mantenimiento explicita), jamas en `insert`/`delete`. El camino caliente no cambia. Coste medido abajo |
| 6 (nuevo) | El conteo de registros del WAL no puede cambiar (semantica de `checkpoint_seq`) | ✅ CUMPLIDO por construccion | El registro borrado **no se elimina**: se **reemplaza** por un `Delete{id}` sin payload ⇒ el conteo (y por tanto el "los primeros N registros ya estan aplicados") es exactamente el mismo antes y despues. Test: `record_count` antes == despues (en `wal::tests` y `wal_sharded::tests`) |

### El coste medido del borrado fisico (Regla 9)

`compact_wal()` completo (flush + backend flush + snapshot del indice + **borrado fisico**) sobre una base con N
registros de ~1 KiB de payload, perfil de test (`opt-level = 0`, el mas pesimo):

| registros | WAL antes | WAL despues | `compact_wal()` | rendimiento del reescrito |
|-----------|-----------|--------------|-----------------|----------------------------|
| 10    | 15 KB    | 1.9 KB  | 167 ms | 0.1 MiB/s |
| 100   | 152 KB   | 18 KB   | 217 ms | 0.7 MiB/s |
| 1.000 | 1.52 MB  | 184 KB  | 457 ms | 3.2 MiB/s |
| 5.000 | 7.61 MB  | 918 KB  | 1.8 s  | 4.0 MiB/s |

Lectura: el tiempo total de `compact_wal` tiene un suelo de ~165 ms (el flush + el snapshot del indice, no el
borrado fisico) y **crece linealmente con el tamano del segmento** ⇒ el reescrito es O(bytes del segmento). El
WAL se encoge ~88 % (los bytes del payload desaparecido). El techo conocido: el auto-rotate marca los 256 MiB ⇒
un segmento a tope cuesta una copia extra de si mismo por compactacion (`ponytail:` en el codigo). El hot path
(insert/delete) **no** se toca: el borrado fisico es una operacion de mantenimiento explicita.

## Spec (SDD) — el fix

**Idea (la que evita el "rediseño de Engine" que la fila del backlog declaraba imposible):** un `delete` ya es un
tombstone. El problema es que el `Insert` anterior (que lleva el payload) **sigue en el WAL**. En vez de *borrar*
el registro (lo que romperia el conteo y la semantica de `checkpoint_seq`), se **sustituye por un tombstone sin
payload**: `WalRecord::Delete { id }`. De aqui sale todo:

- el **conteo** de registros no cambia ⇒ `checkpoint_seq` sigue significando exactamente lo mismo ⇒ el recovery
  salta el mismo numero de registros antes y despues;
- el **estado reproducido** es identico: un nodo cuyo ultimo registro es un `Delete` acaba ausente igual que con
  su `Insert` historico;
- la **cadena VER-01** se recalcula sobre los frames nuevos (el CRC y el `record_hash` cubren el payload ⇒
  cambiarlo cambia el hash ⇒ hay que rehacer la cadena del segmento);
- el **formato en disco no cambia** (mismo `WalHeader`, mismo framing) ⇒ no hay que subir a v5 y el lector viejo
  sigue leyendo.

**Regla de borrado (global, no por shard):** un nodo se borra cuando su **ultimo registro efectivo** en el orden
global del WAL (el mismo que usa el recovery: ordenado por `(indice por shard, indice de shard)`, con el indice
por shard continuando a traves de los segmentos rotados) es un `Delete`. "Efectivo" excluye los registros de una
transaccion que **nunca commiteo** (un `Delete` dentro de una transaccion abortada nunca ocurrio ⇒ el nodo **no**
se borra). La transaccion se resuelve leyendo todos los `Commit`/`Abort` antes (el marcador de commit puede
estar en otro segmento que los registros que cierra).

**Donde corre:** `ShardedWal::redact_erased()` (nuevo), invocado desde `ShardedWal::rotate_all()`, que es a su vez
el hook de `StorageEngine::compact_wal()` (`maintenance.rs:133`) y de `vanta-cli wal compact`. El paso es
**oportunista**: un segmento cifrado (v4), una cola corrupta o un segmento sin nada que borrar se dejan tal cual,
y un fallo **nunca** hace fallar la operacion de mantenimiento (es una mejora de privacidad, no un paso de
durabilidad).

## Deuda técnica (Regla 6 — MUST)

Sin deuda. La supresion de DEK por borrado (crypto-shredding) y la reescritura completa de la base para purgar el
journal del LSM **no** son deuda de este trabajo: estan fuera del boundary (ver `## Deuda que sale de este trabajo`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + los tests del cambio en verde |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica (nunca auto-reporte) — **lo commitea el orquestador, no este agente** |
| **Release** | No aplica (sin release en esta iteracion) |

## Herramientas necesarias
- cargo (check, clippy, fmt) — con `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`
- codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** test-driven-development (el fix necesita un test que fije el
comportamiento roto antes del fix) + systematic-debugging (causa raiz, no sintoma) +
security-and-hardening (cuando el hallazgo es de integridad de datos) +
context-engineering + incremental-implementation.

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).
- **Evidencia verificada (no supuesta)**: el pin `tests/certified_delete.rs::physical_erasure_no_payload_bytes_after_full_maintenance`
  (EGO-16) fallaba con `["<tmp>/0.jnl", "<tmp>/data/vanta.shard0.wal"]`. **Despues** del fix el mismo pin falla
  **solo** con `["<tmp>/0.jnl"]` ⇒ la mitad WAL esta resuelta, la mitad LSM no.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (6/6 steps hechos) - queda el gate P2-01 |
| % completado | 85% (la mitad WAL resuelta; el contrato completo depende del journal del LSM) |

## Steps

1. ✅ **Delimitar el alcance ANTES de codificar**: medido donde queda el payload tras un delete
   (grep de bytes en el WAL rotado, en los shards, en el journal de Fjall y en los snapshots) y con que feature
   activada ⇒ ver `## Medición`. Resultado: **2** ficheros en plaintext (segmento WAL + journal Fjall), **0** con
   `--features encryption`. El alcance se reduce a "la mitad WAL" (VantaDB es dueno del formato) y el journal del
   LSM queda como residual documentado.
2. ✅ **Test primero** (que falla): los tests nuevos se escribieron **antes** del fix y se verificaron fallando
   (RED). Con el fix deshabilitado (un `if` temporal, ya retirado) los dos tests end-to-end fallan con
   `the deleted payload survived in the WAL segments: [".../data/vanta.shard0.wal"]`; con el fix pasan.
3. ✅ **Fix**: `WalWriter::redact_erased` + `ShardedWal::redact_erased` (sustitucion por tombstone sin payload,
   con el conteo de registros intacto y la cadena VER-01 recalculada) + el enganche en `rotate_all()`.
4. ✅ **Versionar el formato si cambia**: **no cambia**. El frame v3 (y el v4 cifrado, que se salta) siguen siendo
   los mismos ⇒ no hay v5, el lector viejo sigue leyendo. La cadena se recalcula ⇒ `verify_wal_file` sigue
   dando `Verified`.
5. ✅ **Medir** el coste del borrado fisico (Regla 9) ⇒ tabla en `## Invariantes de dominio`. El hot path no se toca.
6. ✅ **Verificar**: `cargo fmt --check` (limpio), `cargo clippy -p vantadb --all-targets -- -D warnings` (exit 0),
   suite del lib completa (**2525/2525** en el perfil por defecto, **2634/2634** con `--features encryption`) e
   integracion (564 tests de WAL/durabilidad/certified-delete/storage: **564/564**).

## Tests nuevos (con su nombre)

| Test | Fichero | Que fija |
|------|---------|----------|
| `wal::tests::redact_erased_replaces_the_payload_with_a_tombstone` | `src/wal.rs` | el payload desaparece del segmento, el **conteo** de registros no cambia y la cadena VER-01 sigue verificada |
| `wal::tests::redact_erased_keeps_the_payload_of_a_live_node` | `src/wal.rs` | un nodo **no** borrado conserva su payload |
| `wal::tests::redact_erased_keeps_the_replayed_state` | `src/wal.rs` | el estado reproducido es identico antes y despues del reescrito |
| `wal::tests::redact_scratch_file_is_not_discoverable_by_the_shipper` | `src/wal.rs` | el fichero scratch del reescrito **no** lleva el infix `.wal.` ⇒ el shipper (`*.wal.*`) no lo lee |
| `wal_sharded::tests::redact_erased_erases_across_shards` | `src/wal_sharded.rs` | el `Insert` en el shard 0 y el `Delete` en el shard 1 ⇒ el payload se borra (el set de borrado es **global**) |
| `wal_sharded::tests::redact_erased_erases_the_rotated_segments` | `src/wal_sharded.rs` | el payload tambien se borra de los segmentos **rotados** (`vanta.wal.<ts>`) |
| `storage::engine::delete::physical_erasure::deleted_payload_is_erased_from_the_wal_segments` | `src/storage/engine/delete.rs` | E2E: tras `put` + `delete` + `compact_wal` el payload **no** aparece en `data/` |
| `storage::engine::delete::physical_erasure::measurement_reports_where_the_deleted_payload_survives` | `src/storage/engine/delete.rs` | la medicion (WAL limpio + journal del LSM con el payload ⇒ residual documentado) |
| `storage::engine::delete::physical_erasure::deleted_payload_is_not_readable_anywhere_with_at_rest_encryption` | `src/storage/engine/delete.rs` | con `--features encryption` **0 hits** en todo el directorio de datos |

## Alcance real (lo que queda y por que)

| Superficie | Estado tras este trabajo | Por que |
|------------|--------------------------|--------|
| Segmento WAL vivo (`data/vanta.shardN.wal`) | ✅ **payload borrado** | Es de VantaDB: el fix lo reescribe |
| Segmentos WAL rotados (`vanta.shardN.wal.<ts>`) | ✅ **payload borrado** (in situ, sin borrar el fichero ⇒ el shipper sigue funcionando) | Igual que arriba |
| Journal del LSM (`<db>/0.jnl`, Fjall) | ⚠️ **payload retenido** | El journal es del backend: Fjall escribe cada `put` ahi y solo purga un segmento cuando ha rotado (512 MiB escritos) **y** el LSM ha flusheado. El segmento activo nunca se purga. Purgarlo exigiria **reescribir la base entera** (export → base nueva → swap), que es una feature nueva, no un fix. **Mitigacion: el cifrado en reposo** (0 hits medidos) |
| Material cifrado de un registro borrado (modo cifrado) | ⚠️ **descifrable con la DEK vigente** | Requiere crypto-shredding (destruir la DEK por registro) ⇒ `src/crypto/**`, fuera del boundary de este trabajo |

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

### Checklist para el revisor (P2-01)

- [ ] **Reproducir la medicion**: el pin `tests/certified_delete.rs::physical_erasure_no_payload_bytes_after_full_maintenance`
      (EGO-16) debe fallar **solo** por `0.jnl` (antes fallaba por `0.jnl` **y** `data/vanta.shard0.wal`).
- [ ] **El set de borrado es global y correcto**: un `Insert` en el shard 0 + un `Delete` en el shard 1 borra el
      payload del shard 0 (`wal_sharded::tests::redact_erased_erases_across_shards`); un `Delete` dentro de una
      transaccion **abortada** NO borra el payload (revisar la logica de `committed` en `redact_erased`).
- [ ] **El conteo de registros no cambia** (invariante 6): es la garantia de que `checkpoint_seq` sigue
      significando lo mismo. Verificar que el redigido es una *sustitucion* (`Insert` → `Delete{id}`), no un borrado.
- [ ] **El estado reproducido es identico**: replay de un WAL redigido == replay del WAL original (test
      `wal::tests::redact_erased_keeps_the_replayed_state`).
- [ ] **La cadena VER-01 sigue valida**: `verify_wal_file` → `Verified` sobre el segmento reescrito; el comando
      `vanta-cli wal verify` no reporta manipulacion.
- [ ] **Crash-safety**: el reescrito es `tmp → fsync → rename → fsync(dir)`. Un crash antes del rename deja el
      segmento original ⇒ la base sigue recuperable (invariante 1). Revisar que en Windows el handle del
      segmento se cierra antes del rename (`std::fs::rename` no pisa un fichero abierto).
- [ ] **El shipper no se rompe**: `WalShipper::discover_segments` sigue encontrando los segmentos (mismo nombre) y
      el scratch del reescrito **no** lleva `.wal.` (si lo llevara, el shipper leeria un fichero a medio escribir)
      — pin: `wal::tests::redact_scratch_file_is_not_discoverable_by_the_shipper`.
- [ ] **El hot path no se degrada** (invariante 5): la pasada corre solo en `compact_wal()`; `insert`/`delete` no
      cambian. Revisar el `ponytail:` del techo (256 MiB) y el coste medido.
- [ ] **El cifrado (v4) se salta, no se rompe**: un WAL cifrado no se reescribe (el payload ya es ciphertext) ⇒
      `redact_erased` devuelve `skipped` y el segmento queda intacto. Verificar que el modo cifrado sigue verde
      (2633/2633 con `--features encryption`).
- [ ] **Sin cambios fuera de scope**: solo `src/wal.rs`, `src/wal_sharded.rs` y `src/storage/engine/delete.rs`.
      Ningun fichero de `src/parser/**`, `src/planner.rs`, `src/physical_plan/**`, `src/cost_estimator.rs`,
      `src/index/**`, `src/crypto*`, `src/storage/engine/init.rs`, `src/config.rs`, `src/lib.rs`, `benches/**` ni
      `tests/encryption_chaos.rs` fue tocado.
- [ ] **Sin commits**: el commit lo hace el orquestador.

## Deuda que sale de este trabajo (NO en el scope de FIND-343)

- **Purga del journal del LSM**: para que el payload de un registro borrado desaparezca tambien de `<db>/0.jnl`
  haria falta **reescribir la base** (export → base nueva → swap atomico) o un backend con journal cifrado.
  Propuesta: una task de `vanta-arch` + `vanta-worker` ("rewrite the database to purge the LSM journal"), con su
  propio analisis de coste. Mientras tanto, el **cifrado en reposo** es la mitigacion (y deberia ser el
  default recomendado, no opt-in).
- **Crypto-shredding** (destruir la DEK por registro borrado ⇒ el ciphertext deja de ser recuperable): vive en
  `src/crypto/**` + el registro de claves. Es la forma de cerrar el punto (b) del marco W6.
- **Retencion de los segmentos WAL rotados**: siguen creciendo sin limite (nadie los borra). Este trabajo los
  reescribe (payload fuera) pero no los borra ⇒ el tamano del disco sigue creciendo. Una politica de retencion
  (borrar los rotados ya despachados por el shipper y con su contenido checkpointed) es otra task.
- **La fila del backlog** (`docs/dev/Backlog.md`, FIND-343) sigue marcando "Pendiente" y dice "NO fix worker" /
  "exige rediseño de Engine": el rediseño **no** fue necesario (la sustitucion por tombstone conserva el conteo
  y el estado). Actualizar la fila (lo hace el orquestador o `vanta-docs`).
- **El pin EGO-16** (`tests/certified_delete.rs`) sigue en `#[ignore]`: con este fix pasaria **solo** si el
  journal del LSM se purga. No se des-ignora aqui porque el contrato completo **no** se cumple todavia.

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
- Este agente **no ha hecho commit** (lo commitea el orquestador).
- Coexistencia: `src/wal.rs` y `src/wal_sharded.rs` los editan en paralelo otros trabajos de W6/W7 (cifrado).
  Este trabajo solo anade el bloque de borrado fisico (FIND-343) y sus tests; no toca `src/crypto*` ni el
  `WalCrypto`.
