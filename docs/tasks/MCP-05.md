# MCP-05: D1 — Sección "Hybrid Search" declara búsqueda textual/híbrida no funcional

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 2)
- **Fuente:** Backlog P22 `MCP-05`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
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
| Callers | lectores de la skill (agentes OpenCode/Claude/Cursor) |
| Callees | `skills/vantadb-mcp/SKILL.md` (L117-123, L200-206) |
| Implicaciones | Claim falso sobre feature rota (MCP-01); la skill debe reflejar la realidad post-fix |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `skills/vantadb-mcp/SKILL.md` (verificar L117-123, L200-206)
- **Veredicto impacto:** BAJO — edición de markdown; requiere sync a `.opencode/skills/vantadb-mcp/`

## Contrato
"`skills/vantadb-mcp/SKILL.md` ya no promete text search/hybrid sin nota; tras el fix MCP-01 verificado, la doc coincide con el comportamiento real (o marca la limitación con referencia a MCP-01); copia sync a `.opencode/skills/vantadb-mcp/` con hash SAME"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** la skill es la copia de `skills/` → editar en `skills/vantadb-mcp/` y copiar a `.opencode/skills/vantadb-mcp/` (Regla de edición P22)
- **Comandos de verificación:** diff entre `skills/` y `.opencode/skills/` vacío para los archivos tocados
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Verificar estado post-fix MCP-01
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (L117-123, L200-206)
- **Acción:** confirmar si MCP-01 está resuelto (text search funcional) antes de tocar la doc; si NO está resuelto, marcar la feature como limitada con nota "requiere text index construido; ver MCP-01"
- **Verify:** estado de MCP-01 en Backlog
- **Estado:** ✅ COMPLETO

### Step 2: Editar SKILL.md
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (L117-123, L200-206)
- **Acción:** alinear la sección "Hybrid Search" y la tool `search_memory` con la realidad (feature funcional o nota de limitación)
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md`
- **Acción:** copiar el archivo editado; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- MCP-01 (fix del text index) — ⏸️ BLOQUEADO hasta que MCP-01 esté ✅

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Enfoque:** ¿la doc refleja la realidad verificable?
- **Veredicto:** ⏳ pendiente

## Notas
- Editar en `skills/vantadb-mcp/` (fuente versionada), NUNCA directo en `.opencode/skills/` (copia runtime)

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-06
