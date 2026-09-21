# TSYS-06: Chaos/resilience del task-system — decisión (runner vs tests puntuales)

> **Fecha:** 2026-08-16
> **Autor:** vanta-lead (wave P20)
> **Estado:** ✅ DECISIÓN EMITIDA — veredicto: **IMPLEMENTAR tests puntuales (b)+(c); runner DEFERIR con fecha**
> **Fuente:** `docs/Backlog.md:437` (P17 TSYS-06) y `docs/Backlog.md:453` (P18 TIR-07 — misma brecha) · `docs/architecture/task-system-chaos-resilience.md` (diseño T19) · `REPORTE-FINAL.md:361` (§3.3-24)
> **Alcance:** Investigación/decisión. NO implementa runner, NO toca código del task-system.

---

## 1. Pregunta de decisión

`vanta-chaos` fuzzza el código fuente de VantaDB, no a `campaign-server.mjs` ni a la máquina de estados del task-system (gap-01 §3.3-24, REPORTE-FINAL.md:361). Existe el diseño T19 (`task-system-chaos-resilience.md`, 12 escenarios C1-C12) con runner marcado "DEFER (sin verificación)".

**¿Vale construir el runner que fuzzee campaign-server/state machine, o alcanza con tests de inyección de fallos puntuales?**

---

## 2. Estado actual (verificado 2026-08-16)

| Componente | Estado verificado | Evidencia |
|---|---|---|
| `state-tools.mjs` (gate C0) | Módulo puro de 94L, **sin tests** (no existe `state-tools.test.mjs` ni ningún test en `.opencode/task-system/` fuera de `node_modules/`). Enforce deny-by-default; 10 estados; wildcard por prefijo | `state-tools.mjs:9-65` (STATE_TOOLS), `:73-84` (validateAction deny-by-default), `:86-92` (matchPattern) |
| Parsers de plan file | `parseTasks`/`extractState`/`findTaskById`/`updateState` son **module-private** (no exportados) → no importables directamente para test unitario | `campaign-server.mjs:47-77, 634-649` |
| Escritura de estado | `campaign_update_task_state` hace read→regex-replace→write **sin lock, sin temp-file swap, sin checksum/versión** | `campaign-server.mjs:710-744` |
| Budget | `readBudget` traga TODO error de parse devolviendo `{ tasks: {} }` → reset silencioso de contadores/traceId | `campaign-server.mjs:247-251` |
| Lock de concurrencia | **No existe** ningún mecanismo de lock en el runtime (grep `lock|flock` → 0 hits en código de server) | `campaign-server.mjs` (full grep) |
| Verificaciones de fallo existentes | `campaign_verify_cmd` (mecánico + budget), SARL, budget (5 límites), stall detection, `campaign_validate_command`/`validate_output`, C0 enforcement | `campaign-server.mjs:237-243` (BUDGET_LIMITS), `:1281-1336` (enforce_state) |
| Diseño T19 | 12 escenarios C1-C12; **§6 exige 3 behavior changes NO implementados**: (1) corrupción visible, (2) writes con checksum, (3) WIP atómico | `task-system-chaos-resilience.md:142-156` |
| Runtime | Node v24.16.0 → `node --test` built-in disponible (cero deps nuevas) | verificado en entorno |

---

## 3. Superficie de riesgo real (mapeada con evidencia archivo:línea)

