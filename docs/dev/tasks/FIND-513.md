---
title: "FIND-513: el indice de texto deja el token y la clave del registro en claro"
kind: task
status: in-progress
description: "La clave de un posting es namespace/token/record_key: el token y la clave del registro quedan legibles en disco aunque el valor este cifrado. Entregado en este lote: el limite declarado en SECURITY.md con su magnitud medida y pinneado por test, mas el diseno decidido de la clave ciega (HMAC) con su plan de reindexado. La implementacion queda abierta."
---

# FIND-513: fuga del índice de texto (clave en claro)

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-513`)
- **Origen:** auditoría profunda v0.9.0 (2026-10-20), wave W4 (backend KV) · **Dep:** `FIND-407`
- **Esfuerzo:** 2-3 días (la fila completa) · **Prioridad:** 🟠 Alta
- **Creado / cerrado:** 2026-11-27
- **Estado:** 🟡 IN PROGRESS (mitad 1 entregada: límite declarado + diseño decidido; la implementación de la clave ciega queda abierta — ver §Deuda)

## Premisa (medida)

`src/text_index.rs:535-543`:

```rust
pub(crate) fn posting_key(namespace: &str, token: &str, key: &str) -> Vec<u8> {
    namespace ‖ 0x00 ‖ token ‖ 0x00 ‖ key      // ← token y key, en claro
}
```

El **valor** (tf, posiciones, node id) va cifrado por el backend (`kv_crypto`), pero la **clave** no: el AEAD del backend liga el valor a su clave (moverlo falla) y no cifra la clave (`ADR-0057 §5.3`). Consecuencias medidas:

- El **token** de cada posting es una palabra del payload indexado ⇒ legible con `strings`.
- La **clave del registro** (el id del documento) ⇒ legible y **correlacionable**: quien lea el índice sabe *qué documento* contiene *qué palabra*, aunque no pueda leer el payload.
- El **número y el orden** de las claves de un rango son observables (el KV ordena) ⇒ cardinalidad del término y orden de inserción.
- El **tamaño** del valor se filtra: `longitud = plaintext + 28` (nonce 12 + tag 16).

Lo que **sí** está protegido: el contenido del posting y el payload del nodo. La fuga es de **metadatos y vocabulario**, no de payload.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/text_index.rs` (claves: `posting_key`, `posting_prefix`, `posting_namespace_prefix`, `posting_record_key`, `term_stats_key`, `doc_stats_key`, `namespace_stats_key`, `INTERNAL_PREFIX`), `src/backends/kv_crypto.rs` (envelope por valor, `identifier` del AAD), `docs/user/operations/SECURITY.md` § Encryption at Rest.
- **Referencias hacia dentro (si se implementara la mitad 2):** `posting_key` (2 call sites de escritura), `posting_prefix` (lecturas por token), `posting_namespace_prefix` (`text_index_key_belongs_to_namespace`, purgas por namespace), `posting_record_key` (borrado por registro), `term_stats_key`/`doc_stats_key` (BM25).
- **Referencias entrantes:** sólo `src/text_index.rs` y sus consumidores internos; **ningún binding expone el formato de la clave**.
- **Veredicto impacto:** **nulo en código** en esta entrega (sólo doc + test de pin). El diseño de la mitad 2 tiene impacto **alto en formato** — por eso no se ejecuta aquí.

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Se implementa el cifrado de la clave en este lote? | (a) sí, cambiar el layout de la clave y **reindexar** al abrir (el índice de texto es derivado y reconstruible) · (b) **no**: es un cambio de formato en disco de un índice escrito, y la regla del lote es que eso lo decide el owner · (c) dejarlo sin tocar y sin documentar | ✅ (b) + (1) del contrato: límite declarado con su magnitud + diseño decidido. (a) es técnicamente posible (el índice es derivado: `STORAGE_VERSIONING.md:97`) pero implica un reindexado completo en el primer open y un marcador de versión nuevo; es una decisión de release, no un fix de worker. |
| 2 | ¿Dónde se declara el límite? | (a) ADR-0057 §5.3 (ya dice "las claves no se cifran") · (b) **`SECURITY.md` § Encryption at Rest**, junto a los demás límites que lee el operador · (c) los dos | ✅ (c): `SECURITY.md` para el operador (con la magnitud) y el ADR ya lo dice en general. |
| 3 | ¿Cómo se evita que el límite se pierda? | (a) sólo prosa · (b) **un test que pincha el layout de la clave y exige que el doc lo declare** | ✅ (b): si alguien cambia la clave sin actualizar el doc (o al revés), el test falla. |
| 4 | ¿Qué diseño se decide para la mitad 2? | ver §Diseño | ✅ blindex determinista por HMAC, marcador de versión + reindexado |

