---
title: "LOTE seguridad y cifrado en reposo - 5 tareas, especificacion completa"
kind: task
status: active
description: "Especificacion ejecutable de FIND-586, 513, 343, 496 y 507: contrato, verificacion, comandos, dependencias y skills por tarea. Orden obligatorio por dependencia de formato."
id: LOTE-seguridad
---

# LOTE seguridad/cifrado — 5 tareas

**Owner:** 4 días hasta el 01-dic. Nada diferido. **Un `FIND` nuevo se resuelve en el acto.**

## Orden obligatorio

```
586 (AAD con identidad de fichero)  →  513 (clave del posting)
   →  343 (purga física del WAL)    →  496 (shipper v4)  →  507 (backup/restore)
```

**Por qué:** `586` cambia el AAD de los 4 artefactos (`vfile`, `wal`, `wal_sharded`, `init`). Todo lo que cifra después depende de ese AAD. Si se invierte el orden, `513` y `343` se reescriben dos veces.

---

## FIND-586 — El AAD no ata el ciphertext a su fichero

**Severidad:** 🟠 Alta · **Esfuerzo:** 1-2 semanas · **Estado:** Pendiente

**Qué rompe.** `ADR-0057 D4` dice literalmente *"un ciphertext movido de sitio falla la autenticación"*. Con `slot` **intra-fichero**, mover un frame entre ficheros hermanos al mismo offset produce **AAD idéntico** (mismo artifact, versión, epoch, slot, longitud) y el nonce viaja con el frame ⇒ el tag **sigue autenticando**.

Consecuencia concreta: en el `VantaFile` L0..L3 comparten DEK, así que un nodo de `vstore_L0.vanta@X` se puede sustituir por el de `vstore_L1.vanta@X` (vectores del mismo modelo = misma longitud) ⇒ **el HNSW, que guarda (segmento, offset), resuelve el vector de otro nodo** ⇒ resultados de búsqueda incorrectos **sin error**.

**Dónde.**
- `src/storage/vfile_crypto.rs:170-177` y `:214-221` — sin `identifier` de segmento
- `src/wal.rs:480-484` y `:1346-1350` — el slot es el ordinal intra-fichero
- `src/wal.rs:867`
- `src/storage/engine/init.rs:60-80` — el mismo contexto para L0..L3
- `src/wal_sharded.rs:399-413` — el mismo `WalCrypto` para todos los shards

**Contrato.** Atar la **identidad del fichero** (`identifier` = nombre del segmento / del shard) al AAD en los **cuatro** artefactos. Los formatos v3/v4 son nuevos y sin publicar ⇒ **subir de versión no tiene coste de migración**.

**Efecto lateral valioso:** es la misma raíz que `FIND-498` (el slot es intra-fichero porque el ordinal solo se conoce bajo el lock del escritor). Atar el nombre del fichero permite **sacar el AEAD del lock**.

**Verificación.** Un blob copiado de `L0@X` a `L1@X` **falla la autenticación**. Y un frame movido entre shards del WAL. Test en `src/storage/vfile_crypto.rs` (mod tests) o `tests/`:
```
cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(vfile) or test(aad) or test(wal_sharded) or test(crypto)'   # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail
```

**Depende de:** `FIND-500`, `FIND-498`. **Relación:** extiende `FIND-500` al WAL.
**Skills:** `systematic-debugging`, `test-driven-development`, `doubt-driven-development` (es seguridad: dos opiniones), `code-simplification`.

---

## FIND-513 — El índice de texto deja token y clave EN CLARO

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Qué rompe.** La clave de un posting es `namespace ‖ 0x00 ‖ token ‖ 0x00 ‖ record_key`: el **token** (una palabra del payload) y la **clave del registro** quedan legibles en disco aunque el posting (tf, posiciones, node id) esté cifrado. El índice de namespaces expone namespaces y nombres de clave; **el orden expone cardinalidad y orden de inserción**. El tamaño de cada valor se filtra (longitud = plaintext + 28).

