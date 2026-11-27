---
title: "FIND-667: eval_locomo.py soporta el esquema oficial de snap-research/LoCoMo y falla sin embeddings"
kind: task
status: completed
description: "Adaptador canónico del locomo10.json oficial (1986 QA, 5 categorías, 0 KeyError), filtro locomo-fixed medido (1986 -> 1538) y --strict-embeddings que falla en vez de degradar."
id: FIND-667
type: bug
---

# FIND-667 — `evals/eval_locomo.py` no soporta el esquema oficial de LoCoMo

- **Prioridad:** 🟠 Alta · **Origen:** auditoría 59 benchmarks 2026-11-22 (#55) + AMB 2026
- **Creado:** 2026-11-22 · **Cerrado:** 2026-11-27
- **Estado:** ✅ RESUELTO

## Lo que estaba roto (medido contra el archivo upstream)

`locomo10.json` de `snap-research/LoCoMo`, descargado el 2026-11-27 (2 740 KB):

```
type list len 10
item keys:  ['conversation', 'event_summary', 'observation', 'qa', 'sample_id', 'session_summary']
turn shape: {'speaker': 'Caroline', 'dia_id': 'D1:1', 'text': 'Hey Mel! ...'}
qa keys:    ['answer', 'category', 'evidence', 'question']
categories: {1: 282, 2: 321, 3: 96, 4: 841, 5: 446}   total 1986
```

El loader esperaba `question_id` / `haystack_sessions` / `answer_session_ids`, que
solo produce `generate_synthetic_locomo`. Sobre ese archivo eso es **`KeyError`**.

**Segundo nivel, que la fila no mencionaba.** El propio bloque `qa` tiene formas
heterogéneas, medidas con `collections.Counter` sobre las 1986 preguntas:

| Forma de `answer` | n | Consecuencia si se asume `str` |
|---|---:|---|
| `str` | 1 536 | — |
| **clave `answer` ausente** | **444** (todas adversariales) | `item["answer"]` → `KeyError` |
| **`int`** (`2022`, `2`, `3`) | **6** | `gold.lower()` → `AttributeError` |

`question`, `evidence` y `category` están siempre presentes (0 ausencias). Ese
`AttributeError` es el que apareció al ejecutar la primera corrida real
(`compute_f1_overlap` en la línea 867 del archivo original), así que el arreglo
tuvo que cubrir las dos formas: normalizar en el adaptador **y** coercionar en la
frontera de la métrica.

## Contrato y lo que se cambió

1. **Adaptador canónico** `adapt_official_locomo(raw)` — el único punto que conoce
   los dos esquemas. Las sesiones se descubren por **clave de contenido**
   (`session_N` cuyo valor es una lista), no por `session_N_date_time`: en la
   muestra 0 hay contenido hasta `session_19` y fechas hasta `session_35`, así que
   guiarse por las fechas ingestaría sesiones vacías.
2. **`dia_id` → sesión.** `evidence: ["D8:6","D9:17"]` → `["session_8","session_9"]`.
   Un `dia_id` que apunta a una sesión inexistente se **descarta y se cuenta**
   (`--locomo-fixed`), nunca se deja como falso negativo silencioso.
3. **Filtro `locomo-fixed`** (estándar AMB 2026), con motivos contados por separado:
   `no_evidence`, `unresolvable`, `no_answer`.
4. **`--strict-embeddings`**: sin modelo (o si falla un `query_embed`) **falla**;
   tampoco cae a `FakeLocomoStore`.
5. **`query_embed()` BGE** para las queries (`embed_query_one`, espejo de
   `eval_longmemeval.py:294-312`): los documentos van con `embed()` (pasaje) y las
   queries con el prefijo de instrucción que el modelo espera. Usar `embed()` para
   ambas empobrece el recall de un benchmark cuyo único claim **es** el recall.
6. **Tabla honesta.** Las 5 categorías aparecen (antes `adversarial` no existía en
   la tabla), cada fila lleva su **`n`**, y una categoría sin datos dice
   `(sin datos)` con `Δ —` en vez de un delta inventado contra cero. El RSS no
   medido dice `n/d` (antes imprimía `350.0x menor consumo` calculado sobre `0.0`).

## Medición

```
$ python evals/eval_locomo.py --dataset-path locomo10.json --self-test
Self-test: OK (todas las aserciones de métricas y flujo validadas)          exit 0

$ # 0 KeyError con el archivo oficial, 5 categorías parseadas
detected: locomo-official
adapted items: 1986
categories: {'adversarial': 446, 'multi_hop': 282, 'open_domain': 96,
             'single_hop': 841, 'temporal_reasoning': 321}
samples: 10
sessions per item (min/max): 19 32
items with 0 answer_session_ids: 4
KeyError count: 0 (no exception raised)

$ --locomo-fixed
1986 → 1538 QA (excluidas: {'no_evidence': 4, 'unresolvable': 0, 'no_answer': 444})
   444 de las 446 adversariales del upstream no traen `answer`, así que las
   descarta la regla de respuesta, no la categoría. Sobreviven 2 adversariales.

$ --strict-embeddings sin modelo (fastembed shadowed por un módulo que lanza ImportError)
[eval_locomo] Error: --strict-embeddings y el modelo BAAI/bge-small-en-v1.5 no
está disponible. Instalá fastembed ... o quitá la bandera ...              exit 1
$ mismo caso SIN la bandera
Reporte JSON exportado exitosamente                                        exit 0
```

### Tabla real de §23.4 (BGE `bge-small-en-v1.5` 384d, `--strict-embeddings`, `top_k=10`)

Corridas por categoría sobre el `locomo-fixed` (no hay corrida completa: 1538
preguntas × ingestión del diálogo completo por namespace ≈ 3 h en esta máquina).
Artefactos: `evals/results/locomo_official_fixed_<categoria>.json`.

| Categoría LoCoMo | n | Recall@5 | Recall@10 | MRR | NDCG@5 | F1 | p50 | p95 | QPS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Single-Hop | 100 | **0.9400** | 0.9600 | 0.7432 | 0.7898 | 1.79 | 67.02 ms | 146.86 ms | 11.83 |
| Multi-Hop | 100 | **0.5502** | 0.7583 | 0.6199 | 0.4893 | 0.92 | 55.63 ms | 81.71 ms | 16.42 |
| Temporal Reasoning | 100 | **0.8500** | 0.9600 | 0.6662 | 0.7010 | 0.39 | 75.71 ms | 164.71 ms | 10.56 |
| Open-Domain | 92 | **0.5385** | 0.6836 | 0.5116 | 0.4607 | 1.24 | 89.22 ms | 275.00 ms | 7.41 |

Mem0 publicado (arXiv:2504.19413, F1 en %): single-hop 38.72 / Rec@5 0.820;
multi-hop 28.64 / 0.650; temporal 24.18 / 0.580; open-domain 31.45 / 0.710.
**El F1 de VantaDB no es comparable**: aquí es overlap léxico entre la respuesta
gold y el chunk recuperado (sin LLM generador, `llm_tokens_consumed: 0`), no F1
de respuesta generada. Los números comparables son Recall@k, MRR y NDCG.

Lectura: single-hop y temporal razonan bien sobre el diálogo; multi-hop y
open-domain son los flancos (0.55 y 0.54 Rec@5) — coherente con que el harness no
hace descomposición de consultas.

## Verificación del contrato

- **"Con `locomo10.json` oficial: 0 `KeyError` y las 5 categorías parseadas"** →
  medido arriba: 1986 ítems, 5 categorías, ninguna excepción.
- **"Con `--strict-embeddings` y sin modelo: falla en vez de caer a sintético"** →
  exit 1 con el modelo nombrado; sin la bandera, exit 0 y el JSON exportado.

## Impacto mapeado (Regla 0)

- **Leído completo:** `evals/eval_locomo.py` (1258 líneas), `evals/eval_longmemeval.py`
  §`embed_query_one` (referencia del patrón), `evals/results/…` (forma del reporte),
  y el `locomo10.json` upstream **como dato** (esquema medido, no supuesto).
- **Referencias hacia dentro:** `format_markdown_table` (auto-test), `run_self_test`
  (añadidas las aserciones del adaptador), `docs/user/operations/BENCHMARKS.md`
  §23.4 (consumidor de la tabla).
- **Referencias entrantes:** `evals/README.md` (comandos), `BENCHMARKS.md:2546` (la
  fila citada por el hallazgo).
- **Veredicto:** cambio confinado a `evals/eval_locomo.py` + artefactos de
  `evals/results/`. Cero cambios en `src/**`.

## Deuda declarada

- **`docs/user/operations/BENCHMARKS.md` §23.4 no se actualizó**: ese archivo queda
  fuera de la zona de escritura del lote. La tabla de arriba es la que hay que
  pegar; el JSON de cada corrida está versionado en `evals/results/`.
- **No hay corrida completa** (1538 preguntas): las 4 filas son n=100/100/100/92.
  La corrida completa se hace con el mismo comando sin `--limit`; medido ≈3 h.
- **Lint:** `evals/eval_locomo.py` arrastra **85 errores `ruff` en `HEAD`**
  (`UP006`/`UP045`/`BLE001`/`S110`, todos previos). Las anotaciones que este cambio
  añade usan `dict`/`list`/`tuple` justamente para **no** sumar ninguno: el conteo
  es 85 antes y 85 después. Limpiar el archivo es un diff cosmético de ~440 líneas
  que merece su propio cambio.

## Review (P2-01)

Tier: **Fast** (`evals/**`, `docs/dev/**`). Verify: `--self-test` exit 0, 4 corridas
reales exit 0, `ruff check` con el mismo conteo que `HEAD`, `--strict-embeddings`
probado en las dos direcciones. Veredicto: **approve**.
