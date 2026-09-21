---
title: "Physical Storage Format Versioning Strategy"
status: implemented
tags: [vantadb, architecture, storage]
last_reviewed: 2026-07-25
aliases: []
---

# Physical Storage Format Versioning Strategy

**Status:** Implemented (WEB-04)  
**Last Updated:** 2026-07-25  
**Related:** `ARCHITECTURE.md`, `DURABILITY_GUARANTEES.md`, `src/binary_header.rs`, `src/schema.rs`

---

> **WEB-04 Implementation (2026-07-25):** Added `VantaHeader::validate_compat()` — a range-based
> compatibility check that replaces exact-match validation across all three storage formats.
>
> ## What was changed
>
> 1. **`src/binary_header.rs`** — `validate_compat()` accepts any `format_version ≤ max_version`
>    with matching magic, enabling forward-compatible reads. Old-format files are still readable
>    by newer software; future-format files (version > max) are rejected with a clear error.
>
> 2. **`src/storage/vfile.rs` (VantaFile):**
>    - `create_in_memory()` and `open_with_mode()` init path now stamp `VFILE_VERSION` (2)
>      instead of hardcoded `1`.
>    - Read validation uses `header.validate_compat(*b"VFLE", VFILE_VERSION, "VantaFile")`
>      instead of manual `!= 1 && != VFILE_VERSION` check.
>
> 3. **`src/index/serialize.rs` (HNSW):**
>    - Already used `validate_compat(*b"VNDX", VECTOR_INDEX_VERSION)` — no changes needed.
>
> 4. **`src/wal.rs` (WAL):**
>    - `WalWriter::open()` now stamps `WAL_FORMAT_VERSION` instead of hardcoded `1`.
>    - `WalHeader::deserialize()` uses `base.validate_compat(*b"VWAL", WAL_FORMAT_VERSION)`
>      for magic + upper-bound validation, replacing manual magic check + missing upper bound.
>    - Lower bound (version ≥ 1) preserved since WAL version 0 was never valid.
>
> 5. **`src/lib.rs`** — Public constants `VFILE_VERSION`, `VECTOR_INDEX_VERSION`,
>    `WAL_FORMAT_VERSION`, `WAL_POSTCARD_VERSION` exported from crate root.
>
> All 1600 tests pass (cargo nextest run --profile audit).

## 1. Current State

VantaDB persists data across four distinct physical formats, each with its own magic bytes, version field, and compatibility semantics.

### Format Inventory

| Format | File(s) | Magic | Current Version | Version Type | Defined In |
|---|---|---|---|---|---|
| **VantaFile** | `vector_store.vanta` | `b"VFLE"` | `2` (const `VFILE_VERSION`) | `u16` format_version | `src/storage/vfile.rs` |
| **Vector Index (HNSW)** | `index.bin` | `b"VNDX"` | `8` (const `VECTOR_INDEX_VERSION`) | `u16` format_version | `src/index/graph.rs` |
| **WAL** | `wal.log`, segments | `b"VWAL"` | `1` | `u32` (cast from `u16`) | `src/wal.rs` (`WalHeader`) |
| **Schema** | `.vanta.schema` | `b"VTDBv001"` | `1` | `u32` schema_version | `src/schema.rs` (`StorageHeader`) |

### Text Index (derived)

The text index uses a feature-gated versioning scheme:

| Feature | Version | Const |
|---|---|---|
| Basic tokenizer | `3` | `TEXT_INDEX_SCHEMA_VERSION = 3` |
| Advanced tokenizer (`advanced-tokenizer`) | `4` | `TEXT_INDEX_SCHEMA_VERSION = 4` |

The text index is a derived materialization (rebuildable from canonical data) and uses an internal prefix `b"\xffvanta_text_v3\0"` for key namespacing rather than a magic-byte header.

### Unified Header (`VantaHeader`)

All physical formats (VantaFile, Vector Index, WAL) share a common 16-byte header defined in `src/binary_header.rs`:

```
Offset  Size  Field
─────────────────────────────────
0       4     magic bytes        (e.g. b"VFLE", b"VNDX", b"VWAL")
4       2     format_version     (u16, little-endian)
6       2     schema_version     (u16, little-endian)
8       8     timestamp          (u64, little-endian, creation epoch ms)
─────────────────────────────────
Total: 16 bytes
```

The WAL extends this with a 4-byte CRC32C checksum for a total `WalHeader` of 20 bytes.

### Schema Header (`StorageHeader`)

The `.vanta.schema` file uses a separate 72-byte header with a different magic (`b"VTDBv001"`):

