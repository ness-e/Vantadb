---
title: "VantaDB Studio — technical reconciliation spec (develop-desktop vs develop)"
kind: concept
status: draft
description: "Merge-base inventory, API-break map, compile plan and packaging/signing workflow design for reconciling VantaDB Studio (develop-desktop) with develop"
tags: [vantadb, architecture]
---

# VantaDB Studio — technical reconciliation spec

> **Scope (FIND-645).** This is a *technical* reconciliation specification for
> the `develop-desktop` worktree (VantaDB Studio, Tauri v2) against `develop`.
> **Licensing is a business decision and is explicitly out of scope.** The
> branch freeze (`develop-desktop` congelado hasta 2026-12-01) applies to the
> desktop code; writing this spec does not modify the frozen tree.
>
> Backlog row: `docs/dev/Backlog.md` `FIND-645` — "164 commits de retraso y
> sin CI de empaquetado ni **firma** de instaladores (`.msi`/`.dmg`/`.AppImage`)".
> Verified again on 2026-11-19: the drift is now **258 commits**, not 164.

## 1. Verified current state

| Fact | Value | Source |
|---|---|---|
| Merge-base `develop`/`develop-desktop` | `2b821184a` (2026-10-29) | `git merge-base` |
| `develop-desktop` HEAD | `0ad38c542` — +1 commit, **−258** vs `develop` | `git rev-list` |
| Desktop code (committed) | `feature-freez/desktop/` | `git ls-tree develop-desktop` |
| Desktop code (working tree) | `desktop/` — **staged, uncommitted rename** `feature-freez/desktop → desktop/` | `git status` in worktree |
| Tauri crate | `vantadb-desktop 0.1.0`, own `[workspace]`, `crate-type staticlib+cdylib+rlib` | `desktop/src-tauri/Cargo.toml` |
| Engine dependency | `vantadb = { path = "../.." }` + `vanta-memory = { path = "../../vanta-memory" }` | same file |
| Bundle targets declared | `nsis`, `msi`, `dmg`, `app`, `appimage`, `deb` | `desktop/src-tauri/tauri.conf.json` `bundle.targets` |
| Sidecar binaries | `binaries/vanta-cli*`, `binaries/vantadb-server*` bundled as resources | same file |
| Updater | present but empty (`endpoints: []`, `pubkey: ""`) | same file |
| Build/sign CI | **none** — zero `tauri` hits in `.github/workflows/` on either branch | `grep` |

> [!WARNING] Path resolution bug already latent
> `vantadb = { path = "../.." }` resolves to the repo root **only** from
> `desktop/src-tauri/`. Under the *committed* layout
> (`feature-freez/desktop/src-tauri/`) it resolves to `feature-freez/`, which
> has no `Cargo.toml` — the crate cannot build as committed. The staged rename
> to `desktop/` is therefore **mandatory**, not cosmetic: reconciliation
> starts by finalising it (or fixing the path to `../../..`).

## 2. What `develop` gained since the merge-base (API/behaviour break inventory)

Verified against `develop` on 2026-11-19. Canonical compat reference:
[ADR-0060](./adr/ADR-0060-on-disk-compatibility-v090-v0100.md) (on-disk) and
[ADR-0057](./adr/ADR-0057-encryption-at-rest.md) (encryption design). One fact,
one file — this table *links* to those inventories, it does not copy them.

### 2.1 On-disk format (encryption at rest)

| Artefact | merge-base → develop | Desktop impact |
|---|---|---|
| VantaFile segments | v2 → **VantaFile v3** (`VFILE_ENCRYPTED_VERSION`, encrypted variant) | A desktop built **without** `encryption` cannot open encrypted DBs; it must fail with a legible error or ship the feature (§4.3) |
| WAL | v3 → **WAL v4** (`WAL_ENCRYPTED_FORMAT_VERSION`, per-frame AEAD + hash chain) | Same gate. Plaintext v3 WAL is still readable (ADR-0060) |
| `vector_index.bin` | v8 → v9 (encrypted variant) | Same gate; plain v8 reads or lazily rebuilds |
| `read_only` | now honoured by the Fjall backend (FIND-633) | Desktop "read-only connect" mode becomes honest — retest |

Opening a *plaintext* v0.9.0 database with current `develop` is in-place and
non-destructive (lazy index rebuild). The break is only when the user has an
**encrypted** base — a new capability the desktop UI does not know about.

### 2.2 Rust public API

