---
name: progreso
description: >
  Gestiona el historial unificado de progreso del proyecto VantaDB:
  mueve tareas completadas entre docs/Backlog.md y
  docs/progreso/README.md, registra investigaciones y
  mantiene la documentación del proyecto.
compatibility: opencode
---

# POLÍTICA CRÍTICA DE RETENCIÓN DE HISTORIAL EN ARCHIVO UNIFICADO (FMEA PREVENTIVO)

## Ubicaciones Clave

- **Backlog:** `./docs/Backlog.md` — tareas pendientes priorizadas

- **Historial unificado de progreso:** `./docs/progreso/README.md` — tareas completadas + hitos + auditorías
- **Release notes formales:** `./docs/CHANGELOG.md` — changelog por versión (formato keepachangelog)
- **Investigaciones:** `./docs/Investigaciones/` — research artifacts organizados por tema
- **Archivos de planificación local:** `implementation_plan.md`, `task.md`, `walkthrough.md` (artifacts temporales del agente, NO se persisten en el repo)

### Estructura de Documentación (cómo se relacionan los docs)

```
docs/
├── Backlog.md                          ← Tareas pendientes (activas)
├── CHANGELOG.md                        ← Release notes formales por versión (git-cliff)
├── progreso/README.md                  ← Historial de tareas completadas + hitos
├── Investigaciones.md                  ← Índice de investigaciones
├── Investigaciones/01_*.md ... 05_*.md ← Research artifacts separados por tema
├── VantaDB-MPTS/                       ← Documentación estratégica en español
├── api/                                ← SDK/API docs en INGLÉS (fuente de verdad técnica)
├── architecture/                       ← Arquitectura en INGLÉS
└── operations/                         ← Operaciones/CI en INGLÉS
```

### Flujo de Archivos (cómo se relacionan)

1. **docs/Backlog.md** contiene tareas PENDIENTES con estado ❌ (o ⏸️ en icebox).
3. Cuando una tarea alcanza ✅, se MUeVE de `docs/Backlog.md` a `docs/progreso/README.md` (sección "Tareas Completadas (Migradas desde Backlog)" en tabla de formato `| \`ID\` | Descripción | Prioridad | Estado |`).
4. **docs/progreso/README.md** contiene:
   - Tareas completadas migradas (tabla)
   - Auditorías integrales detalladas
   - Progreso reciente (semanas/hitos importantes)
   - Infrastructure issues conocidos
