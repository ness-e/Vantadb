---
title: "ADR-0060: On-disk compatibility v0.9.0 → v0.10.0 — open in place, lazy index rebuild, no implicit encryption"
kind: adr
status: proposed
description: "Every persisted schema/version between v0.9.0 and v0.10.0 is inventoried and classified: primary formats are byte-compatible so v0.9.0 databases open in place, secondary index state self-heals at open, and encryption of an existing database stays out of the open path (ADR-0057 D7)"
tags: [vantadb, architecture, adr, storage, migration, compatibility]
created: "2026-11-09"
owner: vanta-arch
---

# ADR-0060: On-disk compatibility v0.9.0 → v0.10.0 — open in place, lazy index rebuild, no implicit encryption

## Status

Proposed. Governs FIND-631. Cites [ADR-0057](./ADR-0057-encryption-at-rest.md) D1/D7 and the version
rules centralized by FIND-499 (`src/migration.rs` `ArtifactSpec`/`classify`, documented in
`docs/dev/tasks/FIND-499.md` §Reglas de versión).

## Context and problem statement

Between tag `v0.9.0` and `develop` (the `v0.10.0` line) thirty storage files changed
(`git diff --stat v0.9.0..HEAD -- src/storage src/wal.rs src/index/serialize src/backends
src/lsm.rs src/schema.rs`: +10 813 / −466), mostly to land encryption at rest. The release
contract needs a stated policy: what does `v0.10.0` do when it opens a database directory
written by `v0.9.0`, what may it rewrite, what happens on power loss, and can the database
go back to `v0.9.0`.

### Persisted-version inventory (verified `git show v0.9.0:<path>` vs `HEAD`)

