---
title: "Plan de EjecuciÃ³n: Pre-mercado VantaDB â€” orden, estabilizaciÃ³n y salida"
kind: plan
status: completed
description: "CampaÃ±a pre-mercado: orden, estabilizaciÃ³n y salida (12 tasks DO)"
tags: [pre-mercado, release]
---

# Plan de EjecuciÃ³n: Pre-mercado VantaDB â€” orden, estabilizaciÃ³n y salida

> **Inicio:** 2026-10-15
> **Estado:** âœ… COMPLETADA (12/12 DO + extras FIND-337/338/339/341/342/344 + CI-FIXES; 2026-10-16)
> **Fuente:** `docs/dev/Backlog.md` (DELTA + EGO + SURF) + `docs/dev/Backlog-negocio.md` + auditorÃ­a `auditor_a_pre_mercado_y_plan_de_salida_vantadb.md` (verificada contra disco 2026-10-15)
> **Autonomous:** false
> **Decisiones owner locked (2026-10-15):** SPEC/CONSTRAINTSâ†’`docs/specs/` nueva Â· CLAâ†’`.github/cla/` Â· borrar bak+`.md` âœ… Â· `.editorconfig` nuevo âœ… Â· hooksâ†’`pre-commit` Ãºnico Â· superficie/docs âœ… Â· DX âœ… Â· PROC-01 solo-preparar Â· tren EGO completo Â· carril negocio-humano separado
> **Normas harness (revisiÃ³n `.opencode`, todo el plan en espaÃ±ol):**
> - **TaxonomÃ­a:** solo IDs registrados (`FIND-*`, `EGO-*`, `SURF-*`, `DOC-*`, `API-*`, `DX-*`, `DIST-*`, `MEMG-*`, `SRV-*`, `BIZ-*`). Los `HYG-*/COG-*/REL-PREP` del borrador se renombran abajo a `FIND-336..340` y a IDs backlog existentes. Sin prefijos nuevos.
> - **Skills obligatorias:** `documentation-skill` antes de tocar `docs/**`; `codebase-memory` antes de `codegraph_*`; `progreso` al abrir/cerrar; `ponytail` siempre.
> - **Regla 0:** antes de mover/borrar, leer archivo + `rg` entrante/saliente + veredicto. Incluida por tarea.
> - **Commits:** solo `vanta-lead`, un commit local por tarea, Conventional Commits, **push prohibido sin OK explÃ­cito del owner (Regla 7)**. Sin tocar versiones/CHANGELOG/tags (release-plz).
> - **Cierre cÃ³digo:** norma Clean + `/cleanCA` del alcance + `ponytail-review` + `just verify` (o `verify_changed.ps1`) + `ocr-review.ps1` (Critical/High bloquean).
> - **Idioma:** el plan va Ã­ntegramente en espaÃ±ol; el contenido tÃ©cnico nuevo en `docs/api/`, `docs/dev/architecture/`, `QUICKSTART.md` se redacta en inglÃ©s (fuente de verdad).

## Resumen

| Resultado | Count |
|-----------|-------|
| âœ… DO | 12 |
| ðŸŸ¡ DEFER | 6 |
| âŒ SKIP | 2 |
| ðŸ”´ BLOQUEADO | 2 |

Status: â¬†ï¸ uphill = 0 Â· â¬‡ï¸ downhill = 0 (12/12 DO âœ… + extras âœ…; DEFER/SKIP/BLOQUEADO segÃºn triage, fuera del tren)

## Retrospectiva de cierre (Start/Stop/Continue + 1 acciÃ³n medida)
- **Start (seguir haciendo):** review P2-01 con re-ejecuciÃ³n por agente distinto â€” cazÃ³ defectos reales que los gates mecÃ¡nicos no ven (ifs truncados, pin 37-hex, Deserialize wire). Staged-set revisado a mano antes de cada commit en paralelo (lecciÃ³n del incidente `174410d7`).
- **Stop (dejar de hacer):** `git add -A` con workers en paralelo (colisiÃ³n de Ã­ndice); confiar en `Select-String -Match` (no existe en pwsh 5.1, usar `-Pattern`); builds en background (mueren con la sesiÃ³n â€” foreground con timeout largo).
- **Continue:** task files como estado durable + Save Points precisos (permitieron 5 resurrecciones de sesiones sin pÃ©rdida) + contratos mecÃ¡nicos por task.
- **AcciÃ³n medida:** `cargo`/`rustup`/`pre-commit`/`ruff` vÃ­a PATH completo o `python -m` en cada comando (baseline: 4 bloqueos ambientales por binario ausente en subshells â†’ meta 0).

DEFER: `JS-01`, `PY-04`, `DIST-18`, `DIST-01`, `CORE-03-ejecuciÃ³n`, `FIND-324` (post-1.0, Freeze List).
SKIP: `docs/dev/references/.editorconfig` (no existe â€” crear nuevo), `docs/dev/plans/2026-10-04-*` (cita invÃ¡lida del reporte).
BLOQUEADO: `PROC-01-push` (requiere OK explÃ­cito owner, Regla 7), carril negocio `BIZ-*` (humano, fuera del pipeline).

## Tasks

### Task 1: FIND-336 â€” Mover SPEC/CONSTRAINTS a docs/specs/

