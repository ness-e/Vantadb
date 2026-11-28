---
title: "FIND-903: el borrado certificado no alcanza 0 residuos cuando el registro borrado es el unico de la base"
kind: task
status: partial
description: "Reescritura de segmento vacio (grafo vacio) y reclamacion sin rango muerto (compact_level) con una sola primitiva; la superficie vector_store pasa de not-assessed a scanned con el conteo medido. Medido: 1 residuo en vstore_L0.vanta -> 0. PARCIAL: el predicado que disparaba la reescritura era por grafo y borraba registros vivos sin vector; corregido por FIND-905 a predicado por segmento."
id: FIND-903
---

# FIND-903: el borrado certificado dejaba 1 residuo con registro único

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-903`, medida por el carril de `FIND-826` D2)
- **Origen:** `FIND-826` D2 (medición 2026-11-28) · **Ver:** `FIND-826`, `FIND-343`, `FIND-675`
- **Esfuerzo:** 🟠 1 d · **Creado:** 2026-11-28
- **Estado:** 🟡 **PARCIAL** — el cambio (una sola primitiva de reescritura) se conserva y está
  medido; **el predicado que lo disparaba estaba mal** y se corrigió en `FIND-905` (abajo).

## Corrección `FIND-905` (2026-11-28): el predicado es por segmento, no por grafo

`reset_empty_segment` era la primitiva correcta; **la condición que lo llamaba, no**. El cierre se
midió con **un** registro, y con una sola fila no hay hermanos que perder:

| Sitio | Predicado de este cierre (malo) | Predicado actual |
|---|---|---|
| `compact_layout_bfs` | `hnsw.entry_point() == None` | `archive::live_record_ids(&vstore).is_empty()` |
| `compact_level` | `nodes_promoted == live_ids.len()` | lo anterior **y** todos los vivos del segmento origen promovidos |

"Grafo sin entry point" ≠ "segmento sin registros vivos": un registro escrito con `vector: None`
vive en el segmento, **sí** está en `hnsw.nodes` (`validate_node`), y **nunca** es entry point. En
`vanta-memory` eso es la norma (`create_l1_episode` escribe sin embedding), así que el borrado
certificado de un episodio reescribía el segmento entero y se llevaba por delante episodios que no
le tocaban (`ep_general_002`, medido). Regresión reproducida en los 3 OSes + Coverage por el único
test `vanta-memory/tests/gdpr_cascade_forget.rs:261`.

Además, `compact_layout_bfs` en la rama **no vacía** sólo copiaba `traverse_graph`; ahora copia el
orden del grafo **restringido a los registros que viven en ese segmento** y **añade** los vivos que
el grafo no puede ordenar, y se niega a reescribir si un vivo no tiene offset en el índice.
`compact_level`, cuando el segmento retiene un vivo que no se promovió, **no reclama nada** (antes
rebobinaba `write_cursor`, que entrega los bytes de un vivo al siguiente append).

## Premisa (re-medida en HEAD, no re-derivada)

Instrumento: escaneo crudo de los bytes `f32` del vector **tal como se almacena** (y del
marcador del payload) en cada `vstore_L*.vanta`. Con testigo vivo da `0`; con un registro
único da `1`.

Medición propia sobre `HEAD` con el test nuevo (sin cambios de código), 12 hilos:

```
cargo test -p vantadb --test certified_delete -- \
  certified_delete_of_the_only_record_leaves_no_vector_bytes
→ the sole record's vector bytes survived a certified delete in
  ["…\\data\\vstore_L0.vanta"] (FIND-903)