```
Offset  Size  Field
─────────────────────────────────
0       8     magic bytes        (b"VTDBv001")
8       4     version            (u32, little-endian)
12      4     flags              (u32, little-endian)
16      4     min_compat_version (u32, little-endian)
20      52    reserved (zeroed)
─────────────────────────────────
Total: 72 bytes
```

### Error Handling

Two error variants exist for version incompatibility:

- **`IncompatibleFormat`** (`src/error.rs:134`): magic or version mismatch on any `VantaHeader`-based file. Returns expected vs. found magic/version pairs plus a hint string.
- **`WALVersionMismatch`** (`src/error.rs:115`): WAL-specific version error. Returns expected vs. found `u32` version plus a hint.

---

## 2. Problem Statement

### 2.1 No Backward Compatibility Guarantees

The current `VantaHeader::validate()` performs an **exact match** on both magic bytes and `format_version`:

```rust
// src/binary_header.rs:80
if self.magic != expected_magic || self.format_version != expected_version {
    return Err(VantaError::IncompatibleFormat { ... });
}
```

This means any change to the binary layout — even a forward-compatible field addition — breaks existing databases. There is no concept of "this version can read older versions" or "this version is newer but compatible."

### 2.2 Serialization Format Change Risk

The crate uses `postcard` for all wire serialization (`src/wal.rs`, `src/index/graph.rs`, `src/sdk/serialization/mod.rs`; dep in `Cargo.toml`). This replaced the legacy `bincode` serializer (unmaintained — last release 2021, `bincode 1.3.3`) as part of the WEB-04 work. The ecosystem has since moved to `bincode 2.0` (different API) or alternative frameworks (`rkyv`, `speedy`).

Switching serialization libraries again would change the wire format of:
- Node payloads in VantaFile
- WAL record bodies
- Index metadata blocks

Without a versioning strategy, a serialization change is a silent breaking change.

### 2.2.1 Zero-copy HNSW Serialization (rkyv) — Future Option

The HNSW index (`CPIndex`) currently serializes via postcard — it deserializes on every checkpoint load. For very large indexes (millions of nodes), switching to **rkyv** with `#[repr(C)]` archive structs would allow direct mmap read without deserialization:

```
mmap → cast pointer → use in-place (no copy, no parse)
```

This could reduce startup time and memory pressure at scale. The trade-offs:
- **Upfront cost:** `repr(C)` layout management, unsafe pointer casts, alignment guarantees
- **Migration:** Requires a new index format version + rebuild from canonical storage (HNSW is a derived index)
- **When it matters:** Only when index load time appears in profiles (not a bottleneck today — see `insert_lock` P7)
- **Prior art:** A reference implementation existed at `src/serialization/rkyv_archives.rs` (removed in Jul-11 audit as dead code). Git log has the full implementation.

**Decision:** Do not pursue until index load time is a measured bottleneck. The reference is in git history.

### 2.3 No Migration Path

The `vanta-cli migrate` command exists (`src/cli_handlers/migrate.rs`) but only handles the `.vanta.schema` file version. There is:

- No mechanism to migrate VantaFile contents from v1 to v2
- No mechanism to migrate HNSW index format versions
- No mechanism to migrate WAL format versions
- No rollback plan if a migration fails mid-way

### 2.4 Struct Changes Break Everything

Changes to any binary-serialized struct (adding optional fields, changing field types, reordering) cause silent deserialization failures or data corruption. Current examples:

- `UnifiedNode` in `src/node.rs` — serialized as part of VantaFile
- `Mutation` enum in WAL records — serialized as postcard payloads
- `WalHeader` — hardcoded 20-byte layout

---

## 3. Proposed Solution: Header-Based Versioning

### 3.1 Compatibility Model

Replace the exact-match `validate()` with a **range-based compatibility check**:

| Relationship | Behavior |
|---|---|
| **Exact match** | Open normally |
| **Older file, newer software** | Allow read, warn on write; optionally auto-migrate |
| **Newer file, older software** | Reject with clear error message |
| **Unknown magic** | Reject with "not a VantaDB file" |
| **Version gap > 1** | Reject: "migrate incrementally" (skip one-version migration) |

### 3.2 Version Compatibility Matrix