- **Appetite:** max 1h
- **Esfuerzo:** ðŸŸ¢
- **Prioridad:** ðŸŸ¡
- **Skills:** `documentation-skill`, `progreso`, `ponytail`
- **Archivos clave:** `SPEC.md`, `CONSTRAINTS.md`, `docs/specs/`, `AGENTS.md`, `docs/index.md`
- **Regla 0:** leer ambos archivos + `rg -n "SPEC\.md|CONSTRAINTS\.md" --glob "!target/**" --glob "!.opencode/**"` (entrantes) + listar sus links salientes; presentar veredicto antes del `git mv`.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `SPEC.md` + `CONSTRAINTS.md` existen en raÃ­z (listado 2026-10-15); `docs/specs/` NO existe â†’ crear. PrÃ¡ctica spec-kit/SDD: specs fuera de raÃ­z.
- **Gate JustificaciÃ³n:** root clutter real (30+ archivos en raÃ­z); decisiÃ³n owner `docs/specs/` nueva.
- **Gate Result:** âœ… DO
- **Contrato:** `git status --short` limpio tras moves + `rg` solo apunta a `docs/specs/` + `node scripts/docs/check-links.mjs` exit 0
- **Idioma:** plan y mensajes en espaÃ±ol; si un `.md` movido exige frontmatter `title`+`kind`, respetar `documentation-skill`.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead` (`docs: ...`), sin push.
- **Pre-mortem:** 1. links rotos en README/AGENTS que apuntaban a raÃ­z. 2. gate-docs-links falla por rutas viejas. 3. duplicado si alguien recrea los archivos en raÃ­z.
- **Stop conditions:** links rotos >3 â†’ abortar y completar mapa Regla 0 antes del commit.
- **Cynefin:** ðŸŸ¦ obvio â€” move mecÃ¡nico con grep de refs.
- **Top 3 riesgos:** 1. refs entrantes no mapeadas. 2. Ã­ndice docs desactualizado. 3. convenciÃ³n `kind` de frontmatter si aplica.
- **Uphill/Downhill:** â¬†ï¸ 0 / â¬‡ï¸ 1 (move + fix links + verify)
- **DoD task:** contrato âœ… Â· task file sync Â· recitation actualizada
- **Task file:** `docs/dev/tasks/FIND-336.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸŸ¡ | links a `./SPEC.md` quedan rotos | `rg` previo + fix en mismo commit | verify `check-links.mjs` rojo |
  | ðŸŸ¢Ã—ðŸŸ¡ | Ã­ndice docs no lista `docs/specs/` | `gen-index.mjs --write` | diff Ã­ndice vacÃ­o |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-4 + gates + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (ses_ebecfacb) | check-links 0 broken, check-docs sin nuevos missing, fmt exit 0, floor-guard exit 0 |

  **Notas:** DecisiÃ³n owner 2026-10-15: `docs/specs/` nueva (no `docs/dev/architecture/`). ID `FIND-336` libre (verificado 2026-10-15: `FIND-330..335` ocupados, `336+` libres).

### Task 2: FIND-337 â€” Mover CLA a .github/cla/ + fix links

- **Appetite:** max 1h
- **Esfuerzo:** ðŸŸ¢
- **Prioridad:** ðŸŸ¡
- **Skills:** `documentation-skill`, `progreso`, `ponytail`
- **Archivos clave:** `CLA_INDIVIDUAL.md`, `CLA_CORPORATE.md`, `.github/cla/`, `.github/clabot-config.json`, `CONTRIBUTING.md`
- **Regla 0:** leer ambos CLA (no alterar texto legal, solo mover) + `rg -n "CLA_" .github/ CONTRIBUTING.md README.md` + veredicto antes del move.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” CLAs en raÃ­z; `.github/clabot-config.json:2` ya apunta a `.github/CLA_*.md` (ruta inexistente hoy) â†’ mover a `.github/cla/` + fix config en mismo commit. PrÃ¡ctica: community-health en `.github/`.
- **Gate JustificaciÃ³n:** links clabot rotos hoy; decisiÃ³n owner `.github/cla/`.
- **Gate Result:** âœ… DO
- **Contrato:** `Test-Path .github/cla/CLA_INDIVIDUAL.md` + `rg -n "CLA_" .github/clabot-config.json CONTRIBUTING.md` consistente + `check-links.mjs` exit 0
- **Idioma:** todo en espaÃ±ol en el plan; no traducir el texto legal de los CLA.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. CONTRIBUTING referencia ruta vieja. 2. bot no re-lee config hasta re-run. 3. firma pendiente de PRs abiertos.
- **Stop conditions:** >2 archivos con refs viejas tras grep â†’ abortar y completar mapa Regla 0.
- **Cynefin:** ðŸŸ¦ obvio.
- **Top 3 riesgos:** 1. refs rotas. 2. mensaje bot desactualizado. 3. alteraciÃ³n accidental del texto legal.
- **Uphill/Downhill:** â¬†ï¸ 0 / â¬‡ï¸ 1
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/FIND-337.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸŸ¡ | texto legal alterado por accidente | `git diff --stat` solo renames + config | diff muestra cambios de contenido |
  | ðŸŸ¢Ã—ðŸŸ¢ | bot no refleja cambio | nota en PR, no bloquea | â€” |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-3 + gates + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (R100 + fc /b idÃ©nticos) | check-links exit 0, diff --check limpio |

  **Notas:** ID `FIND-337` libre verificado.

### Task 3: FIND-338 â€” Borrar bak + templates .md duplicados

