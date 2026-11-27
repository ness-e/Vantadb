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
- **Estado:** ✅ COMPLETED (mitad 1 + diseño de la mitad 2; mitad 2 **no** implementada, ver §Deuda)

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

**Idea:** clave *blindex* determinista, prefijo de namespace intacto (para conservar los escaneos por rango):

```
k_blind      = HKDF-SHA256(DEK_text_index, "vanta:text-index:blind:v1")
posting_key  = ns ‖ 0x00 ‖ HMAC(k_blind, token)[0..8] ‖ 0x00 ‖ HMAC(k_blind, token ‖ 0x00 ‖ record_key)[0..16]
term_stats   = 0xFF "vanta_text_v4" 0x00 TAG_TERM ‖ ns ‖ 0x00 ‖ HMAC(k_blind, token)[0..8]
doc_stats    = 0xFF "vanta_text_v4" 0x00 TAG_DOC  ‖ ns ‖ 0x00 ‖ HMAC(k_blind, record_key)[0..8]
```

- **Se conserva:** lookup por `(ns, token)` (escaneo por prefijo con el tag del token), borrado por `(ns, token, key)` (los tres se conocen), purga por namespace (el prefijo `ns\0` sigue en claro), y el borrado por registro usando el conjunto de tokens de `doc_stats` (su **valor** ya va cifrado).
- **Se pierde:** enumerar el vocabulario de un namespace (hoy tampoco hay API pública que lo haga), y la posibilidad de leer el término en claro — que es el objetivo.
- **Colisiones:** 64 bits para el token y 128 para `(token,key)`. Un falso positivo de token se descarta comparando el token real que el **valor** del posting ya contiene; 2^-64 de colisión por par es aceptable y se documenta.
- **Migración:** marcador `\xffvanta_text_v4\0` + **rebuild del índice de texto** en el primer open (derivado de datos canónicos). No hay migración de claves in-place: se descarta el prefijo viejo.
- **Coste:** el índice de texto deja de ser legible por un operador humano (hoy sirve para depurar). Aceptado, se documenta en el runbook.

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
