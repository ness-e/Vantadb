# FIND-48: Split src/index/graph.rs 2031L por concern

## Metadata
- **Plan file:** docs/plans/2026-09-08-backlog.md (Task 1, Wave0)
- **Fuente:** docs/Backlog.md fila FIND-48 + plan Task 1
- **Esfuerzo:** 🟠 2-3d (estimado plan) — ejecución realista en 1 sesión (move mecánico)
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (refactor puro — NO feature-add: cero símbolos públicos nuevos, solo move + re-exports)
- **Turns estimados:** 12
- **Creado:** 2026-09-08T12:00
- **last-synced:** 2026-09-08T13:00
- **Estado:** ✅ COMPLETED (Steps 1-4 ✅ · verify full verde 2026-09-08 retry fresco)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 1 step (Step 4: suite audit + commit full-verify)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (fuera de src/index/) | src/cost_estimator.rs (CPIndex/HnswConfig en tests), src/lib.rs (re-export VECTOR_INDEX_VERSION), src/migration.rs (VECTOR_INDEX_VERSION) |
| Callers (dentro de src/index/) | mod.rs (`pub use graph::*`), flat.rs (HnswNode), ivf.rs (HnswNode/HnswConfig/CPIndex), neighbor_index.rs (NeighborVec), search/{mod,layer,nearest,neighbors,pool,alternate,tests}.rs (CPIndex/NodeSim/NodeSimMin/NeighborVec + `graph::should_prefetch/prefetch_mmap_vector`), serialize/{mod,bytes,file}.rs (CPIndex/HnswConfig/HnswNode/IndexBackend/ENTRY_POINT_NONE + `graph::cached_norms_for_metric`) |
| Callees | crate::index::{search::SearchProfile, distance::*, neighbor_index, ivf, scann}, crate::node::{DistanceMetric, FilterBitset, VectorRepresentations}, crate::config::PrefetchMode, crate::storage::vfile, dashmap, parking_lot, rand, ahash, smallvec, portable-atomic |
| Implicaciones | Contratos intactos (paths `crate::index::graph::*` preservados vía `pub use` en graph/mod.rs); comportamiento público idéntico (move byte-identical, `flat_threshold: Some(10000)` sin tocar); sin impacto perf/mem/serialización (cero cambio lógico); sin migración de datos; tests existentes deben pasar sin modificación salvo su ubicación |

**RIESGO:** bajo (refactor mecánico con precedente: `impl CPIndex` ya distribuido en search/, serialize/, stats.rs).

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0).

- **Archivos leídos (completos):** src/index/graph.rs (2031L — verificado vía Read 1-400 + grep estructural + codegraph; bytelen 75371), src/index/mod.rs (158L completo), src/index/neighbor_index.rs (514L, head 60L + estructura)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** ver Callees arriba; punto crítico: `use crate::index::search::SearchProfile` (L1, usado en insert L755/777/929/951) — el split NO crea graph/search.rs (colisionaría con src/index/search/ existente); el código de insert queda en graph/core.rs con el mismo import
- **Archivos que referencian a los editados (referencias entrantes):** ver Callers arriba (13 archivos); todos usan paths `crate::index::graph::X` o `graph::X` calificado — preservados por re-exports
- **Veredicto impacto:** BAJO — `src/index/graph.rs` → `src/index/graph/{mod,types,prefetch,core,tests}.rs`; ningún caller cambia; `mod.rs` (`pub use graph::*`) sigue resolviendo por glob-re-export

## Contrato

"cargo check -p vantadb 0 warnings + cargo nextest run --profile audit -p vantadb 0 failed + cargo clippy -p vantadb -- -D warnings 0 + ningún caller fuera de src/index/ roto (cost_estimator, lib, migration compilan igual)"

## Spec (SDD)

No requerida — Phase 1b: refactor puro, cero símbolos públicos nuevos (verificación: grep `^pub (fn|struct|enum|type|const)` en graph.rs solo lista items existentes que se mueven; mod.rs no gana exports).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `flat_threshold` default `Some(10000)` (planner depende); `VECTOR_INDEX_VERSION = 8` (migration.rs); orden total NaN de NodeSim/NodeSimMin (AUDREP-29); invariante reachability de shrink (last-inbound); paths públicos `crate::index::graph::{CPIndex, HnswConfig, HnswNode, NeighborVec, NodeSim/Min, IndexBackend, ENTRY_POINT_NONE, VECTOR_INDEX_VERSION, cached_norms_for_metric, random_layer_from_config, set_prefetch_mode, should_prefetch, prefetch/release_mmap_vector, total_cmp_sim, FreshHnswReport}`
- **Comandos de verificación:** `cargo check -p vantadb` / `cargo clippy -p vantadb -- -D warnings` / `cargo nextest run --profile audit -p vantadb` / `cargo fmt --check`
- **Deuda pendiente:** ninguna (al cerrar)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-48: Split src/index/graph.rs 2031L por concern |
| `lastAction` | RETRY fresco Nivel 2: Step 4 ejecutado — verify full verde + commit (ver Context Save Point) |
| `result` | OK |
| `nextAction` | ninguno (FIND-48 done; plan Task 1 sync a COMPLETED en worktree, commit del plan lo cierra el orquestador) |
| `contract` | ver ## Contrato + ## Invariantes de dominio |
| `nextTask` | FIND-49 (Wave0, disjunto) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda — refactor reduce deuda god-file sin introducir nueva. Saldo neto negativo (bien).

