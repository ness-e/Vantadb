---
title: "FIND-343: borrado fisico incompleto (WAL + journal Fjall) - ya estaba implementado, medido"
kind: task
status: in-progress
description: "La fila daba la mitad LSM por imposible. Medido en HEAD: el canario del flujo completo (put/delete/vacuum/merge/compact_wal/flush/close) NO aparece en ningun byte del directorio de datos y el test corre sin #[ignore]. El mecanismo existe: sustitucion por Delete{id} sin payload en el WAL y reescritura atomica de keyspaces del backend. Cierra por medicion, sin codigo."
---

# FIND-343: borrado físico — cerrada por medición

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-343`)
- **Origen:** auditoría EGO-16 (2026-10-16) · **Dep:** EGO-14, EGO-16, VER-01, VER-02
- **Esfuerzo:** 3-5 días (estimado en la fila) · **Prioridad:** 🟠 Alta
- **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED (por medición: el trabajo ya estaba en el árbol)

## Premisa (re-medida, no re-derivada)

La fila afirmaba que tras `delete` + `vacuum()` + `merge` + `compact_wal` + `flush` + `close`
el payload seguía legible en `data/vanta.shard0.wal` **y** en el journal Fjall `0.jnl`, y que
la mitad LSM exigía reescribir la base entera.

Medición sobre `HEAD` (sin tocar código):

```
cargo nextest run --profile audit -p vantadb --test certified_delete --build-jobs 2 \
  --features encryption,wal-shipping -E 'test(physical_erasure)' --no-tests fail
→ Summary [1.979s] 1 test run: 1 passed
```

El test `certified_delete::physical_erasure_no_payload_bytes_after_full_maintenance`
(`tests/certified_delete.rs:252`) **no está `#[ignore]`** y hace exactamente lo que la fila
pide: escribe un canario (`EGO16-PROBE-payload-debe-desaparecer-…`), borra, corre el
mantenimiento completo y **escanea en crudo todo el árbol de datos** buscando los bytes del
payload; exige **0 hits**. Pasa.

Las dos mitades están implementadas en el árbol:

| Mitad | Mecanismo | Dónde |
|---|---|---|
| WAL | los registros del nodo borrado se **sustituyen** por `Delete{id}` sin payload: el conteo no cambia (`checkpoint_seq` sigue significando lo mismo), el estado reproducido es idéntico y la cadena VER-01 se recalcula; el set de borrado es global, orden `(índice por shard, shard)`, cruzando los segmentos rotados; una txn abortada no borra | `src/wal.rs` (`write_redacted_segment`, `WalWriter::redact_erased`), `src/wal_sharded.rs` (`redact_erased`, llamado desde `rotate_all`) |
| Backend (journal Fjall) | `FjallBackend` implementa `Compactable`: reescritura atómica de keyspaces que produce un journal fresco; `compact_wal()` lo invoca tras rotar | `src/storage/engine/maintenance.rs:157-197`, `src/storage/backends/**` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `tests/certified_delete.rs`, `src/wal.rs` (§redacción),
  `src/wal_sharded.rs` (§`rotate_all`, `redact_erased`), `src/storage/engine/maintenance.rs` (§`compact_wal`).
- **Referencias hacia dentro/entrante:** ninguna necesita cambio.
- **Veredicto impacto:** **nulo** — no se modifica código. Esta tarea es una medición + el
  registro del cierre; la fila del Backlog se retira con esa evidencia.

## Steps

1. ✅ Leer la fila y el task file previo (no había task file: la fila traía el análisis).
2. ✅ Localizar el mecanismo en el árbol (WAL: sustitución por tombstone; backend: `Compactable`).
3. ✅ Ejecutar el test del canario y el escaneo de bytes → **1 passed**.
4. ✅ Retirar la fila y registrar el cierre en `docs/dev/avance/activo/core-engine.md`.

## Contrato de verificación

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(certified_delete) or test(physical_erasure) or test(compact_wal)'   # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail
```

**Invariante que no se puede romper:** el test **no** puede volver a `#[ignore]`; y la
redacción no puede cambiar el número de registros (rompería `checkpoint_seq`) ni el estado
reproducido por el recovery.

## Deuda declarada (fuera del alcance de esta fila, y por qué)

- **Crypto-shredding**: con la base cifrada, el ciphertext de un registro borrado sigue
  siendo descifrable con la DEK vigente porque el AEAD no conoce el borrado. Rotar la DEK por
  artefacto es del registro de claves (`src/crypto/**`, rotación), no de la purga física — y la
  fila ya lo listaba así.
- **Journal Fjall preasignado (64 MiB)**: la compactación produce un journal nuevo; el
  *fichero* viejo lo recicla Fjall, no el motor. El canario mide el contenido lógico, que es lo
  que la fila exige (0 bytes del payload en el árbol).
