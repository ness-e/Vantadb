---
title: Workflows — Publish flow per registry
kind: runbook
status: active
description: "Publishing is tokenless (OIDC Trusted Publishing, no PATs). Post-FIND-140"
tags: [vantadb, ci, workflows, publish, release]
---

# Workflows — Publish flow per registry

Publishing is tokenless (OIDC Trusted Publishing, no PATs). Post-FIND-140
state. Rollback policy per plan: `git revert` per commit + re-run.

## Chain overview

```text
merge develop -> main
  -> release.yml (release-plz: bump, changelog, git tag, crates.io)
    -> tag v*.*.*      -> wheels-60 (PyPI) + npm-61 (npm wasm+TS) + sbom-64 (SBOM)
    -> tag node-v*.*.* -> npm-node (npm Node binding)
    -> tag adapters-v* -> adapters-62 (PyPI adapters)
    -> GitHub Release  -> binaries-63 (binaries)
```

A release is **not complete when published** — it is complete when the
[post-release verification](#post-release-verification-release-completion-gate)
below is green (assets + registries + smoke).

## develop→main: merge commit, no squash (2026-10-02)

Los PRs `develop → main` se mergean con **merge commit** (`gh pr merge --merge`), no `--squash`: release-plz lee los commits convencionales del historial para generar el changelog, y con squash solo ve el commit resumen (changelog "### Other" — pasó en 0.7.0 y 0.8.0). Con merge commit el changelog sale rico automáticamente (validado en 0.6.x). Los Release PR de release-plz siguen squash (1 commit).
## crates.io — `release.yml`

- Trigger: `push: branches: [main]` only.
- `release-plz-release` job: version bump, changelog, tag push, `cargo publish`
  via OIDC (`id-token: write`). Timeout 30 min, no cancel (`cancel-in-progress: false`).
- `release-plz-pr` job: opens the Release PR on subsequent pushes.

## PyPI wheels — `release-wheels.yml`

- Tags `v*.*.*` publish to prod PyPI (`environment: pypi`, OIDC + provenance
  attestation); `workflow_dispatch` with `publish_testpypi=true` publishes to
  TestPyPI (`environment: testpypi`).
- PRs to `main` touching `src/`, `vantadb-python/`, `Cargo.*` run build +
  smoke only (no publish).
- Tag namespaces: strict `v*.*.*` (three numeric parts).

## npm WASM + TS — `release-npm-61.yml`

- Tags `v*.*.*` publish both packages (`environment: npm`, OIDC):
  `publish-wasm` builds `vantadb-wasm` with wasm-pack, then `publish-ts`
  rewrites the exact wasm version into `vantadb-ts` and publishes `vantadb`.
- PRs touching `vantadb-wasm/**` or `vantadb-ts/**` run CI only.
- `workflow_dispatch` selects `wasm` / `ts` / `both` + `dry_run`.

## npm Node — `release-npm-node.yml`

- Tags `node-v*.*.*` publish the Node binding (`environment: npm`, OIDC).
- PRs touching `vantadb-node/**` run CI only.
- Own namespace so core `v*` releases never publish the Node package by accident.

## Orden del bump npm (FIND-230) — gate `check-npm-versions`

`vantadb-ts/package.json` debe estar **en la misma versión que `[workspace.package]`** para que el tag publique el tren completo. Si queda atrás, `release-npm-61.yml` pregunta por la versión vieja, npm responde "ya publicada" y el publish **se saltea en un run verde** — pasó en 0.8.0: TS quedó en 0.7.0, el tag `v0.8.0` publicó wasm y para `vantadb` solo imprimió `Version 0.7.0 already published — skipping` (run `37045932895`, conclusión success); el backfill fue manual (`e62e0f62` + dispatch).

Regla mecánica: el gate **`Check npm package versions`** (`gate-docs.yml`, script `scripts/docs/check-npm-versions.mjs` — mismo script local y CI) falla si `vantadb-ts/package.json` ≠ versión del workspace. Flujo correcto:

1. `release-plz` abre el Release PR (bumpea `Cargo.toml` → X.Y.Z).
2. **Antes de mergearlo**, agregar el bump npm a la **misma rama** del Release PR: `vantadb-ts/package.json` → X.Y.Z (y `vantadb-node/package.json` desde el primer release del train node).
3. El gate queda verde en ese PR → merge → tag `vX.Y.Z` → `release-npm-61.yml` publica `vantadb@X.Y.Z` (sin skip).

Cuidado con release-plz: `release-plz-pr` corre en **cada push a `main`** y no preserva commits humanos — si vuelve a correr mientras el bump está en su rama, cierra ese PR y abre uno nuevo **sin el bump**. Por eso: mergear el Release PR sin demora, verificar en el diff que el bump sigue presente justo antes de mergear, y si el PR fue cerrado/reabierto, **re-aplicar el bump** y esperar el gate verde. El fallo nunca es mudo: el gate se pone rojo y el publish emite `::warning::`.

No bumpear npm en `develop` antes del Release PR: `npm ≠ workspace` y el gate lo marca igual — las dos versiones viajan juntas.

Excepción `vantadb-node`: **nunca publicado** en npm (registry 404, 2026-10-03). Su versión puede quedarse en `0.7.0` (versión "never-published" documentada en el script); a partir de su primer release debe igualar la versión del workspace (y la entrada `TARGETS` del script — `scripts/docs/check-npm-versions.mjs` — pasa manualmente a `policy: 'workspace'`). `vantadb-wasm` no necesita gate: su manifest npm lo genera wasm-pack desde el crate, que hereda `version.workspace = true`.

El skip "already published" de `release-npm-61.yml` y `release-npm-node.yml` ahora emite `::warning::` visible en el run — ya no es un skip mudo.

## Adapters — `release-adapters.yml`

- Tags `adapters-v*.*.*` publish 9 adapters
  (langchain, llamaindex, mem0, crewai, dspy, haystack, letta, openai, ollama)
  to prod PyPI after the test matrix passes.
- `workflow_dispatch` with `publish_testpypi=true` goes to TestPyPI.
- Own namespace so adapter-only releases never trigger core wheels/npm.

## Binaries — `release-binaries.yml`

- Trigger: `release: types: [published]` + manual dispatch only (FIND-140:
  the old `push.tags: [v*]` built 5 targets just to discard).
- Builds `vanta-cli` + `vantadb-server` for 5 targets with the custom
  allocator (Windows mimalloc, Linux/macOS jemalloc), uploads
  `tar.gz`/`zip` + sha256 to the GitHub Release.
- Backfill manual: `workflow_dispatch` con input `release_tag` (ej. `v0.8.0`);
  sin input = solo build sin upload.
- v0.8.0+ ships binaries; releases ≤ v0.7.0 predate the `RELEASE_PLZ_TOKEN`
  cascade fix and have **no** binary assets (known gap — see Post-release
  verification below).

## SBOM — `release-sbom.yml`

- Tags `v*` (broad: matches any `v` tag, including `v*.*.*`) generate
  CycloneDX SBOMs (Rust + web + Python) as workflow artifacts.
- Read-only permissions; never blocks publish.

## Tag namespace table

| Namespace | Owner workflow(s) | Publishes to |
|-----------|-------------------|--------------|
| `v*.*.*` | wheels-60, npm-61 | PyPI (`vantadb-py`), npm (`vantadb-wasm`, `vantadb`) |
| `node-v*.*.*` | npm-node | npm (Node binding) |
| `adapters-v*.*.*` | adapters-62 | PyPI (9 adapters) |
| `v*` (broad) | sbom-64 | Artifacts only (no registry) |
| GitHub Release | binaries-63 | Release assets (binaries) |

## Cascadas automaticas — `RELEASE_PLZ_TOKEN` (PAT)

release-plz crea tags/releases con `GITHUB_TOKEN`; GitHub suprime los eventos
generados por ese token, asi que los workflows tag-triggered (`v*.*.*`,
`node-v*.*.*`, `adapters-v*`) y `release: published` (binaries) **no se
disparan**. Solucion: fine-grained PAT `RELEASE_PLZ_TOKEN` (Contents RW +
Pull requests RW sobre `ness-e/Vantadb`) guardado como secret del repo.
`release.yml` lo usa con fallback (`secrets.RELEASE_PLZ_TOKEN ||
secrets.GITHUB_TOKEN`): con el PAT presente, el tag/release los crea el
usuario → wheels/PyPI, npm, SBOM y binaries se disparan solos; el Release PR
ademas gana checks (antes salian 0 por ser GITHUB_TOKEN).

### Fallback manual (PAT ausente/expirado)

```powershell
gh workflow run release-wheels.yml --ref vX.Y.Z
gh workflow run release-npm-61.yml --ref vX.Y.Z -f package=both
gh workflow run release-sbom.yml --ref vX.Y.Z
# environments pypi/npm: aprobar los pending deployments (owner)
```

## Post-release verification (release completion gate)

A release is **not complete when published** — it is complete when verified.
Owner: the release driver (`vanta-lead` / owner). Do not announce the release
(or close its checkpoint) until both steps below are green.

### 1. Mechanical check — artifacts live

```powershell
pwsh scripts/verify-release.ps1 -Tag vX.Y.Z -Smoke
```

Checks, for the tag: the 10 binary assets (5 targets × archive + `.sha256`) and
the 4 wheels on the GitHub Release; `vantadb` on crates.io; `vantadb-py` on
PyPI; `vantadb` + `vantadb-wasm` on npm; and (`-Smoke`) downloads the current
platform's archive, verifies its sha256 and runs `vanta-cli --version`.

Remote alternative (same script on a GitHub runner; also runs weekly against
`releases/latest`):

```powershell
gh workflow run release-verify.yml -f tag=vX.Y.Z
```

Exit 0 = ALL GREEN. If a check fails, remediate and re-run: binaries → backfill
(`gh workflow run release-binaries.yml -f release_tag=vX.Y.Z`, dispatched from a
ref whose version matches the tag); wheels/npm → re-run their workflow
([RUNBOOK.md](./RUNBOOK.md)).

### 2. Clean-machine smoke (manual, ~5 minutes)

Run the three install paths on a machine (or user profile) with no prior
VantaDB state:

- **Windows installer** — fresh PowerShell:

  ```powershell
  irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex
  vanta-cli --version   # expect the released version
  ```

  To pin a release: download the script, then
  `pwsh install.ps1 -Version vX.Y.Z` (`-InstallDir` overrides the default
  `$HOME\.vanta\bin`).
- **Python** — clean venv: `pip install vantadb-py==X.Y.Z` + the quickstart
  snippet ([QUICKSTART.md §5](../../user/QUICKSTART.md)).
- **npm** — temp dir: `npm install vantadb@X.Y.Z` + a `put`/`get` smoke.

### Known gaps (verified 2026-10-04)

| Gap | Status |
|-----|--------|
| Releases ≤ v0.7.0 have **no binary assets** | Historical: the `release:published` cascade was suppressed before `RELEASE_PLZ_TOKEN` (2026-10-02) and `release-binaries` only became operational at v0.8.0 (FIND-229 backfill). `releases/latest` (v0.8.0+) is unaffected. No backfill planned: old refs cannot produce honest binaries with the current workflow. |
| `vanta-memory` not on crates.io | On hold in `release-plz.toml` (DIST-01) — excluded from the check. |
| `vantadb-node` not on npm | Never published (404 documented) — excluded from the check. |
