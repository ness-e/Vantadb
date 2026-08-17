---
title: "Avance — Investigaciones (INV)"
type: catalog
status: active
tags: [vantadb, avance, investigacion, research]
last_reviewed: 2026-08-07
aliases: []
---

# Avance — Investigaciones (INV)

> Catálogo de investigaciones completadas. Los reportes viven en `docs/Investigaciones/`. Cero cambios de código en la mayoría (diseño/auditoría), a menos que se indique.

## Fase 1 — Seguridad y críticos

### INV-001: RUSTSEC Advisory Audit ✅ (2026-07-29)
- **Fuente:** Backlog (Investigaciones de Seguridad) `INV-001`
- **Resultado:** Auditoría de advisories RUSTSEC. Hallazgo: bincode 1.x→2.0 ya migrado (AUD-03), rustls-pemfile ya en v2. Duplicado cerrado: `AUDREP-51` (mismo advisory RUSTSEC-2023-0089).
- **Ids:** `INV-001`

### INV-024: Unsafe Blocks Audit ✅ (2026-07-30)
- **Fuente:** Backlog (Phase 1 — Security & Critical) `INV-024`
- **Resultado:** Auditoría de bloques `unsafe`. 7 hallazgos UB cubiertos por AUDIT-03 (Miri). Reporte: `docs/Investigaciones/` (vía ARCHIVO_HISTORICO §INV-024).
- **Ids:** `INV-024`

## Fase 4 — Engineering Health

### INV-002: Memory Telemetry Correction ✅ (2026-07-30)
- **Fuente:** Backlog (Phase 4 — Engineering Health) `INV-002`
- **Ids:** `INV-002`

## Investigaciones de diseño (julio 2026)

### INV-006: Blog series completion — plan de finalización
- **Resultado:** ✅ Plan para la serie de blogs; cubre MKT-10 ("AI Agent Memory" campaign con DoD medibles).
- **Ids:** `INV-006`

### INV-007: Competitive benchmark vs LanceDB/Chroma — diseño
- **Resultado:** ✅ `docs/Investigaciones/INV-007-competitive-benchmark-lancedb-chroma.md` (19.8KB). Veredicto: NO publicar en `ann-benchmarks` (repo sin mantenimiento, recomienda VIBE) — usar solo datasets HDF5 + metodología Recall-QPS. Harness standalone Python (`benchmarks/competitive/run_competitive_benchmark.py`) con glove-100-angular + sift-128-euclidean. Slicing vertical 3 slices (harness+JSON → tabla web → CI). 10 fuentes citadas. Cero cambios de código.
- **Ids:** `INV-007` (+ `INV-007-B` ✅ implementado 2026-08-05, commit `58061ab8` — competitive_benchmark.json + competitive-table web)

### INV-008: Batch Queries Python SDK — diseño
- **Resultado:** ✅ `docs/Investigaciones/INV-008-batch-queries-python-sdk.md` (10.9KB). Gate: `search_batch(vectors, top_k)` YA existía. Gap real: no acepta SearchRequest completo. Propuesta `search_batch_requests()`. Veredicto YAGNI: método nuevo en binding, wrapper Python puro descartado. Cero cambios de código.
- **Ids:** `INV-008` (+ `INV-008-B` ✅ implementado 2026-08-05, commit `90fd3532`)

### INV-009: Phrase Queries + Term Positions — diseño
- **Resultado:** ✅ `docs/Investigaciones/INV-009-phrase-queries-term-positions.md` (13.8KB). Gate: infrastructure phrase-ready YA existía (`TextQueryPlan.phrases`, `token_positions`, `text_positions_match_phrase` + 12 tests). Gaps: sintaxis IQL, enforcement, highlight. Veredicto tantivy: CUSTOM (YAGNI) — duplicaría índice, ~40 crates. Cero cambios de código.
- **Ids:** `INV-009` (+ `INV-009-B` ✅ implementado 2026-08-05, commit `995258e9` — `Condition::TextMatch` + highlight contiguo)

