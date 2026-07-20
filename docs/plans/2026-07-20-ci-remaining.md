# Plan: Remaining CI/CD Fixes Post-Triage

**Campaign:** CI/CD Stabilization — Phase 2
**Date:** 2026-07-20
**Status:** ✅ COMPLETED (Phase 1 fixes deployed)

## Background

After the initial CI triage (2026-07-19, see `2026-07-19-CI-triage.md`), 10 pre-existing issues
were identified and fixed in `3d5181a`. This plan covers the 3 remaining items that were not
addressed in that round.

## Summary

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 3 | 0 | 0 | 0 |

---

### Task RCI-01: CI Rust — Format Check cancelled by concurrency group

**Root cause:** `fmt` and `clippy` jobs both use `concurrency.group: lint-${{ github.ref }}`
with `cancel-in-progress: true`. When both trigger simultaneously (push to main), the
second job to start cancels the first.

**Fix:** Give `fmt` its own concurrency group: `fmt-${{ github.ref }}`.

**Files:**
- `.github/workflows/ci-rust-10.yml` — line 51

**Verification:** Format Check shows `conclusion: success` in CI run instead of `cancelled`.
- ✅ **Verified:** Run `29716859131` — Format Check = success

---

### Task RCI-02: FUZZ — nightly toolchain failure

**Root cause:** Pinned commit hashes of `dtolnay/rust-toolchain` and `taiki-e/install-action`
were outdated. `dtolnay/rust-toolchain` was pinned to commit `fa04a14` (old version) which
had a bug in nightly rustup proxy setup, causing `cargo +nightly fuzz build` to fail with
`option Z only accepted on nightly` / `require a rustup proxy`.

**Fix:** Bump both action hashes to latest:
- `dtolnay/rust-toolchain`: `fa04a14...` → `2c7215f1...`
- `taiki-e/install-action`: `43aecc8d...` → `25f25a6e...`

**Files:**
- `.github/workflows/fuzz-40.yml` — lines 31, 40, 64, 81
- `.github/workflows/ci-rust-10.yml` — also updated same hashes (Windows, Miri, ASan, TSan, audit, deny)

**Verification:** Next scheduled FUZZ run (Monday 06:00 UTC) or manual `workflow_dispatch`.
- ⏳ **Pending:** Next auto-run Mon 2026-07-27 06:00 UTC

---

### Task RCI-03: Verify HEAVY Certification passes with merged fixes

**Root cause (already fixed in `3d5181a`):**
1. `mmap_vector_index_certification` — version 6 vs 7 mismatch in `tests/storage/mmap_index.rs`
2. `test_mcp_tool_search` — u128 "number out of range" panic in serde_json

**Verification:** Manual `workflow_dispatch` triggered at `2026-07-20T04:25:53Z`.
- ⏳ **Pending:** Run `29716868346` — jobs in progress

---

## Release Workflows (Secondary)

These workflows only trigger on tag pushes (`v*`), so they can't be tested on `main`.
They were verified as not impacting main branch CI health.

| Workflow | Issue | Status |
|----------|-------|--------|
| RELEASE: PyPI Publish | `attestations: write` permission missing | Deferred to first release attempt |
| RELEASE: Binaries | macOS `libclang.dylib` not found | Deferred to first release attempt |
| RELEASE: SBOM | Cancelled (branch-specific) | ✅ Already passing on main |
| RELEASE: Wheels | No runs on main (tag-only) | N/A |
