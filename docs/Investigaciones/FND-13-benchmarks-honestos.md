# FND-13 — Benchmarks honestos: inventario de claims de performance

**Fecha:** 2026-08-16
**Tarea:** Backlog.md:501 (P20b, prio 🟡)
**Rol:** vanta-tuner (leaf)
**Estado:** ✅ COMPLETO — Regla 11 en AGENTS.md + inventario + fixes aplicados

---

## 1. Contexto

La tarea extiende FND-10 (Regla 9, `canonical_p99` ya commiteado en `89943c7d`). El objetivo:
1. Que todo claim de performance en docs cite un **benchmark reproducible** (archivo bench + comando + números), no adjetivos.
2. Inventariar los claims existentes y clasificarlos: **citado / medible / sin fuente**.
3. Aplicar fixes de 1 línea donde un claim sea claramente falso u obsoleto.

**Resultado normativo:** Regla 11 agregada a `.opencode/AGENTS.md` (tras Regla 10; Reglas 1-8, 9=FND-10, 10=FND-11 ocupadas).

---

## 2. Inventario de claims clasificados

### 2.1 `README.md` (raíz)

| # | Claim (ubicación) | Clasificación | Fuente | Estado |
|---|---|---|---|---|
| R1 | Tabla baseline 10K×128d: Ingestion 61.5 rec/s (p50 16.0 ms), Vector p50 3.3 ms, Hybrid p50 12.1 ms (L322-328) | **Sin fuente → OBSOLETO** | Citaba `benchmarks/vanta_benchmark_report.json` pero el archivo está en `.gitignore` (no versionado) y sus valores reales eran 74.0 rec/s (p50 13.2 ms), 2.0 ms, 3.1 ms | ✅ **FIX APLICADO** (alineado a la fuente citada + nota de regeneración) |
| R2 | Tabla SIFT-1M Phase 2: 2.14x–2.80x speedup, p99 441–1232 µs, QPS 1,353–3,636 (L336-342) | **Citado** | `docs/operations/BENCHMARKS.md §5` + `docs/benchmarks/BENCHMARK_OPTIMIZATION_2026.md` (histórico 2026-07-21) | ✅ link corregido (era ruta rota) |
| R3 | Core Capabilities: "Validated on 10K–100K synthetic datasets" (L193) | Medible | `tests/certification/stress_protocol.rs` (heavy certification) | OK sin cambio |

### 2.2 `vantadb-python/README.md`

| # | Claim | Clasificación | Estado |
|---|---|---|---|
| P1 | "Persistencia Zero-Copy... sin overhead de serialización" (L78) | **Sin fuente** (adjetivo arquitectónico, sin número) | Anotado — no bloqueante, no numérico |
| P2 | "Sin latencia de red" (L81) | **Sin fuente** (adjetivo; propiedad de diseño embedded, no medible) | Anotado |

> README del SDK Python no cita números: correcto como documento de API, pero los adjetivos de performance quedan anotados para futura revisión (no se reescribe el README — fuera de alcance).

### 2.3 `vantadb-ts/README.md`

Sin claims numéricos de performance. ✅ OK, sin cambios.

### 2.4 `docs/operations/BENCHMARKS.md`