```
Software Version
    1   2   3   4   5
    ┌───┬───┬───┬───┬───┐
  1 │ ✓ │ ✓ │ ✓ │ ✓ │ ✓ │  (backward compatible reads)
    ├───┼───┼───┼───┼───┤
  2 │ ✗ │ ✓ │ ✓ │ ✓ │ ✓ │
F   ├───┼───┼───┼───┼───┤
i  3 │ ✗ │ ✗ │ ✓ │ ✓ │ ✓ │
l   ├───┼───┼───┼───┼───┤
e  4 │ ✗ │ ✗ │ ✗ │ ✓ │ ✓ │
    ├───┼───┼───┼───┼───┤
  5 │ ✗ │ ✗ │ ✗ │ ✗ │ ✓ │
    └───┴───┴───┴───┴───┘
Forward migration: v1 → v2 → v3 → v4 → v5
No jump migrations (v1 → v4 is rejected)
```

### 3.3 Extended Header Layout

For future formats, extend `VantaHeader` to include a compatibility range:

```
Offset  Size  Field
─────────────────────────────────
0       4     magic bytes
4       2     format_version      (current version)
6       2     min_compat_version  (minimum software version that can read this)
8       2     schema_version      (schema version within format)
10      6     reserved
16      -     payload
─────────────────────────────────
Total: 16+ bytes (extendable via version-dependent payload header)
```

The `VantaHeader` struct already has a `schema_version` field — this can be repurposed as `min_compat_version` for forward compatibility.

### 3.4 Open Behavior by Version

```rust
/// New signature for VantaHeader::validate
pub fn validate_compat(
    &self,
    expected_magic: [u8; 4],
    software_version: u16,
    hint: &str,
) -> Result<CompatResult> {
    // Reject unknown magic
    if self.magic != expected_magic {
        return Err(VantaError::IncompatibleFormat { ... });
    }
    // Reject future versions (newer file, older software)
    if self.format_version > software_version {
        return Err(VantaError::IncompatibleFormat {
            hint: format!("File version {} is newer than this software (max {})", ...)
        });
    }
    // Reject too-old versions beyond compat range
    if self.format_version < self.schema_version  // schema_version used as min_compat
       || self.format_version + MAX_JUMP < software_version
    {
        return Err(VantaError::IncompatibleFormat {
            hint: "Please migrate incrementally".into()
        });
    }
    Ok(CompatResult::Readable(self.format_version < software_version))
}
```

---

## 4. Migration Strategy

### 4.1 `vanta-cli migrate` Command Design

The existing `cmd_migrate` has been extended from schema-only to a **format-level migration runner** with a subcommand interface.

**Command interface:**

```
vanta-cli migrate plan   <target>
vanta-cli migrate run    <target> [--format <format>] [--dry-run] [--force]
vanta-cli migrate check  <target>
```

| Subcommand | Purpose |
|---|---|
| `plan` | Preview migrations without modifying files |
| `run` | Execute migrations to bring formats up to date |
| `check` | Verify storage integrity across all formats |

| `run` flag | Purpose |
|---|---|
| `--format` | Specific format to migrate (vfile, index, wal, schema, all); default: all |
| `--dry-run` | Preview changes without writing |
| `--force` | Skip confirmation prompts |

**Internal flow:**

```
1. Lock database (prevent concurrent access)
2. Read all format headers (VantaFile, Index, WAL, Schema)
3. Build version inventory
4. For each format needing migration:
   a. Validate current version is known
   b. Take a backup marker (rename original, or snapshot)
   c. Read old format data
   d. Write new format data
   e. Verify checksums
   f. On success: remove backup marker
   g. On failure: restore from backup marker, report error
5. Update .vanta.schema version
6. Release lock
```

### 4.2 Forward Migration (v1 → v2)

**VantaFile v1 → v2:**

| Change | Migration Action |
|---|---|
| Add payload checksum footer | Rewrite each record block with appended CRC32C |
| Optional field support | Scan all nodes; fill missing optionals with default sentinel |

**Vector Index v1→v2→v3→v4→v5→v6→v7→v8 (already versioned):**

The vector index already tracks `VECTOR_INDEX_VERSION = 8` (defined in `src/index/graph.rs`). Each version increment should document:
- What changed (e.g., V3 added distance metric byte, V4 added zero-copy aligned vector paging)
- Migration cost (full rebuild vs. in-place header update)

Since HNSW index is a **derived index** (rebuildable from canonical data), the safest migration path for index formats is:
1. Read old index (to serve queries during migration window)
2. Rebuild from VantaFile using new format
3. Atomically swap

**WAL v1 → v2:**

| Change | Migration Action |
|---|---|
| Change serialization from postcard to rkyv (future) | Replay all WAL records, re-serialize with new format |
| Add record-level version tags | Insert version byte after record header |

