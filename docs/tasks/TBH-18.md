# TBH-18 — Evaluar `dhat` 0.3.3 para heap-usage testing (DOC-ONLY)

## Estado
- ✅ Plan: `docs/plans/2026-08-30-testing-bench-harden.md`
- 🔄 In progress
- Pendiente commit

## Fase DISCOVERY ✅

- `git grep "dhat" Cargo.toml` → **0 hits** ✅ (no instalado)
- `git grep -liE "dhat" -- '*.rs' '*.toml'` → 0 hits ✅ (solo aparece en `.opencode/agents/vanta-tuner.md` como mención futura y en `docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md` como herramienta de referencia)
- `git grep -iE "alloc_regression|heap_regression|memory_leak" -- '*.rs' '*.md'` → **0 hits** ✅
  - **Conclusión clave:** NO existe deuda documentada de regresiones de alloc en el workspace
- `tests/memory_*.rs` existen (4 archivos: `memory_telemetry.rs`, `memory_api.rs`, `memory_export_import.rs`, `memory_brutality.rs`) y cubren comportamiento de memoria vía APIs públicas (jemalloc-ctl gauges, RSS via sysinfo/PSAPI, mmap resident, HNSW logical bytes). NO son heap-usage asserts al estilo `dhat::assert_eq!`.
- `benches/memory_budget.rs` existe — bench de presupuesto de memoria con criterion, no heap-usage testing.
- `src/allocator.rs` NO existe — la elección de allocator vive en `src/bin/vanta-cli.rs:12-21` y `vantadb-server/src/main.rs:6-18` con `#[global_allocator]` condicional (mimalloc en Windows via `custom-allocator`, jemalloc en Linux/macOS via `jemalloc` feature).
- Web validado contra docs.rs/dhat/0.3.3 y github.com/nnethercote/dhat-rs (29 Aug 2026 release): el crate mantiene el warning experimental explícito del autor y la nota "maintenance is not a high priority of the author".

## Decisión

**NO agregar `dhat` al workspace.** Justificación:

1. **YAGNI + decisión D5** ("justificar cada dep antes de añadir"): el audit multi-agente del
   2026-08-30 no encontró regresiones de alloc conocidas ni deuda abierta al respecto. `dhat`
   es una herramienta para detectar un problema que no hemos observado.

2. **Estado experimental explícito del crate** (verbatim del crate-level docstring del autor
   Nicholas Nethercote, fuente: docs.rs/dhat/0.3.3 y github.com/nnethercote/dhat-rs, consultado
   2026-08-30):

   > *"This crate is experimental. It relies on implementation techniques that are hard to
   > keep working for 100% of configurations. It may work fine for you, or it may crash,
   > hang, or otherwise do the wrong thing. Its maintenance is not a high priority of the
   > author. Support requests such as issues and pull requests may receive slow responses,
   > or no response at all."*

   Para un dep que tocaría el `#[global_allocator]` (recurso crítico de los binarios
   distribuídos `vanta-cli`/`vantadb-server`, gated por release-ci.md regla 1), introducir
   un crate con mantenimiento no-prioritario rompe la regla "no nuevas deps sin justificación
   robusta".

3. **Conflicto con el allocator de producción**: `dhat::Alloc` es `#[global_allocator]` y solo
   puede haber uno activo por binario. VantaDB usa `mimalloc`/`jemalloc` (release-ci.md
   regla 1, INV-004). Esto significa que heap-usage testing con `dhat` correría bajo el
   allocator de `dhat`, no el de producción — los números no reflejarían el comportamiento
   real del binario distribuido. Para mantener números representativos habría que escribir
   asserts paralelos para cada allocator, duplicando superficie de test.

4. **Restricciones de uso que complican CI**: `dhat` (a) solo funciona en `release` profile,
   (b) requiere que cada heap-usage test corra en su propio proceso (un solo `Profiler` activo
   a la vez), (c) `cargo test -- --test-threads=1` si se mezclan en un solo integration test.
   Esto encaja mal con la Fast Gate actual (<5 min) y rompe el patrón de tests paralelos.

5. **No hay regresión a detectar**: el coverage actual de memoria es funcional (jemalloc-ctl
   gauges, RSS, mmap resident, HNSW bytes) — no detecta alloc-per-operation. Si en el futuro
   surge evidencia concreta (profiling muestra hot path con growth inesperado, benchmark
   muestra drift sin causa de CPU), `dhat` se justifica para ESA superficie específica.

## Fase EJECUCIÓN (pendiente)

1. Crear `docs/research/dhat-evaluation-2026-08-30.md` con la decisión y razonamiento.
2. Commit: `docs(TBH-18): record dhat evaluation decision (not introduced; no alloc regressions documented)`.

## Cierre (pendiente)

- Verify `docs/research/dhat-evaluation-2026-08-30.md` existe.
- `git add` + commit.
- `campaign_update_task_state` → `completed`.