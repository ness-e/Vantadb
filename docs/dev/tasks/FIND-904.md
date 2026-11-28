---
title: "FIND-904: la reescritura del vector store en el path certificado cubre todos los niveles"
kind: task
status: completed
description: "compact_layout_bfs recorre los cuatro segmentos con el accounting del propio segmento (live_records: id + offset local) y compact_segment reescribe cualquiera de ellos: un registro promovido a L1 por pipeline() y luego borrado con delete_certified ya no deja bytes. Medido: 1 segmento con el residuo -> 0, con los octetos del nodo y el payload f32."
id: FIND-904
---

# FIND-904: el path certificado reescribía sólo L0

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-904`, medida por el cierre de `FIND-903`)
- **Origen:** `FIND-903` (2026-11-28) · **Ver:** `FIND-903`, `FIND-905`, `FIND-826`, `FIND-343`
- **Esfuerzo:** 🟡 2-4 h · **Creado:** 2026-11-28 · **Cerrado:** 2026-11-28

## Premisa (re-medida sobre HEAD, no re-derivada)

Instrumento: escaneo crudo de cada `vstore_L*.vanta` por el `node_id` (cabecera `u128` de
cualquier copia superviviente) y por los bytes `f32` del vector **tal como se almacena** — los
dos needles de `attestation::vector_store_surface`, para que el certificado y el test no puedan
estar en desacuerdo sobre qué es "borrado".

Test nuevo, HEAD sin cambios de código, runner por defecto (12 hilos):

```
cargo test -p vantadb --test certified_delete certified_delete_erases_a_record_the_pipeline_promoted_to_l1
→ the promoted record's node id survived a certified delete in ["vstore_L1.vanta"]
  (FIND-904: the certified path rewrites L0 only)
