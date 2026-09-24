# GOV-TK8 — Benchmarks docs + evidencia

> **Plan:** `docs/dev/plans/2026-09-10-code.md` Task 15 · **Rama:** `develop` · **Owner:** E1 (vanta-docs)
> **Appetite:** max 1d · **Estado:** ⏳ IN PROGRESS
> **Contrato:** benches corridos o documentados + tablas + comandos reproducibles + Regla 11 (sin claims sin fuente)
> **Stop condition:** bench no corre → documentar evidencia cruda + DEFER números.
> **SDP:** documentation-and-adrs, writing-guidelines, writing-plans, performance-optimization (SDP v2 keywords: benchmarks/tablas/reproducibles/Regla 11/evidencia; lifecycle BUILD aportó campaign-executor/incremental/test-driven/context/source-driven/doubt/api-design — solo se cargan las 4 de dominio; resto base-only registrado)

## DISCOVERY (zero-code planning)

- `_run_stdout.md` (121L, 2026-08-12): evidencia cruda real — synthetic 2K/50q/euclidean, Vanta/Lance/Chroma/Qdrant, health-check con warnings (disco 5.6%, CPU 65.8%, 15 VS Code) + 2 tracebacks Chroma WinError 32. ✅ Existe.
- `competitive_sdk_bench.json` (generado 2026-08-12 13:23:52 por el harness): 5 motores (incl. Milvus merge de `competitive_sdk_bench_milvus.json`), methodology mediana ×3, versiones pinned. ✅ Fuente machine-readable.
- `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` (57L, 2026-08-12): ya cura el run — tabla 5 motores, comando, caveats (Chroma 1 iter, sintético no extrapolable, adaptación Milvus IndexParams). ✅ Curado existe.
- `docs/user/operations/BENCHMARKS.md` (§1–§17, 951L): §7 solo tabla glove 2026-06-06 (dataset distinto, no comparable); **sin puntero al run 2026-08-12**. ← GAP que cierra esta tarea.
- `docs/benchmarks/COMPETITIVE_ANALYSIS.md` (2026-07-31): histórico + post-mortem B2/batch-size; tablas fechadas, comparabilidad advertida (§6 banner pre-Jul-31). Sin cambios necesarios.
- Decisión: NO re-correr benches (stop-condition path) — run pesado (compete + datasets 1 GB), entorno contaminado documentado en el propio raw, y números ya curados con metodología. Entregable = §18 en BENCHMARKS.md que fecha, enlaza evidencia cruda + JSON + curado, tabla, comando reproducible, entorno y caveats (Regla 11). Números "viejos" quedan fechados, no borrados (pre-mortem).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `docs/benchmarks/_run_stdout.md` (121L), `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` (57L), `docs/benchmarks/COMPETITIVE_ANALYSIS.md` (§1–§13 muestreados), `docs/user/operations/BENCHMARKS.md` (1–951, leído §1–§17 + tail 838–951), `docs/benchmarks/competitive_sdk_bench.json` (vía python), `docs/benchmarks/ivf_bench.md` (73L).
- **Referencias hacia dentro:** §18 enlaza a `docs/benchmarks/_run_stdout.md`, `docs/benchmarks/COMPETITIVE_SDK_BENCH.md`, `docs/benchmarks/competitive_sdk_bench.json`, `benchmarks/competitive_bench.py`; §7 (glove 06-06) referenciado como no-comparable.
- **Referencias entrantes:** `BENCHMARKS.md` es referenciado por Regla 9/11 (AGENTS.md), MEM-70 (§17), `benchmarks/README.md` (verificar, no roto por append de sección).
- **Veredicto:** append-only de una sección al final (§18) + este task file. Sin cambios a tablas existentes, sin código, sin renombres. Blast radius = 2 archivos propios.

## Steps

### Step 1 — DISCOVERY + task file ✅
- Crear este archivo con impacto Regla 0. Verificado: rama `develop`, evidencia leída.

### Step 2 — Auditoría Regla 11 ✅
- Grep adjetivos/claims en `docs/benchmarks/*.md`; verificar cada número con fuente (bench + comando + entorno).
- Resultado: COMPETITIVE_SDK_BENCH.md L33 "rápido/lento" respaldado por tabla misma página; COMPETITIVE_ANALYSIS.md L79 "~1.4-2.2x" deriva de §1; L150 "2.5x rebuild" de historial Fase 2. 0 claims sin fuente nueva. No se edita.
- Post-cambio BENCHMARKS.md: grep superlativos sin fuente = 0 hits (2026-09-10).

### Step 3 — §18 en BENCHMARKS.md ✅
- Append §18: run sintético 2026-08-12 (5 motores), tabla, comando, entorno, caveats (Chroma 1 iter, dataset 2K no extrapolable, health warnings del raw, Milvus adaptación IndexParams), links a raw + JSON + curado, nota de no-comparabilidad con §7.

### Step 4 — Verify + commit ✅
- `Select-String` R11 post-cambio + `scripts/validate-docs-coverage.ps1` (si aplica a docs) + `git status` (solo propios) + commit `docs: GOV-TK8 — ...` en develop + recitation completed.