### INV-010: ACID rollback multi-capa completo — diseño
- **Resultado:** ✅ Diseño de rollback multi-capa. Ver core-engine.md (ACID Fase 2-3).
- **Ids:** `INV-010`

### INV-011: Core-Server Separation — auditoría
- **Resultado:** ✅ **Separación YA limpia — sin cambios requeridos.** Server deps todas optional detrás de features; verificado mecánicamente con `cargo tree` y `cargo check`. Observación menor: `server = ["cli",...]` acopla server→cli (intencional). Doc: `docs/Investigaciones/INV-011-core-server-separation.md`. Cero cambios de código.
- **Ids:** `INV-011`

### INV-012: Anti-Locality Disk Layout — re-evaluación
- **Resultado:** ✅ **WONTFIX CONFIRMADO — NO re-abrir.** Re-run `benches/vfile_search.rs`: mejora locativa ~7.0% (614.5ms vs 571.5ms compacted), inferior al 15% requerido. LSM/multi-level NO alteraron el resultado. Re-apertura hipotética requiere dataset 1M+ cold-cache. Doc: `docs/Investigaciones/INV-012-antilocality-reevaluation.md`. Cero cambios de código.
- **Ids:** `INV-012`

### INV-013: JSON-LD structured data — auditoría
- **Resultado:** ✅ Doc: `docs/Investigaciones/INV-013-jsonld-structured-data.md`. **JSON-LD AUSENTE** — Next.js 16 Metadata API NO genera JSON-LD; propuesta schema.org/SoftwareApplication emitido manualmente. Cero cambios de código.
- **Nota 2026-08-04:** la entrada anterior afirmaba JSON-LD implementado (WEB-13) — **FALSA**: WEB-13 fue Pages Router que ya no existe; los commits citados son de OG/canonical. JSON-LD sigue pendiente en Backlog.
- **Ids:** `INV-013` (+ `INV-013-B` ✅ implementado 2026-08-05, commit `1d072f4a`)

### INV-014: Light mode (CSS muerto) — auditoría
- **Resultado:** ✅ Doc: `docs/Investigaciones/INV-014-light-mode-css.md`. Premisa invertida — **NO existe CSS light muerto; el sitio es LIGHT-ONLY por diseño**. Recomendación: eliminar plomería DARK inerte (`theme-provider` + `theme-toggle` + dep next-themes, YAGNI). NO reactivar dark mode.
- **Ids:** `INV-014` (+ `INV-014-B` ✅ implementado 2026-08-05, commit `6e7b91b8`)

### INV-015: Touch targets < 44px — auditoría
- **Resultado:** ✅ Doc: `docs/Investigaciones/INV-015-touch-targets-44px.md`. ~23 componentes no cumplen 44×44 (2 icon buttons 14px → severo). Inventario priorizado P0-P4. Cero cambios de código.
- **Ids:** `INV-015` (+ `INV-015-B` ✅ implementado 2026-08-05, commit `532788d2`)

### INV-016: Motion-duration tokens — auditoría
- **Resultado:** ✅ Doc: `docs/Investigaciones/INV-016-motion-duration-tokens.md`. **NO existen tokens de duración/easing.** Propuesta CSS vars + mapa JS `MOTION`. Cero cambios de código.
- **Ids:** `INV-016` (+ `INV-016-B` ✅ implementado 2026-08-05, commit `6afb37c3`)

### INV-017: sccache en CI — investigación (2026-08-02)
- **Resultado:** ✅ `docs/Investigaciones/INV-017-sccache-ci.md`. Hallazgo clave: `.opencode/AGENTS.md` afirmaba falsamente sccache implementado (drift, 0 matches en `.github/`); corregido. Diseño: `mozilla-actions/sccache-action@v0.0.11`. **Implementado** vía GH-143 (commits `44404c7d`, `1f9f5c41`): Windows tests 14m29s → 8m35s (−40.7%).
- **Ids:** `INV-017`

### INV-019: Advanced Tokenizer (Unicode + Stopwords)
- **Resultado:** ✅.
- **Ids:** `INV-019`

