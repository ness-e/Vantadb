---
title: Workflows — Publish flow per registry
kind: runbook
status: active
description: "Publishing per registry: crates.io via Trusted Publishing (OIDC); PyPI and npm via repo tokens. Traps and pre-flight in the 0.9.0 post-mortem."
tags: [vantadb, ci, workflows, publish, release]
---

# Workflows — Publish flow per registry

**Mixed model (2026-10-20):** crates.io publishes via **Trusted Publishing
(OIDC)**, PyPI and npm via **repo tokens** (`secrets.PYPI_TOKEN`,
`secrets.NPM_TOKEN`). Tokens are required for a *first* publish of a package
that does not exist yet (npm staging cannot create packages; crates.io
Trusted Publishing requires the crate to exist).

> ⚠️ The state of this doc changed with the 0.9.0 release. It previously
> described publishing as "tokenless (OIDC Trusted Publishing, no PATs)" and a
> set of tag namespaces (`node-v*`, `adapters-v*`, `providers-v*`) that the
> fan-out does not use. See
> [`2026-10-20-release-v0.9.0-postmortem.md`](../reviews/2026-10-20-release-v0.9.0-postmortem.md).

Rollback policy per plan: `git revert` per commit + re-run.

## Pre-flight checklist (obligatoria antes de mergear un release PR)

El release de v0.9.0 fallo **3 veces** por motivos distintos y **ninguno lo
cazo un gate**. Post-mortem completo (cronologia, causas, arreglos y lo que
falta): [`2026-10-20-release-v0.9.0-postmortem.md`](../reviews/2026-10-20-release-v0.9.0-postmortem.md).

1. **Los secrets existen y no estan vacios.** `gh secret list`. En
   `cargo publish` la *presencia* de `CARGO_REGISTRY_TOKEN` **desactiva** el
   Trusted Publishing aunque este vacia → `"please provide a non-empty token"`.
   Para crates.io: **OIDC** (la variable **no** debe estar en el `env` del
   step) o un token con scope de publish. Nunca las dos.
2. **Un token solo esta verificado si un publish real lo acepta.**
   `cargo owner --list` **no autentica** (lee el fichero estatico de owners
   del indice git) y `npm whoami` autentica pero no prueba el scope de
   publish.
3. **Un canal que publica un paquete nuevo** necesita token
   *direct-capable*: el staging de npm no crea paquetes (`E_STAGE_REQUIRED`) y
   el Trusted Publishing de crates.io exige que el crate ya exista.
4. **`rust-toolchain.toml` vs `dtolnay/rust-toolchain`:** si el workflow usa un
   canal distinto del pin del repo, el fichero gana al invocar `cargo`/`pip` y
   puede colisionar con el binario del toolchain por defecto →
   `"clippy-preview … detected conflict: 'bin/cargo-clippy'"`. Fijar
   `RUSTUP_TOOLCHAIN` en el step.
5. **Un smoke test no debe resolver una version recien publicada** en el mismo
   run (carrera de propagacion del registry → `ETARGET`). Backoff, no fallo.
6. **`gh workflow run` sin checkout → `-R "${{ github.repository }}"`.**
7. **La rama del PR de release empieza por `release-plz-`** si el PR debe
   disparar la publicacion: `release-plz` solo publica si el ultimo commit de
   `main` pertenece a un PR cuya rama empieza por `release-plz-`. Mergear desde
   `develop` deja el release sin disparar.
8. **Verificacion final:** `pwsh scripts/verify-release.ps1 -Tag v<version>
   -Smoke` (assets + registries + sha256 + `vanta-cli --version`).
9. **Higiene post-release:** sincronizar `develop` con `main` (ff-only) para que
   un `develop → main` futuro no revierta fixes que solo estan en `main`.

## Chain overview