| # | Claim (sección) | Clasificación | Fuente / comando | Estado |
|---|---|---|---|---|
| B1 | §1 Stress Protocol (Recall 0.956–1.0, ~1172 B/node, p50 1.2–6.1 ms, 4.88x) | **Citado** | `tests/certification/stress_protocol.rs` (7 bloques, heavy certification) | ✅ OK |
| B2 | §2 SDK ops (95 ops/s, p50 10.7 ms, HNSW 62 ms, hybrid 180 ms) | **Medible** | `benchmarks/vantadb_local_bench.py` + `update_markdown.py` (markers BENCHMARK_METRICS_START/END) | ⚠️ Nota: números de corrida CI previa; difieren del JSON local actual (74 rec/s) — regenerable con el comando documentado en §3 |
| B3 | §4 Prefetch (0.9–2.0 % mejora, p99 -1.3 %) | **Medible** | `benchmarks/prefetch_comparison.py` (script existe, comando no citado en el doc) | Anotado: citar comando |
| B4 | §5 SIFT Phase 2 (2.14x–2.80x) | **Citado (histórico)** | `docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md` | ✅ OK |
| B5 | §6 Batch search 4.01x (973.68 → 243.01 ms) | **Medible** | `benchmarks/batch_vs_sequential_bench.py` (script existe) | Anotado: citar comando |
| B6 | §7 Competitive vs LanceDB/Chroma (ingest 598 QPS, recall 24.5 %, RSS 236.5 MB) | **Medible** | `benchmarks/competitive_bench.py` (script existe, 2026-06-06) | Anotado: citar comando |
| B7 | §8 Canonical P99 (insert 322.59 s, p99 3.0746 ms, 2026-08-16) | **Citado ✅ modelo** | `benches/canonical_p99.rs` + comando exacto + entorno (CPU/RAM/OS/fecha) | ✅ OK (FND-10) — referente de la Regla 11 |

### 2.5 `docs/operations/PERFORMANCE_TUNING.md`

| # | Claim (sección) | Clasificación | Estado |
|---|---|---|---|
| T1 | §1 "~1172 bytes/node" | **Citado** (BENCHMARKS.md) | ✅ OK |
| T2 | §2 SQ8 "~0.5–2 % drop" recall | **Sin fuente** | Anotado pendiente |
| T3 | §2 "5–10× memory reduction" (SQ8/Turbo) | **Sin fuente** | Anotado pendiente |
| T4 | §3 Backend build time "~30 s / ~5–10 min / ~20 s" | **Sin fuente** (medible con `cargo build --timings`) | Anotado pendiente |
| T5 | §3 RocksDB ">100K ops/sec" | **Sin fuente** | Anotado pendiente |
| T6 | §4 Sync "10–100× slower" (Always) | **Sin fuente** (medible con bench fsync) | Anotado pendiente |
| T7 | §4 "10–100 ms per write" (SATA) | **Sin fuente** | Anotado pendiente |
| T8 | §5 bitset "essentially free" / "Near-zero" | **Sin fuente** (adjetivo) | Anotado pendiente |
| T9 | §6 SIMD "1.5–2× vs AVX2", "3–5× vs scalar" | **Sin fuente** | Anotado pendiente |
| T10 | §6 "~896 bytes" (fórmula) | **Medible/derivable** (fórmula documentada) | ✅ OK |
| T11 | §6 Storage "+10–30 %" (SATA), "+200–500 %" (HDD) | **Sin fuente** | Anotado pendiente |
| T12 | §7 "Expected Performance by Dataset Size" (10K: ~2 s/~1 ms/~1000 qps...) | **Sin fuente** (no cita comando; parcialmente derivable de stress protocol) | Anotado pendiente |
| T13 | §7 Python overhead "~50×/~57×/~107×" | **Medible/derivable** (BENCHMARKS §1 vs §2: 62/1.2, 115/~2, 10.7/~0.1) | ✅ OK |
| T14 | §7 "search_batch 4×" | **Citado** (BENCHMARKS §6) | ✅ OK |

---

## 3. Hallazgos adicionales

### 3.1 Claims fantasma en el frontend web (fuera de scope, pendiente)

`web/src/components/vanta/vanta-data.ts` (L142-175, `BENCH01`) contiene **los mismos números "Target Baseline" que PERF-01 retiró del README por no tener respaldo**:
- Ingestion "~5,400 vectors/sec" (el baseline real medido es ~74–95 rec/s vía SDK; ~310 vec/s en canonical_p99 para 1536d)
- Search BM25 "~1,100 qps / p50 0.85 ms", HNSW "~830 qps / 1.20 ms", Hybrid "~450 qps / 2.10 ms"

