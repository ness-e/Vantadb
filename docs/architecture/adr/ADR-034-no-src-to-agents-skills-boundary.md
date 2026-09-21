---
title: "ADR-034: No existe boundary `src → .agents/skills` — resolución de falso positivo del codegraph por path-homonymy"
type: adr
status: accepted-pending-owner-review
tags: [vantadb, architecture, adr, codegraph, boundaries, false-positive, path-homonymy, find-42]
created: 2026-08-30
last_reviewed: 2026-08-30
related: [FIND-42, FIND-41, FIND-45, codegraph-20260827, docs/plans/2026-08-29-full-backlog-parallel.md]
owner_articulates: pending
---

> **⚠️ DRAFT — requiere articulación del owner (Regla 5, AGENTS.md)**
>
> Este ADR fue redactado por IA durante la investigación FIND-42
> (W25-3 del plan `2026-08-29-full-backlog-parallel`). El **trade-off
> central — ¿el codegraph necesita un pre-procesador de paths para
> distinguir `src/...` de `.agents/...` por prefijo de raíz, o el
> reporte actual es aceptable con anotación explícita? — debe ser
> articulado por el owner humano**. La IA aporta los datos y la
> evidencia mecánica (métricas, queries Cypher, grep); el humano decide
> si acepta el trade-off y prioriza (o no) la mejora P3-XXX.
>
> Hasta que el owner articule: status `accepted-pending-owner-review`,
> `owner_articulates: pending`. Las refs internas y los hallazgos
> técnicos son válidos independientemente — el contrato de FIND-42 ya
> pasa (`Select-String ... | Count == 0`) sin necesidad de fix.

# ADR-034: No existe boundary `src → .agents/skills` — resolución de falso positivo del codegraph por path-homonymy

## Context

Durante la ejecución del plan paralelo `2026-08-29-full-backlog-parallel`
(W25-3, FIND-42), el task file reportaba:

> **Archivos clave:** `src/` → `.agents/skills/` (173 llamadas)
> **Verificación real:** core llama a skills/agentes (impeccable) —
> inversión dependencia semántica
> **Contrato:** `Select-String -Path "src/**/*.rs" -Pattern
> "\.agents.skills" | Measure-Object Count` ==0 (removido) OR ADR que
> documenta como intencional

La métrica "173 llamadas" venía del reporte `codegraph-20260827 Fase 1`.
La descripción "core llama a skills/agentes (impeccable)" sugería una
**inversión de dependencia semántica** (el core dependiendo de workflows
del agente), lo cual habría sido un anti-pattern arquitectónico severo
(Regla 5 AGENTS.md: el core no debe depender del tooling del agente).

### Investigación

La verificación mecánica reveló que la "inversión de dependencia" **no
existe**. Es un **falso positivo** del codegraph causado por
**path-homonymy** (colisión de nombres en la última componente del
path). Cuatro carpetas distintas comparten el nombre `skills`:

| Carpeta | Naturaleza | Aristas desde `src/`? |
|---|---|---|
| `src/skills/` | **Módulo core** del engine (`SkillStore`, persistencia de skills como memoria de agentes — feature D19 plan vanta-memory, commit previo a codegraph-20260827) | **Sí — 184 edges** (es un sub-módulo interno del core; relación `CONTAINS_FOLDER` + calls/references entre archivos Rust del propio core) |
| `skills/` (raíz del repo) | (vacío / placeholder histórico, no parte de grafo Rust) | No |
| `.opencode/skills/` | Skills del agente OpenCode (workflows de orquestación; markdown, no Rust — **fuera del grafo codegraph**) | **No** — código fuente Rust no se compila desde acá |
| `.agents/skills/` | Skills del proyecto (162 dirs de workflows; markdown, no Rust — **fuera del grafo codegraph**) | **No** — código fuente Rust no se compila desde acá |

El codegraph-20260827 y el `codebase-memory-mcp` (índice `moderate`,
47.7K nodos, 157.7K edges) **colapsaron las 4 carpetas en un único nodo
"skills"** al coincidir en la última componente del path. El resultado
fue una métrica agregada que **no representa inversión de dependencia**
— representa acoplamiento interno del core con su propio sub-módulo
`SkillStore`.

### Evidencia mecánica

Tres verificaciones independientes, todas con resultado `0` (sin inversión):

**1. Verificación textual PowerShell (contrato del task):**
```powershell
Select-String -Path "src/**/*.rs" -Pattern "\.agents\.skills" |
  Measure-Object | Select-Object Count
# → Count = 0 ✅
```

**2. Grep cruzado literal (sanity check):**
```bash
grep -r --include='*.rs' '\.agents/skills|\.opencode/skills' src/
# → No files found ✅
```

