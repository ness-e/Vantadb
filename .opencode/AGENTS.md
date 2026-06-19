# Directivas del Proyecto VantaDB (Rust Core)

## Flujo de Progreso

Este proyecto VantaDB usa un skill de progreso para mantener el historial unificado:
`./.opencode/skills/progreso/SKILL.md`

- **Al iniciar una nueva tarea:** carga el skill `progreso` y sigue sus triggers.
- **Al completar una tarea:** aplica el skill `progreso` (Trigger 1) ANTES de cualquier mensaje de resumen.
- **Backlog maestro:** `C:\Users\Eros\Obsidian\Eros\Backlog.md`
- **Changelog:** `C:\Users\Eros\Obsidian\Eros\Changelog.md`

## Comportamiento General

- Preserva siempre el contenido de `./docs/progreso/README.md` — es el historial inmutable del proyecto.
- No sobrescribas archivos de planificación sin antes haber consolidado la tarea anterior en el historial unificado.
