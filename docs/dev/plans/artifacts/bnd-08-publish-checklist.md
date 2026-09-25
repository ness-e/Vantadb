# BND-08 — Checklist de publicación `vantadb-node` (para el humano)

> **Scope:** SOLO dry-run verificado por el worker. El publish real lo ejecuta el humano
> cuando decida. **Orden: GOV-TK2 (`/ship` GO) primero** — sin GO no hay publish.
> Verificado en seco el 2026-09-05 (BND-08): `npm pack` ✅ + `npm publish --dry-run` ✅.
> Prohibido para agentes: `npm publish` sin `--dry-run`.

## Evidencia dry-run (2026-09-05, máquina Windows, Node v26.8.1 / npm 11.6.0)

| Check | Resultado |
|---|---|
| `npm view vantadb-node@0.5.0 version` | **404 Not Found** — paquete nunca publicado, versión libre |
| `npm pack` (en `vantadb-node/`) | exit 0 → `vantadb-node-0.5.0.tgz` (package 2.0 MB / unpacked 5.5 MB, 6 files) |
| Contenido tarball | `README.md`, `index.cjs`, `index.d.ts`, `index.js`, `package.json`, `vantadb_native.win32-x64-msvc.node` (5.4 MB) — respeta `files` del package.json |
| `npm publish --dry-run ./vantadb-node-0.5.0.tgz` | exit 0 → `+ vantadb-node@0.5.0` (tag `latest`, registry `https://registry.npmjs.org/`, solo aviso "requires you to be logged in (dry-run)") |
| Workflow `.github/workflows/release-npm-node.yml` | job `publish` existe: `id-token: write` (OIDC), `environment: npm`, `registry-url: https://registry.npmjs.org`, skip-if-exists (`check-node`), smoke-test del tarball, `attest-build-provenance` (INFORMATIONAL, con CATEGORY tag OK) |
| shasum tarball local | `30130fb55e1595c12bbfb204b9c47c5cd54f4b1c` (referencia; el tarball de CI llevará los 7 binarios, no 1) |

**Nota local vs CI:** el tarball local incluye UN solo `.node` (win32-x64-msvc, el build de
esta máquina). El tarball real lo arma el job `publish` en CI descargando los 7 binarios
del matrix build (gnu+musl linux x64/arm64, macos x64/arm64, windows msvc) + artefactos JS.
El dry-run local valida empaquetado y metadata, NO los 7 binarios.

## Pre-condiciones (en orden)

1. [ ] **GOV-TK2 `/ship` = GO.** El release global (0.6.0) decide versionado y orden de
   publicación (crates.io → binarios → PyPI/npm). Sin GO, STOP.
2. [ ] **Trusted Publisher (OIDC) configurado en npmjs.com** para `vantadb-node`:
   - Package → Settings → Trusted Publisher → GitHub Actions:
     - Organization/repo del proyecto, workflow file `release-npm-node.yml`,
       environment **`npm`** (el workflow declara `environment: npm` + `id-token: write`).
   - Sin esto el job `publish` falla en auth (no hay `NPM_TOKEN` en el workflow — es OIDC puro).
3. [ ] **Environment `npm` en GitHub** (Settings → Environments): protection rules / reviewers
   según criterio del owner; el job `publish` lo requiere (`environment: npm`).
4. [ ] **Versión decidida:** hoy `0.5.0` libre (404). Si GOV-TK2 bumpea a `0.6.0` primero,
   actualizar la expectativa (`npm view vantadb-node@<ver>` debe seguir 404) — la versión
   npm la gobierna el release global, no este checklist.

## Publicación (humano)

5. [ ] **Ensayo en CI (recomendado):** `gh workflow run "RELEASE: NPM — vantadb-node"`
   con input `dry_run: "true"` → job `publish` corre `npm publish --dry-run *.tgz` en CI
   (con los 7 binarios reales). Verde = luz para el paso 6.
6. [ ] **Tag + push:** `git tag node-v<ver> && git push origin node-v<ver>`
   (el workflow dispara con `tags: ["node-v*.*.*"]`). Alternativa: `workflow_dispatch`
   con `dry_run: "false"`.
7. [ ] **Vigilar CI:** matrix build (7 targets) → `test` (linux gnu + artefactos JS) →
   `publish`: `Verify all targets present` (≥5 `.node`) → `Pack` → `check-node`
   (skip si la versión ya existe) → smoke-test (`VantaDb.connect`) → attest → publish.
8. [ ] **Verificar live:**
   - `npm view vantadb-node@<ver> version` → `<ver>` (ya no 404)
   - `mkdir /tmp/smoke && cd /tmp/smoke && npm init -y && npm install vantadb-node@<ver> && node -e "const { VantaDb } = require('vantadb-node'); console.log(typeof VantaDb.connect)"` → `function`
   - Probar en linux-glibc, linux-musl (Alpine) y macos-arm64 si hay acceso (los 3 libc/archivos críticos).

## Rollback

- npm permite `npm unpublish vantadb-node@<ver>` solo dentro de las **72 h** y si no hay
  dependientes; pasado ese plazo: `npm deprecate vantadb-node@<ver> "<motivo>"` + publicar
  patch de fix. El tag git NO se borra salvo que el publish nunca haya ocurrido.
- Si el publish parcial subió el tarball pero el smoke-test live falla: deprecate inmediato
  + fila `FIND-*` en Backlog con el log del job `publish`.

## Referencias

- Workflow: `.github/workflows/release-npm-node.yml` (build matrix 7 targets, job `publish`)
- Paquete: `vantadb-node/package.json` (`name`, `version`, `files`, `napi.targets`, `engines >=18`)
- `publish = false` en `vantadb-node/Cargo.toml` es del **crate Rust** (no va a crates.io);
  irrelevante para npm. El crate es standalone (`[workspace]` vacío) a propósito
  (MSVC linker crash con cdylib — ver `.opencode/rules/js-ecosystem.md` R-3): no "arreglar".
- Release global: `.opencode/skills/campaign-executor/tasks/GOV-TK2.md` (D5 `/ship` GO).
- Siguiente task técnica: **BND-09** (matriz CI musl — targets ya presentes en
  `package.json` + workflow; BND-08 la desbloquea).
