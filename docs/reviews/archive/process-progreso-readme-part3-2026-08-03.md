# Audit Report: `docs/progreso/README.md` — Lines 2201–3320

**Date:** 2026-08-03
**Scope:** Lines 2201–3320 (end of file, 3320 total lines)
**Section audited:** "Tareas Completadas (Migradas desde Backlog)" (H2 header starts before line 2201; the section runs to EOF)
**Method:** Read in 4 blocks (2201–2500, 2501–2800, 2801–3100, 3101–3320). No other lines read. No edits made — audit only.

---

## 1. Structure — sections and start lines

No H2 headers occur inside the audited range (the parent H2 "Tareas Completadas (Migradas desde Backlog)" begins above line 2201). The range contains 85 H3 session/entry headers, 9 H4 category headers inside the Fleet Fix Session, and 2 bare `---` separators.

### H3 entry/session headers (85)

| Line | Header |
|------|--------|
| 2205 | `### CODE-020: CSP Hardening — Remove unsafe-inline from script-src` |
| 2220 | `### CODE-021: DOMPurify Sanitization on Blog dangerouslySetInnerHTML` |
| 2232 | `### CODE-001: WAL replay no escribe backend metadata — FIXED` |
| 2245 | `### CODE-009: save_vector_index() traga errores de persistencia — FIXED` |
| 2258 | `### CODE-003: Reemplazar process::exit(1) con graceful shutdown + WAL flush` |
| 2275 | `### CODE-002: WAL append antes de validación — FIXED` |
| 2282 | `### CODE-015: search_batch deadlock por GIL — FIXED` |
| 2289 | `### CODE-049: Focus trapping en drawer mobile — FIXED` |
| 2296 | `### CODE-052: marked.parse() en import time — FIXED` |
| 2303 | `### CODE-079: VERCEL_TOKEN expuesto en CLI — FIXED` |
| 2310 | `### CODE-012: Path traversal en Python SDK export/import/constructor — FIXED` |
| 2319 | `### CODE-026: BFS order vacío destruye DB en compact — FIXED` |
| 2326 | `### CODE-011: 100% errores Rust → PyRuntimeError — FIXED` |
| 2334 | `### CODE-018: expect() panic en serialización WASM vectors NaN/Inf — FIXED` |
| 2342 | `### CODE-019: TS close() llama free() no close() del Rust — FIXED` |
| 2350 | `### CODE-005: WASM delete_file() nunca maneja NotFoundError — FIXED` |
| 2357 | `### DOC-12: Update llms.txt Version Ranges` |
| 2365 | `### MKT-07 / BIZ-03: Pricing Page Multi-Tier Implementation` |
| 2376 | `### WEB-08-Refinement: Index Refinements & Anti-AI-Slop Cleanups` |
| 2388 | `### CI-01: Fix All GitHub Actions Workflows` |
| 2410 | `### Batch 4 — Fase 3: Documentación + Frontend (DOC-06/13/14/15/17/18/19, WEB-06/07/17/18/19/20/21)` |
| 2431 | `### Batch 5 — Fase 4: Release Engineering + Database Evolution (REL-01, LEG-02, DB-01/03/04, DEVOPS-08/09, DOC-16, BIZ-01)` |
| 2447 | `### 2026-07-04 — Fleet Fix Session (78 CODE bugs fixed across 9 commits)` |
| 2546 | `### 2026-07-06 — Wave 1-4 Completion: Quick Wins, Performance, Benchmarks & Cleanup (10 tareas movidas a progreso)` |
| 2562 | `### 2026-07-11 — Wave 1-5: Migración u64→u128 (CODE-067)` |
| 2581 | `### 2026-07-06 — Post-Benchmark Deep Investigations (4 paralelas, 25 tareas agregadas al backlog)` |
| 2602 | `### 2026-07-07 — Wave 1-6: CODE-055, Test Fixes, Migration Runner (5 tasks)` |
| 2618 | `### 2026-07-07 — Wave 1-7: Bugfixes & Optimizations (5 tasks)` |
| 2634 | `### 2026-07-07 — Phase 2: SIMD, HNSW Diversity & Python SDK Optimizations (5 tasks across 3 tracks)` |
| 2650 | `### 2026-07-07 — Wave 8: Python SDK, Distance, Async & Tooling (14 tasks)` |
| 2675 | `### 2026-07-07 — Phase 5: Governance, Encryption, WAL Shipping, PITR, WASM, Docs (9 tasks)` |
| 2695 | `### 2026-07-07 — PERF-17/18/19/20: HNSW params, WAL batch, Storage batch` |
| 2706 | `### 2026-07-13 — P1/P2/P3: HNSW micro-batching + WAL contention + ACID Phase 1` |
| 2716 | `### 2026-07-13 — Review Item 1: Clippy warnings cleanup` |
| 2724 | `### 2026-07-13 — P4: VantaFile reversible writes` |
| 2732 | `### 2026-07-23 — REV-003: Coverage campaign 53.85% → 80.55% (CII Silver)` |
| 2752 | `### 2026-07-14 — REV-004: tantivy rlib fix in vantadb-openai` |
| 2760 | `### 2026-07-14 — REV-005: Fix 6x no-explicit-any + prettier in web frontend` |
| 2768 | `### 2026-07-14 — REV-016: Audit vantadb-enterprise premature abstraction` |
| 2776 | `### 2026-07-14 — REV-017: Fix why-vantadb.tsx prettier trailing newline` |
| 2784 | `### 2026-07-14 — REV-015: Fix remaining 2x no-explicit-any in demo.lazy.tsx` |
| 2792 | `### 2026-07-14 — REV-008: Update actions/checkout + setup-node to v4` |
| 2800 | `### 2026-07-14 — REV-006: Workspace-level clippy in CI` |
| 2808 | `### 2026-07-14 — REV-007: reducedMotion in useEffect deps (3 components)` |
| 2816 | `### 2026-07-14 — INT-01: Publish LangChain adapter to PyPI` |
| 2824 | `### 2026-07-14 — INT-02: Publish LlamaIndex adapter to PyPI` |
| 2832 | `### 2026-07-14 — DEVOPS-05: Unified CI pipeline for adapter PyPI publishing` |
| 2840 | `### 2026-07-14 — REL-02: Publish vantadb-ts to npm (WASM build)` |
| 2850 | `### 2026-07-17 — P1-2: Windows test step timeout 25→30 min` |
| 2858 | `### 2026-07-17 — P1-3: Cache key hashFiles para GloVe dataset` |
| 2866 | `### 2026-07-17 — P1-4: macOS unificar con rust-setup action` |
| 2874 | `### 2026-07-17 — P1-5: Re-activar wasm-opt` |
| 2882 | `### 2026-07-17 — P1-6: Worker timeout 5s sin retry — exponential backoff` |
| 2890 | `### 2026-07-17 — P1-7: CI — Version extraction frágil con grep` |
| 2898 | `### 2026-07-17 — P1-8: CI — Inconsistencia de timeouts` |
| 2906 | `### 2026-07-17 — P1-9: WASM — SIMD duplicado eliminado` |
| 2914 | `### 2026-07-17 — P1-10: PyPI CDN propagation sleep → retry loop` |
| 2922 | `### 2026-07-17 — P2-1: OpfsFile::delete() implementado` |
| 2930 | `### 2026-07-17 — P2-2: VantaVector.__array_interface__ UB fix (Vec→Box)` |
| 2936 | `### 2026-07-21 — DOC-API Audit Fixes (6/6 tasks completadas)` |
| 2962 | `### 2026-07-23 — Batch TIER 1 Gate Check (8 tasks completadas)` |
| 2985 | `### 2026-07-24 — DRV-005: SDK unit tests para search/mod.rs` |
| 2995 | `### 2026-07-24 — Pipeline Run: 12 tasks completadas` |
| 3018 | `### 2026-07-24 — Backlog Audit: 4 resolved items moved to progreso` |
| 3031 | `### 2026-07-25 — Phase 0 Release Blockers: 3 completadas, 1 deferida` |
| 3044 | `### 2026-07-25 — P4 Engineering Health Wave 0: DOC-20 (mdBook docs site)` |
| 3054 | `### 2026-07-25 — P4 Engineering Health Wave 0: WEB-03 (Async WAL batching fsyncs)` |
| 3064 | `### 2026-07-25 — P4 Engineering Health Wave 0: VFY-004 (flat.rs O(n²) comment-only)` |
| 3074 | `### 2026-07-25 — P4 Engineering Health Wave 0: WEB-04 (Storage format versioning)` |
| 3084 | `### 2026-07-25 — P4 Engineering Health Wave 0: DRV-121 (Planner CBO optimization)` |
| 3094 | `### 2026-07-25 — P4 Engineering Health Wave 0: DRV-123 (Auto-embedding INSERT polish)` |
| 3104 | `### 2026-07-26 — P4 Engineering Health Wave 0: VFY-011 (ACID Phase 3 — MVCC/Snapshot Isolation)` |
| 3114 | `### 2026-07-26 — DRV-122: IQL JOINs, Subqueries, and SQL Compatibility` |
| 3124 | `### 2026-07-26 — Phase 7: NUEVO-13 HNSW ef_search auto-tuning` |
| 3134 | `### 2026-07-26 — DRV-131: Missing Index Types Beyond HNSW — IVF Flat` |
| 3146 | `### 2026-07-26 — P2 Backlog Housekeeping: DRV-041, VFY-006, VFY-007 ✅` |
| 3158 | `### 2026-07-26 — P8 Post-Launch & Enterprise: CLI-01, DEVOPS-HOMEBREW, DEVOPS-PY313, DEVEX-DEMO, DEVEX-EXAMPLES ✅` |
| 3172 | `### 2026-07-26 — Backlog Cleanup: P0–P4, P7, P9–P10 — 53 items moved to progreso` |
| 3191 | `### 2026-07-27 — P5/P6/P8 Quick Wins: 8 tareas ejecutadas` |
| 3220 | `### 2026-07-27 — COMP-010: Auto-embedding function abstraction` |
| 3230 | `### 2026-07-27 — COMP-008: Pluggable Index Engine (VecIndex Trait)` |
| 3242 | `### 2026-07-27 — COMP-014: FreshHNSW (Background Repair de Enlaces Huérfanos)` |
| 3248 | `### 2026-07-27 — COMP-013: Segment Optimizer Pipeline (Vacuum/Merge/Index)` |
| 3258 | `### 2026-07-27 — COMP-009: Binary Bulk Import` |
| 3270 | `### 2026-07-28 — ECO-001: Remove dead Claude Code hooks` |
| 3280 | `### 2026-07-28 — ECO-002: Fix --no-verify contradiction in AGENTS.md` |
| 3290 | `### 2026-08-02 — INV-017: sccache en CI — investigación` |
| 3300 | `### 2026-08-02 — GH-143: Acelerar CI con sccache y paralelización` |
| 3310 | `### 2026-08-02 — ENT-04: Connection pooling + circuit breaker (server-mode)` |

