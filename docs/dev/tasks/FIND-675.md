---
title: "FIND-675: bench de borrado certificado entregado - certified NO da 0% hasta compactar (aserción FAIL)"
kind: task
status: completed
description: "benchmarks/certified_unlearning_bench.py mide recuperacion de bytes por modo y churn HNSW. Resultado: certified == hard en disco (2 archivos con payload, 3 con el vector) y 0% recien tras compact_wal+compact_layout."
id: FIND-675
type: bug
---

# FIND-675 — Benchmark de Borrado Certificado GDPR y churn HNSW

- **Prioridad:** 🟠 Alta · **Origen:** *Ghost Vectors* (`arXiv:2606.18497`, 2026) · GDPR Art. 17
- **Creado:** 2026-11-22 · **Medido:** 2026-11-27
- **Estado:** ✅ ENTREGADO — y la aserción **FALLA**, que es el resultado: el
  benchmark encontró lo que la fila sospechaba. El hueco queda como **FIND-826**.

## Parte 1 — escaneo forense (la aserción)

Método: base nueva por modo, se inserta un registro de sujeto cuyo payload lleva
un marcador único (`c0ffeec0ffeec0ffe…`, 62 hex) y cuyo vector es determinista, más
**un testigo** con vector y payload distintos que debe sobrevivir (si no, un «0
hits» podría venir de haber borrado la base). Se llama `forget(mode=…)`, `flush()`,
se cierra y se recorren **todos** los archivos buscando el marcador y los primeros
16 bytes del vector.

```
$ python benchmarks/certified_unlearning_bench.py --docs 1000

mode      | forgotten | files | payload found in | vector bytes found in | witness kept
----------+-----------+-------+------------------+-----------------------+-------------
soft      | 1         |    55 | 3                | 5                     | True
hard      | 1         |    55 | 2                | 3                     | True
certified | 1         |    55 | 2                | 3                     | True

ASSERTION certified recovery == 0 %: payload 100 %, vector 3 files -> FAIL
  payload hits: ['0.jnl (10x)', 'data\\vanta.shard0.wal (2x)']
  vector hits:  ['0.jnl (1x)', 'data\\vanta.shard0.wal (1x)', 'data\\vstore_L0.vanta (1x)']

After compact_wal() + compact_layout(): payload in 0 file(s), vector bytes in 0 file(s),
witness kept True
```

**Lectura:** en el instante en que `forget(mode="certified")` devuelve su recibo,
el registro es **byte-idéntico a `hard`**: sobrevive en el journal (`0.jnl`) y en el
WAL del shard, y el vector además en `vstore_L0.vanta`. El `0 %` que la fila exige
**sólo se alcanza tras `compact_wal()` + `compact_layout()`**, y el recibo no lo
dice. Es la clase *Ghost Vectors* exacta: el API dice `forgotten_count: 1` y el
disco dice otra cosa.

**No es el instrumento:** el primer intento del script compartía el vector del
testigo con el del sujeto, y el needle del vector marcaba el testigo como fuga;
corregido antes de medir (testigo con vector propio). El testigo sobrevive en los
tres modos y también tras compactar, así que el «0 archivos» no es una base vacía.

## Parte 2 — churn HNSW (degradación y QPS)

Borrar + reinsertar el mismo volumen (corpus constante: es churn, no encogimiento),
`forget(mode="hard")` sobre claves, 20 consultas, `top_k=10`:

| borrado/reinsertado | corpus | p50 (ms) | p99 (ms) | QPS | resultados no vacíos |
|---|---:|---:|---:|---:|---:|
| 10 % | 1 000 | 0.972 | 1.239 | **1 004.89** | 1.0 |
| 30 % | 1 000 | 1.425 | 1.673 | **733.59** | 1.0 |
| 50 % | 1 000 | 1.488 | 1.676 | **669.21** | 1.0 |

La degradación por churn es **moderada y monótona**: p50 ×1.53 y QPS ×0.67 de 10 %
a 50 % de churn, sin resultados vacíos en ningún nivel. Ese es el número que la
fila pedía como «degradación y recuperación»; la recuperación tras una corrida de
compactación no se midió (fuera del alcance de este script).

## Contrato, verificación y lo que falta

- **Aserción como aserción:** el script devuelve **exit 1** cuando la recuperación
  de `certified` no es 0. Hoy devuelve 1. Un benchmark que siempre sale verde no
  mide nada.
- **Reproducible:** `python benchmarks/certified_unlearning_bench.py --docs 1000
  --json evals/results/certified_unlearning_bench.json` (0.7 min medido).
- **Requiere:** solo `vantadb` (PyO3) — cero dependencias nuevas, cero compilación.
- **Artefacto:** `evals/results/certified_unlearning_bench.json`.

## Hallazgo derivado (registrado, no diferido)

`FIND-826`: `forget(mode="certified")` **no** alcanza el 0 % de recuperación por sí
solo; el borrado byte-level requiere `compact_wal()` + `compact_layout()`, y el
recibo criptográfico no lo declara. Arreglo en `src/sdk/api/memory.rs` (zona de otro
carril): o el `certified` ejecuta la compactación y lo dice en el recibo, o el
contrato/documentación declara que la erasure necesita el paso explícito.

## Impacto mapeado (Regla 0)

- **Leídos:** `vantadb-python` superficie de `Client` (`forget`, `compact_wal`,
  `compact_layout`, `delete_node`), el artefacto de soak para el patrón de
  generación determinista de vectores.
- **Referencias hacia dentro:** ninguna — archivo nuevo bajo `benchmarks/`.
- **Veredicto:** sólo se añade el benchmark. **Cero cambios en `src/**`.**

## Review (P2-01)

Tier: **Fast**. Verify: corrida real (tabla arriba), `ruff check` +
`ruff format --check` verdes, aserción probada en la dirección que importa (FAIL
con exit 1). **approve**.
