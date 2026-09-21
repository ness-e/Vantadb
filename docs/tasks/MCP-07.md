# MCP-07: D3 — `search_semantic` distances son similaridad invertida (doc)

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 2)
- **Fuente:** Backlog P22 `MCP-07`
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
| Callees | `skills/vantadb-mcp/SKILL.md` (L121-123), `references/api-reference.md` (L299-306) |
| Implicaciones | Doc debe explicar el significado real del campo (distancia o similaridad) tras MCP-03 |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `skills/vantadb-mcp/SKILL.md` (L121-123), `references/api-reference.md`
- **Veredicto impacto:** BAJO — markdown; sync a `.opencode/skills/`

## Contrato
"`skills/vantadb-mcp/SKILL.md` documenta el significado exacto del campo devuelto por `search_semantic` (distancia real o similaridad, con rango) alineado con el fix MCP-03; copia sync a `.opencode/skills/` con hash SAME"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Verificar decisión de MCP-03
- **Archivos:** task file MCP-03
- **Acción:** leer si MCP-03 eligió (a) distancia real o (b) renombrar a similarity
- **Verify:** estado de MCP-03
- **Estado:** ✅ COMPLETO

### Step 2: Editar doc
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (L121-123), `references/api-reference.md` (L299-306)
- **Acción:** documentar el significado exacto del campo y su rango
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md`, `.opencode/skills/vantadb-mcp/references/api-reference.md`
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- MCP-03 (decisión semver) — ⏸️ BLOQUEADO

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
- **Próxima tarea:** MCP-08
