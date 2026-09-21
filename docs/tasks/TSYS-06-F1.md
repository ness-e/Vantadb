# TSYS-06-F1: Implementar 3 behavior changes del task-system + extraer parsers.mjs

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W2)
- **Fuente:** deuda del veredicto TSYS-06 (`docs/Investigaciones/TSYS-06-chaos-runner.md` §5-8) + diseño `docs/architecture/task-system-chaos-resilience.md` §6
- **Estado:** ✅ COMPLETO · **Sub-agente:** vanta-arch
- **Prioridad:** 🟡 · **Riesgo:** ALTO (toca el server MCP vivo)

## Objetivo
Implementar los 3 behavior changes del diseño §6 (precondiciones para el runner) + extraer parsers a módulo compartido:
- **(a) Corrupción visible:** `readBudget` (campaign-server.mjs:247-251) traga corrupción → falla loud (error claro, no silencio)
- **(b) Writes con checksum:** `update_task_state` (campaign-server.mjs:710-744) write no-atómico → write atómico + checksum
- **(c) WIP atómico:** `findInProgressTasks` (campaign-server.mjs:128-163) sin sync → lectura/escritura atómica del estado WIP
- **Extra:** extraer parsers (extractState:47-55, parseTasks:57-77, findTaskById:634-639) de campaign-server.mjs a `parsers.mjs` compartido (habilita tests de inyección de fallos)

## Archivos clave
- `.opencode/task-system/mcp/campaign-server.mjs` (1514L), `.opencode/task-system/config/state-tools.mjs` (94L — NO tocar la tabla STATE_TOOLS), `docs/architecture/task-system-chaos-resilience.md` (§6 behavior changes), `docs/Investigaciones/TSYS-06-chaos-runner.md` (plan de implementación §7)

## Steps
1. ✅ DISCOVERY: leer diseño §6 + el plan de implementación del doc de decisión (detalla cada behavior change) + campaign-server.mjs (parsers, readBudget, update_task_state, findInProgressTasks, persistencia)
2. ✅ Extraer parsers.mjs (sin cambio de comportamiento) + tests unitarios `node --test` (Node v24 built-in — cero dependencias nuevas)
3. ✅ Implementar (a) corrupción visible, (b) write atómico + checksum, (c) WIP atómico — con tests RED→GREEN
4. ✅ Verificar: `node --test` pasa (29/29) + smoke del server MCP (bun initialize + `campaign_health_status` → healthy:true) + `campaign_verify_cmd` pasa (exit 0)
5. ✅ Reporte en `docs/Investigaciones/TSYS-06-chaos-runner.md` (§10 Implementación) + task file + RESULTADO

## Contrato (verify mecánico) — ✅ PASA
- `node --test ".opencode/task-system/mcp/*.test.mjs"` → 29/29 pass (exit 0) — verificado via `campaign_verify_cmd`
- `campaign-server.mjs` arranca y las 30+ tools MCP responden (smoke MCP: initialize + health → serverLiveness:true)
- API de tools MCP sin cambios (mismos nombres/schemas; payloads iguales + claves aditivas budgetCorrupted/conflict/currentState)
- parsers.mjs extraído; campaign-server.mjs importa desde ahí (`rg` → 0 functions duplicadas)
- checksum/atómico aplicado a los writes de estado (temp+rename + sha1 check-and-set)

## Invariantes (handoff) — ✅ RESPETADAS
- NO tocado: docs/Backlog.md, AUD-024.md, verify-log.jsonl, completions/_vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/* (worktree ajeno intocado)
- NO git add/commit; NO campaign_update_task_state (no se usó)
- NO cambiada la tabla STATE_TOOLS de state-tools.mjs (gate C0 intacto)
- Sin dependencias nuevas (node --test built-in; SDK MCP ya existía)
- Runner NO implementado (sigue deferido — Fase 4, re-evaluar ≤ 2026-09)

## Fases
- SECURITY: aplica parcial — corrupción visible = fail-loud en input corrupto (no trust boundary nuevo); deny-by-default de validateAction intacto
- PERFORMANCE: n/a (server de orquestación, no hot path)

## Resultado
```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5
PROXIMO_STEP: ninguno (Fase 4 runner sigue deferida — re-evaluar ≤ 2026-09)
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: .opencode/task-system/mcp/parsers.mjs (nuevo), campaign-server.mjs (modificado), parsers.test.mjs (nuevo), state-persistence.test.mjs (nuevo), docs/Investigaciones/TSYS-06-chaos-runner.md (§10)
VERIFY_CONTRATO: pasa (29/29 node --test + smoke MCP healthy + campaign_verify_cmd exit 0)
BLOQUEO: ninguno
```