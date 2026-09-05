# NUEVO-10: Benchmark suite pública reproducible

## Metadata
- **Plan file:** ninguno activo (backlog directo)
- **Fuente:** docs/Backlog.md línea 119 — "Benchmarks internos existen, sin script público standalone" / audit backlog-validation-2026-07-28 línea 105: "⚠️ Scripts existen pero requieren build local; no standalone"
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Mixto (Benchmark infra + Docs)
- **Turns estimados:** 15-25
- **Creado:** 2026-08-02
- **Estado:** ✅ COMPLETADO

## Contexto verificado (2026-08-02)
- `vantadb_py` **0.5.0 está publicado en PyPI** (verified: `pip index versions vantadb-py` → 0.5.0, 0.4.0, 0.2.0, 0.1.5...). El path standalone público YA es posible sin build local.
- Benchmarks existen y funcionan:
  - `benchmarks/vantadb_local_bench.py` — BENCH-01 (ingestión + BM25 + HNSW + hybrid RRF), datos sintéticos, zero-dep además de `vantadb_py`. Auto-exporta `benchmarks/vanta_benchmark_report.json` con schema parity.
  - `benchmarks/competitive_bench.py` — compara VantaDB vs LanceDB vs ChromaDB, auto-descarga datasets ann-benchmarks (glove-100-angular, sift-128-euclidean) vía urllib, warmup + median-of-3 + ground truth numpy JIT. Requiere numpy/h5py/lancedb/chromadb/psutil/tabulate.
  - `benchmarks/batch_vs_sequential_bench.py`, `benchmarks/prefetch_comparison.py`, `benchmarks/wasm_bench.mjs`.