| # | Riesgo | Evidencia | Escenario T19 | ¿Cubierto hoy? |
|---|--------|-----------|---------------|----------------|
| R1 | Gate de tools por estado — función pura, input enumerable (10 estados × ~40 patrones) | `state-tools.mjs:9-65, 73-92` | C10 | ✅ Sí (deny-by-default) |
| R2 | Escritura no-atómica del plan file (read→write sin lock/checksum) | `campaign-server.mjs:710-744` | C4, C6 | ❌ No (last-writer-wins silencioso) |
| R3 | Corrupción de budget JSON tragada silenciosamente (reset de contadores + traceId + lastActivity → falso stall) | `campaign-server.mjs:247-251` | C3, C7 | ❌ No (silencioso) |
| R4 | Estado derivado por regex de Markdown hand-edited (emoji-only; headers rotos se saltan sin warning → tareas desaparecen del DAG) | `campaign-server.mjs:47-55` (extractState), `:57-77` (parseTasks), `:634-639` (findTaskById) | C1, C2 | 🟡 Parcial (degrada sin crash pero sin warning) |
| R5 | Campaign ID race: `getOrCreateCampaignId` escribe el plan durante `get_next_task` | `campaign-server.mjs:115-122, 367-372` | C5 | 🟡 Parcial (idempotente por regex, race real) |
| R6 | WIP hard-limit sin sincronización: dos claims `in-progress` concurrentes pueden pasar ambos | `campaign-server.mjs:128-163` (findInProgressTasks) | C12 | ❌ No (race) |
| R7 | verify-log append best-effort (kill pierde entrada; retry duplica línea con mismo traceId) | `campaign-server.mjs:786-797` (execSync timeout/kill), `:832-837` (appendFileSync en try/catch) | C8, C9 | 🟡 Parcial (diseño acepta best-effort) |

**Lectura de la tabla:** 4 riesgos (R2, R3, R5, R6) son gaps de **diseño de concurrencia/crash**, no "bugs de input fuzzeables". El diseño T19 §6 ya los declara como 3 behavior changes pendientes. Solo R1/R4/R7 son testables por corrupción de input/invocación — y R1 es una función pura enumerable.

---

## 4. Opciones — costo/beneficio

### (a) Runner de fuzzing completo — 🔴 4-8h
- **Qué:** sandbox (via `campaign_run_sandboxed`, ya existe), staged fixtures de plan/budget/verify-log, kill de proceso (`Stop-Process`), doble cliente concurrente, asserts de los 12 escenarios.
- **Cubre:** los 4 escenarios kill/race (C6, C7, C8, C12) que los tests deterministas no alcanzan.
- **Problema:** 5 de 12 escenarios (C3, C4, C5, C7, C12) assertan sobre **behavior changes que NO existen** (diseño §6). Correr hoy = rojo por precondición ausente, no por defecto descubierto. **Orden invertido.** Requiere infra nueva de orquestación de kill en Windows (no existe).
- **Beneficio marginal hoy:** bajo — el fuzzing de input sobre parsers que ya degradan sin crash (R4) no descubre bugs nuevos; los kill no son accionables hasta que exista atomicidad (R2/R3).

### (b) Tests de inyección de fallos puntuales — 🟡 2-4h
- **Qué:** `node --test` (built-in, Node v24) con: fixtures corruptos (plan truncado, header roto, `- **Estado:**` inyectado, budget JSON inválido), llamadas inválidas a `validateAction`/`enforce_state`, doble invocación concurrente a `update_task_state` vía handler.
- **Cubre 8/12 determinísticamente:** C1, C2, C3, C4, C5, C9, C10, C11 (C3/C4/C5 como **RED tests** que definen el contrato exacto de los behavior changes — TDD aplicado al propio harness).
- **Requisito:** extraer parsers a módulo compartido (`parsers.mjs`) — refactor sin cambio de comportamiento, o testear vía protocolo MCP (más pesado).

### (c) Unit tests de `state-tools.mjs` — 🟢 30-60min
- **Qué:** tabla exhaustiva 10 estados × (tool permitida ✓ / denegada ✗ / desconocida ✗ / wildcard por prefijo) + estados desconocidos (deny-all + reason) + `getAllowedTools` para estado inválido.
- **Cubre:** C10 determinísticamente + guarda contra la divergencia canon/prose (P0-2: `iter-loop-tools.md` vs `state-tools.mjs`).
- **Costo mínimo:** cero deps, cero cambios de código (el módulo ya es importable, es puro).

---

## 5. Veredicto

