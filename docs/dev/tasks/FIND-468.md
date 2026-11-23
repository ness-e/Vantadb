---
id: FIND-468
title: "Fallar cerrado (exit 1) en sync-workspace-version.mjs y cablear --check en docs-gates.yml"
type: fix
state: IN_PROGRESS
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
absorbe: FIND-469
depends: FIND-466
---

# FIND-468 — sync-workspace-version: fail-closed + gate CI

## Objetivo

Dos defectos:

1. `scripts/docs/sync-workspace-version.mjs` en modo `--write` sale siempre con
   código 0 (`process.exit(0)` tras el loop) aunque una regex no haga match y el
   manifiesto siga desincronizado — fallo silencioso que arruinaría el bump a
   0.10.0.
2. `docs-gates.yml` no ejecuta `--check`: CI no detecta la deriva.

## Contrato

```
node scripts/docs/sync-workspace-version.mjs --check && rg "sync-workspace-version\.mjs --check" .github/workflows/docs-gates.yml
```

**Estado ANTES (medido):** `--check` exit 1 con
`DRIFT vanta-memory/Cargo.toml: found '(unreadable)'`.

## Diagnóstico del drift de vanta-memory (causa raíz)

`vanta-memory/Cargo.toml` declaraba `version.workspace = true` en `[package]` y
`vantadb = { workspace = true }` en `[dependencies]`. El parser
`parseVantaMemoryDepRequirement` buscaba un literal
`vantadb = { ... version = "X" }` que YA NO EXISTE en ese fichero → devuelve
`null` → se reporta `(unreadable)` → DRIFT falso positivo.

Veredicto: **(iii) expectativa obsoleta de la tabla de targets**. Según el Cargo
Book, `dep = { workspace = true }` hereda el requisito completo de
`[workspace.dependencies]` del raíz
(https://doc.rust-lang.org/cargo/reference/workspaces.html — secciones
"The `package` table" y "The `dependencies` table"). El literal a sync vive
ahora en `Cargo.toml` raíz líneas 820-821:

```toml
vantadb = { path = ".", version = "0.9.0", default-features = false }
vanta-memory = { path = "vanta-memory", version = "0.9.0", default-features = false }
```

El arreglo es del lector/tabla de targets, NO del manifiesto: el target pasa a
ser `Cargo.toml [workspace.dependencies]` verificando los requisitos `version`
de los path-deps internos `vantadb` y `vanta-memory` (el equivalente exacto de
lo que el target anterior protegía). `vantadb-node/Cargo.toml` ya cubre su
`vanta-memory = { path = ..., version = "0.9.0" }` vía `parseNodeCargoVersion`.

## Impacto mapeado (Regla 0)

| Archivo | Rol | Leido |
|---|---|---|
| `scripts/docs/sync-workspace-version.mjs` | parsers (:194-225), targets (:320-326), self-test (:442-453), write loop (:514-574) | SI (completo) |
| `.github/workflows/docs-gates.yml` | jobs de version-check (`check-api-version`, `check-npm-versions`) | SI (completo) |
| `Cargo.toml` | `[workspace.package]` :813-817, `[workspace.dependencies]` :819-822 | Parcial (seccion) |
| `vanta-memory/Cargo.toml` | `version.workspace = true` :3, `vantadb = { workspace = true }` :20 | SI (completo) |
| `docs/dev/workflow/RULES.md`, `PUBLISH.md` | reglas CI / runbook release | Referencia |

**Referencias entrantes:** nadie invoca el script hoy (ese es el defecto b).
`release*.yml` puede invocarlo en sync de manifiestos — el cambio de exit code
de `--write` es compatible: solo pasa de 0 a 1 cuando el fix no se aplicó.

**Veredicto:** blast radius = pipeline de release (versiones descoordinadas
entre registries = trampa del post-mortem 0.9.0). 2 archivos a editar.

## Steps

1. ✅ Script: `--write` verifica `spec.parse(transformed) === spec.expected`
   ANTES de escribir; miss → FAIL + contador `failedCount`; write error también
   cuenta; exit final `failedCount ? 1 : 0`. Actualizar header exit codes.
2. ✅ Script: reemplazar target `vanta-memory/Cargo.toml` por
   `Cargo.toml [workspace.dependencies]` (parse+transform de `vantadb` y
   `vanta-memory` version reqs). Actualizar self-test #10 con la nueva fixture.
3. ✅ `docs-gates.yml`: añadir step `node scripts/docs/sync-workspace-version.mjs --check`
   en el job `check-npm-versions` (gate natural de "versions match workspace").
4. ✅ Verificar: `--self-test` ok, `--check` exit 0, `rg` match, regex-miss en
   `--write` exit != 0 (caso construido con server.json), `--write` idempotente
   (0 updates, exit 0, sin diff).
5. ✅ Commit local `fix(release): fail loud on version sync miss and enforce CI check`.

## Verificación

- `node scripts/docs/sync-workspace-version.mjs --self-test`
- `node scripts/docs/sync-workspace-version.mjs --check` → exit 0
- `rg "sync-workspace-version\.mjs --check" .github/workflows/docs-gates.yml`
- Regex-miss: `server.json` con `"version": 123` → `--write` exit != 0

## Review

**P2-01 — tier FAST** (`scripts/**` + `.github/workflows/**` no matchean globs adversariales).
- mode: fresh · reviewer: vanta-review (subagente contexto fresco) · author: vanta-lead
- verdict: **APPROVE** — red-team: regex no cruza inline tables (`[^}]*`), `vantadb-mcp`/`vantadb-ffi-core` no matchean `^vantadb\s*=`, unreadable/write-error/read-error todos fail-closed en --write, CI placement correcto, diff higiénico.
- Nits no bloqueantes: primera ocurrencia por nombre (documentado), self-test añadido a CI (aplicado).
- Evidencia: `--self-test` 13/13 · `--check` exit 0 (22 targets) · `--write` sobre MCP.md corrupto exit 1 sin tocar el fichero · `--write` idempotente (0 updates, git diff vacío en generados).

SDP: ci-cd-and-automation, shipping-and-launch, git-workflow-and-versioning
