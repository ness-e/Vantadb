# TIR-07 · Chaos runner del task-system — ¿runner de fuzzing o tests puntuales?

> **Fecha:** 2026-08-17 · **Tipo:** Investigación/Decisión (read-only)
> **Fuente:** backlog P18 (re-verificado 2026-08-12) · **Contrato:** `docs/Backlog.md` P18
> **Alcance:** NO se implementó el runner, NO se tocó `campaign-server.mjs`.

## 1. Estado real del sistema (verificado contra código, no contra diseño)

El diseño T19 (`docs/architecture/task-system-chaos-resilience.md`, 2026-08-11) declara en §6 tres behavior changes "no implementados" como pre-condición de la suite. **Verificación del código actual (2026-08-17) — los tres YA están implementados:**

| Behavior change T19 §6 | Estado en `campaign-server.mjs` | Evidencia |
|---|---|---|
| (1) Corrupción de budget visible (C3/C7) | ✅ Implementado | `readBudget` → `{tasks:{}, budgetCorrupted:true}` ante JSON inválido; `writeBudget` strip del flag (L175-194) |
| (2) Writes con checksum + atómicos (C4/C6) | ✅ Implementado | `withPlanLock` (lock file O_EXCL + stale-detect), `sha1` + `detectConflict` antes del write, temp+rename (L579-690) |
| (3) Claim WIP atómico (C12) | ✅ Implementado | `findInProgressTasks` corre DENTRO del lock — check-and-set (L631-642) |

**Cobertura de tests ya existente** (Node v24 built-in, cero deps):

| Escenario T19 | Cobertura | Archivo |
|---|---|---|
| C1 plan corrupto (header truncado, `### Task` suelto, header roto, vacío) | ✅ | `parsers.test.mjs` |
| C2 task removida in-progress | ✅ | `parsers.test.mjs` + `state-persistence.test.mjs` |
| C3 budget JSON truncado/inválido | ✅ | `state-persistence.test.mjs` |
| C4 update concurrente (mecanismo detectConflict) | ✅ (mecanismo) | `state-persistence.test.mjs` |
| C5 campaign ID idempotente | ✅ (mecanismo) | `parsers.test.mjs` |
| C6 write atómico / sin .tmp residual | ✅ (mecanismo) | `state-persistence.test.mjs` |
| C7 writeBudget truncado | ✅ | `state-persistence.test.mjs` |
| C8 verify_cmd killed | ❌ | — |
| C9 retry re-entrante de verify | ❌ | — |
| C10 estado desconocido en validateAction | ⚠️ behavior actual correcto (deny-by-default), sin test directo | `state-tools.mjs:73-84` |
| C11 plan faltante / contenido vacío | ✅ | `parsers.test.mjs` |
| C12 doble claim WIP | ✅ (secuencial) | `state-persistence.test.mjs` |

**Gaps reales:** C8, C9, y las variantes *verdaderamente concurrentes* de C4/C5/C12 (los tests actuales simulan la carrera comprimiendo el tiempo dentro del lock, no con procesos paralelos). C10 es trivial de cubrir.

## 2. Qué cubriría un runner de fuzzing dedicado

Un runner real (según §5 de T19) agregaría:

- **Fuzzing de entradas**: mutación aleatoria de plan files (markdown arbitrario), budget JSON, nombres de estado/tool contra `parseTasks`/`extractState`/`validateAction`.
- **Kills reales**: `Stop-Process` en puntos guionados (read→write, writeBudget, verify_cmd) para validar atomicidad con procesos reales.
- **Concurrencia real**: dos invocaciones paralelas del mismo tool contra el mismo plan (C4/C5/C12 sin simulación).
- **Oráculo**: invariantes por escenario (nunca panic, nunca pérdida silenciosa, corrupción visible).

**Superficie a fuzzear:** el server es 1517 líneas, single-process, con parsers regex sobre markdown + JSON; la máquina C0 es un conjunto cerrado (10 estados × ~10 tools). No es un parser de formato abierto con espacio de entrada ilimitado — es un estado finito con superficie de riesgo ya **enumerada** (C1-C12).

## 3. Comparación: runner dedicado vs tests de inyección puntuales

