# FIND-95 — gemelo-del-gemelo FIND-92: paths `rustdoc-70.yml` ciegos a `vanta-memory/`

> **Plan:** `docs/dev/plans/2026-09-15-find-correcciones.md` Wave9-ext (orquestador 2026-09-16) — disjunto de FIND-92 (`ci-rustdoc.yml` ✅ e395b563), FIND-93 (`txn.rs` ✅ 0cd54e47), FIND-94 (`integrations/`+`vantadb-python/` ✅ 880cd0f3). Última task antes del cierre de campaña.
> **Estado:** ⬜ PENDING → IN PROGRESS (fix + verify + review P2-01 + commit; push vía vanta-lead)
> **Appetite:** 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja
> **Branch:** `develop` · **Commit:** `ci: FIND-95 — ...` (solo tras verify mecánico; solo archivos propios: yml + task file)
> **Ruta:** vanta-lead (CI) · **nextTask:** CIERRE-CAMPAÑA (orquestador)
> **SDP:** `campaign_discover_skills_v2` archivosClave=`.github/workflows/rustdoc-70.yml` phase=BUILD contractKeywords=[github-actions-paths, actionlint, ci-trigger, rustdoc] → 8 skills (ver §5). Cargadas: `ci-cd-and-automation`, `git-workflow-and-versioning`, `doubt-driven-development`, `source-driven-development` (+ base campaign-executor, progreso, ponytail full; `frontend-ui-engineering`/`api-and-interface-design`/`test-driven-development` descartadas por irrelevantes al scope YAML/CI — sin UI, sin API nueva, sin lógica; `incremental-implementation`/`context-engineering` aplican como slice único/contexto materializado sin carga separada).
> **SKILLS_CARGADAS:** ci-cd-and-automation, git-workflow-and-versioning, doubt-driven-development, source-driven-development (+ base campaign-executor, progreso, ponytail full)
> **Referencias:** `.opencode/rules/release-ci.md` (leída COMPLETA 42L — reglas 2/5 aplican: sin sccache duplicado, sin `continue-on-error` nuevo; reglas 1/3/4 N/A: no toca allocators/Dockerfile/version-sync), `.opencode/references/clean-code-clean-architecture.md` (leída — Apéndice V: YAML CI es composición exterior, sin dirección de dependencias que auditar; Boy Scout = no refactor CI), Notion 4 páginas (leídas COMPLETAS vía fetch 2026-09-16: Problema + Propuesta + Nuevas features + Plan de accion; filtro VantaDB: solo memoria/core aplican — Propuesta Anexo C confirma `vanta-memory/` crate activo L0→L3 y Anexo A `consolidate(ns)` pendiente; ninguna hija mapea a triggers CI → sin hijas). FIND-92 (`git show e395b563` + `docs/dev/tasks/FIND-92.md` §HALLAZGO como especificación).
> **Regla 11:** N/A justificado — 0 claims de performance, 0 números, 0 adjetivos de rendimiento en este fix.
> **Research Digest:** sin digest previo (DISCOVERY inline, 1 archivo).

## 1. TAREA — objetivo + contrato exacto + acceptance criteria

Réplica del fix FIND-92 (commit `e395b563`: +2 líneas en `ci-rustdoc.yml` push+PR) sobre `rustdoc-70.yml`: cambios solo-`vanta-memory/` hoy no disparan el job rustdoc (`cargo doc --no-deps --workspace --all-features --document-private-items` en `:59` + upload artifact).
Fix mecánico mínimo: agregar `'vanta-memory/**'` en `on.push.paths` Y `on.pull_request.paths` de `.github/workflows/rustdoc-70.yml`, tras `'Cargo.lock'` (no hay ancla `'vantadb-*/**'` en este archivo — ver §8). Sin re-arquitectar paths.
Acceptance: ambos triggers listan `vanta-memory/**`, actionlint verde, YAML parsea, `git diff --check` limpio.

### Contrato