**Dónde.** `src/text_index.rs` (clave del posting) · `src/backends/kv_crypto.rs`.

**Contrato.** Dos mitades, y **la primera es obligatoria aunque la segunda se difiera**:
1. **Declarar el límite** en `docs/` §límites (el `ADR-0057 §5.3` dice "las claves del KV no se cifran", pero **la magnitud de esta fuga merece su propia línea**).
2. **Cifrar la clave del posting** — implica que el índice deja de poder usar **prefijos por namespace** ⇒ hay que **rediseñar la clave**. Si el rediseño rompe el formato en disco de un índice ya escrito, **decilo y NO lo hagas**: el owner decide. Entonces entrega (1) + el diseño de (2).

**Verificación.** El doc de límites **nombra** la fuga del índice de texto, y el finding de diseño de la clave cifrada está **decidido** (implementado o documentado como decisión pendiente del owner con fecha).
**Comandos:** `cargo nextest list ... -E 'test(text_index) or test(posting) or test(kv_crypto)'` → >0, y el run igual.
**Depende de:** `FIND-407`.

---

## FIND-343 — Borrado físico incompleto (WAL rotado + journal Fjall)

**Severidad:** 🟠 Alta · **Esfuerzo:** 3-5 días (rediseño de Engine) · **Estado:** Pendiente

**Qué rompe.** Auditoría EGO-16: tras `delete` + `vacuum()` + `pipeline(MergeOnly)` + `compact_wal` + `flush` + `close`, **el payload sigue legible** en `data/vanta.shard0.wal` y en el journal `0.jnl`. El certificado (`delete_certified`) **solo garantiza nivel API**.

**La mitad del diagnóstico ya está hecha** (leela antes de re-medir): el *"WAL rotado"* de la fila **no existía** — `ShardedWal::rotate_all()` no rota: reabre el mismo fichero. Y **no hay retención**: el auto-rotate renombra a `vanta.shardN.wal.<ts>` y **nadie lo borra**.
El fix medido que quedó propuesto: **en vez de borrar el registro, sustituirlo por `Delete{id}` sin payload** ⇒ el conteo no cambia (`checkpoint_seq` sigue significando lo mismo), el estado reproducido es idéntico y la cadena VER-01 se recalcula. **El formato en disco no cambia.**

**Dónde.** `src/storage/engine/maintenance.rs:129` (`compact_wal` rota, no borra) · journal Fjall `0.jnl` · `tests/certified_delete.rs` (`physical_erasure_*` está `#[ignore]` como pin).

**Contrato.**
- Implementar la sustitución por `Delete{id}` **o** una alternativa mejor justificada.
- **No romper recovery**: el WAL es commit point + cadena VER-01; borrar historia rompe recovery. Ése es el límite y hay que respetarlo.
- El set de borrado es **global**, en el orden `(índice por shard, shard)` — el mismo del recovery — **cruzando los segmentos rotados**.

**Verificación.** El canario `FIND343-PROBE-` con el flujo completo y un **escaneo de bytes de todo el directorio de datos** ⇒ **0 ocurrencias**. El test `physical_erasure_*` sale de `#[ignore]` o queda un equivalente que corra en CI.
```
cargo nextest list --profile audit -p vantadb -E 'test(certified_delete) or test(physical_erasure) or test(compact_wal)'   # > 0
```
**Requiere** `vanta-arch` (diseño de la purga física) + `vanta-review` (Regla 8).
**Depende de:** EGO-14, EGO-16, VER-01, VER-02.

---

## FIND-496 — El shipper no puede leer segmentos v4 (cifrados)

**Severidad:** 🟠 Alta · **Esfuerzo:** 1 hora · **Estado:** Pendiente

**Qué rompe.** Con W2 el WAL cifrado produce segmentos **v4**, y el shipper los abre con `WalReader::open` **sin clave** ⇒ `EncryptionKeyRequired` sobre un segmento cifrado. **El WAL shipping es el mecanismo de replicación**: un WAL cifrado no se puede replicar tal cual.

**Dónde.** `src/wal_shipping.rs`.

