---
title: "FIND-830: el indice derivado y los sparse postings llevaban la clave del registro en claro"
kind: task
status: complete
description: "Tres familias de claves derivadas (namespace index, payload index, sparse postings) llevaban el record_key legible aunque los postings de texto ya no. Entregado: la misma primitiva tag8, dominios por familia, marcador de version con rebuild y el limite declarado. RED 3/3 familias -> GREEN 0/3."
---

# FIND-830: fuga de la clave del registro en el índice derivado y los sparse postings

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-830`)
- **Origen:** cierre de `FIND-513` (2026-11-27) · **Ver:** `FIND-513`, `FIND-829`
- **Esfuerzo:** 2-4 h (estimado en la fila) · **Prioridad:** 🟠 Alta
- **Creado / cerrado:** 2026-11-27 / 2026-11-28
- **Estado:** ✅ COMPLETE

## Premisa (medida, no citada)

Las tres funciones que la fila nombra llevaban el `record_key` en claro:

| Archivo | Función | Clave |
|---|---|---|
| `src/sdk/serialization/mod.rs:295` | `namespace_index_key` | `namespace ‖ 0x00 ‖ record_key` |
| `src/sdk/serialization/mod.rs:373` | `payload_index_key` | `prefix ‖ record_key` |
| `src/sdk/serialization/impl_sparse_index.rs:49` | `sparse_posting_key` | `v1 ‖ 0x00 ‖ namespace ‖ 0x00 ‖ dim ‖ 0x00 ‖ record_key` |

Medición de la fila, ejecutada **antes** del fix sobre un índice real (un
registro con `key = invoice-2026-0001`, un metadato y un dim sparse):

```
cargo nextest run -p vantadb --profile audit --build-jobs 2 -E 'test(f830_no_clear_record_key)'
→ the cleartext record key is in 3 of 3 key families:
  ["NamespaceIndex: 1", "PayloadIndex: 1", "SparseIndex: 1"]