> **IMPLEMENTAR (b) + (c) — tests puntuales.**
> **Runner: DEFERIR — re-evaluar al completar los 3 behavior changes del diseño §6 (próxima wave de hardening del task-system; fecha de re-evaluación: al cerrar el último behavior change, estimado ≤ 2026-09).**
> **WONTFIT descartado** (el gap es real: REPORTE-FINAL.md:361 + Backlog P17/P18 lo confirman; un harness sin verificación de su propio estado es el único componente sin tests).

**Razón (escalera ponytail + evidencia):**

1. **La superficie crítica es una función pura enumerable.** El gate de tools por estado (`state-tools.mjs:73-92`) tiene espacio de entrada finito: 10 estados × ~40 patrones + estados desconocidos. Un fuzzer de strings aleatorios no descubre más que una tabla exhaustiva — el riesgo real es "alguien editó STATE_TOOLS y rompió deny-by-default", que un test unitario atrapa en 1ms. (c) es el tool correcto; el fuzzer sobre el server entero es **overkill** (exactamente la hipótesis de la tarea).
2. **Los riesgos corruptivos no son fuzzeables por input — son gaps de diseño.** R2/R3/R5/R6 (C3/C4/C5/C7/C12) son problemas de atomicidad/concurrencia que el diseño T19 §6 ya declara como 3 behavior changes. Construir el runner antes = assertar contra comportamiento inexistente (5/12 escenarios rojos por precondición, no por defecto). Los **RED tests** de (b) definen el contrato exacto que esos behavior changes deben cumplir — el runner solo puede validarlos DESPUÉS.
3. **Costo vs cobertura:** (b)+(c) ≈ 2-4h cubren 8/12 escenarios determinísticamente y dejan los 4 kill/race (C6/C7/C8/C12) como especificación del runner futuro. El runner solo (4-8h) hoy no cubre nada que los tests no cubran salvo kill real — y los kill no son accionables hasta que exista atomicidad (R2/R3).
4. **Ladder:** (c) usa `node --test` built-in (stdlib) — cero deps. (b) reusa los parsers existentes vía extracción mínima (ya-existe → reusar, no re-implementar). El runner necesitaría infra nueva (staging de fixtures, orquestación de kill en Windows) que no justifica su costo hoy.
5. **C1/C2/C10/C11 ya pasan hoy** (el diseño §6 lo confirma: "Current code already satisfies several scenarios") — los tests los congelan como regresión; C3/C4/C5/C7/C12 los convierten en deuda accionable con contrato verificable.

**Relación con TIR-07:** esta decisión resuelve las dos entradas del mismo gap (P17 TSYS-06 y P18 TIR-07).

---

## 6. Plan de implementación (NO ejecutado en esta tarea)

### Fase 1 (30-60min) — Unit tests de state-tools.mjs
- **Archivos:** `.opencode/task-system/config/state-tools.test.mjs` (nuevo) · `state-tools.mjs` (sin cambios — ya importable)
- **Pasos:**
  1. Tabla de casos: por cada estado en `STATE_TOOLS` (10): tool permitida → `allowed:true`; tool denegada → `allowed:false` + reason; tool desconocida → `allowed:false`; wildcard (`campaign_*` matchea `campaign_verify_cmd`, `argus_*`, `metasearchmcp_*`).
  2. Estados desconocidos (`"FOO"`, `"planning"`, `""`, `null`) → `allowed:false` con reason "no existe en STATE_TOOLS" (deny-by-default, `state-tools.mjs:75`).
  3. Precedencia: tool que matchea denied Y allowed → blocked (denied se evalúa primero, `state-tools.mjs:77-78`).
  4. Paridad canon/prose (P0-2): cada estado de `iter-loop-tools.md:130-139` existe en `STATE_TOOLS` (STALL incluido — la divergencia histórica que P0-2 quería matar).
- **Verify:** `node --test .opencode/task-system/config/state-tools.test.mjs` → pass.