| Constant / artefact | v0.9.0 | v0.10.0 (HEAD) | Behaviour when a v0.9.0 value is opened | Evidence |
|---|---|---|---|---|
| `VFILE_VERSION` (VantaFile `VFLE` segments `vstore_L{0..3}.vanta`) | 2 | 2 (plain) / 3 (`VFILE_ENCRYPTED_VERSION`) | Read in place — `Header::validate_compat` accepts any `format_version ≤ VFILE_MAX_READABLE_VERSION` | `src/storage/vfile.rs:72,90,96`; `src/binary_header.rs:115-149` |
| `WAL_FORMAT_VERSION` (`VWAL` `vanta.wal` / `vanta.shardN.wal`) | 3 | 3 (plain) / 4 (`WAL_ENCRYPTED_FORMAT_VERSION`) | Read in place — range-based check `≤ WAL_MAX_READABLE_VERSION` (= 3 without `encryption`) plus `WAL_POSTCARD_VERSION == 1` in `schema_version` | `src/wal.rs:26,45,74,283,306-310` |
| `WAL_CHAIN_VERSION` (hash-chain framing) | 3 | 3 | Unchanged — v0.9.0 already chains records | `src/wal.rs:60` (v0.9.0 `:31`) |
| `VECTOR_INDEX_VERSION` (`VNDX` `vector_index.bin`) | 8 | 8 (plain) / 9 (`VECTOR_INDEX_ENCRYPTED_VERSION`) | Read in place; a v8 image deserializes or, if unreadable, the engine rebuilds it from the VantaFile (`None` → rebuild) | `src/index/graph/types.rs:27,44`; `src/index/serialize/file.rs:100-127,164,181`; `src/storage/engine/init.rs:548` |
| `CURRENT_SCHEMA_VERSION` / `MIN_COMPAT_VERSION` (`.vanta.schema`) | 2 / 1 | 2 / 1 | `load_or_create_schema` (write path) / `check_schema_compatibility` (read-only) both run `StorageHeader::is_compatible`; `version > CURRENT` → `SchemaError::TooNew`, `< min_compat` → `TooOld` | `src/schema.rs:17,19,90-104`; `src/storage/engine/init.rs:370-377` |
| `TEXT_INDEX_SCHEMA_VERSION` (+ `TOKENIZER_VERSION`, `ADVANCED_TOKENIZER_VERSION`) | 4 / 3 (cfg `advanced-tokenizer`) | 4 / 3 — identical | Persisted `TextIndexState` matches spec → kept; on any mismatch `ensure_text_index_current_with` rebuilds lazily at open | `src/text_index.rs:16,19,23,29`; `src/sdk/serialization/impl_text_index.rs:12-56,111-117` |
| `DERIVED_INDEX_SCHEMA_VERSION` (`DerivedIndexState`) | 1 | 1 | Same-value match kept; a stale version triggers `rebuild_derived_indexes_with_report` at open | `src/sdk/serialization/mod.rs:55`; `src/sdk/serialization/impl_index.rs:33-77` |
| `SPARSE_INDEX_SCHEMA_VERSION` (`SparseIndexState`) | 1 | 1 | Same-value match kept; a stale version triggers a sparse rebuild at open | `src/sdk/serialization/mod.rs:59`; `src/sdk/serialization/impl_sparse_index.rs:163-195` |
| `EXPORT_SCHEMA_VERSION` / `EXPORT_MANIFEST_SCHEMA_VERSION` / `MD_EXPORT_SCHEMA_VERSION` | 2 / 1 / 2 | 2 / 1 / 2 | Export files only — not part of the database image; importers accept `1..=EXPORT_SCHEMA_VERSION` | `src/sdk/serialization/mod.rs:54,801-806`; `src/sdk/types/record.rs:607`; `src/cli_handlers/export_md.rs:48` |
| `PURGE_CERTIFICATE_SCHEMA_VERSION` / `WRITE_RECEIPT_SCHEMA_VERSION` | 1 / 1 | 1 / 1 | Attestation artefacts; version check is equality → typed error on mismatch | `src/attestation.rs:44,535-540,657,974-979` |
| `CURSOR_VERSION` (search page tokens) | 1 | 1 | Opaque API token, not on-disk state | `src/sdk/search/page.rs:49,231` |
| `KDF_FRAME_VERSION` (key-derivation frame) | 0x01 | 0x01 | Unchanged | `src/crypto.rs:594` (v0.9.0 `:115`) |
| `BACKEND_KV_FORMAT_VERSION` (encrypted-value envelope) | — (no `kv_crypto.rs` at v0.9.0) | 1 | Only reachable inside an encrypted backend; a v0.9.0 (plaintext) backend never carries the envelope | `src/backends/kv_crypto.rs:40-47,172` |
| `*_ENCRYPTED_VERSION` constants | — (feature absent at v0.9.0) | VFLE 3 / VWAL 4 / VNDX 9 | A v0.9.0 build cannot produce them; a v0.10.0 build without `encryption` reports encrypted artefacts as `IncompatibleFormat` with a hint — never "corrupt" | `src/index/serialize/file.rs:106-122`; `src/storage/vfile.rs:96-100`; `src/wal.rs:51-55` |
| Fjall keyspaces (`default`, `tombstone_storage`, `compressed_archive`, `tombstones`, `namespace_index`, `payload_index`, `text_index`, `sparse_index`, `internal_metadata`, `versions`) | 10 keyspaces | 10 keyspaces — identical names | Open-or-create semantics; nothing new required | `src/backends/fjall_backend.rs:55-86` (v0.9.0 `:38-47,:62-94`) |
| Segment file names | `vstore_L{0..3}.vanta`; legacy `vector_store.vanta` auto-renamed | identical | The `vector_store.vanta → vstore_L0.vanta` rename pre-dates v0.9.0 and still runs at open | `src/lsm.rs:59-67,148-156` |

**Conclusion of the inventory:** every schema that existed at `v0.9.0` is byte-identical at
`v0.10.0`. The only new persisted versions belong to the encrypted lineage, which a `v0.9.0`
database cannot contain. No persisted format *requires* migration for a `v0.9.0` upgrade.

## Considered options

- **A — Open in place, self-healing secondary indexes.** Primary artefacts (VFLE/VWAL/VNDX,
  `.vanta.schema`) open unchanged because their versions are identical; secondary index
  state (text / derived / sparse) is reconciled by `ensure_indexes_current` at open, as it
  already is for same-version drift. Pros: zero user action, zero copy of data, matches the
  code as written. Cons: first open may pay one rebuild of secondary indexes; rollback
  guarantee must be stated explicitly.
