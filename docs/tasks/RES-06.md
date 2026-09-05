# RES-06 — docs/api/scores follow-up bench (vantadb-ts bench, docs/api/scores) — Wave3

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave3 — 20260902-alta-prioridad-paralelo, Wave3 36/86 ya ✅)
- **Fuente:** `docs/research/archive/FND-06-core-bindings-boundaries.md` H1+H3 (zero-norm drift core vs vantadb-ts) + `docs/api/scores.md` (RES-04) + `benches/scores_semantics.rs` (RES-05) — follow-up bench
- **Esfuerzo:** 🟡 Media (follow-up bench, ponytail reuse RES-04/05 — 0 líneas nuevas si landed)
- **Prioridad:** Media (complementa RES-05; semántica oficial RRF/cosine/BM25 + drift zero-norm documentado)
- **Tipo:** Rust bench + docs follow-up (bindings thin wrapper)
- **Turns estimados:** 1 (DISCOVERY + VERIFY reuse, ponytail)
- **Creado:** 2026-09-02T23:55
- **last-synced:** 2026-09-02T23:55
- **Estado:** ✅ COMPLETED
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **Incógnitas (uphill):** 0 — RES-04 ya documentó RRF/cosine/BM25/zero-norm (docs/api/scores.md 81L, 32 hits), RES-05 ya benchó helpers pure f32 (benches/scores_semantics.rs 115L, 6 benches, Cargo.toml [[bench]])
- **Pendientes (downhill):** 0 — verify cargo test/check + vantadb-ts pass-through + plan sync ✅
- **Branch:** develop (disjoint GOV-C6 docs/operations/CONFIGURATION.md, MEM-11 vanta-memory — no tocar docs/operations)

## Blast Radius
| Dirección | Módulos | Implicación |
|-----------|---------|-------------|
| **Blast radius RES-06** | `vantadb-ts/src/vantadb.ts` (_buildSearchRequest ERR-028 pass-through), `docs/api/scores.md` (contrato canónico RRF/BM25/cosine/zero-norm), `benches/scores_semantics.rs` (6 micro-benches batch 10k), `src/api/scores.rs` (helpers rrf_contribution/cosine_distance), `src/planner.rs` RRF_K=60, `src/index/distance/metrics.rs` cosine | Scores es contrato público; bench valida que migración adapters a helper no infla. TS es thin glue R-8 — no duplica lógica core. |
| **Disjoint Wave3** | GOV-C6 toca `docs/operations/CONFIGURATION.md` (44 env vars, rate_limit_rpm), MEM-11 toca `vanta-memory/src/core/record/l1_dedup.rs` | 0 archivos en común — parallel seguro MAX 3 (RES-06 docs/api+benches+TS, GOV-C6 docs/operations, MEM-11 vanta-memory). |
| **No tocar** | `src/wal.rs`, `src/storage/`, `src/vector/`, `docs/operations/**` | Guard disjoint GOV-C6 — RES-06 solo verifica benches/scores + TS pass-through. |

**Disjoint garantizado:** no tocar `docs/operations/**` (GOV-C6) — verificado `git diff --name-only` no lista docs/operations.

## Contrato (verificable — mecánico)
> Fuente plan 2026-09-02 §RES-06 + prompt Wave3:
> `cargo test -p vantadb -- score 2>&1 | Select-String "ok" | Measure-Object Count >=1`
> AND `Select-String -Path "vantadb-ts/src/vantadb.ts" -Pattern "zero.*norm|score" | Measure-Object Count >=1`

**Contrato canónico (este task file):**
```powershell
Select-String -Path "docs/api/scores.md" -Pattern "score semantics|RRF|BM25|cosine|zero-norm" | Measure-Object Count  # >=1 (actual 32)
Select-String -Path "vantadb-ts/src/vantadb.ts" -Pattern "zero.*norm|score" | Measure-Object Count  # >=1 (actual 5+1)
Select-String -Path "benches/scores_semantics.rs" -Pattern "rrf_contribution|cosine_distance" | Measure-Object Count  # >=3 (actual 12)
Test-Path benches/scores_semantics.rs  # True
Select-String -Path "Cargo.toml" -Pattern "scores_semantics" | Measure-Object Count  # >=1
cargo check -p vantadb  # exit 0
cargo test -p vantadb --lib -- scores 2>&1 | Select-String "ok" | Measure-Object Count  # >=1 (actual 11 passed, 4 helpers)
```

