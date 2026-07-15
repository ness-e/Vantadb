---
description: Crear plan de campaña desde backlog y obtener comando de ejecución
---

Cargá las skills brainstorming, writing-plans, progreso, ponytail (full).

Backlog: $1
Si no se especificó ruta, usá `docs/Backlog.md`.

Aplicá el triage gate del backlog-executor a CADA tarea en el backlog.
Resultados posibles: ✅ DO, 🟡 DEFER, ❌ SKIP, 🔴 BLOQUEADO.

Reglas del gate:
1. Bug ya inexistente o feature ya implementada → SKIP
2. Cosmético sin queja de usuario → DEFER
3. Esfuerzo >> impacto → DEFER o SKIP
4. Dependencia no lista → BLOQUEADO
5. Prioridad original es sugerencia, no orden

Después del gate, creá `docs/plans/<FECHA>-<nombre>.md` con:
- Nombre descriptivo corto basado en el contenido del backlog
- Solo tareas ✅ DO, ordenadas por prioridad real
- Gate result y Gate Justificación para cada una
- Contrato verificable para cada una (condición booleana que un comando puede verificar)
- Estado inicial ⬜ PENDING
- Tabla resumen al inicio (DO / DEFER / SKIP / BLOQUEADO counts)
- Fuente del backlog

Al final del archivo, mostrame el comando exacto para ejecutar el plan:

**Opción recomendada (bloqueante, espera a que cada iteración termine):**
```
.\harness-executor.ps1 -PlanFile docs\plans\<FECHA>-<nombre>.md -Interval 10
```

**Alternativa (loop plugin — requiere intervalo >0s para evitar overlap):**
```
/loop 15s --prompt-file .opencode/skills/backlog-executor/iter-prompt.md
```
⚠️ NO usar `0s` — causa race condition: el loop se dispara en cada idle event
antes de que la iteración anterior termine. Usar ≥10s para que el intervalo
amortigüe el overlap.