- [ ] Entrada `'vanta-memory/**'` presente en `on.push.paths` Y `on.pull_request.paths` de `rustdoc-70.yml`
- [ ] `actionlint` exit 0 sobre el workflow editado
- [ ] Parse YAML OK (`python -c yaml.safe_load` utf-8)
- [ ] `git diff --check` limpio + sin typo que rompa el trigger
- [ ] Grep global: `rg "vanta-memory" .github/workflows/` confirma 0 gaps restantes; `rg "vantadb-\*/\*\*"` confirma gemelos conocidos (si aparece otro gap → HALLAZGO FIND nuevo, no scope-creep)

## 2. ARCHIVOS — clave / relacionados / prohibidos

- **Clave (re-verificado en DISCOVERY 2026-09-16):** `.github/workflows/rustdoc-70.yml:6-21` (secciones `push` `:4-12` + `pull_request` `:13-21` paths; NI `'vantadb-*/**'` NI `'providers/**'` NI `'vanta-memory/**'` — gap confirmado por `Read` directo + `rg vanta-memory` 0 hits en ese archivo). Job `cargo doc --workspace` en `:59`.
- **Relacionados (leer como referencia, NO tocar):** `.github/workflows/ci-rustdoc.yml:16-36` (modelo del fix gemelo `e395b563` — ya tiene `'vanta-memory/**'` en `:25` push y `:35` PR tras el fix; leer su diff como referencia), `Cargo.toml:704,716` (prueba de membresía: `members` + `default-members` incluyen `"vanta-memory"`; `cargo doc --workspace` la incluye).
- **Prohibidos (WIP ajeno / otras tareas — NO se toca, verificado `git status --short` 2026-09-16):** `.opencode/` (submodule con WIP ajeno — solo lectura), `Justfile` (M), `completions/_vanta-cli*` (M ×3), `desktop/src-tauri/Cargo.lock` (M), `.github/workflows/ocr-delegate.yml` (untracked ajeno), `dev-tools/ocr-review.ps1` (untracked ajeno), `reparacion.bat` (untracked ajeno), `docs/pipeline-state.json` (M), plan file (solo recitation orquestador), `stash@{0..14}`, archivos de FIND-92/93/94 (`ci-rustdoc.yml`, `src/storage/engine/txn.rs:158`, `integrations/`, `vantadb-python/`), `Cargo.toml` (solo lectura), `docs/dev/Backlog.md` (Backlog→avance NO tocar — race paralelo, orquestador).

## 3. DEPENDENCIAS — wave / bloqueantes / paralelas / previa / next

- **Wave:** Wave9-ext (orquestador 2026-09-16; Wave0-9 DONE 30/30 per `campaign_get_next_task` recitation FIND-94 completed).
- **Bloqueantes:** sin bloqueantes (1h, mecánico, fix YAML aislado).
- **Paralelas Wave9-ext:** ninguna (última task antes del cierre de campaña).
- **Previa:** FIND-94 ✅ (última completada según recitation, commit `880cd0f3`); FIND-92 ✅ (modelo, commit `e395b563`); FIND-93 ✅ (`0cd54e47`); FIND-64 ✅ (origen, commit `9a419d65`).
- **Next:** CIERRE-CAMPAÑA (progreso masivo + retrospectiva + archive — orquestador).

## 4. REFERENCIAS — regla del área + modelo + Spec

- `.opencode/rules/release-ci.md` (leída COMPLETA 42L — es la regla del área CI; aplican regla 2 — sccache vía `rust-setup`, no duplicar steps — y regla 5 — sin `continue-on-error` nuevo; este fix no añade jobs/steps, solo 2 paths, por lo que ambas se cumplen por construcción).
- FIND-92: `git show e395b563` como modelo (commit `ci: FIND-92 — add vanta-memory/** to ci-rustdoc paths (push+PR)`, +2 líneas) + `docs/dev/tasks/FIND-92.md` §HALLAZGO (especificación literal de este gap: `rustdoc-70.yml:59` + paths `:6-21`) + Backlog `:237` (fila FIND-95 ⬜ creada por orquestador 2026-09-16).
- Regla 11 N/A justificado (sin claims de performance en este fix — 0 números).
- Si símbolo público nuevo → tabla Spec (no se espera, solo YAML — 0 símbolos `pub`, 0 endpoints, 0 bindings; Gate spec-first N/A).