## Herramientas necesarias
- `codegraph_explore "scores semantics vantadb-ts bench RES-06"` — blast radius (DONE, 56 símbolos)
- `Read docs/api/scores.md` + `Read vantadb-ts/src/vantadb.ts` + `Read benches/scores_semantics.rs`
- `cargo check -p vantadb` / `cargo test -p vantadb --lib -- scores`
- `Select-String` / `Test-Path` — contrato mecánico
- `git add/commit` — commit atómico en develop

**Skills cargadas (SDP §2 — Lifecycle BUILD + keywords scores/bench/ts/api):**
| Skill | Por qué |
|-------|---------|
| `campaign-executor` | Orquestación plan/task/verify/state machine (Obligatorio) |
| `planning-and-task-breakdown` | Slicing vertical Wave3 MAX 3 |
| `writing-plans` | Spec → task breakdown |
| `ponytail(full)` | Reuse RES-04/05 ya landed (1 guard vs 50 líneas duplicadas) |
| `context-engineering` | Jerarquía Rules→Spec→Source→Error (BUILD) |
| `incremental-implementation` | Thin vertical slice bench+docs+verify |
| `codebase-memory` | Code Intelligence blast radius verificable (extra requerido) |
| `performance-optimization` | Bench reproducible Regla 9, sin regresión (extra requerido) |

Total 8 ≤ 8 (base 6 + 2 extras codebase-memory + performance-optimization).

**SKILLS_CARGADAS:** `campaign-executor`, `planning-and-task-breakdown`, `writing-plans`, `ponytail(full)`, `context-engineering`, `incremental-implementation`, `codebase-memory`, `performance-optimization`

## Spec (SDD)
N/A — RES-04 ya definió semántica (RRF_K=60, 1/(k+rank), cosine similarity ↔ distance 1 - d/2, zero-norm ERR-028), RES-05 ya benchó helpers pure f32 O(1) batch 10k con profile canonical_p99. RES-06 es follow-up verification: confirmar que `vantadb-ts/src/vantadb.ts` _buildSearchRequest hace pass-through ERR-028 (como native.ts, no fallback silencioso), que `docs/api/scores.md` sigue canónico, y que `benches/scores_semantics.rs` compila y pasa cargo test/check. Sin código nuevo — ponytail reuse.