| Dimensión | Runner de fuzzing dedicado | Tests puntuales (patrón existente) |
|---|---|---|
| **Costo construir** | 🔴 2-3d: harness (spawn/kill del server), generadores de mutación, oráculo, fixtures, integración sandbox (`campaign_run_sandboxed`) | 🟢 horas: extender `state-persistence.test.mjs`/`parsers.test.mjs` con el patrón ya probado (temp dir + import directo vía isMain guard) |
| **Costo mantener** | Alto: cada cambio de schema del server (tools, recitation, persistencia) rompe fixtures; runner debe syncear con C0 | Bajo: tests por función exportada, sin acoplar al transporte |
| **Valor marginal sobre lo existente** | Bajo: la mayoría de los escenarios ya están cubiertos; el fuzzing de markdown/JSON redis cubriría lo que los tests C1-C3 ya prueban de forma dirigida | Alto: cierra los 2 gaps reales (C8, C9) + variantes concurrentes reales con el mismo patrón |
| **Riesgo de falsos positivos** | Medio: mutaciones aleatorias generan "fallos" que son comportamiento esperado (degradación graceful), hay que clasificar | Bajo: escenarios dirigidos con invariante explícito |

**Observación clave:** la decisión del backlog ("DEFER sin verificación") asumía que faltaba toda la infraestructura. La realidad: los 3 pre-requisitos de T19 ya están implementados y ~9/12 escenarios ya tienen test dirigido. El runner de fuzzing **hoy** sería construir 2-3 días de harness para redis cubrir lo que ya está cubierto, más los 2 gaps que se cierran en horas.

## 4. Recomendación

**DEFERIR el runner de fuzzing dedicado** (no WONTFIT — la preocupación de caos es real, como demostró el propio WIP hard-limit al bloquear tareas concurrentes; no implementar — el costo/valor no lo justifica hoy).

**Justificación:**
1. Los 3 behavior changes pre-condición de T19 ya están en el código; la suite tiene 2 archivos de tests puntuales cubriendo ~9/12 escenarios con el patrón correcto (temp dirs, imports directos, cero deps).
2. El valor restante (C8, C9, concurrencia real) se captura con **tests puntuales** en horas, no con un runner de 2-3 días.
3. La superficie es un estado finito con riesgos enumerados (C1-C12); el fuzzing de entradas tiene rendimientos decrecientes: los parsers ya degradan graceful y los errores se capturan con try/catch.

**Acción inmediata sugerida (fuera de esta tarea read-only, como tarea de test):**
- Extender `state-persistence.test.mjs` con: C8 (kill de verify_cmd — no-op sobre estado), C9 (retry appenda 2ª línea JSONL, budget no doble-gasta), y variantes con 2 procesos reales para C4/C5/C12.
- Un test directo de `validateAction` con estado/tool desconocidos (C10) en `parsers.test.mjs` o test nuevo de `state-tools.mjs`.

**Condición de re-evaluación (cuándo SÍ construir el runner):** si el server crece en superficie de persistencia (nuevos archivos, multi-proceso, tools dinámicas), o si un incidente real de corrupción demuestra que los tests dirigidos no lo capturaron. En ese punto, el runner reutiliza el diseño T19 §5 tal cual (sandbox `campaign_run_sandboxed`, fixtures staged, `vanta-chaos` como owner).

## Referencias

- `docs/architecture/task-system-chaos-resilience.md` (T19 — diseño, escenarios C1-C12, §5 estrategia, §6 pre-condiciones)
- `docs/Investigaciones/2026-08-10-agent-engineering/gap-01-agents.md` — fila 36 / FALTA #24 (chaos/resilience del pipeline, P3) y §6.1-8 fallas reales
- `.opencode/task-system/mcp/campaign-server.mjs` — 1517 líneas; helpers, budget (L163-247), update_task_state con lock/checksum (L570-756), isMain guard (L1508)
- `.opencode/task-system/mcp/state-persistence.test.mjs` (210 líneas) y `parsers.test.mjs` (165 líneas)
- `.opencode/task-system/config/state-tools.mjs` — STATE_TOOLS, validateAction (94 líneas)
- `.opencode/task-system/mcp/parsers.mjs` — extractField/extractState/parseTasks/updateState/getOrCreateCampaignId