# VantaDB — Reglas del Proyecto (`.opencode/rules/`)

> **Propósito:** normativa prescriptiva (must / must-not / por qué) por **área del sistema**, para que cualquier agente — y el humano que lo administra — aplique las mismas reglas al crear, editar, modificar o mejorar código del proyecto.
>
> **Naturaleza:** son **reglas duras**, no material de referencia. No duplican a `references/` (material de consulta) ni a `skills/` (procedimientos).

---

## Cómo las leen los agentes (lazy-loading)

Los agentes **no cargan esta carpeta completa**. La carga es *bajo demanda*, vía instrucción en `.opencode/AGENTS.md`:

> Al crear/editar código, leer el archivo de reglas del área que se toca **antes** de modificar. No cargar reglas de áreas no relacionadas.

**Regla de carga:**
- Identificar el área que se toca (ver tabla de índice abajo).
- Leer SOLO el archivo de esa área con Read tool.
- El contenido cargado es **obligatorio**: no se puede saltar ni relajar una regla.

---

## Índice de archivos

| # | Archivo | Área cubierta (scope) | Relacionado con |
|---|---------|----------------------|-----------------|
| 1 | `core-engine.md` | Núcleo del motor `src/` (node, engine, storage facade, config, error) | INV-002, DRV-119 |
| 2 | `durability.md` | WAL, storage/engine, backends, vfile, gc, lsm, schema, migration | DRV-014, DRV-133 |
| 3 | `indexes.md` | Índices vectoriales (`index/`), text_index, tokenizer | OLD-004, VFY-004 |
| 4 | `concurrency-async.md` | Async/tokio, locks, atomics, wal_sharded | **INV-003** |
| 5 | `query-dsl.md` | Parser + planner + executor (IQL) | DRV-002 |
| 6 | `api-contract.md` | API pública `sdk/`, VantaError/node/config, semver, compat | VFY-002 |
| 7 | `python-bindings.md` | `vantadb-python` (PyO3), providers | DRV-016 |
| 8 | `server-mcp.md` | `vantadb-server` (HTTP) + `vantadb-mcp` (stdio) | — |
| 9 | `js-ecosystem.md` | `vantadb-wasm` + `vantadb-ts` + `vantadb-node` | — |
| 10 | `frontend-web.md` | `web/` (Next.js, Tailwind, motion, i18n) | — |
| 11 | `release-ci.md` | Release, versionado, CI, changelog, publish | INV-017, REGLA 7 |
| 12 | `memory-budget.md` | Presupuesto de memoria: RAM vs disco, límite RSS, back-pressure antes de OOM | **FND-01** |

---

## Reglas para las reglas (cómo añadir / modificar)

> Estas reglas aseguran que la carpeta mantenga formato y estructura consistente. **Se aplican a quien edite los archivos de reglas: agentes y humanos.**

### R1 — Un área por archivo, sin solapamiento

- Cada archivo cubre **una** área de responsabilidad.
- El **scope** se declara en la cabecera (`> **Scope:**`) y no debe solaparse con otro archivo.
- Si una regla aplica a dos áreas: se pone en **una sola**, y en la otra se deja una referencia cruzada (`→ ver durability.md`).
- Antes de crear un archivo nuevo: verificar en el índice que no existe un archivo que ya cubra esa área.

### R2 — Formato obligatorio de cabecera

Todo archivo de reglas EMPIEZA con:

```markdown
# <ÁREA> — Reglas

> **Scope:** <rutas/módulos exactos que cubre>
> **No tocar aquí:** <áreas que pertenecen a otro archivo — para prevenir duplicación>
> **Status:** 🟢 Vigente | 🟡 En revisión | 🔴 Obsoleta
> **Fuentes:** <IDs de investigaciones/ADR que sustentan las reglas, si aplica>

## Reglas

### <NÚMERO> — <Título corto e imperativo>

- **Must:** ...
- **Must not:** ...
- **Por qué:** <razón técnica en 1-2 líneas>

<!-- Referencias cruzadas: → ver <archivo>.md -->
```

### R3 — Estilo de redacción

- **Imperativo y específico**: "Debes envolver X en `spawn_blocking`", no "sería bueno considerar..."
- Cada regla distingue **Must / Must not / Por qué**. El *por qué* es obligatorio: una regla sin razón se ignora.
- Números de línea: si una regla referencia una línea, incluir también el símbolo/función (las líneas cambian).
- Idiomas: mismas reglas que el repo — técnico en **inglés**, notas de contexto pueden ser en español. El repo usa inglés como fuente de verdad.

### R4 — Ciclo de vida

| Acción | Procedimiento |
|--------|---------------|
| **Añadir regla** | Añadir al final de la sección de su archivo, con número secuencial y `Status: 🟡 En revisión`. Aprobada por el humano → `🟢 Vigente`. |
| **Modificar regla** | Editar en sitio. NO borrar el historial sin antes anotar el cambio en una línea `<!-- Changed YYYY-MM-DD: motivo -->`. |
| **Obsoletar** | Marcar `Status: 🔴 Obsoleta` con motivo, NO borrar el archivo (conserva contexto). El humano decide el borrado. |
| **Archivo nuevo** | Verificar R1 (no existe cobertura), añadir al índice de este README, crear con cabecera R2 vacía de reglas, y registrar en `.opencode/AGENTS.md` si aplica un nuevo hook de carga. |

### R5 — No duplicar con el resto del sistema

| Si la info ya está en... | Hacer |
|--------------------------|-------|
| `.opencode/references/` | Referenciar, no copiar |
| `.opencode/skills/` | Referenciar, no copiar el procedimiento |
| `.opencode/AGENTS.md` | No repetir reglas globales — solo remitir a la sección |
| `docs/architecture/adr/` | Citar el ADR como fuente, no re-escribirlo |

### R6 — Validación antes de merge

Todo cambio en `rules/` debe:
1. Mantener el formato de cabecera (R2).
2. No solapar scope con otro archivo (R1).
3. Tener `Status` correcto (R4).
4. Actualizar el índice del README si cambió el inventario.

---

*Última actualización: 2026-08-04*
