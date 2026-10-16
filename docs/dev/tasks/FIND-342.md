---
title: "FIND-342 — Paquete release-gaps B/C/D/E/F (decisiones owner 2026-10-16)"
kind: task
status: active
description: "Cerrar gaps de release: server.json, Formula, manifest, tarball-check y fan-out sin PAT"
tags: [release, ci, packaging]
---

# FIND-342 — Paquete release-gaps B/C/D/E/F

- **Origen:** decisiones owner 2026-10-16 (todo verificado en repo+red por el orquestador)
- **Branch:** `develop` · **Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
- **Tipo auto-detectado:** release/CI (packaging + workflows + scripts, sin lógica de negocio)
- **PROHIBIDO:** push, tags, PRs, tocar versiones para publicar, CHANGELOG (release-plz),
  publicar nada (ni dry-run con upload), `Cargo.toml` versions, `vantadb-python/**`,
  stash, `.opencode/**`, `docs/index.md`, `llms.txt`, `src/**`, `tests/**`,
  `docs/api/**` (solo lectura), `feature-freez/**`, `docs/dev/plans/`,
  archivos de EGO-03 en curso.
- **Dependencias:** independiente de EGO-03 (archivos disjuntos). Next: DIST-19 tras EGO-03.
- **Referencias:** `docs/dev/workflow/PUBLISH.md` (cascada + namespaces + DIST-06),
  `docs/dev/workflow/RULES.md` §4-5 (OIDC, triggers, tags) + release-ci rules,
  Backlog BIZ-16/17 (firma/auditoría: contexto, no ejecutar).
- **SDP:** `codebase-memory` (blast radius por slice, release path = crítico) ·
  `documentation-skill` (edito `PUBLISH.md` + este task file bajo `docs/`) ·
  `progreso` (lectura: Trigger 2 — `FIND-342` no existe en `docs/dev/avance/` ni `Backlog.md`) ·
  ponytail full (modo, no skill). Sin candidatos beyond base.

## Spec

| Slice | Decisión owner | Contrato mecánico |
|---|---|---|
| B | `server.json` 0.6.1→0.8.0 | `version` = workspace (`0.8.0`); `timestamp` honesto (fecha real de regeneración); `schema` queda `2025-12-11` (= `$schema`); `submission_state` queda `pending` (el submit al registry lo hace el owner con login). + gate de versiones para `server.json`: extender patrón `check-npm-versions` (TARGET nuevo, sin duplicar lógica). |
| C | Formula 0.5.0→0.8.0 | `version "0.8.0"` + los 4 sha256 REALES de los sidecars v0.8.0 (verificados HEAD 200 por el orquestador; re-leídos por lectura en esta tarea) + fix refs `release-binaries-63.yml`→`release-binaries.yml` (el `-63` no existe: el archivo real es `release-binaries.yml`). |
| D | `release-manifest.json` | Nuevo, en repo root. Mapea canal→nombre→versión y GENERA la lista esperada; `verify-release.ps1` lo consume (extender a adapters/providers/node donde aplique, resto documentado). El manifest es la única fuente (no dos listas espejadas). |
| E | Tarball de crates.io construye | Check con red (`cargo install --locked` contra la versión publicada) dentro de `release-verify.yml` o `verify-release` con red; si la red falla → `NO-VERIFICADO` declarado, nunca inventado. |
| F | Fan-out `workflow_call` desde `release.yml` | Hacia wheels/npm/sbom/binaries para eliminar la dependencia de `RELEASE_PLZ_TOKEN` (investigar `PUBLISH.md` §cascada primero). NO tocar filenames `release*.yml` (OIDC), NO cambiar namespaces de tags. La prueba real solo ocurre en el próximo release → declararlo + verificación mecánica máxima sin publicar. |

## Impacto mapeado (Regla 0)

