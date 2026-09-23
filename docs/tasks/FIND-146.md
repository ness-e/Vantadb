# FIND-146 — Pins SHA supply-chain + guard doble-run opencode

> **Plan:** docs/plans/2026-09-21-workflows-repair.md (Wave 2) · **Estado:** ⏳ IN PROGRESS
> **Origen:** auditoría 2026-09-21 (28/28 workflows) · **Tipo:** devops (CI/CD supply-chain hardening)
> **SDP:** ci-cd-and-automation, security-and-hardening, source-driven-development, doubt-driven-development, incremental-implementation (campaign_discover_skills_v2 phase=BUILD, keywords [github-actions, supply-chain, sha-pin, permissions], ≤8)

## Objetivo

Eliminar tags móviles en 4 workflows (supply-chain) + guard doble-run en `opencode.yml`. Cambios mínimos, sin cambiar semántica del bot.

## Spec

| Pregunta | Decisión | Evidencia / motivo |
|---|---|---|
| ¿Nuevo comportamiento / API pública? | NO — solo pins + concurrency | N/A feature-add; fix mecánico CI |
| ¿Checkout a qué SHA? | `3d3c42e5aac5ba805825da76410c181273ba90b1` (# v7.0.1) | Mismo SHA que resto del repo (27+ usos) + `gh api repos/actions/checkout/commits/v7.0.1` verificado 2026-09-22 |
| ¿setup-node a qué SHA? | `49933ea5288caeca8642d1e84afbd3f7d6820020` (# v4.4.0) | Mismo SHA que resto del repo + `gh api repos/actions/setup-node/commits/v4.4.0` verificado |
| ¿upload-artifact a qué SHA? | `ea165f8d65b6e75b540449e92b4886f43607fa02` (# v4.6.2) | Mismo SHA que resto del repo (canónico nuevo; `4e7e4d1bb…` # v4.5.0 queda legacy en heavy-cert, fuera de scope) + `gh api repos/actions/upload-artifact/commits/v4.6.2` verificado |
| ¿OCR npm a qué versión? | `@alibaba-group/open-code-review@1.12.9` | `npm view @alibaba-group/open-code-review version` → 1.12.9 (2026-09-22); fuente https://www.npmjs.com/package/@alibaba-group/open-code-review |
| ¿Guard doble-run cómo? | `concurrency: group: opencode-${{ github.event.comment.id }}, cancel-in-progress: false` | Trigger verificado: `issue_comment[created]` + `pull_request_review_comment[created]` (opencode.yml:3-7); group por comment.id evita duplicados del mismo comentario sin cancelar comentarios distintos ni cambiar semántica del bot; patrón repo: ocr-nightly.yml:27-29 |

Tabla Spec N/A para lógica nueva (no hay feature-add; solo pins + concurrency).

## Contrato

- 0 tags móviles (`@v4` / `@latest` / sin SHA) en los 4 archivos.
- `actionlint` 0 en los 4 archivos.
- `git diff --check` limpio.
- `opencode.yml`: trigger intacto + guard añadido, semántica del bot sin cambio.
- Commit `ci: FIND-146 — ...` selectivo (4 workflows + este task file). NO PUSH.

## Archivos

- Clave (4):
  - `.github/workflows/arch-metrics-informational.yml` (checkout@v4:34 + upload-artifact@v4:58)
  - `.github/workflows/ocr-delegate.yml` (checkout@v4:25, setup-node@v4:32, npm latest:39, upload-artifact@v4:67)
  - `.github/workflows/ocr-nightly.yml` (checkout@v4 ×2:37/116, setup-node@v4 ×2:42/121, npm latest ×2:47/126, upload-artifact@v4 ×2:82/153)
  - `.github/workflows/opencode.yml` (ya pineado; solo guard doble-run)
- Prohibidos: resto workflows (FIND-140/141 en paralelo — release-*, heavy-*, fuzz-40, ci-*, gate-*), `src/`, `web/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, stash GOV-C4, `docs/Backlog.md`, plan file (solo recitation), `C:/Users/Eros/.vantadb*`, secretos.

## Impacto mapeado (Regla 0)

- Archivos leídos completos: los 4 workflows (arch-metrics 63L, ocr-delegate 72L, ocr-nightly 159L, opencode 34L) + `.opencode/rules/release-ci.md` (42L) + `definition-of-done.md` + `SPEC.md` raíz.
- Referencias hacia dentro (qué usan): `actions/checkout`, `actions/setup-node`, `actions/upload-artifact`, `npm:@alibaba-group/open-code-review`, `anomalyco/opencode/github` (ya pineado `@10765ff…` # dev, no se toca).
- Referencias entrantes (quién los dispara): arch-metrics = `pull_request(paths src/**)` + dispatch (informational, `continue-on-error: true` global); ocr-delegate = `pull_request` + dispatch (informational); ocr-nightly = `schedule 04:00 UTC` + dispatch (concurrency por workflow+ref ya existe); opencode = `issue_comment[created]` + `pull_request_review_comment[created]` + `if: contains/startsWith /oc|/opencode`.
- Veredicto: impacto BAJO, aislado a CI supply-chain. Sin runtime, sin API pública, sin hot path. Pins alinean a SHAs ya usados en 27+ sitios del repo (grep verificado). Guard es aditivo (concurrency), no altera `if:` del bot. Riesgo residual: versión npm fija 1.12.9 puede quedar atrás (aceptado: renovate/dependabot futuro, fuera de scope).

## Herramientas

- `actionlint` 1.7.12 (verify contrato) + `git diff --check` + `campaign_verify_cmd` (BUG conocido exit -1 → fallback bash directa + mención en recitation).
- `gh api repos/actions/{checkout,setup-node,upload-artifact}/commits/<tag>` (SHAs verificados) + `npm view` (versión OCR).
- Cargo N/A (cero código Rust).

## Pasos atómicos

- [x] S1: `arch-metrics-informational.yml` — checkout@v4 → SHA v7.0.1 + upload-artifact@v4 → SHA v4.6.2 (+ comentarios `# vX` espejo repo)
- [x] S2: `ocr-delegate.yml` — 3 SHAs + `npm install -g @alibaba-group/open-code-review@1.12.9` + header Pins actualizado
- [x] S3: `ocr-nightly.yml` — 6 SHAs (2 jobs) + npm ×2 a 1.12.9 + header Pins actualizado
- [x] S4: `opencode.yml` — guard concurrency por comment.id (trigger + `if:` intactos)
- [x] S5: verify triple (actionlint 4 files + `git diff --check` + grep 0 tags móviles) + commit selectivo `ci: FIND-146 — ...` (NO PUSH)

## Gates evaluados

- Gate P: no (plan owner-aprobado 2026-09-21, Wave 2).
- Gate D: no (blast radius 4 files, sin hot path/API pública/símbolos nuevos, contrato claro).
- Gate V: pendiente (si 2 fallas mismo-error en verify → question).
- Gate C: pendiente (al cierre; colaterales → FIND-* o incluir).

## Context Save Point

Creado 2026-09-22 en DISCOVERY. Rama `develop`, repo limpio (`git status` vacío salvo este task file untracked). SHAs verificados vía `gh api` (checkout 3d3c42e5, setup-node 49933ea5, upload-artifact ea165f8d) + `npm view` 1.12.9. Precedentes: FIND-134/135/136/137/138/139 cerrados sin push. Reanudar: S1.
