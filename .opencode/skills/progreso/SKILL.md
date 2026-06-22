---
name: progreso
description: >
  Gestiona el historial unificado de progreso del proyecto VantaDB:
  mueve tareas completadas entre docs/Backlog.md y
  docs/progreso/README.md, y mantiene la documentación del proyecto.
compatibility: opencode
---

# POLÍTICA CRÍTICA DE RETENCIÓN DE HISTORIAL EN ARCHIVO UNIFICADO (FMEA PREVENTIVO)

## Ubicaciones Clave

- **Backlog activo:** `./docs/Backlog.md` — tareas pendientes
- **Historial unificado de progreso:** `./docs/progreso/README.md` — tareas completadas + hitos
- **Archivos de planificación:** `implementation_plan.md`, `task.md`, `walkthrough.md`

### Flujo de Archivos (cómo se relacionan)

1. **Backlog.md** contiene tareas PENDIENTES con status ❌ (o ⏸️)
2. Cuando una tarea alcanza ✅, se MUeVE de Backlog.md a progreso/README.md (sección "Tareas Completadas (Migradas desde Backlog)")
3. **progreso/README.md** contiene SOLO tareas completadas + hitos + auditorías
4. Backlog.md y progreso/README.md son MUTUAMENTE EXCLUYENTES — ninguna tarea aparece en ambos

El sistema por defecto sobrescribe los artifacts de planificación locales (`implementation_plan.md`, `task.md`, `walkthrough.md`) al iniciar nuevas actividades, lo cual causa pérdida de datos históricos. Para prevenir esto y mantener el repositorio limpio y estructurado sin subcarpetas innecesarias, DEBES cumplir estrictamente este protocolo de registro en el Historial Unificado de Progreso:

## Trigger 1: Al concluir una actividad (Creación/Actualización de Walkthrough)

Inmediatamente después de generar o actualizar el `walkthrough.md` como paso final de una implementación:

### Paso A: Extraer datos de la tarea completada
Identificador, nombre de la tarea, fecha, objetivo del `implementation_plan.md` y `walkthrough.md`.

### Paso B: Mover tarea a Historial Unificado
1. Lee `./docs/Backlog.md` — encuentra la fila de la tarea completada (✅).
2. Lee `./docs/progreso/README.md` — encuentra dónde insertar la nueva entrada.
3. En `./docs/Backlog.md`:
   - **Elimina la fila** de la tarea completada de su tabla.
   - Si la subsección queda vacía, elimínala por completo.
4. En `./docs/progreso/README.md`:
   - Agrega una entrada estructurada en la sección "Tareas Completadas (Migradas desde Backlog)" o en el "Progreso Reciente" si es un hito significativo.
   - Incluye: identificador, objetivo, checklist completado (`[x]`), archivos modificados, walkthrough.
5. **Guarda ambos archivos.**

### Paso C: Registrar en Changelog
Si existe `./docs/CHANGELOG.md`, agrega una entrada con:
- Fecha en formato ISO 8601
- Versión del proyecto (`v<version>`)
- Nombre de la tarea como encabezado
- Objetivo de la tarea
- Archivos afectados (lista)
- Resultado obtenido

### Paso D: Registrar en progreso Reciente (opcional)
Si la tarea fue un hito significativo, agrega una entrada breve en la sección "Progreso Reciente" de `./docs/progreso/README.md` además de la migración.

### Paso E: Commit
```bash
git add docs/Backlog.md docs/progreso/README.md
git commit -m "docs: move <tarea> completada a historial"
```

### Paso F: Notificar
Informa al usuario que Backlog.md y progreso/README.md han sido actualizados.

## Trigger 2: Al recibir una solicitud para una NUEVA actividad

Si el usuario te pide una nueva tarea que requiere un nuevo `implementation_plan.md`, ANTES de sobrescribir los archivos en tu memoria local de artifacts:

1. Abre `./docs/progreso/README.md` y verifica si la última tarea completada ya se encuentra registrada.
2. Si no se encuentra registrada, ejecuta inmediatamente **Trigger 1** usando los artifacts locales de la tarea anterior.
3. Solo cuando la actualización esté confirmada, procede a generar el nuevo Plan de Implementación.
4. Si la nueva tarea existe en `./docs/Backlog.md` como ❌, actualiza su estado a 🟡 En Progreso.

## Trigger 3: Mantenimiento periódico (revisión mensual)

1. Revisa `docs/Backlog.md` para identificar tareas que lleven >30 días como ❌ sin actividad.
   - Si aplica, muévelas a la sección ⏸️ Icebox.
   - Si ya no son relevantes, muévelas a ❌ No Hacer.
2. Revisa `docs/progreso/README.md` para identificar secciones duplicadas o desactualizadas.
3. Verifica que no haya tareas presentes en ambos archivos a la vez.
4. Verifica que `docs/Backlog.md.new` o archivos temporales no hayan quedado huérfanos.