| Change | Symbol / commit | Desktop impact |
|---|---|---|
| **New** `Embedded::forget` | `ForgetMode` {Soft, Hard, Certified}, `ForgetTarget` {Key, Filter, Query}, `ForgetReport` (`src/sdk/api/memory.rs`, `928df0970`, FFI parity `3292c6507`) | New capability worth surfacing in the UI (GDPR erasure); no compile break |
| `chrono` → `jiff` | FIND-652 `f17caf906`: `CoreValue::DateTime` payload is now `jiff::Timestamp` | **Compile break confirmed**: `desktop/src-tauri` converts with `dt.to_rfc3339()` — `jiff::Timestamp` has no such method. Fix: `vantadb::sdk::serialization::tsfmt::to_rfc3339(dt)` (chrono-compatible `+00:00`) or `dt.to_string()` |
| `src/engine.rs` deleted | engine unified under `StorageEngine` (`928df0970`, −1137 LOC) | Any `vantadb::engine::*` path breaks (none found in desktop today — it uses `vantadb::sdk::Embedded`, which survives) |
| Index scale/dimension unification | `c76dde1bf` (`fix(index)!`) + `de45d5961`: single vector dimension per DB → `Error::DimensionMismatch` | UI must surface the new write-time rejection instead of treating it as generic Io |
| Embeddings arm fails loud | `de45d5961` (`fix(embeddings)!`): vector search with a dead provider returns an error, not 0 hits | Desktop must distinguish "no results" from "provider down" in the search view |
| `memory_put` enrichment | `0e86d2dc5`: `confidence`, `confidence_class`, `derived_from` fields | Optional UI surface |
| L1 episodic memory + GDPR cascade | `vanta-memory`: `l1_writer`/`l1_reader` (write-first), `cascade_forget`, `613881a86` | 53 files changed; sampled desktop-used symbols still resolve (`import_seed_str`, `get_persona`, `current_scene`, `list_scenes`, `upsert_scene`, `SceneBlock`, `edit_scene_tool`, `write_scene_tool`, `StoredSkill`, `GenerationStatus`, `try_record`, `SceneIndexEntry`) — signature drift needs the compile probe (§4.1) |

### 2.3 Engine-side surface the desktop consumes (sampled, still present on `develop`)

`vantadb::sdk::{Embedded, OperationalMetrics, SearchProfileConfig,
SearchProfileMode, MemoryListOptions, MemoryListPage, FilterOp}`,
`vantadb::config::Config`, `vantadb::error::Error`,
`vantadb::parser::autocomplete_prefix`, `vantadb::node::SparseVector`,
`vantadb::graph::TraversalDirection`, `vantadb::llm::LocalOnnxProvider`,
`vantadb::audit::AuditEvent`. `Embedded::{open, open_with_config, test_empty,
restore_from, create_snapshot, operational_metrics, query, graph_bfs,
graph_dfs, graph_degree_centrality, in_memory}` all verified present.
Feature flags used by the crate (`fjall`, `fs2`, `memmap2`, `roaring`,
`advanced-tokenizer`, `embed-local`) still exist; `encryption` is a new opt-in.

## 3. Desktop-side audit notes