- **B — Mandatory `vanta-cli migrate` gate.** Refuse to open a `v0.9.0` base until an
  explicit migrate bumps a marker. Pros: explicit operator consent, a natural place to
  force a backup. Cons: pure ceremony — there is nothing to migrate; adds a failure mode
  (a v0.9.0 base without the marker) and contradicts the range-based readers that already
  accept older versions. Rejected.
- **C — Copy-on-upgrade into a new directory.** Materialize a `v0.10.0` layout beside the
  old one. Pros: perfect rollback. Cons: doubles disk usage for every user for no format
  gain; the tmp+rename + WAL machinery already provides crash safety. Rejected.

## Decision outcome

**Option A.** The eight decisions:

1. **Policy — open in place.** A database created by `v0.9.0` opens directly under
   `v0.10.0` with no migration command and no format bump. All primary format versions
   are identical (inventory above). Secondary index state is **reconciled lazily at open**
   by `Embedded::ensure_indexes_current` (`src/sdk/serialization/impl_index.rs:22-31`,
   invoked from `src/sdk/builder.rs:211` and `src/server/bootstrap.rs:385`): persisted
   `schema_version` mismatches or count drift trigger a rebuild of the affected index
   only — never of the VantaFile, WAL, or records. A raw `StorageEngine::open_with_config`
   does *not* run this reconciliation (documented contract: entry points that need query
   indexes call `ensure_indexes_current`).
2. **Migration — none automatic.** No migration is automatic *or* required for `v0.9.0 →
   v0.10.0`. `vanta-cli migrate` remains the tool for genuinely older formats
   (`v < plain_current` → `NeedsMigration`, FIND-499 rules) and for the ADR-046 v1→v2
   records backfill, which a `v0.9.0` base already carries (`CURRENT_SCHEMA_VERSION = 2`).
3. **Backup.** Because no format mutation is needed, no backup is forced at open. Before
   any *optional* mutation — `vanta-cli migrate` on a pre-v0.9.0 base, or the export/import
   encryption path of D6 — the documented prerequisite is `vanta-cli backup`
   (`src/cli.rs:147`, `src/cli_handlers/backup.rs`). On **insufficient disk** the engine
   does not pre-flight a space check: writes fail with the underlying I/O error, index
   rebuilds simply do not persist their state, and the next open retries the
   reconciliation (idempotent by construction). A failed lazy rebuild never leaves the
   database in a state that requires repair — the authoritative data lives in the backend
   and VantaFile.
4. **Power loss.** A power cut during an open-time index rebuild leaves either the old or
   a partial state record; the next open recomputes expected vs. current counts and
   rebuilds again — the check is idempotent and order-independent. The HNSW image is
   written through the atomic `vector_index.bin.tmp` → rename protocol
   (`src/index/serialize/file.rs:294-333`), so a crash can leave a stray `.tmp` but never a
   torn image; WAL replay (`VWAL` range-accepting) restores committed writes regardless.
5. **Rollback to `v0.9.0`.** A plaintext database that `v0.10.0` has only *opened and
   written to* remains openable by `v0.9.0`: every header `v0.10.0` writes in plaintext
   mode carries a version `v0.9.0` already accepts. Rollback is **forfeited** only if the
   database was (re)created or re-written into an encrypted artefact (VFLE 3 / VWAL 4 /
   VNDX 9 / `BACKEND_KV_FORMAT_VERSION` envelope): `v0.9.0` reports those as "newer than
   this software" via `validate_compat` (`src/binary_header.rs:131-149`), a typed
   `IncompatibleFormat`, not silent corruption. Rebuilding secondary indexes does not
   affect rollback — their state versions are unchanged.
6. **Encrypting an existing database.** Still out of the open path (ADR-0057 D7): a key
   configured against an existing plaintext database opens it **plaintext with a
   warning** (`warn_encryption_not_applied`, `src/storage/engine/init.rs:1130`); an
   existing database is never implicitly re-encrypted (`is_new_database`,
   `src/storage/engine/init.rs:1334-1365`). The supported path to encrypt a `v0.9.0` base
   in `v0.10.0` is **export → import into a new encrypted database**. In-place encryption
   of an existing database is **not supported in v0.10.0**.
