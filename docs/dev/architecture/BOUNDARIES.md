---
title: VantaDB Module Boundaries
type: architecture
status: active
tags: [vantadb, architecture]
last_reviewed: 2026-09-23
aliases: []
---

# VantaDB Module Boundaries

> **Status:** Accepted · **Date:** 2026-09-13 · **Task:** C2A1
> (`docs/dev/plans/2026-09-13-cleanCA-fase2.md`, Wave 0, Task A1)
> **Inputs:** M1 baseline (`docs/dev/tasks/C2M1.md`) · Config decision data
> (`docs/dev/tasks/C2D0.md`) · Full architecture: `ARCHITECTURE.md`
>
> **Thesis (why this file exists):** physical reorganization does not pay;
> **explicit boundaries pay**. Layout ≠ architecture. This document declares
> who owns what, which direction dependencies may point, which cycles are
> known debt, and the gate for any future physical move (Phase 3).
> `/cleanCA` (future A3 task) cites rules `BND-01…BND-08` from here.
> **A1 review:** vanta-arch — verified against tree 5a1dae99 on 2026-09-13, no material drift; deriva trivial: engine/mod.rs 34-35, engine/init.rs 12, executor.rs 13; nota: index/flat.rs:18 es fn-local prod (no test-mod), index/search/layer.rs:13 FLAG_TOMBSTONE prod no tabulado — cubierto por plan FLAG_TOMBSTONE→kernel.

## 1. Ownership map

Every top-level module in `src/` has exactly one owner. The owner approves
cross-boundary changes touching their module.

| Module(s) | Owner | Responsibility |
|---|---|---|
| `storage/engine/` (`mod, init, ops, txn, maintenance, archive, partition, get, insert, delete, stats`) | storage owner | Canonical data, WAL apply path, transactions, durability |
| `storage/` rest (`vfile, archive, wal`) | storage owner | Persistence primitives (mmap, WAL, files) |
| `backend.rs` + `backends/` | storage owner | `StorageBackend` KV contract + Fjall/RocksDB/InMemory impls |
| `index/` (flat, ivf, graph, diskann, scann, serialize, search, distance, core, stats, refresh, neighbor_index, auto_tune) | index owner | Derived ANN/lexical indexes, rebuildable from canonical data |
| `sdk/` (api, builder, search, serialization, types, connect, graph, gds, version_history) | sdk owner | Stable embedded boundary (`sdk/types.rs`), bindings surface |
| `entity/` | entity owner | Entity model + (future M2) `EntityRepository` port |
| `planner.rs`, `executor.rs`, `query.rs`, `parser/`, `physical_plan/`, `cost_estimator.rs` | query owner | Parse → plan → execute; `LogicalOperator` chain (future S6) |
| `config.rs` | config owner | 52-field god-struct of *reads* (decision pending: C2D0 QUESTION) |
| `node.rs`, `error.rs`, `wal.rs`, `server/`, `columnar/`, `vector/` | respective owners | Shared kernel types; server wraps the core, never the reverse |

**Rule BND-01 (owners):** every cross-module dependency change names the
affected owners in the PR. No ownerless edits to boundary files
(`backend.rs`, `sdk/types.rs`, `storage/engine/mod.rs`, `index/mod.rs`).

## 2. Dependency directions (what may point at what)

```
                    ┌─────────────┐
                    │     sdk     │  stable boundary (types.rs)
                    └──────┬──────┘
                           │ uses via traits (BND-02)
              ┌────────────┴────────────┐
              ▼                         ▼
     ┌───────────────┐         ┌───────────────┐
     │ query engine  │────────▶│    entity     │
     │ planner/exec  │  uses   │  (M2: trait)  │
     └───────┬───────┘         └───────┬───────┘
             │ uses                    │ via EntityRepository (M2)
             ▼                         ▼
     ┌───────────────┐  rebuilds   ┌───────────────┐
     │    index      │◀────────────│ storage/engine│
     │ (derived,     │  from       │ (canonical,   │
     │  rebuildable) │  canonical  │  durable)     │
     └───────────────┘             └───────────────┘
             │ uses                        │
             ▼                             ▼
     ┌─────────────────────────────────────────────┐
     │              Shared kernel                  │
     │  node.rs · error.rs · config.rs · vfile ·   │
     │  wal.rs (stable + common ONLY, BND-05)      │
     └─────────────────────────────────────────────┘
```

