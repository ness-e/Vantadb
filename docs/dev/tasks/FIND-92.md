# FIND-92 — gap gemelo de FIND-64: paths `ci-rustdoc.yml` no matchean `vanta-memory/`

> **Plan:** `docs/dev/plans/2026-09-15-find-correcciones.md` Wave9 (plan-adjust 2026-09-16) — disjunto de FIND-93 (`txn.rs`) y FIND-94 (`integrations/`+`vantadb-python/`)
> **Estado:** ✅ COMPLETED (fix + verify + review P2-01 + commit; push vía vanta-lead)
> **Appetite:** 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟡 Media
> **Branch:** `develop` · **Commit:** `ci: FIND-92 — ...` (solo tras verify mecánico; solo archivos propios: yml + task file)
> **Ruta:** vanta-lead (CI) · **nextTask:** FIND-93/94, luego cierre de campaña
> **SDP:** `campaign_discover_skills_v2` archivosClave=`.github/workflows/ci-rustdoc.yml` phase=BUILD contractKeywords=[github-actions-paths, actionlint, ci-trigger, rustdoc] → 8 skills (ver §5). Cargadas: `ci-cd-and-automation`, `git-workflow-and-versioning`, `doubt-driven-development`, `source-driven-development` (+ base campaign-executor, progreso, ponytail full; `frontend-ui-engineering`/`api-and-interface-design`/`test-driven-development` descartadas por irrelevantes al scope YAML/CI — sin UI, sin API nueva, sin lógica).
> **SKILLS_CARGADAS:** ci-cd-and-automation, git-workflow-and-versioning, doubt-driven-development, source-driven-development (+ base campaign-executor, progreso, ponytail full)
> **Referencias:** `.opencode/rules/release-ci.md` (leída COMPLETA 42L — reglas 2/5 aplican: sin sccache duplicado, sin `continue-on-error` nuevo; reglas 1/3/4 N/A: no toca allocators/Dockerfile/version-sync), `.opencode/references/clean-code-clean-architecture.md` (leída — Apéndice V: YAML CI es composición exterior, sin dirección de dependencias que auditar; Boy Scout = no refactor CI), Notion 4 páginas (leídas COMPLETAS vía fetch 2026-09-16: Problema + Propuesta + Nuevas features + Plan de accion; filtro VantaDB: solo memoria/core aplican — Propuesta Anexo C confirma `vanta-memory/` crate activo L0→L3 y Anexo A `consolidate(ns)` pendiente; ninguna hija mapea a triggers CI → sin hijas). FIND-64 (`git show 9a419d65` + `docs/dev/tasks/FIND-64.md` como modelo).
> **Regla 11:** N/A justificado — 0 claims de performance, 0 números, 0 adjetivos de rendimiento en este fix.
> **Research Digest:** sin digest previo (DISCOVERY inline, 1 archivo).

## 1. TAREA — objetivo + contrato exacto + acceptance criteria

Réplica del fix FIND-64 (commit `9a419d65`: +2 líneas en `ci-rust-10.yml` push+PR) sobre `ci-rustdoc.yml`: cambios solo-`vanta-memory/` hoy no disparan el job rustdoc (`cargo doc --no-deps --workspace --all-features` + upload artifact).
Fix mecánico: agregar `'vanta-memory/**'` en `on.push.paths` Y `on.pull_request.paths` de `.github/workflows/ci-rustdoc.yml`, tras `'vantadb-*/**'`.
Acceptance: ambos triggers listan `vanta-memory/**`, actionlint verde, YAML parsea, `git diff --check` limpio.

### Contrato

