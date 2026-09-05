# FND-22: Guía de contribución + triage de issues (P20d, prio 🟡)

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ✅ COMPLETED
- **Agente:** vanta-docs

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `.opencode/AGENTS.md` (Reglas 1-11, Ritual, Doc Language Split), `AGENTS.md` raíz, `docs/Backlog.md` (P20d), `Justfile`, `CONTRIBUTING.md` (estado actual), `.github/ISSUE_TEMPLATE/bug_report.yml`, `feature_request.yml`, `config.yml`, `.github/pull_request_template.md`, `docs/plans/2026-08-16-wave-p20-tsys.md`.
- **Referencias hacia dentro (hacia CONTRIBUTING.md):** `.opencode/AGENTS.md:342` referencia `.github/CONTRIBUTING.md` (NO existe — ruta rota; CONTRIBUTING vive en raíz). No se toca AGENTS.md (fuera de scope) — se registra como deuda.
- **Referencias salientes (desde CONTRIBUTING.md):** `just verify` (Justfile:52), `dev-tools/verify.ps1`, `dev-tools/verify_changed.ps1`, `dev-tools/setup_venv.ps1/.sh`, `dev-tools/scripts/validate_python_sdk.ps1/.sh`, `scripts/validate-docs-coverage.ps1` — TODOS verificados existentes con Test-Path.
- **Veredicto:** editar CONTRIBUTING.md existente (expandir secciones faltantes) — NO recrear. No hay estructura de triage en `.github/` → guía como sección "Issue Triage" en CONTRIBUTING.md (opción indicada en la tarea). No tocar Backlog/plan/verify-log/task-files ajenos.

## Contrato (verify mecánico)
"CONTRIBUTING.md existe en raíz con: setup, conventional commits, flujo release (release-plz, no tocar versiones), gates (just verify); guía de triage con clasificación + derivación por dominio."
Verify: grep de secciones en CONTRIBUTING.md (`## Commit Convention`, `## Branch & PR Flow`, `just verify`, `## Issue Triage`, `## Where to Find Work`) + comandos citados existen (Test-Path, ya corrido ✅).

## Pasos
### Step 1: DISCOVERY — leer reglas, backlog, justfile, .github/ — ✅
- Leídos: `.opencode/AGENTS.md` (Regla 7 = release workflow + conventional commits tabla; Regla 1 = pre-push gate; Regla 2 = flaky; Doc Language Split = inglés para docs técnicas), `AGENTS.md` raíz, Backlog P20d (FND-22:516), `Justfile` (verify = fmt+clippy+test+deny; verify-quick; docs), templates de issues (labels: bug/triage/enhancement; áreas: Rust SDK, Python, CLI, server, storage, vector, text, CI, docs).
- Verificados existentes: todos los comandos a citar (Test-Path ✅ — ver Impacto mapeado).
- Hallazgo: CONTRIBUTING.md ya existe (146 líneas: setup, tests, fuzzing, release checklist) pero sin conventional commits, flujo PR explícito, gates `just verify`, ni triage.

### Step 2: Expandir CONTRIBUTING.md — secciones de commits y flujo — ✅
- `## Commit Convention`: tabla semver (feat→minor, fix→patch, docs/test/perf/refactor→patch, ci/chore→no release, feat!/BREAKING→major) con ejemplos del repo (AGENTS.md Regla 7, `.opencode/AGENTS.md:508-531`), nota "commits sin conventional → release-plz los ignora", referencia a Regla 7.
- `## Branch & PR Flow`: develop→main, release-plz Release PR, NUNCA tocar versión en Cargo.toml / CHANGELOG / tags, pre-push gate (just verify / dev-tools/verify.ps1), CI fast gate (`.github/workflows/ci-gate.yml`, dos tiers — CI_POLICY).

### Step 3: Gates y dónde mirar — ✅
- `## Code Quality` actualizado: citar `just verify` como gate único (fmt+clippy+test+deny) + `just verify-quick` para feedback rápido + `just docs` (coverage de docs).
- `## Where to Find Work` (NUEVA): `docs/Backlog.md` (formato ID | descripción | archivos | esfuerzo | prio | estado), etiquetas `good first issue` / `bug` / `enhancement`, enlace a reglas `.opencode/rules/` y AGENTS.md.

### Step 4: Guía de triage — ✅
- `## Issue Triage` (NUEVA): clasificación bug/feature/perf/security + etiquetas existentes (bug, triage, enhancement) + flujo de triage (reproducir → clasificar → etiquetar → derivar) + derivación por dominio según el sistema de agentes: core/engine → vanta-engine, bindings/SDK → vanta-worker, seguridad → vanta-audit, perf → vanta-tuner, docs → vanta-docs, stress/concurrencia → vanta-chaos, review → vanta-review. Nota de seguridad: reportar vía SECURITY.md (config.yml ya lo enlaza).
- Verificar que las secciones existen con grep.

### Step 5: Verify mecánico — ✅
- grep de secciones en CONTRIBUTING.md: `## Commit Convention`, `## Branch & PR Flow`, `## Where to Find Work`, `## Issue Triage`, `just verify`, `release-plz`, `NUNCA`/`Never manually edit`.
- Comandos citados: ya verificados con Test-Path en DISCOVERY (todos True).

## Dependencias
- Ninguna nueva. Solo edición de `CONTRIBUTING.md` (raíz). No se tocan Backlog, plan file, verify-log, ni otros task files.

## Notas
- **Idioma:** CONTRIBUTING.md en inglés (fuente de verdad para docs técnicas — Doc Language Split). Planning quedó en español aquí.
- **Deuda detectada:** `.opencode/AGENTS.md:342` cita `.github/CONTRIBUTING.md` que no existe (el archivo vive en raíz). Fuera de scope de FND-22 — sugerir corregir la referencia en otro task.

## Context Save Point
- **Fecha:** 2026-08-16
- **Branch:** develop
- **Commit:** ninguno (lead commitea — regla de la tarea)
- **CI pendiente:** no (solo docs; verify mecánico = grep de secciones + Test-Path de comandos)
- **Decisiones:** expandir CONTRIBUTING.md existente en vez de recrearlo (ya cubre setup/tests/fuzzing); triage como sección de CONTRIBUTING (no existe estructura en .github/); inglés; citar reglas existentes (AGENTS.md Regla 7, CI_POLICY, manual) en vez de duplicarlas.
- **Problemas conocidos:** ninguna.
- **Próxima tarea:** la que asigne el orquestador (P20d: FND-23, FND-24).