→ test result: FAILED. 0 passed; 1 failed
```

El marcador del payload **no** apareció: los bytes del payload los redacta `compact_wal`
(`FIND-343`); el residuo es específicamente el **vector** en el segmento L0. Premisa confirmada.

## Causa (localizada, verificada — no re-derivada)

1. `compact_layout_bfs` (`src/storage/engine/maintenance.rs`) salía temprano con
   `hnsw.entry_point() == None`: `"empty index, skipping"; return Ok(0)`. Con el único
   registro borrado, el grafo queda vacío ⇒ el vstore no se reescribe.
2. `compact_level` reclamaba el segmento origen con `src.write_cursor = STORAGE_ALIGNMENT`,
   dejando `[STORAGE_ALIGNMENT, write_cursor)` legible en disco.

## Contrato elegido: **ambas**, con una sola primitiva

`compact_layout` no puede usarse para el caso vacío: rechaza un `bfs_order` vacío a propósito
(`"refusing to compact (would destroy the database)"`) porque no distingue "grafo vacío" de
"traversal roto". Por eso la reescritura vacía es explícita:

**`archive::reset_empty_segment(&mut File) -> Result<u64>`** — reescribe el segmento a un layout
vacío válido (cabecera + cursor en 4096 B) con el mismo `tmp + rename + fsync` de
`compact_layout`, conserva el **formato propio** del segmento (un v3 cifrado se reescribe v3,
nunca se degrada a plaintext) y, en memoria (WASM/`InMemory`), borra el rango muerto in situ.
Devuelve los bytes reclamados.

**Las dos hacen falta, y se midió por qué:**

| Llamador | Antes | Ahora |
|---|---|---|
| `compact_layout_bfs` (grafo vacío) | `Ok(0)` y el residuo se queda | `reset_empty_segment` (esto es lo que convierte el 1 medido en 0) |
| `compact_level` (reclamar segmento origen promovido) | `write_cursor = ALIGNMENT`, bytes detrás legibles | `reset_empty_segment` cuando **todos los vivos del segmento** se promovieron; si el segmento aún retiene un vivo, **no se reclama nada** (ver corrección `FIND-905`) |

`compact_layout_bfs` sigue devolviendo `Ok(0)` — el número honesto de nodos compactados — pero
el `tracing::info!` nombra la reescritura (`reclaimed_bytes`, `"segment rewritten empty"`) para
que el llamador no lo lea como "no hice nada".

## Superficie `vector_store`: `not-assessed` → `scanned`

`attestation::vector_store_surface` ahora **mide**: recorre los segmentos mapeados
(`vstore_L*.vanta`) buscando el `node_id` (cabecera `u128` de cualquier copia superviviente) y el
payload `f32` almacenado; `residues` = segmentos que aún lo tienen. `action: scanned`.

Se mantiene `not-assessed` cuando hay **segmentos cifrados**: un escaneo plaintext daría `0` por
la razón equivocada (P2-01 M4) — el hueco que `D2` documentó sigue siendo real para ese caso, y
la rama no se borra.

## `cert_claims_zero` (trampa armada de D2)

`cert_claims_zero` leía sólo `residues == 0`, así que un `not-assessed` (que también lleva `0`)
habría contado como "afirmó limpio" — latente hoy porque `vector_store` no está en la lista de
superficies re-verificables, pero vivo el día que alguien la añada. Ahora es
`residues == 0 && action != "not-assessed"`, con test unitario
(`cert_claims_zero_does_not_read_not_assessed_as_a_clean_claim`).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/maintenance.rs` (§`flush`,
  `save_vector_index`, `compact_layout_bfs`, `compact_level`, `should_compact_level`),
  `src/storage/archive.rs` (§`compact_layout`, `compact_layout_encrypted`, `traverse_graph`,
  `reindex_nodes`), `src/storage/vfile.rs` (§`File`, `open_with_mode`/`open_with_mode_and_crypto`,
  `save_cursor`, `flush`, `grow_to`, `ensure_capacity`, `replace_backing_file`, `read_header`,
  `is_encrypted`), `src/attestation.rs` (§`build_certificate`, `swept_surfaces`,
  `vector_store_surface`, `cert_claims_zero`, `verify_certificate`, `finalize`),
  `src/sdk/api/memory.rs` (§`delete_certified`, `compact_purged`, `purge_certificate`),
  `src/sdk/api/admin.rs` (§`compact_layout`), `tests/certified_delete.rs`.
- **Referencias hacia dentro:** `compact_layout_bfs` ← `sdk::admin::compact_layout`,
  `pipeline`, tests de `maintenance`/`antilocality`/`regression_certification`;
  `compact_level` ← `pipeline_compact_one_level`, tests de `maintenance`.
- **Referencias salientes:** `File` (archivo de segmento), `Header` (formato v2/v3),
  `sync_parent_dir` (durabilidad del rename), `RwLock<File>` del motor (se toma el guard de
  escritura, ya bajo `insert_lock`).
- **Veredicto de impacto:** el cambio altera el **tamaño en disco** de un segmento reclamado
  (4096 B en vez de dejar el rango muerto) y el `status`/`action` del certificado para
  `vector_store`. No cambia el layout de un segmento con nodos vivos, ni el de los nodos
  (offsets y cabeceras intactos).

## Steps

1. ✅ RED: test `certified_delete_of_the_only_record_leaves_no_vector_bytes` (falla con 1 residuo).
2. ✅ `archive::reset_empty_segment` (tmp+rename+fsync, formato preservado, wipe in-memory).
3. ✅ `compact_layout_bfs`: grafo vacío ⇒ reescribe en vez de salir.
4. ✅ `compact_level`: reclamar ⇒ reescribe; guarda si algún vivo no se promovió.
5. ✅ `vector_store_surface` mide; `cert_claims_zero` ignora `not-assessed`.
6. ✅ Docs: `docs/api/CERTIFIED_DELETE.md`, Backlog, avance.

## Verificación

```
cargo test -p vantadb --test certified_delete            # 12 hilos → 15/15 (antes 14/14 + el nuevo en rojo)
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  -E 'test(compact) or test(erasure) or test(certified) or test(maintenance) or test(vstore)' \
  --no-tests fail                                        # 148/148
```

`#[serial]` / `#[ignore]`: ninguno.

## Deuda declarada (`ponytail:`)

- El escaneo de `vector_store` recorre los segmentos mapeados completos (O(bytes de los 4
  segmentos)) por cada certificado. Aceptable en una ruta de clase mantenimiento; si algún día
  se emiten certificados en lote, medir primero.
- Un segmento **cifrado** sigue en `not-assessed`: la medición plaintext no aplica.
- **Techo declarado (candidato a `FIND`, sin ID):** el path certificado reescribe **sólo L0**
  (`compact_layout_bfs`); `compact_level` es multi-nivel y sólo lo invoca `pipeline()`. Un
  registro que viva en `L1+` (alcanzable tras una compactación por pipeline) conserva sus bytes
  allí tras el borrado certificado. Ya **no** produce un `purged` falso — la superficie
  `vector_store` mide los segmentos, así que el certificado sale en `residues` con el conteo
  real — pero el 0 no está garantizado para ese estado. Cerrarlo exige encadenar
  `compact_level` sobre los niveles que retengan nodos promovidos.