### H4 category headers inside Fleet Fix Session (2447)

| Line | Header |
|------|--------|
| 2451 | `#### Python SDK (9 bugs)` |
| 2464 | `#### Core Engine & Index (8 bugs)` |
| 2476 | `#### Rust Code Health (4 bugs)` |
| 2484 | `#### Security & Dependencies (7 bugs)` |
| 2495 | `#### TypeScript SDK (9 bugs)` |
| 2508 | `#### WASM & Build (4 bugs)` |
| 2516 | `#### CI & Infra (6 bugs)` |
| 2526 | `#### Web Frontend (10 bugs)` |
| 2540 | `#### Documentation (2 tasks)` |

### Structural blocks (in file order)

1. **2201–2445** — Individual/batch entries migrated from Backlog, per-task template (CODE-020 → CI-01, Batch 4, Batch 5). Dates 07-04 → 07-02 → 07-03 (not monotonic).
2. **2447–2544** — Fleet Fix Session (2026-07-04): tables by category, `ID | Tarea | Commit`.
3. **2546–2704** — Wave/Phase sessions (07-06 → 07-11 → 07-06 → 07-07): `ID | Tarea | Verificación` and `ID | Tarea | Files | Cambios` tables.
4. **2706–2730** — P1–P4 sessions (07-13): `ID | Tarea | Cambio | Estado`.
5. **2732–2848** — REV/INT/REL sessions (07-23/07-14): `ID | Tarea | Cambio | Estado`.
6. **2850–2934** — P1/P2 sessions (07-17).
7. **2936–3016** — DOC-API (07-21), TIER 1 Gate Check (07-23), DRV-005 (07-24), Pipeline Run (07-24).
8. **3018–3189** — Backlog audits & cleanup (07-24 → 07-26).
9. **3191–3266** — Quick wins & COMP sessions (07-27). Separator `---` at 3218 and 3268.
10. **3270–3319** — ECO/GH/INV/ENT (07-28 → 08-02).