### INV-025: Scoping Search Quality v2
- **Resultado:** ✅ `SEARCH_QUALITY_V2_SCOPING.md`, contrato INV-009-B (2026-08-05, commit `023d6e89`).
- **Ids:** `INV-025`

## Investigaciones de web/UX (5 agosto 2026)

- **INV-005-A:** error.tsx App Router + drop dep muerta @mdxeditor/editor ✅ commit `6d0b84ec`
- **INV-013-B:** JSON-LD schema.org/SoftwareApplication en layout root ✅ commit `1d072f4a`
- **INV-014-B:** Eliminar plomería dark inerte ✅ commit `6e7b91b8`
- **INV-015-B:** Touch targets 44px + iconos X h-5 ✅ commit `532788d2`
- **INV-016-B:** Motion tokens duration/ease ✅ commit `6afb37c3`

## Índice de reportes

| Reporte | Ubicación |
|---|---|
| INV-007 | `docs/Investigaciones/INV-007-competitive-benchmark-lancedb-chroma.md` |
| INV-008 | `docs/Investigaciones/INV-008-batch-queries-python-sdk.md` |
| INV-009 | `docs/Investigaciones/INV-009-phrase-queries-term-positions.md` |
| INV-011 | `docs/Investigaciones/INV-011-core-server-separation.md` |
| INV-012 | `docs/Investigaciones/INV-012-antilocality-reevaluation.md` |
| INV-013 | `docs/Investigaciones/INV-013-jsonld-structured-data.md` |
| INV-014 | `docs/Investigaciones/INV-014-light-mode-css.md` |
| INV-015 | `docs/Investigaciones/INV-015-touch-targets-44px.md` |
| INV-016 | `docs/Investigaciones/INV-016-motion-duration-tokens.md` |
| INV-017 | `docs/Investigaciones/INV-017-sccache-ci.md` |
| AUDIT-02 | `docs/Investigaciones/AUDIT-02-2026-08-06.md` |

## P10 — Competitive features catalog (investigado + decidido)

`COMP-001/002/003/004/005/007/011/015/020/030` catalogados en `historial/backlog-history.md` → P10. Incluyen SQ8/PQ, HNSW persist, in-filter, bitset, params, inline u128, CRUD tombstones, hybrid pipeline, RRF fusion, survival mode.

## Investigaciones de ingeniería de agentes (agosto 2026)

### TIR-03: Mitigación/contención primero en incidentes - decisión (2026-08-12)
- **Fuente:** Backlog P18 `TIR-03` (gap-01 FALTA#15, REPORTE-FINAL §3.3-15)
- **Resultado:** Decisión IMPLEMENTAR docs mínimos — nueva Fase 0.5 Contención/Estabilización en `docs/references/bug-workflow.md:18` (revert/pausar + registrar ANTES del debug; no reemplaza el Iron Law). Doc: `docs/Investigaciones/2026-08-10-agent-engineering/TIR-03-decision.md`. Review P2-01 vanta-review ✅ approve.
- **Ids:** `TIR-03`

### TSYS-06: Chaos/resilience del task-system — decisión runner DEFER (2026-08-16)
- **Fuente:** Backlog P17 `TSYS-06` + P18 `TIR-07` (misma brecha)
- **Resultado:** Decisión **DEFERIR el chaos runner** — tests de inyección de fallos puntuales cubren el riesgo real con fracción del costo. Doc: `docs/Investigaciones/TSYS-06-chaos-runner.md`. Resuelve también TIR-07.
- **Ids:** `TSYS-06`

### FND-24: JTBD/ICP — hipótesis + plan de validación (2026-08-16)
- **Fuente:** Backlog P20d `FND-24`
- **Resultado:** **0 evidencia de usuarios reales — todo hipótesis** (4 perfiles ICP, 10 JTBD) + plan de validación accionable post-Show HN. Regla de la tarea: no inventar evidencia. Doc: `docs/Investigaciones/FND-24-icp-jtbd.md`.
- **Ids:** `FND-24`