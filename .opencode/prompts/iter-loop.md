Cargá las skills campaign-executor, ponytail (full).

INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

1. Encontrá el plan file más reciente en docs/plans/ (Get-ChildItem |
   Sort-Object LastWriteTime -Descending | Select-Object -First 1).
   Si no hay plan file → informá y detenete.

2. Leé el plan file COMPLETO.

3. Buscá la recitation block o la primera tarea ⬜ PENDING / ⏳ IN PROGRESS.

4. Determiná la PRÓXIMA ACCIÓN CONCRETA:
   a. ⬜ PENDING sin task definition → MODO DISCOVERY:
      codegraph_explore para blast radius, crear task file en
      `.opencode/skills/campaign-executor/tasks/<ID>.md`
   b. ⏳ IN PROGRESS con pasos pendientes → MODO EJECUCIÓN:
      implementar un paso atómico (~100 líneas máx)
   c. ✅ Listo para commit → MODO CIERRE:
      verify full (build + nextest + fmt + clippy), commit, skill progreso
   d. ❌ FAILED → pasá a la siguiente tarea, anotá por qué

5. Ejecutá SOLO esa acción.

6. Actualizá: plan file (estado, iteración, notas) + recitation block al final.

7. DETENETE. No sigas a la siguiente tarea ni iteración.

REGLAS:
- Verify = comando mecánico real (cargo check, nextest, fmt, clippy, tsc, pytest)
- Si verify falla 2 veces con mismo error (archivo+línea+mensaje) → ❌ FAILED
- Ponytail ladder: ya existe > stdlib > dependency > mínimo código
- No cambies scope. Si encontrás algo extra → anotalo, no lo implementes
- Recitation block obligatorio al final del plan file
