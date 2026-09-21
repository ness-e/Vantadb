# TIR-02: DORA recovery time + rework rate — viabilidad con telemetría actual

> **Tipo:** Investigación/Decisión (read-only). **Estado:** cerrado 2026-08-17.
> **Fuentes:** `docs/reports/dora.md:183-197`, `eng-03-project.md §8.3`, evidencia directa de telemetría (verificado, no asumido).

## 1. Estado real de la telemetría (verificado)

### `verify-log.jsonl` — EXISTE y se puebla activamente
- **Ruta:** `.opencode/task-system/enforcement/verify-log.jsonl` (gitignored — por eso `glob` no lo encuentra; `Test-Path` ✅).
- **Tamaño/estado:** 6.956 bytes, **23 líneas**, última escritura 2026-08-17 (hoy). Append-only, escrito por `campaign_verify_cmd` (`campaign-server.mjs:800-829`).
- **Campos por entrada:** `ts` (ISO), `taskId` (o null), `traceId` (solo si taskId — P2-05 parcial), `command`, `passed` (bool), `exitCode`, `expectedExitCode`, `elapsed` (s), `summary` (counts nextest), `plan`, `skills`, `toolUsed`.
- **Contenido:** 23 entradas, 10 taskIds distintos, **5 fallos reales** (T1-residuo-consolidado, ERR-036, ERR-043, CI-05 ×2, AUD-033 ×2 — algunos `exitCode:-1` = no ejecutado/kill, no fallo de deploy).

### `traces/<campaignId>.jsonl` — existe pero NO sirve para rework hoy
- `traces/tracer.mjs` emite eventos `task.started/completed/in-progress`, `plan.adjust`, `campaign.*`, `memory.*`.
- `plan.adjust` registra `fromState`/`newState`/`decision_reason`/`pattern`, pero en la práctica `decision_reason` y `pattern` están **casi siempre `null`** (TSYS-09 no se usa consistentemente). No hay evento `task.reopened` dedicado.
- No lo consume `evals/dora.mjs`.

### `evals/dora.mjs` (reporte actual)
- Ya lee `verify-log.jsonl` para **CFR** (intentos/failures). No calcula recovery ni rework.

## 2. Viabilidad

### Recovery time (tiempo en volver a DE tras fallo) — ✅ VIABLE hoy
- **Definición pragmática:** DE = verify verde. Fallo = entrada `passed:false`; recovery = siguiente `passed:true` de la misma tarea/comando. Δ`ts`.
- **Datos actuales:** 3 pares fail→pass identificables en el log:
  - T1-residuo-consolidado: 07:50:42 fail → 20:24:29 pass = **~12.6 h**
  - CI-05: 01:12:11 fail → 05:47:47 pass = **~28.6 h** (fail espurio por harness/cwd, documentado en AUD-029)
  - AUD-033: 06:00:33 fail → 06:00:50 pass = **~17 s** (retry inmediato)
- **Limitaciones (documentables, no bloqueantes):** pairing por `taskId`+`command` (entradas con `taskId:null` no pareables); `exitCode:-1` ≠ fallo real (kill/timeout/harness); fallo sin retry registrado = recovery censurado (abierto); verify-log es muestra de verifies, no todo el estado del pipeline. Suficiente como métrica de estabilidad agregada con estas caveats en el reporte.

### Rework rate (tareas reabiertas/total) — ❌ NO viable hoy
- **Definición pragmática:** tareas `completed` → reabiertas (`in-progress`/`pending`) / total.
- **Qué falta:** `verify-log` no registra transiciones de estado de tareas — solo verifies. `traces/*.jsonl` tiene `plan.adjust` con `fromState`/`newState`, pero sin `pattern` poblado una reapertura es **indistinguible** de cualquier ajuste; requiere cruzar ~200 archivos por campaignId; sin intención de métrica.
- **Alternativas descartadas:** git history de task/plan files (estado in-place, frágil, no estructurado); heurística best-effort (falsos positivos altos — inviable como métrica DORA).

## 3. Recomendación

**IMPLEMENTAR — parcial, con split explícito:**

1. **Recovery time: implementar ya.** ~30 líneas en `evals/dora.mjs` (pairing fail→pass por taskId+command sobre verify-log; reportar count, mediana/máx, recovery abiertos). Esfuerzo 🟢, dato ya existe, cierra el gap de estabilidad con cero instrumentación nueva.
2. **Rework rate: DEFERIR** (no WONTFIT — el dato tiene valor: una tarea "completada" que reabre cuenta como éxito hoy, gap documentado en gap-01). Requiere instrumentación nueva: poblar `pattern`/`decision_reason` consistentemente en `campaign_update_task_state` (TSYS-09) o un state-transition log dedicado en `campaign-server.mjs`. Re-evaluar cuando TSYS-09 esté completo.

**Justificación del split:** la pregunta de la tarea ("¿la telemetría actual alcanza?") tiene respuesta distinta por métrica: recovery SÍ (verify-log alcanza), rework NO (nada registra reabiertos de forma estructurada). Implementar recovery hoy no bloquea nada y da el 50% del valor con costo cero de telemetría.

## Contrato
- ✅ `verify-log.jsonl` verificado por lectura directa (existe, 23 líneas, campos documentados).
- ✅ Recomendación explícita presente (sección 3): **IMPLEMENTAR** (recovery) + **DEFERIR** (rework).
- Read-only: no se tocó código ni telemetría. Sin commit (lead commitea).