5. **docs/CHANGELOG.md** contiene SOLO release notes por versión (cambios significativos que afectan a usuarios/developers) — NO cada tarea individual.
6. **docs/Investigaciones/** contiene research artifacts que no son tareas sino descubrimientos/análisis.
7. Backlog.md y progreso/README.md son MUTUAMENTE EXCLUYENTES — ninguna tarea aparece en ambos.

### Separación de Idiomas

- **Inglés (fuente de verdad técnica):** `docs/api/`, `docs/architecture/`, `docs/operations/`, `docs/QUICKSTART.md`
- **Español (estratégico/planificación):** `docs/VantaDB-MPTS/`, `docs/Backlog.md`, `docs/progreso/`, `docs/Investigaciones/`, `docs/CHANGELOG.md` (sección inferior)
- Los MPTS en español contienen cross-references a los docs técnicos en inglés (ej: `> **Referencia técnica en inglés:** \`docs/api/EMBEDDED_SDK.md\``)

El sistema por defecto sobrescribe los artifacts de planificación locales (`implementation_plan.md`, `task.md`, `walkthrough.md`) al iniciar nuevas actividades, lo cual causa pérdida de datos históricos. Para prevenir esto y mantener el repositorio limpio y estructurado sin subcarpetas innecesarias, DEBES cumplir estrictamente este protocolo de registro en el Historial Unificado de Progreso:

## Definition of Done (DoD) — Checklist Obligatorio

Antes de marcar una tarea como completada, TODOS estos puntos deben cumplirse:

- [ ] **Código**: La implementación compila (`cargo check`) y pasa tests (`cargo test`)
- [ ] **Docs API**: Si se modificó `src/sdk.rs`, verificar que `docs/api/EMBEDDED_SDK.md` refleje los cambios
- [ ] **Docs Python**: Si se modificó `vantadb-python/src/lib.rs`, verificar `docs/api/PYTHON_SDK.md`
- [ ] **Docs Config**: Si se modificó `src/config.rs` o `src/cli.rs`, verificar `docs/operations/CONFIGURATION.md`
- [ ] **Docs HTTP**: Si se modificó `src/cli_server.rs`, verificar `docs/api/HTTP_API.md`
- [ ] **Docs MCP**: Si se modificó `vantadb-mcp/`, verificar `docs/api/MCP.md`
- [ ] **WASM/TS**: Si se modificó `vantadb-wasm/` o `vantadb-ts/`, verificar sus READMEs
- [ ] **MPTS cross-ref**: Si se agregó una feature técnica nueva, verificar que los MPTS relevantes tengan cross-reference al doc en inglés
- [ ] **Validación**: Ejecutar `scripts/validate-docs-coverage.ps1` y verificar que no hayan nuevos gaps

## Principio Doc-Driven Development

Para features NUEVAS (Trigger 2), el orden es:
1. **Identificar** qué docs deben cambiar (`EMBEDDED_SDK.md`, `CONFIGURATION.md`, etc.)
2. **Definir la superficie** en los docs primero (structs, métodos, config fields, errores)
3. **Implementar** el código para cumplir lo documentado
4. **Validar** que el código y los docs coincidan

Esto asegura que la documentación nunca quede detrás del código.

## Trigger 1: Al concluir una actividad

Inmediatamente después de generar o actualizar el `walkthrough.md` como paso final de una implementación:

### Paso A: Documentation Impact Analysis
1. Recolecta la lista de archivos modificados (usa `git diff --name-only` o la información del plan).
2. Para cada archivo modificado, determina qué documentación le corresponde:
   | Archivo modificado | Documentación a verificar |
   |---|---|
   | `src/sdk.rs` | `docs/api/EMBEDDED_SDK.md` |
   | `src/config.rs` | `docs/operations/CONFIGURATION.md` |
   | `src/cli.rs`, `src/cli_handlers.rs` | `docs/operations/CONFIGURATION.md` (sección CLI) |
   | `src/error.rs` | `docs/api/EMBEDDED_SDK.md` (sección VantaError) |
   | `vantadb-python/src/lib.rs` | `docs/api/PYTHON_SDK.md` |
   | `src/cli_server.rs` | `docs/api/HTTP_API.md` |
   | `vantadb-mcp/src/` | `docs/api/MCP.md` |
   | `vantadb-wasm/src/lib.rs` | `vantadb-ts/README.md` |
3. Si algún doc asociado no fue actualizado, actualízalo ahora como parte de esta tarea.
4. Si la tarea agrega algo técnico nuevo (no solo bugfix interno), verifica que el MPTS correspondiente en `docs/VantaDB-MPTS/` tenga un cross-reference al doc técnico en inglés (formato: `> **Referencia técnica en inglés:** \`docs/api/...\``).

### Paso B: Extraer datos de la tarea completada
Identificador, nombre de la tarea, fecha, objetivo del `implementation_plan.md` y `walkthrough.md`.

### Paso C: Mover tarea a Historial Unificado
1. Lee `./docs/Backlog.md` — encuentra la fila de la tarea completada (✅).
2. Lee `./docs/progreso/README.md` — encuentra la sección "Tareas Completadas (Migradas desde Backlog)".
3. En `./docs/Backlog.md`:
   - **Elimina la fila** de la tarea completada de su tabla.
   - Si la subsección queda vacía, elimínala por completo o agrégala a la tabla general.
4. En `./docs/progreso/README.md`:
   - Agrega una entrada en la sección "Tareas Completadas (Migradas desde Backlog)" con formato:
     ```
     | \`ID\` | Descripción | Prioridad | ✅ |
     ```
   - Si la tarea fue un hito significativo, también agrega una entrada detallada en "Progreso Reciente" con checklist, archivos modificados y resultado.
   - Si la tarea fue un descubrimiento/investigación, considera moverla a `docs/Investigaciones/` en vez de o además de progreso.
5. **Guarda ambos archivos.**

### Paso D: Registrar en CHANGELOG (solo si es significativo)
NO agregues cada tarea individual a CHANGELOG.md. Solo si la tarea representa un cambio visible para usuarios/developers (nueva feature, breaking change, fix de bug público, nuevo comando CLI, etc.):
- Fecha en formato ISO 8601
- Versión del proyecto (`v<version>`)
- Nombre de la tarea como encabezado
- Archivos afectados (lista)
- Resultado obtenido

### Paso E: Validación automática
Ejecuta el script de validación:
```bash
pwsh scripts/validate-docs-coverage.ps1
```
Si encuentra métodos públicos no documentados, **detente y documéntalos antes de continuar**.

### Paso F: Commit
```bash
git add docs/Backlog.md docs/progreso/README.md
git commit -m "docs: move <tarea> completada a historial"
```

### Paso G: Notificar
Informa al usuario que Backlog.md y progreso/README.md han sido actualizados, y que la validación de cobertura pasó.

## Trigger 2: Al recibir una solicitud para una NUEVA actividad

Si el usuario te pide una nueva tarea que requiere un nuevo `implementation_plan.md`, ANTES de sobrescribir los archivos en tu memoria local de artifacts:

1. Abre `./docs/progreso/README.md` y verifica si la última tarea completada ya se encuentra registrada.
2. Si no se encuentra registrada, ejecuta inmediatamente **Trigger 1** usando los artifacts locales de la tarea anterior.
3. Solo cuando la actualización esté confirmada, procede a generar el nuevo Plan de Implementación.
4. Si la nueva tarea existe en `./docs/Backlog.md` como ❌, actualiza su estado a 🟡 En Progreso.

## Trigger 3: Mantenimiento periódico (revisión mensual / al inicio de una FASE nueva)

1. Revisa `docs/Backlog.md` para identificar tareas que lleven >30 días como ❌ sin actividad.
   - Si aplica, muévelas a la sección ⏸️ Icebox.
   - Si ya no son relevantes, muévelas a ❌ No Hacer.
2. Revisa `docs/progreso/README.md`:
   - Identifica secciones duplicadas o desactualizadas.
   - Verifica que las tablas en "Tareas Completadas (Migradas desde Backlog)" no tengan entradas duplicadas.
   - Verifica que los cross-links a CHANGELOG.md y Backlog.md sigan siendo válidos.
3. Revisa `docs/Investigaciones/`:
   - Identifica investigaciones huérfanas (sin referencia desde ningún otro doc).
   - Verifica que el índice en `docs/Investigaciones.md` refleje todos los archivos en `docs/Investigaciones/`.
4. Verifica que no haya tareas presentes en ambos archivos a la vez (Backlog.md y progreso/README.md).
5. Verifica que `docs/Backlog.md.new` o archivos temporales no hayan quedado huérfanos.
6. Verifica que los cross-references entre MPTS (español) y docs técnicos (inglés) sigan siendo válidos.
