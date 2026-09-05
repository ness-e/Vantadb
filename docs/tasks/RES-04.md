# RES-04 — Semántica scores (src/api/scores, docs/api/scores) — scoring semántico

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave1 P38 — 20260902-alta-prioridad-paralelo)
- **Fuente:** `docs/research/archive/FND-06-core-bindings-boundaries.md` H3 (score→similarity drift) + `src/planner.rs` RRF_K + `src/index/distance/metrics.rs` cosine zero-norm + `docs/api/*` 0 hits scores/semántica
- **Esfuerzo:** 🟡 Media (docs + 1 helper core, ≤100 líneas, ponytail)
- **Prioridad:** Media (semántica oficial, drift zero-norm documentado FND-06)
- **Tipo:** Rust core SDK + docs (api-contract)
- **Turns estimados:** 2 (DISCOVERY+ EJECUCIÓN + VERIFY)
- **Creado:** 2026-09-02T23:00
- **last-synced:** 2026-09-02T23:30
- **Estado:** ✅ COMPLETED
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **Incógnitas (uphill):** 0 — scoring ya en planner.rs + metrics.rs, solo documentar + helper thin wrapper
- **Pendientes (downhill):** 0 — docs/api/scores.md + src/api/scores.rs + verify cargo check/test + plan sync ✅
- **Branch:** develop (disjoint GOV-A5/RES-03)

## Blast Radius
| Dirección | Módulos | Implicación |
|-----------|---------|-------------|
| **Callers de scores** | `src/planner.rs` (RRF_K, fuse_rrf), `src/index/distance/metrics.rs` (cosine_sim_f32, f32_l2_norm), `src/sdk/search/mod.rs` (ERR-028 zero-norm), `integrations/llamaindex` + `integrations/langchain` (`1.0 - s/2.0`), `vantadb-ts/src/vantadb.ts` (_buildSearchRequest zero-norm fallback) | Semántica de score es contrato público; cambio afecta híbrido search, adapters, y bindings TS/Python/WASM. |
| **Archivos clave nuevos** | `src/api/scores.rs` (helper thin wrapper, no lógica duplicada), `docs/api/scores.md` (semántica oficial) | Glue legítimo R-8: helper centraliza `cosine_distance ↔ similarity` y `RRF contribution`; docs cierran gap FND-06 H3. |
| **Disjoint Wave1c** | RES-03 toca `src/iql/` + `src/sdk/search/` (phrase/TextMatch), GOV-A5 toca registries live | 0 archivos en común — parallel seguro MAX 3. RES-04 solo toca `src/api/scores*` + `docs/api/scores*` + `src/api/mod.rs` wiring. |

**Disjoint garantizado:** no tocar `src/wal.rs`, `src/storage/engine/**`, `src/vector/`, `src/iql/` (RES-03), ni registry files (GOV-A5).

## Contrato (verificable — mecánico)
> Fuente plan 2026-09-02 + misión RES-04:
> `Select-String -Path "docs/api/scores.md" -Pattern "score semantics|RRF|BM25|cosine|zero-norm" | Measure-Object Count >=1`
> AND `Select-String -Path "src/api/scores.rs" -Pattern "cosine_distance|RRF|score" | Measure-Object Count >=1`
> AND `cargo check -p vantadb` exit 0
> AND `cargo test -p vantadb --lib` (o --workspace) pasa relevante

**Contrato canónico (este task file):**
```powershell
Select-String -Path "docs/api/scores.md" -Pattern "score semantics|RRF|BM25|cosine|zero-norm" | Measure-Object Count  # >=1
Select-String -Path "src/api/scores.rs" -Pattern "cosine_distance|rrf|RRF|score" | Measure-Object Count  # >=1
cargo check -p vantadb  # exit 0
cargo test -p vantadb --lib -- scores 2>&1 | Select-String "ok" | Measure-Object Count  # >=1 (si tests scores)
# Fallback si no hay test filter scores: cargo check ya es gate; cargo test -p vantadb general passa
```

## Herramientas necesarias
- `codegraph_explore "scores semántica"` — blast radius (DONE)
- `Read docs/api/SCORING|TS_SDK|EMBEDDED_SDK` — spec existente
- `cargo check -p vantadb` / `cargo test -p vantadb`
- `Select-String` / `Test-Path` — contrato mecánico