## Definition of Done

- Task: contrato ✅ + fmt/clippy/nextest capa determinista + tests del cambio (existentes reubicados) pasan
- Commit: atómico, `feat: FIND-48 — split graph por concern`, solo archivos de esta tarea
- Release: N/A (no release en este task; pre-push gate Regla 1 vía verify full)

## Herramientas necesarias

- cargo check/clippy/fmt/nextest (terminal)
- codegraph_explore (blast radius — hecho)

**Skills cargadas (SDP):** campaign-executor (base task-system) · source-driven-development (verificar patrones Rust oficiales si duda) · doubt-driven-development (gate review fallback, P2-01 sin agente distinto disponible) · incremental-implementation (slices con verify por step) · test-driven-development (RED N/A en refactor puro — tests existentes son el RED; GREEN = move sin romper) · context-engineering (context pack por slice) · api-and-interface-design (preservar boundaries `graph::*`)
SDP extra: grep SKILLS-MANIFEST.md → incremental-implementation, test-driven-development, context-engineering, code-simplification (descartada: el move no simplifica lógica; scope discipline). frontend-ui-engineering (sugerida por lifecycle, DESCARTADA: sin web/ en este task).

## Investigation Notes

- Plan decía "1846L / extraer graph/hnsw.rs, graph/search.rs, graph/serialize.rs" (medición backlog 2026-09-02). Realidad 2026-09-08: **2031L** y `src/index/search/` + `src/index/serialize/` **ya existen** como módulos extraídos. Plan-adjust intra-intento (mismo objetivo, cortes adaptados): `graph/{types,prefetch,core,tests}.rs` + `graph/mod.rs` re-exports. Sin graph/search.rs ni graph/serialize.rs (colisión de nombres).
- Precedente cross-module `impl CPIndex`: search/layer.rs:15, search/nearest.rs:12, serialize/bytes.rs:16, serialize/file.rs:12, stats.rs:24 — el split sigue el patrón establecido.
- Pre-mortem 1 (circulares): mitigado — Rust permite refs intra-crate; core.rs importa `crate::index::search::SearchProfile` igual que hoy; neighbor_index.rs sigue viendo `graph::NeighborVec` vía re-export.
- Pre-mortem 2 (tests huérfanos): mitigado — tests → graph/tests.rs con `use super::*` (= re-exports de mod.rs) + imports explícitos para nombres que antes venían de private-`use` del padre (BinaryHeap, etc.).
- Pre-mortem 3 (flat_threshold): sin tocar — move verbatim, verificado por tests existentes + Default impl intacta.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 1 step (Step 4) |
| % completado | 80% (split ejecutado + scoped verifies ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A justificado: move mecánico sin cambio lógico; sin trust boundaries, inputs, auth, deps, FFI ni red tocados. `unsafe` existentes (madvise L26-91) se mueven verbatim con sus `// SAFETY:` intactos.
- [x] **PERFORMANCE** — N/A bench justificado: cero cambio lógico (verificación = tests existentes, incl. hnsw recall en suite audit si aplica). Regla 9 no dispara (no es optimización).

## Steps

### Step 1: Mover módulos de código (types/prefetch/core + mod.rs, git rm graph.rs)
- **Archivos:** `src/index/graph.rs` → `src/index/graph/{mod.rs,types.rs,prefetch.rs,core.rs}`
- **Acción:** split mecánico por rangos de línea (script reproducible, contenido verbatim + imports por archivo + re-exports en mod.rs); `git rm src/index/graph.rs`
- **Verify:** `cargo check -p vantadb` 0 warnings/errors
- **Estado:** ✅ DONE (lib check verde; único warning ajeno parser/lexer.rs de FIND-50)

### Step 2: Mover tests a graph/tests.rs
- **Archivos:** `src/index/graph/tests.rs` (des-anidar `mod tests`, imports explícitos)
- **Acción:** mover L1239-2031; `use super::*` + imports para nombres ex-private-use (BinaryHeap et al. según feedback del compilador)
- **Verify:** `cargo check -p vantadb --tests` 0 warnings/errors
- **Estado:** ✅ DONE (0 errores graph; fixes: 2× pub(crate) bump + Ordering import; errores parser ajenos FIND-50)

### Step 3: fmt + clippy
- **Archivos:** `src/index/graph/*`
- **Acción:** `cargo fmt` (normaliza indent de tests des-anidados) + corregir warnings hasta 0
- **Verify:** `cargo fmt --check` 0 + `cargo clippy -p vantadb -- -D warnings` 0
- **Estado:** ✅ DONE (rustfmt scoped a graph/*.rs limpio; clippy full bloqueado por parser/sdk ajenos — re-verificar en Step 4)

### Step 4: Suite audit + commit
- **Archivos:** `src/index/graph/*`, `docs/tasks/FIND-48.md`, `docs/plans/2026-09-08-backlog.md` (sync Task 1)
- **Acción:** `cargo nextest run --profile audit -p vantadb` 0 failed → commit `feat: FIND-48 — split graph por concern` (solo archivos tarea) → sync plan file
- **Verify:** nextest 0 failed + `git log --oneline -1` muestra commit
- **Estado:** ✅ DONE (retry fresco 2026-09-08: `cargo check -p vantadb` exit 0 / 0 warnings · `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0 · `cargo nextest run --profile audit -p vantadb` 2145 passed / 0 failed / 1 skipped en 162.8s · `cargo fmt --check` scoped graph/* exit 0; bloqueo ajeno previo resuelto: FIND-49 ✅ COMPLETED e3711dea+72eb4c0a, parser compila en árbol de trabajo)

## Dependencias

- Wave0: FIND-49/FIND-50 disjuntos (src/sdk/, src/parser/ — NO tocar)
- Bloquea a: nada (Wave1 independiente)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (fallback adversarial — sin sub-agente distinto disponible)
- **Enfoque:** ¿re-exports cubren todo `graph::*`? ¿bump pub(crate) mínimo? ¿equivalencia real? → SÍ: 0 errores graph en lib+tests targets; script equivalencia 2031/2031 líneas contabilizadas, 0 missing; bumps solo 2 internos (random_layer, shrink_neighbors), API pública intacta.
- **Cómo se probó:** cargo check lib + --tests (0 graph errors), rustfmt scoped limpio, script find48_equiv.py (ACCOUNTING OK + VERBATIM OK). Full nextest/clippy pendiente por bloqueo ajeno.
- **Checklist anti-hábitos tóxicos:** [x] sin salidas inventadas (outputs citados verbatim) · [x] sin done prematuro (INCOMPLETO declarado) · [x] fallos reportados (parser/sdk colaterales) · [x] sin reintentos ciegos (1 retry con feedback: imports tests) · [x] scope respetado (solo src/index/graph/* + task file)
- **Veredicto:** ✅ approve con condición (Step 4 full-verify tras verde Wave0)

## Context Save Point

RETRY FRESCO 2026-09-08 (sesión RESUME falló por socket, trabajo intacto — no rehecho): Step 4 desbloqueado y ejecutado. Verify full del contrato en árbol de trabajo (con FIND-49 commiteado + FIND-50 en worktree): `cargo check -p vantadb` exit 0/0 warnings · `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0 · `cargo nextest run --profile audit -p vantadb` 2145 passed/0 failed/1 skipped (162.8s) · `cargo fmt --check` scoped graph/* exit 0. `campaign_verify_cmd` MCP devolvió `Budget exceeded` (elapsed 482min > 120max — bug/límite conocido, verificado por terminal directa como en sesiones previas). Commit SOLO scope propio (graph/* + task file, pathspec explícito; plan file NO incluido: contiene syncs ajenos FIND-49/FIND-50 sin commitear en worktree).

SPLIT EJECUTADO 2026-09-08: graph.rs (2031L) → graph/{mod.rs:12L, types.rs:~225L, prefetch.rs:~100L, core.rs:~920L, tests.rs:~790L}. Equivalence proof: 1995 body + 18 imports + 4 structural + 14 blanks = 2031/2031, verbatim missing=0. Scoped verifies verdes. Commit pendiente-condicional + Step 4 (nextest audit) BLOQUEADO por FIND-49 (sdk/types.rs) y FIND-50 (parser/mod.rs) en rojo — reanudar con `cargo nextest run --profile audit -p vantadb` cuando el árbol esté verde, luego commit ya realizado incluirá solo archivos propios.

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 1 (Step 4) |
| % completado | 80% |

## Notas

- Plan-adjust documentado en Investigation Notes (cortes adaptados, mismo objetivo).
- Regla 8 concurrencia: el PR toca índice + DashMap/parking_lot en código MOVIDO (sin cambio lógico) — auditoría chaos/review completa desproporcionada para move verbatim; mitigación: suite audit + `git diff --stat` debe mostrar solo renames/moves (verificable con `git diff -M --summary`).