Archivos leídos completos: `server.json`, `Formula/vantadb.rb`,
`scripts/docs/check-npm-versions.mjs` (completo), `scripts/verify-release.ps1`
(completo), `.github/workflows/release.yml` (completo),
`.github/workflows/release-verify.yml` (completo), `docs/dev/workflow/PUBLISH.md`
(completo), `docs/dev/workflow/RULES.md` (completo), `docs/dev/workflow/TRIGGERS.md`,
`.opencode/rules/release-ci.md` (§1-5), `docs/user/operations/MCP_REGISTRY.md`
(sección `server.json`), `release-plz.toml` (completo). Encabezados `on:` + `if:`
de los 7 workflows downstream (vía shell, lectura parcial dirigida).

Referencias hacia dentro (quién lee lo que toco):

- `server.json` ← `docs/api/MCP.md:671,681-685` (schema + procedimiento), `docs/user/operations/MCP_REGISTRY.md` (contrato: `version` synced con workspace).
- `scripts/docs/check-npm-versions.mjs` ← `docs-gates.yml:130-147` (job `check-npm-versions`: `--self-test` + gate + `--json`), `release-npm-61.yml` + `release-npm-node.yml` (mensajes de skip apuntan al script), `PUBLISH.md` §FIND-230, `docs/api/SURFACE_MANIFEST.md:46`.
- `Formula/vantadb.rb` ← `scripts/update-homebrew-formula.sh` (no lo referencia por nombre de workflow; solo el comentario interno), `scripts/install.ps1/install.sh` (no tocan la formula).
- `scripts/verify-release.ps1` ← `release-verify.yml:33-42` (único caller: `./scripts/verify-release.ps1 @scriptArgs` desde repo root; env `GH_TOKEN`, `TARGET_TAG`).
- `release.yml` ← nada lo llama (entry point `push: [main]`); `RELEASE_PLZ_TOKEN` con fallback `GITHUB_TOKEN` (`release.yml:42,71`).
- `release-verify.yml` ← `schedule` + `workflow_dispatch` (nadie lo llama).

Referencias entrantes a nombres que renombro: `release-binaries-63` vivo SOLO en
`Formula/vantadb.rb:7,43` (verificado con `Get-ChildItem -Recurse` en `Formula/`,
`scripts/`, `.github/workflows/`, `docs/dev/workflow/`). El resto de menciones
(`docs/index.md`, `llms.txt`, `docs/CHANGELOG.md`, tasks históricos) son
generados o históricos: NO se tocan.

Blast radius por slice (release path = crítico):

- B: `server.json` (1 archivo, descriptor público) + `check-npm-versions.mjs`
  (gate de CI: `docs-gates.yml`). Riesgo: si el TARGET nuevo falla, bloquea CI —
  mitigado con `--self-test` + corrida local del gate.
- C: `Formula/vantadb.rb` (1 archivo, solo afecta `brew install`). Riesgo ~nulo.
- D: `release-manifest.json` (nuevo) + `verify-release.ps1` (consume). Riesgo:
  `release-verify.yml` corre el script en runner — mitigado con parse local
  (`[Parser]`/pwsh `-WhatIf`/lectura) + `Test-Json` del manifest.
- E: `release-verify.yml` (añade job, no toca el existente). Riesgo: red externa
  en CI — mitigado con probe que declara `NO-VERIFICADO` sin fallar por red.
- F: `release.yml` + triggers de 4 workflows. Riesgo: **cambiar condiciones de
  publish** — ver slice F §Gate D.

Veredicto de impacto: B/C/D/E son aditivos o de datos, reversibles por `git checkout`.
F requiere diseño (ver abajo). Ningún slice toca `src/`, bindings, versiones de
`Cargo.toml` ni CHANGELOG.

## Problema (verde-vacío por slice)

- F: `release-verify.yml` verde-vacío (verifica assets/registries pero nunca que el tarball compile).
- D: canales sin gate (listas espejadas script↔workflows, divergen en silencio).
- E: tarball sin build (crates.io publica, nadie compila lo publicado).
- B/C: registry/formula stale (`server.json` 0.6.1, Formula 0.5.0 vs workspace 0.8.0).

## Steps

