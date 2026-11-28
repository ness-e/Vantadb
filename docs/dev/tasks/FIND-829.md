---
title: "FIND-829: el tag ciego es recuperable por diccionario - falta la clave del indice"
kind: task
status: active
description: "PARCIAL (D4): medicion del ataque de diccionario contra el indice real (40/40 tokens recuperados, 100 por ciento) y contrato completo de la version con clave (k_blind). NO implementado: exige plumbar un secreto por base desde el registro de claves de ADR-0057 a ocho helpers puros sin handle del motor. Medido: k_blind aparece en 0 ficheros de src."
---

# FIND-829: el `tag8` del índice es reversible por diccionario (PARCIAL: medición + contrato)

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-829`)
- **Origen:** cierre de `FIND-513` (2026-11-27) · **Ver:** `FIND-513`, `FIND-830`
- **Prioridad:** 🟡 Media · **Estado:** 🟡 PARCIAL — **medición + contrato entregados, implementación pendiente**
- **Dueño del delta:** `vanta-arch` (el secreto por base y su registro) → después un carril Rust

## Lo que YA está hecho

### 1. Medición del ataque (el número es el hallazgo)

Precondición de la fila, medida antes de tocar nada:

```
rg -c 'k_blind' src     → 0 coincidencias (0 ficheros)
```

La clave de cifrado **sí** existe (`ADR-0057` D2: master key → KEK → DEK por
artefacto, en `src/crypto/registry.rs`). Lo que no existe es el nombre ni el uso
de un secreto para el índice.

Ataque, medido contra una partición `TextIndex` **real** (no contra la función que
construye la clave: las etiquetas se leen de las claves en disco, que es la vista
del atacante), con 40 tokens indexados y una lista de 200 candidatos:

```
cargo nextest run -p vantadb --profile audit --build-jobs 2 -E 'test(f829)' --no-tests fail
→ 1 test run: 1 passed
```

| Medición | Valor |
|---|---|
| Tokens presentes en el índice | 40 |
| Diccionario probado | 200 palabras |
| **Recuperados** | **40/40 → 100 %** |
| Falsos positivos entre las 160 ausentes | 0 |

Es decir: hoy el índice **no** protege el vocabulario ni los ids de registro
cortos (`k1`, `invoice-2026-0001`) frente a quien tenga el fichero y una lista de
palabras. `FIND-513` cerró la fuga **literal** (`strings`) y `FIND-830` cerró la
del índice derivado; ninguno de los dos cierra el ataque de diccionario, porque el
tag es `SHA-256` desnudo: sin clave, sin sal y sin estiramiento.

El test `impl_index::tests::f829_unkeyed_tag_is_recovered_by_dictionary_measured`
**asserta el número de hoy** (40/40) a propósito: el día que la versión con clave
entre, ese assert falla y obliga a invertirlo (`recovered == 0` sin la clave).

### 2. Contrato de la versión con clave (diseño, no implementado)

#### 2.1 De dónde sale el secreto

```
k_blind_text    = HKDF-SHA256(IKM = DEK_text_index, salt = db_salt,
                              info = "vanta:text-index:blind:v1", L = 32)
