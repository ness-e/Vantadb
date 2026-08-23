# Plan de Ejecución: Reorganización avance-canónico + unificación reviews

> **Campaign ID:** 3959867d-76dd-4116-9e11-6dc782bd45e1
> **Inicio:** 2026-08-23
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** Revisión de skills solicitada por usuario + decisiones Gate P

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 6  | 0     | 0    | 0         |

**Decisiones Gate P:** migración física completa progreso→avance · eliminar vantadb-full-review · podar trigger words del design-orchestrator.

---

### Task 1: M1 — Migración física docs/progreso → docs/avance
- **Archivos clave:** docs/progreso/*, docs/avance/** 🟡
- **Contrato:** Test-Path docs/progreso = False; todos los archivos accesibles bajo docs/avance/historial/fuentes/; rg "docs/progreso" sin matches en archivos vivos del sistema
- **Estado:** ✅ COMPLETED

### Task 2: M2 — Skill progreso reescrita (flujo avance-canónico)
- **Archivos clave:** .opencode/skills/progreso/SKILL.md 🟡
- **Contrato:** la skill ya no referencia docs/progreso como destino de escritura; escribe SOLO en docs/avance; nombre de skill se conserva (puntos de llamada intactos)
- **Estado:** ✅ COMPLETED

### Task 3: M3 — Referencias vivas actualizadas
- **Archivos clave:** .opencode/AGENTS.md, task-system/prompts/pipeline-full.md, references/definition-of-done.md, rules/frontend-web.md, scripts/check-avance-coverage.ps1, commands/*.md 🟢
- **Contrato:** rg -l "docs/progreso" sobre sistema vivo (excluyendo tasks históricos/memory) = 0 matches
- **Estado:** ✅ COMPLETED

### Task 4: U1 — Eliminar .agents/skills/vantadb-full-review/
- **Archivos clave:** .agents/skills/vantadb-full-review/ 🟢
- **Contrato:** directorio eliminado; grep confirma cero referencias activas (manifest ya lo marca REMOVED)
- **Estado:** ✅ COMPLETED

### Task 5: D1 — Podar trigger words vanta-design-orchestrator
- **Archivos clave:** .agents/skills/vanta-design-orchestrator/SKILL.md 🟢
- **Contrato:** SKILL.md ≤6KB; catálogo completo intacto en layers/; reglas de orquestación preservadas
- **Estado:** ✅ COMPLETED

### Task 6: V1 — Informe final de unificación y estado campaign-executor/unified-review
- **Archivos clave:** SKILLS-MANIFEST.md 🟢
- **Contrato:** manifest actualizado (full-review entrada física eliminada); informe de hallazgos entregado al usuario
- **Estado:** ✅ COMPLETED

## Protocolo
Igual que campañas anteriores: MCP tools por paso, Question Gates aplicables, commits por fase.
