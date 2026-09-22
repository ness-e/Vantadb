# FIND-134: hardening `ci-rust-10.yml` sin cambiar qué valida

## Metadata
- **Plan file:** `docs/plans/2026-09-21-workflows-repair.md` (Wave 0, paralelo con FIND-135/136 disjuntos)
- **Fuente:** plan Wave 0 + auditoría 28/28 (2026-09-21) + internet
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** CI/CD / DevOps (devops; `campaign_detect_task_type` + SDP v2 confirman)
- **Turns estimados:** 8
- **Creado:** 2026-09-22
- **last-synced:** 2026-09-22
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `ci-gate.yml` (REQUIRED ×11 lee check-runs por `name:`, no por job id — renombres de `name:` prohibidos); badges README/​README_ES apuntan al filename (FIND-142 los renombra, no esta task); `docs/workflow/ci-rust-10.md` documenta triggers (colateral, scope FIND-143) |
| Callees | `.github/actions/rust-setup` (sccache punto único, Regla release-ci §2 — no se toca); `Cargo.toml` workspace (no se toca); `providers/*/Cargo.toml` (solo lectura scope) |
| Implicaciones | Ningún contrato de código cambia; cero Rust; set de validaciones idéntico (11 required sobreviven con mismos `name:`); `needs` convierte fallos fmt/clippy en skip (no en rojo) de los pesados — tradeoff fast-fail documentado; quitar `develop` de push elimina el duplicado push+PR mismo SHA en PRs develop→main; `providers/**` amplía trigger (más runs, no menos validación) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `.github/workflows/ci-rust-10.yml` (613 líneas), `.github/workflows/ci-gate.yml` (58 líneas), `.opencode/rules/release-ci.md` (42 líneas), `.opencode/references/definition-of-done.md`, `docs/plans/2026-09-21-workflows-repair.md`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `.github/actions/rust-setup` (6 usos), `actions/checkout@3d3c42e…` (14 usos), `taiki-e/install-action@…` (4 usos), `actions/cache`, `actions/upload-artifact`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache` — ninguno se edita
- **Archivos que referencian a los editados (referencias entrantes):** `ci-gate.yml:30-42` (11 REQUIRED por nombre), `README.md:8` + `README_ES.md:8` (badge por filename), `docs/workflow/ci-rust-10.md:64` (triggers documentados), `docs/TEST_MAP.md:64`, ADRs (citas históricas de líneas — drift aceptado, no se reescriben)
- **Veredicto impacto:** bajo — 1 archivo YAML, sin código, sin API pública, sin símbolos nuevos. codegraph N/A (CI, no código). Internet N/A (auditoría ya trae fuentes).

## Los 11 required que deben sobrevivir (nombres exactos, `ci-gate.yml:30-42`)
1. `Format Check` (job `fmt`) · 2. `Clippy Lints` (job `clippy`) · 3. `Tests (Linux)` (job `test`) · 4. `Tests (Windows)` (job `test-windows`) · 5. `Tests (macOS)` (job `test-macos`) · 6. `MSRV Check (1.94.1)` (job `msrv`) · 7. `Experimental Crates Check` (job `experimental-check`) · 8. `Security Audit` (job `audit`) · 9. `Miri (UB Detection)` (job `miri`) · 10. `Dependency Policy Check` (job `deny`) · 11. `Analyze` (CodeQL, workflow externo — fuera de este archivo)
- Ningún `name:` se renombra. `needs:` no altera check names (solo gating). `coverage` (`Code Coverage…`) no es required → su `needs` es seguro.

## Contrato
"`actionlint` exit 0 + `git diff --check` limpio + (a) push solo `main` · (b) 7 jobs pesados con `needs: [fmt, clippy]` · (c) cero `continue-on-error` sin `# CATEGORY:` (los 5 existentes ya tageados; se quitan los 2 `|| echo` que fuerzan exit 0) · (d) `wasm-test` if válido en push y PR · (e) los 11 `name:` intactos (verificado por grep contra `ci-gate.yml`)"
- **Resultado:** ✅ todos los literales verificados en S4 (actionlint 0, diff-check 0, needs=7, `|| echo`=0, coe 5/5 TAGGED, names 10/10 en archivo + Analyze externo).

