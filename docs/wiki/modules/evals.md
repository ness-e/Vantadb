---
title: "Módulo: evals (evaluación, benches, fuzz, tests)"
kind: concept
status: active
description: "Harnesses de evaluación de memoria, benchmarks, fuzzing y suites de test registradas: qué existe y qué no se afirma de ellos."
---

# Módulo: evals (evaluación, benches, fuzz, tests)

Responsabilidad: medición de calidad/rendimiento y ejercicio de fallos. **Nada de esta página afirma resultados**: solo inventario de harnesses.

## evals/

`[HECHO]` Contenido: `eval_locomo.py`, `eval_longmemeval.py` (+`data/fetch_longmemeval_subset.py`, `data/precompute_embeddings.py`, `longmemeval_s_subset.json`), `bench_sequential_niah.py`, `soak_test.py`, `memory_bench.py`, `memory_harness.py`, `calibration.py`, `agent/{golden_cases.json,run_eval.py}`, `runners/{adapters.py,head_to_head.py}` y READMEs por carpeta. `[INFERENCIA]` Son harnesses de evaluación de memoria de agente y resistencia operacional (locomo/longmemeval/sequential-niah/soak/head-to-head); esta página no evalúa procedencia de datasets, leakage ni ground truth — eso requiere lectura de cada harness.

## Benches y benchmarks

`[HECHO]` [`Cargo.toml`](../../../Cargo.toml) registra los 24 benches con `harness = false` (criterion): `tokenizer_bench`, `hybrid_queries`, `high_density`, `stress_test`, `hnsw_pure`, `bench_concurrent`, `ingestion_concurrent`, `backend_compare`, `hnsw_recall_ef`, `vfile_search`, `incremental_bench`, `batch_existing_check`, `batch_txn_overhead`, `param_sweep`, `acorn_filtered_search`, `sparse_hot_path`, `ivf_bench`, `canonical_p99`, `encrypted_p99`, `purge_expired`, `memory_budget`, `wal_throughput`, `crash_recovery`, `scores_semantics`. `benches/` y `benchmarks/` existen como directorios; los claims publicados se limitan a `docs/user/operations/BENCHMARKS.md` con entorno declarado.

## Fuzz y tests

`[HECHO]` `fuzz/` es harness cargo-fuzz fuera del workspace (nightly+Linux, `Cargo.toml:793-795`); proptest en `tests/fuzz_proptest.rs`. `[HECHO]` Las suites `[[test]]` registradas en `Cargo.toml` cubren durabilidad, paridad, seguridad, migraciones y certificación — inventario agrupado en [developer/verification](../developer/verification.md) y [TEST_MAP](../../dev/operations/TEST_MAP.md).

## Fuentes

`evals/` · `Cargo.toml` · `fuzz/` · `tests/` · [BENCHMARKS](../../user/operations/BENCHMARKS.md) · [FUZZING](../../dev/operations/FUZZING.md) · [chaos-testing](../../dev/operations/chaos-testing.md)
