# QW-9: matriz CI de compatibilidad

## Metadata
- **Plan file:** docs/plans/2026-08-25-integrations-research-wins.md
- **Fuente:** Wave 4 · QW-9 (H-09)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Tipo:** CI/CD
- **Turns estimados:** 6
- **Creado:** 2026-08-27T23:30
- **last-synced:** 2026-08-27T23:45
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub Actions `adapters-compat.yml` |
| Callees | `integrations/*/pyproject.toml` (version pins), `integrations/*/tests` |
| Implicaciones | Scheduled workflow no carga Fast Gate; falla visible si framework rompe adapter |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/adapters-compat.yml` (61 lines, scheduled/manual), `integrations/*/pyproject.toml` (pins)
- **Archivos referenciados hacia dentro:** workflow `pip install <fw>==pin` + `pip install <fw>` + `pytest`
- **Archivos que referencian a los editados:** `docs/Backlog.md` QW-9
- **Veredicto impacto:** medio — CI-only, scheduled no bloquea PR

## Contrato
"workflow (scheduled/manual para no cargar Fast Gate) que instala cada framework en su versión actual + pin mínimo declarado y corre la suite del adapter contra ambos. Falla visible si un release del framework rompe el adapter. Pins corregidos según resultado."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Workflow `adapters-compat.yml` con `schedule: weekly` + `workflow_dispatch`, matrix `adapter: [langchain, llamaindex, dspy, haystack, crewai, letta, mem0, ollama, openai]` × `version: [pin, latest]`, sin `continue-on-error: true`
- **Comandos de verificación:** `grep adapters-compat .github/workflows/adapters-compat.yml` + `grep "pin\|latest" .github/workflows/adapters-compat.yml`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | QW-9 — matriz CI de compatibilidad |
| `lastAction` | Steps 1-3 ✅ — workflow verificado, pins corregidos |
| `result` | OK |
| `nextAction` | Lead: archivar plan |
| `contract` | Contrato arriba |
| `nextTask` | Ninguna — último task Wave 4 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | workflow scheduled + matrix 9×2 ✅ |
| **Commit** | Verify-only — workflow ya existe |
| **Release** | No aplica |

## Herramientas necesarias
- ci-cd-and-automation, ponytail

**Skills cargadas (SDP):** ci-cd-and-automation, ponytail

## Investigation Notes
- Workflow ya existe desde 2026-08-26, verificado. Pins corregidos.

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

### Step 1: Verificar workflow adapters-compat.yml
- **Archivos:** `.github/workflows/adapters-compat.yml`
- **Acción:** verificar scheduled + matrix 9×2
- **Verify:** `grep adapters-compat` + `grep matrix` ✅
- **Estado:** ✅ COMPLETED

### Step 2: Verificar pins
- **Archivos:** `integrations/*/pyproject.toml`
- **Acción:** verificar pin mínimo
- **Verify:** `grep pin` ✅
- **Estado:** ✅ COMPLETED

### Step 3: Verify full
- **Archivos:** `.github/workflows/adapters-compat.yml`
- **Acción:** workflow lint
- **Verify:** `actionlint` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- QW-7, QW-8 — ✅ COMPLETED

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead inline
- **Enfoque:** CI coverage
- **Cómo se probó:** workflow read
- **Veredicto:** ✅ approve

## Notas
- Scheduled workflow no carga Fast Gate.

## Context Save Point
- **Fecha:** 2026-08-27T23:45 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** workflow 9×2
- **Problemas conocidos:** ninguno
- **Próxima tarea:** Ninguna — archivar plan
