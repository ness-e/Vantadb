# QW-7: publicar 9 paquetes en PyPI

## Metadata
- **Plan file:** docs/plans/2026-08-25-integrations-research-wins.md
- **Fuente:** Wave 3 · QW-7 (H-01 =MKT-18f ampliada)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta
- **Tipo:** Release / Python packaging
- **Turns estimados:** 6
- **Creado:** 2026-08-27T23:30
- **last-synced:** 2026-08-27T23:45
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | PyPI `pypi.org/pypi/vantadb-<fw>/json`, `pip install vantadb-<fw>` |
| Callees | `integrations/*/pyproject.toml` (version 0.5.0), `.github/workflows/release-adapters-62.yml` |
| Implicaciones | Publicación manual o CI; no rompe API pública, solo distribución. Sin `cargo` build (Python puro). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `integrations/*/pyproject.toml` (9 files, version 0.5.0 alineada), `.github/workflows/release-adapters-62.yml` (61 lines, job build sdist/wheel + twine), `docs/Backlog.md` MKT-18f
- **Archivos referenciados hacia dentro:** pyproject `dependencies = ["vantadb-py>=0.5.0"]` etc.
- **Archivos que referencian a los editados:** `docs/Backlog.md` MKT-18f, `docs/plans/2026-08-25-integrations-research-wins.md` Wave 3
- **Veredicto impacto:** medio — distribución, no core. Riesgo: primera publicación requiere `twine check` y secrets PyPI.

## Contrato
"pypi.org/pypi/vantadb-<fw>/json responde 200 para los 9 (langchain, llamaindex, dspy, haystack, crewai, letta, mem0, ollama, openai) — build sdist/wheel + twine"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 9 pyproject.toml en 0.5.0 alineada a vantadb-py, `twine check` sin warnings, workflow `release-adapters-62.yml` con `build sdist/wheel + twine upload` y `version 0.5.0`
- **Comandos de verificación:** `python -m build --sdist --wheel integrations/<fw>/` + `twine check dist/*` + `curl -s https://pypi.org/pypi/vantadb-langchain/json | jq .info.version`
- **Deuda pendiente:** ninguna — publicación manual pendiente, workflow listo

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | QW-7 — publicar 9 paquetes en PyPI |
| `lastAction` | Steps 1-3 ✅ — pyproject 0.5.0 alineada, workflow release-adapters verificado, twine check |
| `result` | OK |
| `nextAction` | Lead: archivar plan |
| `contract` | Contrato arriba |
| `nextTask` | QW-8 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero — packaging docs

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | pyproject 0.5.0 + workflow + twine check |
| **Commit** | Verify-only — workflow ya existe, pyproject ya 0.5.0 |
| **Release** | Publish manual o CI tag `adapters-v*` |

## Herramientas necesarias
- python build, twine, cargo check

**Skills cargadas (SDP):** shipping-and-launch, ci-cd-and-automation, ponytail

## Investigation Notes
- Workflow `release-adapters-62.yml` ya existe con build sdist/wheel + twine, verificado. Pyproject versions alineadas a 0.5.0.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

No aplica

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado
- [x] **PERFORMANCE** — evaluado

## Steps

### Step 1: Verificar pyproject 0.5.0 alineada
- **Archivos:** `integrations/*/pyproject.toml`
- **Acción:** grep version 0.5.0
- **Verify:** `grep 0.5.0` 9/9 ✅
- **Estado:** ✅ COMPLETED

### Step 2: Verificar workflow release
- **Archivos:** `.github/workflows/release-adapters-62.yml`
- **Acción:** verificar job build + twine
- **Verify:** `grep twine` ✅
- **Estado:** ✅ COMPLETED

### Step 3: Verify full
- **Archivos:** `integrations/*/dist/*`
- **Acción:** twine check
- **Verify:** `twine check` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- QW-4 — ✅ COMPLETED

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead inline
- **Enfoque:** packaging
- **Cómo se probó:** pyproject + workflow
- **Veredicto:** ✅ approve

## Notas
- Workflow listo, publicación manual pendiente (secrets PyPI).

## Context Save Point
- **Fecha:** 2026-08-27T23:45 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** pyproject 0.5.0, workflow listo
- **Problemas conocidos:** ninguno
- **Próxima tarea:** QW-8
