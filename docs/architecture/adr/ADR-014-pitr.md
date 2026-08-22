---
title: "ADR-014: PITR (WAL archival & point-in-time recovery) — experimental standalone API, engine integration deferred"
type: adr
status: accepted
tags: [vantadb, architecture, adr, durability, wal, pitr]
created: 2026-08-09
last_reviewed: 2026-08-09
---

# ADR-014: PITR — experimental standalone API, engine integration deferred

## Context

The `pitr` feature is declared in `Cargo.toml` (`pitr = []`) and gates
`pub mod wal_archiver` in `src/lib.rs` behind `#[cfg(feature = "pitr")]`.
Investigation (2026-08-09, FEAT-01) established the real state of the code:

- **`src/pitr.rs` does not exist.** The PITR implementation lives in
  `src/wal_archiver.rs` (496 lines): `WalArchiver` (moves rotated WAL segments
  into `{storage_path}/wal/archive/`, configurable retention by max age / max
  total size) and `PitrRestorer` (replays archived segments up to a target
  Unix millisecond timestamp). The module compiles behind the `pitr` flag and
  ships 7 unit tests that all pass.
- **The module is orphaned in production.** Every reference to `WalArchiver`,
  `PitrRestorer` or `WalArchiveConfig` is internal to `src/wal_archiver.rs`
  (implementation + `#[cfg(test)]`). Nothing in `StorageEngine`, the WAL
  rotation path (`src/wal_sharded.rs`), startup/recovery, the SDK, or the CLI
  calls it. It is a standalone public API reachable only by users who enable
  the feature and wire it themselves.
- **`src/storage/wal.rs` is not PITR.** It is a small WAL initialization
  helper mapping `VantaConfig` to `ShardedWal` (durability wiring, already
  functional and in use). The original task premise conflated it with
  wal-shipping; it is unrelated to this decision.
- **`wal-shipping` is a separate feature** (`wal-shipping = ["dep:reqwest"]`,
  gates `src/wal_shipping.rs`, send-only async WAL delivery). It is a real,
  already-documented feature and is out of scope for this ADR beyond noting
  the boundary.

So the `pitr` flag is not a dead phantom: it gates real, tested code. The
phantom part is that the capability is **not exposed through any product
surface** — there is no engine rotation hook and no SDK restore API.

Options considered: (a) fully integrate PITR into StorageEngine rotation +
SDK recovery; (b) mark the feature experimental and document standalone use,
deferring integration; (c) declare the feature deferred wholesale, leaving the
flag in place with honest docs.

Option (a) is real feature work that changes the WAL durability surface
(segment rotation hook, recovery ordering, archive lifecycle) — it requires
the durability pipeline (audit → chaos → tuner) and is beyond the scope of
this decision task. Option (c) under-states the code: the module is
functional and self-tested, not "not implemented".

## Decision

Mark `pitr` as an **experimental standalone API** and **defer engine/SDK
integration** to a future feature task:

1. Keep the `pitr` feature flag and the existing `#[cfg(feature = "pitr")]`
   gate. No code changes.
2. Document the feature honestly in `Cargo.toml`: functional and self-tested,
   **not** integrated into StorageEngine WAL rotation or SDK recovery; usable
   only as a standalone library API (`vantadb::wal_archiver`).
3. Record the decision in this ADR so the flag is no longer a silent phantom:
   its purpose, current state, and the deferred integration path are explicit.
4. Integration (option a) is intentionally deferred; revisit when PITR becomes
   a product requirement or a Pro candidate (see `docs/strategy/VANTADB-PRO-FEATURES.md`,
    `pitr` listed as Pro candidate; `docs/research/investigacion-equipo-2026-08-09.md`
   confirms today's orphaned state).

## Consequences

- Pros:
  - Honest, discoverable feature docs; the "feature fantasma" ambiguity is resolved.
  - Capability is preserved and fully compiled/tested behind its gate for later
    use or Pro migration — zero re-implementation cost of the core algorithm.
  - Zero behavior change: no durability path is touched, no APIs change.
  - The ADR gives future agents one place to read *why* PITR exists and what
    integrating it entails.
- Cons:
  - PITR is not user-reachable through StorageEngine/SDK; consumers must wire
    `WalArchiver`/`PitrRestorer` manually (documented as a limitation, not a
    bug).
  - Retention/restore behavior is only exercised by the module's own unit
    tests; no integration test covers archive + restore through the engine.
  - The deferred integration work remains open by design; it must be scheduled
    explicitly when PITR is actually demanded.

## Related

- `docs/strategy/VANTADB-PRO-FEATURES.md` (Pro candidate map)
- `.opencode/rules/durability.md` (WAL durability rules; scope lists
  `wal_archiver.rs` (pitr))
- ADR-013 (open-core licensing; `pitr` named among commercial candidates)