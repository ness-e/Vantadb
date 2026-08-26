# Command & Flow System Audit — consolidación de comandos, planes y auditorías

**Fecha:** 2026-08-25T14:22:32-04:00
**Alcance:** `.opencode/commands/` (10), `.opencode/task-system/prompts/` (11), skills de orquestación/review (campaign-executor, unified-review, review-deep, progreso, backlog-executor), personas `vanta-*` (10).

## 1. Masa de instrucciones (el costo del problema)

| Bloque | Líneas |
|--------|-------:|
| Comandos (`commands/*.md`) | ~1.028 |
| Prompts task-system | ~1.777 |
| campaign-executor SKILL+RULES | 701 |
| unified-review SKILL | ~1.239 |
| Personas vanta-* | ~1.232 |
| **Total orquestación** | **≈6.000 líneas** |

Para ~10 intenciones reales del usuario (crear plan, ejecutar tarea(s), auditar, corregir/backlog, ship, status). La relación instrucción/acción es el síntoma que percibís como "sobrecarga de opciones".

## 2. Inventario: N formas de hacer lo mismo

### Ejecutar tareas — **8 caminos**
| # | Camino | Estado |
|---|--------|--------|
| 1 | `/pipeline task ID` → task.md + pipeline-full.md | ✅ canónico |
| 2 | `/pipeline run [plan]` → loop sub-agentes | ✅ canónico batch |
| 3 | `/pipeline pipeline` → "una tarea por iteración" | ❌ duplica #2 |
| 4 | `/pipeline ejecución\|mcp` → "paso a paso MCP" | ❌ tercera variante de ejecución |
| 5 | `/build` → incremental-implementation + TDD | ⚠️ sistema paralelo |
| 6 | `/build auto` → plan propio + aprobación única | ⚠️ ídem |
| 7 | skill `backlog-executor` (638L) | ❌ marcada SUPERSEDED, sigue instalada |
| 8 | `/loop-goal` + pipeline-full.md manual | ⚠️ escape no documentado |

**Bug estructural confirmado:** `/build` lee planes de `tasks/plan.md` y escribe `docs/last-build-state.json`; `/pipeline` usa `docs/plans/` + recitation MCP. El propio build.md admite el desajuste ("Path sync... Bridge"). Verificado: **`tasks/plan.md` y `last-build-state.json` NO existen** — el puente está muerto; los dos sistemas nunca intercambiaron estado.

### Crear planes — **5 caminos, 2 ubicaciones de salida**
`/pipeline plan` (→ `docs/plans/`) · `/spec` (→ `SPEC.md` raíz o `docs/SPEC.md`) · `writing-plans` · `planning-and-task-breakdown` · `/build auto` (→ `tasks/plan.md`). Tres formatos de salida distintos para el mismo artefacto.

### Auditoría/review — **4 sistemas superpuestos**
| Sistema | Modos | Relación |
|---------|-------|----------|
| `/audit` (audit.md, 196L) | quick/certify/review/full — **implementación propia de 9 fases con waves** | unified-review declara "/audit (legacy alias)" pero audit.md **NO invoca unified-review** salvo fases 7/8 → dos implementaciones divergentes del mismo "full audit" |
| skill `unified-review` (~1.239L) | quick/certify/review/full + profiles + scoring | la más completa; produce reportes en el MISMO directorio con el MISMO naming |
| skill `review-deep` (475L) | deep module review | duplica Phase 6 de /audit full; ni unified-review ni audit la referencian |
| Gates mecánicos | pre-commit hook · pre-push hook (`verify.ps1`, Regla 1) · `just verify/ci/certify` | correctos, pero se solapan conceptualmente con `/audit certify` |

Y `/ship` = fan-out audit+chaos+tuner con GO/NO-GO — cuarto gate pre-entrega que solapa con `/audit certify`. Hoy existen **4 puertas antes de push/release sin jerarquía documentada**: hook pre-commit → hook pre-push → `/audit certify` → `/ship`.

### Hallazgos → backlog — **3 flujos, 3 esquemas de ID**
- `findings.md` (canónico): **FIND-\***, nace en discovery — bien diseñado.
- `audit.md`: deriva hallazgos ≥ medium como **AUD-NN** a "Hallazgos pendientes".
- `unified-review` L11b: deriva como **REVIEW-NN**.
- `progreso` Trigger 4: sincroniza reportes↔backlog (audita huérfanos).
- Esquemas históricos vivos en Backlog: ERR-, DAUD-, AGT-, SEC-, PERF-, COV-… (cada investigación pasada creó el suyo).

## 3. Diagnóstico (causas, no síntomas)