```

**RED: 3/3 familias, 1 coincidencia cada una.** El escaneo es de **bytes** sobre
las claves persistidas (no `contains` sobre el nombre del constructor), y la
población de cada partición se asserta antes (§test), así que un cero no puede
ser vacuo.

Por qué importa más que en `FIND-513`: no expone sólo *la clave* — el índice de
payload con la clave al lado es un **mapa registro → payload** (`invoice-…` tiene
`owner = eros`). El valor va sellado por el AEAD del backend; la clave no
(`ADR-0057 §5.3`).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la parte afectada):**
  `src/text_index.rs` (`blind_tag`, `token_tag`, `record_tag`, `posting_key`,
  `posting_prefix`, `BLIND_TAG_LEN`, `INTERNAL_PREFIX`),
  `src/sdk/serialization/mod.rs` (las cuatro funciones de clave + `encoded_scalar_value`),
  `src/sdk/serialization/impl_index.rs` (state, `derived_put_ops`/`derived_delete_ops`,
  `ensure_derived_indexes_current_with`, `current_derived_index_counts`),
  `src/sdk/serialization/impl_sparse_index.rs` (claves, state, rebuild),
  `src/sdk/serialization/impl_rebuild.rs` (ambos rebuilds + el test `f513`),
  `src/sdk/serialization/impl_export.rs` (`indexed_ids_by_namespace`,
  `indexed_ids_by_filter`), `src/sdk/api/namespaces.rs` (`list_namespaces`,
  `list`), `src/sdk/search/sparse.rs` (`scan_partition_prefix_iter`), `src/attestation.rs`
  (consumidor de `namespace_index_key`).
- **Referencias hacia dentro:** `namespace_index_key` (2 write + 1 delete + `attestation`
  recheck), `payload_index_key` (2 write + 1 delete), `sparse_posting_key` (2 write + 1
  delete), `namespace_index_prefix` (`list_namespaces`, `list`, export), `payload_index_prefix`
  (`list` con `filter_ops`), `sparse_posting_prefix` (`sparse_memory_search`).
- **Ningún lector parsea el `record_key` de la clave**: las tres rutas de lectura
  usan el **valor** (`node_id_bytes`) o el prefijo. Es lo que hace barato el cambio.
- **Ningún binding expone el formato de la clave.**
- **Veredicto impacto:** alto en formato on-disk, nulo en API pública; los tres
  índices son materializaciones derivadas (`STORAGE_VERSIONING.md` § Derived), así
  que el camino es rebuild, no migración in-place (D2 permite el cambio pre-1.0).

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|---|---|---|
| 1 | ¿Primitiva nueva o la de `FIND-513`? | (a) **reusar `blind_tag`**, exponerla `pub(crate)` · (b) moverla a un módulo neutral (`src/blind.rs`): una capa más y un archivo nuevo para 8 líneas · (c) reimplementar `SHA-256` en serialization: dos construcciones que pueden divergir | ✅ (a). `serialization` ya depende de `crate::text_index` (`parse_term_stats_key`, `is_term_stats_key`, `TextNamespaceStats`), así que no añade arista de dependencia; una sola construcción para las cuatro familias. Firma por bytes (`&[u8]`) porque es lo que `Sha256::update` consume: los tres call sites pasan `key.as_bytes()` y `&str` **nunca** excluyó `0x00`, así que el tipo no compraba nada y obligaba a un envoltorio por cada valor ya en bytes (`encoded_scalar_value`, el día que `FIND-829` ciegue `field`/`value`). |
| 2 | ¿Dominio compartido o por familia? | (a) un dominio para todas: los tags del mismo registro coincidirían entre particiones y las claves se podrían **unir** (namespace ↔ payload ↔ sparse) · (b) **un dominio por familia** | ✅ (b): `vanta:derived-index:namespace:v2`, `…:payload:v2`, `vanta:sparse-index:record:v2`. |
| 3 | ¿Se ciega también `field`/`value` del payload index? | (a) sí: `tag8(field)`/`tag8(value)` — compatible con el escaneo por prefijo (el llamador tiene ambos) · (b) **no** | ✅ (b) **en esta fila**: la asociación *registro → payload* es lo que filtraba, y ya está rota por el tag de la clave; cegar field/value no compra confidencialidad frente a quien pueda hashear un diccionario sin clave (`FIND-829`) y **rompe** la introspección del índice (lo único que hoy permite ver qué campos se indexan). Queda declarado en `SECURITY.md` como límite explícito. |
| 4 | ¿Y el namespace? | (a) cegarlo · (b) **dejarlo en claro** | ✅ (b), igual que `FIND-513`: es el prefijo de escaneo (`namespace_index_prefix`, `posting_namespace_prefix`) y de él dependen `list_namespaces` (parsea hasta el primer `0x00`) y las purgas por namespace. `ADR-0057 §5.3.1` ya lo declara. |
| 5 | ¿Marcador de versión? | (a) sólo prosa en el doc · (b) **bump de `schema_version` del estado**, que es el mecanismo que estas dos particiones ya usaban · (c) añadir un `key_format` a los estados | ✅ (b) + `SPARSE_INDEX_KEY_PREFIX` `v1`→`v2`. El estado de derived/sparse lleva `schema_version` desde el primer día y `ensure_*_current` ya compara; añadir `key_format` (como el índice de texto) sería un tercer campo que nadie compara salvo por duplicar la comprobación. El prefijo `v2` de sparse hace que una entrada vieja **ni siquiera esté en el keyspace nuevo**. |
| 6 | ¿Se implementa `k_blind` aquí? | (a) sí · (b) **no**, fila `FIND-829` | ✅ (b): exige un secreto por base plumbado a ocho helpers puros sin handle del motor y toca el registro de claves (`ADR-0057 D2`). Fila propia con medición y contrato. |

## Implementación

```
namespace_index_key = namespace ‖ 0x00 ‖ tag8(ns-index, record_key)
payload_index_key   = namespace ‖ 0x00 ‖ field ‖ 0x00 ‖ encoded_value ‖ 0x00 ‖ tag8(payload-index, record_key)
sparse_posting_key  = "v2" ‖ 0x00 ‖ namespace ‖ 0x00 ‖ dim ‖ 0x00 ‖ tag8(sparse, record_key)
tag8(domain, x)     = SHA-256(domain ‖ x)[0..8]        // crate::text_index::blind_tag
```

| Pieza | Antes | Ahora |
|---|---|---|
| `namespace_index_key` (`mod.rs`) | `ns\0key` | `ns\0tag8(key)` |
| `payload_index_key` (`mod.rs`) | `prefix\0key` | `prefix‖tag8(key)` (el prefijo ya termina en `0x00`) |
| `sparse_posting_key` (`impl_sparse_index.rs`) | `v1\0ns\0dim\0key` | `v2\0ns\0dim\0tag8(key)` |
| `sparse_posting_prefix` | `v1\0ns\0dim\0` | `v2\0ns\0dim\0` — **misma forma**, sigue siendo prefijo exacto de la clave |
| `namespace_index_prefix` / `payload_index_prefix` | sin cambios | sin cambios: nunca llevaron la clave |
| estado derived | `DERIVED_INDEX_SCHEMA_VERSION = 1` | `= 2` ⇒ rebuild en el próximo open escribible |
| estado sparse | `SPARSE_INDEX_SCHEMA_VERSION = 1` | `= 2` ⇒ rebuild, y las claves `v1` quedan fuera del keyspace |
| `blind_tag` (`text_index.rs`) | `fn …(&[u8], &str)` | `pub(crate) fn …(&[u8], &[u8])` |

**El range scan de sparse no cambió de forma**: `sparse_posting_prefix(ns, dim)`
sigue siendo prefijo de `sparse_posting_key(ns, dim, key)`, con el tag **después**
del prefijo — igual que `posting_prefix` tras `FIND-513`. Lo pincha
`sparse_posting_key_roundtrip` (prefijo + tag detrás + `--no-tests fail`) y lo
ejercita de extremo a extremo `f830_clear_key_indexes_are_rejected_and_rebuilt`
(búsqueda sparse antes y después del rebuild, mismo resultado).

## Steps

1. ✅ **RED (medición):** `f830_no_clear_record_key_in_derived_or_sparse_keys` —
   escaneo de bytes de las tres particiones de un índice real + población
   assertada + pin del doc. Falló con **3/3 familias**.
2. ✅ **RED (migración):** `f830_clear_key_indexes_are_rejected_and_rebuilt` —
   (1) estado bajado a v1 ⇒ el reopen debe dejar estado corriente y los mismos
   resultados; (2) una clave v1 con el `record_key` en claro en cada partición ⇒
   el rebuild debe borrarla.
3. ✅ **GREEN:** `blind_tag` `pub(crate)` y por bytes; dominios por familia; las
   tres claves ciegas; bumps de estado; prefijo sparse `v2`.
4. ✅ `SECURITY.md` § "Derived index and sparse postings (FIND-830)" — qué deja de
   ser legible (la asociación registro→payload) y qué **no** (namespace, field,
   value, y la recuperación por diccionario: `FIND-829`, medido).
5. ✅ `STORAGE_VERSIONING.md` — tabla de marcadores y regla de rebuild.
6. ✅ Gates: `cargo fmt --check`, `cargo clippy --workspace --all-targets
   --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace
   --build-jobs 2`, filtro del contrato con `--no-tests fail`. **Corregido en la
   ronda P2-01:** el gate que se midió de verdad fue `cargo fmt --all -- --check` +
   `cargo clippy -p vantadb --lib --all-features -- -D warnings` (§Review, nota de
   proceso).
7. ✅ **Ronda P2-01 — R1 (read-only):** `ensure_sparse_index_query_ready` + test
   `sparse_search_on_a_read_only_open_of_an_old_index_fails_loudly` (rojo: `[]`;
   verde: error explícito). Textos de `SECURITY.md` / `STORAGE_VERSIONING.md`
   acotados y R2/R3 corregidos (§Review).

## Contrato de verificación

```
cargo nextest run -p vantadb --profile audit --build-jobs 2 \
  -E 'test(f830) or test(f829) or test(sparse_posting_key) or test(namespace_index_key) or test(payload_index_key) or test(sparse_search_on_a_read_only)' \
  --no-tests fail