**Contrato — elegí una y justificá:**
- (1) pasar la clave al reader del shipper y **re-cifrar en destino**; o
- (2) **declarar que el WAL shipping no soporta bases cifradas** y **fallar al inicio** del envío con un mensaje claro, en vez de fallar a mitad de la replicación.

⚠️ **Lo que NO se hace:** enviar el material de clave por la red. Está explícitamente descartado en la fila.

**Verificación.** Con una base cifrada: **o** falla al inicio con mensaje explícito, **o** replica y el destino puede descifrar. Nunca falla a mitad.
**Depende de:** `FIND-407`.

---

## FIND-507 — Backup/restore no conoce el registro de cifrado ni la clave

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Qué rompe.** `encryption.registry` **no aparece en ningún sitio** del camino de backup/restore (en todo el repo solo vive en `src/crypto/registry.rs` y en el plan/ADR). El backup copia el `data_dir` (y con él el registro) pero **la master key no viaja** (vive en el entorno). Y `is_mutable_after_snapshot` **no incluye el registro** ⇒ un snapshot lo **hard-linkea** en vez de copiarlo: snapshot y base viva **comparten el mismo fichero de claves**.

**Dónde.** `src/cli_handlers/backup.rs` (y el path de restore) · `src/storage/engine/mod.rs` (`is_mutable_after_snapshot`).

**Contrato — cuatro cosas, las cuatro:**
1. `encryption.registry` en la lista de ficheros que el snapshot **copia** (no linkea).
2. El backup **incluye** el registro y su **manifest lo declara**.
3. El restore **dice explícitamente** que hace falta la master key y **cómo** proporcionarla.
4. **Runbook**: qué pasa si se pierde el registro (la base se pierde: el `db_salt` se genera al crearlo).

**Verificación.** (a) snapshot ⇒ la base viva y el snapshot tienen registros **distintos** (no el mismo inode); (b) restore en otra máquina ⇒ **error explicativo** sobre la clave, no `EncryptionKeyRequired` a secas.
**Depende de:** `FIND-501`.

---

## Proceso del lote (obligatorio)

1. **Leer**: la fila del Backlog + su task file + **el ADR que la gobierna** (`ADR-0057`) antes de tocar.
2. **RED primero**: un test que falle en `HEAD`. Si no se puede, **decilo y no cierres la fila**.
3. **Fix mínimo** que siga el idioma del fichero (los dos carriles Rust previos lo hicieron: `purge_lock` en `FIND-773`, index-first en `FIND-798`).
4. **Una fila = un commit** con el helper, salida capturada, `-ExpectAbsent` declarado.
5. **Review P2-01** en contexto fresco. Si pide cambios, se aplican en el mismo lote.
6. **`FIND` nuevo**: se resuelve en el acto si está en tu zona; si no, se registra **y se resuelve igual** antes de cerrar el lote.

## Gates de validación del lote

| Gate | Criterio |
|---|---|
| Por fila | test RED → GREEN, `list` > 0 = `run` con el **mismo** filtro y `--no-tests fail` |
| `fmt` | `cargo fmt --check` sobre tus ficheros |
| `clippy` | `cargo clippy -p vantadb --all-targets -- -D warnings` |
| `rustdoc` | si tocas un `pub`, `cargo doc -p vantadb --no-deps` con `-D warnings` (**`WIRE-20` rompió esto y tumbó CI**) |
| `check-docs` / `check-links` | exit 0 si tocas `.md` |
| Commit | helper con salida capturada; códigos `7`/`8`/`9` |

## Riesgos declarados

- **Cambio de formato del ciphertext** (`586`): no hay datos publicados, así que no hay migración — pero **verificá que no hay fixtures viejos** en `tests/`.
- **Contención de disco y lock**: dos carriles Rust vivos. `--build-jobs 2`, esperar el lock (no matar nada), `target-cleanup -AutoSafe` si el link falla.
- **`343` es 3-5 días de rediseño**: si no cabe, entrega el **diseño medido** + la mitad implementada, y dilo.