---

## 2. Format

### Findings table

| Line(s) | Problem | Severity |
|---------|---------|----------|
| 2205–2445 | **Per-task template** (`- **Fecha:**`, `- **Objetivo:**`, `- **Checklist:**` `[x]`, `- **Archivos Modificados:**`, `- **Ids:**`) used for individual entries; **session template** (H3 date-title + table + `**Verificación:**` + `**Backlog actualizado:**`) used from 2546 onward. Two incompatible schemas coexist for the same kind of record. | alta |
| 2357, 2365, 2376, 2388 | Entries `DOC-12`, `MKT-07/BIZ-03`, `WEB-08-Refinement`, `CI-01` lack the `- **Ids:**` line that every sibling entry (2205–2355) has. ID only in the H3 title. | media |
| 2452–2544 | Fleet Fix Session tables use `ID | Tarea | Commit`; but the last category (2540–2544, Documentation) uses only `ID | Tarea` — **Commit column dropped mid-session**. | media |
| 2587–2593 | Findings table `Área | Hallazgos | IDs asignados` — a different table schema (backlog assignments, not completions) inside a "progreso" section. | media |
| 3176–3185 | Backlog Cleanup uses `Fase | Acción | Items` — no per-ID rows, only lists. Different schema again. | baja |
| 3244 | COMP-014 table header is `Tarea | Logro | Archivos | Estado`; every other session table is `ID | Tarea | ... | Resultado/Estado`. Column rename mid-file. | baja |
| 3206 | Row with `—` as ID ("Good first issues (18 open)") — ID-less row breaks the `ID | Tarea | Resultado` contract. | media |
| throughout | **Language mix ES/EN**: narrative mostly ES, but whole entries in EN (CODE-021, REV-004/005/006/007/008/015/016/017, INT-01/02, DEVOPS-05, REL-02, WEB-03/04, DRV-121/122/123, VFY-011, NUEVO-13, DRV-131, COMP-008/009/010/013/014, ENT-04, INV-017, GH-143), and intra-entry mixes (e.g. 2247 "save_vector_index() retornaba `()`, no `Result`... Se agregaron llamadas" — ES prose with EN labels). Violates "English as source of truth" guideline. | media |
| throughout | Dates are ISO `YYYY-MM-DD` and consistent in format; **but chronological order is broken** (see §6). | media |
| 2449 | `**Commits:**` line lists 10 hashes while the H3 (2447) claims "9 commits" — count mismatch. | media |
| 2600/2616/2632/2648/2673/2693/2704 | `**Backlog actualizado:**` counters: 98 → 88 → 83 → 79 → 79 → 79 → 79. Stuck at 79 across 4 consecutive sessions (2673, 2693, 2704) despite each completing work. | media |
| 3170 | Backlog counter "Total 90→95" — *increases* after a completion session; unexplained direction. | baja |
| 2310–2317, 2319–2324 | CODE-012/026 entries have checklists but no verification line/commit; contrast with siblings that cite `cargo test` or commit. | baja |
| 3218, 3268 | Two stray `---` separators used inconsistently (only between COMP-010/COMP-008 and COMP-009/ECO-001). | baja |

