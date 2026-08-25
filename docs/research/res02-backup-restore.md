# RES-02 — Physical Backup/Restore Gap Analysis

Date: 2026-08-25 · Agent: ox-alpha (research, read-only) · Status: research complete, implementation pending routing

## 1. Current State (verified file:line)

| Component | Location | Notes |
|---|---|---|
| `FsSnapshot` | `src/storage/engine/mod.rs:154` | `{path, created_at}` |
| `create_snapshot` (Unix) | `src/storage/engine/mod.rs:507` | hard links top-level files → `<data_dir>/snapshots/<name>/data/` |
| `create_snapshot` (Win/WASM) | `src/storage/engine/mod.rs:540` | copy fallback, same flat layout |
| `list_snapshots` | `src/storage/engine/mod.rs:569` | sorted dir listing |
| SDK wrappers | `src/sdk/builder.rs:253, :259` | thin delegation |
| MCP tools | `vantadb-mcp/src/handlers/tools.rs:1488 (list), :1502 (snapshot_create, MCP-34a)` | name validated as identifier |
| `snapshot_restore` | **does not exist** (core/SDK/CLI/MCP) | confirmed via codegraph blast radius |
| Cold-copy restore test | `tests/fjall_cold_copy_restore.rs:71` | validates stop→copy→reopen preserves BM25/phrase/HNSW/hybrid + text-index audit |
| WAL archiving / PITR | `src/wal_archiver.rs:56 (WalArchiver), :219 (PitrRestorer)` | **unwired** — no engine call sites |
| Logical export/import | `src/sdk/serialization/impl_export.rs:127–290` | `VantaExportReport`; memory records only |

### Gaps found in `create_snapshot`
1. **Flat copy only**: iterates `read_dir(data_dir)` and links/copies entries where `path.is_file()` (mod.rs:521-527 / :554-560). Subdirectories (e.g. `wal/`) are silently skipped. Must verify actual data_dir layout; if WAL lives in a subdir, snapshots capture only checkpointed state.
2. **No quiescing**: no write-lock acquisition, no `flush()` before imaging. Per-file hard links are atomic, but the file *set* can be torn under concurrent writes (vstore header cursor vs node payloads vs KV partitions). Windows copy path is worse (copies a live mmap).

## 2. Alternative Designs

### (a) Physical restore via directory swap — RECOMMENDED
Semantics: `snapshot_restore(&self, name) -> Result<()>`.
1. Validate `name` as identifier (same guard as MCP-34a) — trust boundary: prevents `../` traversal out of `snapshots/`.
2. Require exclusivity: fail if another process holds the fs2 lock; caller drops handles (embedded API) or server stops serving first (MCP/server API).
3. Safety: move current `data_dir` → `<snap>/pre_restore_<ts>` (rename, atomic same-volume) instead of deleting.
4. Copy `<snap>/data/*` back into a fresh `data_dir`.
5. Reopen engine (`VantaEmbedded::open_with_config`) → HNSW/text index rebuild from storage (already proven by `tests/index_reconstruction.rs`).
Cost: ~200 LOC core + ~40 SDK + CLI/MCP wrappers. Risk: destructive op (mitigated by pre_restore backup); inherits create-time tear risk → prerequisite fixes above.

### (b) PITR via `wal_archiver` — DEFER
`PitrRestorer.restore_to_timestamp` replays `WalRecord`s into a callback. Blockers: zero engine wiring (rotation never calls `archive_segment`); needs base snapshot + log replay (base+log model); `WalRecord` variant coverage for deletes/tombstones unverified. Cost: high (wiring + retention task + replay handler + tests). Value: point-in-time granularity beyond last snapshot. Route as separate backlog item.

### (c) Logical restore via export/import — EXISTS (document only)
`export_all` + `import_file` already roundtrip memory records. Limits: memory-record scope only (no tombstones/graph state), no atomicity, O(n) re-import. Zero new code — document as interim procedure.

## 3. Implementation Plan (atomic steps)

1. **S1**: Fix `create_snapshot` consistency — acquire write quiesce + `flush()` before imaging; add recursive copy/link OR assert-and-document flat-layout assumption (verify data_dir contents first).
2. **S2**: Core `StorageEngine::snapshot_restore` (+ `#[cfg]` split mirroring create): validate name → exclusivity check → rename-aside → copy-back → return path. Failpoint `snapshot_restore_fail` (mirrors `snapshot_create_fail`).
3. **S3**: SDK `VantaEmbedded::snapshot_restore` wrapper + docs (`docs/api/` first — doc-driven development rule).
4. **S4**: Tests: extend pattern of `fjall_cold_copy_restore.rs` (snapshot→mutate→restore→assert retrieval paths + audit pass); torn-write chaos test using `failpoints`.
5. **S5**: CLI `vanta-cli snapshot restore <name>` + MCP `snapshot_restore` tool (MCP-34b) with identifier validation + explicit destructive-op confirmation arg.

## 4. Recommendation

Ship (a) after S1. Keep (b) deferred behind its own wired-archiver task. Document (c) in `docs/operations/` as the zero-code interim backup procedure today.

## 5. Routed Findings (for Backlog)

- **MCP-34b**: `snapshot_restore` MCP tool (after S1-S4 land).
- **FIND (new)**: `WalArchiver`/`PitrRestorer` are dead code — wire segment rotation into engine or remove.
- **FIND (new)**: `create_snapshot` skips subdirectories and does not quiesce writes — correctness gap independent of restore.
