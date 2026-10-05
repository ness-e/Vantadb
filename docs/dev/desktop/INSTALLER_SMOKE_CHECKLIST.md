---
title: VantaDB Desktop — Installer Smoke Checklist (clean Windows VM)
kind: howto
status: active
description: "Executable smoke test for the VantaDB desktop NSIS/MSI installers on a clean Windows VM: startup, sidecar binaries, vanta:// deep link, WebView2 bootstrapper — with evidence capture per step"
tags: [vantadb, desktop, installer, smoke-test]
---

# VantaDB Desktop — Installer Smoke Checklist (clean Windows VM)

Executable smoke test for the Windows installers produced by
`cd desktop && npm run tauri build` (Step 3 of DESKTOP-24, task DESKTOP-41).

**Why this checklist exists:** the install-time *configuration* was verified
read-only on 2026-10-04 (see [Static pre-verification](#static-pre-verification-read-only--not-a-substitute)),
but startup, sidecar execution, deep-link launch and WebView2 bootstrapping can
only be proven by a **real install on a clean Windows VM**. No such VM was
available in the build environment, so the run is tracked as `FIND-272` in
`docs/dev/Backlog.md`. **Do not mark the smoke as passed from this document —
fill in the evidence tables below.**

## Artifacts under test

| Installer | Path (repo) | Size | SHA256 |
|---|---|---|---|
| NSIS (`-setup.exe`) | `desktop/src-tauri/target/release/bundle/nsis/vantadb-desktop_0.1.0_x64-setup.exe` | 12.18 MB | `6B30B04439E6A493C3A70D9FD1BA7E8F544DAA5742FC9EDA95FA0AEA61511E4F` |
| MSI | `desktop/src-tauri/target/release/bundle/msi/vantadb-desktop_0.1.0_x64_en-US.msi` | 16.75 MB | `0069AB1AEEBA33C44D1364DA8001AB246D72C2D49E5A2271C0F66C988716F668` |

Rebuild (measured 2026-10-04: **6m46s** cold on the dev machine):
`cd desktop && npm run tauri build`

Alternative without a local build: GitHub Actions **Desktop CI** →
artifact `vantadb-desktop-windows-installers` (build-only; never smoke-tested).

## VM prerequisites

- [ ] Windows 10 22H2 or Windows 11, x64, **clean** (no prior VantaDB install,
      no dev tools needed).
- [ ] **Snapshot/checkpoint taken BEFORE the first install** — you will reset
      it between Test A and Test B, and after the uninstall checks.
- [ ] Network available: `embedBootstrapper` downloads the WebView2 runtime
      during install if it is missing
      ([Tauri docs](https://v2.tauri.app/distribute/windows-installer/)).
- [ ] Note the VM's WebView2 state up front (it changes step A7/B6):
      - Windows 11 / Windows 10 (2018+) ship the runtime → bootstrapper path
        is *skipped* (mark it N/A and note it).
      - A VM image **without** WebView2 exercises the bootstrapper path (the
        interesting case).
- [ ] Installers are **unsigned** (no Authenticode) → SmartScreen /
      "unknown publisher" prompts are **expected**; record them, do not treat
      them as failures (signing is a distribution decision, out of scope —
      DESKTOP-43/DEVOPS-10).

## Test A — NSIS (`vantadb-desktop_0.1.0_x64-setup.exe`)

Expected from static inspection: per-user install (`INSTALLMODE=currentUser`),
**no UAC prompt**, install dir `%LOCALAPPDATA%\vantadb-desktop`; registers
`vanta://` under `HKCU\Software\Classes\vanta`.

| # | Step | Command / action | Expected | Evidence | Pass |
|---|------|------------------|----------|----------|------|
| A1 | Copy installer into VM, verify hash | `Get-FileHash .\vantadb-desktop_0.1.0_x64-setup.exe -Algorithm SHA256` | matches table above | hash output | ☐ |
| A2 | Clean-state probe (before install) | `reg query HKCU\Software\Classes\vanta` | `not found` | console output | ☐ |
| A3 | Run installer (GUI) | double-click `-setup.exe` | wizard completes; **no UAC**; SmartScreen prompt expected (unsigned) — note it | screenshot of SmartScreen + final screen | ☐ |
| A4 | Verify install tree | `dir "%LOCALAPPDATA%\vantadb-desktop"` | `vantadb-desktop.exe`, `vanta-cli.exe`, `vantadb-server.exe`, `uninstall.exe` present | `dir` output | ☐ |
| A5 | Registry: deep link registered by installer | `reg query HKCU\Software\Classes\vanta /s` | `URL Protocol` (empty) + `shell\open\command` → `"…\vantadb-desktop.exe" "%1"` | `reg query` output | ☐ |
| A6 | Launch app | start menu / `"%LOCALAPPDATA%\vantadb-desktop\vantadb-desktop.exe"` | window "VantaDB Studio" opens, UI renders (sidebar + workspace, no blank WebView) | screenshot | ☐ |
| A7 | Sidecar binaries run from install dir | `& "$env:LOCALAPPDATA\vantadb-desktop\vanta-cli.exe" --version` and `& "$env:LOCALAPPDATA\vantadb-desktop\vantadb-server.exe" --help` | `vanta-cli 0.5.0` · `VantaDB server … USAGE: vantadb-server [OPTIONS]` | console output | ☐ |
| A8 | WebView2 install-time (only if runtime was absent) | `reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"` (also try `HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\…` and `HKCU\…`) | `pv` value present after install (bootstrapper ran) — else mark **N/A** and say the runtime was preinstalled | `reg query` output + note | ☐ |
| A9 | Deep link — app **closed** | close app, then `Start-Process "vanta://?query=smoke"` (or `start vanta://?query=smoke` in cmd) | app launches; frontend navigates to **MEMORIAS** and runs the `smoke` search | screenshot of app + console | ☐ |
| A10 | Deep link — app **running** | with app open: `Start-Process "vanta://?query=second"` | **no second window**; same window receives the link (MEMORIAS + `second` search) — single-instance forwarding | screenshot (only one window) | ☐ |
| A11 | Uninstall | Settings → Apps → uninstall `vantadb-desktop` (or `"%LOCALAPPDATA%\vantadb-desktop\uninstall.exe"`) | app removed; `%LOCALAPPDATA%\vantadb-desktop` gone; `reg query HKCU\Software\Classes\vanta` → `not found`; `HKCU\…\Uninstall\vantadb-desktop` gone | screenshots + `reg query` output | ☐ |
| A12 | Reset VM to snapshot before Test B | Hyper-V: *Revert to checkpoint* | clean state | — | ☐ |

## Test B — MSI (`vantadb-desktop_0.1.0_x64_en-US.msi`)

Expected from static inspection: per-machine install (`ALLUSERS=1`) →
`C:\Program Files\vantadb-desktop`, **UAC prompt**; `vanta://` under
`HKCR\Software\Classes\vanta` (i.e. `HKLM\Software\Classes`); embedded
WebView2 bootstrapper runs via custom action `InvokeBootstrapper /silent /install`
only when the runtime is missing.

| # | Step | Command / action | Expected | Evidence | Pass |
|---|------|------------------|----------|----------|------|
| B1 | Copy installer, verify hash | `Get-FileHash .\vantadb-desktop_0.1.0_x64_en-US.msi -Algorithm SHA256` | matches table above | hash output | ☐ |
| B2 | Install (GUI or logged silent) | double-click, or `msiexec /i .\vantadb-desktop_0.1.0_x64_en-US.msi /l*v "$env:TEMP\vanta-msi.log"` | completes; **UAC yes**; SmartScreen expected (unsigned) | screenshot + MSI log | ☐ |
| B3 | Verify install tree | `dir "C:\Program Files\vantadb-desktop"` | `vantadb-desktop.exe`, `vanta-cli.exe`, `vantadb-server.exe` present | `dir` output | ☐ |
| B4 | Registry: deep link registered by installer | `reg query HKCR\Software\Classes\vanta /s` (and `HKLM\Software\Classes\vanta`) | `URL Protocol` + `shell\open\command` → install-dir exe | `reg query` output | ☐ |
| B5 | Launch app + sidecars | launch from Start Menu; run the same two sidecar commands as A7 from `C:\Program Files\vantadb-desktop` | UI renders; `vanta-cli 0.5.0`; server help | screenshot + output | ☐ |
| B6 | WebView2 install-time | if runtime was absent: search `vanta-msi.log` for `InvokeBootstrapper`; re-query the `EdgeUpdate\Clients\{F3017226-…}` key | action executed (return check) + `pv` present — else mark **N/A** | log excerpt + `reg query` | ☐ |
| B7 | Deep link (closed + running) | repeat A9/A10 | same behavior (single instance, MEMORIAS + search) | screenshots | ☐ |
| B8 | Uninstall | `msiexec /x {2B59945A-0283-4D40-80AF-F3AF74F4B368}` or Apps & Features | removed; dir gone; `HKCR\Software\Classes\vanta` gone | screenshots + `reg query` | ☐ |
| B9 | Reset VM to snapshot | | clean state | — | ☐ |

## Evidence bundle

Collect in one folder (e.g. `desktop-smoke-<date>/`):

- screenshots named per step (`A3-installer.png`, `A9-deeplink-closed.png`, …)
- console outputs (`reg query`, `dir`, hashes, sidecar `--version/--help`)
- MSI log (`vanta-msi.log`) when B2 used `/l*v`

Then: attach the bundle path to `FIND-272` in `docs/dev/Backlog.md` and record
the outcome (pass/fail per row + any new `FIND-*` with repro) in
`docs/dev/avance/activo/desktop.md`.

## Static pre-verification (read-only — NOT a substitute)

Verified 2026-10-04 by read-only inspection of the generated artifacts
(WindowsInstaller COM over the MSI; the bundler-generated `installer.nsi`):

- `vanta://` registration present in **both** installers (NSIS `installer.nsi:650-653`;
  MSI `Registry` table → `Software\Classes\vanta` + `URL Protocol` + `shell\open\command`).
- Sidecars `vanta-cli.exe` + `vantadb-server.exe` packaged next to the app exe
  in **both** (NSIS `:642-643`; MSI `File` table).
- WebView2 `embedBootstrapper` embedded in **both** (MSI `Binary` +
  `CustomAction InvokeBootstrapper /silent /install`, condition
  `NOT(REMOVE OR INSTALLED_WEBVIEW2_VERSION)`; NSIS section at `:536+`).
- Uninstall cleanup of the protocol key in NSIS (`:766-776`).
- MSI identity: ProductCode `{2B59945A-0283-4D40-80AF-F3AF74F4B368}`,
  UpgradeCode `{7A138BB2-38A0-52A4-946E-4424D3F1F0BD}`, version `0.1.0`.

Sources: [Tauri v2 — Windows Installer](https://v2.tauri.app/distribute/windows-installer/) ·
[Tauri v2 — Deep Linking](https://v2.tauri.app/plugin/deep-linking/).

## Known caveats

- **Deep links only fire for installed apps** (Tauri docs). In dev builds the
  app self-registers at startup (`register_all`), which can mask an
  installer-side registration bug — that is why A5/B4 check the registry
  **before** the first launch.
- `embedBootstrapper` **requires network at install time**; a fully offline
  install needs `offlineInstaller` (~+127 MB), a separate decision.
- NSIS = per-user (`HKCU`), MSI = per-machine (`HKLM`/`HKCR`): different hives
  are correct, not a bug.
- The app re-registers `vanta://` in `HKCU` on every launch
  (`desktop/src-tauri/src/lib.rs:134-139`) — belt-and-suspenders, does not
  replace the installer check.
- `vanta://?query=x` is the easiest deep link to observe end-to-end: it
  navigates to MEMORIAS and runs the search (`desktop/src/components/layout/WorkspaceShell.tsx:533-559`).
