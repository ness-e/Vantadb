# FIND-60 — 49 warnings rustdoc a cero

- **Estado:** ⏳ IN PROGRESS
- **Plan:** `docs/plans/2026-09-07-backlog-triage.md` (Task 3, Wave0)
- **Tipo detectado:** Rust core (docs-only, mecánico)
- **SDP:** discover_skills_v2 BUILD + grep SKILLS-MANIFEST keywords `rustdoc/docs/api/sdk` → `documentation-and-adrs` (cargada), `api-and-interface-design` (en lista v2, no cargada — ponytail: sin cambio de API, no aplica). **SKILLS_CARGADAS: ponytail (full, siempre), documentation-and-adrs**
- **Contrato:** `cargo doc --no-deps -p vantadb -p vantadb_py 2>&1 | grep -c warning` → 0; `cargo doc --no-deps -D warnings -p vantadb` exit 0
- **Gate D:** no disparado — doc-only, 0 símbolos públicos nuevos, contrato mecánico

## DISCOVERY (2026-09-07)

- Conteo plan (49) **stale**: `cargo doc --no-deps -p vantadb` real → **46 warnings** (47 líneas `warning:` incl. resumen).
- Evidencia completa: `C:\Users\Eros\AppData\Local\Temp\opencode\find60-full.txt`
- Causa raíz única: `//!` docs de módulo resuelven links en scope **padre** + links a items `pub(crate)`/privados/cfg-gated.
- Regla pre-mortem respetada: **nunca `pub` para callar rustdoc** → ticks o path público real.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (rangos relevantes):** `src/agentic/mod.rs`, `src/api/scores.rs`, `src/entity/{mod,checker,scene}.rs`, `src/eviction.rs`, `src/gds.rs`, `src/wiki/mod.rs`, `src/skills.rs`, `src/index/{mod.rs,search/nearest.rs}`, `src/metrics/core/mod.rs`, `src/physical_plan/mod.rs`, `src/sdk/api/{graph,memory,namespaces,search}.rs`, `src/storage/{vfile.rs,engine/{mod,maintenance,txn}.rs}`, `src/cli.rs`, `src/lib.rs`, `src/sdk/mod.rs`, `src/storage/mod.rs`
- **Referencias hacia dentro:** ninguna (solo comentarios doc).
- **Referencias entrantes:** ninguna (docs no son dependencia de código).
- **Veredicto:** BLAST RADIUS CERO comportamiento — solo `///`/`//!`. Reversible por slice. Disjoint con FIND-64 (llamaindex py) y MOD-24 (TS): sin conflicto Wave0.

## Fix pattern (2 casos)

1. **Target público existe** → path fully-qualified (`crate::gc::GcWorker`, `crate::entity::*`, `crate::wiki::*`, `crate::eviction::EvictionPolicy`, `crate::accumulator::GraphAccumulator`, `crate::node::UnifiedNode`, `crate::VantaError::*`, `crate::sdk::FIELD_NAMESPACE`, `crate::sdk::memory_record_from_node`, `Self::*` en mismo impl).
2. **Target privado/pub(crate)/cfg-gated/inexistente** → code ticks sin brackets (`` `X` ``), nunca `pub`.

## Steps

- [x] **Slice 1 — trivial ticks:** `src/cli.rs:135` (`<timestamp>` → backticks), `src/api/scores.rs:32,37,57` (`[0,1]`/`[0,2]` → backticks). Verify: `cargo doc` cuenta −4.
- [x] **Slice 2 — sdk/api (7 warnings, 4 archivos):** graph.rs (`VacuumReport`→ticks pub(crate); `FIELD_NAMESPACE`→`crate::sdk::FIELD_NAMESPACE`), memory.rs (`VantaConfig::bulk_commit_interval`→`crate::VantaConfig::bulk_commit_interval`), namespaces.rs (`memory_record_from_node`→`crate::sdk::memory_record_from_node`; `DEFAULT_EXPIRING_SOON_WINDOW_MS`→ticks pub(crate)), search.rs (`VantaError::*`→`crate::VantaError::*`).
- [x] **Slice 3 — entity + módulos (21 warnings, ~11 archivos):** entity/mod.rs ×3, checker.rs ×1, scene.rs ×1, eviction.rs ×1, agentic ×2, gds ×1, wiki ×2, skills ×1, index/mod ×1, nearest ×1, metrics ×1, physical_plan ×6.
- [x] **Slice 4 — storage/engine + vfile (13 warnings, 4 archivos):** engine/mod.rs ×4 (URL batch_insert + Snapshot/VecIndex ticks), maintenance.rs ×3, txn.rs ×3 (`Self::*`), vfile.rs ×3 (Cipher ticks cfg-gated + vfile_mmap ticks).
- [x] **Slice 5 — cierre:** `cargo doc --no-deps -D warnings -p vantadb -p vantadb_py` exit 0 + `grep -c warning` → 0 (incl. fix extra `vantadb-python/src/lib.rs:3` `connect`→ticks, total real 47). `cargo fmt --check` ✅. Diff 100% líneas `///`/`//!` (verificado mecánicamente) → clippy/nextest sin señal nueva. **NO commiteado** (regla tarea: diff listo).

## Deuda / Notas

- `encryption` fuera de default features → `Cipher` nunca linkeable en docs default (ticks permanente).
- `storage::engine`, `sdk::serialization`, `sdk::types` son `pub(crate)` → futuros docs deben usar re-exports públicos (`crate::storage::StorageEngine`, `crate::sdk::*`) o ticks.