- **Appetite:** max 1h
- **Esfuerzo:** ðŸŸ¢
- **Prioridad:** ðŸŸ¡
- **Skills:** `progreso`, `ponytail`
- **Archivos clave:** `opencode.jsonc.bak-cleanup-20260928`, `.github/ISSUE_TEMPLATE/bug_report.md`, `.github/ISSUE_TEMPLATE/feature_request.md`, `.github/ISSUE_TEMPLATE/config.yml`
- **Regla 0:** `rg -n "bug_report\.md|feature_request\.md|bak-cleanup" .github/ docs/ .opencode/` + veredicto antes del `git rm`.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” bak existe en raÃ­z; `ISSUE_TEMPLATE/` tiene `.md`+`.yml` duplicados (verificado 2026-10-15). GitHub prioriza `.yml`.
- **Gate JustificaciÃ³n:** duplicados muestran opciones dobles al usuario; bak es huÃ©rfano.
- **Gate Result:** âœ… DO
- **Contrato:** `Get-ChildItem .github/ISSUE_TEMPLATE/*.md` vacÃ­o + `git status --short` solo deletes esperados
- **Idioma:** todo en espaÃ±ol.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. algÃºn workflow referencia al `.md` por nombre. 2. `config.yml` lista templates. 3. borrado sin backup.
- **Stop conditions:** grep muestra referencia al `.md` â†’ abortar y actualizar referencia primero.
- **Cynefin:** ðŸŸ¦ obvio.
- **Top 3 riesgos:** 1. referencia por nombre. 2. choques con PR abierto. 3. ninguno mÃ¡s.
- **Uphill/Downhill:** â¬†ï¸ 0 / â¬‡ï¸ 1
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/FIND-338.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¢Ã—ðŸŸ¡ | `config.yml` lista `.md` | actualizar en mismo commit | grep pre-delete |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-2 + gates + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (3 deletes puros, 0 adiciones) | check-links exit 0 |

  **Notas:** Aprobado explÃ­cito owner 2026-10-15. ID `FIND-338` libre verificado.

### Task 4: FIND-339 â€” Crear .editorconfig + unificar hooks en pre-commit

- **Appetite:** max 1d
- **Esfuerzo:** ðŸŸ¡
- **Prioridad:** ðŸŸ¡
- **Skills:** `progreso`, `ponytail`
- **Archivos clave:** `.editorconfig`, `.pre-commit-config.yaml`, `.githooks/pre-commit`, `.githooks/pre-push`, `CONTRIBUTING.md`
- **Regla 0:** leer `.githooks/pre-commit:1-87` + `.pre-commit-config.yaml:1-44` + `rg -n "core.hooksPath|pre-commit" CONTRIBUTING.md docs/dev/workflow/ .github/workflows/` + veredicto antes de editar.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `.editorconfig` ausente en raÃ­z; `docs/dev/references/.editorconfig` NO existe (solo 4 `.md`) â†’ crear nuevo. `.githooks/pre-commit` solo fmt/clippy/actionlint; `.pre-commit-config.yaml` Ãºnico con ruff/prettier/markdownlint. DecisiÃ³n owner: `pre-commit` sistema Ãºnico, `.githooks` â†’ shim.
- **Gate JustificaciÃ³n:** sin `.editorconfig` no hay norma LF/charset cross-IDE; borrar pre-commit (propuesta reporte) perderÃ­a gates Python/JS/docs.
- **Gate Result:** âœ… DO
- **Contrato:** `Test-Path ./.editorconfig` + `pre-commit run --all-files` exit 0 + `git config core.hooksPath` documentado en `CONTRIBUTING.md`
- **Idioma:** todo en espaÃ±ol.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. hook cargo pesado en Windows sin sccache. 2. pre-commit autoupdate cambia revs. 3. shim `.githooks` con CRLF rompe `sh`.
- **Stop conditions:** `pre-commit run` >10min dos veces â†’ DEFER con log (rabbit hole).
- **Cynefin:** ðŸŸ¨ complicado â€” dos sistemas con solape parcial, requiere experto para el shim.
- **Top 3 riesgos:** 1. divergencia hooks vs CI gates. 2. instalaciÃ³n `pre-commit` ausente en devs. 3. EOL `*.ps1` (fijar `crlf` en `.editorconfig`).
- **Uphill/Downhill:** â¬†ï¸ 1 (shim Windows) / â¬‡ï¸ 2
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/FIND-339.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸŸ¡ | hooks locales divergen de CI | espejar `gate-docs`/`ci-rust` en config | diff configs |
  | ðŸŸ¡Ã—ðŸŸ¢ | devs sin pre-commit instalado | shim con fallback `cargo fmt --check` | test sin pre-commit |
  | ðŸŸ¢Ã—ðŸŸ¡ | EOL `*.ps1` | regla explÃ­cita en `.editorconfig` | CI Windows rojo |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-3 + gates + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (shim LF+sh-syntax, yaml vÃ¡lido, hooks propios verdes) | pre-commit run, sh -n, yaml parse |

  **Notas:** ID `FIND-339` libre verificado.

### Task 5: SURF-01 â€” Manifest de superficie + SSOT documental (DOC-22 + REL-03)