## Diseño decidido de la mitad 2 (no implementado)

> **Corregido en la ronda P2-01 (M3).** La primera versión de este diseño daba por
> hecho que el posting y las doc-stats guardaban el token y el `record_key`; el
> código dice lo contrario y el diseño se apoya ahora en lo que hay.

Lo que hay hoy (verificado en el código):

| Pieza | Contenido real | Consecuencia |
|---|---|---|
| `TextPosting` (`text_index.rs:98`) | `{ node_id, tf, positions }` — **sin token** | un posting no puede filtrar un falso positivo de token por sí mismo |
| `TextDocStats` | `{ node_id, doc_len }` — **sin set de tokens** | no existe hoy un borrado “por registro”: `posting_prefix` solo se usa en `lexical.rs:66` para buscar, no para borrar |
| `lexical.rs:77-92` | saca el `record_key` **de la clave** y lo usa para leer `doc_stats` | con la clave ciega ese `record_key` desaparece: **BM25 se queda sin `doc_len`** |

**Idea:** clave *blindex* determinista con el prefijo de namespace intacto (para
conservar los escaneos por rango):

```
k_blind     = HKDF-SHA256(DEK_text_index, "vanta:text-index:blind:v1")
posting     = ns ‖ 0x00 ‖ HMAC(k_blind, token)[0..8]
valor       = postcard(TextPosting { node_id, tf, positions })   // sin cambios
doc_stats   = 0xFF "vanta_text_v4" 0x00 TAG_DOC ‖ ns ‖ node_id    // ← deja de llevar el record_key
term_stats  = 0xFF "vanta_text_v4" 0x00 TAG_TERM ‖ ns ‖ 0x00 ‖ HMAC(k_blind, token)[0..8]
record_idx  = 0xFF "vanta_text_v4" 0x00 TAG_REC ‖ ns ‖ 0x00 ‖ HMAC(k_blind, record_key)[0..8]
valor       = postcard(Vec<u64>)   // los tags de token de ese registro (cifrado, como todo valor)
```

Los tres cambios obligatorios que la primera versión **no** decía:

1. **`doc_stats` se keyea por `node_id`**, no por `record_key`: el `node_id` ya
   viaja en el posting, así que BM25 resuelve `doc_len` sin necesitar la clave.
   Eso obliga a tocar `lexical.rs:77-92` (deja de derivar el `record_key` de la
   clave) y a la purga por registro (`delete`), que hoy recorre tokens.
2. **Índice inverso `record_idx`** para el borrado: el `delete` conoce el
   `record_key`, calcula su tag ciego y lee la lista de tokens del registro
   (valor cifrado). Sin él, borrar un documento exigiría reindexar.
3. **Token en la clave, no en el valor**: si se quiere descartar un falso
   positivo por colisión del tag de 64 bits, hay que **añadir** el token al
   valor del posting (hoy no está) o aceptar el riesgo residual (2⁻⁶⁴ por par,
   declarado).

**Se conserva:** lookup por `(ns, token)` (escaneo por prefijo con el tag del
token), borrado por registro (vía `record_idx`), purga por namespace (el
prefijo `ns\0` sigue en claro) y BM25 (con `doc_stats` por `node_id`).

