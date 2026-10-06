---
title: "ADR-0056: Object storage path — snapshot-level first, segment-level on the Pro/Cloud trigger, native backend deferred"
kind: adr
status: proposed
description: "Object storage enters through snapshot-level backup/DR (BIZ-14) now; segment-level WAL shipping is the design for the Pro/Cloud trigger; a native object-storage backend is deferred as a new-engine project"
tags: [vantadb, architecture, adr, storage, s3, object-storage, backup]
created: "2026-10-06"
---

# ADR-0056: Object storage path — snapshot-level first, segment-level on the Pro/Cloud trigger, native backend deferred

> **Produced under the plan 0.9.0 Task 65 contract (STRAT-05)** — research + spec + documented decision, no implementation. Status is `proposed`: it records the evidence-backed path and awaits the owner's ratification (Regla 5, `AGENTS.md`). Evidence: [research doc](../../research/strat-05-object-storage.md).

## Context and problem statement

VantaDB's storage is 100% local today: `BackendKind { RocksDb, Fjall (default), InMemory }` (`src/backend.rs:105-132`, runtime construction via `BackendRegistry` `src/storage/engine/init.rs:275-283`) with mmap vector segments (`vstore_L0..L3.vanta`, `docs/dev/architecture/STORAGE-TIERS.md`) and a local WAL. Two forces push toward object storage (S3/blob):

1. **DR/backup offsite** — BIZ-14 (`docs/dev/Backlog-negocio.md:103`): export `.vantadb` snapshots to network storage; trigger Pro/Cloud; Dep PRO-02/03.
2. **A future storage path** — the 2026 verdict (`docs/dev/archive/research-old/feature-verdicts-2026.md:128,135-137,387`) said "yes to backup, not S3 yet; S3 is Phase 5" and listed "❌ native S3 backup (cloud only, Fase 5+)".

The decision space is not "S3 yes/no" but **where object storage enters**: (a) snapshot-level (backup/DR), (b) segment-level (WAL/segment shipping), or (c) as a native backend (the database living in a bucket). Each has a different cost profile and a different engineering blast radius. Without a decision, the Pro/Cloud trigger would force the choice under delivery pressure.

## Considered options

### Option A — Snapshot-level (backup/DR transport)
Upload whole consistent snapshots (directory `.vantadb` via `Backup`/`Snapshot`, or logical `export_all`) to any S3-compatible endpoint; restore downloads and restores.
- Pros: zero engine/wire changes; matches object-store semantics (atomic whole-object PUT, multipart for large objects); local backup tooling already exists (`src/cli.rs:146-168,409-419`, `src/storage/engine/mod.rs:647,:697,:795`, `src/sdk/serialization/impl_export.rs:315,:531`); implementable as a thin adapter over the `object_store` crate (Apache Arrow; S3/GCS/Azure/R2/local, async, production-proven).
- Cons: RPO = backup cadence (hours/days), not seconds; storage cost proportional to copies × size (mitigable with retention/lifecycle).

### Option B — Segment-level (WAL/segment shipping, Litestream-style)
Asynchronously ship WAL pages and `vstore` segments (packaged with TXID + checksums, plus a state manifest) to object storage; the engine stays 100% local and the hot path is untouched; restore replays.
- Pros: RPO seconds/minutes; request costs are small (~$1-1.5/mo modeled); precedents: Litestream, SQLite Cloud Backed (fixed blocks + manifest), Chroma (log + indexes in object storage).
- Cons: a new replication subsystem (manifest, checkpoint/compaction coordination, restore tooling, crash/replay tests) — a dedicated spec, not this run; WAL↔mmap-segment consistency must be pinned by design.

### Option C — Native backend (`BackendKind::ObjectStore`)
KV operations living directly on object storage with local disk as cache.
- Pros: none for VantaDB today beyond hypothetical serverless/edge deployments without local disk.
- Cons: mmap is impossible (no seek/partial read — `object_store` is explicitly stateless); fsync/durability semantics would need conditional-write/fencing primitives; request latency is 50-100ms (SlateDB) vs µs local; precedents that succeeded built **new engines/formats** (Lance, SlateDB) or restricted to read-only (DuckDB `ATTACH ... (READ_ONLY)`) or delegate locking to the app with corruption risk (sqlite-s3vfs: "S3 does not support the partial replace of an object"; "if multiple writes happen at the same time, the database will probably become corrupt").

## Decision outcome

Chosen path (proposed for ratification):

1. **Now: Option A (snapshot-level) — the decision.** The decided object-storage path is offsite snapshot transport for BIZ-14 (thin adapter over `object_store`, no engine changes); its **implementation** starts when BIZ-14's Pro/Cloud trigger fires — this ADR does not start it. Local-first stays intact (ADR-0004/ADR-0020 unchanged).
2. **On the Pro/Cloud trigger (PRO-02/03): Option B (segment-level)** as the continuity/DR layer, with its own spec. This research fixes the why and the precedents; it does not implement it.
3. **Option C (native backend) is deferred**, with explicit revisit conditions: (a) a serverless/edge product requirement with no local disk; (b) network block-storage costs (EFS-like) dominating a customer's TCO; (c) maturity of a reusable LSM-on-object-store crate (e.g. SlateDB 1.0+). Even then it is a **new engine** project, not a `BackendKind` variant.

Boundary declaration (no re-litigation): the 2026 verdict stands ("no native S3 now; S3 is Phase 5"); this ADR decides the *how* of offsite backup (snapshot-level) and conditions the rest. BIZ-14 remains the implementation owner for offsite backup; the frontier is **backup offsite (enabled)** vs **storage path (deferred)**.

### Positive consequences
- The Pro/Cloud trigger no longer arrives without design: the path, costs and conditions are documented.
- BIZ-14 gets a concrete implementation decision (snapshot-level + `object_store`) with zero engine risk.
- Local-first invariants (ADR-0004/0020) and mmap-based tiers are untouched.

### Negative consequences
- RPO stays at backup cadence until segment-level is specified/implemented; continuity-grade DR remains open.
- A `proposed` ADR means the owner must ratify before BIZ-14 implementation starts.
- Cost model uses scenario assumptions (DB size, cadence, retention) — list prices are verified and dated, but the totals are not measurements.

## Related

- [Research: STRAT-05 object storage path](../../research/strat-05-object-storage.md) — evidence, precedents, cost model, sources (16 fetch-verified 2026-10-06).
- ADR-0004 / ADR-0020 — storage backend selection and default (unchanged).
- `docs/dev/Backlog-negocio.md:103` (BIZ-14) — the implementation item this decision feeds.
- `docs/dev/archive/research-old/feature-verdicts-2026.md` — prior verdict this ADR confirms and refines.
