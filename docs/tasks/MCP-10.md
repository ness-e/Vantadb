# MCP-10: F1 — Sintaxis IQL real NO documentada (la más grave)

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 3)
- **Fuente:** Backlog P22 `MCP-10`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
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
| Callers | usuarios de Graph Operations vía MCP (query_iql) |
| Callees | `skills/vantadb-mcp/SKILL.md` (Graph Operations, L127-129), `vantadb-mcp/tests/mcp_tests.rs:462` (sintaxis de referencia), parser IQL |
| Implicaciones | Sin ejemplos, la sintaxis real es indescubrible (los 4 scripts de prueba tropezaron) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `skills/vantadb-mcp/SKILL.md` (Graph Operations); `vantadb-mcp/tests/mcp_tests.rs:462` para sintaxis exacta verificada
- **Veredicto impacto:** BAJO — markdown; sync a `.opencode/skills/`

## Contrato
"`skills/vantadb-mcp/SKILL.md` incluye sección 'IQL Syntax' con la gramática verificada y 1 ejemplo por statement (INSERT/FROM/UPDATE/DELETE/RELATE) + nota 'LISP no soportado'; copia sync a `.opencode/skills/` con hash SAME"

## Sintaxis verificada (2026-08-17, fuente mcp_tests.rs + parser + prueba en vivo)
- `INSERT NODE#id TYPE X { campo: "val" }` (+ opcional `VECTOR [0.1, -0.4, 0.9]`)
- `FROM NODE#id`
- `UPDATE NODE#id SET campo = "val"` (SET + `=`, NO `{ }`)
- `DELETE NODE#id`
- `RELATE NODE#a--"label"-->NODE#b [WEIGHT 0.9]` (edges)
- `INSERT MESSAGE SYSTEM "..." TO THREAD#7`
- NO es Cypher; `LINK` no existe (ver F7/MCP-12)

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** la sintaxis documentada debe ser la VERIFICADA en tests/parser (no inventar); editar en `skills/vantadb-mcp/`, copiar a `.opencode/skills/vantadb-mcp/`
- **Comandos de verificación:** diff vacío entre copias; cada ejemplo coincide con `vantadb-mcp/tests/mcp_tests.rs`
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Extraer sintaxis verificada
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs` (L462+), parser IQL
- **Acción:** verificar cada statement con su forma exacta (INSERT/FROM/UPDATE/DELETE/RELATE) + ejemplo real del test
- **Verify:** ejemplos citados del código
- **Estado:** ✅ COMPLETO

### Step 2: Escribir sección "IQL Syntax"
- **Archivos:** `skills/vantadb-mcp/SKILL.md` (Graph Operations)
- **Acción:** agregar sección con gramática + 1 ejemplo por statement + nota "LISP no soportado" + referencia a F7 (comportamientos de borde)
- **Verify:** lectura del diff
- **Estado:** ✅ COMPLETO

### Step 3: Sync a .opencode/skills
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md`
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual en ambas rutas
- **Estado:** ✅ COMPLETO

## Dependencias
- Ninguna (Bloque 3 es independiente del Bloque 1)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (docs)
- **Enfoque:** ¿la sintaxis documentada coincide EXACTAMENTE con tests/parser? (no inventar gramática)
- **Veredicto:** ⏳ pendiente

## Notas
- La sintaxis IQL fue el bloqueador histórico (se asumía Cypher); ahora verificada — documentar sin ambigüedad

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** ninguna
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-11
