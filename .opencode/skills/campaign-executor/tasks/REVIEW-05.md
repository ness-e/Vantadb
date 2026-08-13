# REVIEW-05: God files restantes — serialize.rs, distance.rs, physical_plan.rs

## Metadata
- **Plan file:** ninguno activo (Backlog directo)
- **Fuente:** docs/Backlog.md:368 (P14)
- **Esfuerzo:** 🟡 1 semana
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (refactor puro — zero behavior change)
- **Turns estimados:** 25-45
- **Creado:** 2026-08-12T00:00
- **last-synced:** 2026-08-12T00:00
- **Estado:** ✅ COMPLETED (Steps 1-7 ✅ — god files divididos, verificación full exit 0, review P2-01 APPROVE)
## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `serialize.rs`: `impl CPIndex` (graph.rs via index/mod.rs `pub(crate) mod serialize`). `distance.rs`: core.rs, diskann.rs, flat.rs, graph.rs, ivf.rs, search.rs (vía `use crate::index::distance::*` y `pub use distance::*` en index/mod.rs:22). `physical_plan.rs`: `pub mod physical_plan` en lib.rs:110 — operadores usados por el executor/query planner |
| Callees | serialize.rs → CPIndex (definido en index/graph.rs), serde, mmap. distance.rs → solo std (SIMD f32x8/f32x16 intrinsics), sin deps externas de cómputo. physical_plan.rs → StorageEngine, UnifiedNode, RelOp (crate::query), HNSW search |
| Implicaciones | `pub use distance::*` en index/mod.rs:22 DEBE quedar idéntico (kernels internos se quedan pub(crate)). `pub mod physical_plan` en lib.rs:110 y los structs públicos (PhysicalScan, PhysicalFilter, etc.) NO cambian. Cero cambio de comportamiento — refactor de movimiento puro. NO tocar matemática de kernels ni métricas |
| Riesgo | medio — distance.rs es hot path (kernels SIMD); cualquier cambio accidental en los kernels rompe búsquedas. physical_plan.rs es módulo público con 10 operadores acoplados al trait `PhysicalOperator`. serialize.rs es serialization hot path (deuda P2-7 conocida, NO tocar semántica de formato) |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** el implementador DEBE
> leer los 3 archivos COMPLETOS y verificar los re-exports con grep ANTES del primer edit.

- **Archivos leídos (completos):** ✅ `src/index/serialize.rs` (1595L, completo), `src/index/distance.rs` (1721L, completo), `src/physical_plan.rs` (1542L, completo) + `src/index/mod.rs` (re-exports: `pub(crate) mod serialize;` 17, `pub(crate) mod distance;` 7, `pub use distance::*;` 22) + `src/lib.rs` (`pub mod physical_plan;` 110) + `src/query.rs` (`pub enum RelOp` 134, `pub trait PhysicalOperator` 486 — trait en query.rs, confirmado por grep). Baseline `cargo check -p vantadb` tomado el 2026-08-12 (worktree incluye REVIEW-04)
- **Archivos referenciados hacia dentro:** serialize.rs → `crate::index::graph::CPIndex`, serde, mmap. distance.rs → std only. physical_plan.rs → `crate::storage::engine::StorageEngine`, `crate::node::{UnifiedNode, FieldValue}`, `crate::query::RelOp`, HNSW (vector/), `PhysicalOperator` trait
- **Archivos que referencian a los editados:** `src/index/mod.rs` (`pub(crate) mod serialize;`, `pub(crate) mod distance;`, `pub use distance::*;`), `src/lib.rs` (`pub mod physical_plan;`), core.rs/diskann.rs/flat.rs/graph.rs/ivf.rs/search.rs (`use crate::index::distance::*`), executor/planner (PhysicalOperator implementors)
- **Veredicto impacto:** medio — movimiento puro; si `distance::*` re-exports y `physical_plan` públicas quedan idénticas, nada externo cambia. Clave: NO renombrar items públicos, NO cambiar visibilidad pub→pub(crate) de lo que está expuesto, NO alterar kernels/metrics