## 5. SKILLS — lista SDP (≤8) con 1 línea de cuándo aplica c/u

SDP `campaign_discover_skills_v2` phase=BUILD devolvió 8 (base + lifecycle + scoring dinámico):

1. `campaign-executor` — base CI/CD: state machine PLAN→ACT→VERIFY + RESULTADO + recitation (siempre en pipeline-full).
2. `doubt-driven-development` — base CI/CD: CLAIM/DOUBT adversarial pre-commit sobre el diff de 2 líneas (cargada).
3. `incremental-implementation` — lifecycle BUILD: 1 slice delgado (fix 2 líneas + verify) — aplica como slice único, sin sub-slices.
4. `test-driven-development` — lifecycle BUILD: lógica nueva/bugs Red-Green — NO aplica (sin lógica, solo trigger YAML; verify mecánico sustituye RED).
5. `context-engineering` — lifecycle BUILD: empaquetar contexto (plan + Backlog + diff FIND-92) — aplica en DISCOVERY (este archivo lo materializa).
6. `source-driven-development` — lifecycle BUILD: sintaxis `paths` GitHub Actions estable — aplica solo si ambigüedad (verificar contra otro workflow del repo antes que web; cargada por si acaso).
7. `frontend-ui-engineering` — lifecycle BUILD: UI en `web/` — NO aplica (0 archivos web; descartada).
8. `api-and-interface-design` — lifecycle BUILD: APIs/boundaries públicos — NO aplica (0 símbolos nuevos; descartada).
- Extra justificadas fuera de SDP: `ci-cd-and-automation` (setup/modify CI pipelines, quality gates — canónica del área), `git-workflow-and-versioning` (commit `ci:` + `git diff --check` + tag/changelog N/A).
- **SDP registrado:** base + lifecycle; keyword-mapped 0 (keywords CI estables, sin candidatos de manifiesto).

## 6. HERRAMIENTAS+MCP — comandos exactos + MCP (sin grep-loop innecesario)

```powershell
# Gap pre-fix (ya corrido en DISCOVERY — evidencia, no re-correr en loop):
rg -n "vanta-memory" .github/workflows/          # 4 hits, 0 en rustdoc-70.yml = gap
rg -n 'vantadb-\*/\*\*' .github/workflows/       # 4 hits: ci-rust-10.yml:17,34 + ci-rustdoc.yml:24,34 → rustdoc-70.yml sin gemelo conocido
# Verify contrato (post-fix, en orden):
python -c "import yaml; d=yaml.safe_load(open('.github/workflows/rustdoc-70.yml',encoding='utf-8')); ps=d['on']['push']['paths']; pr=d['on']['pull_request']['paths']; assert 'vanta-memory/**' in ps and 'vanta-memory/**' in pr, (ps,pr); print('YAML OK push+PR listan vanta-memory/**')"
actionlint .github/workflows/rustdoc-70.yml      # exit 0 esperado (o vía pre-commit hook si actionlint.exe no está en PATH)
git diff --check                                  # limpio esperado
rg -n "vanta-memory" .github/workflows/          # 6 hits esperados post-fix (2 ci-rust-10 + 2 ci-rustdoc + 2 rustdoc-70) = 0 gaps restantes
rg -n 'vantadb-\*/\*\*' .github/workflows/       # 4 hits estables (sin gemelos nuevos)
git diff --stat; git status --short               # solo 2 archivos propios (yml + este task file)
```