## Spec (SDD)
N/A — no es feature-add: cero símbolos públicos nuevos (`pub fn`/tool/endpoint/binding). Solo reordena gating/triggers de CI existente. Phase 1b: ninguna señal aplica.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** 11 check names idénticos; filename `ci-rust-10.yml` intacto (FIND-142); resto de workflows intactos (FIND-135/136 en paralelo); stash GOV-C4 y WIP ajeno intactos; `docs/Backlog.md` y plan file no tocados por esta task; cero commits a `main`/`develop` salvo el `ci:` propio; NO PUSH (solo vanta-lead pushea)
- **Comandos de verificación:** `actionlint .github/workflows/ci-rust-10.yml` (exit 0) · `git diff --check` (limpio) · `campaign_verify_cmd` con BUG exit -1 conocido → fallback bash directa + mención explícita
- **Deuda pendiente:** ninguna propia; colateral `docs/workflow/ci-rust-10.md:64` (lista de paths sin `providers/**`, sin mención de `needs`) → queda_pendiente al orquestador (scope FIND-143)

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-134: hardening `ci-rust-10.yml` sin cambiar qué valida |
| `lastAction` | S1–S5 completos: triggers + `needs`×7 + wasm-if + `|| echo`×2 + verify + commit `ci:` (NO PUSH) |
| `result` | OK (contrato S4 verde; commit verificado en log) |
| `nextAction` | Ninguno propio — orquestador: review P2-01 batch + Wave 1 |
| `contract` | ver `## Contrato` + invariantes arriba; evidencia: diff `ci-rust-10.yml` (+12/-4 aprox: triggers 3/1, needs 7/0, wasm-if 1/1, echo 0/2) + `actionlint` exit 0 (S1/S2/S3/S4) + `git diff --check` exit 0 + greps S4 (needs 7, echo 0, coe 5/5 TAGGED, names 10/10); confianza alta |
| `nextTask` | Wave 1 (orquestador decide tras Wave 0 verde) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda — cero deuda nueva (solo gating/triggers YAML); no se introduce `continue-on-error` nuevo (Regla 2 AGENTS.md + release-ci §5).

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** ✅ contrato `## Contrato` verde + capa determinista N/A (cero Rust; `cargo` no aplica — justificado) + `actionlint` 0 por edit
- **Commit:** ✅ atómico (~16 líneas), `ci: FIND-134 — …`, staging selectivo (2 archivos), verificación mecánica real (nunca auto-reporte)
- **Release:** N/A (CI-only, sin changelog user-visible; justificado). Pre-push gate Regla 1: NO PUSH — commit local, pushea vanta-lead

## Herramientas necesarias
- `actionlint` (gate por edit) · `git diff --check` · bash directa (fallback BUG `campaign_verify_cmd` exit -1)

**Skills cargadas (SDP):** `ci-cd-and-automation` (quality gates, fast-fail `needs:`, 1-push-1-run triggers — justificación: base type CI/CD) · `git-workflow-and-versioning` (commit atómico `ci:`, staging selectivo, no-push — justificación: cierre con commit) · SDP v2 devolvió además lifecycle genéricas (`incremental-implementation`, `test-driven-development`, `context-engineering`, `source-driven-development`, `frontend-ui-engineering`, `api-and-interface-design`) — descartadas por ponytail: ninguna aplica a YAML-CI sin código (documentado, no cargadas)

## Investigation Notes
- `campaign_discover_skills_v2` (phase=BUILD, keywords github-actions/concurrency/needs/continue-on-error/actionlint): base `ci-cd-and-automation` + lifecycle; manifest sin candidatos (score <0.5 para CI-YAML). Elegidas ≤8: 2 útiles.
- `campaign_detect_task_type` → `devops` (checks: `yamllint .github/` — cubierto por `actionlint`, más estricto para Actions).
- `campaign_get_next_task` → `hasTask:false` (plan en formato checkbox, sin tasks MCP) → sin `claim`; `campaign_update_task_state` N/A → motivo registrado (no hay task MCP que transicionar).
- Duplicado push+PR mismo SHA: push a `develop` con PR abierta develop→main dispara `push` (branches incluye develop) + `pull_request` (branches main) → 2 runs. Quitar `develop` de push lo elimina; `push:main` sigue cubriendo post-merge.
- `contains(github.event.head_commit.modified, …)` en PR: `head_commit` es null → `contains(null,…)` evalúa a skip silencioso (job muerto en PR). Fix: condición explícita por evento sin `head_commit`.
- `needs: [fmt, clippy]`: job IDs exactos `fmt`, `clippy`. Jobs pesados: `test`, `test-windows`, `test-macos`, `coverage`, `miri`, `sanitizer-asan`, `sanitizer-tsan` (30–60 min). Excluidos: `msrv`/`minimal-versions`/`experimental-check`/`audit`/`deny`/`semver-checks`/`adr-gate`/`wasm-test` (rápidos o con gating propio — cambio mínimo).
- `|| echo` quitados: con `continue-on-error: true` a nivel job el fallo ya es non-blocking pero visible; el `|| echo` lo volvía exit 0 (invisible hasta en el log anotado). Quitar = señal preservada, gating intacto.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — No aplica: YAML de CI, cero trust boundaries tocados (sin inputs de usuario, auth, deps, storage, FFI, red). `permissions: contents: read` intacto. Justificado.
- [x] **PERFORMANCE** — No aplica: cero hot paths (sin `vector/`, engine, loops, serialización). Efecto CI-cost: `providers/**` añade runs solo cuando providers cambian; `needs` ahorra 30–60 min por fmt/clippy rojo. Justificado.

## Steps

