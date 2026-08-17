# MCP-09: D5 — Schema real vs doc: `search_semantic.k` required

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 2)
- **Fuente:** Backlog P22 `MCP-09`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Mixto (Rust handler + Docs)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T14:30
- **Estado:** ✅ COMPLETO
- **Incógnitas (uphill):** 1 (¿fix server o fix doc?)
- **Pendientes (downhill):** 0 steps (3/3 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | clientes MCP que leen el inputSchema de `search_semantic` |
| Callees | `vantadb-mcp/src/handlers/tools.rs` (schema search_semantic), `skills/vantadb-mcp/SKILL.md` (L122) |
| Implicaciones | Schema marca `k` required pero handler aplica default 5 → inconsistencia confusa |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `skills/vantadb-mcp/SKILL.md` (L122); handler tools.rs (schema search_semantic)
- **Veredicto impacto:** BAJO — UNA línea de schema o UNA línea de doc; sync a `.opencode/skills/`

## Contrato
"`inputSchema` de `search_semantic` y la skill coinciden: o `k` es opcional en el schema (fix server) o la skill dice 'k requerido en schema, opcional en runtime (default 5)' (fix doc); copia sync a `.opencode/skills/` con hash SAME si se tocó la skill"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** el handler debe seguir tolerando omisión de `k` (default 5) — no romper compatibilidad
- **Comandos de verificación:** `cargo check -p vantadb-mcp` (si fix server); diff vacío entre copias (si fix doc)
- **Deuda pendiente:** ninguna

## Steps

### Step 1: Decidir el fix
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `skills/vantadb-mcp/SKILL.md`
- **Acción:** elegir: (a) marcar `k` opcional en el schema (fix server, mejor UX), o (b) documentar la discrepancia en la skill. Recomendación: (a) — el runtime ya aplica default
- **Verify:** decisión documentada
- **Estado:** ✅ COMPLETO

### Step 2: Implementar
- **Archivos:** el elegido en Step 1
- **Acción:** si (a): quitar `k` de required en el schema JSON del handler; si (b): editar SKILL.md L122
- **Verify:** `cargo check -p vantadb-mcp` (si (a)) o lectura diff (si (b))
- **Estado:** ✅ COMPLETO

### Step 3: Sync + verificación
- **Archivos:** `.opencode/skills/vantadb-mcp/SKILL.md` (si se tocó la skill)
- **Acción:** copiar; verificar hash SAME
- **Verify:** `Get-FileHash` igual; `search_semantic` sin `k` funciona (default 5)
- **Estado:** ✅ COMPLETO

## Dependencias
- Ninguna (independiente)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review
- **Veredicto:** ⏳ pendiente

## Notas
- No es un bug de comportamiento (handler tolera omisión) — es inconsistencia de schema/doc

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** pendiente Step 1
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-10
