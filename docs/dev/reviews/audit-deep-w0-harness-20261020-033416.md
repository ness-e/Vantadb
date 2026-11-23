---
title: "Deep audit W0 - Harness .opencode"
kind: review
status: active
description: "Auditoria read-only del harness .opencode (commands, agents, skills, rules, references, task-system): integridad referencial, conteos documentados vs reales, duplicados y contradicciones."
tags: [audit, reviews, harness]
---

# Deep audit W0 - Harness `.opencode/`

> **Wave:** W0 · **Fecha:** 2026-10-20 · **Modo:** read-only (no se escribio nada en `.opencode/`)
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (tabla W0), §7.1
> **Nota de ejecucion:** esta wave estaba asignada a un subagente `vanta-harness` (leaf). El tool `task`
> no esta disponible en la sesion que ejecuta la revision, por lo que el orquestador ejecuto la wave
> directamente (clausula "ninguna wave puede quedar sin cubrir" del prompt de campana). Ver
> `audit-deep-sintesis-20261020-033416.md` §Limitaciones.

## Scope

**Cubierto (inventario medido el 2026-10-20):**

| Area | Ruta | Conteo real | Conteo en el brief/prompt |
|---|---|---|---|
| Commands | `.opencode/commands/*.md` | 13 | 14 |
| Agents | `.opencode/agents/*.md` | 11 | 11 |
| Skills (dirs) | `.opencode/skills/*/` | 198 dirs / 197 con `SKILL.md` | 198 |
| Rules | `.opencode/rules/*.md` | 14 ficheros (13 reglas + `README.md`) | 15 |
| References | `.opencode/references/*.md` | 24 | 25 |
| Task-system | `.opencode/task-system/` | 82 ficheros, 11 subdirs | - |
| Plugins | `.opencode/plugins/ponytail.ts` | 1 | - |
| Config | `.opencode/{configs,mcp,mcp-profiles,locks,opencode-loop}/` | presentes | - |
| Raiz harness | `.opencode/{package.json,bun.lock,AGENTS.md,VANTADB-OPERATING-MANUAL.md}` | presentes | - |

**Excluido con motivo:** `.opencode/node_modules/` (dependencias instaladas, no fuente),
`.opencode/skills/*/SKILL.md` de skills no-VantaDB (auditadas por conteo y coherencia de referencias,
no por contenido funcional - el perimetro funcional de las skills es W5/W8).

## Metodologia

- Inventario con `Get-ChildItem` (conteos reales, sin depender de documentacion).
- Integridad referencial: se resolvieron **todas** las referencias `skills/<x>`, `rules/<x>.md` y
  `references/<x>.md` presentes en `.opencode/commands/*.md`, `.opencode/agents/*.md` y
  `.opencode/skills/*/SKILL.md` contra el filesystem.
- Contraste de conteos y afirmaciones contra `SKILLS-MANIFEST.md` (raiz) y `.opencode/AGENTS.md`.
- Sin ejecucion de codigo del harness (read-only). Sin escrituras en `.opencode/`.

