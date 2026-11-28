---
title: "FIND-812: el rebuild borraba el id sin prueba de ausencia"
kind: task
status: complete
description: "El rebuild del indice borraba un id cuando la deserializacion fallaba, indistinguible de "no existe": la fila seguia en el backend y el registro desaparecia sin poder purgarse. Ahora enumera el conjunto real de filas del backend, conserva el id ante corrupcion, cuenta y reporta cada caso, y tombstonea las borradas por transaccion."
---

# FIND-812: el rebuild no borra un id sin prueba positiva de ausencia

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-812`)
- **Origen:** cierre de `FIND-799` (2026-11-27) — riesgo residual abierto al verificar la refutación
- **Esfuerzo:** 🟡 2-4 h · **Prioridad:** 🟠 Alta
- **Creado / cerrado:** 2026-10-27 / 2026-11-27
- **Estado:** ✅ COMPLETE

## Premisa (medida)

`src/storage/engine/get.rs:329-348` (antes del fix):

```rust
let Some(metadata) = self.fetch_backend_metadata(id)? else { return Ok(None); };
let Some(storage_offset) = self.lookup_index_offset(id) else { return Ok(None); };
let Some(header) = self.read_header_at(id, storage_offset)? else { return Ok(None); };
if (header.flags & FLAG_TOMBSTONE) != 0 { return Ok(None); }
let Some(vector) = self.decode_vector_at(storage_offset, &header)? else { return Ok(None); };
```

Cuatro causas distintas, un solo `Ok(None)`. Y `scan_nodes()` (`ops.rs:291-402`)
descarta esas filas — además de toda fila cuyo vector no sea `Full`
(`SQ8`/`Binary`/`Turbo` también se caían). El rebuild de los índices derivados
y del índice de texto **borra la partición entera primero** y reconstruye desde
`scan_nodes()`: una fila viva cuyo header se torció salía del índice y, como la
fila del backend sigue existiendo, ni `vacuum` (que purga por `Ok(None)` del
backend) ni un purge posterior podían volver a verla.

`vacuum` sí tenía la prueba positiva: solo empuja a `tombstone_ids` cuando
`backend.get()` dice `Ok(None)`. La mitad que faltaba era la lectura y el
rebuild.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/get.rs`,
  `src/sdk/serialization/impl_rebuild.rs`, `src/sdk/serialization/impl_text_index.rs`,
  `src/storage/engine/maintenance.rs` (`vacuum`), `src/storage/engine/ops.rs`
  (`scan_nodes_page`), `src/storage/archive.rs` (`rebuild_hnsw_from_vstore*`),
  `src/sdk/serialization/impl_index.rs` (`ensure_indexes_current`),
  `src/sdk/search/lexical.rs`.
- **Referencias hacia dentro:** `materialize_uncached` (2 call sites: `get()`,
  tests de `storage/engine`), `fetch_backend_metadata` (2 tests → migrados a
  `fetch_backend_row`, el método quedó muerto y se borró), `scan_nodes()`
  (muchos call sites: **no se tocó** — cambia solo el camino del rebuild).
- **Referencias entrantes:** `Embedded::rebuild_text_index_with_report`,
  `rebuild_derived_indexes_with_report`, `build_text_index_audit_report_*`,
  `ensure_indexes_current`, `purge_expired`.
- **Veredicto impacto:** **medio**. Cambia la forma del retorno de un método
  `pub(crate)` (no público) y el conjunto de filas que el rebuild indexa; el
  comportamiento de `get()` se mantiene **idéntico** (ver §Decisiones).

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Cómo se distingue "no hay fila" de "no se pudo leer"? | (a) `Result<Option<T>>` con `Err` para la corrupción — cambia el contrato de `get()` y mete errores en el hot path de lectura; (b) **enum `Materialized { Node, Degraded, Gap(RowGap) }`** — el motivo viaja, `get()` sigue devolviendo `None` | ✅ (b) |
| 2 | ¿De dónde enumera el rebuild? | (a) `scan_nodes()` — es el bug; (b) **el row set del backend** (`BackendPartition::Default`), que es la única prueba de existencia | ✅ (b) |
| 3 | ¿Qué pasa con una fila cuyo header/vector no decodifica? | (a) descartarla (el bug); (b) **reconstruirla desde su metadata** (los campos de la memoria viven ahí, el vector se pierde) y contarla | ✅ (b) |
| 4 | ¿Qué pasa con una fila cuya metadata no decodifica? | (a) borrar sus entradas en silencio; (b) **contarla y reportarla con su id**, sin poder reconstruir nada (el payload está en esa misma fila) | ✅ (b), declarado |
| 5 | ¿`get()` devuelve el nodo degradado? | (a) sí — un nodo sin vector en el hot path es una mentira silenciosa (flags=0, dims=0); (b) **no**: `get()` sigue respondiendo "no legible", y el id se conserva en el índice para la reparación | ✅ (b) |