**3. Cypher en codebase-memory-mcp (índice `moderate`):**
```cypher
MATCH (src:Folder {file_path: 'src'})-[r]->(target)
WHERE target.file_path CONTAINS '.agents'
   OR target.file_path CONTAINS '.opencode/sk'
   OR target.file_path STARTS WITH 'agents/skills'
   OR target.file_path STARTS WITH 'opencode/skills'
RETURN type(r), target.file_path, count(*) AS edges
// → 0 rows ✅
```

**4. Codegraph `get_architecture aspects=['boundaries']`:**
```
boundaries: 10  (cols: from to calls)
  ...
  src skills 184   ← esta es la métrica agregada
  ...
```
La métrica `src skills 184` **existe**, pero al inspeccionar
`query_graph` sobre `src/skills`:
```cypher
MATCH (src:Folder {file_path:'src'})-[r]->(t:Folder {file_path:'src/skills'})
RETURN type(r), count(r)
// → CONTAINS_FOLDER, 1 edge
```
confirmamos que la métrica viene de **acoplamiento interno** del core
con su sub-módulo, **no** de una inversión hacia skills del agente.

## Invariantes

1. **El core Rust (`src/`) NO importa ni referencia `.agents/skills/` ni
   `.opencode/skills/`.** Las skills del agente son documentación de
   workflow (markdown), no código compilable. Cualquier import Rust
   que apunte a `.agents/skills/` debe ser rechazado en review
   (anti-pattern arquitectónico: el core no debe depender del tooling
   del agente — Hyrum's Law, skill `api-and-interface-design`).
2. **El core Rust (`src/`) SÍ contiene un módulo interno `src/skills/`
   (`SkillStore`).** Es un módulo de dominio (persistencia de skills
   como memoria de agentes), parte legítima del core — NO una fuga
   hacia las skills del agente OpenCode.
3. **Las métricas agregadas del codegraph que mezclan `src/skills`
   con `.agents/skills` requieren anotación explícita** en futuros
   reportes hasta que el codegraph implemente path-prefix-disambiguation
   (deuda P3 — ver `Consequences`).
4. **`grep`-textual sigue siendo el contrato autoritativo** para este
   tipo de auditoría: ignora path-homonymy porque matchea por path
   completo, no por última componente.

## Decision

FIND-42 se resuelve **documentalmente, sin fix mecánico**:

1. **No hay código a eliminar.** El contrato `Select-String ... | Count == 0`
   ya pasa (`0` hits). La "violación" reportada por codegraph-20260827
   no existe como código real.
2. **Se escribe este ADR-034** para explicar (a) por qué el reporte
   reportó 173/184 edges, (b) por qué esos edges NO representan
   inversión de dependencia, (c) la métrica correcta a usar en
   futuros análisis de boundary.
3. **FIND-42 se marca `✅ COMPLETED`** en el plan file
   `2026-08-29-full-backlog-parallel.md`.
4. **El task file `FIND-42.md`** documenta los 4 steps de verificación
   (textual + grep + codegraph + Cypher) y referencia este ADR.
5. **Deuda abierta P3-XXX (registrada en `Consequences`):** el codegraph
   necesita path-prefix-disambiguation. Owners deben priorizar.

### Alternativas consideradas

#### A1 — Asumir la métrica como bug y eliminar `src/skills/`

Tratar el reporte del codegraph como verdad y eliminar el módulo
`SkillStore` para llegar a `Count = 0` también para `src/skills`.

- **Pro:** fuerza simplicidad.
- **Contra:** **rompe feature shipped** — `SkillStore` es la
  implementación del feature D19 (multi-version skill memory) que ya
  está en uso (commits previos, tests en `src/skills/tests.rs`).
  Eliminarlo significa revertir feature shipped, romper tests, y
  romper consumidores que dependen del endpoint REST
  `/api/v2/skills/*`.
- **Rechazado** — costo >> beneficio; el problema está en el
  reportador, no en el código.

#### A2 — Renombrar `src/skills/` a `src/skill_store/`

Renombrar el módulo core para eliminar la homonimia.

- **Pro:** elimina la ambigüedad que origina el falso positivo;
  métrica del codegraph queda limpia.
- **Contra:** cambio cosmético invasivo (rename de módulo + todos los
  imports). No resuelve la causa raíz (el codegraph debería
  pre-procesar paths por prefijo de raíz, no por última componente).
  Renombrar hoy y volver a chocar mañana cuando se agregue otra
  carpeta `skills/` en cualquier otro subdirectorio.
- **Rechazado** — parche cosmético; el fix real es en el codegraph.

#### A3 — Fix mecánico para llegar a `Count == 0` sin ADR

Ejecutar el contrato literalmente: encontrar y eliminar cualquier
referencia a `.agents/skills` en `src/`. Como no hay ninguna (verificado
3 veces), esto sería un no-op.

- **Pro:** técnicamente cumple el contrato textual.
- **Contra:** **falsa sensación de progreso** — el task se marca
  completo sin haber hecho nada. El ADR es el que captura el
  aprendizaje (path-homonymy → reportadores deben mejorar). Sin
  ADR, el próximo plan que vea "src → skills 173" repite el
  diagnóstico.
- **Rechazado** — el ADR es el deliverable de valor.

#### A4 — ADR + fix de codegraph (P3 inmediato)

Implementar path-prefix-disambiguation en codegraph durante FIND-42.

- **Pro:** elimina la clase entera de falsos positivos, no solo este.
- **Contra:** **scope creep**. FIND-42 es de docs (apetite 1d);
  modificar codegraph requiere (a) entender el parser tree-sitter, (b)
  actualizar el schema del grafo, (c) re-indexar, (d) validar contra
  otros 5 reportes donde esta métrica aparece (FIND-24, FIND-41,
  FIND-45, otros). Out of scope.
- **Rechazado** para FIND-42; registrado como **deuda P3** para
  priorización por owner.

## Consequences

### Positivas

1. **Falsa alarma desactivada.** El próximo plan/auditoría que vea
   "src → skills 173" puede leer este ADR y descartar el falso
   positivo en minutos, no en horas de grep.
2. **Métrica correcta documentada.** Cualquier futura investigación
   puede usar `codebase-memory-mcp query_graph` con `WHERE file_path
   CONTAINS '.agents'` como filtro (paso 3 de la sección Evidencia)
   y obtener la métrica limpia.
3. **Patrón transferible.** El método de verificación (4 capas:
   contractual textual → grep literal → codegraph → Cypher) se
   puede aplicar a futuros hallazgos reportados por codegraph cuando
   parezca haber boundary violation.
4. **Cero deuda técnica nueva** (Regla 6): no se introduce código,
   abstracción, ni fix.

### Negativas / Riesgos

1. **Codegraph sigue reportando la métrica agregada** hasta que se
   implemente path-prefix-disambiguation (deuda P3). Mientras tanto,
   cada nuevo reporte con "src → skills N" debe pasar por este ADR
   o re-verificar con Cypher. Riesgo de fatiga si la métrica
   reaparece mucho.
2. **Otros reports basados en codegraph-20260827 Fase 1** pueden
   contener el mismo falso positivo. FIND-24, FIND-41 y FIND-45
   citan métricas derivadas — owners deben leer ADR-034 antes de
   actuar sobre esas.
3. **El ADR queda `accepted-pending-owner-review`** hasta que el
   owner humano articule el trade-off (Regla 5). Mientras tanto,
   las refs técnicas son válidas pero el status no es `accepted`
   definitivo.

### Deuda técnica (Regla 6)

- **Saldo neto: neutral**.
- **Quita:** 0 deuda (no tocamos código core).
- **Agrega:** 0 deuda nueva en código core.
- **Registra (no-fix-aún):**
  - **P3-XXX: codegraph path-prefix-disambiguation.**
    El codegraph debería distinguir carpetas por **prefijo de raíz**
    (`.`, `src`, `tests`, `benches`, `desktop`, etc.) y no por
    **última componente del path**. Mientras esa mejora no esté
    implementada, métricas como "src → skills 184" o "tests →
    Justfile 118" mezclan boundaries reales con homonimias.
    Estimación: 🟡 2-4 hr (modificar el parser tree-sitter del
    indexador + actualizar schema del grafo + re-index + validar
    contra 5 reportes existentes).

## References

- **Task:** `.opencode/skills/campaign-executor/tasks/FIND-42.md`
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` §W25-3
- **Reports potencialmente afectados** (los owners deben re-validar
  contra este ADR antes de actuar):
  - `FIND-24` (W25-1): list fan-out (puede depender de métricas
    agregadas similares)
  - `FIND-41` (W25-2): clusters Leiden (la métrica `src skills 184`
    puede haber contaminado los clusters)
  - `FIND-45` (marcado `DEFER` en plan file): "src→skills violation
    | 🟡 | Duplicate de FIND-42 - ya cubierto" — este ADR confirma
    la nota del plan y permite SKIP/CLOSE definitivo de FIND-45.
- **CodeGraph MCP:** `codegraph_explore` (verificación de boundary),
  `codegraph.db` (índice; artefacto local)
- **codebase-memory-mcp:** `get_architecture aspects=['boundaries']`,
  `query_graph` Cypher (verificación de aristas específicas)
- **Skill `documentation-and-adrs`** (Regla 5 — ADR como memoria
  decisiones arquitectónicas; formato canónico)
- **Skill `api-and-interface-design`** (Hyrum's Law — el core no debe
  depender del tooling del agente)
- **AGENTS.md Regla 0** (análisis de impacto antes de modificar;
  este ADR es el resultado de Regla 0 aplicada a la "inversión
  reportada")
- **Plantilla:** `docs/_templates/adr.md`
- **Último ADR existente:** ADR-033 (prov-04, contrato providers)