**WAL migration is high-risk** because WAL is the durability path. The recommended approach is:
1. Drain WAL (ensure all records are checkpointed to backend)
2. Rotate to new WAL file with v2 header
3. Old WAL segments are naturally removed during compaction

### 4.3 Rollback Considerations

| Scenario | Rollback Action |
|---|---|
| Migration fails during file rewrite | Restore from backup marker (original file renamed) |
| Migration succeeds but software downgrade needed | Manual `vanta-cli migrate --target <path> --format <format> --version <N>` |
| Corrupted migration | Rebuild from backup + WAL replay |

**Rollback constraints:**
- WAL segments written with new format cannot be read by old software
- Index files written with new format trigger `IncompatibleFormat` on old software
- VantaFile v2 is backward-compatible for reads (old software can read v2 if it uses range-based validation)

### 4.4 Testing Strategy for Migrations

1. **Unit tests**: every format-level migration function tests:
   - Empty file
   - Single-record file
   - Multi-record file
   - Corrupted file
   - Partial-write file
   - Version boundary (v1 → v2, v2 → v1 rejection)

2. **Round-trip tests**: write with old version, migrate to new, read with new, verify data integrity

3. **Property-based tests**: generate random databases, migrate, verify all nodes survive

4. **Integration tests**: full `vanta-cli migrate` with real database directories

5. **Stress tests**: concurrent readers during migration (must hold exclusive lock)

---

## 5. Implementation Plan

### Phase 1: Add Version Header to All Formats

**Goal:** Every physical file can report its format version on open.

| Task | Files | Status |
|---|---|---|---|
| Add `VantaHeader` to any format missing it | N/A (all formats already have it) | ✅ Not needed |
| Replace exact-match `validate()` with range-based `validate_compat()` | `src/binary_header.rs` | ✅ Done |
| Add `CompatResult` enum | `src/binary_header.rs` | Skipped (simpler: `Result<()>` with `IncompatibleFormat` error) |
| Update all callers of `validate()` to handle compat | `index/serialize.rs` | ✅ Done (HNSW uses `validate_compat`) |
| Add format version constants to public API | `src/lib.rs` | ✅ Done |
| Add `min_compat_version` field to `VantaHeader` | Skipped — schema_version field repurposed for this | Skipped (not needed for range check) |

### Phase 2: Implement Migration Runner

**Goal:** `vanta-cli migrate` handles all physical format migrations.

| Task | Files | Effort |
|---|---|---|
| Refactor `cmd_migrate` into a `MigrationEngine` struct | `src/cli_handlers/migrate.rs` → `src/migration.rs` | Medium |
| Implement VantaFile migration (v1 → v2 with CRC32C footer) | `src/migration.rs` | Large |
| Implement Vector Index migration (rebuild-based) | `src/migration.rs` | Medium |
| Implement WAL migration (drain + rotate) | `src/migration.rs` | Medium |
| Add `--format`, `--dry-run`, `--force` flags | `src/cli.rs` | Small |
| Add rollback (restore-from-backup) support | `src/migration.rs` | Medium |

### Phase 3: Test with Real Databases

**Goal:** Verified migration works on diverse database states.

| Task | Files | Effort |
|---|---|---|
| Create test databases at each format version | `tests/migration/` | Medium |
| Write round-trip migration tests | `tests/migration/` | Medium |
| Write property-based migration tests | `tests/migration/` | Large |
| Benchmark migration throughput | `benches/migration.rs` | Small |
| Document migration SOP in operations docs | `docs/operations/` | Small |

### Estimated Timeline

| Phase | Duration | Dependencies |
|---|---|---|
| Phase 1 | 2-3 days | None |
| Phase 2 | 5-8 days | Phase 1 |
| Phase 3 | 3-5 days | Phase 2 |

---

## 6. Open Questions

1. **Should `VantaHeader` `schema_version` be renamed to `min_compat_version`?** This would make its semantics clearer but break existing serialized files. A new `VantaHeaderV2` layout could include this change.

2. **Should text index (BM25) get magic-byte header?** Currently it uses a prefix `b"\xffvanta_text_v3\0"` in the KV store key namespace. Adding a proper header would make versioning explicit but requires a migration of all existing text index data.

3. **How to handle cross-format atomicity?** A migration touching VantaFile, Index, and WAL must either succeed completely or roll back completely. The lock-based approach prevents concurrent access but does not guarantee filesystem-level atomicity.

4. **Should we pin a serialization library before Phase 2?** If the team decides on `rkyv` or `postcard`, the migration code should produce the new format directly rather than engineering a two-step migration (v1 layout → v2 same-serializer → v3 new-serializer).