→ test result: FAILED. 0 passed; 1 failed
```

Premisa **confirmada**: el fixture del propio test prueba que `pipeline(CompactOnly)` promovió el
registro a `L1` (`segments_holding(..) == ["vstore_L1.vanta"]`) y que `delete_certified` lo dejó
allí. El payload de texto no aparece en el vstore en ningún caso (lo redacta `compact_wal`); el
residuo es el nodo en el segmento.

## Contrato elegido: **(b)**, reescritura por segmento — y por qué con la medición

Las tres opciones del contrato, contra lo medido:

| Opción | Por qué no / por qué sí |
|---|---|
| **(a)** `compact_level(n)` para cada nivel vivo antes de firmar | **No garantiza 0**: su rama honesta deja el segmento intacto cuando retiene un vivo no promovido, y eso está medido en `compact_level_keeps_live_records_the_graph_does_not_promote` (`reclaimed_bytes == 0`, header del vivo en su sitio). Además **cambia la colocación** de todos los vivos (L0→L1→L2 en cada borrado certificado) para borrar uno. |
| **(c)** extender el predicado de `compact_level` para que reescriba lo que retiene | Correcto en sí, pero **no toca el defecto**: el path certificado no llama `compact_level`; sólo llama `compact_layout_bfs`. Arreglaría un camino que este ticket no ejecuta. |
| **(b)** reescribir los segmentos que contengan el registro | **Elegida.** No depende del grafo ni de la promoción, conserva los vivos del segmento y es la única que cumple el contrato literal ("reescribe todos los niveles que puedan contener el nodo"). |

**El obstáculo real, y lo que lo resuelve:** `compact_layout` leía los offsets por
`hnsw.storage_offset_of`, que devuelve el offset **empaquetado** (los 6 bits bajos son el id de
segmento, `src/lsm.rs:19`). Para `L1+` eso no es un offset de byte del fichero — `File::read_header`
rechaza un offset no alineado a 64 (`src/storage/vfile.rs:936`) —, así que la primitiva **no podía**
reescribir `L1+`. El accounting del segmento ya existía desde `FIND-905`
(`archive::live_record_ids`); se extendió a `live_records`, que devuelve `(id, offset local)` del
mismo recorrido `[STORAGE_ALIGNMENT, write_cursor)`.

## Cambio

- **`src/storage/archive.rs`**
  - `live_records(&File) -> Vec<(u128, u64)>` (id + offset **local**); `live_record_ids` pasa a ser
    la proyección sin offsets (una sola caminata, no dos).
  - `compact_layout(vstore, hnsw, bfs_order, header_size)` **conserva su firma pública** y se vuelve
    el adaptador: resuelve `(id, offset)` por el índice y delega en
    `compact_segment(vstore, &[(id, offset)], header_size)`, que es el cuerpo de antes. La variante
    cifrada (`compact_layout_encrypted`) toma la misma lista, así que un segmento v3 se reescribe v3
    en cualquier nivel (ADR-0057 W3 intacto).
- **`src/storage/engine/maintenance.rs`**
  - `compact_layout_bfs` itera **todos** los segmentos (`rewrite_segment(level)`); `L0` ordena por el
    grafo (localidad del tier caliente) y los tiers fríos por el orden del propio segmento.
  - `authoritative_records(level)` decide **qué filas se copian**: la caminata del segmento filtrada
    por el índice (ver «Ronda P2-01»). Sin ese filtro la reescritura hacía **permanente** una copia
    que ya no era la del registro.
  - El predicado sigue siendo **por segmento** (`live_records`), no por grafo: `FIND-905` intacto.
  - Desaparece la guarda "un vivo sin offset en el índice ⇒ no tocar": con los offsets del
    segmento, todo vivo es copiable; antes esa guarda era el único modo de no perderlo.
  - `save_vector_index()` tras cada segmento reescrito: los offsets cambian, y un fallo a mitad del
    bucle no debe dejar el índice en disco apuntando a un fichero que ya no tiene esos bytes.
  - Segmento sin vivos **y ya vacío** (`write_cursor == STORAGE_ALIGNMENT`): no se reescribe — no hay
    nada detrás de la cabecera, y hacerlo sería un `tmp+rename+fsync` por cada tier vacío en cada
    compactación.
- **`src/attestation.rs`**: sólo el doc de `vector_store_surface` (la historia L0-only y qué la cerró).
- **`docs/api/CERTIFIED_DELETE.md`**: la superficie `vector_store` describe la cobertura por nivel.

## Ronda P2-01 (review adversarial, aplicada)

El review encontró **uno real y bloqueante**: la copia huérfana seguía viva cuando el segmento
origen **no se pudo resetear**. `compact_level` copia el registro al siguiente tier y repunta el
índice, pero **no retira la fila origen** (no la tombstonea); si el segmento retiene además un vivo
que no puede promover, su rama honesta lo deja intacto — y la reescritura, que sólo miraba cabeceras
no tombstoneadas, **arrastraba el residuo**. Peor: la misma forma aparece sin pipeline, con un
`put` sobre una clave existente (append de una copia nueva, la vieja queda sin tombstone) — medido
en HEAD: `status: residues`, `vector_store.residues: 1`, el id todavía en `vstore_L0.vanta`.

**Fix: la autoridad es el offset del índice, no la cabecera.** `authoritative_records` copia una
fila sólo si (a) el índice apunta **exactamente** a ella, o (b) no hay entrada en el índice **y** el
registro sigue existiendo (su fila KV en `Default`). Una fila que el índice apunta en **otro**
segmento o en otro offset del mismo es una copia vieja; una fila sin entrada de índice cuyo registro
ya no existe en KV es basura huérfana. Con eso caen los dos casos con la misma regla: el promovido y
el actualizado.

Tests añadidos en esta ronda:

- `src/storage/engine/tests/maintenance.rs::compact_layout_bfs_drops_leftovers_of_records_that_live_in_another_segment`
  (engine en disco: un vivo sin entrada de índice + un promovido; la fila del promovido **no** se
  copia, el vivo se conserva). RED medido quitando el filtro: `[901, 900]`.
- `tests/certified_delete.rs::certified_delete_erases_the_previous_copy_of_an_updated_record`
  (`put` v1 + v2, borrado certificado → **0 segmentos** con el id). RED medido contra `HEAD` real
  (`git show HEAD:…` de los dos ficheros): `the previous copy … survived … ["vstore_L0.vanta"]`.

Nits del review, decididos: `if self.wal.is_some()` antes de `save_vector_index()` es redundante
(la función ya sale con `wal.is_none()`) — se deja porque el índice de segmentos fríos sólo existe
si hay WAL, y el `if` documenta la intención a coste cero; el mensaje "BFS order is empty" se deja
(describe el contrato del primitivo, que sigue rechazando un conjunto vacío).


## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/maintenance.rs` (`flush`,
  `save_vector_index`, `compact_layout_bfs`, `compact_level`, `should_compact_level`,
  `merge_segments`, `pipeline_*`), `src/storage/archive.rs` (`reset_empty_segment`,
  `live_record_ids`, `compact_layout`, `compact_layout_encrypted`, `traverse_graph`,
  `reindex_nodes`), `src/storage/vfile.rs` (`read_header`, `write_header`, `node_stride`,
  `is_encrypted`, `replace_backing_file`), `src/lsm.rs` (`pack_offset`/`unpack_offset`),
  `src/sdk/api/memory.rs` (`delete_certified`, `compact_purged`, `forget`),
  `src/sdk/api/admin.rs` (`compact_layout`), `src/attestation.rs`
  (`vector_store_surface`), `tests/certified_delete.rs`,
  `src/storage/engine/tests/{mod,maintenance}.rs`.
