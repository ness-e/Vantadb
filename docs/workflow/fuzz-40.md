---
title: "`fuzz-40.yml` — FUZZ: LibFuzzer — Corpus + Regression"
type: workflow
status: active
tags: [vantadb, ci, fuzz]
last_reviewed: 2026-09-15
aliases: []
related: [".github/workflows/fuzz-40.yml"]
---

# `fuzz-40.yml` — FUZZ: LibFuzzer — Corpus + Regression

## ¿Qué hace?

Ejecuta fuzzing con cargo-fuzz/LibFuzzer sobre 4 objetivos del core de VantaDB para encontrar bugs de seguridad, crashes y comportamientos inesperados mediante entradas aleatorias.

## ¿Cómo lo hace?

2 jobs secuenciales:

1. **`build`**: compila todos los targets de fuzzing con `cargo fuzz build` (toolchain nightly)
2. **`fuzz`** (matrix con 4 targets): corre cada target con `cargo fuzz run <target> -- -max_total_time=<segundos>`

Targets de fuzzing:
- `fuzz_parser` — fuzzing del parser de queries
- `fuzz_node_deserialize` — fuzzing de deserialización de nodos
- `fuzz_wal` — fuzzing del Write-Ahead Log (WAL)
- `fuzz_archive` — fuzzing del archive/compaction

El corpus de cada target se cachea entre ejecuciones para guiding eficiente.

## ¿Qué tests usa?

No usa tests tradicionales. Usa **cargo-fuzz** (LibFuzzer) que genera inputs aleatorios y monitorea crashes.

## ¿Qué verifica?

Que ningún input malformado cause:
- Pánicos (panics)
- Desbordamientos de buffer
- Violaciones de memoria
- Cuelgues infinitos (timeouts)
- Comportamientos indefinidos

## Funcionalidad final

Detección temprana de vulnerabilidades y bugs de memoria en componentes críticos (parser, WAL, serialización) mediante fuzzing continuo con corpus persistente.

## ¿Cuándo se ejecuta?

- **Semanal** (cada lunes 06:00 UTC) vía `schedule`
- **Workflow dispatch** manual con parámetro configurable de segundos por target

## CI Gate (`ci-gate`)

Antes de `build` corre el job `ci-gate` (reutiliza `.github/workflows/ci-gate.yml`). Verifica que los 11 checks requeridos del ruleset de `main` estén en verde en el commit objetivo; si alguno está en rojo, el gate falla y se salta el fuzzing (correr fuzz sobre un core que no compila/verifica es tiempo perdido). El job `fuzz` depende de `ci-gate` y `build`.

- En **schedule**: el gate se aplica.
- En **workflow_dispatch**: el gate se salta (fuerza la ejecución).

Requiere `checks: read` en el calling job.

## PR Gate (`fuzz-pr`)

Job acotado para pull requests que tocan `src/**` o `fuzz/**` (paths-filter nativo — los PRs docs-only no lo disparan). Corre los 4 targets en paralelo con `-max_total_time=75 -max_len=8192` (≈5-8 min). Ubuntu-only (Windows da flaky en fuzz); timeout ≤ 15 min; sin `continue-on-error` (Regla 2). El fuzz completo (sin límite de tiempo de PR) queda para `schedule`/`workflow_dispatch` en los jobs `build` + `fuzz`.

## Seeds y artefactos

- **Seeds:** `fuzz/corpus/<target>/` trae 1 seed mínimo por target commiteado al repo (bytes, no MB); el resto del corpus evoluciona por cache entre runs (`fuzz-corpus-<target>-*`).
- **Artefactos:** tras cada run (jobs `fuzz` y `fuzz-pr`) se sube `fuzz/artifacts/<target>/` (crashes) + `fuzz/corpus/<target>/` (corpus) vía `actions/upload-artifact` con `if: always()` — los crashes quedan disponibles aunque el job falle.