```text
merge develop -> main
  -> release.yml (release-plz: bump, changelog, git tag, crates.io)
    -> tag v*.*.*  -> 7 downstream lanes, TODOS con el MISMO tag core:
         release-wheels.yml     (PyPI vantadb-py)
         release-npm-61.yml     (npm vantadb + vantadb-wasm)
         release-npm-node.yml   (npm vantadb-node)
         release-adapters.yml   (PyPI, 7 adapters)
         release-providers.yml  (PyPI, 3 providers)
         release-sbom.yml       (SBOM)
         release-binaries.yml   (GitHub Release assets)
```

> Los lanes no usan namespaces de tag propias: el fan-out los despacha a todos
> con `--ref <tag core>` (ver "Cascadas automaticas").

A release is **not complete when published** — it is complete when the
[post-release verification](#post-release-verification-release-completion-gate)
below is green (assets + registries + smoke).

## develop→main: merge commit, no squash (2026-10-02)

Los PRs `develop → main` se mergean con **merge commit** (`gh pr merge --merge`), no `--squash`: release-plz lee los commits convencionales del historial para generar el changelog, y con squash solo ve el commit resumen (changelog "### Other" — pasó en 0.7.0 y 0.8.0). Con merge commit el changelog sale rico automáticamente (validado en 0.6.x). Los Release PR de release-plz siguen squash (1 commit).
## crates.io — `release.yml`

- Trigger: `push: branches: [main]` only.
- `release-plz-release` job: version bump, changelog, tag push, `cargo publish`
  via OIDC (`id-token: write`). Timeout 45 min, no cancel (`cancel-in-progress: false`).
- `release-plz-pr` job: opens the Release PR on subsequent pushes. Timeout 45 min:
  `release-plz release-pr` runs cargo-semver-checks, a full cold compile of the
  workspace + the published baseline (13-23 min depending on cache and runner
  contention), plus ~4 min to determine the version. At 15 min this job was
  killed mid-check (run 39324117624, job 118306356063) - do not lower the
  timeout without a fresh measurement.

### Trampa: `CARGO_REGISTRY_TOKEN` no puede existir "de sobra"

Cargo decide entre token y Trusted Publishing (OIDC) por la **PRESENCIA** de la
variable de entorno, no por su valor. Si el workflow la define y el secret no
existe, la variable queda **definida y vacía**: cargo la toma como "token
provisto", se salta el intercambio OIDC y el publish muere con

```
error: failed to publish vantadb v0.9.0 to registry at https://crates.io
Caused by: please provide a non-empty token
```

(pasó en el publish de 0.9.0: run 39368867477, job 118404027216 - el crate
compiló bien en 2m22s y solo falló al autenticar. v0.8.0 había publicado bien
justo porque esa línea no estaba.)

Regla: **si defines `CARGO_REGISTRY_TOKEN`, el secret tiene que existir y
tener valor**. Si prefieres OIDC, quita la línea del todo. Con las dos cosas
a la vez, el OIDC nunca arranca.

### Modelo de autenticación por canal (2026-10-20)

| Canal | Mecanismo | Secret | Workflow |
|---|---|---|---|
| crates.io | **Trusted Publishing (OIDC)** | — (el secret existe, pero **no** va en el `env` del step) | `release.yml` |
| PyPI (prod) | token | `PYPI_TOKEN` | `release-wheels`, `release-providers`, `release-adapters` |
| PyPI (test) | token | `TEST_PYPI_TOKEN` | los mismos tres, en el job `publish-testpypi` |
| npm | token (`NODE_AUTH_TOKEN`) | `NPM_TOKEN` | `release-npm-61`, `release-npm-node` |
| GitHub (tag/PR) | PAT con fallback | `RELEASE_PLZ_TOKEN` | `release.yml` |

**Por qué crates.io va con OIDC y los demás con token:** el token de crates.io
disponible devuelve 403 al publicar (`403 Forbidden: authentication failed`,
run 39377761363), mientras que el Trusted Publishing funcionó para el 0.8.0
(`trustpub_data.run_id = 37045089945`). En PyPI y npm los tokens **sí**
autentican y **sí** pueden crear paquetes nuevos (el staging de npm no puede:
`E_STAGE_REQUIRED`).

**Trampa de verificación:** `cargo owner --list` **no autentica** (lee el
fichero estático de owners del índice git) - un token "validado" así puede
seguir dando 403 al publicar. Un token solo está verificado si un publish
real lo acepta.

Ventaja del token frente a OIDC: sirve igual para el **primer publish** de un
paquete o crate que aún no existe (`vantadb-node` en npm, `vanta-memory` en
crates.io), donde Trusted Publishing todavía no puede configurarse.

## PyPI wheels — `release-wheels.yml`

- Tags `v*.*.*` publish to prod PyPI (`environment: pypi`, token
  `PYPI_TOKEN` + provenance attestation); `workflow_dispatch` with
  `publish_testpypi=true` publishes to TestPyPI (`environment: testpypi`, token
  `TEST_PYPI_TOKEN`).
- PRs to `main` touching `src/`, `vantadb-python/`, `Cargo.*` run build +
  smoke only (no publish).
- Tag namespaces: strict `v*.*.*` (three numeric parts).

## npm WASM + TS — `release-npm-61.yml`

- Tags `v*.*.*` publish both packages (`environment: npm`, token
  `NPM_TOKEN` via `NODE_AUTH_TOKEN`):
  `publish-wasm` builds `vantadb-wasm` with wasm-pack, then `publish-ts`
  rewrites the exact wasm version into `vantadb-ts` and publishes `vantadb`.
- PRs touching `vantadb-wasm/**` or `vantadb-ts/**` run CI only.
- `workflow_dispatch` selects `wasm` / `ts` / `both` + `dry_run`.

## npm Node — `release-npm-node.yml`

- Tags `node-v*.*.*` publish the Node binding (`environment: npm`, OIDC).
- PRs touching `vantadb-node/**` run CI only.
- Own namespace so core `v*` releases never publish the Node package by accident.

## Orden del bump npm y sincronización multi-manifiesto (FIND-230, FIND-379) — gate `check-npm-versions`

### Automatic version synchronization across manifests (FIND-230, FIND-379)

A partir de FIND-379, la sincronización de versiones entre todos los manifiestos del monorepo está completamente automatizada mediante el script `scripts/docs/sync-workspace-version.mjs` y el job `release-plz-pr` en `.github/workflows/release.yml`.

Cuando `release-plz` abre o actualiza el Release PR en la rama `release-plz-*`, el paso `Sync workspace version across monorepo manifests on Release PR branch` ejecuta:
```bash
node scripts/docs/sync-workspace-version.mjs --write
```
Este script sincroniza automáticamente con la versión de `[workspace.package]` en `Cargo.toml`:
1. `vantadb-ts/package.json`: actualiza `"version": "<version>"`.
2. `server.json`: actualiza `"version": "<version>"` (descriptor público del registro MCP).
3. `docs/api/openapi.yaml`: actualiza `  version: "<version>"` en el bloque `info:`.
4. `docs/api/MCP.md`: actualiza `Current MCP implementation version: <version>`.
5. Todos los 9 adapters en `integrations/{crewai,dspy,haystack,langchain,letta,llamaindex,mem0,ollama,openai}/pyproject.toml`: actualiza la cota superior `"vantadb-py>=X.Y.Z,<0.N.0"` a `<0.${minor+1}.0`.
6. `docs/api/SURFACE_MANIFEST.md`: regenera el inventario de superficies públicas vía `node scripts/docs/gen-surface-manifest.mjs --write`.

Si se detectan cambios, el bot de GitHub Actions commitea directamente a la rama del Release PR (`chore(release): sync monorepo surface manifests to v<version>`), garantizando que cuando el PR se mergee a `main`, todos los manifiestos estén alineados y los gates de CI (`check-npm-versions`, `version_coherence.rs`, etc.) pasen en verde.

El script puede ejecutarse manualmente o en CI:
- `node scripts/docs/sync-workspace-version.mjs --check`: verifica que no haya desalineación (exit 0 si está sincronizado, exit 1 si hay drift).
- `node scripts/docs/sync-workspace-version.mjs --write`: actualiza los archivos desalineados en disco.
- `node scripts/docs/sync-workspace-version.mjs --self-test`: ejecuta validaciones de transformación en memoria (exit 0).

`vantadb-ts/package.json` debe estar **en la misma versión que `[workspace.package]`** para que el tag publique el tren completo. Si queda atrás, `release-npm-61.yml` pregunta por la versión vieja, npm responde "ya publicada" y el publish **se saltea en un run verde** — pasó en 0.8.0: TS quedó en 0.7.0, el tag `v0.8.0` publicó wasm y para `vantadb` solo imprimió `Version 0.7.0 already published — skipping` (run `37045932895`, conclusión success); el backfill fue manual (`e62e0f62` + dispatch).

Regla mecánica: el gate **`Check npm package versions`** (`gate-docs.yml`, script `scripts/docs/check-npm-versions.mjs` — mismo script local y CI) falla si `vantadb-ts/package.json` ≠ versión del workspace.

Excepción `vantadb-node`: **nunca publicado** en npm (registry 404, 2026-10-03). Su versión puede quedarse en `0.7.0` (versión "never-published" documentada en el script); a partir de su primer release debe igualar la versión del workspace (y la entrada `TARGETS` del script — `scripts/docs/check-npm-versions.mjs` — pasa manualmente a `policy: 'workspace'`). `vantadb-wasm` no necesita gate: su manifest npm lo genera wasm-pack desde el crate, que hereda `version.workspace = true`.

El skip "already published" de `release-npm-61.yml` y `release-npm-node.yml` ahora emite `::warning::` visible en el run — ya no es un skip mudo.

## Adapters — `release-adapters.yml`

- Tags `adapters-v*.*.*` publish the 7 adapters
  (langchain, llamaindex, mem0, crewai, dspy, haystack, letta) to prod PyPI
  after the test matrix passes.
- `workflow_dispatch` with `publish_testpypi=true` goes to TestPyPI.
- Own namespace so adapter-only releases never trigger core wheels/npm.
- **FIND-273 — resolved 2026-10-06 (owner decision):** the
  `integrations/openai` + `integrations/ollama` Python twins were **retired
  from the release** (their PyPI names/modules belong to the Rust providers);
  they stay in-repo as source-only, and the workflow matrices now carry the
  7-adapter list. Details: MKT-20 §F6-2.
- **Pins — bumped 2026-10-06 (owner decision):** the 7 adapters now declare
  `vantadb-py>=0.6.1,<0.9.0` (was `>=0.5.0,<0.7.0`, which excluded the live
  core and admitted 0.5.0 — no `Client` API). Evidence: MKT-20 §F6-3.
- **Publish = owner-assisted (F6/Task 71). Checklist:**
  1. **Publishers — new names need a token first (PyPI constraint, verified
     2026-10-07):** PyPI allows **one pending publisher per
     (repository, workflow, environment) tuple** — a second pending
     publisher for another project name with the same tuple is rejected
     ("already been registered for a different project name"). Since all 7
     adapters share `ness-e/Vantadb` + `release-adapters.yml` + `pypi`, the
     pending-publisher route **cannot create the 7 names**. Correct path:
     (a) **first publish of each new name with an API token** (`twine
     upload`; back off on 429 — PyPI rate-limits new-project creation), then
     (b) configure the **normal** GitHub publisher per project (Manage →
     Publishing; normal publishers are shared objects and CAN carry the same
     tuple across projects). `publish-pypi` (tag) has `skip-existing: true`,
     so it re-runs gracefully once the names exist. Environments already
     exist (verified 2026-10-06). Historical state: 4/7 published by token
     2026-10-07 (langchain, crewai, dspy, haystack).
  2. **Dry-run:** `gh workflow run release-adapters.yml -f publish_testpypi=true`
     → approve the `testpypi` environment deployment → ≥5 dists on
     test.pypi.org (expected: 14 — 7 wheels + 7 sdists).
  3. **Tag:** `git tag adapters-v0.5.0 && git push --tags` → approve the
     `pypi` environment deployment → `publish-pypi` green (OIDC).
  4. **Post:** `GET https://pypi.org/pypi/<name>/json` = 200 ×7; remove the
     "Not on PyPI yet" notices from the READMEs + `FRAMEWORKS.md` and re-run
     `node scripts/docs/gen-index.mjs --check` (cleanup commit).

## Providers — `release-providers.yml`

- Tags `providers-v*.*.*` publish the 3 Rust providers (`vantadb-openai`,
  `vantadb-ollama`, `vantadb-litellm`) to prod PyPI after the 12-wheel matrix
  passes (3 providers × linux-x86_64 / macOS / Windows / linux-aarch64,
  maturin-action, abi3-py311).
- `workflow_dispatch` with `publish_testpypi=true` goes to TestPyPI
  (+ `verify-testpypi-install`); PRs touching `providers/**` build the same
  matrix without publishing.
- Own namespace so provider releases never trigger core wheels/npm/adapters.
- **Name ownership (owner decision 2026-10-04, PROV-12):** the three canonical
  names belong to the Rust providers; the `integrations/openai` +
  `integrations/ollama` Python twins must be renamed or retired before their
  F6 publish (`FIND-273`).
- **Publish = owner-assisted (PROV-12). Checklist:**
  1. GitHub environments `pypi` + `testpypi`; publishers for the three names
     — **same PyPI pending-publisher constraint as §Adapters (2026-10-07):
     token-first for new names, then normal publishers** — or token.
  2. Dry-run: `gh workflow run release-providers.yml -f publish_testpypi=true`
     → 12 wheels on test.pypi.org + `verify-testpypi-install` green.
  3. Tag: `git tag providers-v0.5.0 && git push --tags` → `publish-pypi` +
     `verify-pypi-install` green.
  4. Post: `GET https://pypi.org/pypi/<name>/json` = 200 ×3; remove the
     "after first release" caveats from the three provider READMEs.

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
| `adapters-v*.*.*` | adapters-62 | PyPI (7 adapters) |
| `providers-v*.*.*` | release-providers.yml | PyPI (`vantadb-openai`, `vantadb-ollama`, `vantadb-litellm`) |
| `v*` (broad) | sbom-64 | Artifacts only (no registry) |
| GitHub Release | binaries-63 | Release assets (binaries) |
| manual dispatch only | publish-mcp-registry.yml | MCP Registry (`io.github.ness-e/vantadb`) — owner-gated, first publish = owner login/claim |

## MCP Registry — `publish-mcp-registry.yml` (FIND-344)

- `server.json` (repo root) is the version source; validated against its
  `$schema` by `scripts/validate-mcp-server.py` (offline, fail-closed).
- The workflow runs ONLY on `workflow_dispatch` (environment `mcp-registry`,
  owner approval): validate → stamp version → `mcp-publisher publish`
  (OIDC). No tag triggers until the first manual publish succeeds.
- First publish requires the owner: registry claim for `io.github.ness-e/*`
  (`mcp-publisher login github`) + `.mcpb` bundle decision (GAP documented
  in `docs/dev/tasks/FIND-344.md` — no `packages[]` invented).

## Crates — publish decision per crate (DIST-06)

> **Policy (reviewed 2026-10-04, DIST-06).** One place to answer "what ships
> where" per crate — this removes the ambiguity behind FIND-230 (a publish
> silently skipped in a green run). A crate reaches crates.io only if its
> manifest is publishable **and** release-plz manages it; crates that ship
> through another registry (PyPI, npm) or inside a binary keep
> `publish = false` as the cargo-level enforcement.
>
> **Review triggers:** v1.0.0 (Freeze List lift) · first `vantadb-node` npm
> publish · first `providers-v*` publish (PROV-12 prepared 2026-10-04 —
> publish = owner) · `vanta-memory` bootstrap (DIST-01 unblock).

