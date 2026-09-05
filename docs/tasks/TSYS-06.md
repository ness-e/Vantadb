# TSYS-06: Chaos/resilience del task-system — decisión runner vs tests puntuales

## Metadata
- **Plan file:** `docs/plans/2026-08-16-wave-p20-tsys.md` (wave P20)
- **Fuente:** docs/Backlog.md:437 (P17) · misma brecha en P18 → `TIR-07` (Backlog.md:453)
- **Esfuerzo:** 🔴 (investigación/decisión, NO runner)
- **Prioridad:** 🟢
- **Tipo:** Investigación/Decisión (Planning/Docs — no toca código del task-system)
- **Turns estimados:** 5-8
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ✅ COMPLETED (veredicto emitido; fila Backlog migrada por el lead al cerrar la wave)
- **Incógnitas (uphill):** 0 — resuelta: tests puntuales (b)+(c) cubren 8/12 escenarios al menor costo; runner DEFERIDO con fecha (tras behavior changes §6 del diseño T19)
- **Pendientes (downhill):** 0 — 3/3 steps ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/Backlog.md` (P17 TSYS-06:437, P18 TIR-07:453), `docs/architecture/task-system-chaos-resilience.md` (diseño T19), `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md:361` (§3.3-24), plan wave P20 |
| Callees | `.opencode/task-system/config/state-tools.mjs` (gate C0, 94L, sin tests), `.opencode/task-system/mcp/campaign-server.mjs` (parsers + persistencia, 1514L), `.opencode/task-system/prompts/iter-loop-tools.md` (prose C0), `campaign_verify_cmd`/SARL/budget/stall (verificaciones existentes) |
| Implicaciones | Decisión habilita: (c) tests unitarios de state-tools (cero cambios de código), (b) tests de inyección de fallos (requiere extraer parsers a módulo compartido — refactor sin cambio de comportamiento), y 3 behavior changes como tareas separadas (diseño §6). NO implementa runner. No cambia API pública, no toca código del server |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `task-system-chaos-resilience.md` (177L — 12 escenarios C1-C12, §6 exige 3 behavior changes NO implementados), `state-tools.mjs` (94L — STATE_TOOLS 10 estados, validateAction deny-by-default:73-84, wildcard:86-92, sin tests), `iter-loop-tools.md` (384L — prose C0), `campaign-server.mjs` §parsers/persistencia (readBudget:247-251 traga corrupción; update_task_state:710-744 write no-atómico; extractState:47-55 emoji-only; parseTasks:57-77 skips bad headers; findTaskById:634-639; getOrCreateCampaignId:115-122; findInProgressTasks:128-163 sin sync; verify-log:832-837 best-effort; execSync kill:786-797), `REPORTE-FINAL.md:336-365` (§3.3 FALTA, item 24 = chaos propio), `docs/Backlog.md:437,453` (TSYS-06 + TIR-07), task template + TIR-03 (formato de task de decisión).
- **Archivos referenciados hacia dentro:** `state-tools.mjs` → importado por `campaign-server.mjs:11`; `chaos-resilience.md` → citado en Backlog P17/P18 y en el plan wave P20.
- **Archivos que referencian a los editados:** el entregable `docs/Investigaciones/TSYS-06-chaos-runner.md` es nuevo (sin referencias entrantes); el task file TSYS-06.md es nuevo. Nada más.
- **Veredicto impacto:** BAJO — solo docs de investigación + task file. No se toca código (regla explícita de la tarea).

## Contrato
"`docs/Investigaciones/TSYS-06-chaos-runner.md` existe con veredicto explícito (implementar/WONTFIT/deferir + fecha) y evidencia citada (archivo:línea de state-tools.mjs / chaos-resilience.md)." Verificación mecánica: `Test-Path docs/Investigaciones/TSYS-06-chaos-runner.md` + grep de veredicto y citas `archivo:línea` en el doc.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. NO se implementa runner ni se toca código (campaign-server.mjs, state-tools.mjs, prompts) — la decisión puede *recomendar* un refactor de extracción de parsers, pero NO se ejecuta en esta tarea
  2. NO se toca `docs/Backlog.md`, `verify-log.jsonl`, `completions/_vanta-cli.ps1`, plan wave P20 — la migración de la fila es del lead al cerrar la wave
  3. La decisión debe cubrir las DOS entradas del mismo gap: TSYS-06 (P17) y TIR-07 (P18)
  4. El diseño T19 (`chaos-resilience.md` §6) declara 3 behavior changes como precondiciones — el veredicto debe respetar ese orden (tests RED primero, runner después)
- **Comandos de verificación:** `Test-Path docs/Investigaciones/TSYS-06-chaos-runner.md` · `rg "Veredicto|deferir|state-tools\.mjs:" docs/Investigaciones/TSYS-06-chaos-runner.md` (citas archivo:línea presentes)
- **Deuda pendiente:** 3 behavior changes (corrupción visible, writes con checksum, WIP atómico) + extracción de parsers + runner deferido — ver sección 8 del doc de decisión

## Steps (Plan → Act → Verify)

1. **✅ DISCOVERY** — mapear estado actual y superficie de riesgo real: leer `chaos-resilience.md` (T19), `state-tools.mjs`, `iter-loop-tools.md`, parsers/persistencia de `campaign-server.mjs`, REPORTE-FINAL §3.3-24 y Backlog P17/P18. Confirmar: state-tools sin tests (Test-Path falso), sin lock en server (grep lock → 0 hits en runtime), readBudget traga corrupción (247-251), write no-atómico (710-744), parsers module-private no exportados. Verify: evidencia archivo:línea recolectada para cada punto de fallo.

   **Hallazgos (2026-08-16):** superficie crítica = state-tools.mjs (gate puro, enumerable, cero tests); riesgos corruptivos (C3/C4/C5/C7/C12) = gaps de diseño de concurrencia/crash que el diseño §6 declara como 3 behavior changes NO implementados; 4 escenarios kill/race (C6/C7/C8/C12) requieren infra de kill que no existe. Ver doc de decisión §2-3.
2. **✅ ANÁLISIS DE OPCIONES** — (a) runner fuzzing 4-8h 🔴 vs (b) tests inyección de fallos 2-4h 🟡 vs (c) unit tests state-tools 30-60min 🟢. Evaluar cada escenario C1-C12 contra cada opción. Verify: tabla de cobertura por opción en el doc de decisión §4.

   **Resultado (2026-08-16):** (b)+(c) cubren 8/12 determinísticamente (C1, C2, C3, C4, C5, C9, C10, C11 — C3/C4/C5 como RED tests contrato); runner solo agregaría valor en 4 kill/race (C6/C7/C8/C12), 3 de los cuales requieren behavior changes. Ver doc §4-5.
3. **✅ DECISIÓN + CIERRE** — escribir `docs/Investigaciones/TSYS-06-chaos-runner.md` con: estado actual, superficie de riesgo (evidencia archivo:línea), opciones costo/beneficio, **veredicto explícito: IMPLEMENTAR (b)+(c) tests puntuales; runner DEFERIR con fecha** + razón (ponytail ladder + evidencia), y plan de implementación detallado (NO ejecutado). Verify: `Test-Path` del doc ✅ + `rg` veredicto/citas ✅ (contrato mecánico).

   **Veredicto (2026-08-16):** IMPLEMENTAR (b)+(c) — tests puntuales (unit de state-tools + inyección de fallos con fixtures corruptos + RED tests de los 3 behavior changes). Runner: DEFERIR — re-evaluar al completar los 3 behavior changes del diseño §6 (próxima wave de hardening del task-system). Razón: gate crítico es función pura enumerable (unit test = tool correcto, fuzzer = overkill); riesgos corruptivos son gaps de diseño, no de input; runner antes de behavior changes = assertar contra comportamiento inexistente (5/12 escenarios rojos por precondición). Ver `docs/Investigaciones/TSYS-06-chaos-runner.md` §5.

## Dependencias
- Ninguna (investigación autónoma); resuelve además la entrada TIR-07 de P18
- Siguientes (deuda del veredicto): 3 behavior changes del diseño §6 + extracción `parsers.mjs` + runner deferido

## Fases explícitas — SECURITY | PERFORMANCE

- [ ] **SECURITY** — NO aplica: investigación/decisión sobre el task-system, no toca trust boundaries ni input de usuario.
- [ ] **PERFORMANCE** — NO aplica: no toca hot path ni código.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** pendiente (vanta-review) — el lead lo asigna al cerrar la wave P20.
- **Enfoque:** validar que el veredicto (tests puntuales > runner, runner deferido con fecha) está justificado por la evidencia archivo:línea y respeta el orden behavior-changes-primero del diseño §6.
- **Veredicto:** pendiente.

## Notas
- La pregunta central de la tarea ("¿vale construir el runner?") se responde con la escalera ponytail: la superficie crítica (gate de tools por estado) es una función pura con espacio de entrada enumerable — un fuzzer de strings aleatorios no descubre más que una tabla exhaustiva. Los riesgos de corrupción persistida no son "bugs de input fuzzeables" sino gaps de diseño de concurrencia/crash, que el propio diseño T19 §6 declara como 3 behavior changes pendientes. Construir el runner antes de esos cambios sería assertar contra comportamiento que no existe.
- Node v24.16.0 verificado: `node --test` built-in disponible — cero dependencias nuevas para (b)+(c).