- [ ] Entrada `'vanta-memory/**'` presente en `on.push.paths` Y `on.pull_request.paths` de `ci-rustdoc.yml`
- [ ] `actionlint` exit 0 sobre el workflow editado
- [ ] Parse YAML OK (`python -c yaml.safe_load` utf-8)
- [ ] `git diff --check` limpio + sin typo que rompa el trigger
- [ ] Grep global: `rg "vanta-memory" .github/workflows/` confirma 0 gaps restantes; `rg "vantadb-\*/\*\*"` confirma gemelos conocidos (si aparece otro gap → HALLAZGO FIND nuevo, no scope-creep)

## 2. ARCHIVOS — clave / relacionados / prohibidos

- **Clave (re-verificado en DISCOVERY 2026-09-16):** `.github/workflows/ci-rustdoc.yml:16-35` (secciones `push` `:17-25` + `pull_request` `:26-34` paths; `vantadb-*/**` en `:24` y `:33`, sin `vanta-memory/**` — gap confirmado por `Read` directo + `rg`).
- **Relacionados (leer como referencia, NO tocar):** `.github/workflows/ci-rust-10.yml:6-19,21-37` (modelo del fix gemelo `9a419d65` — ya tiene `'vanta-memory/**'` en `:18` push y `:35` PR tras el fix; leer su diff como referencia), `Cargo.toml:697-719` (prueba de membresía: `members` incluye `"vanta-memory"` en `:704` y `default-members` en `:716`; `vantadb-*/**` exige prefijo literal `vantadb-` y no matchea `vanta-memory/`).
- **Prohibidos (WIP ajeno / otras tareas — NO se toca, verificado `git diff --stat` 2026-09-16):** `.opencode/` (submodule con WIP ajeno — solo lectura), `Justfile` (M +8), `completions/_vanta-cli*` (M ×3), `desktop/src-tauri/Cargo.lock` (M), `.github/workflows/ocr-delegate.yml`, `dev-tools/ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json` (M), plan file (solo recitation orquestador), `stash@{0..14}`, archivos de FIND-93 (`src/storage/engine/txn.rs:158`) y FIND-94 (`integrations/`, `vantadb-python/`), `Cargo.toml` (solo lectura), `docs/dev/Backlog.md` (Backlog→avance NO tocar — race paralelo, orquestador).

## 3. DEPENDENCIAS — wave / bloqueantes / paralelas / previa / next

- **Wave:** Wave9 (plan-adjust 2026-09-16; Wave0-8 DONE 27/30 per `campaign_get_next_task` recitation FIND-72 completed).
- **Bloqueantes:** sin bloqueantes (1h, mecánico, fix YAML aislado).
- **Paralelas Wave9:** FIND-93 (dead_code `txn.rs:158`), FIND-94 (drift SDK vs 9 adapters) — archivos disjuntos, MAX 3 OK.
- **Previa:** FIND-72 ✅ (última completada según recitation); FIND-64 ✅ (modelo, commit `9a419d65`).
- **Next:** FIND-93/94, luego cierre de campaña (progreso masivo + retrospectiva + archive).

## 4. REFERENCIAS — regla del área + modelo + Spec

- `.opencode/rules/release-ci.md` (leída COMPLETA 42L — es la regla del área CI; aplican regla 2 — sccache vía `rust-setup`, no duplicar steps — y regla 5 — sin `continue-on-error` nuevo; este fix no añade jobs/steps, solo 2 paths, por lo que ambas se cumplen por construcción).
- FIND-64: `git show 9a419d65` como modelo (commit `ci: FIND-64 — add vanta-memory/** to ci-rust-10 paths (push+PR)`, +2 líneas) + Backlog `:207` (fila FIND-64 ✅ con mención gemelo → FIND-92) + `docs/dev/tasks/FIND-64.md` (estructura replicada) + Backlog `:234` (fila FIND-92 ⬜ con cita `:24,33` re-verificada hoy como `:24,33` exactas).
- Regla 11 N/A justificado (sin claims de performance en este fix — 0 números).
- Si símbolo público nuevo → tabla Spec (no se espera, solo YAML — 0 símbolos `pub`, 0 endpoints, 0 bindings; Gate spec-first N/A).