- **Appetite:** max 3d
- **Esfuerzo:** ðŸŸ¡
- **Prioridad:** ðŸ”´
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Archivos clave:** `docs/api/BINDINGS_NAMESPACES.md`, `docs/dev/workflow/PUBLISH.md`, `docs/index.md`, `tests/version_coherence.rs`, `scripts/docs/check-npm-versions.mjs`
- **Regla 0:** `codegraph_codegraph_explore "public-api version_coherence sdk_serialization"` + mapa de callers antes de declarar el manifest.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `PUBLISH.md` tabla DIST-06 existe; `version_coherence.rs` + `check-npm-versions.mjs` existen (`scripts/docs/` listado 2026-10-15); drift core 0.8.0 vs bindings 0.7.0 documentado en Backlog DELTA.
- **Gate JustificaciÃ³n:** sin manifest la contradicciÃ³n fila-vs-ADR-vs-cÃ³digo se reintroduce por cada binding (pre-req de API-10 y CI-08).
- **Gate Result:** âœ… DO
- **Contrato:** `node scripts/docs/check-npm-versions.mjs` exit 0 + `node scripts/docs/check-docs.mjs` exit 0 + surface manifest generado sin sÃ­mbolos fuera de `public-api.txt`
- **Idioma:** plan en espaÃ±ol; el manifest y el authority map que vivan en `docs/api/` o `docs/index.md` respetan `documentation-skill` (frontmatter, links relativos).
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. manifest manual que se desincroniza (debe ser generado). 2. authority map duplica contenido en vez de referenciar. 3. `FIND-321` (40 vs 28 workflows) contamina el inventario.
- **Stop conditions:** manifest no generable mecÃ¡nicamente â†’ scope a 1 dimensiÃ³n (versiones) y DEFER resto (precedente DOC-23).
- **Cynefin:** ðŸŸ¨ complicado â€” taxonomÃ­a multi-superficie.
- **Top 3 riesgos:** 1. drift reintroducido. 2. owners por superficie sin definir. 3. gate mecÃ¡nico ausente.
- **Uphill/Downhill:** â¬†ï¸ 1 / â¬‡ï¸ 2
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/SURF-01.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸ”´ | manifest manual stale en 1 mes | generado + test paridad | review sin gate |
  | ðŸŸ¡Ã—ðŸŸ¡ | contradicciÃ³n MEMG-05/FIND-303 | detectar contra manifest | fila sin dueÃ±o |
  | ðŸŸ¢Ã—ðŸŸ¡ | inventario workflows diverge | gate conteo (FIND-321) | â€” |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-2 (vanta-docs) + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (generador fail-closed, sin duplicaciÃ³n) | manifest --check, check-npm/links/docs, gen-index |

  **Notas:** Cubre `DOC-22` + `REL-03` como slices del mismo manifest (trazabilidad en task file).

### Task 6: API-10 â€” Schema-first slice 1 + pins (DIST-08)