**Rule BND-02 (search/state access):** `sdk/search/` and `sdk/serialization/`
must reach `storage` **only through a trait** (`StorageBackend` roles after
S1, `EntityRepository` after M2) — never by importing `StorageEngine`
concrete methods directly for new code.

> **Known violation (grandfathered debt, not a license):** as of 2026-09-13
> these files import `crate::storage::StorageEngine` directly and are
> **exempt until S1/M2 land**, then migrate:
> `sdk/search/text_index.rs:12`, `sdk/search/debug.rs:16`,
> `sdk/serialization/impl_index.rs:11`,
> `sdk/serialization/impl_sparse_index.rs:29`,
> `sdk/serialization/impl_text_index.rs:6`,
> `sdk/serialization/impl_rebuild.rs:11`, `sdk/builder.rs:6`,
> `executor.rs:10`. New code MUST NOT add to this list.

**Rule BND-03 (sibling cycles = error):** cycles between sibling modules
(`storage ↔ index`, `sdk ↔ planner`, `sdk ↔ query`) are **errors**, not
style. Gate: `cargo modules dependencies --lib --acyclic` must exit 0
(M1 baseline recorded the current state; M3 breaks the sdk cycles).
**Exception BND-03X (signed):** the single self-edge
`vantadb::accumulator::GraphAccumulator ↔ GraphAccumulator::new` reported by
`cargo modules dependencies --lib --acyclic` is an **analyzer artefact, not an
architectural cycle**, and does not count toward this rule. Cause: item-level
granularity — the struct *owns* its field (`values: DashMap<u128, AtomicU64>`,
`src/accumulator.rs:31-34`) while the constructor *returns* `Self`
(`src/accumulator.rs:41-45`), so every `new() -> Self` ctor trivially produces
an owns + `-> Self` edge pair. Pre-existing since before Fase 2 (recorded in
the C2M1 baseline `docs/dev/reviews/acyclic-baseline.txt`, byte-identical
post-Fase-2 per the F3G re-measurement
`docs/dev/reviews/acyclic-post-fase2.txt`). Scope: this one self-edge only — any
other cycle, including any new inter-module edge, remains an error under
BND-03. Disposition per human decision Q2 (Fase 3 plan, Gate P): sign the
exception, do not fight the tool — changing prod code to silence an analyzer
artefact has cost without benefit.
> **Exception signed:** vanta-worker F3G, acting on human decision Q2 (Fase 3
> plan Gate P), 2026-09-13. A1 architecture review sign-off: pending (F3G/G3).

**Rule BND-04 (`pub use` re-export convention):** when a symbol moves
between modules, keep the old path alive with a `pub use` re-export for
one minor version (precedent: `storage/engine/mod.rs:34`
`pub use crate::index::FreshHnswReport;`). Moves never break importers
silently.

## 3. Known cycle: storage ↔ index (documented debt with a plan)

Bidirectional dependency, verified 2026-09-13 by `rg` (C2A1 discovery).
Not broken by M3 — needs its own trait-split task or explicit DEFER.

**Direction A — storage → index:**

| File | Line | Import |
|---|---|---|
| `src/storage/archive.rs` | 4 | `use crate::index::CPIndex;` |
| `src/storage/engine/mod.rs` | 33–34 | `use crate::index::CPIndex;` + `pub use crate::index::FreshHnswReport;` |
| `src/storage/engine/init.rs` | 17 | `use crate::index::{CPIndex, IndexBackend};` |
| `src/storage/engine/maintenance.rs` | 9 | `use crate::index::{CPIndex, IndexBackend};` |

**Direction B — index → storage:**

