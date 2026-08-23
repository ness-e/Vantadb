# VantaDB — Manual de Operación del Sistema (ÍNDICE)

> ⚠️ **DEPRECATED COMO FUENTE DE DETALLE (2026-08-23).** Este documento duplicaba
> ~60% del contenido vivo y divergía de las fuentes canónicas. Ahora es un
> **índice**: cada sección apunta a su fuente única de verdad. No agregues
> detalle acá — editá la fuente canónica.

## Fuentes canónicas por tema

| Tema | Fuente única de verdad |
|------|------------------------|
| Reglas del proyecto, AI Guardian (Reglas 1-11), ritual de sesión, MCP servers, path resolution | `.opencode/AGENTS.md` |
| Ciclo de vida de tarea, plan/task file format, recitation, budget, SARL (resumen) | `.opencode/skills/campaign-executor/SKILL.md` |
| North star + reglas invariantes del executor | `.opencode/skills/campaign-executor/RULES.md` |
| Ejecución de UNA tarea completa (canónico) | `.opencode/task-system/prompts/pipeline-full.md` |
| Orquestación multi-tarea / waves paralelas | `.opencode/task-system/prompts/pipeline-run.md` |
| Loop de 1 iteración + state machine C0 (prosa) | `.opencode/task-system/prompts/iter-loop-tools.md` |
| Creación de plan desde backlog (triage, pre-mortem, Cynefin) | `.opencode/task-system/prompts/plan.md` |
| Question Gates HITL (P/D/V/C) + spec-driven guiado | `.opencode/task-system/prompts/question-gates.md` + `prompts/spec-template.md` |
| Recuperación de sub-agentes (RESUME/RETRY/STRATEGY/ESCALATE) | `.opencode/task-system/prompts/subagent-recovery.md` |
| State machine C0 enforcement (código, fuente #1) | `.opencode/task-system/config/state-tools.mjs` |
| Presupuestos (números únicos) | `BUDGET_LIMITS` en `.opencode/task-system/mcp/campaign-server.mjs` |
| Skills: catálogo y carga | `SKILLS-MANIFEST.md` (raíz) + `references/skills-engineering.md` |
| Agents: roles, tabla de límites de tools por rol | `.opencode/AGENTS.md` § Límites de herramientas por rol |
| MCP servers activos/deshabilitados | `.opencode/AGENTS.md` § MCP Servers Disponibles |
| Reglas normativas por área de código (lazy-loading) | `.opencode/rules/README.md` + archivo del área |
| Troubleshooting técnico (Windows, cargo, tests) | `docs/references/troubleshooting.md` |
| DoD por nivel (task/commit/release) | `.opencode/references/definition-of-done.md` |

## Flujos rápidos (resumen de bolsillo)

```
Desarrollo diario:   skill progreso → git status → codegraph → code → verify_changed → commit
Feature completa:    /pipeline plan docs/Backlog.md → Gate P → /pipeline run (waves, SARL)
Bug:                 skill systematic-debugging → repro → fix → verify → commit
Pre-push:            dev-tools/verify.ps1 (NUNCA --no-verify) → push
Una tarea:           /pipeline task <ID> → pipeline-full.md → RESULTADO block
```

## Historial

El contenido detallado anterior (v1, 917 líneas, secciones 1-14 + apéndices)
vive en `git history` (`git log --follow .opencode/VANTADB-OPERATING-MANUAL.md`)
por si necesitás recuperar alguna sección que no tenga fuente canónica hoy.
