---
title: Compile feature matrix — the combinations the project promises, and what CI covers
kind: concept
status: active
description: "Which Cargo feature combinations VantaDB promises to build, which gate compiles or tests each one, and the measured runner cost of the gates"
tags: [vantadb, features, cargo, ci]
---

# Compile feature matrix

One question this page answers: *if I turn these Cargo flags on, is that combination
built anywhere before a user hits it?*

It does **not** repeat the per-feature registry (what each flag gates, with `file:line`)
— that is [FEATURES.md](./dev/architecture/FEATURES.md) — nor the product boundary
(core-promise vs labs) — that is
[EXPERIMENTAL_FEATURES.md](./user/operations/EXPERIMENTAL_FEATURES.md) — nor the CI tier
policy — that is [CI_POLICY.md](./dev/operations/CI_POLICY.md).

## 1. Inventory (measured against the tree, 2026-11-27)

| Crate | `[features]` | Count |
| --- | --- | --- |
| `vantadb` (core) | `Cargo.toml:155-199` | **27** (`default` + 26 opt-in) |
| `vanta-memory` | `vanta-memory/Cargo.toml:61-102` | 8 |
| `vantadb-server` | `vantadb-server/Cargo.toml:66-84` | 8 |
| `vantadb-wasm` | `vantadb-wasm/Cargo.toml:46-50` | 3 (`default` = `tracing-wasm`) |
| `vantadb-mcp` | `vantadb-mcp/Cargo.toml:40-43` | 2 |
| `vantadb-python`, `vantadb-node`, `vantadb-ffi-core` | — | none (fixed feature sets) |

**`default` is five flags, not nine:** `cli`, `fjall`, `memmap2`, `fs2`, `roaring`
(`Cargo.toml:156`). `arrow`, `advanced-tokenizer`, `sysinfo` and `rayon` are **not**
default, despite the stale aggregate row in `FEATURES.md:32` (it also cites a
`ci-rust-10.yml` that no longer exists; the live workflow is `ci-rust.yml`). Same class of
drift at `FEATURES.md:51`: `vantadb-python/Cargo.toml:25` does **not** pass
`default-features = false`, so the wheel is built with the defaults **on** plus
`fjall, memmap2, rayon` — net effect `default + rayon`.

27 binary flags is 2²⁷ combinations. That is not a number to cover; it is a reason to
publish the short list of combinations somebody actually promises.

## 2. The combinations the project promises

"Covered" means what the named gate really does — `check` compiles, `test` runs.
Durations are job wall-clock from run `45949538624` (`gh api …/runs/45949538624/jobs`,
warm sccache).

| # | Combination | Features on top of `default` | Who needs it (evidence) | Gate that covers it today | Covered |
| --- | --- | --- | --- | --- | --- |
| 1 | `default` | — | Every `cargo add vantadb` consumer; docs examples (`Cargo.toml:156`) | `MSRV Check` — `cargo check --locked` (1.8 min) | ✅ compiled. Runtime only as a subset of #2 |
| 2 | `test-lane` | `arrow`, `tls`, `opentelemetry` | The CI contract itself; `tls`/`opentelemetry` are documented operator flags (`DEPLOYMENT_GUIDE.md:410,472`) | `Tests (Linux)` 8 min · `Tests (Windows)` 30 min · `Tests (macOS)` 11.4 min · `ASan` 7.9 min · `Coverage` 8 min | ✅ compiled **and** tested, 3 OS |
| 3a | `remote-inference` | `remote-inference` | LLM providers (labs); `AI_IDES.md:78` | `Tests (embeddings features)` leg `remote-inference` — `--lib` (3.3 min) | ✅ lib tests only |
| 3b | `embed-local` | `embed-local` | `QUICKSTART.md:261`; core-promise per `EXPERIMENTAL_FEATURES.md` | Same job, leg `embed-local` — `--lib` (3.7 min) | ✅ lib tests only |
| 4 | `miri` | **`--no-default-features`** + `cli,fjall,memmap2,fs2,encryption` | The only gate that compiles **without** `roaring` (`croaring` is C FFI, unrunnable under Miri) | `Miri (UB Detection)` (3.9 min) | ✅ compiled + executed |
| 5 | `release-binary` | `server`, `embed-local`, `jemalloc` (unix) / `custom-allocator` (windows) | **What users download**: `release-binaries.yml:120-127` | `Release Combo (server + allocator, -D warnings)` (1.6 min) | 🟡 **partial** — the gate checks `server,jemalloc` in the release profile, **without `embed-local`** |
| 6 | `server-full` | `cli,server,tls,opentelemetry,advanced-tokenizer,remote-inference` | The documented self-hosted build (`DEPLOYMENT_GUIDE.md:65`) | `Feature Combo (server-full)` — added by FIND-219 | ✅ compiled (job `check` only) |
| 7 | `python-sdk` | `vantadb_py` crate: `default` + `rayon` (see §1) | PyPI wheel, core-promise | Built and tested by the three `Tests (*)` lanes (`-p vantadb_py`); `maturin build` in `ci-fleet.yml` | ✅ |
| 8 | `wasm` | core `--no-default-features --features wasm`, `--target wasm32-unknown-unknown`; crate `tracing-wasm` (+ `opfs`) | npm `vantadb-wasm` (labs) | `WASM (wasm-pack browser tests)` — path-gated and `continue-on-error` | 🟡 runs, **does not block** |
| 9 | `minimal-cli` | `--no-default-features --features cli` | **Nothing.** No README, QUICKSTART, DEPLOYMENT_GUIDE or workflow asks for it | — | ❌ not covered, and deliberately so: a combination nobody ships is a combination nobody should pay a runner for |