| File | Line | Import |
|---|---|---|
| `src/index/flat.rs` | 18 (fn-local `use` inside `flat_search`, prod — not the `#[cfg(test)]` mod at :216) | `use crate::storage::engine::FLAG_TOMBSTONE;` |
| `src/index/mod.rs` | 20 | `use crate::storage::vfile::File;` |
| `src/index/serialize/file.rs` | 2 | `use crate::storage::vfile::MmapMut;` |
| `src/index/graph/types.rs` | 5 | `use crate::storage::vfile::MmapMut;` |
| `src/index/search/layer.rs` | 13 (prod hot-path `search_layer`) | `use crate::storage::engine::FLAG_TOMBSTONE;` |

(Paths normalized: on disk `serialize/file.rs` and `graph/types.rs` are
subdirectories; the plan's `serialize-file.rs` / `graph-types.rs` shorthand
refers to these.)

**Why it exists:** the engine orchestrates index builds/rebuilds (A is
structural: orchestration needs the index types), while the index reuses
persistence primitives and the tombstone flag (B is mostly legitimate
layering toward `vfile`; the two `FLAG_TOMBSTONE` prod imports are the smell,
covered by the FLAG_TOMBSTONE→kernel plan).

**Plan (pick one, no silent third option):**

1. **Trait-split task** (preferred): extract an `IndexBackend`-shaped port so
   `storage/engine` depends on the trait and `index` implements it; move
   `FLAG_TOMBSTONE` (and any shared const) down to the Shared kernel
   (`node.rs` or a `storage/api-types` leaf with zero deps) so `flat.rs`
   stops importing `storage::engine`.
2. **Explicit DEFER** with rationale + revisit date, recorded here and in
   Backlog as a `FIND-*` row. "It works, don't touch it" without a row is
   not a decision — it is drift.

## 4. Hybrid doctrine (validated against industry consensus)

**Slices outside, layers inside, incremental migration — never big-bang.**

- **Slices outside:** new features land as vertical slices (one use case,
  all concerns together — cf. Bogard 2018, VSA). A slice owns its request,
  handler, validation and tests in one place.