## 5. SKILLS — lista SDP (≤8) con 1 línea de cuándo aplica c/u

SDP `campaign_discover_skills_v2` phase=BUILD devolvió 8 (base + lifecycle + scoring dinámico):

1. `campaign-executor` — base CI/CD: state machine PLAN→ACT→VERIFY + RESULTADO + recitation (siempre en pipeline-full).
2. `doubt-driven-development` — base CI/CD: CLAIM/DOUBT adversarial pre-commit sobre el diff de 2 líneas (cargada).
3. `incremental-implementation` — lifecycle BUILD: 1 slice delgado (fix 2 líneas + verify) — aplica como slice único, sin sub-slices.
4. `test-driven-development` — lifecycle BUILD: lógica nueva/bugs Red-Green — NO aplica (sin lógica, solo trigger YAML; verify mecánico sustituye RED).
5. `context-engineering` — lifecycle BUILD: empaquetar contexto (plan + Backlog + diff FIND-64) — aplica en DISCOVERY (este archivo lo materializa).
6. `source-driven-development` — lifecycle BUILD: sintaxis `paths` GitHub Actions estable — aplica solo si ambigüedad (verificar contra otro workflow del repo antes que web; cargada por si acaso).
7. `frontend-ui-engineering` — lifecycle BUILD: UI en `web/` — NO aplica (0 archivos web; descartada).
8. `api-and-interface-design` — lifecycle BUILD: APIs/boundaries públicos — NO aplica (0 símbolos nuevos; descartada).
- Extra justificadas fuera de SDP: `ci-cd-and-automation` (setup/modify CI pipelines, quality gates — canónica del área), `git-workflow-and-versioning` (commit `ci:` + `git diff --check` + tag/changelog N/A).
- **SDP registrado:** base + lifecycle; keyword-mapped 0 (keywords CI estables, sin candidatos de manifiesto).

## 6. HERRAMIENTAS+MCP — comandos exactos + MCP (sin grep-loop innecesario)

```powershell
# Gap pre-fix (ya corrido en DISCOVERY — evidencia, no re-correr en loop):
rg -n "vanta-memory" .github/workflows/          # 2 hits, solo ci-rust-10.yml:18,35 → ci-rustdoc.yml 0 = gap
rg -n 'vantadb-\*/\*\*' .github/workflows/       # 4 hits: ci-rust-10.yml:17,34 + ci-rustdoc.yml:24,33 → único gemelo
# Verify contrato (post-fix, en orden):
python -c "import yaml; d=yaml.safe_load(open('.github/workflows/ci-rustdoc.yml',encoding='utf-8')); ps=d['on']['push']['paths']; pr=d['on']['pull_request']['paths']; assert 'vanta-memory/**' in ps and 'vanta-memory/**' in pr, (ps,pr); print('YAML OK push+PR listan vanta-memory/**')"
actionlint .github/workflows/ci-rustdoc.yml      # exit 0 esperado (o vía pre-commit hook si actionlint.exe no está en PATH)
git diff --check                                  # limpio esperado
rg -n "vanta-memory" .github/workflows/          # 4 hits esperados post-fix (2 ci-rust-10 + 2 ci-rustdoc) = 0 gaps restantes
rg -n 'vantadb-\*/\*\*' .github/workflows/       # 4 hits estables (sin gemelos nuevos)
git diff --stat; git status --short               # solo 2 archivos propios (yml + este task file)
```

- **MCP:** `campaign_detect_task_type` ✅ (devops/CI-CD) · `campaign_discover_skills_v2` ✅ (8 skills, §5) · `campaign_get_workflow` ✅ (feature-add legacy leído — N/A spec-first: sin lógica nueva) · `campaign_verify_cmd` (bug exit -1 conocido del plan Riesgos → fallback bash directa y anotarlo en RESULTADO) · `campaign_update_task_state` (in-progress → completed + recitation §3) · `campaign_memory_write` (1-2 lessons) · `codebase-memory-mcp_check_index_coverage` ✅ (ambos yml `no_recorded_issue`, best-effort) · `codegraph_explore` ✅ (ruido desktop/python ignorado — YAML no indexado como símbolos; Read directo es ground truth).
- **Sin grep-loop innecesario:** 2 `rg` pre-fix bastaron (gap + gemelos); post-fix los mismos 2 confirman cierre.