```

Medido al cerrar la ronda P2-01: población **7** tests del filtro
(`f829_…dictionary_measured`, `f830_no_clear_record_key…`,
`f830_clear_key_indexes…`, `sparse_search_on_a_read_only_open_of_an_old_index_fails_loudly`,
`sparse_posting_key_roundtrip`, `test_namespace_index_key_format`,
`test_payload_index_key`) → **7/7 verdes** con `--no-tests fail`; y el barrido del
área (`test(serialization) or test(text_index) or test(sparse) or test(posting) or
test(derived) or test(namespace) or test(rebuild) or test(index)`) → **664/664**;
y la suite del crate (`cargo nextest run -p vantadb --profile audit
--build-jobs 2`) → **3033/3033**.

**Invariante que no se puede romper:** los prefijos de las tres familias siguen
siendo prefijos de sus claves (los escaneos por namespace, por filtro y por dim no
cambian); ninguna clave nueva lleva el `record_key` en claro; y **ningún índice con
layout viejo puede responder a una consulta** — ni con resultados vacíos. Ese
último es el que faltaba: el rebuild sólo corre en open **escribible**, así que el
rechazo tiene que estar en el camino de consulta (§Review R1).

## Review

### P2-01, ronda 1 (contexto fresco) — changes-required, aplicado

Verdict del reviewer: **1 fallo silencioso real**, 2 textos que mentían en la
dirección tranquilizadora, 1 justificación que no correspondía al código.

**R1 — apertura read-only: sparse devolvía vacío sin aviso (aplicado).**
`src/sdk/builder.rs:244-246` salta `ensure_indexes_current` cuando
`config.read_only`, y `read_only` es opción pública (`Config::with_read_only`). El
índice derivado sobrevive porque su prefijo no cambió; **sparse no**:
`SPARSE_INDEX_KEY_PREFIX` pasó `v1`→`v2` y `sparse_memory_search` sólo escanea el
prefijo nuevo ⇒ una base pre-`FIND-830` abierta read-only devolvía **lista vacía**,
indistinguible de "no hay coincidencias" — el modo de fallo que este mismo task
file llama "peor que la fuga".

Opción elegida: **(a) fallar en voz alta**, `ensure_sparse_index_query_ready`
(`src/sdk/search/sparse.rs`), llamada al principio de `sparse_memory_search`.
Razones, en orden: (i) es la que **no miente** — un error se ve, un `[]` no; (ii) es
la que el índice de texto **ya** usa (`ensure_text_index_query_ready`, y
`tests/text_index_recovery.rs:741-753` pina ese comportamiento), así que hay **un**
comportamiento que el operador puede razonar en vez de dos; (iii) el rebuild está a
**un open escribible** de distancia y el mensaje lo dice; (iv) el fallback (b)
añade un segundo camino de scoring que ningún test ejercita hoy y **oculta** un
despliegue mal configurado — pagar código nuevo para esconder el síntoma es lo
contrario de lo que la fila pide.

Test: `sparse_search_on_a_read_only_open_of_an_old_index_fails_loudly`
(`src/sdk/search/sparse.rs`). Escribe la base, comprueba que **antes** del
downgrade la búsqueda devuelve 1 hit (si no, el test no probaría nada), reescribe
las postings al layout `v1` + baja el estado a `v1`, reabre **read-only** y exige
`Err` con `sparse_index_schema` y "reopen it writable". **Rojo medido**: sin el
guard, la búsqueda devolvía `[]` (no un error) — el test fallaba con
`a stale sparse index must not answer an empty list: []`.

**R2 — la colisión de tag estaba descrita al revés (aplicado).** §Deuda: ahora
dice que un `Put` con la misma clave **sobreescribe** (`impl_index.rs:118-140`) ⇒
una entrada para dos registros ⇒ `namespace_entries < canonical_records` ⇒
**rebuild en cada open escribible para siempre** y el registro colisionado
**desaparece de `list()`** (clase del CRITICAL de `FIND-812`), no "una entrada de
más".

**R3 — el absoluto heredado (aplicado).** `FIND-829.md` §2.3: "rechazado, nunca
leído como v5" queda acotado a **el siguiente open escribible**, y dice que el
rechazo efectivo en read-only lo hace el **guard del camino de consulta**, no el
rebuild.

**Texto del `&[u8]` (aplicado).** La justificación de la decisión 1 decía que el
payload index planta un valor ya codificado — falso: los tres call sites pasan
`key.as_bytes()` (`mod.rs:414` incluido). Ahora dice lo que el código hace: bytes
porque es lo que `Sha256::update` consume, y `&str` **nunca** excluyó `0x00`, así
que el tipo no compraba nada.

**Textos que mentían (aplicado).** `SECURITY.md` y `STORAGE_VERSIONING.md` decían
"an old index is never read as if it were new": falso con read-only. Ahora dicen
que el rebuild ocurre en el **siguiente open escribible** y qué pasa mientras
tanto, por índice (derived responde bien porque el prefijo no cambió; sparse
**falla con error explícito**).

**Nota de proceso aceptada.** La ronda anterior declaró `SKIP=cargo-fmt` con el
motivo "el WIP de otro carril rompe el fmt"; el reviewer midió
`cargo fmt --all -- --check` → **0** con ese WIP incluido. El motivo era una
**suposición**, no una medición. En esta ronda el gate se midió antes de commitear
y su salida está en el mensaje del commit.

## Deuda declarada

- **El tag sigue sin clave**: `SHA-256` desnudo ⇒ recuperable por diccionario
  (medido: **40/40 tokens, 100 %**). Es `FIND-829`, con contrato en
  `docs/dev/tasks/FIND-829.md`. No se tapa en `SECURITY.md`.
- **`field`/`value` del payload index siguen en claro** (decisión 3). Declarado.
- **El namespace sigue en claro** en las cuatro familias (decisión 4). Declarado.
- **Colisión de tag**: 64 bits ⇒ 2⁻⁶⁴ por par. Si dos registros colisionan,
  el `Put` de la clave de índice **sobreescribe** (el valor es
  `node_id_bytes(record.node_id)`, `impl_index.rs:118-140`, y la clave es la
  misma): quedan **una entrada para dos registros**, no dos entradas. Eso hace
  `namespace_entries < canonical_records`, que es **condición de rebuild**
  (`impl_index.rs:62`) ⇒ el índice se reconstruye en **cada** open escribible,
  para siempre, y el registro colisionado **desaparece de `list()`** (su
  `node_id` se perdió). Es la clase del CRITICAL de `FIND-812`, no una entrada
  de más. Probabilidad despreciable (no bloquea la entrega), pero el impacto está
  descrito al derecho. Mismo residual que `FIND-513`.

## Context Save Point

- **Hecho:** `src/text_index.rs`, `src/sdk/serialization/mod.rs`,
  `src/sdk/serialization/impl_index.rs` (tests), `src/sdk/serialization/impl_sparse_index.rs`,
  `src/sdk/serialization/impl_rebuild.rs` (test), `src/sdk/search/sparse.rs`
  (guard `ensure_sparse_index_query_ready` + su test, ronda P2-01),
  `docs/user/operations/SECURITY.md`, `docs/dev/architecture/STORAGE_VERSIONING.md`,
  este task file, `Backlog.md`, `docs/dev/avance/activo/core-engine.md`.
- **No hacer:** cambiar las claves sin subir `DERIVED_INDEX_SCHEMA_VERSION` /
  `SPARSE_INDEX_SCHEMA_VERSION` y el prefijo sparse. Un índice viejo leído como
  nuevo devuelve cero resultados sin error.
