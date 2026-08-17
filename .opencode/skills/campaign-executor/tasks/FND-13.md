# FND-13 — Benchmarks honestos (P20b, prio 🟡)

**Backlog:** docs/Backlog.md:501
**Rol:** vanta-tuner
**Wave:** 4 (P20-TSYS) — tras W3 FND-10 (Regla 9 + canonical_p99)
**Estado:** ✅ COMPLETO (wave lead commitea)

## Resultado (2026-08-16)

- Regla 11 agregada a `.opencode/AGENTS.md` (L594, tras Regla 10): claims DEBEN citar benchmark reproducible (archivo + comando + entorno) + números; adjetivos no son evidencia; fuentes gitignored no son válidas.
- Inventario completo en `docs/Investigaciones/FND-13-benchmarks-honestos.md`: 14 claims en README/bindings/BENCHMARKS/PERFORMANCE_TUNING clasificados citado/medible/sin-fuente + hallazgo BENCH01 frontend (claims fantasma ~5,400 vec/s retirados del README en PERF-01 pero vivos en web/) + inconsistencia BENCHMARKS §2 vs JSON.
- Fixes aplicados: tabla baseline README alineada a fuente citada (74.0 rec/s, p50 13.2/2.0/3.1 ms; antes 61.5/16.0/3.3/12.1), 2 links rotos a BENCHMARK_OPTIMIZATION_2026.md corregidos (README + README_ES), nota de outlier BM25 corregida (0.0035 ms).
- Deuda anotada (no tocada): BENCH01 frontend, comandos faltantes en BENCHMARKS §4/§6/§7, PERFORMANCE_TUNING T2-T12 sin fuente.

## Steps

## Objetivo

Los claims de performance en READMEs/docs DEBEN citar benchmark reproducible (archivo bench + comando) + números, no adjetivos. Revisar claims existentes, clasificarlos, aplicar fixes de 1 línea donde sean claramente falsos/obsoletos, documentar el inventario.

## Contrato (verify mecánico)

1. grep AGENTS.md → "Regla 11" referenciando benchmark reproducible
2. análisis existe en docs/Investigaciones/ con inventario de claims clasificados
3. fixes de 1 línea aplicados si había claims falsos

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `.opencode/AGENTS.md` (593 líneas) — reglas 1-10; la Regla 11 se agrega tras la Regla 10 (final de archivo)
- `README.md` (403 líneas) — §Benchmarks: tabla baseline (L322-328) + tabla SIFT-1M (L336-342) + link fuente (L346)
- `docs/operations/BENCHMARKS.md` (212 líneas) — §1 stress protocol, §2 SDK ops, §4 prefetch, §5 SIFT, §6 batch, §7 competitive, §8 canonical (FND-10)
- `docs/operations/PERFORMANCE_TUNING.md` (494 líneas) — claims de HNSW, sync modes, SIMD, storage
- `vantadb-python/README.md` (94 líneas) — claims adjetivos ("Zero-Copy", "sin latencia de red")
- `vantadb-ts/README.md` (158 líneas) — sin claims numéricos de performance
- `README_ES.md` (grep parcial) — mismo link roto L343
- `benchmarks/vanta_benchmark_report.json` (28 líneas) — fuente citada por README; en `.gitignore` (no versionado)
- Plan `docs/plans/2026-08-16-wave-p20-tsys.md` — FND-13 en Wave 4

**Referencias hacia dentro (entrantes):**
- `.opencode/AGENTS.md` — archivo fuente de reglas; referenciado por docs/plans, VANTADB-OPERATING-MANUAL.md, skills. Agregar una regla no rompe referencias existentes.
- `README.md` — archivo raíz; link L346 a `docs/benchmarks/BENCHMARK_OPTIMIZATION_2026.md` está ROTO (archivo vive en `docs/benchmarks/docs/`)
- `docs/operations/BENCHMARKS.md` — referenciado por README.md; no se modifica en esta tarea (solo análisis)

**Referencias hacia afuera (salientes):**
- README.md L346 → `docs/benchmarks/BENCHMARK_OPTIMIZATION_2026.md` (ROTO, fix 1 línea → `docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md`)
- README.md L322-328 → `benchmarks/vanta_benchmark_report.json` (en .gitignore, números NO coinciden con el JSON actual)

**Veredicto de impacto:**
- Editar `.opencode/AGENTS.md` (agregar Regla 11 al final): impacto bajo, solo adición.
- Editar `README.md` (fix link roto L346 + alinear 3 números de tabla baseline L322-328 con el JSON citado): impacto bajo, corrección de información obsoleta.
- Editar `README_ES.md` (mismo link roto L343): impacto bajo, consistencia.
- Crear `docs/Investigaciones/FND-13-benchmarks-honestos.md` (nuevo, español — planificación): impacto nulo.
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan files, .opencode/agents/*, src/.

## Steps

- [x] 1. DISCOVERY: leer README.md, READMEs bindings, BENCHMARKS.md, PERFORMANCE_TUNING.md — ✅
- [x] 2. Análisis de claims → clasificación citado/medible/sin-fuente — ✅ (ver doc de investigación)
- [x] 3. Agregar Regla 11 en `.opencode/AGENTS.md` (tras Regla 10) — ✅
- [x] 4. Aplicar fixes de 1 línea: link roto README.md + README_ES.md; alinear números baseline README con JSON citado — ✅
- [x] 5. Escribir `docs/Investigaciones/FND-13-benchmarks-honestos.md` (inventario + fixes) — ✅
- [x] 6. Verify mecánico del contrato (grep Regla 11 + existencia análisis) — ✅

## Context Save Point

- Fuente canónica de baseline: `benches/canonical_p99.rs` + BENCHMARKS.md §8 (FND-10) — insert 100k×1536d, search p99 3.0746ms (2026-08-16, i5-1235U)
- JSON `benchmarks/vanta_benchmark_report.json` (worktree, gitignored): insert 74.0 rec/s p50 13.17ms; vector p50 2.02ms; hybrid p50 3.11ms
- BENCHMARKS.md §2 (markers START/END): 95 ops/s, vector p50 62ms, hybrid p50 180ms — NO coincide con JSON ni README (inconsistencia documentada, fix >1 línea → anotado como pendiente)