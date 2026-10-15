---
title: "Plan de Ejecución: Pre-mercado VantaDB — orden, estabilización y salida"
kind: plan
status: active
description: "Campaña pre-mercado: orden, estabilización y salida (12 tasks DO)"
tags: [pre-mercado, release]
---

# Plan de Ejecución: Pre-mercado VantaDB — orden, estabilización y salida

> **Inicio:** 2026-10-15
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/dev/Backlog.md` (DELTA + EGO + SURF) + `docs/dev/Backlog-negocio.md` + auditoría `auditor_a_pre_mercado_y_plan_de_salida_vantadb.md` (verificada contra disco 2026-10-15)
> **Autonomous:** false
> **Decisiones owner locked (2026-10-15):** SPEC/CONSTRAINTS→`docs/specs/` nueva · CLA→`.github/cla/` · borrar bak+`.md` ✅ · `.editorconfig` nuevo ✅ · hooks→`pre-commit` único · superficie/docs ✅ · DX ✅ · PROC-01 solo-preparar · tren EGO completo · carril negocio-humano separado

> **Normas harness (revisión `.opencode`, todo el plan en español):**
> - **Taxonomía:** solo IDs registrados (`FIND-*`, `EGO-*`, `SURF-*`, `DOC-*`, `API-*`, `DX-*`, `DIST-*`, `MEMG-*`, `SRV-*`, `BIZ-*`). Los `HYG-*/COG-*/REL-PREP` del borrador se renombran abajo a `FIND-336..340` y a IDs backlog existentes. Sin prefijos nuevos.
> - **Skills obligatorias:** `documentation-skill` antes de tocar `docs/**`; `codebase-memory` antes de `codegraph_*`; `progreso` al abrir/cerrar; `ponytail` siempre.
> - **Regla 0:** antes de mover/borrar, leer archivo + `rg` entrante/saliente + veredicto. Incluida por tarea.
> - **Commits:** solo `vanta-lead`, un commit local por tarea, Conventional Commits, **push prohibido sin OK explícito del owner (Regla 7)**. Sin tocar versiones/CHANGELOG/tags (release-plz).
> - **Cierre código:** norma Clean + `/cleanCA` del alcance + `ponytail-review` + `just verify` (o `verify_changed.ps1`) + `ocr-review.ps1` (Critical/High bloquean).
> - **Idioma:** el plan va íntegramente en español; el contenido técnico nuevo en `docs/api/`, `docs/dev/architecture/`, `QUICKSTART.md` se redacta en inglés (fuente de verdad).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 12 |
| 🟡 DEFER | 6 |
| ❌ SKIP | 2 |
| 🔴 BLOQUEADO | 2 |

Status: ⬆️ uphill = 3 (contrato Ego completo, pre-commit shim Windows, pins npm) · ⬇️ downhill = 12 tasks pendientes

DEFER: `JS-01`, `PY-04`, `DIST-18`, `DIST-01`, `CORE-03-ejecución`, `FIND-324` (post-1.0, Freeze List).
SKIP: `docs/dev/references/.editorconfig` (no existe — crear nuevo), `docs/dev/plans/2026-10-04-*` (cita inválida del reporte).
BLOQUEADO: `PROC-01-push` (requiere OK explícito owner, Regla 7), carril negocio `BIZ-*` (humano, fuera del pipeline).

## Tasks

### Task 1: FIND-336 — Mover SPEC/CONSTRAINTS a docs/specs/

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Skills:** `documentation-skill`, `progreso`, `ponytail`
- **Archivos clave:** `SPEC.md`, `CONSTRAINTS.md`, `docs/specs/`, `AGENTS.md`, `docs/index.md`
- **Regla 0:** leer ambos archivos + `rg -n "SPEC\.md|CONSTRAINTS\.md" --glob "!target/**" --glob "!.opencode/**"` (entrantes) + listar sus links salientes; presentar veredicto antes del `git mv`.
- **Verificación real:** ✅ CÓDIGO-REAL — `SPEC.md` + `CONSTRAINTS.md` existen en raíz (listado 2026-10-15); `docs/specs/` NO existe → crear. Práctica spec-kit/SDD: specs fuera de raíz.
- **Gate Justificación:** root clutter real (30+ archivos en raíz); decisión owner `docs/specs/` nueva.
- **Gate Result:** ✅ DO
- **Contrato:** `git status --short` limpio tras moves + `rg` solo apunta a `docs/specs/` + `node scripts/docs/check-links.mjs` exit 0
- **Idioma:** plan y mensajes en español; si un `.md` movido exige frontmatter `title`+`kind`, respetar `documentation-skill`.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead` (`docs: ...`), sin push.
- **Pre-mortem:** 1. links rotos en README/AGENTS que apuntaban a raíz. 2. gate-docs-links falla por rutas viejas. 3. duplicado si alguien recrea los archivos en raíz.
- **Stop conditions:** links rotos >3 → abortar y completar mapa Regla 0 antes del commit.
- **Cynefin:** 🟦 obvio — move mecánico con grep de refs.
- **Top 3 riesgos:** 1. refs entrantes no mapeadas. 2. índice docs desactualizado. 3. convención `kind` de frontmatter si aplica.
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 1 (move + fix links + verify)
- **DoD task:** contrato ✅ · task file sync · recitation actualizada
- **Task file:** `docs/dev/tasks/FIND-336.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | links a `./SPEC.md` quedan rotos | `rg` previo + fix en mismo commit | verify `check-links.mjs` rojo |
  | 🟢×🟡 | índice docs no lista `docs/specs/` | `gen-index.mjs --write` | diff índice vacío |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Steps 1-4 + gates + review P2-01 | ✅ COMPLETED — APPROVE vanta-review (ses_ebecfacb) | check-links 0 broken, check-docs sin nuevos missing, fmt exit 0, floor-guard exit 0 |

  **Notas:** Decisión owner 2026-10-15: `docs/specs/` nueva (no `docs/dev/architecture/`). ID `FIND-336` libre (verificado 2026-10-15: `FIND-330..335` ocupados, `336+` libres).

### Task 2: FIND-337 — Mover CLA a .github/cla/ + fix links

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Skills:** `documentation-skill`, `progreso`, `ponytail`
- **Archivos clave:** `CLA_INDIVIDUAL.md`, `CLA_CORPORATE.md`, `.github/cla/`, `.github/clabot-config.json`, `CONTRIBUTING.md`
- **Regla 0:** leer ambos CLA (no alterar texto legal, solo mover) + `rg -n "CLA_" .github/ CONTRIBUTING.md README.md` + veredicto antes del move.
- **Verificación real:** ✅ CÓDIGO-REAL — CLAs en raíz; `.github/clabot-config.json:2` ya apunta a `.github/CLA_*.md` (ruta inexistente hoy) → mover a `.github/cla/` + fix config en mismo commit. Práctica: community-health en `.github/`.
- **Gate Justificación:** links clabot rotos hoy; decisión owner `.github/cla/`.
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path .github/cla/CLA_INDIVIDUAL.md` + `rg -n "CLA_" .github/clabot-config.json CONTRIBUTING.md` consistente + `check-links.mjs` exit 0
- **Idioma:** todo en español en el plan; no traducir el texto legal de los CLA.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. CONTRIBUTING referencia ruta vieja. 2. bot no re-lee config hasta re-run. 3. firma pendiente de PRs abiertos.
- **Stop conditions:** >2 archivos con refs viejas tras grep → abortar y completar mapa Regla 0.
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** 1. refs rotas. 2. mensaje bot desactualizado. 3. alteración accidental del texto legal.
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 1
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/FIND-337.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | texto legal alterado por accidente | `git diff --stat` solo renames + config | diff muestra cambios de contenido |
  | 🟢×🟢 | bot no refleja cambio | nota en PR, no bloquea | — |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** ID `FIND-337` libre verificado.

### Task 3: FIND-338 — Borrar bak + templates .md duplicados

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Skills:** `progreso`, `ponytail`
- **Archivos clave:** `opencode.jsonc.bak-cleanup-20260928`, `.github/ISSUE_TEMPLATE/bug_report.md`, `.github/ISSUE_TEMPLATE/feature_request.md`, `.github/ISSUE_TEMPLATE/config.yml`
- **Regla 0:** `rg -n "bug_report\.md|feature_request\.md|bak-cleanup" .github/ docs/ .opencode/` + veredicto antes del `git rm`.
- **Verificación real:** ✅ CÓDIGO-REAL — bak existe en raíz; `ISSUE_TEMPLATE/` tiene `.md`+`.yml` duplicados (verificado 2026-10-15). GitHub prioriza `.yml`.
- **Gate Justificación:** duplicados muestran opciones dobles al usuario; bak es huérfano.
- **Gate Result:** ✅ DO
- **Contrato:** `Get-ChildItem .github/ISSUE_TEMPLATE/*.md` vacío + `git status --short` solo deletes esperados
- **Idioma:** todo en español.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. algún workflow referencia al `.md` por nombre. 2. `config.yml` lista templates. 3. borrado sin backup.
- **Stop conditions:** grep muestra referencia al `.md` → abortar y actualizar referencia primero.
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** 1. referencia por nombre. 2. choques con PR abierto. 3. ninguno más.
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 1
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/FIND-338.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟡 | `config.yml` lista `.md` | actualizar en mismo commit | grep pre-delete |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Aprobado explícito owner 2026-10-15. ID `FIND-338` libre verificado.

### Task 4: FIND-339 — Crear .editorconfig + unificar hooks en pre-commit

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Skills:** `progreso`, `ponytail`
- **Archivos clave:** `.editorconfig`, `.pre-commit-config.yaml`, `.githooks/pre-commit`, `.githooks/pre-push`, `CONTRIBUTING.md`
- **Regla 0:** leer `.githooks/pre-commit:1-87` + `.pre-commit-config.yaml:1-44` + `rg -n "core.hooksPath|pre-commit" CONTRIBUTING.md docs/dev/workflow/ .github/workflows/` + veredicto antes de editar.
- **Verificación real:** ✅ CÓDIGO-REAL — `.editorconfig` ausente en raíz; `docs/dev/references/.editorconfig` NO existe (solo 4 `.md`) → crear nuevo. `.githooks/pre-commit` solo fmt/clippy/actionlint; `.pre-commit-config.yaml` único con ruff/prettier/markdownlint. Decisión owner: `pre-commit` sistema único, `.githooks` → shim.
- **Gate Justificación:** sin `.editorconfig` no hay norma LF/charset cross-IDE; borrar pre-commit (propuesta reporte) perdería gates Python/JS/docs.
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path ./.editorconfig` + `pre-commit run --all-files` exit 0 + `git config core.hooksPath` documentado en `CONTRIBUTING.md`
- **Idioma:** todo en español.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. hook cargo pesado en Windows sin sccache. 2. pre-commit autoupdate cambia revs. 3. shim `.githooks` con CRLF rompe `sh`.
- **Stop conditions:** `pre-commit run` >10min dos veces → DEFER con log (rabbit hole).
- **Cynefin:** 🟨 complicado — dos sistemas con solape parcial, requiere experto para el shim.
- **Top 3 riesgos:** 1. divergencia hooks vs CI gates. 2. instalación `pre-commit` ausente en devs. 3. EOL `*.ps1` (fijar `crlf` en `.editorconfig`).
- **Uphill/Downhill:** ⬆️ 1 (shim Windows) / ⬇️ 2
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/FIND-339.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | hooks locales divergen de CI | espejar `gate-docs`/`ci-rust` en config | diff configs |
  | 🟡×🟢 | devs sin pre-commit instalado | shim con fallback `cargo fmt --check` | test sin pre-commit |
  | 🟢×🟡 | EOL `*.ps1` | regla explícita en `.editorconfig` | CI Windows rojo |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** ID `FIND-339` libre verificado.

### Task 5: SURF-01 — Manifest de superficie + SSOT documental (DOC-22 + REL-03)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Archivos clave:** `docs/api/BINDINGS_NAMESPACES.md`, `docs/dev/workflow/PUBLISH.md`, `docs/index.md`, `tests/version_coherence.rs`, `scripts/docs/check-npm-versions.mjs`
- **Regla 0:** `codegraph_codegraph_explore "public-api version_coherence sdk_serialization"` + mapa de callers antes de declarar el manifest.
- **Verificación real:** ✅ CÓDIGO-REAL — `PUBLISH.md` tabla DIST-06 existe; `version_coherence.rs` + `check-npm-versions.mjs` existen (`scripts/docs/` listado 2026-10-15); drift core 0.8.0 vs bindings 0.7.0 documentado en Backlog DELTA.
- **Gate Justificación:** sin manifest la contradicción fila-vs-ADR-vs-código se reintroduce por cada binding (pre-req de API-10 y CI-08).
- **Gate Result:** ✅ DO
- **Contrato:** `node scripts/docs/check-npm-versions.mjs` exit 0 + `node scripts/docs/check-docs.mjs` exit 0 + surface manifest generado sin símbolos fuera de `public-api.txt`
- **Idioma:** plan en español; el manifest y el authority map que vivan en `docs/api/` o `docs/index.md` respetan `documentation-skill` (frontmatter, links relativos).
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. manifest manual que se desincroniza (debe ser generado). 2. authority map duplica contenido en vez de referenciar. 3. `FIND-321` (40 vs 28 workflows) contamina el inventario.
- **Stop conditions:** manifest no generable mecánicamente → scope a 1 dimensión (versiones) y DEFER resto (precedente DOC-23).
- **Cynefin:** 🟨 complicado — taxonomía multi-superficie.
- **Top 3 riesgos:** 1. drift reintroducido. 2. owners por superficie sin definir. 3. gate mecánico ausente.
- **Uphill/Downhill:** ⬆️ 1 / ⬇️ 2
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/SURF-01.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | manifest manual stale en 1 mes | generado + test paridad | review sin gate |
  | 🟡×🟡 | contradicción MEMG-05/FIND-303 | detectar contra manifest | fila sin dueño |
  | 🟢×🟡 | inventario workflows diverge | gate conteo (FIND-321) | — |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Cubre `DOC-22` + `REL-03` como slices del mismo manifest (trazabilidad en task file).

### Task 6: API-10 — Schema-first slice 1 + pins (DIST-08)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴
- **Prioridad:** 🔴
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Reglas área:** `.opencode/rules/api-contract.md` (SDK/API pública) + `.opencode/rules/query-dsl.md` si toca IQL.
- **Archivos clave:** `src/sdk/types/`, `tests/api/public-api.txt`, `tests/api/openapi_yaml_parity.rs`, `tests/version_coherence.rs`, `tests/sdk_serialization.rs`, `docs/user/QUICKSTART.md`
- **Regla 0:** `codegraph_codegraph_explore "Value sdk types public-api"` + blast radius bindings antes de tocar DTOs.
- **Verificación real:** ✅ CÓDIGO-REAL — `public-api.txt`, `public_api.rs`, `openapi_yaml_parity.rs`, `version_coherence.rs`, `sdk_serialization.rs` existen; pins 0.7.0 vs live verificados en Backlog DIST-08.
- **Gate Justificación:** sin slice 1 (JSON Schema DTOs + paridad Rust→TS/Python) cada binding reintroduce drift.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --test public_api --test version_coherence --test sdk_serialization --test openapi_yaml_parity` verde + `check-npm-versions.mjs` exit 0
- **Idioma:** plan en español; schemas y docs `docs/api/` en inglés.
- **Cierre harness:** Clean + `/cleanCA` del alcance + `ponytail-review` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. codegen completo pre-1.0 (scope creep — solo slice 1). 2. breaking WASM-sync (ventana 0.x, decidir en DIST-19). 3. P99 varianza (falso positivo FIND-154).
- **Stop conditions:** slice 1 >appetite → recortar a DTOs wire + 1 test paridad, resto post-1.0.
- **Cynefin:** 🟧 complejo — drift emerge por binding; probe-sense con 1 dimensión primero.
- **Top 3 riesgos:** 1. scope creep a codegen. 2. breaking JS. 3. varianza runner.
- **Uphill/Downhill:** ⬆️ 1 / ⬇️ 2
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/API-10.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🔴×🟡 | codegen total pre-1.0 | slice 1 solo | appetite excedido |
  | 🟡×🔴 | breaking consumidores WASM | spec en DIST-19 | test binding rojo |
  | 🟡×🟢 | perf gate falso positivo | re-run + baseline | FIND-154 |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** No usar `scripts/check_openapi_parity.mjs` (ruta inválida del reporte) — lo real está en `scripts/docs/check-*.mjs`. Regla dura `-p`: iterar con `-p vantadb`, `--workspace` solo al cierre.

### Task 7: DX-13 — Doctor, completions y dead code (DX-02 + DEAD-01)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Skills:** `codebase-memory`, `progreso`, `ponytail`
- **Archivos clave:** `src/cli_handlers/diagnostics.rs`, `completions/`, `src/cli.rs`, `Justfile`
- **Regla 0:** `rg -n "allow\(dead_code\)" src/ vanta-memory/src/` + `codegraph` de `doctor` antes de tocar.
- **Verificación real:** ✅ CÓDIGO-REAL — `diagnostics.rs` existe (doctor dry-run+fix); `completions/` con drift (DX-13); `#[allow(dead_code)]` dispersos por grep.
- **Gate Justificación:** quick wins pre-anuncio; receta `surface-check` fija el gate.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo run -p vantadb --bin vanta-cli -- doctor` exit 0 + `cargo clippy -p vantadb --all-targets -- -D warnings` 0 warnings + receta `surface-check` en `Justfile`
- **Idioma:** todo en español.
- **Cierre harness:** Clean + `/cleanCA` alcance + `ponytail-review` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. regenerar completions a mano (prohibido — desde binario). 2. `dead_code` con consumidores via feature flag. 3. doctor con fix destructivo (solo aditivo).
- **Stop conditions:** clippy con >20 warnings legacy → acotar a módulos tocados + FIND nueva deuda.
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** 1. hand-edit completions. 2. flags sin consumidor real. 3. doctor destructivo.
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/DX-13.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟡 | completions hand-edit | regenerar desde binario | diff manual |
  | 🟡×🟢 | dead_code bajo feature | `rg` por feature | — |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Cubre `DX-02` (preflight doctor) + `DEAD-01` en el mismo barrido.

### Task 8: EGO-01 — Contrato crítico (EGO-05/09 + FIND-221)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Skills:** `codebase-memory`, `progreso`, `ponytail`
- **Reglas área:** `.opencode/rules/api-contract.md` + `.opencode/rules/js-ecosystem.md` (Node) + `.opencode/rules/durability.md` (flock/WAL).
- **Archivos clave:** `vantadb-node/src/lib.rs`, `src/sdk/types.rs`, `src/sdk/api/memory.rs`, `tests/storage/multi_process_lock.rs`, `tests/sdk_serialization.rs`
- **Regla 0:** `codegraph` de `list`, `Value`, `put_batch`, `validate_namespace` + callers Node/WASM/Python antes de editar.
- **Verificación real:** ✅ CÓDIGO-REAL — rutas citadas existen en Backlog EGO-01/05/09 + FIND-221 (`put_batch` chunk 1000). Alcance P0 del tren 0.9.0.
- **Gate Justificación:** cursor Null + Value plano + flock bloquean a Ego hoy (workarounds en adapter); atomicidad `put_batch` ambigua.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --test sdk_serialization --test multi_process_lock` + `list({cursor: undefined})` y `cursor: "0"` no fallan + `put_batch` todo-o-nada documentado
- **Idioma:** todo en español.
- **Cierre harness:** concurrencia (Regla 8: `vanta-chaos` + `vanta-review`, P2-01 contextos distintos) + Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. NAPI Null vs undefined (serialización). 2. cambio `Value` rompe wire WASM/Python. 3. `open_readonly` sin lock pierde garantías WAL.
- **Stop conditions:** breaking cross-binding sin paridad → abortar y acotar a Node-only + FIND.
- **Cynefin:** 🟨 complicado — FFI + tipos wire.
- **Top 3 riesgos:** 1. paridad bindings. 2. ABI NAPI. 3. semántica lock Windows (requiere test real, ver DIST-21).
- **Uphill/Downhill:** ⬆️ 1 / ⬇️ 3
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/EGO-01.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | breaking wire | Deserialize tolerante retrocompatible | test cross-binding rojo |
  | 🟡×🟡 | lock Windows | test en Windows real (DIST-21) | solo-CI-linux |
  | 🟢×🔴 | atomicidad parcial | todo-o-nada + doc contrato | chunk commiteado |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** EGO-01/02/03/04 tienen commits (`82317d89`, `c82f4b2d`, `36ac14ff`, `b04d33bf`) pendientes de review P2-01 + backfill `FIND-335` — verificar antes de re-implementar. Cubre `EGO-05`, `EGO-09` y `FIND-221` bajo el épico `EGO-01` (trazabilidad en task file, sin prefijos nuevos).

### Task 9: EGO-03 — Resto tren 0.9.0 (búsqueda, storage, DX, grafo)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴
- **Prioridad:** 🟠
- **Skills:** `codebase-memory`, `progreso`, `ponytail`
- **Reglas área:** `.opencode/rules/indexes.md` (HNSW/tokenizer) + `.opencode/rules/query-dsl.md` (wildcards) + `.opencode/rules/durability.md` (vacuum/borrado) + `.opencode/rules/js-ecosystem.md` + `.opencode/rules/python-bindings.md` (zero-copy).
- **Archivos clave:** `src/sdk/search/multi.rs`, `src/text_index.rs`, `src/index/`, `src/sdk/api/graphrag.rs`, `vantadb-node/src/`, `src/storage/engine/`
- **Regla 0:** `codegraph` por símbolo (wildcards, tokenizer, finalizer, `VantaValue`, `reopen`, vacuum, `GraphRagOptions`) + blast radius antes de cada slice.
- **Verificación real:** 🟡 VERIFICAR en DISCOVERY — IDs EGO-02/03/04/06/10/11/12/13/14/15/16/17 del Backlog; confirmar símbolo por símbolo (algunos ya implementados pendientes de review).
- **Gate Justificación:** decisión owner 2026-10-15: las 16 entran al tren 0.9.0; sin vacuum/borrado físico/wildcards la vida larga del agente degrada recall y disco.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --test multilingual_tokenizer_integration --test mmap_hnsw --test fjall_cold_copy_restore --test graphrag_test --test message_thread_test` verde + wildcards solo-search + `reopen(path)` + zero-copy bench before/after (Regla 9: `canonical_p99`)
- **Idioma:** todo en español.
- **Cierre harness:** bench before/after obligatorio en perf + concurrencia Regla 8 + Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. wildcard contamina validación put/get. 2. vacuum sin presupuesto rompe P99. 3. zero-copy NAPI/PyO3 diverge por plataforma.
- **Stop conditions:** bench before/after ausente en perf → bloquear merge (Regla 9).
- **Cynefin:** 🟧 complejo — HNSW/vacuum/recall solo emergen con bench.
- **Top 3 riesgos:** 1. validación compartida search/write. 2. recall degradado. 3. GC V8 en ingestas.
- **Uphill/Downhill:** ⬆️ 2 / ⬇️ 4
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/EGO-03.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | wildcard en write path | `validate_namespace_pattern` solo-search | test put con `*` verde (mal) |
  | 🟡×🟡 | vacuum degrada P99 | presupuesto + background | bench rojo |
  | 🟡×🟡 | borrado solo API-level | vacuum físico o API borrado seguro | auditoría disco |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Épico `EGO-03` cubre el resto del tren (trazabilidad por ID en el task file, sin prefijos nuevos).

### Task 10: DIST-19 — Fachada cognitiva + memoria-proyecto (MEMG + DIST-21)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴
- **Prioridad:** 🟠
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Reglas área:** `.opencode/rules/js-ecosystem.md` + `.opencode/rules/server-mcp.md` (contrato) + `.opencode/rules/api-contract.md`.
- **Archivos clave:** `vantadb-node/src/`, `vanta-memory/`, `src/graph.rs`, `.github/workflows/`, `src/sdk/types/record.rs`
- **Regla 0:** `codegraph` de `VantaCognitiveAPI`, `code_*`, `MemoryScope`, `retrieve()` + arquitectura (`overview`, `clusters`) antes de diseñar.
- **Verificación real:** 🟡 VERIFICAR en DISCOVERY — `vanta-memory` existe; `code_*` tools existen pero `code_files` no (Backlog MEMG-09); DIST-21 exige Electron real.
- **Gate Justificación:** sin fachada Ego sigue en subprocess MCP-stdio; sin MEMG-03/09 memoria y grafo siguen separados.
- **Gate Result:** ✅ DO
- **Contrato:** `NativeVantaDB.memory.* / context.* / tasks.*` expuestos + suite contrato Ego (`MEMG-24`) verde + CI Electron (Windows x64/macOS/Linux, `asarUnpack`) verde
- **Idioma:** plan en español; spec de scopes/contrato en inglés si vive en `docs/api/`.
- **Cierre harness:** Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. AbortSignal no propaga a Tokio. 2. `retrieve()` trust-aware rompe byte-identidad (precedente MEMG-21 opt-in). 3. scopes sin enforcement (spec-first, enforcement = MGR-04/FIND-301).
- **Stop conditions:** Electron CI sin runner real → DEFER DIST-21 con evidencia, resto sigue.
- **Cynefin:** 🟧 complejo — contrato cross-runtime.
- **Top 3 riesgos:** 1. ABI drift. 2. overhead JSON-RPC residual. 3. scopes planos vs jerarquía.
- **Uphill/Downhill:** ⬆️ 2 / ⬇️ 3
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/DIST-19.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | ABI drift silencioso | CI Electron real | 404/ABI error |
  | 🟡×🟡 | retrieve() cambia defaults | opt-in byte-idéntico | test golden rojo |
  | 🟢×🟡 | scopes sin enforcement | spec-first + tipos | — |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Épico `DIST-19` cubre `MEMG-03/09/24/25/26`, `FIND-320`, `DIST-20/21` (trazabilidad en task file).

### Task 11: SRV-10 — Cifrado server + consolidación workflows (CI-08)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Skills:** `documentation-skill`, `codebase-memory`, `progreso`, `ponytail`
- **Reglas área:** `.opencode/rules/server-mcp.md` + `.opencode/rules/release-ci.md` (workflows) + `.opencode/rules/open-core-licensing.md` si toca Pro.
- **Archivos clave:** `vantadb-server/`, `docs/api/HTTP_API.md`, `.github/workflows/`, `docs/dev/workflow/TRIGGERS.md`
- **Regla 0:** `codegraph` de `encryption`, rutas server + lectura de `on:` de cada workflow tocado + `TRIGGERS.md` antes de consolidar.
- **Verificación real:** ✅ CÓDIGO-REAL — `HTTP_API.md:701` declara SRV-10 roadmap; core ya tiene feature `encryption`; 40 workflows verificados (vs 28 en CI_POLICY → FIND-321).
- **Gate Justificación:** server sin cifrado en reposo; 40 workflows × matrices queman runners.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb-server` (e2e+security) verde + workflows ≤18 sin renombrar `release.yml`/`release-npm-61.yml`/`release-npm-node.yml` + `actionlint` 0 + `TRIGGERS.md` sincronizado
- **Idioma:** todo en español en el plan; `TRIGGERS.md` y docs workflow en inglés.
- **Cierre harness:** Clean + `/cleanCA` + `just verify` + `ocr-review.ps1` + `skill progreso` + commit local `vanta-lead`, sin push.
- **Pre-mortem:** 1. renombrar release file rompe OIDC (prohibición permanente). 2. consolidar demos rompe aceptación F5/F6. 3. `continue-on-error` sin CATEGORY.
- **Stop conditions:** consolidación toca publish path → abortar y acotar a informational/demo→nightly.
- **Cynefin:** 🟨 complicado — triggers + OIDC.
- **Top 3 riesgos:** 1. OIDC filename-bound. 2. matrices perdidas. 3. branch protection desactualizado.
- **Uphill/Downhill:** ⬆️ 1 / ⬇️ 2
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/SRV-10.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🔴×🔴 | rename release file | prohibido, checklist | diff con rename |
  | 🟡×🟡 | jobs huérfanos | migrar o documentar muerte | FIND-321 |
  | 🟢×🟡 | timeouts ausentes | `timeout-minutes` por job | RULES.md §2 |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** Cubre `CI-08` + `FIND-321` (inventario primero, consolidación después).

### Task 12: FIND-340 — Preparación release 0.8.0 sin push + docs finales

- **Appetite:** max 1d
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴
- **Skills:** `documentation-skill`, `progreso`, `ponytail`
- **Archivos clave:** `scripts/git-backup.ps1`, `scripts/verify-release.ps1`, `docs/user/QUICKSTART.md`, `docs/dev/FASE-A.md`, `docs/dev/architecture/ARCHITECTURE.md`
- **Regla 0:** leer `RULES.md §8` + `PUBLISH.md` (gate de release completo) + `rg` de pins viejos antes de editar docs.
- **Verificación real:** ✅ CÓDIGO-REAL — `git-backup.ps1` con `-Dest` + verify fail-closed (RULES.md §8); `ARCHITECTURE.md` stale v0.6.1 (FIND-322); pins 0.6.1 vs 0.7.0 live (DIST-08).
- **Gate Justificación:** decisión owner: solo-preparar; release no existe hasta verificado (PUBLISH.md).
- **Gate Result:** ✅ DO
- **Contrato:** `pwsh scripts/git-backup.ps1 -Dest <externo>` exit 0 + `git bundle verify` exit 0 + `verify-release.ps1 -Tag v0.8.0 -Smoke` ejecutado en local (sin push) + QUICKSTART pins alineados + ARCHITECTURE refresh scoped
- **Idioma:** plan en español; `QUICKSTART.md` y `ARCHITECTURE.md` se editan en inglés.
- **Cierre harness:** `skill progreso` + commit local `vanta-lead`, **push prohibido sin OK explícito**.
- **Pre-mortem:** 1. backup en mismo volumen (script advierte). 2. conteo `139 commits` puede variar — recontar. 3. FASE-A requiere humano (no automatizable).
- **Stop conditions:** bundle verify ≠0 → STOP, no seguir (R2 GitLab).
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** 1. backup mismo disco. 2. push accidental (prohibido sin OK). 3. pins docs vs registry.
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3
- **DoD task:** contrato ✅ · task file sync · recitation
- **Task file:** `docs/dev/tasks/FIND-340.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🔴×🔴 | push sin OK owner | prohibido Regla 7 | — |
  | 🟡×🔴 | backup no verificado | fail-closed + `git clone <bundle>` | verify ≠0 |
  | 🟢×🟡 | pins stale | DIST-08 | smoke falla |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** `PROC-01-push` queda 🔴 BLOQUEADO hasta tu `push` explícito. Carril negocio `BIZ-04/10/11`, `STRAT-01/02`, `FASE-A/R-05`, `BIZ-16/17` es humano y vive en `Backlog-negocio.md` — fuera de este plan. ID `FIND-340` libre verificado. Sin tocar versiones/CHANGELOG/tags (release-plz).

plan-adjust [2026-10-15]: revisión `.opencode` — `HYG-*/COG-*/SRV-CI/REL-PREP/EGO-P0/EGO-FULL` renombrados a taxonomía (`FIND-336..340` + épicos `SURF-01/API-10/DX-13/EGO-01/EGO-03/DIST-19/SRV-10`); añadidas `documentation-skill`, Regla 0, cierre Clean/`/cleanCA`/`just verify`/`ocr-review`, roles git y regla de idioma. Plan íntegro en español.
- ⬆️ uphill antes: destino SPEC/CLA/hooks sin decidir → después: locked (docs/specs, .github/cla, pre-commit único).
