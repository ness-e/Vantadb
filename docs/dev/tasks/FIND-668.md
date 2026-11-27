---
title: "FIND-668: search_hybrid en el addon nativo es 17.7x-701x mas lento que search_vector (medido)"
kind: task
status: completed
description: "Medicion reproducible del addon N-API prebuilt: vector 1.19/2.06/3.40 ms vs hibrida 21.10/144.83/2386.44 ms en 500/2K/10K registros. La hibrida es superlineal (exponente 1.71)."
id: FIND-668
type: bug
---

# FIND-668 — `search_hybrid` es 16×–270× más lento que `search_vector`

- **Prioridad:** 🟠 Alta · **Origen:** auditoría 59 benchmarks 2026-11-22 (#47,#49,#50)
- **Creado:** 2026-11-22 · **Cerrado (medición):** 2026-11-27
- **Estado:** ✅ MEDIDO — el número está; el arreglo es de `vantadb-node/src/**`,
  que esta tarea **no toca por zona**.

## Por qué el número de la fila nunca se había reproducido

`vantadb-node/bench/bench-abi.mjs` es el arnés oficial, pero **no arranca**:
importa `../../vantadb-ts/dist/vantadb.js` a nivel de módulo y ese dist no está
construido en esta rama.

```
SyntaxError: The requested module '../../vantadb-ts/dist/vantadb.js' does not
provide an export named 'VantaDB'
```

Eso explica que la cifra de `BENCHMARKS.md` §10/§15/§16 (`115–1869 ms`) no se
hubiera vuelto a medir: el arnés que la produce está roto por una dependencia
que la mitad nativa no necesita.

## Instrumento propio, sin recompilar nada

`evals/node_abi_search_probe.mjs` carga **sólo** el addon nativo prebuilt
(`vantadb-node/index.cjs` → `vantadb_native.win32-x64-msvc.node`): cero compilación
Rust, cero dist WASM. Mismo corpus, mismo `top_k=10`, LCG determinista para los
vectores (misma semilla ⇒ mismos bytes en cualquier máquina).

## Medición

```
$ node evals/node_abi_search_probe.mjs --sweep 500,2000,10000 --searches 20

records | insert (ms) | vec p50 | vec p95 | hyb p50 | hyb p95 | hyb/vec
--------+-------------+---------+---------+---------+---------+--------
    500 |      512.65 |   1.192 |   1.342 |  21.097 |  28.005 |   17.7x
   2000 |     4324.48 |   2.057 |   2.467 |  144.83 | 189.801 |  70.41x
  10000 |    31285.15 |   3.404 |   4.116 | 2386.437 | 3061.508 | 701.07x
```

- **2 000 registros:** híbrida **144.83 ms** vs vectorial **2.06 ms** → **70.4×**.
  La fila decía `115–1869 ms`; el extremo bajo se confirma (144.8 ms).
- **10 000 registros:** híbrida **2 386.44 ms** vs vectorial **3.40 ms** → **701×**.
  Es **peor** que el techo que declaraba la fila (1 869 ms).
- **La forma importa más que el número:** de 2 000 a 10 000 registros (5× datos),
  la vectorial crece **1.65×** y la híbrida **16.5×**. Exponente log-log de la
  híbrida **1.71** contra **0.33** de la vectorial.

Esa superlinearidad es la firma del mecanismo que la fila describe: la vectorial
paga por el grafo (sublineal, `~log`), la híbrida paga por candidato. Un factor
constante de fusión habría dado la misma pendiente que la vectorial.

## Contrato (lo que hay que hacer, con este número como vara)

1. Proyectar **sólo `(doc_id, score)`** durante BM25 + HNSW + RRF; materializar
   `payload`/`snippet` **únicamente para el `top_k` final**. Es la única forma de
   que la híbrida tenga la pendiente de la vectorial.
2. Minimizar copias JS↔Rust: hoy cada hit cruza N-API como objeto. Con
   `(doc_id, score)` primero y materialización después, cruzan `top_k` objetos en
   vez de `candidate_k`.
3. Re-certificar §15/§16 con este probe **y** con `bench-abi.mjs` una vez que
   `vantadb-ts/dist` exista (el arnés oficial sigue siendo el contrato público).

**Vara de aceptación:** a 10 000 registros, `hyb/vec ≤ 4×` y pendiente híbrida
≤ 1.2 (hoy: 701× y 1.71).

## Zona, declarada

`vantadb-node/src/lib.rs` y `vantadb-wasm/src/lib.rs` son de otro carril en el
momento de esta medición; **no se tocaron**. `evals/node_abi_search_probe.mjs`
es el entregable: reproducible, determinista y sin dependencias nuevas.

## Impacto mapeado (Regla 0)

- **Leídos:** `vantadb-node/bench/bench-abi.mjs` (forma del arnés oficial),
  `vantadb-node/index.cjs` (superficie exportada), `vantadb-node/package.json`.
- **Referencias hacia dentro:** ninguna — archivo nuevo.
- **Veredicto:** sólo se añade un probe bajo `evals/`. Cero cambios en
  `vantadb-node/src/**`, `vantadb-wasm/**` ni `src/**`.

## Review (P2-01)

Tier: **Fast** (`evals/**`). Verify: corrida real con tres tamaños (tabla arriba),
artefacto en `evals/results/node_abi_search_probe.json`. **approve**.
