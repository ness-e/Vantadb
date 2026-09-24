# FIND-128: fix script ADR-Gate + matriz triggers push-vs-PR (duplicados)

## Metadata

- **Plan file:** docs/dev/plans/2026-09-19-cierre-total.md
- **Fuente:** plan cierre-total-182 (Wave A) + PR #182 checks post-push `cb954abc`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Tipo:** CI/CD (devops — bug-fix en workflow YAML, sin código Rust)
- **Turns estimados:** 8
- **Creado:** 2026-09-19
- **last-synced:** 2026-09-19
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 (6/6 steps ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub Actions runner (evento `pull_request` → job `adr-gate` en `ci-rust-10.yml`); PR #182 consume su conclusión como check requerido |
| Callees | `git diff --name-only BASE HEAD` (two-tree diff, depth-1 fetch); `$GITHUB_OUTPUT` file-command parser del runner; regex `^docs/dev/architecture/adr/ADR-[0-9]{3}` |
| Implicaciones | contrato no cambia (misma semántica: ADR presente/ausente); sin cambio de comportamiento público; sin impacto performance/memoria; sin migración; tests existentes no afectados (CI-only) |
| Riesgo | bajo (1 línea, solo rama `adr != ''` ya-verde se vuelve escribible) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rust-10.yml` (610 líneas: `on:` 1-38, `adr-gate` 128-190, resto skim jobs), `.github/workflows/gate-docs-21.yml` (87 líneas, completo), `.github/workflows/sec-codeql-30.yml` (37 líneas, completo), `.github/workflows/providers-ci.yml` (1-30 + resto skim), triggers `on:` de los 24 restantes (vía script `triggers.py`), `.opencode/rules/release-ci.md` (completo), `definition-of-done.md` (completo)
- **Archivos referenciados hacia dentro:** el job `adr-gate` no importa actions externas salvo `actions/checkout` (SHA-pinned); el script bash solo usa `git`, `grep`, `printf`, `paste` (coreutils, presente en `ubuntu-latest`)
- **Archivos que referencian a los editados:** ningún workflow llama a `ci-rust-10.yml` (`ci-gate.yml` es `workflow_call` sin callers en el repo — verificado por ausencia de `uses: ./.github/workflows/`); branch protection consume el check por nombre, no por contenido
- **Veredicto impacto:** bajo — editar SOLO `ci-rust-10.yml:162` (+ comentario). `gate-docs-21.yml` y resto: solo lectura para la matriz

## Contrato

"El step `Detect API surface + ADR changes` escribe `adr` en `$GITHUB_OUTPUT` como UNA línea (válido para el file-command parser con N ADRs cambiados); `actionlint` verde en `ci-rust-10.yml`; matriz triggers 28 workflows + propuesta dedup documentadas en este task file; commit `ci: FIND-128 — ...` selectivo sin push"

## Spec

N/A — bug-fix CI, Phase 1b negativa: no agrega `pub fn`, tools, endpoints ni métodos de binding. Sin Gate P/D.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) semántica del gate: API-change sin ADR sigue fallando; `[no-adr]` sigue en warning; (2) WIP ajeno intocable (`git status` muestra D/M de ISSUE_TEMPLATE, dependabot, Backlog, etc. — NO incluir en el commit); (3) prohibidos del contrato: `src/`, bindings, `web/src/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, `docs/dev/Backlog.md`, plan recitation, secretos
- **Comandos de verificación:** simulación bash old-vs-new con input multiline real + `actionlint .github/workflows/ci-rust-10.yml` (o evidencia de instalación fallida como deuda) + `git status --short` antes del commit
- **Deuda pendiente:** re-run real del workflow requiere push (prohibido en esta tarea) → queda como `queda_pendiente` para Wave B/orquestador

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva (1 línea + comentario; elimina el fallo, no agrega superficie). No aplica moneda P2 (CI-only).

## Definition of Done (niveles aplicables)

- **Task:** contrato arriba ✅ (simulación + actionlint + matriz + propuesta escrita)
- **Commit:** atómico (~5 líneas), `ci:` conventional + ID, solo 2 archivos (`ci-rust-10.yml` + este task file), verificación mecánica previa
- **Release:** N/A (tarea CI sin release; justificar: no toca versionado ni packaging)

## Herramientas necesarias

- `gh run view <id> --job <jid> --log` (lectura logs reales — evidencia causa raíz)
- `actionlint` (lint del workflow tocado)
- `git` (diff selectivo, commit sin push)
- `campaign_verify_cmd` (contrato; bug exit -1 → bash + mención)

**Skills cargadas (SDP):** ci-cd-and-automation (quality gates + feedback loop CI) · git-workflow-and-versioning (commit atómico `ci:`, sin push) · documentation-and-adrs (matriz triggers + propuesta dedup como decisión escrita) · systematic-debugging (Iron Law: repro + hipótesis antes del fix) · doubt-driven-development (gate review adversarial 1-línea). SDP v2 además sugirió lifecycle genéricas (incremental/test-driven/context/source-driven, frontend-ui, api-design) — descartadas: CI YAML sin lógica nueva ni UI ni API (ponytail: no cargar por cargar).

## Investigation Notes

- **Root cause (evidencia alta):** `gh run view 35467546726 --job 105962550017 --log` → step `Detect API surface + ADR changes` imprime `--- changed files ---` y el runner emite `##[error]Unable to process file command 'output' successfully.` + `##[error]Invalid format 'docs/dev/architecture/adr/ADR-015-coverage-policy.md'`. Causa: PR #182 cambió 40 ficheros bajo `docs/dev/architecture/adr/` (30 matchean `ADR-[0-9]{3}`) → `$adr` multiline → `echo "adr=$adr" >> $GITHUB_OUTPUT` escribe líneas 2..N sin formato `name=value` → el parser del runner falla. La línea 1 (`adr=...ADR-014...`) es válida; la 2ª (`docs/...ADR-015...`) dispara el error. Fix: `| paste -sd' ' -` (una línea, `paste` es coreutils en `ubuntu-latest`).
- **Ruido secundario (no fix):** `printf: write error: Broken pipe` — `grep -q` cierra el pipe tras el primer match; stderr cosmético, exit 0. Se deja (cambio mínimo).
- **Duplicados (evidencia alta):** mismo commit `cb954abc` en `develop` disparó 2 runs con 6s de diferencia: `35467540846` (`push`, success) y `35467546726` (`pull_request` PR #182, failure). Par completo en ci-rust-10: push-verde vs PR-rojo = confusión pass/fail. `github.ref` difiere (`refs/heads/develop` vs `refs/pull/182/merge`) → `concurrency.group: ${{ workflow }}-${{ ref }}` NO los cancela entre sí. Workflows con doble disparo en push-develop con PR abierto (paths mediante): chaos-45, ci-examples-12, ci-rust-10, ci-rustdoc, ci-web-11, desktop, gate-docs-21, providers-ci (push sin branches!), rustdoc-70 (PR incluye develop).
- **Conteo CI_POLICY:** `docs/user/operations/CI_POLICY.md:17` dice 26; real 28 (incluye `ci-gate.yml` + `opencode.yml` probablemente). NO se toca (cero refactors) — propuesta escrita abajo.
- Web research: N/A (formato `$GITHUB_OUTPUT` es conocimiento estable del runner + log local como evidencia; task lo declara).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes de ejecución | 4 (Step 3 fix, Step 4 evidencia, Step 5 actionlint, Step 6 commit+RESULTADO) |
| % completado | 100% (Steps 1-6 ✅) |

## Fase 1 — Evidencia de Debugging (GATE Bug)

- **Repro:** determinístico sin red extra: con N≥2 ADRs en el diff, `adr="a\nb"` + `echo "adr=$adr" >> $GITHUB_OUTPUT` → línea 2 sin `=` → runner `Invalid format`. Repro local = simular el `echo` con el input real (30 paths) y mostrar multiline (old) vs single-line (new).
- **Hipótesis (pre-fix):** el file-command parser exige `name=value` por línea; valor multiline sin heredoc `<<EOF` rompe el step. Escrita ANTES de editar.
- **1 variable controlada:** solo línea 162 (`grep ... || true` → `grep ... | paste -sd' ' - || true`) + comentario. `api` ya es single-value (break). Nada más.
- **Test RED:** log del run fallido (job 105962550017) = RED verificado en CI real; simulación local old=multi-line (inválido) / new=single-line (válido).

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — N/A (justificación): el cambio no toca `permissions:`, secretos, ni inputs de usuario; el `git diff` es de commits ya pusheados, no input libre. Sin `security-and-hardening` (YAML CI de 1 línea, sin trust boundary nuevo).
- [x] **PERFORMANCE** — N/A (justificación): no toca hot paths (HNSW/engine/search); `paste` es O(n) en <100 líneas. Sin baseline (Regla 9 no aplica a YAML).

## Steps

### Step 1: Discovery matriz + root cause

- **Archivos:** `.github/workflows/*.yml` (28, lectura triggers), log run 35467546726
- **Acción:** extraer matriz `on:`, confirmar causa ADR-Gate con `gh`, confirmar duplicados push-vs-PR
- **Verify:** `gh run view ... --log | Select-String "Invalid format|Unable to process"` con las 2 líneas + par de runs mismo SHA
- **Estado:** ✅ COMPLETED

### Step 2: Crear task file

- **Archivos:** `docs/dev/tasks/FIND-128.md`
- **Acción:** DISCOVERY completo (este archivo)
- **Verify:** existe + secciones Regla 0/Fase 1/Contrato pobladas
- **Estado:** ✅ COMPLETED

### Step 3: Fix ADR-Gate (1 línea)

- **Archivos:** `.github/workflows/ci-rust-10.yml:162`
- **Acción:** `grep -E '...' || true` → `grep -E '...' | paste -sd' ' - || true` + comentario de 3 líneas (multiline `$GITHUB_OUTPUT` inválido, ref run 35467546726)
- **Verify:** `git diff -- .github/workflows/ci-rust-10.yml` muestra solo ese hunk
- **Estado:** ✅ COMPLETED

### Step 4: Evidencia local old-vs-new

- **Archivos:** ninguno (bash efímero)
- **Acción:** simular con 2+ paths ADR reales: old escribe 2 líneas (inválido), new escribe 1 (válido)
- **Verify:** `campaign_verify_cmd` (si bug exit -1 → bash directo + mención)
- **Estado:** ✅ COMPLETED (bash: OLD=2 líneas INVALID incl. ADR-015 idéntico al log; NEW=1 línea VALID; empty=`adr=` correcto; `campaign_verify_cmd` devolvió error de resolución de plan —sin planFile en schema— documentado, bash hace fe)

### Step 5: actionlint archivo tocado

- **Archivos:** `.github/workflows/ci-rust-10.yml`
- **Acción:** `actionlint` (instalar release rhysd/actionlint si falta; 1 intento; si falla → deuda documentada)
- **Verify:** exit 0 en el archivo
- **Estado:** ✅ COMPLETED (`actionlint` v1.7.12 preinstalado, exit 0 sin findings)

### Step 6: Commit selectivo + RESULTADO

- **Archivos:** `.github/workflows/ci-rust-10.yml`, `docs/dev/tasks/FIND-128.md`
- **Acción:** `git add` SOLO esos 2 + `git commit -m "ci: FIND-128 — ..."` (SIN push); bloque RESULTADO §7 + Gates D/V/C
- **Verify:** `git status --short` no incluye WIP ajeno; `git log --oneline -1`
- **Estado:** ✅ COMPLETED

## Dependencias

- Wave A paralelo (FIND-133 + CODEX disjuntos — archivos no solapados, verificado en plan). Sin bloqueantes.
- NextTask: Wave B (orquestador: FIND-129 + WIN-flaky + cierre; incluye re-run/push + `progreso` + archive — fuera de scope aquí).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development self-gate (degradado declarado): la skill excluye "one-line changes with obvious correctness" de su ciclo completo; se aplicó escrutinio dirigido: (a) ¿`paste` preserva `adr != ''`? vacío→`adr=` (falsy ✅), N paths→1 línea no vacía (truthy ✅); (b) ¿alternativa heredoc? más líneas, mismo efecto — ponytail elige `paste`; (c) ¿`api` necesita lo mismo? No: single-value por `break`. Cross-model skipped: contexto no-interactivo (anunciado, no silencioso)
- **Enfoque:** ¿`paste -sd` preserva la semántica `adr != ''`? Sí: vacío→vacío, N paths→1 línea no vacía. ¿Alternativa heredoc `<<EOF`? Más líneas, mismo efecto — ponytail elige `paste`.
- **Cómo se probó:** log CI real (RED) + simulación local (old inválido/new válido) + actionlint
- **Checklist anti-hábitos tóxicos:** ✅ verificado en Step 6 — salidas 100% reales (gh log, sim bash, actionlint exit 0); `campaign_verify_cmd` devolvió error de resolución de plan (sin planFile en schema) y se documentó en vez de ocultarse; múltiples fuentes (log + diff + run list + workflow list); citas con run/job IDs; steps conectados al contrato; sin loops, sin degradación de errores, sin presupuesto infinito
- **Veredicto:** ✅ approve (doubt self-gate dirigido + actionlint mecánico; review-distinto con re-run real queda en Wave B con push)

## Notas

### Matriz triggers (28 workflows) — push-develop vs PR-main

| # | Workflow | push | pull_request | Otros | Doble disparo en push-develop+PR182? |
|---|----------|------|--------------|-------|--------------------------------------|
| 1 | adapters-compat.yml | — | — | schedule dom 03:00, dispatch | No |
| 2 | arch-metrics-informational.yml | — | paths `src/**`,Cargo* (sin branches = toda PR) | dispatch | No (solo PR) |
| 3 | bench-canonical-p99-informational.yml | — | paths `src/index/**`,`src/storage/**`,Cargo* | dispatch | No (solo PR) |
| 4 | chaos-45.yml | [main,develop] paths src/tests/Cargo/nextest/self | [main] mismos paths | dispatch | **SÍ** |
| 5 | ci-examples-12.yml | [main,develop] paths examples/src/Cargo/python/self | [main] idem | dispatch | **SÍ** |
| 6 | ci-gate.yml | — | — | `workflow_call` (reusable; sin callers en repo) | No |
| 7 | ci-rust-10.yml | [main,develop] paths amplios + `!web/**` | [main] idem | dispatch | **SÍ** (par 35467540846/35467546726) |
| 8 | ci-rustdoc.yml | [main,develop] paths src/tests/Cargo/vantadb-*/vanta-memory/providers | [main] idem | dispatch | **SÍ** |
| 9 | ci-web-11.yml | [main,develop] paths `web/**` | [main] idem | dispatch | **SÍ** (si el push toca web/) |
| 10 | desktop.yml | [main,develop] paths desktop/src/server/Cargo/self | [main] idem | dispatch | **SÍ** |
| 11 | fuzz-40.yml | — | paths `src/**`,`fuzz/**` (toda PR) | schedule lun 06:00, dispatch | No |
| 12 | gate-docs-21.yml | [main,develop] paths `docs/**`,router/routing,scripts | [main] idem | dispatch | **SÍ** (docs → casi siempre) |
| 13 | heavy-bench-nightly-51.yml | — | paths benches/benchmarks/scripts | schedule diario 03:00, dispatch | No |
| 14 | heavy-certification-50.yml | — | — | schedule dom 03:00, dispatch | No |
| 15 | ocr-delegate.yml | — | (toda PR, sin filtro) | dispatch | No (solo PR) |
| 16 | ocr-nightly.yml | — | — | schedule diario 04:00, dispatch | No |
| 17 | opencode.yml | — | — | issue_comment + review_comment | No |
| 18 | perf-bench-40.yml | [main,develop] paths src/python/benchmarks/Cargo | — | dispatch | No (solo push) |
| 19 | providers-ci.yml | paths `providers/**`+self (**sin branches = toda rama**) | paths idem | dispatch (primero) | **SÍ** (y en cualquier rama) |
| 20 | release-adapters-62.yml | tags `adapters-v*` | — | dispatch | No |
| 21 | release-binaries-63.yml | tags `v*` (+`branches:[main]`+paths en release-npm-61, inválido-combinado) | — (+PR paths solo en release-npm-61) | release published, dispatch | No |
| 22 | release-npm-61.yml | tags `v*.*.*` (+branches+paths) | paths wasm/ts/self | dispatch | Parcial (PR sí si toca wasm/ts; push-develop no por branches+tags) |
| 23 | release-npm-node.yml | tags `node-v*` (+branches main+paths) | — | dispatch | No |
| 24 | release-sbom-64.yml | tags `v*` | — | dispatch | No |
| 25 | release-wheels-60.yml | tags `v*.*.*` | [main] paths src/python/Cargo/self | dispatch | No (PR solo; push solo por tag) |
| 26 | release.yml | [main,develop] (sin paths) | — | (release-plz) | No (solo push; pero corre en CADA push a develop) |
| 27 | rustdoc-70.yml | [develop] paths src/docs/Cargo/vanta-memory/*.md/self | **[main,develop]** mismos paths | dispatch | **SÍ** (único con PR→develop) |
| 28 | sec-codeql-30.yml | [main] | [main] | schedule dom, dispatch | No en develop (push-develop no matchea; 1 run vía PR). El fail CodeQL del plan es init/permiso, no triggers |

**Lectura push-vs-PR:** con PR #182 abierto (develop→main), cada `push` a develop con paths coincidentes genera run `push` (ref `refs/heads/develop`) + run `pull_request` (ref `refs/pull/182/merge`). `concurrency.group = workflow-ref` no los cruza → duplicados visibles como pass/pass o pass/fail según qué run se mire.

### Propuesta dedup mínima (ESCRITA — no aplicar; cero refactors CI)

1. **Quitar `develop` de `push.branches`** en los 7 con par push[main,develop]+PR[main] (chaos-45, ci-examples-12, ci-rust-10, ci-rustdoc, ci-web-11, desktop, gate-docs-21): la validación pre-merge ya la cubre el evento PR; `push[main]` cubre post-merge. 1 línea por archivo, en Wave B con re-run propio.
2. **providers-ci:** añadir `branches: [main, develop]` al `push` (hoy dispara en cualquier rama + Dependabot) o mover validación a PR-only. Decisión pendiente del orquestador (cambia cobertura de ramas cortas).
3. **rustdoc-70:** alinear PR branches a `[main]` si el flujo es develop→main siempre, o documentar por qué develop es target válido.
4. **release-npm-61 / release-npm-node `push.tags + branches + paths`:** combinación inválida (tags ignoran branches/paths) — separar en 2 bloques `push:` o documentar intención. Solo propuesta (release-sensitive).
5. **CI_POLICY.md:17 `26` → `28`** + inventario (falta `ci-gate.yml`/`opencode.yml` u otros 2): hacerlo cuando se toque ese archivo (no ahora).
6. **No propuesto:** concurrency unificada por SHA, `paths-ignore`, ni migrar a `merge_group` — refactors, fuera del contrato mínimo.

## Review P2-01 (transcripción dictamen revisor distinto, 2026-09-19 — VEREDICTO: approve)
- Self-gate degradado declarado por implementador (sin question tool) — subsanado por este dictamen distinto, no re-abrir fix.
- Revisor verificó: `ddd9d58c` diff 1 línea + comentario; sim OLD 2-líneas INVALID / NEW 1-línea VALID; `actionlint` EXIT 0; matriz triggers coherente con `on:` reales (push[main,develop]+PR[main] → duplicados mismo SHA); cero refactors CI.
