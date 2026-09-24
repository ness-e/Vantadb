# FIND-135 — `timeout-minutes` a todos los jobs sin límite

> **Plan:** `docs/dev/plans/2026-09-21-workflows-repair.md` (Wave 0, paralelo disjunto con FIND-134/136)
> **Estado:** ⏳ IN PROGRESS
> **Tipo:** devops (CI/CD) · **Scope:** solo añadir `timeout-minutes`, cero lógica
> **SDP:** ci-cd-and-automation (sugerida tarea + base) · keywords: [github-actions, timeout, actionlint, concurrency]

## Contrato (ley)

- (a) ningún job sin `timeout-minutes` en los 5 archivos (9 jobs total)
- (b) valores = duración real medida + margen (tabla §Investigación)
- (c) `actionlint` exit 0

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/desktop.yml` (245L),
  `.github/workflows/ci-gate.yml` (58L), `.github/workflows/ocr-delegate.yml` (71L),
  `.github/workflows/ocr-nightly.yml` (156L), `.github/workflows/opencode.yml` (33L),
  `.opencode/rules/release-ci.md` (42L, solo lectura — prohibido editar `.opencode/`).
- **Referencias hacia dentro:** ninguna (YAML workflows, sin imports de código).
- **Referencias entrantes:** `ci-gate.yml` es `workflow_call` reusable (callers: heavy/scheduled —
  solo se añade timeout al job, la firma `inputs.event_name` no cambia);
  `ocr-nightly.full` tiene `needs: [check-key]` (timeout no altera DAG).
- **Veredicto:** impacto mínimo — `timeout-minutes` es kill-switch del runner, no cambia
  steps, triggers, permisos ni lógica. Gate `*)` de `ci-gate.yml:51` NO se toca (es FIND-139).

## Gate D

`no disparado` — blast radius 5 YAML sin hot path ni API pública, contrato mecánico claro
(solo timeouts), sin símbolos públicos nuevos, sin spec pendiente.

## Investigación — job → timeout + evidencia

Duraciones reales medidas con `gh run view` / `gh run list` (2026-09-22):

| Job | Timeout | Evidencia |
|---|---|---|
| `desktop.yml` windows | 60 | run 35693124393: 41.5 min (06:23→07:04) → 1.45× margen |
| `desktop.yml` macos | 45 | mismo run: 24 min (06:30→06:54) → 1.9× margen |
| `desktop.yml` linux | 30 | mismo run: 11 min (06:06→06:17) → 2.7× margen |
| `ci-gate.yml` ci-gate | 5 | loop `gh api` de ~11 checks, segundos; reusable sin runs directos |
| `ocr-delegate.yml` ocr-delegate | 15 | 5 runs recientes: 39s–9.9 min (max ~10) → 1.5× margen |
| `ocr-nightly.yml` delegate | 15 | runs 09-21/22: ~23s + self-test loop; 15 cubre ventana 24h grande |
| `ocr-nightly.yml` check-key | 5 | 1 step env-only, segundos |
| `ocr-nightly.yml` full | 30 | sin runs con key observados; LLM review 24h → conservador |
| `opencode.yml` opencode | 30 | solo runs `skipped`; agent LLM → estimación conservadora |

Internet: N/A (defaults GitHub 360 min conocidos, sin ambigüedad API).

## Steps (~100 líneas c/u, archivos disjuntos)

- [x] S1 · `desktop.yml` — `timeout-minutes: 60/45/30` en windows/macos/linux
- [x] S2 · `ci-gate.yml` — `timeout-minutes: 5` en job `ci-gate`
- [x] S3 · `ocr-delegate.yml` — `timeout-minutes: 15` en job `ocr-delegate`
- [x] S4 · `ocr-nightly.yml` — `15/5/30` en delegate/check-key/full
- [x] S5 · `opencode.yml` — `timeout-minutes: 30` en job `opencode`
- [x] S6 · Verify (`actionlint` + `git diff --check`) + commit `ci:` selectivo (NO PUSH)

## Verificación (2026-09-22)

- `actionlint` (5 archivos y full repo): exit 0
- `git diff --check`: limpio
- `campaign_verify_cmd`: BUG exit -1 documentado (tarea §6) → verificación real vía bash directa
- 9/9 jobs con `timeout-minutes`; diff total = 9 líneas añadidas, cero lógica

## Prohibidos (intocables)

`ci-rust-10.yml` y resto de workflows (FIND-134/136 en paralelo), `src/`, `web/`,
`desktop/` código, `reparacion.bat`, `.opencode/` (edición), `completions/*`, `*.lock`,
stash GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation), `C:/Users/Eros/.vantadb*`, secretos.