Combinations 1–6 are the supported set. 7 and 8 are supported surfaces with their own
pipelines. 9 is named in the finding that produced this page and is answered here: it is
**not** a promise, so it gets no gate.

## 3. Combinations that are "unsupported" and how they resolve

There is no `compile_error!` guard anywhere in the tree (0 hits in `src/`,
`vantadb-wasm/src/`, `vantadb-server/src/`), so every combination below either *works by
configuration* or fails with a generic compiler/linker message. That distinction is the
whole point of this section: a combination that is unsupported silently is a debugging
session for the user.

| Combination | What actually happens | Verdict |
| --- | --- | --- |
| `jemalloc` + `custom-allocator` | Resolved by `cfg` precedence, not an error: jemalloc wins on unix, mimalloc on Windows (`src/bin/vanta-cli.rs:13-22`) | ✅ tolerated, allocator chosen deterministically |
| `custom-allocator` on Windows alone | mimalloc applies (`src/bin/vanta-cli.rs:18-22`) | ✅ supported |
| `roaring` under Miri | Compiles; the C FFI aborts at runtime under Miri — that is why the Miri gate excludes it (§2 row 4) | ❌ not runnable under Miri, by design |
| `python_sdk` (core flag) | Unused by the wheel (core is consumed with default features, §1). Enabling it links `pyo3` into a build that is not an extension module | ⚠️ dormant path, no consumer |
| `server` without `cli` | `server` does not depend on `cli` in `Cargo.toml`, but `FEATURES.md:48` records "requires `cli`". Not verified by any gate | ⚠️ **unverified** — declared, not claimed |
| `wasm` on a non-wasm target | No `#[cfg]` in the core gates on it (`FEATURES.md:60`); it only satisfies a `getrandom` alias | 💀 no-op, harmless |

A `compile_error!` that names the fix ("`jemalloc` and `custom-allocator` are mutually
exclusive on unix") would turn the ⚠️ rows into one-line answers, but it needs
`src/bin/vanta-cli.rs` plus a feature interaction, which is outside the scope of the
documentation-and-coverage change that produced this page. It is carried as debt, not
fixed here.

## 4. Two caveats that make naive "covered" claims false

1. **`--features` with several `-p` unifies.** `cargo nextest … --features "cli,arrow,tls,opentelemetry" -p vantadb -p vantadb_py -p vanta-memory …`
   builds `vantadb` **once** with the union of what every selected package asks for
   (including `rayon`, which `vantadb-python/Cargo.toml:25` enables). The feature string in
   the workflow is therefore a lower bound, not an exact description of the build.
2. **`clippy --all-features` compiles exactly one combination: the union.** Because Cargo
   features are additive, it cannot catch a `#[cfg(not(feature = …))]` branch that rotted.
   Only a gate that builds a *smaller* set can (§2 row 1, 4, 6).

## 5. Cost ledger — why this matrix adds one job and not eight

Measured on the same run (`45949538624`): `Release Combo` 1.6 min and `Experimental
Crates Check` 1.6 min (compiles 4 crates), `Miri` 3.9 min, embedding legs 3.3/3.7 min. A
single `cargo check` job on a warm sccache costs **~1.5–3 min**, cold closer to 8.

| Candidate | Gap it would close | Cost | Decision |
| --- | --- | --- | --- |
| `server-full` (§2 row 6) | The documented operator build had **no** gate | ~2 min, one step, one job | **added** — `Feature Combo (server-full)` |
| `embed-local` in the release combo (§2 row 5) | Release lane parity with `release-binaries.yml` | ~1–3 min **on an existing job** | **declared, not added**: it changes the release profile under `-D warnings`, where `FIND-231` proved bugs hide. Unverifiable without a release build here; the one-string fix is `--features "server,embed-local,jemalloc"` on `ci-rust.yml:972,975` |
| `minimal-cli` (§2 row 9) | Nothing promised | ~2 min/run forever | **not added** |
| One job per remaining opt-in flag (11 flags) | Only `clippy --all-features` compile coverage | ~11 × 2 min/run | **not added**: additive-feature coverage already comes free with the union build |

The new job is **not** in the `develop` ruleset, so a red there costs a look, not a
blocked merge — the deliberate blast radius while it earns a track record.

## 6. Adding a combination

1. Check it against a real promise (README, `docs/user/`, a release workflow). If nothing
   in the repository asks for it, do not add the gate — add a row here saying why not.
2. Prefer `cargo check -p <crate> --locked --features "…"`: scoped, ~2 min, no test build.
3. Gate it with `needs: [fmt, clippy]` so doc-only PRs skip it through the existing chain
   (`rust-scope` → `clippy`), and do not add `continue-on-error` (see the taxonomy in
   [CI_POLICY.md](./dev/operations/CI_POLICY.md)).
4. Add the row to §2 in the same change, with the gate's name and its measured minutes.
   A matrix without the cost column turns into a wish list.
