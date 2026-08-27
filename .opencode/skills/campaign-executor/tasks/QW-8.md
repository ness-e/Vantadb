# QW-8: posicionamiento en READMEs

## Metadata
- **Plan file:** docs/plans/2026-08-25-integrations-research-wins.md
- **Fuente:** Wave 3 · QW-8 (H-11)
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟡 Media
- **Tipo:** Documentation
- **Turns estimados:** 4
- **Creado:** 2026-08-27T23:30
- **last-synced:** 2026-08-27T23:45
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Usuarios que leen `integrations/*/README.md` |
| Callees | `engine embebido Rust local-first` vs `zep`/`cognee`/`memoria nativa` |
| Implicaciones | Sin cambio de código, solo docs. Sin claims sin benchmark (Regla 11). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `integrations/*/README.md` ×9 (verificados con grep Why VantaDB)
- **Archivos referenciados hacia dentro:** README `## Why VantaDB` sections
- **Archivos que referencian a los editados:** `docs/Backlog.md` QW-8
- **Veredicto impacto:** bajo — docs-only

## Contrato
"cada README de adapter tiene sección 'Why VantaDB' honesta: engine embebido Rust local-first vs zep (requiere servidor) / cognee (KG runtime propio) / memoria nativa del framework (cuándo basta la nativa). Sin claims de performance sin benchmark (Regla 11)."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Cada README tiene `## Why VantaDB` con comparativa honesta, sin `recall>0.998` o `zero deps` sin fuente
- **Comandos de verificación:** `grep -r "Why VantaDB" integrations/*/README.md` 9/9
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | QW-8 — posicionamiento en READMEs |
| `lastAction` | Steps 1-3 ✅ — grep Why VantaDB 9/9 |
| `result` | OK |
| `nextAction` | Lead: archivar plan |
| `contract` | Contrato arriba |
| `nextTask` | QW-9 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | 9 READMEs con Why VantaDB ✅ |
| **Commit** | Verify-only — ya existen |
| **Release** | No aplica |

## Herramientas necesarias
- documentation-and-adrs, ponytail

**Skills cargadas (SDP):** documentation-and-adrs, ponytail

## Investigation Notes
- Verificado grep 9/9 READMEs con Why VantaDB.

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

### Step 1: Verificar Why VantaDB en 9 READMEs
- **Archivos:** `integrations/*/README.md`
- **Acción:** grep Why VantaDB
- **Verify:** `grep -r "Why VantaDB" integrations/*/README.md` 9/9 ✅
- **Estado:** ✅ COMPLETED

### Step 2: Verificar sin claims sin benchmark
- **Archivos:** `integrations/*/README.md`
- **Acción:** grep recall|zero deps sin fuente
- **Verify:** `grep recall` 0 ✅
- **Estado:** ✅ COMPLETED

### Step 3: Verify full
- **Archivos:** `integrations/*/README.md`
- **Acción:** docs coverage
- **Verify:** `grep Why VantaDB` 9/9 ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- QW-7 — ✅ COMPLETED

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-docs
- **Enfoque:** docs quality
- **Cómo se probó:** grep
- **Veredicto:** ✅ approve

## Notas
- Docs-only, no código.

## Context Save Point
- **Fecha:** 2026-08-27T23:45 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** Why VantaDB 9/9
- **Problemas conocidos:** ninguno
- **Próxima tarea:** QW-9
