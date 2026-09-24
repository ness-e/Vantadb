# FIND-139 — Endurecer `ci-gate.yml` + `gate-docs-21.yml`

> **Plan:** `docs/dev/plans/2026-09-21-workflows-repair.md` (Wave 1, paralelo disjunto con FIND-137/138)
> **Estado:** ✅ COMPLETED
> **Tipo:** devops (CI/CD) · **Scope:** 2 YAML, cero lógica de negocio, sin rediseño del gate
> **SDP:** ci-cd-and-automation · git-workflow-and-versioning (+ base campaign-executor/progreso/doubt-driven-development; discover v2 lifecycle genérico descartado por N/A a YAML/CI)
> **Regla leída:** `.opencode/rules/release-ci.md` (completa, 42L) · **Ref:** `definition-of-done.md`

## Contrato (ley — AC a/b/c/d)

- (a) el gate falla en rojo cuando un check falta o está pendiente (el `*)` actual lo deja pasar)
- (b) REQUIRED incluye semver/ADR **o documenta por qué no, con 1 línea por ausente**
- (c) gate-docs corre también en PR→develop
- (d) `actionlint` exit 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `fuzz-40.yml:30`, `heavy-bench-nightly-51.yml:31`, `heavy-certification-50.yml:25` (usan `ci-gate.yml` vía `workflow_call` + `needs: ci-gate`) |
| Callees | ninguno (bash `gh api check-runs` + `git diff`; sin imports de código) |
| Implicaciones | contrato de gate más estricto (pending/not-found ahora rojo); sin cambio de triggers salvo gate-docs PR→develop; sin API pública ni performance |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-gate.yml` (59L), `.github/workflows/gate-docs-21.yml` (87L), `.opencode/rules/release-ci.md` (42L, solo lectura), `ci-rust-10.yml:92-194` (jobs semver/ADR), `sec-codeql-30.yml:1-35` (job Analyze), plan file `2026-09-21-workflows-repair.md:44-46`.
- **Archivos referenciados hacia dentro:** ninguna (YAML workflows, sin imports de código; `gh api` y `git` en bash).
- **Archivos que referencian a los editados:** `fuzz-40` / `heavy-bench-nightly-51` / `heavy-certification-50` → `ci-gate.yml` (firma `inputs.event_name` intacta, solo cambia severidad del `case` + comentarios); nadie referencia `gate-docs-21.yml` (trigger-only).
- **Veredicto impacto:** bajo — 2 archivos, ~4 líneas, sin rediseño, sin tocar timeout (FIND-135) ni resto de workflows (FIND-137/138 en paralelo).

## Spec (SDD)

N/A — no es feature-add: cero `pub fn`/structs/endpoints/tools/componentes nuevos; solo severidad de un `case` bash + 1 trigger + 2 comentarios. Sin decisiones abiertas.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** timeout `ci-gate: 5` intacto (FIND-135); firma `workflow_call.inputs.event_name` intacta; resto de workflows intactos (FIND-137/138 en paralelo); WIP ajeno intacto; NO PUSH (pushea solo vanta-lead); `cargo -j 2` N/A (sin Rust).
- **Comandos de verificación:** `actionlint .github/workflows/ci-gate.yml .github/workflows/gate-docs-21.yml` → exit 0; `git diff --check` → limpio; `campaign_verify_cmd` con BUG exit -1 → fallback bash directa + mención.
- **Deuda pendiente:** ninguna (o “ver Notas” si aparece).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — 0 líneas de deuda nueva, 0 pago requerido (cambio de severidad + trigger + comentarios).

## Definition of Done (P2-08)

- Task: contrato (a)(b)(c)(d) verificable abajo + `git diff --check` limpio.
- Commit: atómico (~10 líneas), `ci: FIND-139 — ...`, staging selectivo (3 files), NO PUSH.
- Release: N/A (no es release; gate de CI).

## Herramientas necesarias

- `actionlint` (1.7.12 verificado) + `git diff --check` + `campaign_verify_cmd` (BUG exit -1 → bash directa + mención).
- codegraph/Cargo/Internet: N/A (YAML estático + `gh api` rulesets como fuente de verdad local).

**Skills cargadas (SDP):** ci-cd-and-automation (gates bash fail-fast, no tragar pending) · git-workflow-and-versioning (commit atómico `ci:`, staging selectivo, NO PUSH).

## Investigation Notes — matriz checks-requeridos vs checks-cubiertos

Fuente de verdad: ruleset `develop` id 23692587 (`gh api repos/ness-e/Vantadb/rulesets/23692587`) y `main` id 16844590 — ambos exigen los mismos 11 contextos. `ci-gate.yml:31-43` REQUIRED los cubre 1:1.

| # | Check requerido (ruleset develop) | En REQUIRED ci-gate | Origen job visible |
|---|-----------------------------------|---------------------|--------------------|
| 1 | Format Check | ✅ | `ci-rust-10.yml` fmt |
| 2 | Clippy Lints | ✅ | `ci-rust-10.yml` clippy |
| 3 | Tests (Linux) | ✅ | `ci-rust-10.yml` test |
| 4 | Tests (Windows) | ✅ | `ci-rust-10.yml` test-windows |
| 5 | Tests (macOS) | ✅ | `ci-rust-10.yml` test-macos |
| 6 | MSRV Check (1.94.1) | ✅ | `ci-rust-10.yml` msrv |
| 7 | Experimental Crates Check | ✅ | `ci-rust-10.yml` experimental |
| 8 | Security Audit | ✅ | `ci-rust-10.yml` audit |
| 9 | Miri (UB Detection) | ✅ | `ci-rust-10.yml` miri |
| 10 | Dependency Policy Check | ✅ | `ci-rust-10.yml` deny |
| 11 | Analyze | ✅ | `sec-codeql-30.yml:18` (`analyze`, `name: Analyze`) — **sí visible**, el hallazgo “sin job visible” queda refutado |
| — | Semver Checks (public API) | ❌ intencional | `ci-rust-10.yml:92-128` existe pero `if: ref==main \|\| base_ref==main` → en develop queda skipped/sin check-run → añadirlo a REQUIRED dejaría el gate en rojo permanente en develop. |
| — | ADR Gate (public API) | ❌ intencional | `ci-rust-10.yml:130-194` existe pero `if: event==pull_request` → en push/schedule (lo que gatea ci-gate) no hay check-run → añadirlo daría falso rojo. |

Hallazgo (a) confirmado: `ci-gate.yml:45-53` filtra `select(.status=="completed")`, así que un check pendiente o inexistente da `CONCLUSION` vacía y cae en `*) ;;` que no hace nada → verde falso. Fix: `*) FAILED=1`.
Hallazgo (c) confirmado: `gate-docs-21.yml:4-9` push mira `main+develop` pero `pull_request.branches: [main]` → PR→develop no corre. Fix: `[main, develop]`.
Timeout: ya puesto por FIND-135 (`ci-gate: timeout-minutes: 5`, L22) — no tocar. Concurrency en reusable `workflow_call` no aplica al gate (lo ponen los callers) — fuera de scope, sin rediseño.
Internet: N/A (sin ambigüedad API; `gh` local es fuente de verdad).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 3 (S2, S3, S4) |
| % completado | 100% (S1–S4 ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries (YAML estático, `GH_TOKEN: github.token` read-only ya existente, sin input de usuario ni deps nuevas). Justificación: cambio de severidad + trigger + comentarios.
- [x] **PERFORMANCE** — no aplica: sin hot path (loop `gh api` de ~11 checks, segundos; gate-docs añade 1 trigger, sin jobs nuevos).

## Steps

### Step 1: DISCOVERY completo
- **Archivos:** `ci-gate.yml`, `gate-docs-21.yml`, rulesets `develop`/`main`, `ci-rust-10.yml:92-194`, `sec-codeql-30.yml:1-35`
- **Acción:** leer ambos archivos enteros + lista exacta de checks del ruleset + matriz + task file
- **Verify:** matriz §Investigation poblada + este task file existe
- **Estado:** ✅ COMPLETED

### Step 2: `ci-gate.yml` — `*)` a rojo + 1 línea por ausente (semver/ADR)
- **Archivos:** `.github/workflows/ci-gate.yml`
- **Acción:** (a) `*) ;;` → `*) FAILED=1 ;;` (+ comentario fail-closed); (b) 2 comentarios de 1 línea sobre REQUIRED documentando por qué Semver/ADR quedan fuera (guards `ref==main` / `event==pull_request`); sin tocar timeout ni REQUIRED existente ni “Analyze”
- **Verify:** `actionlint .github/workflows/ci-gate.yml` exit 0 + `git diff --check` limpio
- **Estado:** ✅ COMPLETED

### Step 3: `gate-docs-21.yml` — PR→develop
- **Archivos:** `.github/workflows/gate-docs-21.yml`
- **Acción:** `pull_request.branches: [main]` → `[main, develop]` (1 línea; push ya mira ambas)
- **Verify:** `actionlint .github/workflows/gate-docs-21.yml` exit 0 + `git diff --check` limpio
- **Estado:** ✅ COMPLETED

### Step 4: VERIFY + commit selectivo + RESULTADO
- **Archivos:** los 2 workflows + este task file
- **Acción:** `actionlint` (2 files + full) + `git diff --check` + secrets-grep vía bash directa (BUG `campaign_verify_cmd` exit -1 → mencionar); `git add` solo 3 files + `git commit -m "ci: FIND-139 — ..."` (NO PUSH); `campaign_update_task_state completed` + RESULTADO §7 + Gates D/V/C
- **Verify:** commit existe + `git status --short` limpio en los 3 paths + RESULTADO parseable
- **Estado:** ✅ COMPLETED

## Dependencias

- Sin bloqueantes. Wave 1 paralelo disjunto (FIND-137/138 no tocan estos 2 files). NextTask: Wave 2 (orquestador).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** pendiente (orquestador vía P2-01 batch al cierre del plan; fallback: doubt-driven-development ya cargada como base).
- **Enfoque:** ¿fail-closed `*)` + documentar-en-vez-de-añadir semver/ADR es el approach correcto vs añadirlos a REQUIRED?
- **Cómo se probó:** pendiente (S4: actionlint + diff --check + secrets-grep).
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** pendiente.

## Notas

S2–S4 2026-09-22: actionlint 2 files exit 0 + full exit 0; `git diff --check` limpio; secrets-grep sin matches (bash directa — BUG `campaign_verify_cmd` exit -1); commit `ci: FIND-139 — ci-gate fail-closed + gate-docs PR-develop` NO PUSH.

## Gate D

No disparado — blast radius 2 YAML sin hot path ni API pública, contrato mecánico (a)(b)(c)(d), cero símbolos públicos nuevos, sin spec pendiente.

## Prohibidos (intocables)

Resto workflows (FIND-137/138 en paralelo), `src/`, `web/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, stash GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation vía `campaign_update_task_state`), `C:/Users/Eros/.vantadb*`, secretos.