## Steps

1. ✅ Test RED `d1a_materialize_uncached_miss_absent` (ahora exige `RowGap::Absent`) + `f812_rebuild_keeps_a_row_with_a_corrupt_header` + `f812_rebuild_purges_a_row_the_backend_lost`.
2. ✅ `Materialized`/`RowGap`/`RebuildRows` + `node_from_metadata` + `scan_rows_for_rebuild` en `get.rs`; `fetch_backend_row` separa I/O de deserialización, y `materialize_row` evita re-leer la fila que el scan ya trae.
3. ✅ `rebuild_text_index_with_report` y `rebuild_derived_indexes_with_report` enumeran el row set; `report_rebuild_gaps` cuenta y loguea cada id con su motivo; `corrupt_rows` en los **dos** reports.
4. ✅ `expected_text_index_entries` (auditoría) y `ensure_indexes_current` usan el mismo row set — si no, el conteo esperado y el reconstruido discreparían y el rebuild se dispararía en cada open.
5. ✅ **Ronda P2-01, `changes-required`, aplicada** (ver §Review): `rebuild_sparse_index_with_report` (C1) también enumera el row set; `deleted_by_txn` ⇒ `Tombstoned` (H1); `get()` sigue devolviendo `Err` con la metadata ilegible (H3); tests por variante (M4).