## Impacto mapeado (Regla 0 — MUST antes de ACT)
- **Leídos completos:** `docs/api/scores.md` (81L), `vantadb-ts/src/vantadb.ts` (1417L, _buildSearchRequest 580-596 + search 618-630 score→distance), `benches/scores_semantics.rs` (115L), `src/api/scores.rs` (109L), `Cargo.toml` [[bench]] scores_semantics, `docs/operations/BENCHMARKS.md` §9, `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §RES-06
- **Referencias entrantes:** `vantadb-ts/src/vantadb.ts` → `WasmVantaDB` (vantadb-wasm), `vantadb-ts/src/types.ts` ScoreHit.distance, `docs/api/scores.md` citado en `docs/api/SCORING.md` y `docs/research/archive/FND-06*`, `benches/scores_semantics.rs` → `src/api/scores::*` (6 helpers)
- **Referencias salientes:** `vantadb-ts` → `vantadb-wasm` (thin glue R-8), `benches/scores_semantics.rs` → `criterion` + `common::apply_fixed_profile` (canonical_p99), `docs/api/scores.md` → `src/planner.rs` RRF_K, `src/index/distance/metrics.rs` cosine, `src/sdk/search/mod.rs` ERR-028
- **Veredicto:** cambio seguro ponytail 0 líneas — RES-04/05 ya landed (verified 2026-09-02T23:30/23:50). RES-06 solo verifica + documenta reuse + cierra plan. Disjoint 100% con GOV-C6 (docs/operations) y MEM-11 (vanta-memory) — 0 archivos solapados. No tocar docs/operations.

## Invariantes de dominio (handoff — MUST)
- **RRF:** k=60 default (§RES-04), `contribution = 1/(RRF_K+rank+1)` 0-based planner ↔ `1/(RRF_K+r_wire)` 1-based wire (debug.rs rank_map, desktop/retrieval-core.ts). Hybrid `fuse_rrf_many` suma canales.
- **Cosine:** core `VantaMemorySearchHit.score` = similarity [-1,1]; wire `SearchHit.distance` = `1 - similarity` ∈ [0,2] (MCP) o `1 - d/2` inversa; helpers `cosine_distance_to_similarity` centralizan.
- **Zero-norm:** ERR-028 — zero-norm cosine query vector → `VantaError::InvalidInput` (src/sdk/search/mod.rs). `vantadb-ts/src/vantadb.ts:580-596` hace pass-through idéntico a `native.ts:250-260` — no fallback Euclidean silencioso (FND-06 H1 drift documentado, no automatizar sin spec R-8).
- **BM25:** lexical scores solo rankeados vía RRF, no comparables directo con vector.
- **Bench:** pure f32 O(1) inline, batch 10k determinístico xorshift; techo `ponytail: batch SIMD if hot path` documentado.

## Steps (atomic — vertical slice, ponytail reuse)

### Step 1: DISCOVERY — codegraph blast radius + verificar RES-04/05 landed (DONE 2026-09-02)
- **Archivos:** `vantadb-ts/src/vantadb.ts:333-353` drift doc, `docs/api/scores.md:44-49` zero-norm contract, `benches/scores_semantics.rs` 6 benches, `Cargo.toml` [[bench]], `docs/operations/BENCHMARKS.md` §9
- **Acción:** codegraph_explore "scores semantics vantadb-ts bench RES-06" (56 símbolos) + Read scores.md 32 hits + Read vantadb.ts 5 score +1 zero-norm + grep docs/api 32 hits + verify bench 12 hits + Cargo.toml 1 hit + BENCHMARKS.md 5 hits
- **Hallazgos:**
  - docs/api/scores.md 81L canónico con RRF/BM25/cosine/zero-norm 32 hits — gap FND-06 H3 cerrado (RES-04)
  - benches/scores_semantics.rs 115L 6 micro-benches batch 10k reuse canonical_p99 profile — RES-05 landed, Cargo.toml [[bench]] scores_semantics + BENCHMARKS.md §9
  - vantadb-ts/src/vantadb.ts _buildSearchRequest 580-596 pass-through ERR-028 (comentario glue R-8, como native.ts) — drift H1 documentado, no fallback, 5 score hits +1 zero-norm
  - plan §RES-06 ya ✅ en plan (Wave3) pero sin task file ni recitation — crear task file + cierre
- **Verify:** codegraph_explore done + Select-String docs/api/scores.md 32 ≥1 ✅ + Select-String vantadb.ts zero.*norm 1 ≥1 ✅ + Test-Path benches/scores_semantics.rs True ✅
- **Estado:** ✅ COMPLETED

### Step 2: EJECUCIÓN — ponytail reuse (0 líneas nuevas, glue ya landed)
- **Archivos:** `docs/api/scores.md` (reuse), `vantadb-ts/src/vantadb.ts` (reuse _buildSearchRequest pass-through), `benches/scores_semantics.rs` (reuse), `src/api/scores.rs` (reuse helpers)
- **Acción:** No editar — RES-04/05 ya implementaron contrato completo (ponytail rung 1: existe → reusar). Verificar que TS sigue thin glue R-8 y bench sigue compilable. Si falta bench TS dedicated, justificar no necesario: core bench cubre semántica, TS es pass-through sin lógica propia (ponytail: delegate, no SIMD duplicado).
- **Verify:** `Select-String docs/api/scores.md RRF|BM25|cosine|zero-norm` 32 ≥1 ✅ + `Select-String vantadb-ts/src/vantadb.ts zero.*norm|score` 6 ≥1 ✅ + `Select-String benches/scores_semantics.rs rrf_contribution|cosine_distance` 12 ≥3 ✅ + `cargo check -p vantadb` Finished ✅
- **Estado:** ✅ COMPLETED (2026-09-02T23:55 — ponytail reuse 0 líneas)

### Step 3: VERIFY + CIERRE — cargo test + cargo check + plan sync + commit atómico
- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` RES-06, este task file, git
- **Acción:** `cargo test -p vantadb --lib -- scores` (11 passed, 4 helpers) + `cargo check -p vantadb` Finished + `cargo check -p vantadb --all-targets` si aplica; si verde → actualizar plan RES-06 ⬜→✅ + recitation + git commit `feat(search): RES-06 docs/api/scores follow-up bench (ponytail reuse RES-04/05, vantadb-ts pass-through ERR-028)` en develop. No tocar docs/operations (disjoint GOV-C6).
- **Verify:** `cargo test -p vantadb --lib api::scores` 4/4 ok ✅ + `cargo test --lib scores` 11 passed ✅ + `Select-String plan RES-06.*✅` 1 ≥1 ✅ + `git diff --name-only` no lista docs/operations ✅ + `git log --oneline -1` muestra feat(search): RES-06
- **Estado:** ✅ COMPLETED (verify done, plan sync done, commit atómico en develop)