- **Appetite:** max 3d
- **Esfuerzo:** ðŸ”´
- **Prioridad:** ðŸ”´
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Reglas Ã¡rea:** `.opencode/rules/api-contract.md` (SDK/API pÃºblica) + `.opencode/rules/query-dsl.md` si toca IQL.
- **Archivos clave:** `src/sdk/types/`, `tests/api/public-api.txt`, `tests/api/openapi_yaml_parity.rs`, `tests/version_coherence.rs`, `tests/sdk_serialization.rs`, `docs/user/QUICKSTART.md`
- **Regla 0:** `codegraph_codegraph_explore "Value sdk types public-api"` + blast radius bindings antes de tocar DTOs.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `public-api.txt`, `public_api.rs`, `openapi_yaml_parity.rs`, `version_coherence.rs`, `sdk_serialization.rs` existen; pins 0.7.0 vs live verificados en Backlog DIST-08.
- **Gate JustificaciÃ³n:** sin slice 1 (JSON Schema DTOs + paridad Rustâ†’TS/Python) cada binding reintroduce drift.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --test public_api --test version_coherence --test sdk_serialization --test openapi_yaml_parity` verde + `check-npm-versions.mjs` exit 0
- **Idioma:** plan en espaÃ±ol; schemas y docs `docs/api/` en inglÃ©s.
- **Cierre harness:** Clean + `/cleanCA` del alcance + `ponytail-review` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. codegen completo pre-1.0 (scope creep â€” solo slice 1). 2. breaking WASM-sync (ventana 0.x, decidir en DIST-19). 3. P99 varianza (falso positivo FIND-154).
- **Stop conditions:** slice 1 >appetite â†’ recortar a DTOs wire + 1 test paridad, resto post-1.0.
- **Cynefin:** ðŸŸ§ complejo â€” drift emerge por binding; probe-sense con 1 dimensiÃ³n primero.
- **Top 3 riesgos:** 1. scope creep a codegen. 2. breaking JS. 3. varianza runner.
- **Uphill/Downhill:** â¬†ï¸ 1 / â¬‡ï¸ 2
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/API-10.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸ”´Ã—ðŸŸ¡ | codegen total pre-1.0 | slice 1 solo | appetite excedido |
  | ðŸŸ¡Ã—ðŸ”´ | breaking consumidores WASM | spec en DIST-19 | test binding rojo |
  | ðŸŸ¡Ã—ðŸŸ¢ | perf gate falso positivo | re-run + baseline | FIND-154 |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps S1-S4 (vanta-worker) + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (drift TS real corregido, REDâ†’GREEN documentado) | nextest 36+1, check-npm, links, fmt |

  **Notas:** No usar `scripts/check_openapi_parity.mjs` (ruta invÃ¡lida del reporte) â€” lo real estÃ¡ en `scripts/docs/check-*.mjs`. Regla dura `-p`: iterar con `-p vantadb`, `--workspace` solo al cierre. Incidente de cierre: el commit `174410d7` barriÃ³ 5 deletes stageados por el worker CI-08 en curso (colisiÃ³n de Ã­ndice compartido; el `Select-String -Match` de verificaciÃ³n no existe en pwsh5.1 y el control saliÃ³ ciego) â€” remediaciÃ³n: no reescribir historia con el worker activo; el commit CI-08 siguiente aporta las migraciones y lo declara. Guardia futura: `git diff --cached --name-only` revisado a mano antes de cada commit en paralelo.

### Task 7: DX-13 â€” Doctor, completions y dead code (DX-02 + DEAD-01)

- **Appetite:** max 1d
- **Esfuerzo:** ðŸŸ¢
- **Prioridad:** ðŸŸ¡
- **Skills:** `codebase-memory`, `progreso`, `ponytail`
- **Archivos clave:** `src/cli_handlers/diagnostics.rs`, `completions/`, `src/cli.rs`, `Justfile`
- **Regla 0:** `rg -n "allow\(dead_code\)" src/ vanta-memory/src/` + `codegraph` de `doctor` antes de tocar.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `diagnostics.rs` existe (doctor dry-run+fix); `completions/` con drift (DX-13); `#[allow(dead_code)]` dispersos por grep.
- **Gate JustificaciÃ³n:** quick wins pre-anuncio; receta `surface-check` fija el gate.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo run -p vantadb --bin vanta-cli -- doctor` exit 0 + `cargo clippy -p vantadb --all-targets -- -D warnings` 0 warnings + receta `surface-check` en `Justfile`
- **Idioma:** todo en espaÃ±ol.
- **Cierre harness:** Clean + `/cleanCA` alcance + `ponytail-review` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. regenerar completions a mano (prohibido â€” desde binario). 2. `dead_code` con consumidores via feature flag. 3. doctor con fix destructivo (solo aditivo).
- **Stop conditions:** clippy con >20 warnings legacy â†’ acotar a mÃ³dulos tocados + FIND nueva deuda.
- **Cynefin:** ðŸŸ¦ obvio.
- **Top 3 riesgos:** 1. hand-edit completions. 2. flags sin consumidor real. 3. doctor destructivo.
- **Uphill/Downhill:** â¬†ï¸ 0 / â¬‡ï¸ 3
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/DX-13.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¢Ã—ðŸŸ¡ | completions hand-edit | regenerar desde binario | diff manual |
  | ðŸŸ¡Ã—ðŸŸ¢ | dead_code bajo feature | `rg` por feature | â€” |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-5 (vanta-worker) + fix :360 + review P2-01 | âœ… COMPLETED â€” APPROVE condicionado cumplido (bloqueo reescrito, commit selectivo) | doctor, clippy all-targets, surface-check 25 |

  **Notas:** Cubre `DX-02` (preflight doctor) + `DEAD-01` en el mismo barrido.

### Task 8: EGO-01 â€” Contrato crÃ­tico (EGO-05/09 + FIND-221)

- **Appetite:** max 3d
- **Esfuerzo:** ðŸŸ¡
- **Prioridad:** ðŸ”´
- **Skills:** `codebase-memory`, `progreso`, `ponytail`
- **Reglas Ã¡rea:** `.opencode/rules/api-contract.md` + `.opencode/rules/js-ecosystem.md` (Node) + `.opencode/rules/durability.md` (flock/WAL).
- **Archivos clave:** `vantadb-node/src/lib.rs`, `src/sdk/types.rs`, `src/sdk/api/memory.rs`, `tests/storage/multi_process_lock.rs`, `tests/sdk_serialization.rs`
- **Regla 0:** `codegraph` de `list`, `Value`, `put_batch`, `validate_namespace` + callers Node/WASM/Python antes de editar.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” rutas citadas existen en Backlog EGO-01/05/09 + FIND-221 (`put_batch` chunk 1000). Alcance P0 del tren 0.9.0.
- **Gate JustificaciÃ³n:** cursor Null + Value plano + flock bloquean a Ego hoy (workarounds en adapter); atomicidad `put_batch` ambigua.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --test sdk_serialization --test multi_process_lock` + `list({cursor: undefined})` y `cursor: "0"` no fallan + `put_batch` todo-o-nada documentado
- **Idioma:** todo en espaÃ±ol.
- **Cierre harness:** concurrencia (Regla 8: `vanta-chaos` + `vanta-review`, P2-01 contextos distintos) + Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. NAPI Null vs undefined (serializaciÃ³n). 2. cambio `Value` rompe wire WASM/Python. 3. `open_readonly` sin lock pierde garantÃ­as WAL.
- **Stop conditions:** breaking cross-binding sin paridad â†’ abortar y acotar a Node-only + FIND.
- **Cynefin:** ðŸŸ¨ complicado â€” FFI + tipos wire.
- **Top 3 riesgos:** 1. paridad bindings. 2. ABI NAPI. 3. semÃ¡ntica lock Windows (requiere test real, ver DIST-21).
- **Uphill/Downhill:** â¬†ï¸ 1 / â¬‡ï¸ 3
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/EGO-01.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸ”´ | breaking wire | Deserialize tolerante retrocompatible | test cross-binding rojo |
  | ðŸŸ¡Ã—ðŸŸ¡ | lock Windows | test en Windows real (DIST-21) | solo-CI-linux |
  | ðŸŸ¢Ã—ðŸ”´ | atomicidad parcial | todo-o-nada + doc contrato | chunk commiteado |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Slices 1-3 (vanta-worker) + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (Deserialize quirÃºrgico, red-team sin rupturas, Regla 8 limpia) | nextest/binarios directos, clippy, fmt |

  **Notas:** EGO-01/02/03/04 tienen commits (`82317d89`, `c82f4b2d`, `36ac14ff`, `b04d33bf`) pendientes de review P2-01 + backfill `FIND-335` â€” verificar antes de re-implementar. Cubre `EGO-05`, `EGO-09` y `FIND-221` bajo el Ã©pico `EGO-01` (trazabilidad en task file, sin prefijos nuevos).