| Crate | Manifest | Decision | Channel (real) | Produced by | Why |
|-------|----------|----------|----------------|-------------|-----|
| `vanta-memory` | publishable | Publish to crates.io (**hold**) | crates.io | `release.yml` (release-plz) | DIST-01: publishable + `release = false` hold until Trusted Publishing bootstrap (owner) — see DIST-01 |
| `vantadb-python` | `publish = false` | No crates.io publish | PyPI `vantadb-py` wheels | `release-wheels.yml` (`v*.*.*`) | cdylib extension module (PyO3) — PyPI is the product channel |
| `vantadb-node` | `publish = false` | No crates.io publish | npm `vantadb-node` | `release-npm-node.yml` (`node-v*.*.*`) | napi cdylib — npm is the channel; never published yet (404 documented) |
| `vantadb-server` | `publish = false` | No crates.io publish | GitHub Release binaries (5 targets) | `release-binaries.yml` | Binary product — distributed as release assets, not as a crate |
| `vantadb-mcp` | `publish = false` | No publish | Library embedded in `vantadb-server` (`--mcp`) | `release-binaries.yml` (transitive) | Library-only crate (no bin target) — ships inside the server binary |
| `vanta-proxy` | `publish = false` | No publish (until 1.0.0) | Source build only | — | Freeze List (owner 2026-10-01): frozen, publish deferred to 1.0.0 (DIST-18 re-scoped) |
| `providers/openai` | `publish = false` | No crates.io publish | PyPI `vantadb-openai` wheels | `release-providers.yml` (`providers-v*.*.*`) | PyO3 extension (engine embedded); PyPI is the product channel — PROV-12 |
| `providers/ollama` | `publish = false` | No crates.io publish | PyPI `vantadb-ollama` wheels | `release-providers.yml` (`providers-v*.*.*`) | Same as `providers/openai` |
| `providers/litellm` | `publish = false` | No crates.io publish | PyPI `vantadb-litellm` wheels | `release-providers.yml` (`providers-v*.*.*`) | Same as `providers/openai` |
| `vantadb-ffi-core` | `publish = false` | No publish | None — compiled into the python/wasm/node bindings | — | Std-only leaf shared by the FFI transports; zero external consumers |
| `fuzz` | `publish = false` | No publish | None (dev tool) | `fuzz.yml` (CI) | cargo-fuzz harness; outside the workspace; never distributed |