- **MCP:** `campaign_detect_task_type` ✅ (devops/CI-CD) · `campaign_discover_skills_v2` ✅ (8 skills, §5) · `campaign_get_workflow` ✅ (bug-fix legacy leído — localizing→planning→implementing→testing→review→accept→close; N/A spec-first: sin lógica nueva) · `campaign_verify_cmd` (bug exit -1 conocido del plan Riesgos → fallback bash directa y anotarlo en RESULTADO) · `campaign_update_task_state` (in-progress → completed + recitation §3) · `campaign_memory_write` (1-2 lessons) · `codebase-memory-mcp_check_index_coverage` ✅ (ambos yml `no_recorded_issue`, best-effort) · `codegraph_explore` ✅ (ruido wal.rs/path ignorado — YAML no indexado como símbolos; Read directo es ground truth).
- **Sin grep-loop innecesario:** 2 `rg` pre-fix bastaron (gap + gemelos); post-fix los mismos 2 confirman cierre.

## 7. INVESTIGACIÓN CÓDIGO — blast radius (DISCOVERY)

- **Triggers:** `on.push.paths` (`rustdoc-70.yml:4-12`) y `on.pull_request.paths` (`:13-21`) → job `rustdoc` (`:32-83`: `cargo doc --no-deps --workspace --all-features --document-private-items` + tar + upload artifact `rustdoc-html.tar.gz`). Hoy un push que toca SOLO `vanta-memory/**` no matchea ningún patrón (`src/**`, `docs/**`, `Cargo.*`, `*.md`, self) → job ciego.
- **Implicaciones:** typo en el path rompe el trigger silenciosamente (GitHub no valida globs contra el árbol); orden irrelevante; comillas simples consistentes con el resto del archivo y con `ci-rustdoc.yml:25,35` + `ci-rust-10.yml:18,35`.
- **Riesgo:** otro workflow gemelo con el mismo gap → `rg` global pre-fix confirma que NO (solo `ci-rust-10.yml` y `ci-rustdoc.yml` usan `vantadb-*/**`, ambos ya fixed; `rustdoc-70.yml` ni siquiera tiene ese patrón); si aparece otro gap en el futuro → HALLAZGO FIND nuevo, no scope-creep. Gaps NOTICED en FIND-92 §HALLAZGO/RECONCILE (`providers/**`, colisión `concurrency.group`, comentario stale, `vanta-proxy` sin path) → futuros FINDs, NO este diff.
- **Veredicto:** blast radius = 1 archivo + 2 bloques trigger (2 líneas añadidas, mismo patrón existente). Sin API pública, sin símbolos nuevos, sin hot path, sin `cargo build`. Riesgo 🟢 mínimo.

## 8. INVESTIGACIÓN PROBLEMA — root cause + fix

Mismo segmento literal (`vantadb-` vs `vanta-`) que FIND-64/92, agravado: `rustdoc-70.yml` ni siquiera lista `'vantadb-*/**'` — sus paths cubren solo raíz (`src/**`, `docs/**`, `Cargo.*`, `*.md`, self). `vanta-memory/` existe en raíz, es workspace member (`Cargo.toml:704`) y default-member (`:716`), incluida en `cargo doc --workspace`, pero sus cambios en solitario nunca disparan `rustdoc-70.yml`. Fix mecánico mínimo 2 líneas: insertar `      - 'vanta-memory/**'` tras `      - 'Cargo.lock'` en push (`:10`) y PR (`:19`) — ancla elegida por agrupar paths de código antes de globs de docs (`*.md`) y del self-trigger; orden irrelevante para GitHub. Ponytail full: 2 líneas, sin re-arquitectar paths, sin tocar jobs/steps/concurrency.

## 9. INVESTIGACIÓN INTERNET — no se espera

Sintaxis `paths` GitHub Actions estable y verificable intra-repo (el fix gemelo `e395b563` + `ci-rustdoc.yml:25,35` es la fuente primaria — mejor que web). Si ambigüedad → verificar contra otro workflow del repo antes que web. Sin red → `[cita NO VERIFICADA]` + deuda TSYS-13 (no aplica hoy: 0 citas externas, 0 claims).

