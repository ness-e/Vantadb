# Métricas — landing zone de artefactos de medición

Todo lo que **mide** el proyecto y **no es código funcional** vuelca sus
artefactos aquí, divididos por área. El código que mide sigue viviendo donde
su runner lo exige (`benches/` para `cargo bench`, `tests/` para `cargo
nextest`, `benchmarks/` y `evals/` para harnesses ad-hoc multilingües);
**esta carpeta centraliza solo las salidas**.

## Estructura

| Carpeta | Quién escribe | Qué contiene |
|---|---|---|
| `certificacion/` | `tests/common/mod.rs` (`VantaHarness`) | `vanta_certification.json`: sidecar de los tests de `tests/certification/` (correctitud + tiempos + memoria por bloque) |
| `hnsw/` | benches HNSW (`hnsw_pure`, `hnsw_recall_ef`, …) | reportes de recall/latencia/ef |
| `grafo/` | benches y tests de grafo/GDS | recorridos, pagerank, centralidad |
| `query/` | benches de búsqueda (`hybrid_queries`, …) | latencias, RRF, perfiles |
| `temporal/` | tests de bordes temporales | ventanas, compat |
| `gobierno/` | governor, GC, memoria (`memory_budget`, …) | presupuestos, GC, telemetría |
| `embeddings/` | `benchmarks/embed_bench.py`, `evals/` | calidad de embeddings, beir |

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