Publishable crates for context (they complete the release-plz invariant):

| Crate | Manifest | Channel | Produced by | Why |
|-------|----------|---------|-------------|-----|
| `vantadb` (core + `vanta-cli`) | publishable | crates.io + GitHub Release binaries | `release.yml` (release-plz) + `release-binaries.yml` | The only crates.io-published crate today (0.8.0 live) |
| `vantadb-wasm` | publishable | npm `vantadb-wasm` (wasm-pack) | `release-npm-61.yml` (`v*.*.*`) | WASM bindings ship via npm — `release = false` in `release-plz.toml` |

### Invariant — `release-plz.toml` consistency

- `[workspace] publish = true` — release-plz manages the publishable
  workspace members; the `[[package]]` overrides are the only exceptions and
  must stay a subset of this page.
- Current overrides: `vantadb` (changelog path `docs/CHANGELOG.md`),
  `vantadb-wasm` (`release = false` — npm channel), `vanta-memory`
  (`release = false` — DIST-01 hold). `release = false` means release-plz
  ignores the package entirely (no update/changelog/tag/publish); a
  `publish = false` manifest is a hard `cargo publish` block.
- Empirically (0.8.0 Release PR `ef2e33bd`): the release touched only the
  workspace version (`Cargo.toml`), `Cargo.lock`, `docs/CHANGELOG.md` (single
  `vantadb` changelog), two API docs (`MCP.md`, `openapi.yaml`) and this
  document — no `publish = false` crate got its own version bump, changelog
  file or tag.
