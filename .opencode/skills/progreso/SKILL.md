---
name: progreso
description: >
  Gestiona el historial unificado de progreso del proyecto VantaDB:
  registra tareas completadas en docs/progreso/README.md, mantiene
  sincronizado el vault MPTS en Obsidian y actualiza el backlog.
compatibility: opencode
---

# POLÍTICA CRÍTICA DE RETENCIÓN DE HISTORIAL EN ARCHIVO UNIFICADO (FMEA PREVENTIVO)

## Ubicaciones Clave

- **Backlog maestro (Obsidian):** `C:\Users\Eros\Obsidian\Eros\Backlog.md`
- **Changelog del proyecto (Obsidian):** `C:\Users\Eros\Obsidian\Eros\Changelog.md`
- **Documentación del proyecto (Obsidian MPTS):** `C:\Users\Eros\Obsidian\Eros\VantaDB-MPTS\`
- **Historial unificado de progreso:** `./docs/progreso/README.md`
- **Archivos de planificación:** `implementation_plan.md`, `task.md`, `walkthrough.md`

El sistema por defecto sobrescribe los artifacts de planificación locales (`implementation_plan.md`, `task.md`, `walkthrough.md`) al iniciar nuevas actividades, lo cual causa pérdida de datos históricos. Para prevenir esto y mantener el repositorio limpio y estructurado sin subcarpetas innecesarias, DEBES cumplir estrictamente este protocolo de registro en el Historial Unificado de Progreso:

## Trigger 1: Al concluir una actividad (Creación/Actualización de Walkthrough)

Inmediatamente después de generar o actualizar el `walkthrough.md` como paso final de una implementación:

1. Extrae los datos clave de la tarea recién completada (identificador/nombre de la tarea, fecha, objetivo).
2. Lee el archivo unificado de progreso: `./docs/progreso/README.md`.
3. Concatena (añade al final) una nueva entrada estructurada para la tarea completada que consolide:
   - **Metadatos de la Tarea:** Identificador, nombre de la tarea y fecha de finalización.
   - **Objetivo y Plan:** Resumen muy breve de lo que se planificó en el `implementation_plan.md`.
   - **Checklist de Tareas:** Copia de las tareas con estado completado (`[x]`) de `task.md`.
   - **Walkthrough y Cambios:** Resumen de las modificaciones de archivos y el impacto descrito en `walkthrough.md`.
   - **Modificaciones de última hora:** Si el plan cambió durante la ejecución, si se agregaron más pasos, o si se hicieron tareas adicionales antes de la consolidación final, detállalas bajo esta entrada.
4. Guarda las actualizaciones en `./docs/progreso/README.md`.
5. **Agrega una entrada en el Changelog** en `C:\Users\Eros\Obsidian\Eros\Changelog.md` con:
   - Fecha en formato ISO 8601
   - Versión del proyecto (`v<version>`)
   - Nombre de la tarea como encabezado
   - Objetivo de la tarea
   - Archivos afectados (lista)
   - Resultado obtenido
6. **Actualiza los documentos relevantes del vault MPTS** en `C:\Users\Eros\Obsidian\Eros\VantaDB-MPTS\` según el tipo de tarea completada:
   - **Cambios de CI/CD, testing, calidad** → actualiza `Operaciones, Calidad y Riesgos.md`
   - **Tareas del roadmap completadas** → actualiza `Roadmap e Hitos de Ingeniería.md` (marcar checkboxes, % de progreso)
   - **Cambios de arquitectura o core** → actualiza `Arquitectura Técnica y Core Engine.md`
   - **Cambios de API o SDK** → actualiza `Especificaciones Funcionales y SDK API.md`
   - **Cambios de estrategia o producto** → actualiza `Estrategia de Ecosistema y GTM.md` y/o `Visión y Posicionamiento Estratégico.md`
   - **Cambios de versión o estado general** → actualiza `Master Index.md` (versión, estado, last_refined)
7. Realiza un commit en Git que incluya `./docs/progreso/README.md`, `Changelog.md` y los archivos MPTS actualizados (ej: `git commit -m "docs: append <nombre-tarea-completada> to progress history, changelog, MPTS"`).
8. Informa al usuario que el "Historial Unificado de Progreso", "Changelog" y el vault MPTS han sido actualizados con éxito.

## Trigger 2: Al recibir una solicitud para una NUEVA actividad

Si el usuario te pide una nueva tarea que requiere un nuevo `implementation_plan.md`, ANTES de sobrescribir los archivos en tu memoria local de artifacts:

1. Abre `./docs/progreso/README.md` y verifica si la última tarea completada ya se encuentra registrada en él de forma consolidada.
2. Si no se encuentra registrada, ejecuta inmediatamente el proceso del **Trigger 1** utilizando el contexto y los artifacts locales de la tarea anterior que aún conservas.
3. Solo cuando la actualización del Historial Unificado esté confirmada y guardada en `./docs/progreso/README.md`, procede a sobrescribir y generar el nuevo Plan de Implementación local en tus artifacts.
4. Actualiza el backlog de Obsidian marcando la tarea como completada en `C:\Users\Eros\Obsidian\Eros\Backlog.md`.
5. Verifica que `C:\Users\Eros\Obsidian\Eros\Changelog.md` tenga una entrada para la tarea completada; si no, créala.
6. Verifica que los documentos del vault MPTS en `C:\Users\Eros\Obsidian\Eros\VantaDB-MPTS\` reflejen el estado actual del proyecto; si no, actualízalos antes de iniciar la nueva tarea.
