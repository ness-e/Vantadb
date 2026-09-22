# FIND-145 — Verificar CodeQL verde post-#202 en `main`

- **Plan:** `docs/plans/2026-09-21-workflows-repair.md` (Wave 4)
- **Tipo:** devops (verificación read-only, sin fix)
- **Contrato:** check verde o causa declarada con dueño; `actionlint` 0 si se toca YAML
- **SDP:** systematic-debugging · ci-cd-and-automation · security-and-hardening (discovery BUILD keywords codeql/security/analysis/actionlint)

## Impacto mapeado (Regla 0)

- **Leído completo:** `.github/workflows/sec-codeql-30.yml` (37L: push/PR `main`, schedule semanal, `concurrency` sin cancel, job `Analyze` con checkout v7.0.1 + init v4.38.1 + analyze v4.38.0, lenguajes rust/python/javascript-typescript, build-mode none)
- **Referencias hacia dentro:** ninguna (workflow standalone, sin reusable/local actions)
- **Referencias entrantes:** check externo "CodeQL" (GitHub code scanning) agregado por servicio, no declarado en YAML
- **Veredicto:** blast radius = 1 archivo, solo lectura. Sin edición salvo rojo con fix mínimo seguro.

## Discovery (solo GET, systematic-debugging Fase 1)

- `gh run list --workflow=sec-codeql-30.yml --limit 15`: tope `35693192298` (`main`, push, `success`, 28m44s, 2026-09-22T06:03) — POST-#202.
- `gh run view 35693192298 --json jobs`: `Analyze {completed, success}`.
- `gh run view 35648430919` (merge #202, `bcc92693`): `{completed, success}` en `main`.
- Serie post-#202 en `main` push: `35664428706` success, `35664461751` success, `35681585401` success, `35693192298` success. Dos `cancelled` de 5s son superados por pushes simultáneos (concurrency `cancel-in-progress: false` no cancela; son runs reemplazados en cola, no fallo).
- Hipótesis previa (baseline stale "configurations not found", job interno verde + check externo rojo) → **no reproducible**: el job Analyze está verde en `main` y en el merge #202; sin evidencia de rojo actual no hay causa que diagnosticar ni fix que aplicar.

## Steps

- [x] S1 DISCOVERY: workflow leído + serie de runs post-#202 verificada (solo GET)
- [x] S2 VEREDICTO: verde — sin cambio, sin `actionlint` (no se tocó YAML)
- [x] S3 CIERRE: task file + commit selectivo, sin push

## Evidencia

- `gh run list`: `35693192298 success main push 28m44s 2026-09-22` (post-#202)
- `gh run view 35693192298`: `Analyze completed/success`
- `gh run view 35648430919` (#202 `bcc92693`): `completed/success`
- `git status`: limpio salvo este task file; `git diff --check`: limpio

## Deuda / Invariantes

- Deuda: ninguna (causa stale-baseline calmada sola tras #202; si el check externo re-enrojece, reabrir con log del run rojo).
- Invariantes: resto workflows / `src/` / `web/` / `desktop/` / locks / plans / Backlog intactos; NO PUSH; sin merge/aprobación de deploys.