- **Layers inside (Clean, per slice):** inside a slice, dependencies point
  inward toward domain policies (cf. Martin, *Clean Architecture* ch. 21–22:
  the architecture screams use cases, frameworks stay at arm's length).
- **Shared domain at the center:** slices may share the domain model
  (entities, value objects). Cross-slice reuse goes *down* into the domain
  or into thin services — slices never call each other directly
  (cf. Jovanović 2024: per-feature `Shared/` folders; global sharing only
  for the stable and common).
- **Rule BND-05 (Shared discipline):** `Shared` (kernel: `node.rs`,
  `error.rs`, kernel types) accepts only the **stable + common**. Small
  duplication between slices is tolerated over premature shared
  abstractions. Hoisting code to Shared requires: used by ≥2 slices AND
  unchanged for ≥1 release (or a merged ADR).
- **Rule BND-06 (no slice-to-slice calls):** slices communicate via domain
  events or thin shared services, never by direct `use` of another slice's
  internals.
- **Rule BND-07 (incremental migration):** new code goes in slices;
  old code migrates only when touched; stable code is never moved for
  aesthetics. One domain at a time, starting with the cleanest.
- **Rule BND-08 (layout ≠ architecture):** folder moves without boundary
  changes are cosmetics and need no ADR; boundary changes (new edge,
  reversed direction, new Shared member) need an ADR-delta note in the PR.

## 5. Phase 3 gate: reactivation of physical reorganization (Screaming)

The Screaming-layout move stays **DEFERRED** until ALL of these are green
after Phase 2 (plan §Cierre + §Fase 3, restated here as the citable gate):

1. **S3 + M2 + S1 closed** — real ports, not promises (`TxnManager`
   extracted, `EntityRepository` live, `StorageBackend` segregated).
2. **M1 re-measured with D falling** for `storage/engine` and `sdk`
   (same-tool ratchet; rust-dsm undercounts `Ca` at module rollup —
   compare same-tool only, see C2M1 §Sesgos).
3. **`cargo modules dependencies --lib --acyclic` green** AND the
   `sdk↔planner` / `sdk↔query` cycles broken (M3).
4. **This document signed** — owners named (§1) and BND-01…BND-08 accepted.
5. **storage↔index** has its own trait-split task closed OR an explicit
   DEFER row (§3, option 2).

Even then: incremental per domain (one at a time, cleanest first),
never big-bang; stable code is not touched.

## 6. Internet digest (≤500 words, sources)

- **Screaming Architecture (Martin, blog 2011-09-30; book ch. 21, 2017):**
  the system's structure should shout its use cases, not its frameworks;
  frameworks/databases stay at arm's length so use cases are unit-testable
  without them. Sources:
  https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html ·
  https://blog.cleancoder.com/uncle-bob/2011/11/22/Clean-Architecture.html ·
  https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html ·
  https://www.oreilly.com/library/view/clean-architecture-a/9780134494272 (ch. 21–22 listing).
- **Vertical Slice Architecture (Bogard, 2018):** organize around requests,
  not layers; couple vertically along the axis of change ("things that
  change together belong together"); CQRS falls out naturally (GET vs
  POST/PUT/DELETE); minimize coupling *between* slices, maximize it
  *within* a slice; shared abstractions (repository/service-per-layer)
  melt away except where tools demand them. Sources:
  https://www.jimmybogard.com/vertical-slice-architecture (2018-04-19) ·
  https://github.com/SSWConsulting/SSW.VerticalSliceArchitecture.Documentation
  (history: term coined 2018, NDC Sydney follow-up).
- **Hybrid consensus (Jovanović 2024; NDepend 2025; DEV 2026 — latter two
  per plan digest, not re-fetched here):** real projects land on slices
  outside + Clean layers inside, with a shared domain at the center;
  per-feature `Shared/` folders for related-slice reuse; global sharing
  only for cross-cutting stable code (validation/logging/caching via
  pipeline behaviors); duplication tolerated over wrong abstraction.
  Sources: https://milanjovanovic.tech/blog/screaming-architecture ·
  https://www.milanjovanovic.tech/blog/vertical-slice-architecture ·
  https://www.milanjovanovic.tech/blog/vertical-slice-architecture-where-does-the-shared-logic-live ·
  https://github.com/jonyeezs/clean-verticalslice-architecture ("slices +
  Clean layers" metaframework). [NDepend 2025 / DEV 2026 consensus claims
  — cita NO VERIFICADA en esta sesión, heredada del plan Fase 2.]
- **Acyclic design (Lakos; Rust gate):** Lakos (*Large-Scale C++
  Software Design*, 1996; *Large-Scale C++ v1*, 2019, ch. 3:
  levelization, no cyclic link-time deps) is the origin of
  levelized/acyclic packaging discipline; the Rust analogue enforced here
  is `cargo modules dependencies --lib --acyclic` (reports first cycle;
  green ≠ total proof — M1 documents the limit). Sources:
  https://www.pearson.com/en-us/subject-catalog/p/large-scale-c-process-and-architecture-volume-1/P200000009513/9780133927665 ·
  https://github.com/rakhimov/cppdep (Lakos dep_utils rewrite) ·
  https://github.com/regexident/cargo-modules [cita NO VERIFICADA línea
  por línea en esta sesión — pin 0.27.0 verificado en vivo por C2M1].
  (~440 words)

## 7. References

- Plan: `docs/dev/plans/2026-09-13-cleanCA-fase2.md` (Wave 0 · A1 · §Fase 3)
- Metrics: `docs/dev/tasks/C2M1.md` (+ `docs/dev/reviews/dsm-baseline.json`,
  `coupling-baseline.json`, `acyclic-baseline.txt`)
- Config decision data: `docs/dev/tasks/C2D0.md` (human decision pending)
- Architecture: `ARCHITECTURE.md` (layered view — layers live *inside*)
- Task file: `docs/dev/tasks/C2A1.md`
- ADRs: `docs/dev/architecture/adr/` (convention: existing files rule)
