---
title: Upgrade Guide
type: operations
status: active
tags: [vantadb, upgrade, migration]
last_reviewed: 2026-08-23
aliases: []
---

# Upgrade Guide

How to upgrade VantaDB between versions, what changes to expect, and how to
migrate safely. See [`docs/api/VERSIONING.md`](../api/VERSIONING.md) for the
underlying stability policy.

## Golden rule: backup before upgrade

Always take a backup before upgrading. Two options:

1. **Logical export** (preferred — version-independent format):

   ```python
   import vantadb

   db = vantadb.VantaDB("./vanta_data")
   db.export_all("./backup-pre-upgrade")
   ```

2. **Filesystem copy** (fastest — stop the process first):

   ```bash
   # Stop the database process, then:
   cp -r ./vanta_data ./vanta_data.backup
   ```

If the new version fails or behaves unexpectedly, restore by pointing back at
the backup directory (or downgrade the package and reopen the copy). Full
details and backup verification: [BACKUP_RESTORE.md](BACKUP_RESTORE.md).

## Version history

Each section lists what changed for consumers and any required migration steps.

### Upgrading to 0.5.0 (from 0.4.x)

**Released:** 2026-07-31 (tag `v0.5.0`).

**What changed (user-facing):**

- **IVF Flat index**: inverted-file index with k-means clustering, available as
  `IndexType::Ivf` on `HnswConfig`. Lazy-built on first search; serialized in v8
  format. ~50x faster than brute-force Flat on 1M vectors at ~90% recall.
- **Multi-level LSM compaction (L0–L3)**: `StorageEngine.vector_store` now splits
  into per-level VantaFiles. Write amplification drops from O(all data) to
  O(L0 size). New `PipelineMode::CompactOnly` / `CompactL0Only` variants.

**Breaking changes:** none reported.

> Evidence: `docs/CHANGELOG.md` § `[0.5.0] - 2026-07-31` lists only *Added*
> entries; no `BREAKING CHANGE` markers in the release notes, and no breaking
> commits surfaced between `0.4.0` and `v0.5.0`. The legacy `SegmentRegistry`
> handles migration of pre-existing single-level stores automatically on open.

**Migration steps:** none required. Existing data directories open unchanged;
legacy vector store segments are migrated transparently by the new
`SegmentRegistry` on first open. To opt into the IVF index afterwards, set
`IndexType.Ivf` in your index config.

**Note:** there is no `v0.4.0` git tag — all tags prior to `v0.5.0` were removed
in the 0.4.0 clean-versioning reset (see `docs/CHANGELOG.md` § 0.4.0 *Changed*).
Use the changelog, not tags, to diff against 0.4.0.

<!-- TEMPLATE — copy for each new release:
### Upgrading to X.Y.Z (from A.B.C)

**Released:** YYYY-MM-DD (tag ` vX.Y.Z`).

**What changed (user-facing):**
- ...

**Breaking changes:** <list each with old → new signature/behavior, or "none reported">

**Migration steps:**
1. ...
-->

## After upgrading

1. Open the database and run a read smoke test (list namespaces, one search).
2. Check `docs/CHANGELOG.md` for the versions you skipped — MINOR releases may
   stack multiple changes.
3. Delete the backup only after a full workload cycle completes cleanly.