7. **Future versions.** Anything `v > max(plain, encrypted)` for its artefact class is
   `FormatStatus::Newer` in `check_integrity` and `IncompatibleFormat { found_version }`
   at open — a typed error naming the version, never "corrupt" and never a migration
   target (`src/migration.rs:104-150`, `src/binary_header.rs:131-149`,
   `src/index/serialize/file.rs:106-122`).
8. **Verification.** Fase 2 must produce: (a) a reproducible fixture generator — a script
   that builds the `v0.9.0` tag binary, creates a database covering every artefact in the
   inventory (records + vectors + text + sparse + WAL with rotated segments), and packs
   it; (b) `compat_v090_*` tests asserting at minimum: open-in-place succeeds; reads and
   queries return identical results pre/post open; `check_integrity` reports zero issues;
   `plan_all` is empty; a forced secondary-state rebuild converges and a re-run is a
   no-op; rollback — the same fixture opened+written by `v0.10.0` still opens with the
   `v0.9.0` binary; and a fixture doctored to a future version yields the typed
   `IncompatibleFormat` / `FormatStatus::Newer` path, never a panic.

### Positive consequences

- Zero-friction upgrade: a `v0.9.0` user opens the database and is done.
- The decision is enforced by already-shipped code (`ArtifactSpec::classify`,
  `ensure_indexes_current`, `validate_compat`) — the ADR records behaviour, it does not
  request new machinery.
- Rollback is preserved for the common (plaintext) case, keeping `v0.10.0` reversible.

### Negative consequences

- First open of a `v0.9.0` base can pay secondary-index rebuilds proportional to the
  record count (`O(records)`); acceptable one-time cost, but visible on large bases and
  worth a release note.
- No pre-flight disk-space check means a disk-full rebuild surfaces as a late I/O error;
  documented here rather than silently adding a check.
- `vector_store.vanta` rename and stray `.tmp` files can confuse forensic inspection of a
  crashed open; minor.

### Amendment 2026-11-09 — findings from the Fase 2 verification

The Fase 2 fixture (`tests/fixtures/v0.9.0/`, generated by
`dev-tools/scripts/gen-fixture-v090.ps1` with the real `v0.9.0` tag binary) and the
`compat_v090_*` tests (`tests/compat_v090.rs`) confirmed every decision above. Three
points refine decision 8:

1. **Sparse coverage is not reachable from the `v0.9.0` CLI.** Its `put` subcommand has
   no sparse-vector flag, so the fixture carries records, dense vectors, text, a delete
   and rotated WAL segments, but no sparse entries. The sparse half of D8(a) is covered
   only by the same-value schema rule in the inventory (`SPARSE_INDEX_SCHEMA_VERSION`
   is `1` on both sides); `MANIFEST.json` records this under `coverage_notes`.
2. **A future version can surface as `EncryptionKeyRequired`, not only as
   `IncompatibleFormat`.** In an `encryption` build the format version doubles as the
   encryption marker (VantaFile 3, WAL 4 and index 9 mean "encrypted"), so a header
   doctored to a higher version takes the fail-closed encrypted path first. Both errors
   are typed and neither is "corrupt", which satisfies decision 7. Consequence for
   future formats: **a new plaintext format must not reuse a number already assigned to
   an encrypted variant**; each artefact's plaintext and encrypted version numbers have
   to be allocated together.
3. **The rollback check needs the `v0.9.0` binary.**
   `compat_v090_rollback_fixture_reopened_by_v090_binary` runs only when
   `VANTADB_V090_CLI` points to a `v0.9.0` build (`dev-tools/scripts/run-compat-v090-rollback.ps1`
   provides one); it passed locally on 2026-11-09. In regular CI it is skipped, so the
   permanent guarantee rests on the inventory showing identical plaintext formats.

## References

- [ADR-0057: Encryption at rest](./ADR-0057-encryption-at-rest.md) — D1 artefact set, D7
  no implicit migration to encrypted.
- `docs/dev/tasks/FIND-499.md` §Reglas de versión — the `classify` contract this ADR
  cites.
- `src/migration.rs`, `src/binary_header.rs`, `src/sdk/serialization/impl_index.rs`,
  `src/index/serialize/file.rs` — evidence cited inline per row.
