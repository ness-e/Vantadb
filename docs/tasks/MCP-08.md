# MCP-08: D4 — `VantaError::DimensionMismatch` documentado pero no ocurre vía MCP

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 2)
- **Fuente:** Backlog P22 `MCP-08`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Docs
- **Turns estimados:** 5-10
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T14:30
- **Estado:** ✅ COMPLETO
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps (3/3 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | lectores de la skill |
| Callees | `skills/vantadb-mcp/references/api-reference.md` (Error Handling, L272) |
| Implicaciones | Doc del error debe coincidir con la realidad post-fix MCP-04 |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `skills/vantadb-mcp/references/api-reference.md`
- **Veredicto impacto:** BAJO — markdown; sync a `.opencode/skills/`

## Contrato
"`skills/vantadb-mcp/references/api-reference.md` documenta el error `DimensionMismatch` tal como se entrega vía MCP (isError content con expected/got) tras MCP-04; copia sync a `.opencode/skills/` con hash SAME"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Verificar fix MCP-04
- **Archivos:** task file MCP-04
- **Acción:** confirmar que MCP-04 está ✅ (error DimensionMismatch llega al cliente MCP)
- **Verify:** estado de MCP-04
- **Estado:** ✅ COMPLETO

### Step 2: Editar doc
- **Archivos:** `skills/vantadb-mcp/references/api-reference.md` (Error Handling)
- **Acción:** documentar la variante y su formato MCP exacto
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/references/api-reference.md`
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- MCP-04 (fix) — ⏸️ BLOQUEADO

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Veredicto:** ⏳ pendiente

## Notas
- Editar en `skills/` primero

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-09