**Skills cargadas (SDP §2 — Lifecycle BUILD):**
| Skill | Por qué |
|-------|---------|
| `campaign-executor` | Orquestación plan/task/verify/state machine (Obligatorio) |
| `planning-and-task-breakdown` | Slicing vertical Wave1c |
| `writing-plans` | Spec → task breakdown |
| `ponytail(full)` | 1 helper thin wrapper vs duplicar lógica (1 guard) |
| `api-design-principles` | Contrato REST score semantics estable |
| `documentation-and-adrs` | Docs api oficial scores |
| `test-driven-development` | RED helper test si no existe |
| `incremental-implementation` | Thin vertical slice docs+helper+verify |

Total 8 ≤ 8 (SDP base 4 + 4 extras keywords scores/semantica/search/api).

**SKILLS_CARGADAS:** `campaign-executor`, `planning-and-task-breakdown`, `writing-plans`, `ponytail(full)`, `api-design-principles`, `documentation-and-adrs`, `test-driven-development`, `incremental-implementation`

## Spec (SDD)
N/A — semántica ya definida en `src/planner.rs` (RRF_K=60, `1/(k+rank)`) y `src/index/distance/metrics.rs` (cosine_sim_f32 zero-norm → 0.0, ERR-028 guard en sdk/search). Este task solo documenta contrato oficial en `docs/api/scores.md` y expone helper `src/api/scores.rs` para evitar duplicación adapters (`1.0 - s/2.0`). Sin cambios breaking.

## Invariantes de dominio (handoff — MUST)
- **RRF:** `score = Σ 1/(k+rank)` k=60 default, rank 1-indexed wire (debug.rs rank_map) ↔ 0-indexed planner; hybrid fusion server-side, no pesos intermedios (MEM-01 profile solo mode/rrf_k/candidate_k).
- **Cosine:** similarity ∈ [-1,1] (parallel →1, orthogonal →0, opposite → -1), distance ∈ [0,2] via `distance = 1 - similarity`? Core HNSW reporta similarity (higher-is-better `VantaMemorySearchHit.score`); MCP `search_semantic` convierte a distance `1-similarity` para wire; adapters usan `similarity = 1 - distance/2`. Documentar ambas.
- **Zero-norm:** ERR-028: zero-norm cosine query vector → `VantaError::InvalidInput` (core rechaza, no fallback silencioso). TS WASM workaround drift (FND-06 H1) documentado, no automatizar fallback sin spec.
- **BM25:** lexical scoring por término, normalizado por RRF fusión (no comparable directo con vector scores — RRF ignora scores, solo ranks).
- **No tocar:** wal/storage/vector hot paths; disjoint guard.

## Steps (atomic — vertical slice)

### Step 1: DISCOVERY — mapear scoring existente (DONE 2026-09-02)
- **Archivos:** `src/planner.rs:25 RRF_K`, `src/index/distance/metrics.rs`, `src/sdk/search/mod.rs:106 ERR-028`, `docs/api/TS_SDK.md:Distance vs Score`, `docs/research/archive/FND-06*`, `integrations/*/_cosine_sim`, `desktop/retrieval-core.ts:RRF_K`
- **Acción:** codegraph_explore "scores semántica" + Read scores files + grep docs/api 0 hits scores/semántica + map drift zero-norm + RRF contrib + BM25.
- **Hallazgos:**
  - RRF_K=60.0 planner.rs, fuse_rrf Many, contribución 1/(k+rank+1) 0-based
  - cosine_sim_f32 zero-norm → 0.0, con query_norm guard <EPSILON; sdk/search ERR-028 rechaza zero-norm cosine query → InvalidInput (AUDREP-55)
  - TS vantadb.ts fallback silencioso a Euclidean (drift H1) vs native no fallback → semántica divergente
  - docs/api 0 hits score semantics (gap FND-06 H3) — falta doc oficial
  - adapters duplican `1.0 - s/2.0` (H3) sin helper core
- **Verify:** codegraph_explore done + grep docs/api 0 hits confirmado
- **Estado:** ✅ COMPLETED

### Step 2: EJECUCIÓN — crear docs/api/scores.md + src/api/scores.rs (ponytail thin wrapper)
- **Archivos:** `docs/api/scores.md` (nuevo), `src/api/scores.rs` (nuevo), `src/api/mod.rs` (wiring pub mod)
- **Acción:** 
  - docs/api/scores.md: secciones RRF (fórmula + k=60 + rank 1-indexed wire), BM25, cosine distance vs similarity (tabla por binding), zero-norm ERR-028, helpers mapping, ejemplos.
  - src/api/scores.rs: ponytail thin wrapper — re-export RRF_K + helpers `cosine_distance_to_similarity`, `cosine_similarity_to_distance`, `rrf_contribution` (delegan a planner/metrics, no duplican SIMD), con doc comments y tests unitarios mínimos (DAMP).
  - src/api/mod.rs: `pub mod scores;` + doc.
  - Sin tocar RES-03 (iql) ni GOV-A5.