- **Referencias hacia dentro:** `compact_layout_bfs` ← `sdk::admin::compact_layout` (que es lo que
  llama `compact_purged`), `merge_segments`, `run_pipeline`, `server::handlers`, `sdk::api`;
  `compact_layout` ← `compact_layout_bfs` y tests de `archive`/`vfile_crypto`.
- **Referencias salientes:** `File` (segmento), `storage::ops`/`vfile` (escritura), `hnsw`
  (`set_storage_offset`), `lsm::pack_offset` (convención del offset), `insert_lock`.
- **Veredicto de impacto:** cambia **qué ficheros** se reescriben en una compactación (L0 → L0..L3) y
  el **offset almacenado** de los registros de `L1+` reescritos (se compactan dentro de su segmento,
  no se promueven). No cambia el layout de nodos ni de cabeceras, no cambia qué segmento contiene un
  registro, ni el grafo, ni ninguna firma pública.

## Steps

1. ✅ RED: test `certified_delete_erases_a_record_the_pipeline_promoted_to_l1`.
2. ✅ `archive::live_records` + `compact_segment` (offsets locales; cifrado incluido);
   `compact_layout` como adaptador sin cambio de firma.
3. ✅ `compact_layout_bfs` → `rewrite_segment(level)` para los 4 niveles; `segment_order`.
4. ✅ Ronda P2-01: `authoritative_records` (autoridad = offset del índice + existencia en KV) y los
   dos tests de la regresión.
5. ✅ Docs (attestation doc, `CERTIFIED_DELETE.md`, Backlog, avance) + task file.

## Verificación (medida, en este orden)

```
cargo test -p vantadb --test certified_delete                              # 12 hilos → 17/17 (15 + 2 nuevos)
cargo test -p vanta-memory -j 2                                           # 52 binarios ok, 0 fallos
cargo test -p vanta-memory --test gdpr_cascade_forget                     # 1/1
cargo nextest run -p vantadb --profile audit --build-jobs 2               # 3040 passed, 4 skipped
cargo test -p vantadb --lib -- storage::archive storage::engine::tests::maintenance   # 130/130
cargo clippy -p vantadb --all-targets -- -D warnings                      # limpio
cargo fmt --all -- --check                                                # limpio (medido, no SKIP)
cargo check -p vantadb --features encryption --all-targets                # limpio
cargo test -p vantadb --features encryption --lib -- storage::vfile_crypto storage::archive   # 45/45
```

Filtro del contrato, población medida antes de correrlo (`cargo nextest list` → 65 tests) y corrido
con **el mismo filtro** + `--no-tests fail`: `test(compact) or test(erasure) or test(certified) or
test(vstore)` → **65/65**.

`#[serial]` / `#[ignore]`: ninguno.

## Deuda declarada (`ponytail:`)

- Una compactación certificada reescribe **los cuatro** segmentos (coste proporcional a la base, no
  al conjunto borrado — el techo que `Embedded::forget` ya declara para `FIND-826`). El
  `save_vector_index` por segmento reescrito añade una persistencia del índice por nivel: se elige
  durabilidad sobre coste porque un índice stale apunta a bytes que ya no están.
- `authoritative_records` hace **una lectura puntual al backend** por cada fila sin entrada de
  índice (normalmente cero o una: es el caso "copia previa de un registro actualizado"). Si algún día
  ese conjunto crece, el barrido pasa a lote.
- `compact_level` sigue sin reescribir el segmento que retiene un vivo no promovido (opción (c),
  fuera de este contrato): es honesto —el vivo sigue vivo y localizable— y ya no puede producir un
  `purged` falso, porque el path certificado no depende de él.