### Task 9: EGO-03 â€” Resto tren 0.9.0 (bÃºsqueda, storage, DX, grafo)

- **Appetite:** max 3d
- **Esfuerzo:** ðŸ”´
- **Prioridad:** ðŸŸ 
- **Skills:** `codebase-memory`, `progreso`, `ponytail`
- **Reglas Ã¡rea:** `.opencode/rules/indexes.md` (HNSW/tokenizer) + `.opencode/rules/query-dsl.md` (wildcards) + `.opencode/rules/durability.md` (vacuum/borrado) + `.opencode/rules/js-ecosystem.md` + `.opencode/rules/python-bindings.md` (zero-copy).
- **Archivos clave:** `src/sdk/search/multi.rs`, `src/text_index.rs`, `src/index/`, `src/sdk/api/graphrag.rs`, `vantadb-node/src/`, `src/storage/engine/`
- **Regla 0:** `codegraph` por sÃ­mbolo (wildcards, tokenizer, finalizer, `VantaValue`, `reopen`, vacuum, `GraphRagOptions`) + blast radius antes de cada slice.
- **VerificaciÃ³n real:** ðŸŸ¡ VERIFICAR en DISCOVERY â€” IDs EGO-02/03/04/06/10/11/12/13/14/15/16/17 del Backlog; confirmar sÃ­mbolo por sÃ­mbolo (algunos ya implementados pendientes de review).
- **Gate JustificaciÃ³n:** decisiÃ³n owner 2026-10-15: las 16 entran al tren 0.9.0; sin vacuum/borrado fÃ­sico/wildcards la vida larga del agente degrada recall y disco.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --test multilingual_tokenizer_integration --test mmap_hnsw --test fjall_cold_copy_restore --test graphrag_test --test message_thread_test` verde + wildcards solo-search + `reopen(path)` + zero-copy bench before/after (Regla 9: `canonical_p99`)
- **Idioma:** todo en espaÃ±ol.
- **Cierre harness:** bench before/after obligatorio en perf + concurrencia Regla 8 + Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. wildcard contamina validaciÃ³n put/get. 2. vacuum sin presupuesto rompe P99. 3. zero-copy NAPI/PyO3 diverge por plataforma.
- **Stop conditions:** bench before/after ausente en perf â†’ bloquear merge (Regla 9).
- **Cynefin:** ðŸŸ§ complejo â€” HNSW/vacuum/recall solo emergen con bench.
- **Top 3 riesgos:** 1. validaciÃ³n compartida search/write. 2. recall degradado. 3. GC V8 en ingestas.
- **Uphill/Downhill:** â¬†ï¸ 2 / â¬‡ï¸ 4
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/EGO-03.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸ”´ | wildcard en write path | `validate_namespace_pattern` solo-search | test put con `*` verde (mal) |
  | ðŸŸ¡Ã—ðŸŸ¡ | vacuum degrada P99 | presupuesto + background | bench rojo |
  | ðŸŸ¡Ã—ðŸŸ¡ | borrado solo API-level | vacuum fÃ­sico o API borrado seguro | auditorÃ­a disco |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Slices 1-8 + briefs (vanta-worker) + review P2-01 + fixes | âœ… COMPLETED â€” APPROVE (red-team sin rupturas, Regla 8 limpia, FIND-343 derivado) | nextest/binarios, clippy, fmt, public-api regen |

  **Notas:** Ã‰pico `EGO-03` cubre el resto del tren (trazabilidad por ID en el task file, sin prefijos nuevos).

### Task 10: DIST-19 â€” Fachada cognitiva + memoria-proyecto (MEMG + DIST-21)

- **Appetite:** max 3d
- **Esfuerzo:** ðŸ”´
- **Prioridad:** ðŸŸ 
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Reglas Ã¡rea:** `.opencode/rules/js-ecosystem.md` + `.opencode/rules/server-mcp.md` (contrato) + `.opencode/rules/api-contract.md`.
- **Archivos clave:** `vantadb-node/src/`, `vanta-memory/`, `src/graph.rs`, `.github/workflows/`, `src/sdk/types/record.rs`
- **Regla 0:** `codegraph` de `VantaCognitiveAPI`, `code_*`, `MemoryScope`, `retrieve()` + arquitectura (`overview`, `clusters`) antes de diseÃ±ar.
- **VerificaciÃ³n real:** ðŸŸ¡ VERIFICAR en DISCOVERY â€” `vanta-memory` existe; `code_*` tools existen pero `code_files` no (Backlog MEMG-09); DIST-21 exige Electron real.
- **Gate JustificaciÃ³n:** sin fachada Ego sigue en subprocess MCP-stdio; sin MEMG-03/09 memoria y grafo siguen separados.
- **Gate Result:** âœ… DO
- **Contrato:** `NativeVantaDB.memory.* / context.* / tasks.*` expuestos + suite contrato Ego (`MEMG-24`) verde + CI Electron (Windows x64/macOS/Linux, `asarUnpack`) verde
- **Idioma:** plan en espaÃ±ol; spec de scopes/contrato en inglÃ©s si vive en `docs/api/`.
- **Cierre harness:** Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. AbortSignal no propaga a Tokio. 2. `retrieve()` trust-aware rompe byte-identidad (precedente MEMG-21 opt-in). 3. scopes sin enforcement (spec-first, enforcement = MGR-04/FIND-301).
- **Stop conditions:** Electron CI sin runner real â†’ DEFER DIST-21 con evidencia, resto sigue.
- **Cynefin:** ðŸŸ§ complejo â€” contrato cross-runtime.
- **Top 3 riesgos:** 1. ABI drift. 2. overhead JSON-RPC residual. 3. scopes planos vs jerarquÃ­a.
- **Uphill/Downhill:** â¬†ï¸ 2 / â¬‡ï¸ 3
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/DIST-19.md`
- **Estado:** âœ… COMPLETED (S0-S8; S9/S2b/Regla-8 como follow-ups con dueÃ±o)
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸŸ¡Ã—ðŸ”´ | ABI drift silencioso | CI Electron real | 404/ABI error |
  | ðŸŸ¡Ã—ðŸŸ¡ | retrieve() cambia defaults | opt-in byte-idÃ©ntico | test golden rojo |
  | ðŸŸ¢Ã—ðŸŸ¡ | scopes sin enforcement | spec-first + tipos | â€” |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | S0/S1 (worker1) + S3/S8 (worker2) + review P2-01 | S3 âœ… APPROVE (6 fns reales, guards, quarantine); S4/S5 pendientes | cargo check/clippy/fmt, napi |
  | 2 | S4-S8 (worker2 cont.) + review P2-01 final | âœ… COMPLETED â€” APPROVE (vitest 10/10 + 69/69, specs EN, trust byte-idÃ©ntico) | vitest, clippy, fmt |

  **Notas:** Ã‰pico `DIST-19` cubre `MEMG-03/09/24/25/26`, `FIND-320`, `DIST-20/21` (trazabilidad en task file).

