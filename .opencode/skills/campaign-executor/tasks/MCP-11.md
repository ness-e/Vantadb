# MCP-11: F2/F3/F6 — Envelope MCP + wire-format + canales de error

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 3)
- **Fuente:** Backlog P22 `MCP-11`
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Docs
- **Turns estimados:** 15-30
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T14:30
- **Estado:** ✅ COMPLETO
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps (3/3 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | integradores MCP (los 4 scripts de prueba tropezaron con el envelope) |
| Callees | `skills/vantadb-mcp/SKILL.md` (Returns de cada tool), `references/api-reference.md` (VantaValue), `references/mcp-protocol.md` |
| Implicaciones | Sin documentar, integrar contra el MCP requiere adivinar el protocolo |

## Detalle de los 3 puntos
- **F2 Envelope:** todas las respuestas reales son `{"content":[{"type":"text","text":"<json>"}]}` con `isError` — no el JSON directo que asumen los "Returns:" de la skill
- **F3 Wire-format:** metadata entrada JSON plano `{"priority":2}`, salida serde-tagged `{"priority":{"Int":2}}`
- **F6 Canales mixtos:** `rehydrate` con id inválido → JSON-RPC `-32602`; `get_node_neighbors` con nodo inexistente → `isError` content

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** ejemplos del envelope deben ser REALES (obtenidos de la batería de pruebas, no inventados); editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Documentar envelope MCP (F2)
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (Returns), `references/mcp-protocol.md`
- **Acción:** explicar el envelope `{"content":[{"type":"text","text":"<json>"}]}` + `isError`; ejemplo de cómo extraer payload/isError
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 2: Documentar wire-format de VantaValue (F3)
- **Archivos:** `references/api-reference.md` (VantaValue)
- **Acción:** documentar entrada plana / salida serde-tagged (`{"priority":{"Int":2}}`) con ejemplo
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Documentar canales de error (F6) + sync
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (o api-reference), `.opencode/skills/vantadb-mcp/...`
- **Acción:** documentar ambos canales (JSON-RPC error vs isError content) y cuándo ocurre cada uno; copiar archivos editados a `.opencode/skills/` y verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- Ninguna (Bloque 3 independiente)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Enfoque:** ¿los ejemplos del envelope son reales (de la batería)?
- **Veredicto:** ⏳ pendiente

## Notas
- Evidencia en test scripts (temp) — los sub-agentes de la batería documentaron el envelope real

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-12