- Channel-change protocol: manifest + `release-plz.toml` + this table change
  in the same PR (the `vanta-memory` unblock checklist in `release-plz.toml`
  is the template).

Re-verify by command — both must match this page:

```powershell
# (a) manifest scan → exactly the 10 `publish = false` crates above
rg -n "^\s*publish = false" --glob "**/Cargo.toml" -g "!target/**" -g "!node_modules/**"

# (b) release-plz overrides → subset of {vantadb, vantadb-wasm, vanta-memory}
#     (output also shows the workspace-level `publish = true` on line 6)
rg -n "^\[\[package\]\]|^name = |^\s*(release|publish) = " release-plz.toml
```

## Cascadas automaticas — cascada implicita por PAT (primaria) + job `fanout` (fallback)

### Primario: cascada implicita por PAT (lo que dispara de verdad)

`release-plz` crea el tag y el release con `RELEASE_PLZ_TOKEN` (fine-grained
PAT, Contents RW + Pull requests RW sobre `ness-e/Vantadb`) con fallback
`secrets.RELEASE_PLZ_TOKEN || secrets.GITHUB_TOKEN`. **Con el PAT presente, el
tag lo crea el usuario, no el bot**: GitHub NO suprime los eventos generados
por un PAT, asi que el push del tag dispara por si solo los 7 workflows
tag-triggered, y `release: published` dispara el de binarios. Ademas el Release
PR gana checks (antes salian 0 por generarse con `GITHUB_TOKEN`).