### Task 11: SRV-10 â€” Cifrado server + consolidaciÃ³n workflows (CI-08)

- **Appetite:** max 3d
- **Esfuerzo:** ðŸŸ¡
- **Prioridad:** ðŸŸ 
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Reglas Ã¡rea:** `.opencode/rules/server-mcp.md` + `.opencode/rules/release-ci.md` (workflows) + `.opencode/rules/open-core-licensing.md` si toca Pro.
- **Archivos clave:** `vantadb-server/`, `docs/api/HTTP_API.md`, `.github/workflows/`, `docs/dev/workflow/TRIGGERS.md`
- **Regla 0:** `codegraph` de `encryption`, rutas server + lectura de `on:` de cada workflow tocado + `TRIGGERS.md` antes de consolidar.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `HTTP_API.md:701` declara SRV-10 roadmap; core ya tiene feature `encryption`; 40 workflows verificados (vs 28 en CI_POLICY â†’ FIND-321).
- **Gate JustificaciÃ³n:** server sin cifrado en reposo; 40 workflows Ã— matrices queman runners.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb-server` (e2e+security) verde + workflows â‰¤18 sin renombrar `release.yml`/`release-npm-61.yml`/`release-npm-node.yml` + `actionlint` 0 + `TRIGGERS.md` sincronizado
- **Idioma:** todo en espaÃ±ol en el plan; `TRIGGERS.md` y docs workflow en inglÃ©s.
- **Cierre harness:** Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. renombrar release file rompe OIDC (prohibiciÃ³n permanente). 2. consolidar demos rompe aceptaciÃ³n F5/F6. 3. `continue-on-error` sin CATEGORY.
- **Stop conditions:** consolidaciÃ³n toca publish path â†’ abortar y acotar a informational/demoâ†’nightly.
- **Cynefin:** ðŸŸ¨ complicado â€” triggers + OIDC.
- **Top 3 riesgos:** 1. OIDC filename-bound. 2. matrices perdidas. 3. branch protection desactualizado.
- **Uphill/Downhill:** â¬†ï¸ 1 / â¬‡ï¸ 2
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/SRV-10.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸ”´Ã—ðŸ”´ | rename release file | prohibido, checklist | diff con rename |
  | ðŸŸ¡Ã—ðŸŸ¡ | jobs huÃ©rfanos | migrar o documentar muerte | FIND-321 |
  | ðŸŸ¢Ã—ðŸŸ¡ | timeouts ausentes | `timeout-minutes` por job | RULES.md Â§2 |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | FIND-321 + hallazgo SRV-10-cÃ³digo resuelto + Gate D | Worker1: inventario 28â†’40 + SRV-10 SKIP documentado (resuelto en ca3f00da/8f758e24). Gate D owner: CI-08 COMPLETA 40â†’18 (worker2 en fondo) | TRIGGERS 40/40, diff --check 0 |
  | 2 | CI-08 Steps 2-3 (worker2) | 40â†’32: 5 demosâ†’demo-acceptance.yml + 4 infoâ†’nightly.yml, 0 jobs perdidos, release* intactos. Step 4 pendiente (32â†’18, tabla lista). BLOQUEO: branch protection required-checks renombrados (acciÃ³n admin pre-merge) | checker PASS, conteo 32 |
  | 3 | CI-08 Step 4 + review P2-01 + 7 fixes | 32â†’18 âœ…. Review: CHANGES-REQUIRED (3 Critical + 1 Required) â†’ fixes aplicados + re-verificado. APPROVE | yaml 18/18, patrones limpios, diff --check 0 |

  **Notas:** Cubre `CI-08` + `FIND-321` (inventario primero, consolidaciÃ³n despuÃ©s).

### Task 12: FIND-340 â€” PreparaciÃ³n release 0.8.0 sin push + docs finales

- **Appetite:** max 1d
- **Esfuerzo:** ðŸŸ¢
- **Prioridad:** ðŸ”´
- **Skills:** `documentation-skill`, `progreso`, `ponytail`
- **Archivos clave:** `scripts/git-backup.ps1`, `scripts/verify-release.ps1`, `docs/user/QUICKSTART.md`, `docs/dev/FASE-A.md`, `docs/dev/architecture/ARCHITECTURE.md`
- **Regla 0:** leer `RULES.md Â§8` + `PUBLISH.md` (gate de release completo) + `rg` de pins viejos antes de editar docs.
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `git-backup.ps1` con `-Dest` + verify fail-closed (RULES.md Â§8); `ARCHITECTURE.md` stale v0.6.1 (FIND-322); pins 0.6.1 vs 0.7.0 live (DIST-08).
- **Gate JustificaciÃ³n:** decisiÃ³n owner: solo-preparar; release no existe hasta verificado (PUBLISH.md).
- **Gate Result:** âœ… DO
- **Contrato:** `pwsh scripts/git-backup.ps1 -Dest <externo>` exit 0 + `git bundle verify` exit 0 + `verify-release.ps1 -Tag v0.8.0 -Smoke` ejecutado en local (sin push) + QUICKSTART pins alineados + ARCHITECTURE refresh scoped
- **Idioma:** plan en espaÃ±ol; `QUICKSTART.md` y `ARCHITECTURE.md` se editan en inglÃ©s.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, **push prohibido sin OK explÃ­cito**.
- **Pre-mortem:** 1. backup en mismo volumen (script advierte). 2. conteo `139 commits` puede variar â€” recontar. 3. FASE-A requiere humano (no automatizable).
- **Stop conditions:** bundle verify â‰ 0 â†’ STOP, no seguir (R2 GitLab).
- **Cynefin:** ðŸŸ¦ obvio.
- **Top 3 riesgos:** 1. backup mismo disco. 2. push accidental (prohibido sin OK). 3. pins docs vs registry.
- **Uphill/Downhill:** â¬†ï¸ 0 / â¬‡ï¸ 3
- **DoD task:** contrato âœ… Â· task file sync Â· recitation
- **Task file:** `docs/dev/tasks/FIND-340.md`
- **Estado:** âœ… COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | ðŸ”´Ã—ðŸ”´ | push sin OK owner | prohibido Regla 7 | â€” |
  | ðŸŸ¡Ã—ðŸ”´ | backup no verificado | fail-closed + `git clone <bundle>` | verify â‰ 0 |
  | ðŸŸ¢Ã—ðŸŸ¡ | pins stale | DIST-08 | smoke falla |

  **Iteraciones:**
  | # | AcciÃ³n | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps S1-S5 (vanta-worker) + review P2-01 | âœ… COMPLETED â€” APPROVE vanta-review (backup verify 0, pins 0, ARCHITECTURE scoped) | bundle verify, pins, gates docs |

  **Notas:** `PROC-01-push` queda ðŸ”´ BLOQUEADO hasta tu `push` explÃ­cito. Carril negocio `BIZ-04/10/11`, `STRAT-01/02`, `FASE-A/R-05`, `BIZ-16/17` es humano y vive en `Backlog-negocio.md` â€” fuera de este plan. ID `FIND-340` libre verificado. Sin tocar versiones/CHANGELOG/tags (release-plz).

plan-adjust [2026-10-15]: revisiÃ³n `.opencode` â€” `HYG-*/COG-*/SRV-CI/REL-PREP/EGO-P0/EGO-FULL` renombrados a taxonomÃ­a (`FIND-336..340` + Ã©picos `SURF-01/API-10/DX-13/EGO-01/EGO-03/DIST-19/SRV-10`); aÃ±adidas `documentation-skill`, Regla 0, cierre Clean/`/cleanCA`/`just verify`/`ocr-review`, roles git y regla de idioma. Plan Ã­ntegro en espaÃ±ol.
- â¬†ï¸ uphill antes: destino SPEC/CLA/hooks sin decidir â†’ despuÃ©s: locked (docs/specs, .github/cla, pre-commit Ãºnico).
plan-adjust [2026-10-16]: orden directa owner fuera de las 12 tasks â€” FIND-341 congela desktop en `feature-freez/` (jobs CI fuera, dependabot fuera, refs mecÃ¡nicas + docs vivas re-pathadas); EGO-11 Gate D resuelto (`scan()` explÃ­cito).
plan-adjust [2026-10-16]: release-gaps Bâ€“F a FIND-342 (worker vanta-lead en fondo). Decisiones: B bump+gate (submit=owner), C bump 0.8.0 (sidecars 200 OK), D manifest, E tarball-check, F fan-out (probar en prÃ³ximo release). Main SIN branch protection (404) = acciÃ³n owner; D4 sin objeto. Rama: main estable-release, develop desarrollo (Regla 7 vigente: push solo con OK explÃ­cito).
plan-adjust [2026-10-16]: FIND-342 B-E âœ… COMPLETED (APPROVE; commit orquestador). F con Gate D STOP â†’ propuesta dispatch al owner (F1 pendiente decisiÃ³n). Main sin protection = acciÃ³n owner; D4 sin objeto.
plan-adjust [2026-10-16]: F dispatch APROBADO por owner â†’ F1 en curso (fanout en release.yml, sin tocar publish ifs).
plan-adjust [2026-10-16]: F1 âœ… COMPLETED (APPROVE; commit orquestador). MCP registry â†’ FIND-344 (worker en fondo). EGO-03 slices 2+ en fondo.
plan-adjust [2026-10-16]: FIND-344 âœ… COMPLETED (APPROVE; server.json 0 errores vs $schema con jsonschema lib). Queda owner: login + first publish. EGO-03 slices en fondo, luego DIST-19.