---

## 3. Duplicates (same task ID appearing more than once)

| ID | Lines | Notes |
|----|-------|-------|
| DOC-19 | 2414, 2614, 2630 | 3 occurrences: Batch 4 (6 ADRs), Wave 1-6 (ARCHITECTURE.md v0.2.0), Wave 1-7 (same work repeated) |
| DB-01 | 2437, 2612 | Batch 5 entry + Wave 1-6 row |
| CODE-067 | 2560, 2562 | Free-standing note inside Wave 1-4 (07-06) + full section 07-11 |
| INT-01 | 2820, 3001 | 07-14 publish section + 07-24 Pipeline Run row |
| INT-02 | 2828, 3002 | 07-14 publish section + 07-24 Pipeline Run row |
| DOC-20 | 2688, 3050 | Phase 5 (LanceDB guide) + P4 Wave 0 (mdBook) — **different tasks, same ID** (see §4) |
| DEVOPS-10 | 3040, 3178 | Phase 0 "DEFERIDO" + Backlog Cleanup "removido" — **conflicting states** (see §4) |
| DEVOPS-14 | 3008, 3178 | Pipeline Run "Ya existía, usado por 5 workflows" + Cleanup "removidos" — **conflicting states** |
| DEVOPS-15 | 3037, 3178, 3189 | WONTFIX ×3 (consistent, but triplicated) |
| DRV-005 | 2983, 2985 | Row inside TIER 1 Gate Check table + dedicated 07-24 section (same 18 tests) |
| DRV-041 | 3152, 3180 | Housekeeping section + Cleanup P2 list |
| VFY-006 | 3153, 3180 | Housekeeping section + Cleanup P2 list |
| VFY-007 | 3154, 3180 | Housekeeping section + Cleanup P2 list |
| WEB-03 | 3060, 3182 | P4 Wave 0 + Cleanup P4 list |
| WEB-04 | 3080, 3182 | P4 Wave 0 + Cleanup P4 list |
| VFY-004 | 3070, 3182 | P4 Wave 0 + Cleanup P4 list |
| VFY-011 | 3110, 3182 | P4 Wave 0 + Cleanup P4 list |
| DRV-121 | 3090, 3182 | P4 Wave 0 + Cleanup P4 list |
| DRV-122 | 3120, 3182 | P4 Wave 0 + Cleanup P4 list |
| DRV-123 | 3100, 3182 | P4 Wave 0 + Cleanup P4 list |
| DRV-131 | 3140, 3144, 3182 | Section + `Ids:` line + Cleanup P4 list |
| NUEVO-13 | 3130, 3183 | Section + Cleanup P7 list |
| COMP-009 | 3185, 3258 | Listed in Cleanup P10 + dedicated 07-27 section |
| PERF-15…38 (PERF-15/16/17/18/19/20/21/22/24/25/26/27/28/29/30/31/32/33/34/35/36/37/38) | 2587–2593 (assignment table) vs 2640–2702 (completion sections) | Structural reference pattern (assigned → later completed). Lower severity, but every PERF ID appears twice in the same section. |