- **Verify:** `Select-String docs/api/scores.md RRF|BM25|cosine >=1` 32 ≥1 ✅ + `Select-String src/api/scores.rs cosine_distance|RRF >=1` 26 ≥1 ✅ + `cargo check -p vantadb` Finished ✅ + `cargo check --all-targets` Finished ✅
- **Estado:** ✅ COMPLETED (2026-09-02T23:30 — docs/api/scores.md 32 hits + src/api/scores.rs 26 hits + scores 4/4)

### Step 3: VERIFY + CIERRE — cargo test + plan sync + commit atómico
- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` RES-04, este task file, git
- **Acción:** `cargo test -p vantadb --lib` (o general) + `cargo check -p vantadb` + `cargo check -p vantadb --all-targets` si aplica; si verde → actualizar plan RES-04 ⬜→✅ + recitation + git commit `feat(search): RES-04 semántica scores — docs/api/scores + src/api/scores helper (ponytail)` en develop.
- **Verify:** `cargo test -p vantadb --lib api::scores` 4/4 ok ✅ + `cargo test --lib phrase` 18/18 disjoint ✅ + `Select-String plan RES-04.*✅` 1 ≥1 ✅ + `git log --oneline -1` muestra feat(search): RES-04 — pendiente commit atomico
- **Estado:** ✅ COMPLETED (verify done, plan sync done, commit next)

## Dependencias
- **Depende de:** RES-02 (Wave1 P38 quiesce) — ya ✅ 2026-09-02T22:00, no bloquea scores (disjoint); FND-06 reporte ya existe.
- **Paralelo seguro:** RES-03 (phrase iql) + GOV-A5 (registries) — 0 archivos overlap, MAX 3.
- **No bloquear:** siguiente Wave2 MEM-01 etc.

## Deuda técnica (Regla 6 — MUST)
- **Saldo neto 0:** RES-04 añade docs + 1 helper thin wrapper (≤100 líneas) que REDUCE deuda H3 (duplicación `1.0 - s/2.0` en 2 adapters) — helper centraliza fórmula, adapters pueden migrar follow-up (no scope creep aquí). No se introduce deuda nueva.
- **P2 conocida no tocada:** P2-8 collect_all_deduped O(n) wasm — fuera scope.
- **ponytail:** global comment si se deja simplificación: `// ponytail: helper delega a planner::RRF_K y metrics::cosine_sim_f32, sin SIMD duplicado — upgrade si se necesita batch vectorizado`

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — scores.md es doc-only, scores.rs es pure fn sin input trust boundary (f32 ops, no alloc), no unsafe, no FFI; validación zero-norm ya en sdk/search ERR-028.
- [x] **PERFORMANCE** — helpers son O(1) inline, no hot-path; no benchmark requerido (Regla 9 no aplica — no optimiza hot path, solo documenta + wrapper). Si se vectoriza luego, exigir `cargo bench --bench canonical_p99`.

## Notas
- **Ponytail:** 1 helper thin wrapper vs 50 líneas de lógica duplicada — reuse `crate::planner::RRF_K` y `crate::index::distance::*`.
- **Disjoint:** RES-04 solo toca `src/api/scores*` + `docs/api/scores*` — validado grep 0 hits previo, no contención con GOV-A5/RES-03.
- **Commit:** `feat(search): RES-04 semántica scores — docs/api/scores + src/api/scores helper (ponytail)` en develop, no main.

## Context Save Point
- **Fecha:** 2026-09-02T23:30
- **Branch:** develop
- **CI pendiente:** ninguno — `cargo check` + `cargo test api::scores 4/4` + `phrase 18/18` ✅
- **Decisiones:** helper thin wrapper delega a core existente, docs cierran gap FND-06 H3 sin breaking change; commit atomico feat(search): RES-04 pending
- **Problemas conocidos:** ninguno bloqueante; disjoint GOV-A5/RES-03 verificado
- **Próxima tarea:** RES-05/RES-06 siguen H3 follow-up si helper necesita exposición adapters — o Wave2 MEM-01

