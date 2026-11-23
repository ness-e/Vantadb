---
id: FIND-655
title: "ACT-02 gate fail medido (recall_all@5=0.1383): causa raíz en el harness de evaluación, certificado tras corrección (hybrid_rrf=0.8872)"
status: completed
type: bug
plan: docs/dev/tasks/ACT-02.md
---

# FIND-655: ACT-02 gate fail medido — causa raíz en el harness LongMemEval y certificación final (`hybrid_rrf = 0.8872`)

## Metadata
- **Fuente:** corrida oficial ACT-02 (`longmemeval_v0100_official.json`, 500q, `--strict-embeddings`)
- **Prioridad:** 🔴 P0 (gate de certificación pre-anuncio)
- **Creado:** 2026-11-21
- **Cerrado:** 2026-11-22
- **Estado:** ✅ RESUELTO — motor híbrido nativo certificado (`hybrid_rrf = 0.8872 >= 0.85`, `vector_only = 0.8830 >= 0.85`); `hybrid_graphrag` (`0.8489`) relegado a experimento documentado; trabajo futuro hacia `graphrag_search` nativo registrado en `FIND-656`
- **Evidencia:** `evals/results/longmemeval_v0100_official.json`, `evals/results/longmemeval_ablation_bm25.json`, `evals/results/longmemeval_ablation_all_fixed.json`

## Veredicto ejecutivo

El gate `recall_all@5 >= 0.85` falló medido (`0.1383`) en la primera corrida oficial de 500
preguntas por defectos en `evals/eval_longmemeval.py`, no en el motor Rust de VantaDB.
Tras corregir los defectos objetivos del harness (`150abdbba`, `0846c4a7e`, `81cbb300b`)
y ejecutar el estudio completo de 4 ablaciones sobre las 500 preguntas con embeddings
locales reales (`BAAI/bge-small-en-v1.5`, 384d, `157 926` vectores únicos), **el motor
nativo de VantaDB supera holgadamente el gate**:

- **`hybrid_rrf` (target oficial — HNSW + BM25 + RRF nativo):** `recall_all@5 = 0.8872` (**PASS**), `recall_any@5 = 0.9787`, `answer_f1 = 0.66`, búsqueda p50 `33.79 ms` (`24.9 QPS`).
- **`vector_only` (HNSW denso aislado):** `recall_all@5 = 0.8830` (**PASS**), `recall_any@5 = 0.9660`, `answer_f1 = 0.67`, búsqueda p50 `37.00 ms` (`23.3 QPS`).
- **`bm25_only` (BM25 léxico aislado, sin embeddings):** `recall_all@5 = 0.8000` (informativo), `recall_any@5 = 0.9596`, `answer_f1 = 0.66`, búsqueda p50 `5.73 ms` (`139.5 QPS`).
- **`hybrid_graphrag` (heurística Python experimental del harness):** `recall_all@5 = 0.8489` (`FAIL` por `−0.0011`), `recall_any@5 = 0.9340`, `answer_f1 = 0.52`.

## Desglose por tipo de pregunta y diagnóstico residual de `hybrid_graphrag`

| Tipo (`n` sobre 470 no-abs) | `vector_only` | `bm25_only` | `hybrid_rrf` | `hybrid_graphrag` |
|---|---:|---:|---:|---:|
| `knowledge-update` (72) | 0.9583 | 0.9444 | 0.9722 | **0.9861** |
| `multi-session` (121) | **0.8595** | 0.6033 | 0.7934 | 0.8347 |
| `single-session-assistant` (56) | **1.0000** | **1.0000** | **1.0000** | **0.6071** |
| `single-session-preference` (30) | **0.9000** | 0.7000 | **0.9000** | **0.9000** |
| `single-session-user` (64) | 0.9531 | **1.0000** | 0.9844 | 0.9688 |
| `temporal-reasoning` (127) | 0.7717 | 0.7402 | **0.8268** | 0.8189 |

- **Por qué `bm25_only` queda en `0.8000`:** techo natural léxico por paráfrasis sin solape de términos en el segundo/tercer hop de `multi-session` (`0.6033` all vs `0.9752` any) y en `single-session-preference` (`0.7000`). No es un bug; es la razón de ser del brazo denso en `hybrid_rrf`.
- **Por qué `hybrid_graphrag` pierde `−3.83 pp` frente a `hybrid_rrf`:** ayuda en `multi-session` (`+4.13 pp`) y `knowledge-update` (`+1.39 pp`), pero destruye 22/56 preguntas en `single-session-assistant` (`1.0000 -> 0.6071`). Las respuestas extensas del asistente contienen decenas de sustantivos capitalizados no presentes en la query (`cand_ents` no condicionado por `q_entities` ni IDF fino), y `inject_cap = 0.6 * max_score` (`~0.0197`) supera el score RRF de un hit legítimo de un solo brazo (`1/61 = 0.0164`), desplazando la evidencia real.

## Decisión de cierre (aprobada por owner 2026-11-22)

1. **Opción A adoptada:** certificar el gate oficial sobre `hybrid_rrf` (`0.8872 >= 0.85`, path híbrido real del motor Rust), manteniendo `hybrid_graphrag` en el arnés como ablación experimental documentada (evitando sobre-ajustar heurísticas Python contra el holdout `EVAL-02`).
2. **Opción C registrada como trabajo futuro (`FIND-656`):** evaluar la construcción de aristas tipadas en ingesta y la invocación de `db.graphrag_search` nativo del motor en lugar de expansión regex en Python.
3. **Publicación completa:** `BENCHMARKS.md` (§19 y §23), `README.md`, `README_ES.md`, `docs/user/COMPARISON.md` y `evals/README.md` actualizados con las 4 ablaciones y toda la trazabilidad histórica.
