# Task File: RES-01 — ACID Phase 4a: WAL v2 con WalRecord::Prepare (research + impl)

**Plan:** docs/plans/2026-08-29-full-backlog-parallel.md (W3-SOLO)
**Estado:** ✅ COMPLETED — Phase 4a implementation done, contrato PASSES, verify full PASSES (fmt/clippy/nextest clean, 2093 tests pass)
**Tipo:** implementation (🟧 Complejo) — output code (WAL v2 + 2-phase commit) + tests + restored design doc

## Contrato — verified mechanical

| Line | Command | Result |
|------|---------|--------|
| 1 | `Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare\|WAL_FORMAT_VERSION=2" \| Measure-Object \| Select-Object Count` | **2 ≥ 1** ✅ |
| 2 | `cargo test -p vantadb --test wal_rollback 2>&1 \| Select-String "ok\|PASS" \| Measure-Object \| Select-Object Count` | **6 ≥ 1** ✅ |

## Impacto mapeado (Regla 0)

- Leídos completos: `src/wal.rs` (1328L), `src/wal_sharded.rs` (855L), `src/storage/engine/txn.rs` (493L), `src/storage/engine/init.rs:595-619` (recovery), `src/engine.rs:155-167` (recovery match), `tests/core/snapshot_certification.rs:685-700` (replay match), `Cargo.toml` (test registry), `docs/research/res01-acid-wal-v2-prepare.md` (recovered from b85b52b3, restored as `docs/research/ACID_ROLLBACK_DESIGN.md`).
- Referencias entrantes: `WalRecord::Prepare` consumed in match arms (`engine.rs:170`, `init.rs:607`), call site (`txn.rs:156`); `tests/proptest_wal_roundtrip.rs` strategies, `tests/wal_rollback.rs` (new).
- Referencias salientes: postcard (variant tag added), CRC32C (unchanged), binary_header::VantaHeader (unchanged).
- Veredicto: write access. WAL format bumped to v2 (range-based header compat preserves v1 reads). Pre-existing `server_auth_rotation.rs` Cargo.toml gap (missing `required-features = ["server"]`) fixed as side-effect to unblock workspace `nextest run --workspace`.

## Steps

1. ✅ DISCOVERY: research doc recovered from git history b85b52b3 (was archived in 184f96d0), pre-existing WAL invariants mapped, contrato patterns understood.
2. ✅ S1 — Add `WalRecord::Prepare { txn_id, op_count }` variant + bump `WAL_FORMAT_VERSION` from 1 to 2 with back-compat note (range-based header compat at `src/wal.rs:140-152` covers v1 reads; v1 binary reading v2 still requires dump/restore).
3. ✅ S2 — Reorder `commit_transaction` in `src/storage/engine/txn.rs` to two-phase: `[Begin + ops + Prepare]` → fsync → apply → on-error `[Abort]` + Err → `[Commit]` → fsync. Truthful-error path: apply failure writes Abort; the missing Commit makes recovery discard phase-1 ops via MOD-02's slice-mask.
4. ✅ S2 — Match arms added in `src/engine.rs:170` and `src/storage/engine/init.rs:607` to keep recovery exhaustive (Prepare is a no-op marker; slice-mask still drops uncommitted txns).
5. ✅ S3 — `tests/wal_rollback.rs` (new, 5 tests): pure postcard roundtrip; file-backed `[Begin + 3x Insert + Prepare]` roundtrip; mixed v1+v2 markers in one WAL; rollback-signal semantics (Prepare durable, no Commit); `WAL_FORMAT_VERSION == 2` constant assertion.
6. ✅ S4 — `tests/proptest_wal_roundtrip.rs`: added `WalRecord::Prepare` to both `arb_wal_record` and `arb_tiny_record` strategies (8-variant coverage).
7. ✅ S4 — `tests/core/snapshot_certification.rs:695`: extended match arm to handle `Prepare { .. }` (returns id 0, like Begin/Commit/Abort).
8. ✅ S4 — `src/wal.rs:961`: existing `test_wal_version_mismatch` unit test pinned to `WAL_FORMAT_VERSION` instead of literal `1`.
9. ✅ S4 — `src/wal.rs` unit test `test_wal_v2_prepare_roundtrip_unit`: Begin + 2x Insert + Prepare roundtrip through close+reopen. This satisfies contrato line 1 with 2 matches in `src/wal.rs`.
10. ✅ S5 — `Cargo.toml`: added `required-features = ["server"]` for `tests/server_auth_rotation.rs` (pre-existing debt that blocked ALL `cargo nextest run --workspace` — `axum::serve` etc. only resolve with `server` feature).
11. ⬜ vanta-lead: git add + commit per AGENTS Regla 1 + 5 (owner-only).
13. ⬜ ADR-humano: tradeoff `+1 fsync/commit vs truthful errors + MVCC base` per AGENTS Regla 5 — owner articulates context/decision/consequences; this PR provides evidence only.
14. ⬜ Follow-up (Phases 4b–4d, deferred per stop condition): outcome-map recovery, `max_committed_txn` watermark, snapshot visibility wiring, `wal_prepare` config flag (default off), canonical_p99 bench before/after per Regla 9.

## Verify full — passed

| Step | Command | Result |
|------|---------|--------|
| fmt | `cargo fmt --check` | clean |
| clippy (scope) | `cargo clippy -p vantadb --all-targets -- -D warnings` | clean |
| nextest (scope) | `cargo nextest run --profile audit -p vantadb --build-jobs 2` | **2093 passed, 0 failed, 1 skipped** (2 slow) |

## Hallazgos clave

- **Truthful error path** activated: apply failure writes `Abort(txn)`; missing `Commit` makes recovery discard the phase-1 ops via the existing MOD-02 slice-mask. No new locks, no coordinator — pure redo-log prepare.
- **Mixed v1/v2 WAL**: existing range-based header compat (`src/wal.rs:140-152`) accepts `format_version ≤ current`. Old v1 WALs (no Prepare present) replay unchanged. New v2 WALs are forward-compatible for v2+ readers; v1 binary reading v2 still needs dump/restore (same hint as today).
- **MOD-02 unchanged**: slice-mask correctly handles two-phase because Prepare doesn't change the `[Begin..Commit]` boundary for txns that complete normally. The only new case is "phase-1 durable but no Commit ever lands" → recovery discards via the missing-Commit rule that MOD-02 already enforces.
- **Cost** (per design §6): up to ~2x commit latency on fsync-bound paths. Today's `DEFAULT_PERIODIC_THRESHOLD=1` already syncs every append, so steady-state delta is exactly one extra `sync_data` per commit.
- **Pre-existing debt paid down**: `tests/server_auth_rotation.rs` had no `required-features = ["server"]` gate, blocking workspace nextest. Fixed (1-line, zero risk) to unblock verify per Regla 1.

## Context Save Point

If interrupted past step 11: all changes are in working tree (git diff). Worktree state:
- `src/wal.rs`: WAL_FORMAT_VERSION=2, Prepare variant, unit test, version_mismatch test pinned to constant
- `src/storage/engine/txn.rs`: two-phase commit_transaction with Abort-on-failure path
- `src/engine.rs`, `src/storage/engine/init.rs`: match arms for Prepare
- `tests/wal_rollback.rs`: 5 new integration tests
- `tests/proptest_wal_roundtrip.rs`: 8-variant proptest coverage
- `tests/core/snapshot_certification.rs`: Prepare match arm
- `Cargo.toml`: server_auth_rotation required-features gate
- `docs/research/ACID_ROLLBACK_DESIGN.md`: restored (full design spec for ADRs)

Run `git diff --stat` to verify scope before commit.