### Fase 2 (1-2h) — Extracción de parsers + tests de corrupción
- **Archivos:** `.opencode/task-system/mcp/parsers.mjs` (nuevo — mover `extractField`, `extractState`, `parseTasks`, `parseRecitation`, `findTaskById`, `updateState`, `updateRecitation`, `extractCampaignId`, `getOrCreateCampaignId` desde `campaign-server.mjs`) · `campaign-server.mjs` (import desde parsers.mjs — **refactor sin cambio de comportamiento**) · `parsers.test.mjs` (nuevo)
- **Pasos:**
  1. Mover funciones puras de parsing a `parsers.mjs` y re-exportar; `campaign-server.mjs` solo cambia la línea de import.
  2. Fixtures de corrupción: plan truncado a mitad de header (`### Task 5` sin título), `- **Estado:**` roto (sin emoji → PENDING, `campaign-server.mjs:47-55`), `### Task` suelto en una nota (no rompe `parseTasks` porque el regex exige `\d+:`), budget `{ "tasks": {` truncado (simula kill mid-write), budget con `tasks: null`.
  3. Asserts C1/C2/C11: `parseTasks` no crashea, tarea con header roto se salta (documentando el comportamiento actual como contrato — el warning es el behavior change pendiente), `findTaskById` no encuentra bloque fantasma.
  4. Assert C9: doble `campaign_verify_cmd` del mismo taskId → dos líneas JSONL válidas con el mismo `traceId` (con fixture de plan + budget válidos).
- **Verify:** `node --test .opencode/task-system/mcp/parsers.test.mjs` → pass; `node --check campaign-server.mjs` → sin errores de sintaxis; `rg "parseTasks" campaign-server.mjs` → import desde parsers.mjs.

### Fase 3 (1h) — RED tests de los 3 behavior changes (contrato de las tareas de implementación)
- **Archivos:** `.opencode/task-system/mcp/state-persistence.test.mjs` (nuevo) — tests que HOY fallan y pasan cuando se implementen los behavior changes §6 del diseño T19.
- **Pasos:**
  1. C3/C7: `readBudget` con JSON inválido → respuesta con `budgetCorrupted:true` (hoy devuelve `{tasks:{}}` silencioso — `campaign-server.mjs:247-251`).
  2. C4/C5: doble `update_task_state` concurrente → perdedor recibe `conflict:true`/`updated:false` con estado ganador (hoy last-writer-wins — `campaign-server.mjs:710-744`).
  3. C12: doble claim `in-progress` → solo uno gana, el otro `wipBlocked:true` (hoy ambos pueden pasar — `findInProgressTasks:128-163` sin sync).
  4. Marcar los 3 como `skip` con motivo hasta que la tarea de implementación del behavior change los active (contrato vivo, no suite roja permanente).
- **Verify:** `node --test --test-skip-pattern` (o skip explícito) → suite verde con 3 skips documentados.

### Fase 4 (DEFERIDA — no implementar ahora) — Runner completo
- **Cuándo:** al completar los 3 behavior changes de la Fase 3 (estimado ≤ 2026-09). Re-evaluar costo/beneficio con el doc §4-5 como spec: cubriría los 4 escenarios kill/race (C6/C7/C8/C12) que los tests deterministas no alcanzan — kill real de `campaign-server.mjs` en sandbox entre read/write, truncado de budget en caliente, doble cliente concurrente.
- **Nota:** C6/C7 solo son accionables cuando exista atomicidad (temp-file swap / checksum) — sin eso, el kill "pasa" sin afirmar nada.

---

## 7. Contrato de verificación mecánica

- `Test-Path docs/research/TSYS-06-chaos-runner.md` → **True** (este doc).
- `rg "Veredicto|IMPLEMENTAR|DEFERIR|state-tools\.mjs:|chaos-resilience\.md:" docs/research/TSYS-06-chaos-runner.md` → veredicto explícito + citas archivo:línea presentes.
- Evidencia citada mínima requerida por la tarea: `state-tools.mjs:73-92` (deny-by-default/wildcard) y `task-system-chaos-resilience.md:142-156` (§6 behavior changes) — ambas presentes en §2/§3/§5.

