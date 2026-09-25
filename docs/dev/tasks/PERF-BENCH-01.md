# Task PERF-BENCH-01 — A/B vantadb-node nativo vs vantadb-ts WASM

> **Plan:** `docs/dev/plans/2026-09-07-followup-bench-a11y.md` (Task 2, Wave0)
> **Estado:** ⏳ IN PROGRESS
> **Branch:** `develop` · **Commit previsto:** `bench: node native vs wasm A/B numbers (PERF-BENCH-01)` (NO commitear — cierre sin commit por orden)
> **SDP:** ponytail(full) + performance-optimization + observability-and-instrumentation + documentation-and-adrs + source-driven-development + incremental-implementation + context-engineering (SDP v2: 8 candidatos, 7 cargadas; descartadas frontend-ui-engineering, api-and-interface-design, doubt-driven-development, test-driven-development — sin UI/API nueva/lógica security-sensitive; bench-runs son la verificación)

## Contrato (ley)

Nueva § BENCHMARKS (`docs/user/operations/BENCHMARKS.md`) con tabla insert/search p50/p99 + tamaño
binario (`.node` vs wasm pkg) + entorno + comandos ×3 corridas mediana; 0 comparativas
externas; 0 adjetivos sin número. Regla 9/11.

## Spec (decisiones por evidencia — no hay feature nueva, es medición)

| # | Decisión | Evidencia / justificación |
|---|----------|---------------------------|
| 1 | Metodología NO ambigua → sin fork vanta-research | Harnesses existen y ya corrieron (§10 native, §15 WASM ×3+mediana); fairness caveat pre-diseñado en `bench-abi.mjs:10-14`; Cynefin 🟨 pero harness listo → probe = correr, no investigar |
| 2 | Shape A/B único 2000×384d×200q, seed 42 | §15 precedente (rápido, reproducible); mismo shape ambos lados = comparación justa; canónico 100k×1536 diferido (budget wall documentado en §10) |
| 3 | ×3 corridas + mediana + rango | Pre-mortem Fallo 2 (varianza GC) + precedente §15 |
| 4 | Nueva §16 (no reescribir §10) | §10 = rama native sola + decisión condicionada; §16 = tabla lado-a-lado que la cierra; append-only, rollback-friendly |
| 5 | Solo números propios, 0 externas | D1 + Gate Justificación del plan |

## Gate D (question-gates.md) — evaluado en DISCOVERY

Blast radius: 1 archivo editado (append §16 a BENCHMARKS.md) + 2 scripts ejecutados
read-only (sin edición) + task file nuevo. Sin símbolos públicos nuevos, sin API,
contrato mecánico claro. **→ no disparado, sin `question`.**

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vantadb-node/bench/bench-abi.mjs` (241L), `vantadb-ts/bench/bench.mjs` (132L), `docs/user/operations/BENCHMARKS.md` (§10 L263-340, §15 L739-804)
- **Referencias hacia dentro (qué importa el cambio):** §16 referencia §10 (baseline native) y §15 (baseline WASM); scripts importan `vantadb-node/index.js` + `vantadb-ts/dist/vantadb.js` (prebuilt, no se tocan)
- **Referencias entrantes (quién lee §16):** ninguno (sección nueva); `scripts/validate-docs-coverage.ps1` valida cobertura docs (verificar al cierre)
- **Veredicto:** impacto LOCAL/inocuo — append docs + ejecución read-only. Fuera de scope: `vantadb-node/index.d.ts`, `src/lib.rs`, `tests/api.test.ts` (FIND-BND12-01 en paralelo — NO tocar)

## Steps atómicos

- [x] **S0 DISCOVERY** — leer harnesses + §10/§15, Gate D, SDP, env check (Node v26.8.1, dist + .node prebuilt) → `campaign_verify_cmd` N/A (docs/bench)
- [x] **S1 Smoke** — 1 corrida A/B shape chico (`--records 200 --dim 64 --searches 20`): ambas ramas verdes en Node 26, linea `JSON:` OK
- [x] **S2 Medición** — ×3 corridas A/B 2000×384d×200q (logs `$env:TEMP\opencode\bench_ab_run{A,B,C}.log`) + binarios medidos (`Get-Item ... Length`: .node 5 379 072 B, wasm 2 512 705 B)
- [x] **S3 Docs** — nueva §16 lado-a-lado p50/p99 + binarios + entorno + comandos; 0 externas; 0 adjetivos sin número
- [x] **S4 Cierre** — verify full ✅ (fmt exit 0 · clippy workspace 0 warnings · nextest 2945 passed/1 skipped · validate-docs-coverage 0 gaps vía pwsh7) + RESULTADO, SIN commit (por orden)

## Context Save Point

- S0: Node v26.8.1 ✅ · `vantadb-ts/dist/` built ✅ · `.node` 5 379 072 B (5.12 MiB — difiere de §10 4.51MB: toolchain, se reporta medido) · `vantadb_wasm_bg.wasm` 2 512 705 B (2.40 MiB — difiere de §10 1.35MB: build distinto, se reporta medido) · branch `develop` · FIND-BND12-01 modifica index.d.ts/lib.rs/api.test.ts — NO tocar