- **El gap (cita audit):** los 3 scripts Python hacen `import vantadb_py` y si falla muestran "run 'maturin develop' in vantadb-python first" (batch_vs_sequential_bench.py:16, vantadb_local_bench.py:18-22, competitive_bench.py:66-71) — es decir, fuerzan build local. No hay `requirements.txt` para el path público ni doc de reproducción standalone.
- `docs/operations/BENCHMARKS.md` sección 3 ("Reproducing the Benchmark Locally") solo documenta `maturin develop` (líneas 68-74). CI `perf-bench-40.yml` ya corre `vantadb_local_bench.py` y actualiza la tabla vía `update_markdown.py`.
- NO existe `benchmarks/requirements.txt`, `benchmarks/README.md`, ni `BENCHMARKS.md` en raíz.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/operations/BENCHMARKS.md` (sección reproduce), CI `perf-bench-40.yml`, README badge (workflow perf-bench-40) |
| Callees | `vantadb_py` (PyPI wheel), numpy/h5py/lancedb/chromadb/psutil/tabulate (pip) |
| Implicaciones | NO toca Rust core ni bindings. Scripts Python + docs. El cambio a error-message y requirements.txt NO rompe el path local (`maturin develop`) — solo agrega el path público (`pip install -r benchmarks/requirements.txt`) |

**RIESGO:** bajo. Sin cambios en `src/`, CI jobs, ni API.

## Contrato
"`pip install -r benchmarks/requirements.txt` en un venv limpio instala `vantadb-py` desde PyPI; `python benchmarks/vantadb_local_bench.py --size 1000 --queries 50 --output <tmp>.json` corre y emite JSON con insert/rebuild/query_text/query_vector/query_hybrid; `docs/operations/BENCHMARKS.md` documenta el path standalone público (pip install, sin maturin); los 3 scripts muestran hint `pip install vantadb-py` en lugar de 'maturin develop'."

## Herramientas necesarias
- Python 3.11+ + pip (venv limpio para verificar el path público)
- Python (root `.venv` con `vantadb_py` 0.5.0 ya instalado — verificación rápida)
- No cargo-mcp (no se toca Rust)

## Investigation Notes
- PyPI distribution name: `vantadb-py` (con guion); import: `vantadb_py` (con underscore).
- `competitive_bench.py` requiere datasets grandes (~1GB glove.6B.zip en scripts/download_benchmark_datasets.sh; ann-benchmarks HDF5 en competitive_bench.py) — el path standalone "público" debe documentar el dataset chico sintético por defecto y datasets reales como opción.
- `update_markdown.py` ya auto-actualiza `docs/operations/BENCHMARKS.md` (markers `BENCHMARK_METRICS_START/END`) — no necesita cambios.

## Steps

### Step 1: Crear `benchmarks/requirements.txt`
- **Archivos:** `benchmarks/requirements.txt`
- **Acción:** crear con deps del path público reproducible: `vantadb-py>=0.5.0`, `numpy`, `h5py`, `lancedb`, `chromadb`, `psutil`, `tabulate` (para competitive_bench). Comentar cuáles son obligatorias vs opcionales (local_bench solo necesita vantadb-py).
- **Verify:** `pip install -r benchmarks/requirements.txt` en venv limpio instala `vantadb_py` desde PyPI (`pip show vantadb-py` → 0.5.0)
- **Estado:** ✅ COMPLETADO

### Step 2: Corregir hints de instalación en los 3 scripts
- **Archivos:** `benchmarks/vantadb_local_bench.py` (líneas 18-22), `benchmarks/competitive_bench.py` (líneas 66-71), `benchmarks/batch_vs_sequential_bench.py` (líneas 14-17)
- **Acción:** reemplazar el hint "run 'maturin develop' in vantadb-python first" por "pip install vantadb-py" (path público PyPI) manteniendo la mención a `maturin develop` solo como alternativa para devs locales (una línea).
- **Verify:** grep confirma que los 3 scripts mencionan `pip install vantadb-py` y ya no son el único hint el maturin
- **Estado:** ✅ COMPLETADO

### Step 3: Crear `benchmarks/README.md` (guía pública de reproducción)
- **Archivos:** `benchmarks/README.md`
- **Acción:** documentar (a) path rápido standalone: venv → `pip install -r requirements.txt` → `python benchmarks/vantadb_local_bench.py --size 10000 --queries 1000 --output report.json`; (b) competitive: mismo + datasets ann-benchmarks; (c) variante dev local con `maturin develop`; (d) referencia a resultados publicados en `docs/operations/BENCHMARKS.md` y badge CI.
- **Verify:** markdownlint pasa (`npx markdownlint-cli2 benchmarks/README.md` o reglas del repo)
- **Estado:** ✅ COMPLETADO

### Step 4: Actualizar `docs/operations/BENCHMARKS.md` sección 3
- **Archivos:** `docs/operations/BENCHMARKS.md` (sección "Reproducing the Benchmark Locally", líneas 64-78)
- **Acción:** agregar el path standalone público (pip install desde PyPI) ANTES del path dev local (maturin). Referenciar `benchmarks/README.md`. Mantener la tabla de métricas generada por CI intacta.
- **Verify:** `scripts/validate-docs-coverage.ps1` pasa; grep confirma que la sección 3 documenta `pip install`
- **Estado:** ✅ COMPLETADO

### Step 5: Smoke test end-to-end path público
- **Archivos:** `benchmarks/vanta_benchmark_report.json` (generado)
- **Acción:** en el venv limpio del Step 1 (o venv aislado temporal), correr `python benchmarks/vantadb_local_bench.py --size 1000 --queries 50 --output <tmp>/report.json` y validar que el JSON contiene las 5 claves con valores > 0. (Usar size/querys chico para rapidez — no es benchmark de medición, es smoke test de reproducibilidad.)
- **Verify:** JSON válido con `insert/rebuild/query_text/query_vector/query_hybrid` no vacíos; exit code 0
- **Estado:** ✅ COMPLETADO

## Dependencias
- Ninguna (task independiente)

## Notas
- El audit 2026-07-28 marcó NUEVO-10 como ⚠️ "requieren build local; no standalone". Con `vantadb_py` 0.5.0 publicado en PyPI, el fix es documental + requirements.txt — NO requiere code changes en Rust ni en los algoritmos de benchmark.
- No renombrar los scripts existentes — solo agregar el path público.
- `perf-bench-40.yml` y `update_markdown.py` ya cubren CI — no tocar.

## Context Save Point
- **Fecha:** 2026-08-02
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** path público = `pip install -r benchmarks/requirements.txt` + hint en scripts; dev local (maturin) se mantiene como alternativa. No crear BENCHMARKS.md en raíz — `docs/operations/BENCHMARKS.md` es la fuente canónica.
- **Problemas conocidos:** competitive_bench requiere datasets grandes (descarga lenta) — el smoke test usa local_bench sintético.
- **Próxima tarea:** — (una sola task)

## Verification Log (2026-08-02)
- ✅ `pip install -r benchmarks/requirements.txt` en venv limpio (Python 3.11.9) → `vantadb-py 0.5.0` desde PyPI, exit 0.
- ✅ Smoke test: `python benchmarks/vantadb_local_bench.py --size 1000 --queries 50 --output <tmp>/report.json` → exit 0, JSON válido con las 5 claves (`insert`/`rebuild`/`query_text`/`query_vector`/`query_hybrid`) no vacías, valores > 0.
- ✅ grep: los 3 scripts Python mencionan `pip install vantadb-py` (vantadb_local_bench.py:21, batch_vs_sequential_bench.py:18, competitive_bench.py:71).
- ✅ `docs/operations/BENCHMARKS.md` sección 3 documenta el path pip install (3a standalone) antes del path maturin (3b).
- ⚠️ `scripts/validate-docs-coverage.ps1` → exit 1 por causas preexistentes NO relacionadas: error interno del script (`src\sdk\search.rs` no existe) + gaps preexistentes en `config.rs`, `error.rs`, `cli.rs`, `vantadb-python` (métodos SDK). Ningún gap toca benchmarks.
- ✅ Commit: `3083b561` (amend) `feat: public reproducible benchmark suite (NUEVO-10)` — 7 archivos, solo archivos de NUEVO-10.