---

## 4. Contradictions (same ID, different state/description)

| ID | Lines | Conflict | Severity |
|----|-------|----------|----------|
| DOC-20 | 2688 vs 3050 | 2688 (Phase 5, 07-07): "LanceDB migration guide — `docs/tutorials/migration-from-lancedb.md`". 3050 (07-25): "mdBook adoption for docs site". **Two unrelated tasks under one ID.** | alta |
| DEVOPS-14 | 3008 vs 3178 | 3008 (07-24): "Composite action Rust setup — ✅ Ya existía, usado por 5 workflows". 3178 (07-26): "DEVOPS-10/12/14 removidos" as stale. Verified-exists then deleted-as-stale one day later. | alta |
| DEVOPS-10 | 3040 vs 3178 | 3040 (07-25): "🔵 DEFERIDO (ponytail)… agregar Azure Trusted Signing cuando release lo requiera". 3178 (07-26): "DEVOPS-10… removidos" (stale). Deferred ≠ removed. | alta |
| INT-01 | 2820 vs 3001 | 2820: "Publish LangChain adapter to PyPI". 3001: "Tag adapters-v0.3.0 → origin". Different task scoped to same ID. | media |
| INT-02 | 2828 vs 3002 | 2828: "Publish LlamaIndex adapter to PyPI". 3002: "Mismo tag (LlamaIndex + 7 adapters)". | media |
| CODE-067 | 2560 vs 2562 | 2560 note sits inside the **2026-07-06** Wave 1-4 section and claims completion; 2562 dedicates a **2026-07-11** section to the same migration. Completion date is ambiguous (07-06 or 07-11). | media |
| DOC-19 | 2614 vs 2630 | Wave 1-6 and Wave 1-7 (both 07-07) both claim "DOC-19 — ARCHITECTURE.md actualizado a v0.2.0 / → v0.2.0 + sharded WAL docs". Same work logged twice in consecutive sessions. | media |
| DRV-005 | 2983 vs 2985 | 2983 row lives in the 07-23 Gate Check but says "Verificado Jul 24"; dedicated section dated 07-24 repeats identical content. Internal date/state inconsistency. | media |
| REV-003 | 2732 vs 2752–2848 | REV-003 dated **07-23** precedes REV-004…017 all dated **07-14** — sequence number and date order contradict (see §6). | media |
| DOC-19 | 2414 vs 2614 | 2414 (Batch 4): "6 ADRs creados (004-009)". 2614 (Wave 1-6): "ARCHITECTURE.md actualizado". Same ID, different deliverables. | media |