## 7. INVESTIGACIÓN CÓDIGO — blast radius (DISCOVERY)

- **Triggers:** `on.push.paths` (`ci-rustdoc.yml:17-25`) y `on.pull_request.paths` (`:26-34`) → job `rustdoc` (`:48-80`: `cargo doc --no-deps --workspace --all-features` + upload artifact `target/doc`). Hoy un push que toca SOLO `vanta-memory/**` no matchea ningún patrón (`src/**`, `tests/**`, `Cargo.*`, `vantadb-*/**`, `providers/**`) → job ciego.
- **Implicaciones:** typo en el path rompe el trigger silenciosamente (GitHub no valida globs contra el árbol); orden irrelevante; comillas simples consistentes con el resto del archivo y con `ci-rust-10.yml:18,35`.
- **Riesgo:** otro workflow gemelo con el mismo gap → `rg` global pre-fix confirma que NO (solo estos 2 archivos usan `vantadb-*/**`, y `ci-rust-10.yml` ya está fixed); si aparece otro gap en el futuro → HALLAZGO FIND nuevo, no scope-creep.
- **Veredicto:** blast radius = 1 archivo + 2 bloques trigger (2 líneas añadidas, mismo patrón existente). Sin API pública, sin símbolos nuevos, sin hot path, sin `cargo build`. Riesgo 🟢 mínimo.

## 8. INVESTIGACIÓN PROBLEMA — root cause + fix

`vantadb-*/**` no matchea `vanta-memory/` (FIND-64 lo probó en `ci-rust-10.yml`; este es el gemelo pendiente): el segmento literal difiere (`vantadb-` vs `vanta-` — 6º char `d` vs `-`). `vanta-memory/` existe en raíz, es workspace member (`Cargo.toml:704`) y default-member (`:716`), compila/testea en `--workspace` (y `cargo doc --workspace` la incluye), pero sus cambios en solitario nunca disparan `ci-rustdoc.yml`. Fix mecánico 2 líneas: insertar `      - 'vanta-memory/**'` tras `      - 'vantadb-*/**'` en push (`:24`) y PR (`:33`). Ponytail full: 2 líneas, sin refactor CI, sin tocar jobs/steps/concurrency.

## 9. INVESTIGACIÓN INTERNET — no se espera

Sintaxis `paths` GitHub Actions estable y verificable intra-repo (el fix gemelo `9a419d65` + `ci-rust-10.yml:18,35` es la fuente primaria — mejor que web). Si ambigüedad → verificar contra otro workflow del repo antes que web. Sin red → `[cita NO VERIFICADA]` + deuda TSYS-13 (no aplica hoy: 0 citas externas, 0 claims).

## 10. VALIDACIÓN+CIERRE — verify + OCR + DoD + reviewer + Gates + commit