---

## 8. Deuda / follow-ups (resultado de la decisión)

| Item | Tipo | Dueño sugerido | Estado |
|------|------|----------------|--------|
| Fase 1 — unit tests `state-tools.test.mjs` | Implementación 🟢 30-60min | vanta-worker | Pendiente |
| Fase 2 — extracción `parsers.mjs` + tests corrupción | Implementación 🟡 1-2h | vanta-worker | Pendiente |
| Behavior change 1 — corrupción visible (`readBudget` + `budgetCorrupted`) | Implementación 🟢 (diseño §6.1) | vanta-worker | Pendiente |
| Behavior change 2 — writes con checksum/conflict + campaign ID atómico | Implementación 🟡 (diseño §6.2) | vanta-worker | Pendiente |
| Behavior change 3 — WIP claim atómico | Implementación 🟡 (diseño §6.3) | vanta-worker | Pendiente |
| Fase 3 — RED tests de los 3 behavior changes | Implementación 🟢 1h | vanta-worker | Pendiente |
| Fase 4 — runner completo | **DEFERIDO** (re-evaluar al cerrar el último behavior change, ≤ 2026-09) | vanta-chaos | ⏸ |
| Migrar filas Backlog P17 TSYS-06 + P18 TIR-07 | Progreso | vanta-lead (wave close) | Pendiente |

---

## 9. Notas

- **Ponytail note:** la tentación de "build a chaos runner" es el fuzzer por el fuzzer; la escalera manda: ya-existe (parsers) → stdlib (`node --test`) → mínimo (unit tests del gate puro). El runner no es YAGNI definitivo (los kill/race son reales) sino **prematuro**: su valor depende de behavior changes que primero hay que implementar.
- **Límite del veredicto:** los tests puntuales NO cubren kill real entre read/write (C6/C7) ni race de WIP (C12) — esos 4 escenarios quedan explícitamente como spec del runner diferido (Fase 4). No se afirma que "tests puntuales == runner"; se afirma que hoy cubren el 8/12 accionable al menor costo y congelan el comportamiento actual.

---

## 10. Implementación (TSYS-06-F1) — ✅ COMPLETO 2026-08-17

> **Ejecutor:** vanta-arch · **Task:** `.opencode/skills/campaign-executor/tasks/TSYS-06-F1.md` · **Plan:** `docs/plans/2026-08-16-wave-followups.md` (W2)

### 10.1 Qué se implementó

Los 3 behavior changes del diseño §6 + extracción de parsers (precondiciones del runner Fase 4):

| Change | Escenarios | Implementación |
|--------|-----------|----------------|
| **(a) Corrupción visible** | C3, C7 | `readBudget` distingue `ENOENT` (sin budget = primer uso, vacío limpio) de error real de parse → `{tasks:{}, budgetCorrupted:true}`. `writeBudget` hace strip del flag (la corrupción es transitoria, nunca se persiste). `budgetStatus`, `consumeBudget`, `campaign_budget_consume` y `campaign_stalled_tasks` surfacen `budgetCorrupted` en la respuesta. Fix colateral: `consumeBudget` re-lee el budget tras `initTaskBudget` (evita crash por state stale en budget corrupto) |
| **(b) Writes con checksum + atómicos** | C4, C6 | `updateTaskStateCore`: checksum sha1 del contenido original (`node:crypto` `createHash`) + re-read antes del write → si cambió, el perdedor recibe `{updated:false, conflict:true, currentState}` con el estado ganador (C4). Write vía temp+rename (C6): kill entre write y rename deja el archivo original intacto. `getOrCreateCampaignId` sigue idempotente (C5) |
| **(c) WIP atómico** | C12 | helper `withPlanLock(planPath, fn)` — lock file exclusivo (`O_EXCL`) con retry 50ms, stale-detection por mtime (10s, crash previo) y timeout 5s con error claro. El scan WIP corre DENTRO del lock (check-and-set): ningún claim pasa entre scan y write. `findInProgressTasks` ahora deriva `opencodeRoot` del worktree (`resolve(worktree, ".opencode")`) — mismo resultado en producción, testeable con worktrees temporales |
| **Extra: parsers.mjs** | C1, C2, C11 | 9 funciones puras + `STATE_MAP` movidas verbatim a `.opencode/task-system/mcp/parsers.mjs`; `campaign-server.mjs` importa desde ahí. Refactor sin cambio de comportamiento (bodies idénticos) |