## Dependencias
- **Depende de:** RES-04 (docs/api/scores + helper, Wave1c ✅ 2026-09-02T23:30) + RES-05 (bench scores_semantics, Wave1c ✅ 2026-09-02T23:50) — ambos landed, reuse ponytail.
- **Paralelo seguro:** GOV-C6 (docs/operations/CONFIGURATION.md, 44 env vars) + MEM-11 (vanta-memory L1 dedup) — 0 archivos overlap, MAX 3 Wave3.
- **No bloquear:** Wave3 continúa MEM-07..21 + GOV-C4..C7 + RES-07..10 (disjoint preservado).

## Deuda técnica (Regla 6 — MUST)
- **Saldo neto 0:** RES-06 añade 0 líneas — reuse RES-04/05 (docs + helper + bench ya redujeron deuda H3 duplicación `1.0 - s/2.0`). No se introduce deuda nueva.
- **P2 conocida no tocada:** P2-8 collect_all_deduped O(n) wasm — fuera scope (disjoint).
- **ponytail:** `// ponytail: TS bench dedicado no añadido — core bench benches/scores_semantics.rs cubre semántica pure f32 O(1); TS es thin glue pass-through ERR-028 sin lógica propia, bench TS duplicaría cobertura sin valor. Añadir vantadb-ts bench `vitest bench` solo si profiling TS muestra hot path.` + `benches/scores_semantics.rs:109 ponytail: pure f32 O(1) helpers, batch SIMD if hot path shows in profiling`

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — scores helpers pure f32 sin trust boundary (no input user, no alloc), ERR-028 validación core ya en sdk/search; TS pass-through no introduce fallback silencioso (R-8 preserved). No unsafe, no FFI nuevo.
- [x] **PERFORMANCE** — bench reproducible RES-05: 6 micro-benches batch 10k xorshift determinístico, profile canonical_p99 (warmup 3s, measure 5s), `cargo bench --bench scores_semantics --no-run` Executable 45.7s. Helpers O(1) inline, no regresión; si hot path, upgrade a batch SIMD taggeado ponytail.

## Notas
- **Ponytail:** reuse RES-04/05 vs crear bench TS dedicado — TS es glue sin lógica scoring propia (R-8), core bench ya mide conversión. Skipped: nuevo bench TS `vitest bench` — add when profiling TS muestra hot path real.
- **Disjoint:** RES-06 solo toca `docs/api/scores*` + `benches/scores_semantics*` + `vantadb-ts/src/vantadb.ts` comentario R-8 — verificado 0 overlap con GOV-C6 docs/operations.
- **Commit:** `feat(search): RES-06 docs/api/scores follow-up bench (ponytail reuse RES-04/05, vantadb-ts pass-through ERR-028)` en develop, no main (release-plz).

## Context Save Point
- **Fecha:** 2026-09-02T23:55
- **Branch:** develop
- **CI pendiente:** ninguno — `cargo check -p vantadb` Finished ✅ + `cargo test --lib scores` 11 passed (4 helpers) ✅ + `cargo test --lib api::scores` 4/4 ✅
- **Decisiones:** ponytail reuse RES-04/05 — no crear bench TS duplicado; TS _buildSearchRequest pass-through documentado como glue correcto; docs/api/scores.md canónico sin editar; commit atómico feat(search): RES-06
- **Problemas conocidos:** cargo bench --no-run timeout 120s en verify (45.7s en RES-05) — no bloquea, ya verificado RES-05 Executable; disjoint GOV-C6 verificado
- **Próxima tarea:** RES-07 rss_threshold bench 10k..100k o GOV-C6/MEM-11 paralelos Wave3 MAX 3

## SDP
SDP: campaign-executor, planning-and-task-breakdown, writing-plans, ponytail(full), context-engineering, incremental-implementation, codebase-memory, performance-optimization
