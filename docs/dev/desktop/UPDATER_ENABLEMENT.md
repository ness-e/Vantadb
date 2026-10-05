---
title: "VantaDB Desktop — Auto-update enablement (deferred)"
kind: howto
status: active
description: "Defer trigger and ordered checklist to enable tauri-plugin-updater for VantaDB Desktop: signing keys, plugin wiring, release-mode CI, E2E validation, trust model"
tags: [vantadb, desktop, updater]
---

# VantaDB Desktop — Auto-update enablement (deferred)

> **Status: DEFERRED (2026-10-04).** `tauri-plugin-updater` is **not** installed or configured. This document is the enablement contract registered by [DESKTOP-43](../tasks/DESKTOP-43.md): why it is deferred, the trigger that re-opens it, and the ordered checklist to enable it. Decision record: [wontfix.md](../avance/decisiones/wontfix.md) § DEFER (`DESKTOP-43`).

## Why deferred

| Blocker | Evidence |
|---|---|
| Public-distribution decision not taken | Backlog `DESKTOP-43` ("Desbloquear tras decisión de distribución pública"; fila removida al registrar el defer — nota vigente en `docs/dev/Backlog.md` P46) · master plan 0.9.0 Task 33 ("decisión aún abierta") · `desktop/README.md` "No public installer yet" |
| Updater signing is mandatory — and installer signing is already deferred | [Tauri docs](https://v2.tauri.app/plugin/updater/): "Tauri's updater needs a signature to verify that the update is from a trusted source. **This cannot be disabled.**" · signing deferred by HITL decision (`wontfix.md` H-08 / `DEVOPS-10` — "when the public release requires it") · CI has no signing secrets (`gh secret list` → only `RELEASE_PLZ_TOKEN`, 2026-10-04) |

**Trigger to re-open:** the owner takes the **public-distribution decision** for the desktop app (release channel + signing). Then execute this checklist top to bottom and supersede the `wontfix.md` defer entry.

## 1. Signing keys (owner — release-critical)

The update channel is only as trustworthy as this keypair: the public key is embedded in the app as the trust anchor, and anyone holding the private key can ship a valid update to installed users. Losing the private key permanently blocks updating installed users (official docs).

- [ ] Generate the keypair: `cd desktop && npm run tauri signer generate -- -w ~/.tauri/vantadb-desktop.key` (set a password)
- [ ] Store private key + password in the team secrets manager — never in the repo
- [ ] Add repo secrets: `TAURI_SIGNING_PRIVATE_KEY` (content or path) + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — env only; `.env` files do not work (docs)
- [ ] Put the **public key content** into `plugins.updater.pubkey` (content, not a file path)

## 2. Wire the plugin (code)

- [ ] `desktop/src-tauri/Cargo.toml`: add `tauri-plugin-updater = "2"`
- [ ] `desktop/src-tauri/src/lib.rs`: register `tauri_plugin_updater::Builder::new().build()` in the Tauri builder chain
- [ ] `desktop/src-tauri/capabilities/default.json`: add `"updater:default"` (check / download / install / download-and-install)
- [ ] `desktop/src-tauri/tauri.conf.json`:

  ```json
  {
    "bundle": { "createUpdaterArtifacts": true },
    "plugins": {
      "updater": {
        "pubkey": "<content of the public key>",
        "endpoints": ["https://github.com/ness-e/Vantadb/releases/latest/download/latest.json"],
        "windows": { "installMode": "passive" }
      }
    }
  }
  ```

- [ ] Frontend: `@tauri-apps/plugin-updater` + `@tauri-apps/plugin-process`; `check()` → show version/notes/size and confirm → `downloadAndInstall()` → `relaunch()` (never download >100 MB without explicit confirmation — research §2.2, `RESEARCH.md:81`)

## 3. Release channel (CI)

- [ ] Release-mode desktop workflow (tag-driven — today's `desktop.yml` is build-only: no `tagName`/`releaseId`): pass `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to `tauri-action` and set `uploadUpdaterJson: true` (pinned tauri-action input; it defaults to `true`) → the release gets the `.sig` files + `latest.json`. Confirm which Windows bundle `latest.json` points to when both NSIS and MSI are built (`updaterJsonPreferNsis`, default `false`)
- [ ] First release must contain, per platform: installer + `.sig` + `latest.json` with **all** platform entries filled (Tauri validates the whole file before checking the version)
- [ ] macOS: coordinate with the signing decision — distributed macOS apps need Developer ID signing + notarization ([Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/)), on top of the updater's own update signature

## 4. Validation

- [ ] **Simulated E2E (local, throwaway keypair — no production keys needed):** build vN with a test pubkey; build vN+1 signed with the test private key; serve `latest.json` + artifact locally; confirm detect → signature verify → install → relaunch
- [ ] **Negative:** tampered manifest / bad signature → rejected (no install)
- [ ] **Real E2E (clean VM):** install vN → publish vN+1 release → app updates; reuse the evidence style of [INSTALLER_SMOKE_CHECKLIST.md](./INSTALLER_SMOKE_CHECKLIST.md)
- [ ] Keep the version triple in sync per release (`desktop/package.json` + `tauri.conf.json` + `src-tauri/Cargo.toml`; isolated from release-plz — see DESKTOP-QW8)

## 5. Trust model & docs (the enabling PR)

- [ ] Short ADR (AGENTS.md Regla 5): pubkey = embedded trust anchor; private-key custody; rollback story (the updater only moves forward unless `version_comparator` is customized)
- [ ] Update [ARCHITECTURE.md](./ARCHITECTURE.md) § Distribution & updates (remove "deferred") and `desktop/README.md` installer status
- [ ] Supersede the `wontfix.md` DEFER entry for `DESKTOP-43`

## References

- [Tauri v2 — Updater](https://v2.tauri.app/plugin/updater/) — setup, signing, static JSON, permissions
- [tauri-action](https://github.com/tauri-apps/tauri-action) — `uploadUpdaterJson` generates the static JSON
- `docs/dev/research/installer-personalizado/RESEARCH.md` §F3 — "updater Tauri firmado" (internal research)
