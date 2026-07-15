Cargá las skills backlog-executor, ponytail (full).

Archivos de referencia (leer al inicio):
- `docs/plans/2026-07-13-verified-backlog-execution.md` — plan de ejecución, tareas, estados, recitation
- `docs/Backlog.md` — backlog verificado con IDs (INT-*, VFY-*, NUEVO-*, MKT-*, DEVOPS-*, ENT-*, LEG-*)
- `docs/bitacora.md` — bitácora consolidada (Jul 13)
- `.opencode/AGENTS.md` — Skill Loading Guide con tabla de skills por fase, arquitectura, comandos

── INSTRUCCIONES — UNA ITERACIÓN POR TURNO ──

Cada invocación ejecuta EXACTAMENTE UNA acción. No loopees. El plugin/harness
itera por vos. Leé el plan file COMPLETO primero, luego decidí qué rama seguir.

=== RAMA A: PRIMERA VEZ EN ESTA TAREA (⬜ PENDING) ===

PASO 0 — PROGRESO
Cargá `skill progreso` para leer el backlog, verificar WIP, y preparar
contexto de la sesión. Esto se hace al inicio de CADA nueva tarea.

PASO 1 — SKILLS
Cargá las skills según la tabla **Skill Loading Guide — Ingeniería** del
`AGENTS.md`. La tabla mapea cada fase del lifecycle:

| Fase    | Skills a cargar                                          |
|---------|----------------------------------------------------------|
| DEFINE  | `spec-driven-development`, `idea-refine`                 |
| PLAN    | `planning-and-task-breakdown`, `writing-plans`            |
| BUILD   | `ponytail` (full), `incremental-implementation`,          |
|         | `doubt-driven-development`, `source-driven-development`   |
| BUILD   | `api-and-interface-design` (si toca API pública)         |
| (UI)    | `frontend-ui-engineering` (si toca web/)                  |
| VERIFY  | `debugging-and-error-recovery` (si tests fallan)          |
| REVIEW  | `code-review-and-quality`, `code-simplification`,         |
|         | `security-and-hardening`                                  |
| SHIP    | `git-workflow-and-versioning`, `documentation-and-adrs`   |

Ponytail full siempre activo como base.

PASO 2 — BLAST RADIUS
Usá `codegraph_explore` para mapear el impacto del cambio:
  - Callers: qué módulos llaman a los archivos a modificar
  - Callees: de qué dependen esos archivos
  - Implicaciones:
    · ¿Se rompen contratos existentes?
    · ¿Cambia comportamiento público (API, CLI, SDK)?
    · ¿Afecta performance, memoria, serialización?
    · ¿Requiere migración de datos o re-indexación?
    · ¿Afecta tests existentes?

PASO 3 — IMPLEMENTAR
  - Cambio atómico (~100 líneas máx por iteración)
  - Ponytail ladder: ¿ya existe? > stdlib > dependency instalada > mínimo código
  - Si el cambio es complejo, usá sub-agentes via `task` tool:
    · Un sub-agente para codegraph_explore + análisis
    · Otro para implementar
    · Otro para verify

PASO 4 — VERIFICAR
Ejecutá el comando mecánico real:

| Lenguaje | Comando |
|----------|---------|
| Rust     | `cargo build && cargo nextest run --profile audit --workspace --build-jobs 2` |
| Rust     | `cargo fmt --check && cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Web      | `npx tsc --noEmit` |
| Python   | `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/ -v` |

Si verify falla:
  - 1ª vez → corregí el error específico
  - 2ª vez con el mismo error (archivo+línea+mensaje) → ❌ FAILED + documentar

PASO 5 — DOCUMENTAR RELACIONES
En el commit message, incluí:
  - Referencia cruzada (REC-NN, PERF-NN, PN, WN, etc.)
  - Dependencias/blast radius detectados
  - Breaking changes o implicaciones (si las hay)

=== RAMA B: TAREA YA IMPLEMENTADA (⏳ IN PROGRESS / ❌ FAILED) ===

PASO 1 — SKILLS DE REVISIÓN
Cargá: `code-review-and-quality`, `doubt-driven-development`, `code-simplification`

PASO 2 — VERIFICACIÓN POST-IMPLEMENTACIÓN
Usá `codegraph_explore` para verificar el impacto completo:
  - ¿Los módulos dependientes compilan y pasan tests?
  - ¿Se rompió alguna interfaz pública o contrato existente?
  - ¿Hay implicaciones no contempladas?
    · Seguridad (input validation, auth, data exposure)
    · Performance (allocaciones, locks, I/O)
    · Migración de datos o formatos
    · Edge cases (nulls, empty collections, concurrent access)

PASO 3 — OPTIMIZACIÓN
  - ¿Código más simple de lo que quedó? ¿Se puede reducir?
  - ¿Tests faltantes para cubrir el cambio?
  - ¿Casos borde no manejados?
  - ¿Documentación afectada?

PASO 4 — READINESS CHECK
Decidí y registrá en el plan file:
  - ✅ Listo para mergear
  - ❌ Falta algo (especificar qué en notas)
  - 🟡 Deferir tareas adicionales encontradas

=== PARA AMBAS RAMAS (post-acción) ===

1. Ejecutá SOLO la acción de la rama que elegiste. No mezcles.
2. Si la acción es codegraph, implementar o verify → usá sub-agentes via `task` tool
   para ejecutar pasos en paralelo.
3. Si la tarea está completa (✅) → ejecutá `skill progreso` (Trigger 1) para migrar
   TODAS las tareas completadas (Backlog.md + bitacora.md + plan file) a
   docs/progreso/README.md ANTES de actualizar los archivos. Verificá que no se
   dupliquen entradas existentes (chequear IDs en progreso antes de agregar).
4. Actualizá los 3 archivos:
   - `docs/plans/2026-07-13-verified-backlog-execution.md`: estado, iteraciones, notas, commit
   - `docs/Backlog.md`: marcar tarea como completada si corresponde
   - `docs/bitacora.md`: marcar issue como resuelto si corresponde
5. **REVISIÓN CADA 10 TAREAS.** Contá las tareas completadas ✅ en el status
   tracker del plan file. Si el total es múltiplo de 10 (10, 20, 30...):
   - Re-leé los 3 archivos (plan file, Backlog.md, bitacora.md)
   - Verificá que progreso/README.md tenga todas las entradas sin duplicados
   - Corregí entradas faltantes, errores, o desorganización
   - Anotá "Review N/10: OK" en el plan file
   esto evita acumular deuda de doc y mantiene consistencia.

6. **ESCRIBÍ EL RECITATION BLOCK** al final del plan file con:

   ```
   === RECITATION ===
   Objetivo activo: TASK-NN — Ref ID
   Estado: implemented / verifying / CI / completed / failed
   Última acción: qué se acaba de hacer
   Resultado: ✅ pasa / ❌ falla (con error)
   Próxima acción: el PRÓXIMO paso concreto (archivo + comando)
   Contrato: "just verify pasa"
   Próxima tarea si completa: TASK-NN+1 — Ref ID
   === END RECITATION ===
   ```

7. **DETENETE.** No sigas a la siguiente tarea ni iteración.

REGLAS GLOBALES:
- Ponytail full: ¿ya existe? > stdlib > dependency instalada > mínimo código
- Verify = comando mecánico real (no auto-reporte)
- Verify falla 2 veces con mismo error → ❌ FAILED + documentar por qué
- No cambies scope. Si encontrás algo extra → anotalo en notas, no lo implementes
- Cada acción termina con plan file actualizado + recitation escrita
- Sub-agentes para paralelizar: codegraph_explore, implementación, verify