### Step S1: triggers — `develop` fuera, `providers/**` dentro
- **Archivos:** `.github/workflows/ci-rust-10.yml`
- **Acción:** `branches: [ "main", "develop" ]` → `[ "main" ]` (solo `push`); `providers/**` tras `integrations/**` en push y PR
- **Verify:** `actionlint` exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step S2: fast-fail — `needs: [fmt, clippy]` ×7
- **Archivos:** `.github/workflows/ci-rust-10.yml`
- **Acción:** `needs: [fmt, clippy]` en `test`, `test-windows`, `test-macos`, `coverage`, `miri`, `sanitizer-asan`, `sanitizer-tsan`
- **Verify:** `actionlint` exit 0 ✅ + grep = 7 hits ✅
- **Estado:** ✅ COMPLETED

### Step S3: wasm-if válido + quitar `|| echo` ×2
- **Archivos:** `.github/workflows/ci-rust-10.yml`
- **Acción:** `if:` explícito por evento (PR siempre válido; push scopado a `vantadb-wasm/`); quitados ambos `|| echo` (ASan/TSan), `--skip …` como última línea
- **Verify:** `actionlint` exit 0 ✅ + `|| echo` = 0 hits ✅ + coe 5/5 TAGGED ✅
- **Estado:** ✅ COMPLETED

### Step S4: verify final contratos (a)-(e)
- **Archivos:** (ninguno — solo comandos)
- **Acción:** `actionlint` 0 ✅; `git diff --check` 0 ✅; names 10/10 + Analyze externo ✅; coe 5/5 TAGGED ✅; status solo archivos propios + WIP ajeno ✅
- **Verify:** todos exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step S5: commit selectivo (NO PUSH)
- **Archivos:** `.github/workflows/ci-rust-10.yml`, `docs/tasks/FIND-134.md`
- **Acción:** staging selectivo 2 archivos + `git commit -m "ci: FIND-134 — …"`; pre-commit hygiene (secrets grep 0 hits)
- **Verify:** commit en `git log --oneline -1` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Wave 0 paralelo (FIND-135 ✅ commit `217d583a`; FIND-136 IN PROGRESS en archivos disjuntos — no tocados). NextTask: Wave 1 (orquestador).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** pendiente (orquestador asigna batch P2-01 tras Wave 0; sin `question` tool en este contexto → motivo registrado, no bloquea commit local)
- **Enfoque:** ¿`needs` en 7 jobs es el set correcto? ¿wasm-if preserva intent push-scoped + PR explícito?
- **Cómo se probó:** `actionlint` 0 por edit (S1/S2/S3/S4) + `git diff --check` + greps de conteo (evidencia mecánica, no auto-reporte)
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (todas copiadas de tool results)
  - [x] No saltarse clarificación (contrato del plan explícito; Gate D no dispara: 1 archivo CI, sin API pública)
  - [x] No declarar done sin verificar (verify por step con `actionlint`)
  - [x] No ignorar fallos (cero fallos verify en esta task)
  - [x] No un solo intento de búsqueda (grep repo-wide + lectura completa 613 líneas + `ci-gate.yml` + rules + re-verificación de estado real tras resume)
  - [x] No copiar sin citar (líneas exactas citadas por claim)
  - [x] No reintentar en bucle (sin fallos; retry ladder + Gate V listos, no disparados)
  - [x] No dejar huérfanos los pasos (cada step → contrato (a)-(e))
  - [x] No degradar chequeo de errores (se *quita* enmascaramiento `|| echo`, no se añade)
  - [x] No gastar presupuesto infinito (5 steps acotados, ~16 líneas de diff)
- **Veredicto:** ⬜ pendiente (post-commit, agente distinto)

## Notas
- Gates: P:no (plan aprobado por owner 2026-09-21, Gate P en plan file) · D:no (blast radius 1 archivo CI, sin símbolos públicos, contrato explícito) · V:no (sin fallos verify) · C:registrado (colateral `docs/workflow/ci-rust-10.md:64` → scope FIND-143; sin `question` tool disponible → no bloquea, queda_pendiente al orquestador).
- `campaign_update_task_state` N/A: `campaign_get_next_task` → `hasTask:false` (plan checkbox, sin tasks MCP). Motivo registrado aquí en vez de transición.
- `campaign_verify_cmd` N/A por BUG exit -1 conocido → bash directa para `actionlint`/`git diff --check`/greps (mención explícita aquí y en RESULTADO).
- Resume 2026-09-22: el intento previo SÍ había persistido (S1 + `needs` en `test` + task file). Re-verificado estado real con `git diff` antes de continuar; no se re-hizo trabajo.
- Plan file NO tocado por esta task (prohibido en esta invocación; su `M` es del orquestador/FIND-136). `skill progreso` NO ejecutado (migraría Backlog/plan — prohibidos; lo corre el orquestador en cierre Wave 0). `campaign_diagnose_pipeline` omitido (YAML-only, nada que diagnosticar).
- `docs/Backlog.md` NO tocado (prohibido). `C:/Users/Eros/.vantadb*` y secretos: no tocados, nada a disco/logs.