---

## 5. Non-task entries (investigations, autopsies, no-ops)

| Line(s) | Entry | Type | Severity |
|---------|-------|------|----------|
| 2560 | "**CODE-067 COMPLETADO** — migración u64→u128 finalizada… 444 tests pasando" — free-standing note after Wave 1-4 table, duplicated by the 2562 section | note / duplicate | media |
| 2581–2600 | "Post-Benchmark Deep Investigations (4 paralelas, **25 tareas agregadas al backlog**)" — no tasks completed; it *created* backlog items (PERF-15…38, CODE-092) and reports "Pendientes: 98 items open" | investigation (backlog intake logged as progreso) | media |
| 2589 | CODE-092 "Bug crítico… Fix estimado: 1 hora" — assigned to backlog, not completed, yet appears in a completed section | backlog assignment | media |
| 2968–2983 | TIER 1 Gate Check: 7 rows SKIP / "ALREADY FIXED" (DRV-031/026/116/040/025/047/012/009/016/007/049) — verification work with zero implementation; only DRV-019 actually fixed code | no-op gate check | media |
| 2981, 2993 | "**Patrón detectado:**" and "**Recomendación:**" analysis paragraphs — meta-commentary inside a task log | prose, not tasks | baja |
| 3146–3156 | P2 Backlog Housekeeping: 3 rows "✅ Document-only. Backlog actualizado." + "Sin code changes (ponytail — ya estaba corregido)" | no-op documentation-only | media |
| 3158–3170 | P8 Post-Launch: 2 "Document-only", 1 "Ya implementada" (DEVOPS-HOMEBREW), 1 already-existing (DEVOPS-PY313 CI-only), CLI-01/DEVEX-DEMO real work | mixed; 3/5 are no-ops | media |
| 3172–3189 | Backlog Cleanup: "53 items moved to progreso" — 6 stale removed, 24+24+7 stale moved, counters adjusted. Housekeeping, not implementation | no-op bookkeeping | baja |
| 3191–3216 | P5/P6/P8 Quick Wins: TSK-106 "Ya estaba habilitado (sin cambios)"; COM-03/COM-04 "⚠️ Parcial" (require manual Discord UI); MKT-03/04 drafts; NUEVO-21 investigation | mixed; several no-ops/partials | media |
| 3290–3298 | INV-017 "sccache en CI — **investigación**" — research deliverable (`docs/Investigaciones/INV-017-sccache-ci.md`), no implementation (implemented next day as GH-143) | investigation | baja |
| 2768–2774 | REV-016: "Delivered audit report **then deleted entire crate**" — audit + deletion, not a feature | audit/autopsy | baja |
| 3064–3072 | VFY-004: "comment-only" change (0 code logic changes) | no-op code change | baja |

---

## 6. Order

**Not chronologically sorted.** The dominant criterion is *grouping by work session/batch*, with several backward jumps:

| Lines | Jump | Severity |
|-------|------|----------|
| 2205–2355 (all 07-04) → 2357–2386 (07-02) → 2388–2445 (07-03) | Section opens with 07-04 entries, then goes **back** to 07-02 and 07-03. Block seems ordered by ID family (CODE → DOC → MKT → WEB → CI), not by date. | alta |
| 2546 (07-06) → 2562 (07-11) → 2581 (07-06) | Wave 1-5 (07-11) sits between two 07-06 sessions. | alta |
| 2724 (07-13) → 2732 (07-23) → 2752 (07-14) | REV-003 (07-23) inserted between 07-13 and 07-14 sessions. REV-004…017 (07-14) all *follow* REV-003 (07-23) despite earlier dates. | alta |
| 2962 (07-23) → 2985 (07-24) | DRV-005 (07-24) section duplicates a row already inside the 07-23 Gate Check table. | media |
| Within 07-04 block (2205–2355) | Neither numeric nor chronological: CODE-020, 021, 001, 009, 003, 002, 015, 049, 052, 079, 012, 026, 011, 018, 019, 005. | baja |
| Within 07-07 block (2602–2704) | Sessions appear in a plausible sequence but with an out-of-band 07-11 insert before them (see above). | media |

**Suggested criterion:** sort strictly by session date ascending; inside same-day blocks, keep batch grouping but note the sub-order.

---

## 7. Quality — what is missing / what is surplus

| Line(s) | Finding | Severity |
|---------|---------|----------|
| 2205–2355 | Individual entries (CODE-020/021/001/009/003/002/015/049/052/079/012/026/011/018/019/005, DOC-12, MKT-07/BIZ-03, WEB-08-Refinement, CI-01) have **no commit hash** — only textual verification ("440 tests pasan", "cargo check ✅"). Contrast: all table-based sessions (2447+) carry commits. | media |
| 2282–2308 | CODE-015/049/052/079 are pure "Auditoría confirmó ✅" entries with no verifiable evidence (no command, no file, no commit) — unverifiable claims. | media |
| 2546–2704 | `**Backlog actualizado:**` counter stuck at "79" for 4 sessions (2673/2693/2704) — stale counter, no longer reflects work done. | media |
| 2447, 2546, 2602, 2650, 2962, 3031, 3185 | **Count mismatches in titles**: "78 bugs" vs 59 table rows; "9 commits" vs 10 listed; "10 tareas" vs 7 rows; "5 tasks" vs 7 rows; "14 tasks" vs 12 rows; "8 tasks completadas" vs 13 rows; "12 ✅" vs 13 IDs; "3 completadas" vs 4 rows (1 WONTFIX + 2 ✅ + 1 DEFERIDO). | media |
| 3170 | "Total 90→95" — counter direction reversed (increase) after a completion session. | baja |
| 2449 | `**Commits:**` lists hashes without mapping to rows (tables carry their own Commit column) — redundant and partial (Documentation table has none). | baja |
| 3316 | ENT-04 entry is ~14 lines of implementation detail; siblings are 2–4 lines. Over-documented relative to the rest. | baja |
| throughout | Duplicated completion records (§3) inflate the file: every Cleanup/Housekeeping session re-lists items already logged with full sections. | alta |
| 3020 | Backlog Audit describes deleting 19+8 stale items "sin mover" — items removed without trace in this file (acceptable policy, but reduces auditability of what happened to them). | baja |

---

## Summary of top issues

1. **Order is not chronological** (3 backward jumps; worst: 07-02 after 07-04, 07-23 between 07-13/07-14, 07-11 between 07-06 sessions).
2. **ID collisions with conflicting tasks**: DOC-20 (LanceDB guide vs mdBook), DEVOPS-14 (verified-existing then removed), DEVOPS-10 (deferred then removed).
3. **23+ duplicated IDs**, mostly session section + cleanup/housekeeping re-listing.
4. **Title counts don't match table row counts** (78 bugs vs 59; "9 commits" vs 10; "14 tasks" vs 12; etc.).
5. **Mixed ES/EN** throughout; two incompatible record templates; 4 entries missing `Ids:` line.
6. **Backlog counters stale/frozen** (79 ×4 sessions; 90→95 increase).
7. **Non-task content** (investigations, no-op gate checks, document-only entries) occupies ~15% of the range.