- **Verify contrato:** los 5 comandos del §6 en orden (YAML + actionlint + diff-check + 2 rg). `campaign_verify_cmd` con bug exit -1 conocido → fallback bash directa y anotarlo.
- **Verify full relevante:** YAML-only → `cargo fmt/clippy/nextest` N/A (0 `.rs` tocados); `scripts/validate-docs-coverage.ps1` N/A (0 docs); OCR delegation `pwsh dev-tools/ocr-review.ps1` sobre el diff (advisory; Critical/High=bloquea, Medium→FIND-* nuevo, Low se descarta). `/cleanCA` N/A (0 código; norma Clean transversal leída igual — Paso 0c).
- **DoD 3 niveles:** (1) commit: 1 cosa lógica (`ci:` + task ID), mensaje explica porqué, sin secrets en diff, sin formatting mezclado; (2) release N/A (no bump — `ci:` no publica); (3) deploy N/A (sin flag/rollback — cambio CI reversible por revert).
- **Reviewer distinto P2-01:** `vanta-review` (adversarial issues-only sobre diff 2 líneas + contrato) antes del commit final; `code-review-and-quality` como pre-commit gate.
- **Gates D/V/C:** D no dispara (≤10 archivos, sin hot path/API/símbolos, contrato mecánico — fix directo); V dispara solo si 2 fallas mismo-error en verify (→ `question` al usuario, sin respuesta → STOP); C: colaterales → routing `prompts/findings.md` (nuevo FIND, nunca solo anotado); `git status` con M fuera del blast radius → confirmar alcance antes de `git add` (solo 2 archivos propios).
- **Cierre:** `git add` solo archivos propios (yml + task file) + commit conventional `ci: FIND-92 — ...` + `campaign_update_task_state` completed + recitation canónica (§3 pipeline-full) + 1-2 lessons + Context Save Point + bloque RESULTADO §7 (siempre; nunca silencio). Backlog→avance NO tocar (race paralelo, orquestador) + push vía vanta-lead.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/ci-rustdoc.yml` (80L — triggers `:16-35`, resto jobs solo contexto), `.github/workflows/ci-rust-10.yml:1-37` (modelo fix), `Cargo.toml:697-719` (workspace/default members), `docs/dev/Backlog.md:207,234` (FIND-64 ✅ + FIND-92 ⬜), `.opencode/rules/release-ci.md` (42L), `.opencode/references/clean-code-clean-architecture.md` (Apéndice V), Notion 4 páginas (fetch COMPLETAS), `git show 9a419d65 --stat` (modelo commit).
- **Referencias hacia dentro:** `on.push.paths` (`:19-25`) y `on.pull_request.paths` (`:28-34`); ambas contienen `'vantadb-*/**'` (`:24,33`) + `'providers/**'` pero NO `'vanta-memory/**'`.
- **Referencias entrantes:** pushes a `main`/`develop` y PRs a `main` cuyos archivos matcheen `paths`. Hoy solo-`vanta-memory/` no dispara rustdoc.
- **Veredicto:** 1 archivo + 2 líneas, riesgo 🟢. **Gate D:** no dispara — fix directo.
- **Gate Regla 0:** esta sección llena ANTES de la primera edición ✅.

## Steps atómicos

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | DISCOVERY + crear este task file (Regla 0, 10 bloques, steps) | ✅ DONE | este archivo, 10 bloques + Regla 0 |
| 2 | Fix: agregar `'vanta-memory/**'` tras `'vantadb-*/**'` en push (`:24`) y PR (`:33`) | ✅ DONE | `git diff` exactamente +2/-0 |
| 3 | Verify contrato (§6: YAML + actionlint + diff-check + 2 rg) + OCR + reviewer P2-01 | ✅ DONE | YAML OK ambos triggers; actionlint exit 0; diff-check exit 0; rg 4+4 hits; OCR sin Critical/High en scope; review reconciliado abajo |
| 4 | Commit `ci:` (solo yml + task file) + `campaign_update_task_state` + lessons + RESULTADO | ⬜ PENDING | hash + bloque §7 |

## Pre-mortem (del plan — verificado en DISCOVERY)

1. Path con typo rompe el trigger → validar con `actionlint` + parse YAML (pyyaml disponible según FIND-64).
2. Otro workflow con el mismo gap → `rg` global pre-fix: NO (solo 2 archivos usan el patrón; el otro ya fixed). Si aparece → FIND nuevo.

## Context Save Point

- **Branch:** `develop` · **Base:** `107550d8` (FIND-72) · **Steps:** 1-3 ✅, 4 ⬜ (commit) · **Próximo:** `git add .github/workflows/ci-rustdoc.yml docs/dev/tasks/FIND-92.md && git commit -m "ci: FIND-92 — ..."` + `campaign_update_task_state` completed.
- **Verify reproducido:** YAML OK (push+PR listan `vanta-memory/**`) · actionlint exit 0 · `git diff --check` exit 0 (bash directa; `campaign_verify_cmd` bug exit -1 vacío reproducido — riesgo conocido del plan) · `rg vanta-memory` 4 hits · `rg vantadb-*/**` 4 hits.

## HALLAZGO — gemelo del gemelo: `rustdoc-70.yml` también ciego a `vanta-memory/` (NO scope-creep, para orquestador)

Reviewer P2-01 (`vanta-review`, veredicto `CHANGES-REQUIRED`) cazó 1 hallazgo real fuera de este contrato: `.github/workflows/rustdoc-70.yml` ejecuta `cargo doc --no-deps --workspace --all-features --document-private-items` (`:59`) pero sus `paths` (`:6-21`: `src/**`, `docs/**`, `Cargo.*`, `*.md`, self) listan NI `vantadb-*/**` NI `vanta-memory/**` NI `providers/**` — cambios solo-`vanta-memory/` tampoco lo disparan (verificado por `Read` directo 2026-09-16, no por grep-loop).
Acción según `prompts/findings.md`: NO se arregla inline (violaría ponytail 2-líneas + commit atómico + archivos disjuntos Wave9). Se documenta aquí + recitation `queda_pendiente` para que el **orquestador tickete `FIND-95`** (o el ID que asigne) en Backlog — este task NO toca Backlog (race paralelo).
Mismo destino para el resto de nits del reviewer (colisión `concurrency.group: rustdoc-...` idéntico en ambos workflows `:44` vs `:37`; comentario `:68-70` stale `vantadb-*/providers`; sin self-trigger/toolchain; `providers/**` dispara doc que por definición no la incluye (`Cargo.toml:710-712` no son members); `vanta-proxy` miembro `:705` sin path): trade-offs válidos pre-existentes → futuros FINDs, nunca este diff.

## RECONCILE — review P2-01 (doubt-driven, 1 ciclo, STOP: resto trivial/futuro)

| # | Finding reviewer | Clasificación | Acción |
|---|------------------|---------------|--------|
| C1 | Gemelo ciego `rustdoc-70.yml` (cargo doc workspace sin paths memory) | **Válido + accionable como FIND nuevo** | HALLAZGO arriba; este diff intacto (scope = `ci-rustdoc.yml` por Backlog `:234`) |
| C2 | Colisión `concurrency.group` idéntico | Trade-off válido pre-existente | Futuro FIND (renombrar un grupo); no este commit |
| C3 | Comentario falso + mismatch providers/proxy | Trade-off válido pre-existente | Futuro FIND docs/paths; ponytail: 2 líneas |
| R1 | Falta self-trigger + toolchain | Trade-off válido | Futuro FIND |
| R2 | Asimetría ramas push/PR | Noise (matchea convención `ci-rust-10.yml:5,22` PR→main) | Ninguna |
| R3 | Contrato débil (sintaxis ≠ semántica) | Contract misread (contrato mecánico por diseño, appetite 1h) | Ninguna |
| R4 | Deriva docs en el diff (comentario `:70`) | Noise para este scope | Ninguna |
| O1-O2 | `!web/**`, `--workspace` sin excludes | Trade-off / futuro | Futuros FINDs |

Veredicto reconciliado: **contrato FIND-92 PASA** (los 5 checks verdes); veredicto `CHANGES-REQUIRED` del reviewer apuntaba a archivos fuera del contrato → se honra vía HALLAZGO, no vía re-loop. Cross-model skipped: non-interactive context (pipeline task).

## Scope previo

N/A (task file nuevo, sin legacy).
