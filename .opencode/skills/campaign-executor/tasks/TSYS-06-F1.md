# TSYS-06-F1: Implementar 3 behavior changes del task-system + extraer parsers.mjs

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W2)
- **Fuente:** deuda del veredicto TSYS-06 (`docs/Investigaciones/TSYS-06-chaos-runner.md` §5-8) + diseño `docs/architecture/task-system-chaos-resilience.md` §6
- **Estado:** ⏳ IN PROGRESS · **Sub-agente:** vanta-arch
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
1. DISCOVERY: leer diseño §6 + el plan de implementación del doc de decisión (detalla cada behavior change) + campaign-server.mjs (parsers, readBudget, update_task_state, findInProgressTasks, persistencia)
2. Extraer parsers.mjs (sin cambio de comportamiento) + tests unitarios `node --test` (Node v24 built-in — cero dependencias nuevas)
3. Implementar (a) corrupción visible, (b) write atómico + checksum, (c) WIP atómico — con tests RED→GREEN
4. Verificar: `node --test` pasa (tests nuevos) + smoke del server: `campaign_health_status` o arranque del server MCP + `campaign_verify_cmd` sigue funcionando
5. Reporte en `docs/Investigaciones/TSYS-06-chaos-runner.md` (sección implementación) + task file + RESULTADO

## Contrato (verify mecánico)
- `node --test` verde (tests de los 3 behavior changes + parsers)
- `campaign-server.mjs` sigue arrancando y las 30+ tools MCP responden (smoke via health/verify)
- API de tools MCP sin cambios (mismos nombres/schemas)
- parsers.mjs extraído; campaign-server.mjs importa desde ahí
- checksum/atómico aplicado a los writes de estado

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl (es OUTPUT del server — no editarlo), completions/_vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- NO cambiar la tabla STATE_TOOLS de state-tools.mjs (gate C0 por estado — funciona, no tocar)
- Sin dependencias nuevas (node --test built-in)
- Runner NO se implementa (sigue deferido; se re-evalúa tras estos changes)

## Fases
- SECURITY: aplica parcial — corrupción visible = fail-loud en input corrupto (no trust boundary nuevo); mantener deny-by-default de validateAction
- PERFORMANCE: n/a (server de orquestación, no hot path)

## Resultado
```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO
STEPS_OK: <n>/<M>
PROXIMO_STEP: <...>
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | ...>
```