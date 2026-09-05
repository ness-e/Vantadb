# MCP-12: F4/F5/F7/F8/F9/F10/F11 — Comportamientos de borde no documentados

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 3)
- **Fuente:** Backlog P22 `MCP-12`
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
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
| Callers | usuarios del MCP en casos de borde |
| Callees | `skills/vantadb-mcp/SKILL.md` (sección "Behavior Notes" nueva), `references/api-reference.md` |
| Implicaciones | La skill solo describe casos felices; los bordes sorprenden (LINK falla silenciosa, FROM multi-id, etc.) |

## Detalle de los 7 puntos (verificados en batería 2026-08-17)
- **F4:** `memory_get` not-found → `isError` content `"Record not found"` (no JSON-RPC error)
- **F5:** cursor de `memory_list` es offset numérico (usar `next_cursor` como `cursor`)
- **F7:** parser IQL: `LINK` no existe (falla silenciosa — inserta sin edge); trailing garbage aceptado; `FROM NODE#a,b` multi-id devuelve solo el primero; `FROM` de nodo borrado → `[]`
- **F8:** `get_node_neighbors` solo muestra edges salientes hacia nodos vivos (dangling omitidos)
- **F9:** `read_axioms` devuelve 4 objetos `{id, name, description}`
- **F10:** `rehydrate` requiere nodos archivados por summary previo — no alcanzable con tools MCP solas (una pasada simple → `recovered_count: 0`)
- **F11:** `memory_put` devuelve campos extra `version`/`node_id`/`expires_at_ms` (solo en api-reference, no en SKILL.md)

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** cada nota de comportamiento debe estar respaldada por evidencia (test script o sesión), no por intuición; editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Compilar evidencia de cada punto
- **Archivos:** scripts de prueba en temp (test-memoria.py, test-grafo.py)
- **Acción:** verificar cada uno de los 7 puntos contra los resultados reales de la batería; anotar el test/evidencia que lo respalda
- **Verify:** cada punto con fuente citada
- **Estado:** ✅ COMPLETO

### Step 2: Escribir sección "Behavior Notes"
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (sección nueva)
- **Acción:** agregar sección con cada punto + 1 línea de ejemplo/evidencia
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md` (+ api-reference si F11 lo requiere)
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- Ninguna (Bloque 3 independiente)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Enfoque:** ¿cada comportamiento documentado tiene evidencia real?
- **Veredicto:** ⏳ pendiente

## Notas
- F10 (rehydrate) puede merecer nota de limitación explícita: "no alcanzable sin summary previo"

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-13
