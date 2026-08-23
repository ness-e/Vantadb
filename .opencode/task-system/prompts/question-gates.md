> **CANONICAL SPEC — Question Gates HITL (fuente única, TSYS-H7)**
> Referenciado por: plan.md (Gate P) · pipeline-full.md / iter-loop-tools.md (Gates D/V/C)
> · pipeline-run.md · subagent-recovery.md. Los prompts REFERENCIAN este archivo;
> nunca lo redefinen. Si este archivo cambia, los gates cambian en un solo lugar.

# Question Gates — control del usuario vía `question` tool

## Principios

1. Las questions SIEMPRE llevan opciones concretas + default recomendado
   (`(Recomendado)` primero en la lista). Nunca preguntas abiertas sin contexto.
2. El resultado de cada gate se registra en la recitation (`contract` o
   `lastAction`) y, si cambia dirección, vía `campaign_emit_event` con
   `decision_reason` (queda en el trace log).
3. Máximo 1 ronda de questions por gate por tarea (el usuario ya decidió; no
   re-preguntar lo decidido salvo que el código contradiga la decisión).
4. Si el harness no expone `question`, fallback: reporte estructurado y STOP
   esperando input del usuario (nunca asumir GO).

## Routing de questions (quién pregunta)

| Contexto | Cómo se pregunta |
|---|---|
| Orquestador (`/pipeline`, `/pipeline run`, vanta-lead) | Directo vía `question` tool |
| Sub-agente CON `question` en permissions (hoy: solo vanta-lead, vanta-review) | Directo |
| Sub-agente SIN `question` (worker/arch/engine/audit/chaos/tuner/docs/research) | Devolver bloque `RESULTADO` con `BLOQUEO: <gate + opciones>` — **el orquestador hace la pregunta** al recibirlo y reanuda vía SARL con la respuesta. El sub-agente NUNCA asume GO. |

> Estado 2026-08-23 verificado con grep sobre `.opencode/agents/*.md`: solo
> vanta-lead y vanta-review referencian `question`. Al dar `question` a más
> agentes (TSYS-11), actualizar esta tabla.

## Gate P — Plan/Triage (en plan.md)

**Cuándo:** durante el triage del backlog, ANTES de fijar el gate final.

| Disparador | Question |
|---|---|
| Tarea prioridad 🔴 | Confirmar inclusión/scope: opciones GO (como está) / Ajustar scope / DEFER / SKIP |
| Contrato ambiguo (≥2 interpretaciones que cambian el resultado) | Mostrar las interpretaciones y pedir elección |
| feature-add o lógica nueva | **Spec-driven guiado**: generar mini-spec con `prompts/spec-template.md` → preguntar al usuario las decisiones abiertas de §5 (una ronda, opciones + default recomendado) ANTES de aprobar DO. La spec confirmada viaja al task file (sección `## Spec`) — sin ella no hay ACT. |
| Resumen final del triage | Confirmar el set DEFER/SKIP completo antes de escribir el plan file |

## Gate D — Discovery (en pipeline-full.md §Discovery, iter-loop-tools MODO DISCOVERY)

**Cuándo:** tras zero-code planning, ANTES de escribir el task file.

| Disparador | Question |
|---|---|
| Blast radius > 10 archivos o toca hot path/WAL/API pública | Mostrar blast radius resumido + approach propuesto → GO / ajustar / dividir en tareas |
| Contrato ambiguo descubierto en código (Paso 0 no lo vio) | Detener y elegir entre caminos válidos |
| Tarea feature-add/lógica nueva sin mini-spec | Generar mini-spec y confirmar criterio de aceptación con el usuario |

## Gate V — Verify-falla×2 (en pipeline-full.md, subagent-recovery.md)

**Cuándo:** se alcanzó el umbral único de **2 fallas de verify con el mismo error**
(archivo+línea+mensaje). Reemplaza el escalate-a-humano silencioso.

Question obligatoria con evidencia: los 2 errores, qué intentos hubo, opciones:
1. Reintentar con contexto fresco (RETRY, consume budget)
2. Cambiar de estrategia (STRATEGY — describí la alternativa propuesta)
3. Marcar ❌ FAILED y pasar a la siguiente (FAIL_MODE aplica)

Sin respuesta → STOP (no marcar FAILED unilateralmente salvo FAIL_MODE=stop explícito).

## Gate C — Cierre (en pipeline-full.md MODO CIERRE)

**Cuándo:** durante el cierre, antes del commit final.

| Disparador | Question |
|---|---|
| Errores colaterales encontrados | Arreglar ahora (<30min, mismo archivo) / mandar a Backlog / incluir en este commit |
| `git status` muestra archivos FUERA del blast radius declarado | Incluirlos en el commit / dejarlos sin commit (staged aparte) / investigar origen |
| Plan completado (cierre de campaña) | Confirmar archivado del plan y migración masiva a progreso |

## Anti-abuso

- Los gates NO aplican a tareas 🟢 triviales con contrato mecánico claro.
- Una familia aprobada explícitamente por el usuario en esta sesión (mismo plan,
  mismo objetivo) no re-dispara Gate P/D individualmente (ver subagent-recovery §5).
- Si el plan file declara `> **Autonomous:** true` (leído vía
  `campaign_get_next_task` → campo `autonomous`), solo operan Gate V y
  Gate C-casos-de-seguridad (archivos fuera de blast radius). Sin el flag,
  operan los 4 gates.
