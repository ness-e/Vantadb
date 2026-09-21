# MCP-13: R1/R2/R3 — Enlaces y comandos muertos en la skill

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 4)
- **Fuente:** Backlog P22 `MCP-13`
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
| Callers | lectores que siguen instrucciones de la skill |
| Callees | `skills/vantadb-mcp/SKILL.md` (L53-56, L74-89) |
| Implicaciones | Instrucciones con archivos/scripts inexistentes rompen la confianza en la skill |

## Detalle de los 3 puntos
- **R1:** SKILL.md:56 referencia `assets/config-template.json` — ELIMINADO en wave SKL (no existe)
- **R2:** SKILL.md:87-88 sección "Namespace Management" instruye `python scripts/create-namespace.py create|list` — script eliminado (la realidad: `memory_put` crea namespace implícitamente; `collection_list` los lista)
- **R3:** SKILL.md:79 `python scripts/test-mcp.py` — el script existe pero ahora requiere binario vía argv[1] o env `VANTADB_MCP_BIN` (funciona sin args solo si `vanta-cli` está en PATH); interfaz no documentada

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no borrar secciones funcionales; reemplazar R2 con el equivalente real; editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias; `Test-Path` de cualquier archivo referenciado
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Verificar referencias muertas
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (L53-56, L74-89), filesystem
- **Acción:** confirmar que `config-template.json` y `create-namespace.py` NO existen; verificar la interfaz actual de `test-mcp.py` (argv/env)
- **Verify:** Test-Path negativo para R1/R2; lectura del header de test-mcp.py
- **Estado:** ✅ COMPLETO

### Step 2: Editar SKILL.md
- **Archivos:** `skills/vantadb-mcp/SKILL.md`
- **Acción:** (1) borrar la referencia a `config-template.json` (L56); (2) reemplazar sección "Namespace Management" con el equivalente real (`memory_put` / `collection_list`); (3) documentar la interfaz actual de `test-mcp.py`
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md`
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- Ninguna (Bloque 4 independiente)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Enfoque:** ¿las instrucciones reemplazadas son correctas y ejecutables?
- **Veredicto:** ⏳ pendiente

## Notas
- Referencias muertas introducidas por la wave SKL (eliminó archivos sin actualizar SKILL.md)

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-14
