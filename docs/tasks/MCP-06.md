# MCP-06: D2 — `distance_metric` documentado como funcional (sin efecto observable)

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 2)
- **Fuente:** Backlog P22 `MCP-06`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
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
| Callees | `skills/vantadb-mcp/SKILL.md` (L118), `skills/vantadb-mcp/references/api-reference.md` |
| Implicaciones | Doc debe reflejar el comportamiento real de `distance_metric` (por-request vs config-time) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `skills/vantadb-mcp/SKILL.md` (L118), `references/api-reference.md`
- **Veredicto impacto:** BAJO — markdown; sync a `.opencode/skills/`

## Contrato
"`skills/vantadb-mcp/SKILL.md` y `api-reference.md` describen correctamente si `distance_metric` es por-request (post-fix MCP-02) o config-time (si MCP-02 eligió (b)); copia sync a `.opencode/skills/` con hash SAME"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Verificar decisión de MCP-02
- **Archivos:** task file MCP-02
- **Acción:** leer si MCP-02 eligió (a) propagar métrica o (b) config-time
- **Verify:** estado de MCP-02
- **Estado:** ✅ COMPLETO

### Step 2: Editar doc
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (L118), `references/api-reference.md`
- **Acción:** alinear doc con la decisión real
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md`, `.opencode/skills/vantadb-mcp/references/api-reference.md`
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- MCP-02 (decisión sobre distance_metric) — ⏸️ BLOQUEADO

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
- **Próxima tarea:** MCP-07
