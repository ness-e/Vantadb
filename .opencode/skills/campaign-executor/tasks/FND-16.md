# Task FND-16 — Multi-target CI (wheels + WASM por PR)

- **Prioridad:** 🟢 · **Wave:** P20c · **Estado:** ✅ ANALYSIS COMPLETE (pendiente aprobación lead para implementar)
- **Tipo:** research/analysis (NO implementación — aprobación del lead antes de tocar workflows)
- **Owner:** vanta-lead

## Objetivo

Determinar si los builds multi-target (wheels Windows/Mac/Linux + WASM) corren en cada PR o solo en release, analizar el costo, y entregar un plan aprobable o una decisión de defer justificada. **NO implementar el job.**

## Archivos clave

- `.github/workflows/release-wheels-60.yml` — wheels Python (maturin)
- `.github/workflows/release-npm-61.yml` — WASM (wasm-pack) + TS
- `.github/workflows/ci-rust-10.yml` — Fast Gate
- `.github/workflows/release.yml` — release-plz (grep dirigido)
- Entregable: `docs/Investigaciones/FND-16-multi-target-ci.md`

## Pasos

1. ✅ DISCOVERY: mapear jobs que compilan wheels (maturin) y WASM (wasm-pack), triggers, matrix OS × target. Citar archivo:línea. → Hecho: wheels ya corren en PR (`release-wheels-60.yml:10-17`); WASM solo en tag/dispatch (`release-npm-61.yml:16-17`); wasm-test push-only non-blocking (`ci-rust-10.yml:388-395`).
2. ✅ Análisis de costo: minutos × matrix por PR; qué se valida hoy en PR; identificar el gap real. → Gap = wasm32+TS no se compilan en PR; costo incremental ~1 runner × ~10min solo con paths filter.
3. ✅ Decisión: plan de job multi-target por PR O defer con razón. → PLAN (no defer): agregar trigger `pull_request` con paths filter a `release-npm-61.yml`, reusando job `tests` existente. Sin tocar publish.
4. ✅ Escribir análisis en `docs/Investigaciones/FND-16-multi-target-ci.md` → creado (129 líneas).

## Contrato (verify mecánico)

- Análisis existe en `docs/Investigaciones/` con triggers/matrix citados (archivo:línea de los workflows).
- Plan propuesto o decisión de defer explícita.

## Reglas

- NO git add/commit (el lead commitea al cerrar la wave).
- NO modificar workflows — solo el entregable en `docs/Investigaciones/`.
- NO tocar `docs/Backlog.md`, `AUD-024.md`, `verify-log.jsonl`, `completions/_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`.
- NO usar `campaign_update_task_state`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** los 4 workflows (release-wheels-60 295L, release-npm-61 199L, ci-rust-10 602L, release.yml vía grep).
- **Referencias hacia dentro:** ninguno de los archivos a escribir referencia estos workflows; el entregable es un doc nuevo en `docs/Investigaciones/`.
- **Referencias entrantes:** `docs/Backlog.md:508` (FND-16), `docs/plans/2026-08-16-wave-p20-tsys.md:24,58`.
- **Veredicto:** solo se CREAN 2 archivos nuevos (task file + análisis). Cero archivos existentes modificados → impacto nulo.

## Resultado

> RESULTADO: ✅ COMPLETO (análisis) — decisión PLAN: agregar trigger PR a release-npm-61.yml, pendiente aprobación del lead antes de implementar.