## Contrato
"`cargo check -p vantadb` pasa, `cargo nextest run --profile audit -p vantadb --build-jobs 2` pasa, `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` sin warnings nuevos, `cargo fmt --check` pasa, re-exports idénticos (`grep 'pub use distance' src/index/mod.rs` y `grep 'pub mod physical_plan' src/lib.rs` sin cambio), y cero cambios de comportamiento/API (diff de lógica = solo movimiento)"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `pub use distance::*` (index/mod.rs:22) y todos los callers `crate::index::distance::{...}` resuelven idénticos
  2. `pub mod physical_plan` (lib.rs:110) y los structs/impls públicos (PhysicalScan, PhysicalFilter, PhysicalTextFilter, PhysicalVectorSearch, PhysicalProject, PhysicalLimit, PhysicalSort, PhysicalNestedLoopJoin, PhysicalSubqueryFilter, PhysicalVectorRefine) con MISMA firma/semántica
  3. `impl CPIndex` de serialize.rs — los métodos públicos (serialize_to_bytes, serialize_to_writer, deserialize_from_bytes, persist_to_file, load_from_file, sync_to_mmap) mantienen firma y SEMÁNTICA de formato byte-para-byte idéntica (P2-7 NO se toca en esta tarea)
  4. Kernels SIMD de distance.rs (f32x8/f32x16) y las métricas (cosine_sim_f32, euclidean_distance_squared_f32, calculate_similarity, MetricMapper) matemáticamente intocadas
  5. Cero `unwrap()`/`expect()` nuevos (core-engine.md R-3)
  6. Sin deps nuevas, sin feature-gating cambiado
- **Comandos de verificación:**
  - `cargo check -p vantadb` → exit 0
  - `cargo nextest run --profile audit -p vantadb --build-jobs 2` → exit 0 (misma suite, misma cantidad de tests)
  - `cargo fmt --check` → exit 0
  - `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → exit 0
- **Deuda pendiente:** ninguna si el refactor preserva invariantes; P2-7 (serialización completa sin zero-copy) NO es deuda de esta tarea — se documenta como diferida

## Recitation (canónico — estructura única)

- **activeGoal:** ✅ COMPLETED — REVIEW-05: god files serialize.rs/distance.rs/physical_plan.rs divididos en submódulos sin cambio de API ni comportamiento
- **lastAction:** Steps 1-7 ejecutados: splits completos + verificación full exit 0 + review P2-01 APPROVE (vanta-review)
- **result:** ✅ COMPLETED — 1878/1878 tests, clippy -D warnings limpio, fmt limpio, kernels byte-idénticos
- **nextAction:** vanta-lead: stage los 11 archivos nuevos (untracked) + commit convencional refactor:, luego marcar COMPLETED en campaign. NO commitear archivos de REVIEW-04 (docs/* eliminados, REVIEW-04.md)
- **contract:** verificación = `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2` + clippy -D warnings + fmt + re-exports idénticos
- **nextTask:** REVIEW-04 (god modules node.rs/config.rs/vfile.rs — node.rs ya partido en otra sesión)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — refactor de movimiento no introduce deuda nueva; paga la deuda "god files P14". P2-7 queda documentada como diferida (no es moneda de pago de esta tarea)

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file se cumple (check + nextest + clippy + fmt + re-exports idénticos) |
| **Commit** | Commits atómicos por archivo (serialize.rs, distance.rs, physical_plan.rs), conventional commit `refactor:`, diff de lógica ZERO (solo movimiento) |
| **Release** | No aplica release (refactor interno sin cambios API) — justificado: cambio interno, semver no cambia |

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (diagnostics, goto def para mover items con precisión)
- codegraph_explore (verificar callers — nota: presupuesto agotado esta sesión; usar grep si codegraph no responde)

## Investigation Notes
- Backlog 2026-08-09 reportaba: serialize.rs 1452L → hoy 1595L; distance.rs 1591L → hoy 1721L; physical_plan.rs 1380L → hoy 1542L. Los god files CRECIERON — refactor sigue justificado
- tests.rs 4076L (storage/engine) ya fue dividido ✅ — mismo patrón a replicar
- **serialize.rs** (1595L): `impl CPIndex` (19-707, ~700L de impl) + `#[cfg(test)] mod tests` (709-1595, ~886L de tests). Split natural: `serialize/` subdir con `bytes.rs` (serialize_to_bytes/writer + deserialize_from_bytes), `file.rs` (persist_to_file/load_from_file/sync_to_mmap), `mod.rs` (re-exports) + tests distribuidos
- **distance.rs** (1721L): kernels SIMD (18-140 f32x8, 188-324 f32x16) + MetricMapper (56-75) + métricas públicas (141-186, 325-364) + sq8_similarity (365-463) + calculate_similarity (464-540) + tests (542-1721, ~1180L). Split natural: `distance/kernels.rs` (SIMD), `distance/mapper.rs` (MetricMapper + calculate_similarity), `distance/metrics.rs` (cosine/euclidean públicas), `mod.rs` re-exports + tests distribuidos
- **physical_plan.rs** (1542L): 10 operadores por sección (11-784) + tests (821-1542). Split natural por operador: `physical_plan/scan.rs`, `physical_plan/filter.rs` (Filter+TextFilter), `physical_plan/vector.rs` (VectorSearch+VectorRefine), `physical_plan/project.rs` (Project+Limit+Sort), `physical_plan/join.rs` (NestedLoopJoin+SubqueryFilter), `mod.rs` re-exports + tests

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 2 — (1) dónde vive el trait `PhysicalOperator` y qué re-exports públicos necesita physical_plan/ (decidir en ejecución con grep); (2) si los tests de physical_plan usan helpers compartidos que requieren módulo `tests` unificado (impacto mínimo, decidir en ejecución) |
| Pendientes de ejecución (downhill) | 7 steps |
| % completado | 0% |

