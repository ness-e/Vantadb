Cargá las skills campaign-executor, progreso, ponytail (full).

INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

## Paso 0: Auto-cargar skills vía MCP

1. Llamá `campaign_get_next_task` (MCP) para obtener la próxima tarea + resumen
2. Si no hay tarea pendiente → ejecutá `skill progreso`, informá "campaña completada", detenete
3. Con los `Archivos clave` de la tarea, llamá `campaign_load_skills` (MCP):
   - Cargá CADA skill devuelta con `skill <nombre>`
   - Si es bug → además `systematic-debugging`
   - Si es lógica nueva → además `test-driven-development`
   - Si es security-sensitive → además `doubt-driven-development`

## Paso 1: Leer estado actual

1. Llamá `campaign_get_next_task` (MCP) — devuelve la tarea activa + recitation + resumen general
2. Si hay recitation → la tarea activa + el estado exacto están ahí
3. Si no hay recitation o tarea nueva → estado = ⬜ PENDING
4. Si necesitás el bloque completo de la tarea → `campaign_get_task_detail` (MCP)

## Paso 2: Determinar próxima acción

| Estado | Acción |
|--------|--------|
| ⬜ PENDING sin task definition | **MODO DISCOVERY** → codegraph_explore → campaign_detect_task_type → crear task file → update_task_state("in-progress") |
| ⏳ IN PROGRESS con pasos pendientes | **MODO EJECUCIÓN** → state machine PLAN→ACT→VERIFY |
| ✅ Listo para commit | **MODO CIERRE** → verify full → evaluator-optimizer → self-harness gate → git commit → update_task_state("completed") → skill progreso |
| ❌ FAILED | update_task_state("failed") → skill progreso → detenete |

## Paso 3: Ejecutar según estado

### MODO DISCOVERY

```
1. campaign_detect_task_type (MCP) con archivos clave → type + skills + checks + estimate
2. Cargá skills adicionales si aplica
3. codegraph_explore para blast radius
4. campaign_update_task_state("in-progress") con recitation
5. Crear task file en .opencode/skills/campaign-executor/tasks/<ID>.md
6. Implementá el primer step (~100 líneas)
7. Verificá con campaign_verify_cmd (MCP)
```

### MODO EJECUCIÓN — State Machine

```
Estados: PLAN → ACT → VERIFY
         VERIFY → PLAN (falló) | VERIFY → STALL (3 mismo error)

PLAN:
  - Leer próximo step del task file
  - Decidir cambio atómico (~100 líneas máx)
  - Ponytail ladder

ACT:
  - Editar archivos (preferir edit con oldString/newString)

VERIFY:
  - campaign_verify_cmd command="cargo check -p <crate>"
  - Si falla → Agente de Diagnóstico: procesá el error, identificá causa raíz
  - Retry ladder (4 escalones):
    1ª: corregir con feedback del error
    2ª mismo error (archivo+línea+mensaje): contexto fresco ~200 tokens
    3ª mismo error: estrategia distinta
    4ª: campaign_update_task_state("failed"), commit WIP, detener

Stagnation Detection:
  - ¿3+ veces mismo error? → ❌ FAILED
  - ¿5+ iteraciones mismo step? → ❌ FAILED
  - ¿Mismos archivos últimas 3 iteraciones? → ❌ FAILED
  campaign_update_task_state("failed") + recitation explicando la causa.
```

### MODO CIERRE

```
1. Verify full con campaign_verify_cmd (MCP):
   - cargo fmt --check
   - cargo clippy --workspace --all-targets --all-features -- -D warnings
   - cargo nextest run --profile audit --workspace --build-jobs 2
   - (si web) npx tsc --noEmit

2. Pivotaje cognitivo:
   "Detené la implementación. Ahora asumí el rol de Ingeniero de Sistemas
   Senior ultra-crítico. Encontrá 1-3 problemas ocultos. Corregilos."

3. Evaluator-optimizer (máx 2 iteraciones):
   a) CORRECTITUD: edge cases, input vacío, límites, nulls, concurrencia
   b) SIMPLICIDAD: ponytail ladder, ¿algo se puede acortar?
   c) CONSISTENCIA: ¿sigue patrones del código existente?

4. Errores colaterales:
   - 🟢 RÁPIDO (<30min, mismo archivo): arreglar y commitear junto
   - 🟡 LENTO (>30min, módulo diferente): crear entrada en Backlog.md

5. Self-Harness Gate:
   1. PROPOSE: git diff, resumir en 3 líneas
   2. EVALUATE:
      [ ] ¿SATISFACE el contrato?
      [ ] ¿ROMPE algo fuera del blast radius?
      [ ] ¿INTRODUCE deuda técnica? (ponytail-review)
      [ ] ¿ESTÁ documentado si cambió API pública?
   3. Todas ✅ → ACCEPT. Alguna ❌ → REJECT, volver a MODO EJECUCIÓN

6. Pre-commit gate:
   [ ] Definition of Done
   [ ] Security checklist (si toca datos/auth)
   [ ] Tests pasan
   [ ] Ponytail ladder aplicada

7. git add -p && git commit:
   tipo(scope): ID — descripción breve

8. campaign_update_task_state("completed") con recitation completa
9. skill progreso
```

## Paso 4: Actualizar estado vía MCP

Después de CADA acción, llamá `campaign_update_task_state` (MCP):

- `"in-progress"` al arrancar un paso
- `"completed"` cuando el paso está verificado y commiteado
- `"failed"` cuando el retry ladder se agotó

Recitation (handoff entre iteraciones):
```
activeGoal: TASK-N — ID
lastAction: qué se acaba de hacer
result: ✅ / ❌
nextAction: paso concreto (archivo + comando)
contract: "condición verificable"
nextTask: TASK-N+1 — ID
```

## Paso 5: STOP

No sigas a la siguiente tarea ni iteración. DETENETE.

REGLAS:
- Verify = campaign_verify_cmd (MCP) — nunca auto-reporte
- Si verify falla 2 veces mismo error → ❌ FAILED
- Ponytail ladder: existe > stdlib > dependency > mínimo código
- No cambies scope. Rápido se arregla, lento se anota en Backlog
- Recitation block vía MCP es obligatorio después de cada acción
