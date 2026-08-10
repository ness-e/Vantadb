> **ACTIVE INSTRUCTION — Create Plan from Backlog**
> Cargado por `commands/pipeline.md` (modo PLAN).
> Path resolution: skills por nombre → `.opencode/skills/<nombre>/`
> Aplicar triage gate (✅ DO / 🟡 DEFER / ❌ SKIP / 🔴 BLOQUEADO) a cada tarea.
> Crear `docs/plans/<FECHA>-<nombre>.md` solo con tareas ✅ DO.
> Al finalizar: mostrar comando recomendado (`/pipeline run` o `/pipeline task <ID>`).

Cargá las skills brainstorming, writing-plans, idea-refine, progreso, ponytail (full).

Backlog: {{BACKLOG_PATH}}
Si no se especificó ruta, usá `docs/Backlog.md`.

## INSTRUCCIONES — CREAR PLAN DE CAMPAÑA

Aplicá el **Triaje gate** del campaign-executor a CADA tarea en el backlog.
Resultados posibles: ✅ DO, 🟡 DEFER, ❌ SKIP, 🔴 BLOQUEADO.

**ANTES del triaje** ejecutá el **Paso 0 — Verificación de Realidad** (abajo).
Sin él el gate decide sobre texto no verificado del backlog, no sobre el código real.

### Paso 0 — Verificación de Realidad (obligatorio, por tarea)

> **Propósito:** verificar que la tarea es **real y aplicable** contra el código actual:
> los archivos/símbolos que menciona existen, el comportamiento descrito persiste
> (no está ya implementado), y el work propuesto no es stale. Esto evita que el
> harness se coma una tarea obsoleta y descubra el problema recién en VERIFY.

**Skills que aplican a este paso (cargar las que matcheen — no todas):**

| Situación de la tarea | Skill a cargar | Por qué |
|---|---|---|
| Bug reportado (crash, panic, UAF, comportamiento roto) | `systematic-debugging` | Root-cause first: confirma que el bug existe con repro, no solo por descripción |
| Tarea toca API pública / bindings / breaking changes | `source-driven-development` | Grounding: verifica que los símbolos que se van a tocar existen y su comportamiento documentado es el real |
| Tarea ambigua o de alto riesgo | `doubt-driven-development` | Adversarial review: no confía en la premisa del backlog |
| Cualquiera con código Rust | `ponytail` (siempre activo) | Escalera YAGNI antes de incluir una tarea al plan |
| Al migrar/completar algo | `progreso` | Evita duplicar tareas ya migradas a progreso |

**Pasos por tarea (mientras mayor el esfuerzo/ambiguïdad, más exhaustivo):**

1. **Extrae referencias** del texto de la tarea:
   - Rutas de archivos (ej: `src/storage/vfile.rs`, `vantadb-python/src/convert.rs`)
   - Símbolos (funciones, structs, tests, CLIs, endpoints)
   - Features Cargo / flags de build / API pública
2. **Verifica en el código real** con `codegraph_explore "símbolos o archivos"`:
   - ¿Existe el archivo/símbolo? (si no existe → tarea stale o ruta renombrada)
   - ¿El comportamiento ya está implementado? (una tarea "fix X" con X ya arreglado → STALE)
   - ¿Qué llama a esto y que afecta? (blast radius — también sirve para el task file posterior)
   - ¿Docs/API referenciadas coinciden con el código actual?
3. **Clasifica el resultado del gate** según la verificación:

| Evidencia real | Gate |
|---|---|
| Referencias existen + gap de comportamiento real + cambio acotado | ✅ DO |
| Referencias existen pero gap ambiguo / esfuerzo alto vs impacto | 🟡 DEFER |
| Referencias NO existen, comportamiento ya implementado, o tarea completada en otro plan | ❌ SKIP |
| Depende de tarea no lista o bloqueada por otra | 🔴 BLOQUEADO |
| No se puede verificar sin investigación (bug sin repro, API externa) | registra en `Notas` y aplica DO si impacto justifica |

4. **Escribe la evidencia en el plan file** — la verificación es parte del contract:

   ```
   - **Verificación real:** `codegraph_explore` → `src/vector/hnsw.rs` existe, gap real en `search_knn` (línea 412), callers: `engine.rs`, `sdk/search.rs`
   - **Gate Justificación:** bug persistente en hot path, 2 callers afectados, fix acotado a 1 archivo
   ```

   (Si el símbolo NO existe, anota también: `mo existe → la tarea menciona código renombrado` como evidencia del SKIP)

### Pre-mortem — ¿por qué fracasaría esta tarea? (obligatorio)

