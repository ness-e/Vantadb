# Task INV-007-B — JSON contrato + competitive-table.tsx (absorbe MKT-17)

- **Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 47
- **Estado:** ✅ COMPLETED 2026-08-05
- **MKT-17:** no existe task file propio (`.opencode/skills/campaign-executor/tasks/MKT-17.md`); ya tachada en Backlog (Task 4 del plan) — se cierra aquí por absorción.

## Qué se hizo

1. **JSON versionado** — `web/src/lib/data/competitive-benchmark.json` (schema_version 1):
   fecha/hardware/versiones/dataset/metodología + 3 filas de resultados (VantaDB, LanceDB, ChromaDB).
   **Fuente (sin números inventados):** run real del harness 2026-08-04 publicado en
   `docs/blog/benchmarks_vs_lancedb_chroma.md` (MKT-05, glove-100-angular 10K, 100q, top_k=10,
   cosine, `--batch-size 999`). El run histórico de `docs/operations/BENCHMARKS.md` §7 (2026-06-06)
   está marcado NO-comparable por MKT-15.md (doble rebuild) → no usado.
   - Hardware/versiones del run 2026-08-04 no registradas → `null` + nota honesta; el harness las
     rellena automáticamente en runs futuros.

2. **Script** — `benchmarks/competitive_bench.py`:
   - `--json-output` (default `web/src/lib/data/competitive-benchmark.json`)
   - `write_json_report()` + `detect_hardware()` + `detect_versions()` + `_pkg_version()`
   - `index_time_ms: null` cuando el motor usa índice incremental (ChromaDB), match del source.
   - Verificado: `python -m py_compile` ✅

3. **Web** — `web/src/lib/vanta-data.ts` (tipado `CompetitiveBenchmark*`, re-export del JSON, 0
   números hardcodeados) + `web/src/components/vanta/competitive-table.tsx` (tabla renderizada del
   JSON, estilo design system border-4/FF5500, disclaimer honesto + source + comando de
   regeneración) + `web/src/app/benchmarks/page.tsx` (monta `<CompetitiveTable />` bajo
   `<BenchmarkRace />`).

## Contrato

- ✅ JSON versionado y correcto (validado `json.load`, 3 results, schema 1)
- ✅ competitive-table.tsx renderiza datos del JSON (vía vanta-data.ts, no hardcode)
- ✅ `npm run build` en web/ pasa — `/benchmarks` prerenderizado estático

## Commit

`feat(INV-007-B): competitive_benchmark.json + tabla web (MKT-17)`

Staging selectivo: `benchmarks/competitive_bench.py`, `web/src/lib/data/competitive-benchmark.json`,
`web/src/lib/vanta-data.ts`, `web/src/components/vanta/competitive-table.tsx`,
`web/src/app/benchmarks/page.tsx`. Este task file NO se commitea (convención del pipeline).