Verificado en el release 0.9.0: el push del tag `v0.9.0` lanzo los 7 lanes
(wheels, npm x2, adapters, providers, sbom, binaries) sin que el job `fanout`
tuviera que hacer nada.

### Fallback: job `fanout` en `release.yml` (FIND-342/F1)

Si el tag se crease con `GITHUB_TOKEN`, GitHub suprime los eventos derivados y
nadie publicaria. Para ese caso, tras `release-plz-release` (si hubo release,
`releases_created`), un job `fanout` despacha por API los **7** lanes sobre el
tag publicado (`gh workflow run <downstream> -R "${{ github.repository }}" --ref
<tag>`, tag derivado de los outputs `releases`/`releases_created`):

- `release-wheels.yml --ref <tag>` (sin inputs)
- `release-npm-61.yml --ref <tag> -f package=both` (`package` es requerido)
- `release-npm-node.yml --ref <tag> -f publish=true`
- `release-adapters.yml --ref <tag> -f publish_pypi=true`
- `release-providers.yml --ref <tag> -f publish_pypi=true`
- `release-sbom.yml --ref <tag>` (sin inputs)
- `release-binaries.yml --ref <tag> -f release_tag=<tag>`

El `-R` es **obligatorio**: el job no hace checkout, asi que sin el repo
explicito `gh` aborta con `fatal: not a git repository` (run 39379695535).

> Con el PAT presente esta cascada es **redundante**: el push del tag ya
> dispara los 7 lanes y el fan-out los duplica (cada duplicado es un no-op
> gracias al gate de "version already published", pero quema minutos de
> runner). Es el precio de cubrir el caso `GITHUB_TOKEN`.

No toca ningun `if:` downstream: cada run corre SOBRE el tag, asi que
`github.ref` es `refs/tags/v*` dentro de cada lane (wheels/npm/sbom publican
por su gate de tag; binaries por `inputs.release_tag != ''`; npm-node publica
por la rama `workflow_dispatch` de su `if:`). pypi/npm siguen detras de su
`environment:` — el owner aprueba los pending deployments.

> Con `GITHUB_TOKEN` el dispatch NO dispara downstreams (GitHub suprime
> eventos derivados), asi que sin PAT el job falla en rojo en vez de
> verde-vacio.

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