## Contrato de verificación

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(text_index) or test(posting) or test(rebuild) or test(get)'   # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail
```

**Cuatro caras, cada una con test:**
- header/vstore corrupto (offset a un slot inexistente) ⇒ el id **sigue** en el
  índice (`corrupt_rows == 1`, `record_count == 2`, las 4 postings
  reconstruidas), la fila sana se sigue sirviendo por `search`, y hay evento
  observable (`tracing::warn!(node_id, reason)` + el contador del report).
- fila del backend ausente ⇒ el id **se purga** (`corrupt_rows == 0`).
- fila **borrada por transacción** (`deleted_by_txn` + sin offset) ⇒ se purga,
  no se re-indexa.
- fila que el índice vectorial nunca vio ⇒ se conserva (degradada) y se cuenta;
  **no** se sirve por `search` (decisión de abajo).

**Invariante:** `vacuum` ya purgaba solo con `Ok(None)` del backend; ese contrato
no cambia. `get()` conserva su comportamiento observable: `Ok(None)` para
ausente/tombstone/degradada y **`Err`** para metadata que no decodifica.

## Decisiones declaradas

- **`get()` NO sirve la fila degradada** (sigue devolviendo `Ok(None)`): un nodo
  sin vector en el hot path sería un fantasma indistinguible de uno real, y el
  contrato de la fila es sobre el **rebuild** (que no borre), no sobre la
  lectura. Consecuencia declarada: el id queda indexado pero **invisible** hasta
  que una reparación lo materialice; el test lo dice explícitamente en vez de
  prometer un hit que el código no da.
- **`get()` sigue fallando ruidosamente** (`Err`) cuando la **metadata** no
  decodifica: eso es lo que hacía `HEAD` (`deserialize_node_payload(...)? `) y
  tragarlo a `Ok(None)` habría convertido la corrupción en "no encontrado" — la
  misma clase de fallo que esta fila ataca.
- **Una fila con la metadata ilegible no puede reconstruirse**: su payload vive
  en esa misma fila. Se cuenta y se reporta con su id (`corrupt_rows` + warn),
  pero sus entradas de índice se recalculan desde cero (no hay de dónde). El id
  sigue visible en el backend para un purge/reparación posterior.
- **`corrupt_rows` cuenta las dos cosas** (degradadas + ilegibles): el log las
  distingue con `reason`, y una degradada **sí** se indexa mientras que una
  ilegible no. El report no pretende distinguirlas; el operador tiene el `warn`.

## Review

### P2-01 ronda 1 (contexto fresco, `vanta-review`, agente distinto) — **changes-required**, aplicado

**1 CRITICAL / 3 HIGH / 5 MEDIUM / 5 LOW.** Premisa re-ejecutada contra `HEAD` por el reviewer: confirmada en las dos filas.

Aplicado en la misma ronda:

- **C1 (CRITICAL)** — `rebuild_sparse_index_with_report` seguía enumerando `scan_nodes()` mientras `ensure_indexes_current` le pasaba el row set: con ≥1 fila que `scan_nodes` descarta (SQ8/Binary/Turbo, header torcido), el conteo esperado nunca coincidía y el rebuild de sparse se disparaba **en cada open escribible**. Ahora los **tres** rebuilds (texto, derivado, sparse) consumen `scan_rows_for_rebuild()`.
- **H1** — una fila borrada por transacción (metadata con `deleted_by_txn`, sin offset en HNSW) caía en `NotIndexed` y se **re-indexaba**: resucitaba en texto/derivado/sparse. `materialize_row` consulta `deleted_by_txn` **antes** de `lookup_index_offset` ⇒ `Tombstoned`. Test `f812_rebuild_purges_a_row_deleted_by_a_transaction`.
- **H3** — `get()` sí cambiaba: la metadata ilegible devolvía `Err` en `HEAD` y yo la tragaba a `Ok(None)`. Ahora `Materialized::Unreadable { error }` la propaga: `get()` vuelve a fallar ruidosamente.
- **H2** — el contrato decía "`search` lo encuentra" y `get_many` nunca resuelve una fila degradada: reescrito a lo que el código hace (§Contrato, §Decisiones) en vez de prometer un hit.
- **M1** — comentarios de `RebuildRows`/`scan_rows_for_rebuild` que afirmaban "se preservan las entradas": no se preserva nada, se cuentan y se reportan.
- **M4** — tests por variante: `Tombstoned` (H1), `NotIndexed` (`f812_rebuild_keeps_a_row_the_index_never_saw`), `Absent`, `Node`.
- **M5** — `DerivedIndexRebuildReport::corrupt_rows` (antes el derivado descartaba el conteo); el `warn` distingue degradada de ilegible.
- **L1** — `KEY_FORMAT` describía un `\0` entre tags que el layout no tiene.
- **L2** — el error de `ensure_text_index_query_ready` citaba "schema v3"; ahora deriva la versión de la spec.
- **L3** — `bm25_terms_for_record` (explain) respondía vacío en silencio sobre un índice v3: ahora llama a `ensure_text_index_query_ready`.
- **L4** — `scan_rows_for_rebuild` tiraba el `value` del scan y re-leía cada fila: `materialize_row(id, raw)` reutiliza el valor (una lectura por fila en vez de dos).
- **L5** — claves que no son 16 bytes se saltean con `warn!` en vez de en silencio.

No aplicado, declarado: **M2/M3** — `SECURITY.md` ahora acota qué se fue **del índice de texto** y declara que el índice derivado y los sparse postings **siguen** llevando la clave en claro, y que el tag sin clave es **recuperable por diccionario** (no "confirmable por conjetura"). Las dos filas de seguimiento (tag keyed con `k_blind`; claves del derivado/sparse en claro) necesitan ID del orquestador — declaradas en `FIND-513.md` §Deuda.

## Context Save Point

- **Hecho:** `src/storage/engine/get.rs`, `src/sdk/serialization/impl_rebuild.rs`,
  `src/sdk/serialization/impl_index.rs`, `src/sdk/serialization/impl_sparse_index.rs`,
  `src/sdk/types/search.rs`, `src/storage/engine/tests/engine.rs`, este task file,
  `Backlog.md`, `avance/activo/core-engine.md`.
- **No hacer:** volver a enumerar el rebuild con `scan_nodes()` — descarta filas
  por diseño (no es un error: es una vista "solo completa"). La enumeración del
  rebuild es `scan_rows_for_rebuild`, y la usan los **tres** rebuilds.