### 10.2 Archivos

- `.opencode/task-system/mcp/parsers.mjs` (nuevo — parsers puros)
- `.opencode/task-system/mcp/campaign-server.mjs` (modificado — behavior changes + import parsers + exports para tests + guard `isMain` para no conectar stdio al importar)
- `.opencode/task-system/mcp/parsers.test.mjs` (nuevo — 14 tests: fixtures corrupción C1/C2/C11, contrato parsers)
- `.opencode/task-system/mcp/state-persistence.test.mjs` (nuevo — 15 tests: C3/C4/C6/C7/C12 + regresión ENOENT)

**API MCP sin cambios:** mismos nombres/schemas de las 30+ tools; payloads iguales salvo claves ADITIVAS (`budgetCorrupted`, `conflict`, `currentState`). `state-tools.mjs` NO se tocó (gate C0 intacto).

### 10.3 Contrato verificado

| Check | Resultado |
|-------|-----------|
| `node --test ".opencode/task-system/mcp/*.test.mjs"` | ✅ 29/29 pass (exit 0) |
| `node --check campaign-server.mjs` / `parsers.mjs` | ✅ sin errores de sintaxis |
| Smoke MCP (bun): initialize + `campaign_health_status` | ✅ `healthy:true`, `serverLiveness:true` |
| `rg "function (extractField\|parseTasks\|...)" campaign-server.mjs` | ✅ 0 hits — parsers solo en parsers.mjs |
| Import del módulo bajo node (guard isMain) | ✅ 9 exports, sin conexión stdio |

### 10.4 Estado de fases (§6 del doc de decisión)

| Fase | Estado |
|------|--------|
| Fase 1 — unit tests `state-tools.mjs` | ⬜ Pendiente (tarea vanta-worker separada; `state-tools.mjs` NO se tocó por invariante) |
| Fase 2 — extracción `parsers.mjs` + tests corrupción | ✅ HECHA (esta tarea) |
| Behavior changes 1-3 (§6) | ✅ HECHA (esta tarea) |
| Fase 3 — RED tests | ✅ HECHA como GREEN (esta tarea implementó los behavior changes, no solo los tests) |
| Fase 4 — runner completo | ⏸ **Sigue DEFERIDA** — re-evaluar ≤ 2026-09; ahora los C3/C4/C5/C7/C12 ya tienen contrato verificable, así que el runner asserta contra comportamiento existente (no precondiciones ausentes) |

### 10.5 Notas / deuda

- **Ceiling documentado (`ponytail:`):** `withPlanLock` serializa writers del MISMO plan file; la carrera cross-plan del claim WIP (dos sesiones sobre planes distintos) sigue como spec del runner Fase 4. El caso real (2 sesiones sobre el mismo plan) queda cubierto.
- **ENOENT vs corrupción:** `readBudget` distingue archivo inexistente (primer uso, no corrupción) de JSON inválido/truncado (corrupción visible). `tasks:null` parsea sin error → NO es corrupción detectable por `readBudget` (requeriría validación de shape, fuera de alcance de los 3 behavior changes).
- **C6/C7 kill real** (proceso muerto entre read/write, truncado en caliente) y **race WIP real** (dos procesos paralelos) siguen siendo dominio del runner Fase 4 — los tests deterministas cubren el mecanismo (detectConflict, lock lifecycle, check-and-set secuencial).