**Se pierde:** leer el vocabulario en claro — que es el objetivo — y la
posibilidad de enumerar términos de un namespace (hoy tampoco hay API pública).

**Migración:** marcador `\xffvanta_text_v4\0` + **rebuild del índice de texto**
en el primer open (es materialización derivada de datos canónicos,
`STORAGE_VERSIONING.md:97`). Sin migración in-place: se descarta el prefijo
viejo.

**Coste:** el índice deja de ser legible por un operador humano (hoy sirve para
depurar); se documenta en el runbook. Y el borrado pasa a necesitar
`record_idx` (una escritura más por documento indexado).

## Steps

1. ✅ Test RED `text_index_posting_key_leak_is_declared_in_the_security_guide`: pincha la fuga (token y clave legibles en la clave) **y** exige que `SECURITY.md` la declare — falla en `HEAD` porque el doc no la nombra.
2. ✅ Sección propia en `SECURITY.md` § Encryption at Rest con la magnitud medida.
3. ✅ Diseño de la mitad 2 en este task file (§Diseño) con la decisión de reindexado.
4. ✅ GREEN + gates.

## Contrato de verificación

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(text_index) or test(posting) or test(kv_crypto)'   # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail
```

**Invariante que no se puede romper:** ningún camino de escritura/lectura del índice cambia de comportamiento en esta entrega (es doc + pin); el test que pincha el layout falla si alguien cambia la clave sin tocar el doc.

## Deuda declarada

- **La mitad 2 (clave cifrada) NO está implementada.** Requiere: (a) decisión del owner sobre el reindexado obligatorio en el primer open de un índice de texto existente, (b) bump del marcador `INTERNAL_PREFIX` a `v4`, (c) derivar `k_blind` del registro (una DEK nueva o derivada de la del backend). Es media wave, no un fix. Queda con su contrato cumplido (límite declarado + diseño decidido) y **sin** cerrar la parte de implementación.

## Context Save Point

- **Hecho:** `src/text_index.rs` (test de pin), `docs/user/operations/SECURITY.md`, este task file, `Backlog.md`, `avance/activo/core-engine.md`.
- **No hacer:** cambiar `posting_key` sin el marcador `v4` y el rebuild — el índice viejo devolvería *cero* resultados en silencio (no error), que es peor que la fuga.

## Review

### P2-01 ronda 1 (contexto fresco, `vanta-review`, agente distinto) - **changes-required**, aplicado

VERDICT: changes-required con **0 CRITICAL / 0 HIGH / 7 MEDIUM / 9 LOW**. Confirmado sano: cobertura total del AAD (los 4 constructores pasan `identifier`), identidad estable en rotacion y shards, compaction cifrada coherente, rechazo en el open de vfile/WAL, FIND-496 bien acotado y FIND-507 en el orden correcto.

Aplicado en la misma ronda: **M1** (`verify_wal_file` rechaza el v4 legacy y reporta la version real), **M2** (`migrate`/`check_integrity` dicen "refused / rewrite", no "newer"), **M3** (§Diseno de FIND-513 reescrito con lo que el codigo tiene de verdad), **M4** (la medicion de 343 limpia la clave y asserta base plana), **M5** (`encryption_block` sin fail-open y sin segundo recorrido), **M6** (el Step 5 falso de 507 corregido), **M7** (el pin de rotacion, commiteado) y los LOW de prosa (L1, L6, L7, L8, L9).

No aplicado y declarado: **L2** (un hunk ajeno de `FIND-669` horneado en el commit de cripto: ruido de carril, no enmendable sin `--amend`), **L3** (fila `FIND-500` colgando: es de otra fila, fuera de las 5 del lote) y **L4** (la deuda de crypto-shredding queda en el §Deuda de 343 en vez de como fila nueva: los IDs los asigna el orquestador).