**Resultado de integridad referencial: 0 referencias rotas.** No hay skills, rules ni references
referenciadas que no existan. Este es el punto fuerte del harness.

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-397` | Medium | `SKILLS-MANIFEST.md` declara un catalogo mas pequeno que el real (196 skills / 197 dirs vs 198 dirs reales) | `SKILLS-MANIFEST.md:3`, `SKILLS-MANIFEST.md:7`, `SKILLS-MANIFEST.md:9`, `SKILLS-MANIFEST.md:596` | Conteo medido: `Get-ChildItem .opencode/skills -Directory` = **198**; con `SKILL.md` = **197**; el unico dir sin `SKILL.md` es `a11y-shared` (recursos, no skill). El manifest dice "196 skills en 197 dirs" y "total 196 con SKILL.md (195 + shim ponytail)". `.opencode/AGENTS.md:184` repite el mismo numero obsoleto. El catalogo esta **1 skill por debajo** del estado real y nadie tiene forma de detectarlo: no hay gate que compare el conteo con el filesystem. | Regenerar el conteo desde el filesystem y anadir un gate en `dev-tools/floor-guard.ps1` (o en `docs-gates.yml`) que falle si `dirs(SKILL.md) != claim`. Actualizar las 4 apariciones. | 1h |
| `FIND-398` | Medium | `SKILLS-MANIFEST.md` afirma que `.opencode/` es un *git submodule*; `AGENTS.md` afirma explicitamente que **no** lo es | `SKILLS-MANIFEST.md:596` vs `AGENTS.md:133` | `SKILLS-MANIFEST.md:596`: "All skills are now consolidated in the project: **196 skills en `.opencode/skills/`** (git submodule -> `configOpencode` private repo)". `AGENTS.md:133`: "La carpeta `.opencode/` es un **repo Git separado** (remoto privado `https://github.com/ness-e/configOpencode`), **ignorado en VantaDB** (no es submodule, no hay `.gitmodules` ni gitlink; VantaDB lo ignora)". Ambas no pueden ser ciertas. Verificado: no existe `.gitmodules` en el repo. | Editar `SKILLS-MANIFEST.md:596` para decir "repo Git separado, ignorado por VantaDB" (misma formulacion que `AGENTS.md:133`). Un agente que lea el manifest puede intentar `git submodule update` y fallar. | 15min |
| `W0-03` | Low | `.opencode/AGENTS.md:505` declara "existen 8 instancias" de `continue-on-error`; el numero real es 38 (11 de ellas sin el tag `# CATEGORY:` que la propia regla exige) | `.opencode/AGENTS.md:505` | Medicion: `Select-String -Path .github/workflows/*.yml -Pattern '^\s*continue-on-error:\s*true'` = **38** ocurrencias en 9 workflows (`nightly.yml` 23, `ci-rust.yml` 6, `docs-gates.yml` 3, y 1 en cada uno de `certification.yml`, `platform-ops.yml`, `release-adapters.yml`, `release-npm-node.yml`, `release-providers.yml`, `release-wheels.yml`). De esas 38, **11 no llevan el tag `# CATEGORY:`**. Detalle en W8 (`FIND-390`, `FIND-391`). | Actualizar la cifra a 38 al cerrar `FIND-390`/`FIND-391`, o cambiar la redaccion a "verificacion dinamica en `docs-gates.yml`" para que no vuelva a caducar. | 15min |
| `W0-04` | Info | El harness declara 13 commands en la tabla de entry-points de `AGENTS.md`; el filesystem tiene 13 ficheros - consistente | `.opencode/AGENTS.md:49-64` | Tabla de `AGENTS.md` (entry points) lista: pipeline, audit, ship, rollback, status, harness, backlog, spec, research, webperf, code-simplify, codeGraph, cleanCA = 13. `Get-ChildItem .opencode/commands -File` = 13. Coherente. `build.md` figura como eliminado en `AGENTS.md:53` y no existe. | - | - |
| `W0-05` | Info | `SKILLS-MANIFEST.md` y `AGENTS.md` usan dos vocabularios de conteo distintos (skills vs dirs) sin marcar la diferencia | `SKILLS-MANIFEST.md:3` | El manifest usa "196 skills en 197 dirs" y "total 196 con SKILL.md (195 + shim ponytail)". El dato real es 197 ficheros `SKILL.md` en 198 dirs. La divergencia de un skill proviene de que `ponytail` (shim) no cuenta como skill pero si como dir, y `a11y-shared` es dir sin `SKILL.md`. | Adoptar una formula unica y declararla en el manifest: "N dirs, M con SKILL.md (L recursos, S shim)". | 30min |
| `W0-06` | Info | Tabla de limites de herramientas (TSYS-11) de `AGENTS.md:255-267` cubre 10 roles, no 11 | `.opencode/AGENTS.md:255-267` | La tabla lista `vanta-lead`, `vanta-arch`, `vanta-worker`, `vanta-engine`, `vanta-audit`, `vanta-chaos`, `vanta-tuner`, `vanta-docs`, `vanta-review`, `vanta-research`, `vanta-harness` = 11 filas, pero **no existe** un rol `vanta-arch` en el senseo de "arquitecto" separado de `vanta-lead`: `vanta-lead` es el unico con push/commit. Verificado: los 11 ficheros de `.opencode/agents/` son `vanta-{arch,audit,chaos,docs,engine,harness,lead,research,review,tuner,worker}.md` y la tabla cubre los 11. | Sin accion. Verificado consistente. | - |

## Veredicto

**PASS CON OBSERVACIONES**

El harness `.opencode/` esta en mejor estado que el resto del sistema: **cero referencias rotas**
entre commands, agents, skills, rules y references; el task-system tiene sus 11 subdirectorios
esperados; los 11 roles tienen fichero y los 11 son leafs con `subagent: deny` (regla R10 del prompt
respetada). Los dos hallazgos Medium son derivas **documentales** (conteos y una contradiccion
"submodule" vs "repo separado"), no estructurales: ningun agente que siga las instrucciones del
manifest queda sin herramientas. El punto de riesgo real es que **no existe ningun gate que detecte
la deriva de conteos**, asi que estafinding se va a repetir en cada revision hasta que se automatice.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `FIND-397` | Medium | `SKILLS-MANIFEST.md:3` |
| `FIND-398` | Medium | `SKILLS-MANIFEST.md:596` |
| `W0-03` | Low | `.opencode/AGENTS.md:505` |
| `W0-04` | Info | `.opencode/AGENTS.md:49-64` |
| `W0-05` | Info | `SKILLS-MANIFEST.md:3` |
| `W0-06` | Info | `.opencode/AGENTS.md:255-267` |