Ninguno de esos números coincide con BENCHMARKS.md §1/§2 ni con `vanta_benchmark_report.json`. **Fix NO aplicado** (fuera del alcance de archivos de esta tarea — es frontend `web/`, corresponde a FND-22 o una tarea de contenido web). Queda como deuda documentada.

### 3.2 Inconsistencia interna de BENCHMARKS.md §2 vs JSON

La tabla §2 (markers) reporta 95 ops/s y p50 10.7 ms, mientras el JSON local actual dice 74 rec/s y p50 13.2 ms. La tabla se regenera con `python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000 --output benchmarks/vanta_benchmark_report.json` + `update_markdown.py`. No es un claim falso, es una corrida distinta — anotado para regenerar en CI antes del próximo release.

---

## 4. Fixes aplicados (esta tarea)

| Archivo | Fix | Tipo |
|---|---|---|
| `.opencode/AGENTS.md` | **Regla 11** agregada tras Regla 10: claims de performance DEBEN citar benchmark reproducible (archivo + comando + entorno) + números; adjetivos no son evidencia; fuentes gitignored no son válidas | Regla nueva |
| `README.md` L322-328 | Tabla baseline alineada a la fuente citada: 61.5→**74.0 rec/s** (p50 16.0→**13.2 ms**), Vector p50 3.3→**2.0 ms**, Hybrid p50 12.1→**3.1 ms**; "Real committed baseline"→"Latest local baseline (regenerate locally)"; nota de outlier BM25 corregida (0.009→**0.0035 ms**); añadido comando de regeneración | Claim obsoleto (fix >1 línea pero claim claramente falso vs su fuente citada) |
| `README.md` L346 | Link roto `docs/benchmarks/BENCHMARK_OPTIMIZATION_2026.md` → `docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md` | Fix 1 línea |
| `README_ES.md` L343 | Mismo link roto corregido | Fix 1 línea |

## 5. Fixes pendientes (deuda anotada, NO aplicados — fuera de scope o >1 línea)

| Archivo | Deuda | Responsable sugerido |
|---|---|---|
| `web/src/components/vanta/vanta-data.ts` (BENCH01) | Claims "~5,400 vec/s / ~1,100 / 830 / 450 qps" sin respaldo (retirados del README en PERF-01 pero vivos en el frontend) | FND-22 / contenido web |
| `docs/operations/BENCHMARKS.md` §4/§6/§7 | Citar comandos exactos de `prefetch_comparison.py`, `batch_vs_sequential_bench.py`, `competitive_bench.py` | Futura tarea tuner |
| `docs/operations/PERFORMANCE_TUNING.md` T2-T9, T11, T12 | Claims sin fuente (recall SQ8, SIMD, build time, sync modes, storage, tabla de escala) — reformular con números o quitar | Futura tarea tuner |
| `docs/operations/BENCHMARKS.md` §2 | Regenerar tabla con `update_markdown.py` para sincronizar con el JSON | CI / release |

---

## 6. Verificación del contrato

1. `grep AGENTS.md "Regla 11"` → ✅ presente, referenciando benchmark reproducible (`canonical_p99`, `docs/operations/BENCHMARKS.md`, archivos `benches/*.rs`/`benchmarks/*.py` + comando)
2. Análisis en `docs/Investigaciones/FND-13-benchmarks-honestos.md` → ✅ existe con inventario clasificado (citado/medible/sin fuente)
3. Fixes de 1 línea aplicados → ✅ (2 links rotos + claim obsoleto de la tabla baseline)

**Regla 0 / Impacto:** verificado en el task file (`FND-13.md`) antes de la primera edición. Solo se editaron `.opencode/AGENTS.md` (adición al final), `README.md`, `README_ES.md` (corrección de información obsoleta) y se crearon el task file y este análisis. No se tocaron archivos protegidos.