- [x] B1 — `server.json`: `version` → `0.8.0`, `timestamp` → fecha real, resto intacto
- [x] B2 — `check-npm-versions.mjs`: TARGET `server.json` (policy `workspace`) + `why` opcional + 1 assertion `--self-test`; gate local verde
- [x] C1 — `Formula/vantadb.rb`: `version 0.8.0` + 4 sha256 reales + 2 refs `-63`→nombre real + comentario
- [x] D1 — `release-manifest.json` nuevo (canal→nombre→versión; adapters/providers/node documentados)
- [x] D2 — `verify-release.ps1` consume el manifest (única fuente; SKIP documentados)
- [x] E1 — `release-verify.yml`: job `tarball-build` con probe de red (`NO-VERIFICADO` honesto)
- [x] F1 — fan-out dispatch implementado (release.yml: job `fanout` + outputs; PUBLISH.md §cascada) — review+commit los hace el orquestador
- [x] VAL — yaml parse + `node` gate + pwsh parse + `git diff --check`

## Slice F — hallazgo de DISCOVERY (Gate D)

Investigada `PUBLISH.md` §cascada primero, como pide el contrato. La cascada con
`RELEASE_PLZ_TOKEN` existe porque release-plz taggea con `GITHUB_TOKEN` y GitHub
suprime los eventos derivados (tags `v*` y `release: published` no disparan
wheels/npm/sbom/binaries).

Bloqueador para `workflow_call` puro (verificado en repo, sin red):

- `release-wheels.yml:175` — `publish-pypi`: `if: startsWith(github.ref, 'refs/tags/v')`.
  Vía `workflow_call` el ref es `refs/heads/main` → el job se saltearía. Hacer que
  publique exigiría reescribir ese `if:` = **cambiar publish path → Gate D dispara → STOP**.
- `release-npm-61.yml:171,257` — `publish-wasm` / `publish-ts` con `if:` multilínea
  (pendiente de lectura exacta, misma sospecha tag-gated).
- `release-binaries.yml:146` — upload gatea por `github.event_name == 'release'`
  `|| inputs.release_tag != ''`: SÍ admite fan-out sin tocar lógica (espejar el input
  en un bloque `workflow_call`, triggers-only).
- `release-sbom.yml` — read-only, sin gate de publish: admite `workflow_call` sin riesgo.

Alternativa que respeta Gate D (NO implementada — propuesta al owner): en vez de
`workflow_call`, un job `fanout` en `release.yml` que dispare
`gh workflow run <downstream> --ref <tag>` (dispatch vía API, no suprimido).
Con `--ref vX.Y.Z`: wheels ve `refs/tags/v*` y publica sin tocar ningún `if:`;
binaries recibe `-f release_tag=`; npm/sbom igual. Solo se toca `release.yml`
(+ permiso `actions: write` en ese job) y `PUBLISH.md`. Requiere: leer los `if:`
de npm-61 exactos + outputs de `release-plz/action@v0.5.139` (¿expone tag?) +
aprobar elevación de permisos. Prueba real: próximo release (declarado).

## Slice F — validación DISCOVERY→ACT (Gate D owner-APROBADO 2026-10-16, implementar)

Verificado leyendo (cero cambios en downstreams — Gate D intacto):

- `release-npm-61.yml:171-173` — `publish-wasm`: `startsWith(github.ref,'refs/tags/v') || (dispatch && package!='ts')`. Dispatch `--ref <tag>` ⇒ primera cláusula TRUE ⇒ publica. Requiere `-f package=both` (`package` es required).
- `release-npm-61.yml:257-261` — `publish-ts`: `always() && wasm!='failure' && (tags/v || dispatch && package!='wasm')`. Dispatch `--ref <tag>` ⇒ TRUE ⇒ publica.
- `release-wheels.yml:175` — `publish-pypi`: `startsWith(github.ref,'refs/tags/v')`. Dispatch `--ref <tag>` ⇒ TRUE ⇒ publica. Sin inputs en `workflow_dispatch`.
- `release-binaries.yml:146` — upload: `event=='release' || inputs.release_tag!=''`. Dispatch con `-f release_tag=<tag>` ⇒ TRUE ⇒ sube. `release_tag` ya existe como input.
- `release-npm-node.yml:141-143` — `publish`: `startsWith(ref,'refs/tags/node-v') || event=='workflow_dispatch'`. Dispatch ⇒ segunda cláusula TRUE ⇒ publica. (Step `Attach to Release:221` salta bajo dispatch — step-level, sin cambios.)
- `release-sbom.yml:3-6` — `push tags v*` + `workflow_dispatch` sin inputs ni gates: read-only, dispatch directo.
- `release-plz/action` outputs (action.yml oficial): `releases` (JSON de `release -o json`) + `releases_created`. Tag derivado con `jq '.[0].tag_name // .[0].tag // empty'` + fail-closed si vacío.

