---
title: "TASK PROC-02: Mergear los 5 PRs de Dependabot abiertos"
kind: task
description: "Deps PRs #237/#232/#231/#230/#224 (squash + bypass admin configurado) — contrato: deps open=0 y alerts ≤1"
---

# TASK PROC-02: Mergear los 5 PRs de Dependabot abiertos

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 1, Wave 0)
- **Fuente:** `docs/dev/Backlog.md` (PROC-02)
- **Esfuerzo:** 🟢 1h (+CI por PR) | **Appetite:** 1d
- **Prioridad:** 🟠
- **Tipo:** release-CI (merges remotos)
- **Creado:** 2026-10-03T01:50Z | **last-synced:** 2026-10-03T02:42Z
- **Estado:** ⏳ IN PROGRESS
- **Campaign ID:** post-release-0.8.0-20261002

## Hallazgos de reconocimiento (2026-10-03)

- Ruleset `develop` (id 23692587, activo): 11 required checks + `strict_required_status_checks_policy: true`, **con `bypass_actors: RepositoryRole 5 (admin) — always`** (bypass admin = configuración explícita del ruleset, con precedente del owner 2026-09-22: batch #190-#194).
- Los 11 required checks los producen `ci-rust.yml` (10) + `sec-codeql.yml` (Analyze) — ambos con trigger `pull_request: branches: ["main"]` **solamente**: en PRs a develop no pueden satisfacerse; el bypass admin del ruleset es el mecanismo previsto para deps→develop.
- `allow_auto_merge: true`; `delete_branch_on_merge: false`; estilo histórico de merges de deps: **squash**.
- **#232 (undici) tenía base `main`, NO develop** — el plan asumía "todos contra develop" (incorrecto para #232). Dependabot lo dirigió a main (el ecosistema npm /desktop fue "locked a solo-alertas" en `f12ffd9d`; el PR era legacy previo al lock). Mergeado a main: cierra las 7 alerts de undici (el fix debe estar en la default branch).
- #238 (release v0.9.0): HOLD por decisión del owner — fuera de scope.

## Ledger de merges

| PR | Base | Merge commit | Estado | Nota |
|----|------|--------------|--------|------|
| #237 (rust-toolchain) | develop | `266f416d` | ✅ MERGED | squash |
| #232 (undici /desktop) | **main** | `5c2fc71d` | ✅ MERGED | base main (dependabot); alerts undici 7→0 |
| #231 (codeql-action/init) | develop | `db4f46fd` | ✅ MERGED | squash |
| #230 (codeql-action/analyze) | develop | `dee48711` | ✅ MERGED | squash |
| #224 (croaring 2.8.0) | develop | — | ⬜ PENDING | esperando `Generate API reference (rustdoc)` |

## Steps

- [x] **Step 1 — Reconocimiento:** PR list + ruleset + protección + estilo de merges — ✅
- [x] **Step 2 — Update-branch #237/#232/#231/#230 + rebase #224** — ✅
- [x] **Step 3a — Merge #237/#231/#230 → develop** — ✅ (squash + bypass, secuencial)
- [x] **Step 3b — Merge #232 → main (su base)** — ✅ + verificación de impacto (main CI + release-plz en curso)
- [ ] **Step 3c — Merge #224 (croaring) → develop** — ⬜ esperando rustdoc
- [ ] **Step 3d — Fix residual brace-expansion (alert #62, medium, vantadb-ts)** — ⬜ evaluar `npm audit fix`
- [ ] **Step 4 — Verify contrato:** deps open = 0; dependabot alerts ≤1 (residuales triados); develop CI verde post-merges — ⬜
- [ ] **Step 5 — Review P2-01 + cierre** — ⬜

## Contrato (del plan)

1. `gh pr list --state open --json title --jq '[.[] | select(.title | startswith("chore(deps)"))] | length'` = 0
2. `gh api '/repos/ness-e/Vantadb/dependabot/alerts?state=open' --jq 'length'` ≤ 1 (residuales triados con nota)
3. develop con CI verde post-merges

## Pre-mortem (del plan)

1. #224 stale por el ciclo 0.8.0 → rebase pedido (`@dependabot rebase`) + CI re-corrido.
2. Updater de brace-expansion caído → `npm audit fix` en `vantadb-ts` si hace falta (residual ≤1 aceptado por contrato).
3. Merge en batch rompe el lock → merge secuencial con CI verde por PR.

## Review (P2-01)

| Campo | Valor |
|-------|-------|
| Reviewer | (pendiente — vanta-review al cierre) |
| reviewer_context | (pendiente) |
| Verdict | (pendiente) |

## DoD (3 niveles)

- **task:** contrato verde por comando
- **commit:** n/a (merges remotos; la actualización de plan/checkpoint se commitea con la campaña)
- **release:** n/a

## Deuda técnica (Regla 6)

Ninguna introducida. Notas: (1) el workflow de Builds (no requerido por el ruleset) también corre en PRs de deps — informativo; (2) el acoplamiento "required checks main-only vs ruleset develop" queda documentado acá y va como FIND propuesto en el reporte de campaña.