## Steps

### Step 1: Inventario exacto + estructura de directorios
- **Archivos:** `src/index/serialize.rs`, `src/index/distance.rs`, `src/physical_plan.rs`, `src/index/mod.rs`, `src/lib.rs`, `src/query.rs`
- **Acción:** Leer los 3 archivos COMPLETOS (Regla 0). Inventariar todos los items `pub`/`pub(crate)` top-level y tests por archivo. Verificar re-exports actuales (`grep "use crate::index::distance" src/*.rs`, `grep "pub mod physical_plan" src/lib.rs`). Confirmar dónde vive el trait `PhysicalOperator`. Convertir cada archivo en directorio (`serialize/`, `distance/`, `physical_plan/`) con `git mv` + `mod.rs` vacío que re-exporta todo (primero: `serialize.rs` → `serialize/mod.rs`)
- **Verify:** `cargo check -p vantadb` (funciona igual con mod.rs)
- **Estado:** ✅ COMPLETED

### Step 2: Split serialize.rs — separar impl CPIndex por concern
- **Archivos:** `src/index/serialize/mod.rs`, `src/index/serialize/bytes.rs`, `src/index/serialize/file.rs`
- **Acción:** Mover métodos del impl CPIndex a submódulos por concern (bytes.rs: serialize_to_bytes/serialize_to_writer/deserialize_from_bytes; file.rs: persist_to_file/load_from_file/sync_to_mmap) con `pub use` re-exports en mod.rs. Mantener firmas EXACTAS y semántica de formato byte-para-byte
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2` (tests de serialización pasan igual)
- **Estado:** ✅ COMPLETED

### Step 3: Mover tests de serialize.rs a módulos de test por submódulo
- **Archivos:** `src/index/serialize/**` (test modules)
- **Acción:** Distribuir los ~886L de tests a `#[cfg(test)] mod tests` dentro de bytes.rs/file.rs según el item que testean. Cero tests eliminados/reducidos
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2`
- **Estado:** ✅ COMPLETED

### Step 4: Split distance.rs — kernels SIMD, mapper, métricas
- **Archivos:** `src/index/distance/mod.rs`, `src/index/distance/kernels.rs`, `src/index/distance/mapper.rs`, `src/index/distance/metrics.rs`
- **Acción:** Mover kernels SIMD (f32x8/f32x16, dot products, euclidean sq) a kernels.rs; MetricMapper + calculate_similarity + sq8_similarity a mapper.rs; métricas públicas (f32_l2_norm, cosine_sim_*, euclidean_*) a metrics.rs. `pub use` re-exports en mod.rs. **Los kernels quedan pub(crate) — la visibilidad desde index/mod.rs `pub use distance::*` NO cambia para items públicos.** OJO: `pub use distance::*` en index/mod.rs re-exporta SOLO items pub — verificar que los callers (core.rs, diskann.rs, flat.rs, graph.rs, ivf.rs, search.rs) sigan resolviendo
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2` + comparar resultados de un test de similitud (sanity: resultados matemáticos idénticos)
- **Estado:** ✅ COMPLETED

### Step 5: Mover tests de distance.rs a módulos de test por submódulo
- **Archivos:** `src/index/distance/**` (test modules)
- **Acción:** Distribuir los ~1180L de tests a los submódulos según el item que testean (kernels → kernels.rs, mapper → mapper.rs, metrics → metrics.rs). Cero tests eliminados/reducidos
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2` (misma suite, misma cantidad)
- **Estado:** ✅ COMPLETED

### Step 6: Split physical_plan.rs — operadores por submódulo
- **Archivos:** `src/physical_plan/mod.rs`, `src/physical_plan/scan.rs`, `src/physical_plan/filter.rs`, `src/physical_plan/vector.rs`, `src/physical_plan/project.rs`, `src/physical_plan/join.rs`
- **Acción:** Mover cada operador a su submódulo (scan.rs: PhysicalScan; filter.rs: PhysicalFilter+PhysicalTextFilter; vector.rs: PhysicalVectorSearch+PhysicalVectorRefine; project.rs: PhysicalProject+PhysicalLimit+PhysicalSort; join.rs: PhysicalNestedLoopJoin+PhysicalSubqueryFilter). Re-exports `pub use` en mod.rs. Verificar `impl PhysicalOperator` — el trait y sus imports (crate::query::RelOp, StorageEngine, UnifiedNode) resuelven desde cada submódulo
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2` (tests del planner/executor pasan)
- **Estado:** ✅ COMPLETED

### Step 7: Verificación full + re-exports + distribución + review
- **Archivos:** todo el refactor
- **Acción:** `cargo fmt --check`, `cargo clippy -p vantadb --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit -p vantadb --build-jobs 2`, comparar re-exports (`grep 'pub use distance' src/index/mod.rs`, `grep 'pub mod physical_plan' src/lib.rs` idénticos al previo), confirmar zero diff de lógica (`git diff --stat`: solo movimientos), y ejecutar Review por agente DISTINTO (P2-01 gate — vanta-audit o vanta-review)
- **Verify:** todos los comandos exit 0
- **Estado:** ⬜ PENDING
- **Estado:** ✅ COMPLETED
## Dependencias
- Ninguna en bloqueo (standalone desde Backlog). Nota: REVIEW-04 (node.rs split) está en progreso en otra sesión con worktree sucio — REVIEW-05 toca archivos disjuntos (index/serialize.rs, index/distance.rs, physical_plan.rs) y NO debe commitear los archivos node/ de REVIEW-04. Verificar que el commit de REVIEW-05 stage SOLO sus archivos

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (leaf — solo revisa, nunca implementa)
- **Enfoque:** ¿el refactor es movimiento puro o cambió semántica? ¿los kernels SIMD y métricas quedaron matemáticamente idénticos? ¿los re-exports de distance::* y physical_plan públicos quedaron intactos? ¿serialize mantiene formato byte-para-byte?
- **Cómo se probó:** comparación byte-a-byte original staged (`git show :src/index/distance/mod.rs`, `:src/index/serialize/mod.rs`, `:src/physical_plan/mod.rs`) vs archivos nuevos con script de brace-matching + comandos reales ejecutados (check/nextest/clippy/fmt), no auto-reporte. Detalle abajo.
- **Checklist anti-hábitos tóxicos:** (se llena abajo)
- **Veredicto:** ✅ APPROVE — ver sección "Evidencia" para el detalle completo

### Evidencia (2026-08-12, contexto fresco, agente distinto)

**1. Movimiento puro (comparación con git):**
- `distance`: 15/15 kernels/métricas listados en el task file **byte-idénticos** (euclidean_distance_sq_f32x8/x16, f32_dot_product_f32x8/x16, f32_dot_and_norm_b_sq_f32x8/x16, f32_l2_norm, cosine_sim_f32, cosine_sim_cached_norms, cosine_sim_with_query_norm, euclidean_distance_squared_f32, euclidean_distance_sq_with_norms, sq8_similarity, calculate_similarity, f32_slice_similarity). Cero funciones de producción ausentes.
- `serialize`: los 6 métodos públicos de `impl CPIndex` (serialize_to_bytes, serialize_to_writer, deserialize_from_bytes, persist_to_file, load_from_file, sync_to_mmap) con firma y cuerpo idénticos. Formato VNDX no tocado (P2-7 diferida, sin mezclar).
- `physical_plan`: 10 structs públicos idénticos (PhysicalScan, PhysicalFilter, PhysicalTextFilter, PhysicalVectorSearch, PhysicalProject, PhysicalLimit, PhysicalSort, PhysicalNestedLoopJoin, PhysicalSubqueryFilter, PhysicalVectorRefine). Cero funciones de producción ausentes.
- API pública (`pub` items) por módulo: distance 8=8, serialize 6=6, physical_plan 10=10 — **removed: [], added: []**.

**2. Re-exports intactos:**
- `src/index/mod.rs:22` → `pub use distance::*;` ✓ (idéntico al previo)
- `src/lib.rs:110` → `pub mod physical_plan;` ✓ (idéntico)
- `src/physical_plan/mod.rs` re-exporta los 10 operadores vía `pub use scan/filter/vector/project/sort/join::*` ✓

**3. Visibilidad correcta:**
- kernels internos (DistanceKernels, KERNELS, select_kernels, f32_dot_product, f32_dot_and_norm_b_sq, type aliases) pasaron de privados a `pub(crate)` — **necesario** porque ahora viven en submódulo `kernels.rs` y `metrics.rs`/`mapper.rs` (hermanos) los consumen. Anticipado en el task file ("Los kernels quedan pub(crate)"). No cambia API pública (seguían siendo internos al crate).
- `evaluate_condition` es `pub(crate) fn` en `filter.rs:59` y re-exportado solo para tests: `#[cfg(test)] pub(crate) use filter::evaluate_condition;` en `mod.rs:24` ✓

**4. Cero unwrap()/expect() nuevos, sin deps nuevas:**
- Conteos idénticos: distance (0,3), serialize (40,1), physical_plan (132,6) — antes vs después, delta 0. Los `unwrap_unchecked` que parecían faltar eran menciones en comentarios (falsos positivos del script).
- `Cargo.toml`/`Cargo.lock`: sin diff ✓

**5. Tests preservados completos:**
- distance: 66 = 66 tests (#[test] antes vs después, distribuidos kernels/mapper/metrics/mod)
- serialize: 28 = 28
- physical_plan: 43 = 43 (consolidados en `mod.rs` con MockScan/helpers compartidos — consistente con incógnita #2 del task file)

**6. Comandos ejecutados (exit code real):**
- `cargo check -p vantadb` → 0
- `cargo fmt --check` → 0
- `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → 0
- `cargo nextest run --profile audit -p vantadb --build-jobs 2 --lib -E 'test(euclidean) or test(cosine) or test(sq8) or test(calculate_similarity) or test(metric_mapper) or test(f32_slice_similarity) or test(distance)'` → **100/100 passed**
- `cargo nextest run --profile audit -p vantadb --build-jobs 2 --lib -E 'test(physical_) or test(evaluate_condition) or test(relop)'` → **44/44 passed**
- `cargo nextest run --profile audit -p vantadb --build-jobs 2` (suite completa) → **1878/1878 passed** (1 slow: concurrent_insert_preserves_hnsw_invariants 65.6s, no falla)

**Checklist anti-hábitos tóxicos:**
| Hábito tóxico | Estado |
|---|---|
| Cambio de semántica disfrazado de refactor | ✅ No detectado — bodies byte-idénticos, solo movimiento + visibilidad pub(crate) interna necesaria |
| Kernels/métricas alteradas silenciosamente | ✅ 15/15 byte-idénticos |
| Re-export roto / API pública cambiada | ✅ `pub use distance::*`, `pub mod physical_plan`, 10 structs y 6 métodos idénticos |
| Tests eliminados/reducidos | ✅ 66+28+43 preservados, suite 1878/1878 |
| unwrap/expect nuevos o deps nuevas | ✅ cero |
| Auto-reporte sin verificación | ✅ comandos ejecutados en esta sesión |

**Notas (no bloqueantes):**
- 🟡 `physical_plan/mod.rs` (748L) contiene el módulo de tests consolidado — no es "re-export puro" como el nombre sugiere, pero es la decisión correcta dado MockScan/helpers compartidos (incógnita #2 del task file). No es código de producción nuevo.
- 🟢 Los archivos están staged/untracked correctamente (3 RM + 11 archivos nuevos). Al commitear, stage explícito SOLO de los archivos de REVIEW-05 (el worktree tiene docs/ y REVIEW-04 ajenos sin commitear).
- 🟢 `serialize/mod.rs` y `distance/mod.rs` tienen el test `#[cfg(miri)]` `miri_distance_public_dispatch_paths` que existía en el original — preservado.

## Notas
- Este refactor replica el patrón de REVIEW-04 (node.rs → node/ con submódulos por concern) — es el MISMO approach validado
- NO tocar la matemática de kernels/metrics: es hot path de búsqueda, cualquier cambio silencioso rompe resultados. Verificar con un sanity test de similitud post-split
- P2-7 (serialización completa sin zero-copy) es deuda de performance CONOCIDA en serialize.rs — esta tarea SOLO mueve código, no optimiza. No mezclar
- El worktree tiene el split de REVIEW-04 sin commitear (src/node.rs → src/node/). REVIEW-05 NO toca esos archivos; al commitear, stage explícito de solo los archivos de REVIEW-05

## Context Save Point
- **Fecha:** 2026-08-12T00:00
- **Branch:** develop (verify branch actual)
- **CI pendiente:** sí — verify.ps1 antes de push
- **Decisiones:** split por concern replicando REVIEW-04; kernels SIMD intocables; P2-7 diferida; commit con stage explícito (worktree tiene cambios de REVIEW-04 ajenos)
- **Problemas conocidos:** codegraph presupuesto agotado esta sesión — usar grep para verificar callers
- **Próxima tarea:** REVIEW-04 (completada en otra sesión — verificar estado antes de migrar)