Implementado (solo `release.yml` + `PUBLISH.md` §cascada):

- `release-plz-release`: `outputs.releases`/`releases_created` + `id: release-plz` en el step (aditivo, OIDC/PAT intactos).
- Job `fanout` (`needs: release-plz-release`, `if: releases_created=='true'`, `actions: write`, `timeout-minutes: 10`): guard PAT (sin PAT falla en rojo — el dispatch con `GITHUB_TOKEN` no dispara downstreams por supresión de eventos derivados) + 5 `gh workflow run --ref $TAG` (inputs requeridos incluidos).
- `PUBLISH.md`: §cascada ahora documenta fan-out como primario y PAT como fallback. PAT conservado en el workflow (no borrado).
- Supuesto declarado NO verificado contra docs oficiales (fetch 404, sin reintento por budget): que `GITHUB_TOKEN` no dispara `workflow_dispatch` vía API. El diseño es robusto en ambos casos (con PAT funciona siempre; sin PAT falla en rojo, nunca verde-vacío). Verificar en el próximo release.

## Save Point (reanudación sin pérdida)

- B/C/D/E: trabajo en worktree sin commitear (el orquestador hace review+commit).
  Verificación por slice en §Validación del RESULTADO.
- F: IMPLEMENTADO (archivos: `release.yml` job fanout + outputs, `PUBLISH.md`
  §cascada). Incidente en el camino: un edit con oldString amplio borró los
  steps de `release-plz-release` — reparado con `git checkout -- release.yml`
  (estaba limpio antes) y reaplicado en 3 edits mínimos secuenciales.
  Validación pendiente (orquestador o este worker en verify): yaml parse +
  `git diff --check` + confirmar que solo `release.yml` cambió bajo
  `.github/workflows/`. Prueba real imposible sin release (declarado).
- `PROXIMO_STEP="review + commit"` (orquestador).

## Decisions log (owner 2026-10-16, resumidas del prompt)

- B: timestamp/build_info honestos si el schema lo pide; `submission_state` = `pending`.
- C: los 4 sha256 son los sidecars v0.8.0 (URLs `releases/download/v0.8.0/vantadb-<plat>.tar.gz.sha256).
- D: manifest = única fuente; adapters/providers/node se extienden donde aplique.
- E: sin red = `NO-VERIFICADO` declarado.
- F: no tocar filenames (OIDC) ni namespaces; sin publicar nada.

## Review (Gate P2-01)
- **Revisor:** vanta-review, sesión distinta al implementador (veredicto 2026-10-16, contratos re-ejecutados).
- **Veredicto:** ✅ APPROVE FIND-342 (B-E; F pendiente decisión) — shas = sidecars oficiales por red, manifest fail-closed sin fallback, tarball probe honesto, Gate D en F respetado (0 archivos fan-out). Sin Required/Optional.
- **Veredicto F1:** ✅ APPROVE F1 (2026-10-16, re-ejecución: originals intactos, downstreams sin cambios, diseño robusto con/sin PAT). 2 Optional + 2 nits como follow-up (tabla invariante, `.[0].tag` primario, DIST-01 multi-release).
- **Estado:** ✅ COMPLETED (B-E + F1 fan-out dispatch aprobado por owner).
