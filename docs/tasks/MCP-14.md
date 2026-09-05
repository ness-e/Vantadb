# MCP-14: I1/I2/I3/I4 — Contradicciones internas de la skill

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 5)
- **Fuente:** Backlog P22 `MCP-14`
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
| Callers | lectores que configuran VantaDB |
| Callees | `skills/vantadb-mcp/SKILL.md` (L20, L60-72, L236, L242), `references/configuration.md` |
| Implicaciones | Claims falsos (config file, HNSW env, read-only env) y ejemplo OpenCode roto (`~` no expandido) |

## Detalle de los 4 puntos
- **I1:** Quick Start L20 dice setup "creates default configuration" pero NO existe config file (`configuration.md:3` — "there is no config.json")
- **I2:** Performance Optimization L236 "Adjust HNSW parameters" — HNSW no se expone vía env vars (`configuration.md:155` — solo programático)
- **I3:** Security L242 "read-only mode" — no existe `VANTADB_READ_ONLY` (`configuration.md:167` — solo programático vía SDK)
- **I4:** ejemplo OpenCode L67 usa `~/.vantadb` — OpenCode NO expande `~` en spawn directo (verificado en sesión; `opencode.jsonc` usa ruta absoluta `C:/Users/Eros/.vantadb`)

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** el ejemplo OpenCode debe usar ruta absoluta (o relativa al cwd) con nota; no inventar env vars que no existen; editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias; cada claim verificado contra `configuration.md`
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Verificar cada contradicción
- **Archivos:** `skills/vantadb-mcp/SKILL.md`, `references/configuration.md`
- **Acción:** confirmar los 4 puntos (config file inexistente, HNSW/read-only programáticos, `~` no expandido)
- **Verify:** lectura de configuration.md (L3, L155, L167)
- **Estado:** ✅ COMPLETO

### Step 2: Corregir SKILL.md
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (L20, L60-72, L236, L242)
- **Acción:** (I1) quitar claim de config file creado; (I2) aclarar HNSW programático; (I3) aclarar read-only programático; (I4) usar ruta absoluta en ejemplo OpenCode con nota
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md`
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- Ninguna (Bloque 5 independiente)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Enfoque:** ¿cada corrección está respaldada por configuration.md real?
- **Veredicto:** ⏳ pendiente

## Notas
- El ejemplo `~` fue verificado en vivo en la sesión (opencode.jsonc con ruta absoluta)

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** — (fin de P22)