1. **Cada campaña/investigación creó su propio entry point** en vez de extender el canónico → proliferación de modos y esquemas de ID.
2. **El patrón correcto ya existe y no se aplicó**: `question-gates.md` y `findings.md` usan "fuente única + prompts que referencian" (TSYS-H7). `audit.md` y `/build` son anteriores a ese patrón y nunca migraron.
3. **Falta una tabla única intención→comando**; AGENTS.md tiene un Entry Points parcial que no menciona /build ni resuelve ambigüedades.
4. **Nadie elimina**: backlog-executor superseded convive; review-deep flota; los modos redundantes de /pipeline acumulan.

## 4. Plan de consolidación propuesto (enfocado a soluciones)

### Fase A — Unificar ejecución (alto impacto, bajo riesgo)
- **A1.** `/pipeline`: reducir a 3 modos — `plan` / `task [ID]` / `run [plan]`. Eliminar `pipeline` y `ejecución|mcp` (son variantes de run/task; mapearlas como alias internos durante 1 mes y borrar).
- **A2.** Deprecar `/build`: convertirlo en alias fino de `/pipeline task` con las skills TDD+incremental (su valor real es la selección de skills, no otro orquestador). Borrar puente `tasks/plan.md` y `last-build-state.json`; estado único = recitation MCP + `docs/plans/`.
- **A3.** Eliminar skill `backlog-executor` (superseded por campaign-executor desde hace 2 generaciones).

### Fase B — Unificar auditoría (mayor ahorro)
- **B1.** `/audit` pasa a ser **router fino**: `quick/certify/review/full` → invocan `unified-review` con el modo correspondiente (+ profile vantadb fijo). audit.md baja de 196L a ~30L y desaparece la implementación paralela de 9 fases.
- **B2.** `review-deep` se referencia explícitamente como el motor de la fase deep-module de unified-review (hoy duplicado flotante).
- **B3.** Jerarquía de gates documentada en UN lugar (AGENTS.md §Regla 1):
  ```
  commit   → hook pre-commit (verify_changed)
  push     → hook pre-push (verify.ps1)          [= /audit quick mecánico]
  merge    → /audit certify (unified-review certify)
  release  → /ship (certify + nocturnal + fan-out GO/NO-GO + rollback plan)
  ```
  Cada nivel incluye al anterior; ningún comando redefine los checks.

### Fase C — Un solo flujo de hallazgos
- **C1.** `findings.md` queda como ÚNICO esquema: todo hallazgo derivado de cualquier auditoría/review nace como fila **FIND-NN** con `ref:` al reporte. Los reportes conservan su nombre/timestamp, pero el ticket en Backlog siempre es FIND-*.
- **C2.** Actualizar la sección "Output" de audit.md/unified-review y progreso Trigger 4 para derivar FIND-* (los AUD-NN/REVIEW-NN históricos existentes NO se renombran — solo se cierra la creación de nuevos esquemas).
- **C3.** Regla nueva en findings.md: prohibido crear prefijos nuevos por campaña (ERR/DAUD/AGT fueron el anti-patrón); investigación ≠ esquema de tickets.

### Fase D — Descubribilidad
- **D1.** Tabla única intención→comando en AGENTS.md ("quiero X → escribí Y"), reemplazando Entry Points parciales.
- **D2.** Regla de mantenimiento: todo comando nuevo debe declarar qué comando existente reemplaza o por qué ninguno cubre la necesidad (mata la proliferación en la fuente).

## 5. Ahorro estimado

| Métrica | Hoy | Tras consolidación |
|---------|----|--------------------|
| Comandos | 10 | 8 (−/build, −backlog-executor) |
| Modos de /pipeline | 6 | 3 |
| Implementaciones de "auditoría completa" | 2 | 1 (unified-review) |
| Esquemas de ID de hallazgos activos | 3 | 1 (FIND-*) |
| Líneas de orquestación | ≈6.000 | ≈4.800 (−20%) |

## 6. Qué NO tocar (funciona bien)

- Patrón "fuente canónica + referencias" (question-gates.md, findings.md) — es el modelo a extender.
- MCP campaign (task system, recitation, budgets) — único estado real del pipeline.
- Personas vanta-* leaf/orchestrator con tabla de tools — buena separación.
- unified-review profiles/scoring — es la implementación madura que debe ganar.

## Veredicto
El sistema no le falta funcionalidad — le sobran **vías**. La consolidación propuesta no agrega nada nuevo: elimina 2 comandos, 3 modos, 1 implementación de auditoría y 2 esquemas de ID, y documenta la pirámide de gates.
