# Métricas — landing zone de artefactos de medición

Todo lo que **mide** el proyecto y **no es código funcional** vuelca sus
artefactos aquí, divididos por área. El código que mide sigue viviendo donde
su runner lo exige (`benches/` para `cargo bench`, `tests/` para `cargo
nextest`, `benchmarks/` y `evals/` para harnesses ad-hoc multilingües);
**esta carpeta centraliza solo las salidas**.

## Estructura

| Carpeta | Quién escribe | Qué contiene |
|---|---|---|
| `certificacion/` | `tests/common/mod.rs` (`VantaHarness`, default `VANTA_CERT_REPORT`) | `vanta_certification.json`: sidecar de `tests/certification/*` (correctitud + tiempos + memoria por bloque) |
| `benchmarks/` | `benchmarks/vantadb_local_bench.py` (`--output`, default aquí) | `vanta_benchmark_report.json`: throughput/latencias del bench local. Lo lee `benchmarks/update_markdown.py` (fallback) para la tabla de `docs/user/operations/BENCHMARKS.md` |
| `benchmarks/` | benches Criterion (`common::write_metric_json`) | `canonical_summary.json`, `recall_ef_curve.json`, `recall_param_sweep.json`, `recall_acorn.json`, `hybrid_recall.json`, `encrypted_summary.json`, `benchmark_report_*.json`, `bench_comparison_*.json`, `*_results.txt`, `w7-results.txt` |
| `embeddings/` | `benchmarks/embed_bench.py` (`--output`, default aquí) | `embed_bench_report.json`: recall/QPS/RSS/p50 por modelo |
| `memoria/` | `evals/memory_bench.py`, `evals/memory_harness.py` (`--output`), `evals/runners/head_to_head.py` (nombre fijo `report_h2h_<label>_<system>.json`) | reportes de harness y head-to-head de memoria |

## Panel (`dashboard.html`, generado)

`python dev-tools/gen_metrics_dashboard.py` lee todos los JSON de arriba
(todos opcionales: lo ausente se muestra "sin datos") + el manifiesto de
datasets + el baseline de criterion, y genera `metricas/dashboard.html`
autocontenido (sin CDN ni servidor: doble clic). Secciones: tarjetas,
curva recall-QPS, latencias, tablas de sweeps, historial de certificación,
datasets y gaps. Regenerar tras cada corrida que quieras visualizar.

## Convención (obligatoria para harnesses nuevos)

1. **Nunca escribir fuera de `metricas/`**: ni en la raíz, ni junto al
   script. La ruta se resuelve desde la raíz del repo.
2. **Ruta configurable por env** (patrón `VANTA_CERT_REPORT`): el default
   apunta aquí; el env la overridea para corridas ad-hoc
   (ej. `$env:VANTA_CERT_REPORT="target/memory_telemetry.json"`).
3. **Crear el directorio si no existe** (`create_dir_all`, sin fallar si no
   se puede escribir: la métrica nunca debe romper el test).
4. **Baselines commiteados se quedan con su script lector**
   (`benchmarks/criterion_baseline.json`, etc.): moverlos rompería a quien
   los compara. Solo el índice y esta convención viven aquí trackeados;
   todo lo demás está gitignored.

## Leer resultados

* Certificación: `metricas/certificacion/vanta_certification.json`
  (array de corridas; cada una agrupa `block_name -> {duration_secs,
  memory_*, timestamp}`).
* Docs de telemetría: `docs/user/operations/MEMORY_TELEMETRY.md`.
