---
title: "Build troubleshooting (Windows dev)"
kind: research
status: active
description: "Acceso denegado (os error 5) / LNK1104 when relinking target/debug/*.exe while a VantaDB MCP server is alive"
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
`FIND-192`; the gate file is not owned by this document.

---

## See also

- [VantaDB — Troubleshooting Común (Windows)](../../user/troubleshooting.md) — user-facing Windows errors (linker stack buffer overrun, page file, SDKs).
- `vanta-mcp-local.ps1` (repo root) — the launcher, `WIN-LOCK` block: temp copy of `vanta-cli.exe` + `vantadb-server.exe`.
- [WIRE-04](../tasks/closed/WIRE-04.md) — why the HTTP E2E tests live in the lib test target.
