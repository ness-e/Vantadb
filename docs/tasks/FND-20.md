# FND-20 — Documentar trade-off HNSW (ef_search/M: recall vs latencia) + argumento vs IVF/FAISS

- **Plan:** docs/plans/2026-08-16-wave-r2-r7-fnd.md (Task 9, Wave 2)
- **Estado:** ✅ COMPLETED
- **Tipo:** documentation (research + nota técnica)
- **Prio/Effort:** 🟡 / 🟢

## Objetivo

Nota técnica defensible para Show HN en `docs/architecture/FND-20-hnsw-tradeoff.md` (inglés) explicando:
parámetros HNSW actuales (M, ef_construction, ef_search) con citas archivo:línea, el trade-off
recall vs latencia, memoria, y por qué HNSW y no IVF/FAISS/exacta para local-first.

## Archivos clave

- `src/index/graph.rs` (HnswConfig defaults: 255-269)
- `src/index/search/nearest.rs` (ef_search resolution: 71-77; routing flat/ivf/scann: 45-64)
- `src/index/search/neighbors.rs` (use_flat_search: 57-62)
- `src/index/mod.rs` (IndexType + VecIndex: 27-90, create_index: 100-139)
- `src/index/ivf.rs` (k-means build: 79-228)
- `src/index/auto_tune.rs` (auto-tuner: 11-53)
- `docs/operations/BENCHMARKS.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/PERFORMANCE_GUIDE.md`
- `docs/architecture/adr/005_hnsw_parameters.md` (drift detectado: dice ef_construction=200, código dice 100)

## Impacto mapeado (Regla 0)

- Archivo nuevo en `docs/architecture/` — no rompe referencias.
- DRIFT documentado: ADR 005 (ef_construction=200) y PERFORMANCE_TUNING.md (ef_construction=400)
  no coinciden con el código (100). La nota cita el CÓDIGO como fuente de verdad y lo señala.
- Verificado: nota existe, en inglés, con citas archivo:línea y sección "Why not FAISS/IVF".

## Steps

- [x] S1 DISCOVERY: parámetros actuales + backends + benchmarks (codegraph + reads)
- [x] S2 Escribir task file + Impacto mapeado
- [x] S3 Escribir `docs/architecture/FND-20-hnsw-tradeoff.md` (entregable)
- [x] S4 Verify contrato: nota existe + citas archivo:línea + sección why-not-FAISS/IVF + validate-docs-coverage.ps1

## Context Save Point

- Commit: NINGUNO (el lead commitea al cerrar la wave — Regla plan wave R2-R7).
- Backlog/plan: NO tocados.