---
title: "Build troubleshooting (Windows dev)"
kind: research
status: active
description: "Build-lock (os error 5 / LNK1104), disk exhaustion (LNK1140), and git hooks that never fire (empty core.hooksPath, nodeenv ValueError)"
tags: [troubleshooting, windows]
---

# Build troubleshooting (Windows dev)

> **What this is:** the dev-side companion to [VantaDB — Troubleshooting Común
> (Windows)](../../user/troubleshooting.md). That page covers errors a *user* of
> the crates hits (linker, page file, SDK, WASM). This page covers errors only
> someone building or gating *this repo* hits.
> **How to add an entry:** append to the section it belongs to. Every entry needs
> Symptom, Root cause, Fix, and the exact command — an entry without the command
> is a rumour.

| Section | Symptom it answers |
|---|---|
| [Build lock — running MCP holds `target/debug/*.exe`](#build-lock--a-running-mcp-server-holds-targetdebugexe) | `Acceso denegado (os error 5)` / `LNK1104` / `failed to remove file ...\target\debug\vantadb-server.exe` |
| [Disk exhaustion — `target/` fills C: and the link fails](#disk-exhaustion--target-fills-c-and-the-link-fails-lnk1140) | `LNK1140` / `LNK1104` out of nowhere, drive at 0 bytes free, `target/` at tens of GB |
| [git hooks](#git-hooks--commits-pass-with-no-gate-or-nodeenv-valueerror-traceback) | `git commit` passes in 1 s with no gate output; or every commit dies on `nodeenv ... ValueError: invalid literal for int() with base 10: '0-alpha'` |

---

## Build lock — a running MCP server holds `target/debug/*.exe`

**Symptom.** Every build that relinks an executable a live process has mapped
fails, deterministically, and always at the same file:

```
error: failed to remove file `C:\...\VantaDB\target\debug\vantadb-server.exe`
Caused by:
  Acceso denegado (os error 5)
```

`Acceso denegado` is the Spanish-locale rendering of `Access is denied`
(`verrouillage` / `Permission denied` on other locales — the `(os error 5)` is
the stable part). When MSVC's linker is the one that notices, it reports the
same condition on its output file as `error LNK1104: cannot open file
'C:\...\target\debug\vantadb-server.exe'`.

**Root cause.** Windows will not delete or replace the image file of a running
process. `vanta-mcp-local.ps1` starts `vanta-cli server --mcp`, and that server
spawns the **sibling** `vantadb-server.exe` sitting next to it; the launcher's
`WIN-LOCK` block copies both binaries to `%TEMP%\vanta-mcp-bin\` first, precisely
so the MCP runs off a copy and `target/debug` stays writable (`a431ae0a`). Two
paths still reach the lock:

1. **The MCP was started in place** — `target\debug\vantadb-server.exe --mcp`,
   `target\debug\vanta-cli.exe server --mcp`, or a client whose config points at
   `target/debug` (an `opencode.jsonc` `mcp` entry, typically launched by
   `opencode.exe serve --service`, which respawns the child on its next call).
2. **The temp copy did not happen.** `Copy-Item -ErrorAction SilentlyContinue`
   fails silently; the child then resolves `vantadb-server.exe` from the candidates
   next to itself and can still end up mapping the one in `target/debug`.

The lock is held on the **image file**, so the holder is exactly the process
whose `Path` equals that file — not "every process called `opencode`". Killing
your own client session is not the fix and loses your work.

**Fix.** Identify the holder by path, stop that process only, run the gate, and
let the client respawn the MCP afterwards.

```powershell
# 1. WHO holds it? A holder whose Path is the target/ file is the one.
$locked = (Resolve-Path .\target\debug\vantadb-server.exe -ErrorAction SilentlyContinue).Path
if (-not $locked) { throw 'no binary at target\debug\vantadb-server.exe — rebuild first, the lock is not your problem' }
Get-Process -Name vantadb-server, vanta-cli -ErrorAction SilentlyContinue |
    Select-Object Id, ProcessName, Path | Format-Table -AutoSize

# Command line, when there are several candidates (shows --mcp and which exe it runs from)
Get-CimInstance Win32_Process -Filter "Name='vantadb-server.exe' OR Name='vanta-cli.exe'" |
    Select-Object ProcessId, ExecutablePath, CommandLine | Format-List
```

Read the two outputs before killing anything:

| Holder's `Path` | Meaning | Action |
|---|---|---|
| `...\target\debug\vantadb-server.exe` | Started in place from the repo | **Stop it** — this is the lock |
| `%TEMP%\vanta-mcp-bin\vantadb-server.exe` | Launcher's protected copy | **Leave it** — it does not lock `target/debug` |
| `...\vanta-cli.exe` (no `target\debug\` in the path) | Global install on `PATH` | **Leave it** |

```powershell
# 2. Stop only the real holder — matched by image path, never by bare process name.
$locked = (Resolve-Path .\target\debug\vantadb-server.exe -ErrorAction SilentlyContinue).Path
if (-not $locked) { throw 'nothing to unlock' }
Get-Process -Name vantadb-server, vanta-cli -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -eq $locked } |
    Stop-Process -Force -PassThru | Select-Object Id, ProcessName

# 3. Gate. The client respawns the MCP on its next tool call, so re-run the build
#    in one go if you still have a session running.
cargo build -p vantadb-server
```

Same procedure with `vanta-cli` / `.\target\debug\vanta-cli.exe` when that is the
locked binary (it is replaced normally *because* the launcher runs it from
`%TEMP%` — the lock shows up when it was started in place).

**Gate without stopping the MCP (the `--lib` workaround).** The HTTP end-to-end
tests live in the `vantadb` **lib test target** behind `--features server`
(moved there by [WIRE-04](../tasks/closed/WIRE-04.md) for this exact reason), so
they do not need the `vantadb-server` binary at all:

```powershell
cargo nextest run --profile audit -p vantadb --lib --features server --build-jobs 2
```

That is a *scoped* substitute, not an equivalent gate: it covers `src/server/**`
(HTTP E2E included) but **not** `vantadb-server/tests/**`, which need the binary,
and not the `vanta-cli` bin target.

**Where the gate actually falls.**

| Command | Relinks the locked binary? |
|---|---|
| `cargo nextest run --profile audit --workspace` (close/certify gate) | **Yes** — always |
| `cargo build -p vantadb-server` / `cargo nextest -p vantadb-server` | **Yes** — always |
| `dev-tools/verify.ps1` step `nextest` (line 87, `-p vantadb`) | No — never builds `vantadb-server` |
| `dev-tools/verify.ps1` step `cli-probes` (line 106, `cargo run --bin vanta-cli`) | Only if `vanta-cli.exe` is held |
| CI (`.github/workflows`) | Never — Linux runners do not lock images |

**Prevention.** Keep the launcher on its temp copy (do not point an `mcp` entry
at `target/debug`), and stop in-place MCP servers before a `--workspace` gate.
The advisory probe proposed for `dev-tools/verify.ps1` is tracked in
`FIND-192`; the gate file is not owned by this document (the disk side of
`FIND-192` did ship — [Disk
exhaustion](#disk-exhaustion--target-fills-c-and-the-link-fails-lnk1140)).

---

## Disk exhaustion — `target/` fills C: and the link fails (LNK1140)

**Symptom.** A build that was fine an hour ago dies at the link step:

```
error: linking with `link.exe` failed: exit code: 1120
  = note: LINK : fatal error LNK1140: cannot open file 'C:\...\target\debug\deps\vanta_cli.exe'
```

and the drive is at (or near) zero:

```powershell
Get-PSDrive C | Select-Object Name, @{n = 'FreeGB'; e = { [math]::Round($_.Free / 1GB, 2) } }
# Name FreeGB
# C      0.00
```

`LNK1104` (`cannot open file`) shows up the same way. It is neither a lock
([the section above](#build-lock--a-running-mcp-server-holds-targetdebugexe))
nor a linker bug: there is no space to write the image, so the linker reports
the file it could not open.

**Root cause.** Two things inside `target/` grow with every build and nothing in
the normal build cycle prunes them:

| Artefact | Why it grows | Measured 2026-11-24 |
|---|---|---|
| `target/<profile>/incremental` | one session directory per crate per profile; superseded sessions stay behind while builds keep succeeding | 12.07 GB (`debug`) |
| `*.pdb` anywhere under `target/` | every linked **test, bench and example** target emits debug symbols (`Cargo.toml:743` `[profile.bench] debug = 1`, `:856-857` `[profile.test] debug = 1` — deliberate: flamegraphs and usable backtraces) | 445 files / 9.98 GB |

At that moment `target/debug` was 36.12 GB and C: had **5.10 GB** free — 22.05 GB
of it was regenerable by the next build. A co-batch wave that builds several
crates grows this within hours; when the drive hits 0 the next link fails and
the wave stops. On 2026-11-23 that is exactly what happened: 31 GB had to be
freed by hand in the middle of a pipeline.

**Fix — the pre-wave gate. Declared threshold: 15 GB free on the repo's drive.**
Run it before the first build of a wave, never during one:

```powershell
# 1. Report only: free space, per-profile size, and the "GB liberables" line.
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -DryRun

# 2. What the gate would do — threshold applied, nothing deleted (safe during builds).
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -DryRun

# 3. The gate: if free < 15 GB, delete {*}/incremental + *.pdb; otherwise do nothing.
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe

# Smaller disk, or a deliberately bigger co-batch wave:
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -MinFreeGB 25
```

| Exit | Meaning |
|---|---|
| `0` | report only, dry-run, or the cleanup left the drive **above** the threshold |
| `3` | it ran (or had nothing to prune) and the drive is **still below** the threshold — **not** green; free space by hand with `-Clean [-Full]` first |
| `2` | `cargo`/`rustc` were running → skipped on purpose, retry between builds |
| `1` | refused by the path safety check, incompatible flags (`-AutoSafe` with `-Clean`/`-Full`), or the repo's drive could not be determined |

What `-AutoSafe` will never do: delete `target/` itself, delete anything outside
`target/`, delete the `deps/`, `debug/` or `release/` **directories**, follow a
junction/reparse point, or kill a process. It *does* delete the `*.pdb` files
**inside** those directories (381 of the 445 files measured on 2026-11-24 lived
in `target/debug/deps`, ~11 GB in total) plus every `{profile}/incremental`. The
only mode that drops `debug/deps` is the interactive `-Clean -Full`, and that
costs a cold rebuild — a human decision, not a gate's.

**Incremental after a cleanup — what actually breaks, and what does not.**
Deleting `incremental/` *while a build is running* leaves `.fingerprint` entries
pointing at session directories that no longer exist; the next `rustc` that
reuses one dies with an incremental-cache error (`could not read incremental
cache`, or an ICE), and the failure reads like a corrupted repository. That is
why the gate refuses to run when `cargo`/`rustc` are alive (exit `2`) instead of
"cleaning anyway". Deleting it **between** builds costs recompilation time only —
no correctness loss, no data loss. If a cache does end up inconsistent:

```powershell
cargo clean -p <crate>          # drops fingerprint + artifacts for one crate only
# or take incremental out of the picture for a single gate:
$env:CARGO_INCREMENTAL = 0
cargo nextest run --profile audit -p vantadb --build-jobs 2
```

**Self-check (the safety property is asserted, not promised).**
`-SelfCheck` verifies the prefix ends in a separator, that `target2/`,
`target/../src/`, `target/..` and `target/` itself are **rejected**, and that
every path in the real cleanup list starts with `target\` and is either
`incremental/` or `*.pdb`:

```powershell
pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -SelfCheck   # SELFCHECK: PASS (16/16 asserts)
```

---

## git hooks — commits pass with no gate, or `nodeenv` `ValueError` traceback

**Symptom.** Two failures that look unrelated but hit the same stage — the
commit gate of the gate pyramid (Regla 1 of `.opencode/AGENTS.md`):

1. `git commit` finishes in about a second and prints **no gate output at all**:
   `verify_staged`, `cargo fmt`/`clippy`, `ruff`, `prettier` and
   `markdownlint` never ran. `git config --get core.hooksPath` prints nothing
   and exits `1`, and `.git/hooks/` contains only `*.sample` files.
2. Every commit aborts with a Python traceback ending in
   `ValueError: invalid literal for int() with base 10: '0-alpha'`, preceded by
   `CalledProcessError: command: ('...python.exe', '-mnodeenv', '--prebuilt',
   '--clean-src', ...)` and a line `* Install prebuilt node (27.0.0-alpha...)`.

**Root cause.** Two independent defects; fixing one leaves the other.

1. **Wiring (per clone, never versioned).** The hooks live in `.githooks/`, not
   `.git/hooks/`. Git only looks there when `core.hooksPath` says so, and that
   key is **local clone config**. A fresh clone — or a clone made before the key
   was ever set — runs with **zero hooks** and says nothing, because a commit
   without hooks is a perfectly valid commit. This is the silent false-green:
   it does not break, it just stops measuring.
2. **`nodeenv` vs a prerelease Node (upstream).** On Windows, `pre-commit`
   provisions its `language: node` hooks through `nodeenv` with no `-n` when
   `language_version` is the default (`pre_commit/languages/node.py`,
   `install_environment`: `version != C.DEFAULT` is what adds `-n <version>`).
   Default means *latest*, and nodeenv resolves that to the **first entry of**
   `nodejs.org/dist/index.json`. Since 2026-10-29 that entry is a prerelease
   (`v27.0.0-alpha.0.1.1`); `nodeenv.parse_version()` splits on dots and calls
   `int()` on the third segment (`0-alpha`) → `ValueError` before anything is
   downloaded. Tracked upstream: [ekalinin/nodeenv#453][ne453], fixed by
   [PR #454][ne454] (merged 2026-10-29, shipped in **1.11.1**: *"1.11.1 is
   1.11.0 plus this fix only"*; 1.11.0 and older still break). Nothing in
   VantaDB pinned a Node version, so the gate broke the
   day the upstream index changed — not the day anything here changed.

**Fix.** Both, in this order.

1. Wire the clone (local config — **do not** commit it, and never run
   `pre-commit install`: it writes to `.git/hooks/`, which `core.hooksPath`
   makes dead):

   ```powershell
   git config --local core.hooksPath .githooks
   git config --get core.hooksPath    # must print: .githooks
   ```

2. Pin pre-commit's Node to a **released LTS**, repo-level (versioned, so it
   fixes every clone and survives the next index churn) in
   `.githooks/pre-commit-config.yaml`:

   ```yaml
   default_language_version:
     node: lts
   ```

   Then rebuild the cached environments — `lts` is a *new* env key, so the old
   broken one is not reused:

   ```powershell
   pre-commit install-hooks --config .githooks/pre-commit-config.yaml
   ```

   `lts` resolves to a stable release (26.13.1 on 2026-11-24) instead of the
   bleeding edge, so it cannot be poisoned by an alpha at the top of the index.

**Verify the wiring actually fires.** Do it in a throwaway worktree: the hook
runs `git diff --cached`, so a test commit on your working branch would drag
along whatever else is staged — and on a shared checkout, someone else's files.
The hooks are committed with the executable bit set (`100755`); on POSIX, git
**silently ignores** a hook without it (`advice.ignoredHook`), so if
`git config core.hooksPath` is right and the hook still never prints anything,
check `git ls-files -s .githooks/`.

```powershell
git worktree add ..\hooktest -b test-hook develop
# in ..\hooktest: create a file, git add <file>, git commit -m "test"
#   -> "pre-commit: running .githooks/pre-commit-config.yaml gates..." + gate lines + "gates passed."
git worktree remove --force ..\hooktest
git branch -D test-hook
```

**Degraded mode is loud, never silent.** Each fallback names what it did **not**
validate: no `pre-commit` binary → the shim runs only `cargo fmt --check` and
says so; no PowerShell → the staged-area gate is skipped and the hook states the
yaml gates still ran; `actionlint` missing → skipped by name. A hook that cannot
run exits non-zero with a pointer to this section. Never `--no-verify` past a
red gate, and never install the alpha to "make it work".

### The hook never stashes a shared tree (`FIND-726`)

**Symptom.** Tracked files you are editing change under you while *someone else*
commits: an edit reverts, a batch of files vanishes for a minute and comes back,
or `git diff` is empty on files you know you changed. `git stash list` is empty -
the stash was temporary and internal.

**Root cause.** `pre-commit run` reverts every tracked file to its index content,
runs the hooks, then replays the unstaged diff:

```python
# pre_commit/commands/run.py (v4.6.2 and main)
stash = not args.all_files and not args.files
```

```python
# pre_commit/staged_files_only.py::_unstaged_changes_cleared
cmd_output_b(*_CHECKOUT_CMD, ...)   # git checkout -- .
yield                               # hooks run on index content only
_git_apply(patch_filename)          # replay of the diff captured before
```

This is deliberate upstream - see
<https://pre-commit.com/#pre-commit>: *"pre-commit only runs on the staged
contents of files by temporarily stashing the unstaged changes while running
hooks"* - and it is correct for one developer with one checkout. It is wrong for
one checkout shared by several agents: any write that lands inside the window is
dropped by the replay (the log still says `Restored changes`), and a run killed
mid-way leaves the tree reverted with no stash to recover from. It cost other
agents' work three times before this was fixed (`FIND-720`, `FIND-721`,
`FIND-723`).

**Fix.** `.githooks/pre-commit` passes the staged list explicitly. That is the
only documented lever over `stash` (`--no-stash` exists only on the `hook-impl`
CLI, not on `pre-commit run`):

```sh
git diff --staged --name-only -z --diff-filter=ACMRTUXB \
  | xargs -0 pre-commit run --config .githooks/pre-commit-config.yaml --files
```

`--files` receives exactly the file set `pre-commit` would have derived itself,
so the verdict is unchanged. The hook prints the tracked files it is leaving
alone and never writes into the working tree.

**Verify the invariant** - in a throwaway clone, never in the shared checkout:

```powershell
git hash-object <tracked-file-with-unstaged-edits>   # before
git commit -m "probe"
git hash-object <tracked-file-with-unstaged-edits>   # must be identical
```

**Trade-off, measured.** Hooks now see the **working tree**, not the index. Hooks
that pass filenames (`ruff`, `prettier`, `markdownlint`, junk guard) still
receive only the staged names and are unaffected. Whole-tree hooks (`cargo fmt
--check`, `cargo clippy`, the per-member clippy in `verify_staged`) can fail on a
file another agent is mid-edit. That failure is loud and true - bypass it only
for the file you know is foreign:

```powershell
$env:SKIP = "cargo-fmt,cargo-clippy"; git commit ...
```

Aborting whenever foreign WIP exists was rejected: on a shared checkout WIP is
the normal state, so an abort would block every commit and push everyone to
`--no-verify`, which is no gate at all.

**Recovering a stash that never came back.** An interrupted run leaves the patch
at `~/.cache/pre-commit/patch*`. Inspect it with `git apply --stat <patch>`, apply
with `git apply <patch>`. Do not re-run the hook first - it would revert the tree
again.


[ne453]: https://github.com/ekalinin/nodeenv/issues/453
[ne454]: https://github.com/ekalinin/nodeenv/pull/454

---

## See also

- [VantaDB — Troubleshooting Común (Windows)](../../user/troubleshooting.md) — user-facing Windows errors (linker stack buffer overrun, page file, SDKs).
- `vanta-mcp-local.ps1` (repo root) — the launcher, `WIN-LOCK` block: temp copy of `vanta-cli.exe` + `vantadb-server.exe`.
- `dev-tools/target-cleanup.ps1` — the disk gate documented above (`-DryRun`, `-AutoSafe`, `-SelfCheck`).
- [WIRE-04](../tasks/closed/WIRE-04.md) — why the HTTP E2E tests live in the lib test target.
