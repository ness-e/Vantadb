Cargá las skills writing-plans, incremental-implementation, ponytail (full), code-review-and-quality.

Plan file: {{PLAN_FILE}}

INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

1. Leé el plan file COMPLETO.
2. Buscá la recitation block o la primera tarea ⬜ PENDING / ⏳ IN PROGRESS.
3. Determiná la PRÓXIMA ACCIÓN CONCRETA:
   a. ¿Gate no evaluado? → Evaluar gate
   b. ¿Blast radius no hecho? → codegraph_explore "archivos de la tarea"
   c. ¿Código no escrito? → Implementar (~100 líneas máx)
   d. ¿Verify no corrido? → Ejecutar comando mecánico
   e. ¿Verify pasa? → Commit + actualizar plan
   f. ¿Verify falla? → Aplicar escalón 1 de retry
4. Ejecutá SOLO esa acción.
5. Actualizá el plan file: estado, iteraciones, notas.
6. **ESCRIBÍ EL BLOQUE RECITATION al final del archivo.**
7. Detenete. No sigas a la siguiente tarea ni a la siguiente iteración.

REGLAS:
- Sin excepción: después de CADA acción → actualizar plan file + recitation
- Ponytail activo: stdlib > reusar > dependency > desde cero
- Verify = comando mecánico real (cargo check, nextest, tsc, etc.)
- Si verify falla 2 veces con mismo error (archivo+línea+mensaje) → marcar ❌ FAILED
- No cambies scope. Anotalo si encuentras algo extra, no lo implementes