- `tauri.conf.json` already documents unsigned macOS/Linux bundles ("no
  notarization — no Apple certificate in CI"). Signing strategy is §5.
- Dependabot: the FIND-645 task notes carry "15 alertas" for
  `feature-freez/desktop`; visible open branches today are 3
  (`rustls-0.23.46` cargo, `source-map-js-1.2.2`, `vite-7.3.7` npm) —
  re-triage during the rebase, not in this spec.
- `dev-dependencies` on `axum 0.7` + tokio for tests — keep them out of the
  production bundle profile.

## 4. Compile plan (`src-tauri` against current `develop`)

### 4.1 Mechanical steps

1. **Finalise the rename**: commit `feature-freez/desktop → desktop/` (it is
   already staged) — otherwise `vantadb = { path = "../.." }` does not resolve.
2. **Rebase choice**: `develop-desktop` is a 2-commit tree over the
   merge-base. Prefer *replay* (`git rebase --onto develop 2b821184a
   develop-desktop` for the restore commit + a fresh commit for the move)
   over merge — keeps the desktop delta reviewable (~1 directory).
3. **Compile probe**: `cargo check` inside `desktop/src-tauri` (own workspace)
   against `develop`. Expected fix list (minimal, evidence-based):
   - `dt.to_rfc3339()` → `vantadb::sdk::serialization::tsfmt::to_rfc3339(dt)`
     (keeps chrono's `+00:00` wire form; `dt.to_string()` emits `Z` — a
     different wire string, pick deliberately).
   - Error enum drift: re-verify `vantadb::error::Error` variants the desktop
     maps (`NodeNotFound`, `DatabaseBusy`, `Io`, new `DimensionMismatch`).
   - `MemoryInput`/`ListResult`/search request fields added since 10-29
     (defaults exist; only named-field construction breaks).
   - `vanta-memory` signature drift (§2.2 list) — compile probe is the arbiter.
4. **Gates**: `cargo check`, `cargo test`, `npm run build` (vite), `vitest`,
   Playwright e2e — run in the desktop worktree before touching CI.

### 4.2 Engine feature matrix decision

| Option | Consequence |
|---|---|
| Keep `encryption` **off** | Smallest diff; encrypted DBs must fail with a legible error (the engine already returns typed errors — surface them, never "corrupt DB") |
| Enable `encryption` + key UX | Master key via OS keyring (`tauri-plugin-stronghold`/`keyring`) or passphrase prompt; adds the `encryption` feature + UX for unlock/rotate (`encryption verify-key` exists on the CLI since FIND-646) — a product decision, documented here as the natural long-term answer |
| Enable `embed-local` | ONNX session now counts against the RAM governor (FIND-642); ISA floor must be enforced at installer level — `SIGILL` on pre-AVX2 CPUs is a real packaging hazard (FIND-643). Publish a CPU baseline in the installer/release notes |

### 4.3 Sidecar binary coherence

`binaries/vanta-cli*` and `vantadb-server*` are bundled. They must be **the
same engine version** as the embedded crate — rebuild from the reconciled
`develop` (or consume `release-binaries.yml` artifacts of the same tag). Add a
version-coherence check (compare `vanta-cli --version` output against the
crate version at bundle time); a stale sidecar is a silent data-format hazard.

## 5. Packaging & signing workflow (design)

No workflow exists today. Proposed `desktop-release.yml`, aligned with
`docs/dev/workflow/PUBLISH.md` conventions (tag namespaces, draft-first — the
runbook is git-ignored local tooling, hence a code-span reference, not a link):

| Aspect | Design |
|---|---|
| Trigger | manual `workflow_dispatch` first; tag `desktop-v*.*.*` when stable — a **new tag namespace**, same pattern as `node-v*`/`providers-v*` |
| Matrix | `windows-latest` (`.msi` via WiX + `.nsis`), `macos-latest` (`.dmg`/`.app`, universal2 or arm64+x86_64), `ubuntu-22.04` (`.AppImage` + `.deb`) |
| Build | `npm ci` → `npm run build` → `cargo tauri build` with the reconciled lockfile; sidecars from §4.3 staged before bundling |
| Windows signing | Authenticode via `signtool` (cert in GitHub secrets) or a cloud signer (Azure Code Signing / eSigner) — needs cert procurement (owner decision) |
| macOS signing | Developer ID Application + `notarytool` submit/staple — needs Apple Developer account (owner decision); until then keep current unsigned `.dmg` with a loud caveat |
| Linux | unsigned + `sha256sums.txt` per release (documented verification) |
| Updater | `updater.endpoints`/`pubkey` are empty — either **remove** the updater block or provision a `tauri signer` keypair and a JSON endpoint; an enabled-but-empty updater is dead config |
| Gates | `cargo check` + `cargo test` + `vitest` before bundling; post-bundle smoke: install the artifact on the runner and `--version` probe |
| Publish | draft GitHub Release `desktop-v*.*.*` + artifacts + checksums; publish only on explicit owner approval (mirrors repo's push rules) |
| Version | `productName`/`version` in `tauri.conf.json` must track the workspace version — add a `check-docs`-style assertion so `0.1.0` cannot ship against `0.10.x` |

## 6. Ordered work register

| # | Item | Depends on | Owner decision needed |
|---|---|---|---|
| 1 | Commit the `feature-freez/desktop → desktop/` rename | — | no |
| 2 | Rebase `develop-desktop` onto `develop` (replay) | 1 | no |
| 3 | Compile probe + minimal fix list (jiff, error variants, signatures) | 2 | no |
| 4 | Encryption UX decision (off+legible-error vs keyring) | 3 | **yes** |
| 5 | Updater decision (remove vs provision keypair) | — | **yes** |
| 6 | `desktop-release.yml` + cert procurement | 3 | **yes** (certs, Apple account) |
| 7 | Dependabot triage on `desktop/` paths | 2 | no |
| 8 | Licensing | — | **business — out of scope** |

## 7. Residuals (explicit)

- **Unverified until the compile probe runs**: signature-level drift in
  `vanta-memory` (53 changed files) and any `MemoryInput`/search request
  named-field construction inside `src-tauri`. Symbol-level checks pass;
  signatures are the probe's job.
- The `swarm-*` worktrees listed by `git worktree list` carry unrelated
  in-flight work; none shares `desktop/` paths.
- `docs/dev/Backlog.md` says "164 commits" — measured 2026-11-19 at **258**;
  the delta grows with every `develop` merge until this work lands.