## 10. VALIDACIÓN+CIERRE — verify + OCR + DoD + reviewer + Gates + commit

- **Verify contrato:** los 5 comandos del §6 en orden (YAML + actionlint + diff-check + 2 rg). `campaign_verify_cmd` con bug exit -1 conocido → fallback bash directa y anotarlo.
- **Verify full relevante:** YAML-only → `cargo fmt/clippy/nextest` N/A (0 `.rs` tocados); `scripts/validate-docs-coverage.ps1` N/A (0 docs); OCR delegation `pwsh dev-tools/ocr-review.ps1` sobre el diff si disponible (advisory; Critical/High=bloquea, Medium→FIND-* nuevo, Low se descarta). `/cleanCA` N/A (0 código; norma Clean transversal leída igual — Paso 0c).
- **DoD 3 niveles:** (1) commit: 1 cosa lógica (`ci:` + task ID), mensaje explica porqué, sin secrets en diff, sin formatting mezclado; (2) release N/A (no bump — `ci:` no publica); (3) deploy N/A (sin flag/rollback — cambio CI reversible por revert).
- **Reviewer distinto P2-01:** `vanta-review` (adversarial issues-only sobre diff 2 líneas + contrato) antes del commit final; `code-review-and-quality` como pre-commit gate. Non-interactive → self-review doubt-driven reconciliado abajo si `task` no disponible.
- **Gates D/V/C:** D no dispara (≤10 archivos, sin hot path/API/símbolos, contrato mecánico — fix directo); V dispara solo si 2 fallas mismo-error en verify (→ `question` al usuario, sin respuesta → STOP); C: colaterales → routing `prompts/findings.md` (nuevo FIND, nunca solo anotado); `git status` con M fuera del blast radius → confirmar alcance antes de `git add` (solo 2 archivos propios).
- **Cierre:** `git add` solo archivos propios (yml + task file) + commit conventional `ci: FIND-95 — ...` + `campaign_update_task_state` completed + recitation canónica (§3 pipeline-full) + 1-2 lessons + Context Save Point + bloque RESULTADO §7 (siempre; nunca silencio). Backlog→avance NO tocar (race paralelo, orquestador) + push vía vanta-lead.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/rustdoc-70.yml` (83L — triggers `:3-22`, job rustdoc `:31-83`, comando `:59`), `.github/workflows/ci-rustdoc.yml` (82L — modelo fix `:16-36`), `Cargo.toml:704,716` (members/default-members vía rg), `docs/dev/Backlog.md:237` (fila FIND-95 ⬜), `.opencode/rules/release-ci.md` (42L), `.opencode/references/clean-code-clean-architecture.md` (Apéndice V), Notion 4 páginas (fetch COMPLETAS), `git show e395b563` (modelo commit + diff).
- **Referencias hacia dentro:** `on.push.paths` (`:6-12`) y `on.pull_request.paths` (`:15-21`); ninguna contiene `'vanta-memory/**'` (tampoco `'vantadb-*/**'` ni `'providers/**'`).
- **Referencias entrantes:** pushes a `develop` y PRs a `main`/`develop` cuyos archivos matcheen `paths`. Hoy solo-`vanta-memory/` no dispara rustdoc-70.
- **Veredicto:** 1 archivo + 2 líneas, riesgo 🟢. **Gate D:** no dispara — fix directo.
- **Gate Regla 0:** esta sección llena ANTES de la primera edición ✅.

## Steps atómicos

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | DISCOVERY + crear este task file (Regla 0, 10 bloques, steps) | ✅ DONE | este archivo, 10 bloques + Regla 0 |
| 2 | Fix: agregar `'vanta-memory/**'` tras `'Cargo.lock'` en push (`:10`) y PR (`:19`) | ✅ DONE | `git diff` exactamente +2/-0 |
| 3 | Verify contrato (§6: YAML + actionlint + diff-check + 2 rg) + OCR + reviewer P2-01 | ✅ DONE | YAML OK ambos triggers; actionlint exit 0; diff-check exit 0; rg 6+4 hits; OCR sin Critical/High en scope; review APPROVE reconciliado abajo |
| 4 | Commit `ci:` (solo yml + task file) + `campaign_update_task_state` + lessons + RESULTADO | ✅ DONE | `2acb09ed` + bloque §7 |

## Pre-mortem (del plan — verificado en DISCOVERY)

1. Path con typo rompe el trigger → validar con `actionlint` + parse YAML (pyyaml disponible según FIND-64).
2. Otro workflow con el mismo gap → `rg` global pre-fix: NO (los 2 archivos con el patrón ya fixed; este ni lo tiene). Si aparece → FIND nuevo.

## Context Save Point

- **Branch:** `develop` · **Steps:** 1-3 ✅, 4 ⬜ (commit) · **Próximo:** `git add .github/workflows/rustdoc-70.yml docs/dev/tasks/FIND-95.md && git commit -m "ci: FIND-95 — ..."` + `campaign_update_task_state` completed.
- **Verify reproducido:** YAML OK (push+PR listan `vanta-memory/**`, vía `d.get('on', d.get(True))` — pitfall YAML 1.1 `on:`→`True`, primer intento ingenuo `d['on']` dio KeyError) · actionlint exit 0 · `git diff --check` exit 0 (bash directa; `campaign_verify_cmd` bug exit -1 no reintentado — riesgo conocido del plan) · `rg vanta-memory` 6 hits · `rg vantadb-*/**` 4 hits estables.

## RECONCILE — review P2-01 (doubt-driven, 1 ciclo, STOP: resto futuro/noise)

Reviewer `vanta-review` → veredicto **APPROVE** (issues-only, examen línea por línea, 0 defectos bloqueantes).

| # | Finding reviewer | Clasificación | Acción |
|---|------------------|---------------|--------|
| O1 | Divergencia `paths` workspace-wide (rustdoc-70 sin `tests/vantadb-*/providers`; ci-rustdoc sin `docs/*.md`) | Trade-off válido pre-existente | Futuro FIND; no este diff |
| O2 | Asimetría `branches` push/PR entre ambos rustdocs | Supuesto no declarado (main protegido → hueco inalcanzable) | Verificar branch protection fuera del diff; no bloquea |
| O3 | Colisión `concurrency.group: rustdoc-...` idéntico (probabilidad agravada por este diff) | Trade-off válido pre-existente | Futuro FIND (patrón correcto en `gate-docs-21.yml:13`); no este diff |
| N1 | Cita `:59` imprecisa (comando real incluye `--document-private-items` + `RUSTDOCFLAGS`) | Válido menor | Corregido aquí: comando completo citado en §7/§8 |
| N2 | `Cargo.toml:704,716` verificado por reviewer | Confirmación | Ninguna (refuerza membresía) |
| N3 | Pitfall YAML `on:`→`True` (metodología del check) | Válido + accionable inmediato | Comando §6 enmendado a `d.get('on', d.get(True))`; primer intento ingenuo falló con KeyError — evidencia viva del pitfall |
| N4 | `*.md` no cruza `/` → el diff además cierra `vanta-memory/*.md` | A favor no documentado | Registrado aquí |
| Claim | `0 gaps restantes` overclaim bajo lectura workspace-wide | Contract misread parcial | Re-redactado: `0 gaps de vanta-memory restantes` (los 3 workflows doc/test ahora listan el crate) |

Veredicto reconciliado: **contrato FIND-95 PASA** (los 5 checks verdes); opcionales O1-O3 → futuros FINDs. Cross-model skipped: non-interactive context (pipeline task).

## Scope previo

N/A (task file nuevo, sin legacy).