> **Propósito:** antes de comprometer esfuerzo en una tarea ✅ DO, listar los
> modos de fallo más probables. Es la técnica cognitiva de mayor ROI: expone
> rabbit holes antes de que el approach elegido se convierta en un compromiso
> fuerte (y caro de revertir).

Para cada tarea candidata a DO, escribir en el plan file:

```
- **Fallo probable 1:** ... (ej: el fix asume un hot path que no es el real)
- **Fallo probable 2:** ... (ej: la feature depende de una API externa no verificable offline)
- **Fallo probable 3:** ... (ej: el cambio rompe un caller no mapeado en el blast radius)
```

Si el pre-mortem revela un modo de fallo de alta probabilidad y alto impacto,
bajar el gate (🟡 DEFER) o registrarlo como riesgo vivo en el **Risk Register**.

### Stop conditions / circuit breaker (appetite)

Criterios explícitos de CANCELACIÓN/abort definidos ANTES de arrancar la tarea:

| Stop condition | Trigger | Acción |
|---|---|---|
| Appetite excedido | tiempo invertido > estimado ×2 | abortar → re-triaje como 🟡 DEFER |
| Rabbit hole | N iteraciones sin progreso verificable (contrato sin green) | abortar → re-planear approach |
| Presupuesto agotado | budget de tool calls / tiempo de campaña agotado | abortar → registrar en `Notas` |
| Premisa invalidada | evidencia nueva contradice el Paso 0 | abortar → re-evaluar el gate |

Al dispararse una stop condition, la tarea pasa a ⬛ CANCELADO (no ❌ SKIP)
y se documenta el motivo en el plan file. Las stop conditions se escriben en
el plan file junto al contrato de cada tarea.

### Reglas del gate (aplicar DESPUÉS del Paso 0)

1. Bug ya inexistente o feature ya implementada (**verificado**) → SKIP
2. Cosmético sin queja de usuario → DEFER
3. Esfuerzo >> impacto → DEFER o SKIP
4. Dependencia no lista → BLOQUEADO
5. Prioridad original es sugerencia, no orden
6. La verificación del Paso 0 es la base del gate — no re-evaluar por texto del backlog si codegraph contradice

### Para cada tarea ✅ DO

Registrá en el plan file con:

- **ID** único (ej: DRV-068)
- **Descripción** corta (máx 80 chars)
- **Esfuerzo:** 🟢 1h | 🟡 1d | 🔴 2-3d
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Archivos clave:** paths relevantes
- **Verificación real:** evidencia del Paso 0 (símbolos existentes, gap confirmado, callers)
- **Gate Justificación:** por qué pasó el gate
- **Contrato:** condición verificable por comando mecánico
- **Pre-mortem:** 2-3 modos de fallo probables (ver Paso 0)
- **Stop conditions:** criterios de cancelación (ver Paso 0)
- **Risk Register:** máx 5-8 riesgos vivos (Prob×Impacto, respuesta, trigger/due)
- **Estado inicial:** ⬜ PENDING
- **Task file:** `skills/campaign-executor/tasks/ID.md` (aún no existe — se creará bajo demanda)

### Auto-detección de formato

Si el backlog tiene estructura conocida (TIER, Estado ❌, etc.) → parseá determinísticamente.
Si no reconoce el formato → el agente interpreta con LLM para extraer tareas.

### Formato del plan file

```markdown
# Plan de Ejecución: [Nombre]

> **Inicio:** YYYY-MM-DD
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** [ruta al backlog]

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | N |
| 🟡 DEFER | N |
| ❌ SKIP | N |
| 🔴 BLOQUEADO | N |

## Tasks

### Task 1: ID — Descripción

- **Esfuerzo:** 🟢 | 🟡 | 🔴
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Archivos clave:** `path/to/file.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — símbolo X existe en `src/...`, gap confirmado; o 🟡 VERIFICAR — sin referencias, confirmar en DISCOVERY; o ❌ STALE — no existe/reimplementada
- **Gate Justificación:** por qué pasó
- **Gate Result:** ✅ DO
- **Contrato:** "comando mecánico para verificar"
- **Task file:** `skills/campaign-executor/tasks/ID.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:** (máx 5-8 riesgos vivos — parte del contract, no opcional)
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | ej: fix toca el hot path de `search_knn` | ej: pruebas acotadas a callers, plan de rollback | ej: 2 iteraciones sin green en VERIFY |
  | — | — | — | — |

  > Al materializarse un riesgo: decidir (mitigar/aceptar) y registrar la decisión
  > vía Regla 5 (`campaign_memory_write(file="decisions", entry="...")`) o ADR.

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 2: ...
```

### Al finalizar

Mostrá el comando exacto para ejecutar:

```
/pipeline run -PlanFile docs/plans/YYYY-MM-DD-<nombre>.md
```
