---
title: Upgrade Guide
kind: runbook
status: active
description: "How to upgrade VantaDB between versions, what changes to expect, and how to"
tags: [vantadb, upgrade, migration]
---

# Upgrade Guide

How to upgrade VantaDB between versions, what changes to expect, and how to
migrate safely. See [`docs/api/VERSIONING.md`](../../api/VERSIONING.md) for the
underlying stability policy.

## Golden rule: backup before upgrade

Always take a backup before upgrading. Two options:

1. **Logical export** (preferred — version-independent format):

   ```python
   import vantadb

   db = vantadb.connect("./vanta_data")
   db.export_all("./backup-pre-upgrade")
   db.close()
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

### Upgrading to 0.8.0 (from 0.7.x)

**Released:** pending — ships through the release-plz Release PR (tag `v0.8.0`).

**What changed (user-facing):**

- **Records are now bitemporal (schema v2).** Every record gains a *valid-time*
  window — `valid_at_ms` / `invalid_at_ms` — separate from the *transaction
  time* the engine already tracked (`version`, `updated_at_ms`,
  `superseded_at_ms`; retained per key). Valid time says when the content was
  true about the world (user-controlled; retroactive updates are a later
  release); transaction time says when the system recorded it. Validity is a
  half-open interval `[valid_at_ms, invalid_at_ms)`: a record is valid at `T`
  when `valid_at_ms <= T < invalid_at_ms`; an open `invalid_at_ms` (`None`)
  means "still valid". `valid_at_ms` defaults to `created_at_ms`.
- **Point-in-time queries (`AS OF`).** IQL version 2 adds the opt-in
  `AS OF <unix-ms>` clause (valid time, not transaction time), and search/list
  accept the equivalent `as_of_ms` and `valid_window` params. Without them,
  queries behave exactly as in 0.7.x (opt-in; defaults unchanged). See
  [`IQL.md` § Valid-Time Queries](../../api/IQL.md#valid-time-queries-as-of).
- **Confidence per record.** `confidence_class` (`asserted` | `derived`),
  `confidence` (finite, in `[0,1]`), `last_validated_at_ms`, `derived_from`.
  A `derived` record must declare its parents and cannot carry a declared
  score: the engine computes `min(parents) × 0.9` (rejected with a validation
  error otherwise). Search/list accept an opt-in `min_confidence` filter.
- **Quarantine.** Records can enter quarantine (`quarantined_at_ms`,
  `quarantine_reason`, `quarantined_by`, `quarantine_review_due_ms`).
  Quarantined records are excluded from search/list/retrieval by default;
  `include_quarantined` opts in, and `get` by key always returns them with the
  state visible (never a silent miss). Quarantine is sticky, never
  auto-promotes, and its 30-day review deadline is a signal only.
- **Selective abstention (opt-in).** With `confidence_threshold` configured
  (env `VANTADB_CONFIDENCE_THRESHOLD`, default OFF), a search whose page ends
  empty returns an explicit `abstained: true` + `abstention_reason` signal
  instead of a silently empty result.
- **The v2 fields and query params cross every surface** — Python, TypeScript,
  Node, WASM, HTTP, MCP, and IQL — with the same wire names
  ([`BINDINGS_NAMESPACES.md`](../../api/BINDINGS_NAMESPACES.md)).

**Breaking changes:** 0.8.0 is a MINOR pre-1.0 release, so it may carry
breaking changes. The **authoritative** list is the `[0.8.0]` changelog entry (release-plz);
[`COMPATIBILITY.md` § Pre-release deltas](../../api/COMPATIBILITY.md#pre-release-deltas-vs-published-070);
the ones most likely to touch consumers:

| Area | Change | What to do |
|------|--------|------------|
| Storage (on-disk) | Schema v2: new record fields; JSONL export switches to v2 (`schema_version: 2`); the storage header bumps to `2` when you run the migration. A 0.8.0 binary reads a 0.7.x directory as-is (normalization on read); a 0.7.x binary refuses a migrated directory (`TooNew`). | Run the migration below, or keep reading the v1 directory with 0.8.0 without rewriting it. v1 exports stay importable (`schema_version` 1 or 2 accepted, `> 2` rejected). |
| Rust SDK | `MemoryRecord` / `MemoryInput` / `MemoryListOptions` / `MemorySearchRequest` / `Query` gain public fields (struct literals must be updated). `Embedded::import_records` / `import_file` gain a `quarantine: bool` argument. `VantaHeader` is renamed to `Header` (deprecated alias kept). `QueryResult::Write.node_id` serializes `u128` as a decimal string. | Update literals/calls; rename or keep using the alias. See [`EMBEDDED_SDK.md`](../../api/EMBEDDED_SDK.md). |
| Cargo features | `feature = "server"` no longer enables `cli`. | If you relied on the implication, use `features = ["server", "cli"]`. |
| CLI | `--json` is now a global flag; the query is a positional argument (alias `--query`); `--limit` is canonical (alias `--top-k`); `--in` / `--out` are symmetric; new `vanta-cli mcp-call`. | Re-check scripts against `vanta-cli --help`; the aliases keep most 0.7.x spellings working. |
| IQL | `IQL_VERSION` 1 → 2 (adds `AS OF`; version-gated). | Feature-detect with `IQL_VERSION_MIN_AS_OF` / `iql_supports`; v1 statements keep parsing. |
| HTTP / MCP / TS bindings | The API-standardization wave lands in 0.8.0: OpenAPI-first HTTP routes, canonical MCP tool names/schemas, object-shaped TypeScript API, `u128` as string on every JSON wire. | Align clients with [`HTTP_API.md`](../../api/HTTP_API.md), [`MCP.md`](../../api/MCP.md), [`BINDINGS_NAMESPACES.md`](../../api/BINDINGS_NAMESPACES.md). |
| Eviction / LLM prompts | Migrated records get `confidence = 1.0` (previously an implicit node-level 0.5 fed the same formula), so eviction weights and prompt scores move. | If you tuned `eviction_weight_confidence` against 0.5, recalibrate per ADR-046 §D4c. |

**Migration steps (data directory v1 → v2):**

1. **Back up first** — follow [Golden rule](#golden-rule-backup-before-upgrade)
   above (logical export preferred; filesystem copy for large trees). The
   migration rewrites stored records and snapshots in place; the backup is your
   rollback.
2. Upgrade the package: `pip install -U vantadb-py==0.8.0` /
   `npm i vantadb@0.8.0` / `cargo update -p vantadb`.
3. Inspect and migrate the directory with `vanta-cli` (stop writers first):

   ```bash
   vanta-cli migrate plan  ./vanta_data                        # what will change (v1 → v2 per format)
   vanta-cli migrate check ./vanta_data                        # integrity pre-check
   vanta-cli migrate run   ./vanta_data --format records --dry-run  # preview the backfill + record count
   vanta-cli migrate run   ./vanta_data --format records       # backfill records (idempotent)
   vanta-cli migrate run   ./vanta_data --format all           # remaining formats + schema-header bump (last)
   ```

   - The backfill is deterministic and idempotent: re-running recomputes the
     same values, and an interrupted run leaves a readable database (header v1
     + partially backfilled data) that both 0.7.x and 0.8.0 open — re-run to
     finish.
   - Order is always expand → backfill → header bump. The header bump marks the
     migration complete, so run `--format all` (or `--format schema`) **after**
     `records`, never before.
   - New in 0.8.0: the `records` format. `--format` accepts `all`, `vfile`,
     `index`, `wal`, `records`, `schema`.
   - Optional determinism check: migrate two copies of the same 0.7.x
     directory and compare them byte for byte.
4. Verify after migrating:

   ```bash
   vanta-cli audit-index --json    # → "passed": true
   # read smoke test: list namespaces + one search, then roundtrip:
   vanta-cli export --namespace <ns> --out export-v2.jsonl   # exports are v2
   vanta-cli import --in export-v2.jsonl
   ```

   v1 JSONL files remain importable (`vanta-cli import --in export-v1.jsonl`).
5. Roll back if needed: restore the backup and reopen it with 0.7.x (or
   re-import the logical export). Once the header is v2 a 0.7.x binary refuses
   the migrated directory — always roll back from the backup.

### Upgrading to 0.7.0 (from 0.6.x)

**Released:** 2026-09-25 (tag `v0.7.0`).

**What changed (user-facing):**

- **Monorepo restructure**: the Next.js web app moved out to its own repository
  ([`ness-e/Vantadb-web`](https://github.com/ness-e/Vantadb-web)); the docs tree
  was reorganized under `docs/user/` + `docs/dev/`.
- **Docs & version coherence**: API doc version headers pinned to 0.7.0 (rule
  R-2) and Python SDK docs updated — no code-facing contract changes.
- **Release tooling**: release-plz now releases only when the Release PR is
  merged (`release_always = false`).

**Breaking changes:** none for consumers. Evidence: `docs/CHANGELOG.md`
§ `[0.7.0] - 2026-09-25` carries only an *Other* entry (no `feat!` /
`BREAKING CHANGE` markers), and `git diff v0.6.1 v0.7.0 -- src/` touches five
internal lines only (no public API changes).

**Migration steps:** none required beyond upgrading the package
(`pip install -U vantadb-py==0.7.0` / `npm i vantadb@0.7.0`). If you consumed
the web app from this monorepo, it now lives in `ness-e/Vantadb-web`.

### Upgrading to 0.6.1 (from 0.5.x)

**Released:** 2026-09-23 (tag `v0.6.1`).

**What changed (user-facing):**

- **Published artifacts**: `vantadb-py` 0.6.1 on PyPI (multi-platform wheels, no
  Rust toolchain needed), `vantadb` + `vantadb-wasm` 0.6.1 on npm, `vantadb`
  0.6.1 on crates.io. TestPyPI carries the same version (trusted publishing,
  no tokens).
- **MCP structured output**: `search_memory` / `memory_search` /
  `search_semantic` / `search_with_method` now return a budgeted envelope
  object (`{hits, byte_count, truncated}`) in `structuredContent`; the text
  payload stays the raw hits array. Only affects consumers parsing
  `structuredContent` as a bare array.
- **Docs moved**: this tree now lives under `docs/user/` + `docs/dev/`
  (see `docs/README.md`).

**Breaking changes:** none. Data directories open unchanged (no storage
migration; verify with `vanta-cli audit-index --json` → `"passed": true`).

**Migration steps:** none required beyond upgrading the package
(`pip install -U vantadb-py==0.6.1` / `npm i vantadb@0.6.1`).

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