tag8k(domain, x) = HMAC-SHA256(k_blind, domain ‖ x)[0..8]        // en vez de SHA-256
```

- **Por qué el DEK del artefacto y no la master key ni el KEK:** `ADR-0057` D2
  define el KEK sobre `db_salt` + master key y un DEK **por artefacto**, guardado
  envuelto en el registro. Rotar la master key **re-envuelve los DEK pero no los
  cambia**; si el tag se deriva del DEK, una rotación no invalida el índice. Si se
  derivara del KEK o de la master key, **cada rotación exigiría un rebuild
  completo** de los cuatro keyspaces — un coste O(datos) en una operación que hoy
  es O(artefactos).
- **Por qué un `k_blind` propio y no el DEK crudo:** dominio separado
  (`info = "vanta:text-index:blind:v1"`). Usar el DEK como clave HMAC mezcla dos
  usos del mismo secreto (AEAD y PRF); HKDF da una clave independiente por uso y
  permite versionar el esquema de etiquetas por separado (`blind:v2` cuando haya
  que cambiar el largo o el algoritmo).
- Un `k_blind` **por keyspace**, derivado del mismo DEK con `info` distinto
  (`vanta:derived-index:blind:v1`, `vanta:sparse-index:blind:v1`), para conservar
  la separación de dominio que `FIND-830` ya introdujo.

#### 2.2 Qué pasa cuando el cifrado está apagado (la decisión abierta)

`encryption` **no está en `default`** y sin master key no hay DEK ⇒ no hay
`k_blind`. Tres opciones, ninguna gratis:

| Opción | Qué da | Qué cuesta |
|---|---|---|
| (a) sin master key, el tag queda **sin clave** (hoy) | cero fricción, `0.9.x` sigue funcionando igual | la protección depende de tener cifrado activo — hay que decirlo tal cual |
| (b) fichero de secreto local generado en el primer open | tags con clave **siempre** | gestión de un secreto nuevo para una feature que `ADR-0057 §5.3.6` declara "apagada dos veces"; el fichero es un artefacto más que respaldar/rotar |
| (c) rechazar el open sin secreto | garantía incondicional | rompe todas las bases existentes y el caso "solo quiero memoria local" |

**Recomendación (para el owner, no decidida aquí):** (a) + declaración. La
garantía queda acotada a "con cifrado activo, el índice no es reversible por
diccionario", que es una frase verdadera y medible; (b) reintroduce justo el
problema de gestión de claves que motivó dejar el cifrado fuera de `default`.

#### 2.3 Versionado y rebuild

Mismo mecanismo que `FIND-513`/`FIND-830`, con marcadores nuevos:

| Keyspace | Marcador hoy | Marcador con clave |
|---|---|---|
| Text index | `\xffvanta_text_v4\0`, schema 5 (`advanced-tokenizer`) / 4, `KEY_FORMAT = "namespace\0tag8(token)tag8(record_key)"` | `\xffvanta_text_v5\0`, schema 6/5, `KEY_FORMAT = "namespace\0tag8k(token)tag8k(record_key)"` |
| Derived (namespace + payload) | `DERIVED_INDEX_SCHEMA_VERSION = 2` | `= 3` |
| Sparse | `SPARSE_INDEX_SCHEMA_VERSION = 2` + prefijo `v2` | `= 3` + prefijo `v3` |

El rebuild ya existe y no hay que inventarlo: `ensure_indexes_current` compara
`schema_version`/`KEY_FORMAT` y borra + re-materializa desde los registros
canónicos. Un índice v4 sigue siendo **rechazado**, nunca leído como v5.

`KEY_FORMAT` tiene que cambiar **en la misma entrega** que el `k_blind`: si sólo
cambia la derivación, un índice viejo se leería con las búsquedas nuevas y
devolvería **cero resultados en silencio**.

#### 2.4 Qué falta exactamente (el delta)

1. **El secreto (Arch).** Exponer el DEK del artefacto de índice (o un
   `k_blind` ya derivado) desde el registro de claves a quien construye claves.
   Hoy `src/text_index.rs` es un módulo **puro** sin handle del motor, y
   `derived_put_ops` / `sparse_put_ops` son funciones **estáticas** sin `&self`.
   Camino recomendado: un `Arc<TagKey>` en `StorageEngine` (que ya tiene el
   registro de claves), y las funciones puras pasan a recibir `&TagKey`.

2. **Las firmas (carril Rust).** Siete constructores de clave necesitan la clave:
   `token_tag`/`record_tag` (→ `posting_key`, `posting_prefix`, `doc_stats_key`,
   `term_stats_key`), `derived_record_tag` (→ `namespace_index_key`,
   `payload_index_key`) y el tag sparse. Con ellos cambian ~10 call sites de
   escritura/borrado/lectura (`impl_text_index.rs`, `impl_index.rs`,
   `impl_sparse_index.rs`, `lexical.rs`, `sparse.rs`, `api/memory.rs` ×3 purgas,
   `attestation.rs`). Los **prefijos no llevan clave** y no cambian de forma
   (namespace index, payload `(field, value)`, `sparse_posting_prefix`).

3. **Rotación.** Verificar que rotar la master key **no** cambia los tags (test:
   rotar y reabrir → mismos resultados, estado del índice intacto). Es la razón de
   derivar del DEK y no del KEK.

4. **La declaración.** `SECURITY.md` § "Derived index and sparse postings" y
   § "Text index keys" tienen que pasar de "recoverable by dictionary" a "no
   recoverable without the key **cuando el cifrado está activo**", y la opción
   (a/b/c) de §2.2 decidida por el owner aparece tal cual.

5. **El test invertido.** `f829_unkeyed_tag_is_recovered_by_dictionary_measured`
   pasa a exigir `recovered == 0`; y el nuevo contrato de la fila: **dos bases con
   distinto `k_blind` producen tags distintos para el mismo token**.

**Coste estimado:** 2-4 h de la fila **no** cubren el plumbing del secreto; es un
día con el diseño de Arch ya escrito.

## Contrato de verificación (cuando se implemente)

```
cargo nextest run -p vantadb --profile audit --build-jobs 2 \
  -E 'test(f829) or test(f830) or test(f513) or test(text_index) or test(posting)' \
  --no-tests fail
```

1. Dos bases con `k_blind` distinto ⇒ mismo token, tag distinto.
2. Diccionario sobre el índice real ⇒ **0 coincidencias** sin la clave.
3. Rotar la master key ⇒ mismos resultados, sin rebuild del índice.
4. Índice v4/v2 viejo ⇒ rechazado y reconstruido (los guardians ya existen:
   `f513_v3_index_rebuilds_to_the_same_search_result`,
   `f830_clear_key_indexes_are_rejected_and_rebuilt`).

## Por qué NO se implementó aquí (D4, nunca cierre falso)

Lo dice la propia fila: "NO lo implementes si exige tocar el registro de claves
(fuera de zona)". Exige exactamente eso (punto 1 de §2.4), así que la fila queda
**abierta** con la medición y el contrato, no cerrada. La mitad entregada es la
que no necesita el secreto: el número medido y el diseño con sus opciones.

## Deuda declarada

- `k_blind` **no existe**: la protección del índice depende hoy de tener cifrado
  activo sólo en el sentido de que el *valor* va sellado; la **clave** nunca lo
  estuvo.
- La decisión de §2.2 (a/b/c) es del owner: cambia la garantía que se puede
  publicar.

## Context Save Point

- **Hecho:** medición (`impl_index::tests::f829_unkeyed_tag_is_recovered_by_dictionary_measured`),
  este task file, `SECURITY.md` (residual declarado), `Backlog.md` (fila → PARCIAL).
- **No hecho a propósito:** ninguna línea de `k_blind`, ningún cambio de firma.
- **Próximo paso:** `vanta-arch` decide §2.2 y expone el secreto; después un
  carril Rust ejecuta §2.4 con los cuatro tests del contrato.
